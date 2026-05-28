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

use nautilus_core::UnixNanos;
use nautilus_model::{data::Data, identifiers::InstrumentId};
use nautilus_okx::websocket::{messages::OKXWsFrame, parse::parse_trade_msg_vec};
use serde_json::{Value, json};

const CASE_ID: &str = "adapter_payload.okx_ws_trade.001";

#[test]
fn okx_ws_trade_payload_replays_adapter_golden_trace() {
    let case = load_case(CASE_ID);
    let input_event = event_by_type(&case, "input", "adapter.okx.ws.payload");
    let expected = case
        .get("expected")
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .expect("expected.events must be an array");

    let actual = parse_okx_trade_payload(input_event);

    assert_eq!(
        actual, *expected,
        "OKX adapter payload parser output must match the golden trace"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root should resolve from crates/adapters/okx")
}

fn load_case(case_id: &str) -> Value {
    let trace = repository_root().join("tests/golden/adapter_payload_schema.jsonl");
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

fn parse_okx_trade_payload(input_event: &Value) -> Vec<Value> {
    let payload = input_event
        .get("payload")
        .expect("input event payload is required");
    let fixture = string_field(payload, "fixture");
    let instrument_id = InstrumentId::from(
        input_event
            .get("instrument_id")
            .and_then(Value::as_str)
            .expect("instrument_id is required"),
    );
    let price_precision = u8_field(payload, "price_precision");
    let size_precision = u8_field(payload, "size_precision");
    let ts_init = UnixNanos::from(
        input_event
            .get("ts_init")
            .and_then(Value::as_str)
            .expect("ts_init is required"),
    );

    let fixture_path = repository_root().join(fixture);
    let raw = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_path.display()));
    let frame: OKXWsFrame = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", fixture_path.display()));
    let data = match frame {
        OKXWsFrame::Data { data, .. } => data,
        _ => panic!("expected OKX data frame in {}", fixture_path.display()),
    };

    parse_trade_msg_vec(
        data,
        &instrument_id,
        price_precision,
        size_precision,
        ts_init,
    )
    .expect("OKX trade payload should parse")
    .into_iter()
    .map(|item| match item {
        Data::Trade(trade) => trade_event(&trade),
        _ => panic!("expected parsed trade tick"),
    })
    .collect()
}

fn trade_event(trade: &nautilus_model::data::TradeTick) -> Value {
    json!({
        "event_type": "adapter.okx.trade_tick",
        "ts_event": trade.ts_event.as_u64().to_string(),
        "ts_init": trade.ts_init.as_u64().to_string(),
        "instrument_id": trade.instrument_id.to_string(),
        "venue": "OKX",
        "payload": {
            "price": trade.price.to_string(),
            "size": trade.size.to_string(),
            "aggressor_side": format!("{:?}", trade.aggressor_side),
            "trade_id": trade.trade_id.to_string(),
        }
    })
}

fn string_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a string"))
}

fn u8_field(value: &Value, key: &str) -> u8 {
    string_field(value, key)
        .parse::<u8>()
        .unwrap_or_else(|err| panic!("{key} must be a u8 string: {err}"))
}
