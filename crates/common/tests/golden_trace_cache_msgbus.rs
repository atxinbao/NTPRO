// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use std::{
    any::Any,
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use nautilus_common::{
    cache::Cache,
    msgbus::{
        BusTap, Endpoint, MStr, MessageBus, Topic, TypedHandler, clear_bus_tap, get_message_bus,
        publish_quote, set_bus_tap, set_message_bus, subscribe_quotes,
    },
};
use nautilus_core::UUID4;
use nautilus_model::{
    data::QuoteTick,
    identifiers::{InstrumentId, TraderId},
    instruments::{Instrument, InstrumentAny, stubs::crypto_perpetual_ethusdt},
    types::{Price, Quantity},
};
use serde_json::{Value, json};

const CASE_ID: &str = "cache_msgbus.quote_cache_publish.001";

#[test]
fn rust_common_cache_msgbus_replays_quote_ordering_golden_trace() {
    let case = load_case(CASE_ID);
    let input_event = event_by_type(&case, "input", "cache_msgbus.quote_input");
    let expected = case
        .get("expected")
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .expect("expected.events must be an array");

    let actual = run_cache_msgbus_replay(input_event);

    assert_eq!(
        actual, *expected,
        "Rust cache/message-bus lifecycle must match the golden trace"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/common")
}

fn load_case(case_id: &str) -> Value {
    let trace = repository_root().join("tests/golden/cache_msgbus_schema.jsonl");
    fs::read_to_string(&trace)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", trace.display()))
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(|line| {
            serde_json::from_str::<Value>(line)
                .unwrap_or_else(|err| panic!("{} invalid JSON: {err}", trace.display()))
        })
        .find(|row| row.get("case_id").and_then(Value::as_str) == Some(case_id))
        .unwrap_or_else(|| panic!("case {case_id} not found in {}", trace.display()))
}

fn event_by_type<'a>(case: &'a Value, section: &str, event_type: &str) -> &'a Value {
    case.get(section)
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{section}.events must be an array"))
        .iter()
        .find(|event| event.get("event_type").and_then(Value::as_str) == Some(event_type))
        .unwrap_or_else(|| panic!("{section} event {event_type} not found"))
}

fn run_cache_msgbus_replay(input_event: &Value) -> Vec<Value> {
    let payload = input_event
        .get("payload")
        .expect("input event payload is required");
    let topic = string_field(payload, "topic").to_string();
    let quote = quote_from_event(input_event);
    let instrument_id = quote.instrument_id;
    let venue = string_field(input_event, "venue").to_string();
    let observed = Rc::new(RefCell::new(Vec::new()));

    set_message_bus(Rc::new(RefCell::new(MessageBus::new(
        TraderId::from("TRADER-RCORE-009"),
        UUID4::new(),
        Some("RCORE-009-cache-msgbus".to_string()),
        None,
    ))));
    clear_bus_tap();

    let mut cache = Cache::default();
    cache.add_quote(quote).expect("quote should cache");
    let cached = cache
        .quote(&instrument_id)
        .expect("quote should be available after add_quote");
    observed.borrow_mut().push(cache_quote_event(
        "cache.quote.stored",
        0,
        instrument_id,
        &venue,
        cached,
        true,
    ));

    set_bus_tap(Rc::new(RecordingTap {
        events: observed.clone(),
        instrument_id,
        venue: venue.clone(),
    }));

    let subscriber_events = observed.clone();
    let subscriber_topic = topic.clone();
    let subscriber_venue = venue.clone();
    let handler = TypedHandler::from(move |quote: &QuoteTick| {
        subscriber_events.borrow_mut().push(msgbus_quote_event(
            "msgbus.publish.subscriber",
            2,
            quote.instrument_id,
            &subscriber_venue,
            &subscriber_topic,
            quote,
        ));
    });
    subscribe_quotes(topic.as_str().into(), handler, None);
    publish_quote(topic.as_str().into(), &quote);

    clear_bus_tap();
    cache.dispose();
    observed.borrow_mut().push(cache_disposed_event(
        instrument_id,
        &venue,
        cache.quote(&instrument_id).is_some(),
    ));

    get_message_bus().borrow_mut().dispose();
    observed.borrow_mut().push(msgbus_disposed_event(
        instrument_id,
        &venue,
        get_message_bus().borrow().pub_count(),
    ));

    observed.borrow().clone()
}

