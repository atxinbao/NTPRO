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

const BACKTEST_CASE_ID: &str = "backtest_live.single_quote_replay.001";
const LIVE_SANDBOX_CASE_ID: &str = "backtest_live.sandbox_lifecycle_stop.001";
const PARITY_CASE_ID: &str = "backtest_live.semantic_parity.scope.001";

#[test]
fn rust_backtest_and_live_sandbox_match_scoped_semantic_parity_trace() {
    let backtest_case = load_case("backtest_replay_schema.jsonl", BACKTEST_CASE_ID);
    let live_case = load_case("live_sandbox_lifecycle_schema.jsonl", LIVE_SANDBOX_CASE_ID);
    let parity_case = load_case("backtest_live_semantic_parity_schema.jsonl", PARITY_CASE_ID);

    let source_event = event_by_type(&parity_case, "input", "backtest_live.parity.sources");
    let source_payload = source_event
        .get("payload")
        .expect("parity source payload is required");
    assert_eq!(
        string_field(source_payload, "backtest_case_id"),
        BACKTEST_CASE_ID
    );
    assert_eq!(
        string_field(source_payload, "live_sandbox_case_id"),
        LIVE_SANDBOX_CASE_ID
    );

    let input_event = event_by_type(&backtest_case, "input", "market_data.quote_tick");
    let expected_backtest = event_by_type(&backtest_case, "expected", "backtest.result");
    let actual_backtest = run_single_quote_backtest(BACKTEST_CASE_ID, input_event);

    assert_eq!(
        actual_backtest, *expected_backtest,
        "Rust BacktestEngine output must keep matching its source golden trace"
    );

    let actual_parity = parity_event(&actual_backtest, &live_case);
    let expected_parity = event_by_type(&parity_case, "expected", "backtest_live.semantic_parity");

    assert_eq!(
        actual_parity, *expected_parity,
        "Rust backtest/live scoped parity summary must match the parity golden trace"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/backtest")
}

fn load_case(file_name: &str, case_id: &str) -> Value {
    let trace = repository_root().join("tests/golden").join(file_name);
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
    assert_eq!(
        string_field(input_event, "instrument_id"),
        instrument_id.to_string()
    );

    let venue = Venue::from(string_field(input_event, "venue"));
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

fn parity_event(backtest_result: &Value, live_case: &Value) -> Value {
    let backtest_payload = backtest_result
        .get("payload")
        .expect("backtest result payload is required");
    let live_events = live_case
        .get("expected")
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .expect("live expected events must be an array");
    let live_states = live_events
        .iter()
        .map(|event| {
            event
                .get("payload")
                .and_then(|payload| payload.get("state"))
                .and_then(Value::as_str)
                .expect("live expected state must be a string")
        })
        .collect::<Vec<_>>();
    let live_environment = live_events
        .first()
        .and_then(|event| event.get("payload"))
        .and_then(|payload| payload.get("environment"))
        .and_then(Value::as_str)
        .expect("live environment must be present");

    json!({
        "event_type": "backtest_live.semantic_parity",
        "ts_event": "0",
        "ts_init": "0",
        "payload": {
            "source_cases": {
                "backtest": BACKTEST_CASE_ID,
                "live_sandbox": LIVE_SANDBOX_CASE_ID,
            },
            "backtest": {
                "engine": "BacktestEngine",
                "environment": "Backtest",
                "result_event": string_field(backtest_result, "event_type"),
                "instrument_id": string_field(backtest_result, "instrument_id"),
                "venue": string_field(backtest_result, "venue"),
                "run_config_id": string_field(backtest_payload, "run_config_id"),
                "iterations": string_field(backtest_payload, "iterations"),
                "total_orders": string_field(backtest_payload, "total_orders"),
                "total_positions": string_field(backtest_payload, "total_positions"),
                "backtest_start": string_field(backtest_payload, "backtest_start"),
                "backtest_end": string_field(backtest_payload, "backtest_end"),
            },
            "live_sandbox": {
                "node": "LiveNode",
                "environment": live_environment,
                "state_path": live_states,
                "terminal_state": live_states.last().copied().unwrap_or_default(),
                "state_count": live_states.len().to_string(),
            },
            "shared_invariants": {
                "category": "backtest_live",
                "deterministic_fixture": "true",
                "python_surface_required": "false",
                "external_io_required": "false",
                "order_side_effects": "none",
            }
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

fn nanos_to_string(value: Option<nautilus_core::UnixNanos>) -> String {
    value
        .map(|value| value.as_u64().to_string())
        .unwrap_or_default()
}
