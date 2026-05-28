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
    fs,
    path::{Path, PathBuf},
};

use nautilus_backtest::{
    config::{BacktestEngineConfig, SimulatedVenueConfig},
    engine::BacktestEngine,
};
use nautilus_model::{
    data::{Data, QuoteTick},
    enums::{AccountType, BookType, OmsType},
    identifiers::{InstrumentId, Venue},
    instruments::{Instrument, InstrumentAny, stubs::crypto_perpetual_ethusdt},
    types::{Money, Price, Quantity},
};
use serde_json::{Value, json};

const CASE_ID: &str = "backtest_live.single_quote_replay.001";

#[test]
fn rust_backtest_engine_replays_single_quote_golden_trace() {
    let case = load_case(CASE_ID);
    let input_event = event_by_type(&case, "input", "market_data.quote_tick");
    let expected_event = event_by_type(&case, "expected", "backtest.result");

    let actual_event = run_single_quote_backtest(CASE_ID, input_event);

    assert_eq!(
        actual_event, *expected_event,
        "Rust BacktestEngine output must match the backtest golden trace"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/backtest")
}

fn load_case(case_id: &str) -> Value {
    let trace = repository_root().join("tests/golden/backtest_replay_schema.jsonl");
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

fn run_single_quote_backtest(case_id: &str, input_event: &Value) -> Value {
    let instrument = InstrumentAny::CryptoPerpetual(crypto_perpetual_ethusdt());
    let instrument_id = instrument.id();
    let expected_instrument_id = instrument_id.to_string();
    assert_eq!(
        string_field(input_event, "instrument_id"),
        expected_instrument_id.as_str()
    );

    let venue = venue(input_event);
    let mut engine = BacktestEngine::new(BacktestEngineConfig::default()).unwrap();
    engine
        .add_venue(
            SimulatedVenueConfig::builder()
                .venue(venue)
                .oms_type(OmsType::Netting)
                .account_type(AccountType::Margin)
                .book_type(BookType::L1_MBP)
                .starting_balances(vec![Money::from("1_000_000 USDT")])
                .build(),
        )
        .unwrap();
    engine.add_instrument(&instrument).unwrap();
    engine
        .add_data(
            vec![Data::Quote(quote_from_event(instrument_id, input_event))],
            None,
            true,
            true,
        )
        .unwrap();
    engine
        .run(None, None, Some(case_id.to_string()), false)
        .unwrap();

    let result = engine.get_result();
    json!({
        "event_type": "backtest.result",
        "ts_event": nanos_to_string(result.backtest_end),
        "ts_init": nanos_to_string(result.backtest_end),
        "instrument_id": instrument_id.to_string(),
        "venue": venue.to_string(),
        "payload": {
            "run_config_id": result.run_config_id.unwrap_or_default(),
            "iterations": result.iterations.to_string(),
            "total_orders": result.total_orders.to_string(),
            "total_positions": result.total_positions.to_string(),
            "backtest_start": nanos_to_string(result.backtest_start),
            "backtest_end": nanos_to_string(result.backtest_end),
        }
    })
}

fn quote_from_event(instrument_id: InstrumentId, event: &Value) -> QuoteTick {
    let payload = event
        .get("payload")
        .expect("input event payload is required");
    let ts_event = timestamp(event, "ts_event");
    let ts_init = timestamp(event, "ts_init");
    QuoteTick::new(
        instrument_id,
        Price::from(string_field(payload, "bid")),
        Price::from(string_field(payload, "ask")),
        Quantity::from(string_field(payload, "bid_size")),
        Quantity::from(string_field(payload, "ask_size")),
        ts_event.into(),
        ts_init.into(),
    )
}

fn venue(event: &Value) -> Venue {
    Venue::from(string_field(event, "venue"))
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

fn nanos_to_string(value: Option<nautilus_core::UnixNanos>) -> String {
    value
        .map(|value| value.as_u64().to_string())
        .unwrap_or_default()
}