struct RecordingTap {
    events: Rc<RefCell<Vec<Value>>>,
    instrument_id: InstrumentId,
    venue: String,
}

impl BusTap for RecordingTap {
    fn on_publish(&self, topic: MStr<Topic>, message: &dyn Any) {
        if let Some(quote) = message.downcast_ref::<QuoteTick>() {
            self.events.borrow_mut().push(msgbus_quote_event(
                "msgbus.publish.tap",
                1,
                self.instrument_id,
                &self.venue,
                topic.as_ref(),
                quote,
            ));
        }
    }

    fn on_send(&self, _endpoint: MStr<Endpoint>, _message: &dyn Any) {}
}

fn quote_from_event(event: &Value) -> QuoteTick {
    let instrument = InstrumentAny::CryptoPerpetual(crypto_perpetual_ethusdt());
    let instrument_id = instrument.id();
    assert_eq!(
        string_field(event, "instrument_id"),
        instrument_id.to_string()
    );

    let payload = event
        .get("payload")
        .expect("input event payload is required");
    QuoteTick::new(
        instrument_id,
        Price::from(string_field(payload, "bid")),
        Price::from(string_field(payload, "ask")),
        Quantity::from(string_field(payload, "bid_size")),
        Quantity::from(string_field(payload, "ask_size")),
        timestamp(event, "ts_event").into(),
        timestamp(event, "ts_init").into(),
    )
}

fn cache_quote_event(
    event_type: &str,
    sequence: u64,
    instrument_id: InstrumentId,
    venue: &str,
    quote: &QuoteTick,
    cached: bool,
) -> Value {
    json!({
        "event_type": event_type,
        "ts_event": sequence.to_string(),
        "ts_init": sequence.to_string(),
        "instrument_id": instrument_id.to_string(),
        "venue": venue,
        "payload": {
            "bid": quote.bid_price.to_string(),
            "ask": quote.ask_price.to_string(),
            "cached": cached.to_string(),
        }
    })
}

fn msgbus_quote_event(
    event_type: &str,
    sequence: u64,
    instrument_id: InstrumentId,
    venue: &str,
    topic: &str,
    quote: &QuoteTick,
) -> Value {
    json!({
        "event_type": event_type,
        "ts_event": sequence.to_string(),
        "ts_init": sequence.to_string(),
        "instrument_id": instrument_id.to_string(),
        "venue": venue,
        "payload": {
            "topic": topic,
            "bid": quote.bid_price.to_string(),
            "ask": quote.ask_price.to_string(),
        }
    })
}

fn cache_disposed_event(instrument_id: InstrumentId, venue: &str, cached: bool) -> Value {
    json!({
        "event_type": "cache.disposed",
        "ts_event": "3",
        "ts_init": "3",
        "instrument_id": instrument_id.to_string(),
        "venue": venue,
        "payload": {
            "cached": cached.to_string(),
        }
    })
}

fn msgbus_disposed_event(instrument_id: InstrumentId, venue: &str, pub_count: u64) -> Value {
    json!({
        "event_type": "msgbus.disposed",
        "ts_event": "4",
        "ts_init": "4",
        "instrument_id": instrument_id.to_string(),
        "venue": venue,
        "payload": {
            "pub_count": pub_count.to_string(),
        }
    })
}

fn timestamp(event: &Value, key: &str) -> u64 {
    let value = event
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a decimal string timestamp"));
    value
        .parse()
        .unwrap_or_else(|err| panic!("{key} must parse as u64: {err}"))
}

fn string_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a string"))
}
