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
    collections::BTreeMap,
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
use nautilus_trading::examples::strategies::EmaCross;
use serde_json::{Value, json};

const CASE_ID: &str = "backtest_live.single_quote_replay.001";
const MVP_EMA_CASE_ID: &str = "mvp.ema_cross_deterministic.001";

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

#[test]
fn rust_backtest_engine_replays_mvp_ema_strategy_canonical_result() {
    let actual = run_mvp_ema_backtest();
    println!(
        "mvp_ema_canonical_result={}",
        serde_json::to_string(&actual).expect("canonical result should serialize")
    );

    let expected = json!({
        "backtest_end": "1735689719000000000",
        "backtest_start": "1735689600000000000",
        "case_id": "mvp.ema_cross_deterministic.001",
        "general_stats": {
            "Long Ratio": "0.330000000000",
        },
        "instrument_id": "AUD/USD.SIM",
        "iterations": 120,
        "pnl_stats": {
            "USD": {
                "Avg Loser": "-1.306666666667",
                "Avg Winner": "NaN",
                "Expectancy": "-1.306666666667",
                "Max Loser": "-1.310000000000",
                "Max Winner": "NaN",
                "Min Loser": "-1.300000000000",
                "Min Winner": "NaN",
                "PnL (total)": "-3.920000000042",
                "PnL% (total)": "-0.000392000000",
                "Win Rate": "0.000000000000",
            },
        },
        "quotes": 120,
        "return_stats": {
            "Average (Return)": "NaN",
            "Average Loss (Return)": "NaN",
            "Average Win (Return)": "NaN",
            "Profit Factor": "NaN",
            "Returns Volatility (252 days)": "NaN",
            "Risk Return Ratio": "NaN",
            "Sharpe Ratio (252 days)": "NaN",
            "Sortino Ratio (252 days)": "NaN",
        },
        "strategy": "ema-cross",
        "total_events": 9,
        "total_orders": 3,
        "total_positions": 3,
    });
    assert_eq!(
        actual, expected,
        "MVP EMA strategy result must remain deterministic"
    );
}

fn run_mvp_ema_backtest() -> Value {
    let mut engine = BacktestEngine::new(BacktestEngineConfig::default()).unwrap();
    engine
        .add_venue(
            SimulatedVenueConfig::builder()
                .venue(Venue::from("SIM"))
                .oms_type(OmsType::Hedging)
                .account_type(AccountType::Margin)
                .book_type(BookType::L1_MBP)
                .starting_balances(vec![Money::from("1_000_000 USD")])
                .build(),
        )
        .unwrap();

    let instrument = InstrumentAny::CurrencyPair(nautilus_model::instruments::stubs::audusd_sim());
    let instrument_id = instrument.id();
    engine.add_instrument(&instrument).unwrap();
    engine
        .add_strategy(EmaCross::new(
            instrument_id,
            Quantity::from("100000"),
            10,
            20,
        ))
        .unwrap();
    engine
        .add_data(mvp_ema_quotes(instrument_id), None, true, true)
        .unwrap();
    engine
        .run(None, None, Some(MVP_EMA_CASE_ID.to_string()), false)
        .unwrap();

    let result = engine.get_result();
    let pnl_stats = result
        .stats_pnls
        .iter()
        .map(|(currency, stats)| {
            let values = stats
                .iter()
                .map(|(name, value)| (name.clone(), canonical_float(*value)))
                .collect::<BTreeMap<_, _>>();
            (currency.clone(), values)
        })
        .collect::<BTreeMap<_, _>>();
    let return_stats = result
        .stats_returns
        .iter()
        .map(|(name, value)| (name.clone(), canonical_float(*value)))
        .collect::<BTreeMap<_, _>>();
    let general_stats = result
        .stats_general
        .iter()
        .map(|(name, value)| (name.clone(), canonical_float(*value)))
        .collect::<BTreeMap<_, _>>();

    json!({
        "case_id": MVP_EMA_CASE_ID,
        "instrument_id": instrument_id.to_string(),
        "strategy": "ema-cross",
        "quotes": 120,
        "iterations": result.iterations,
        "total_events": result.total_events,
        "total_orders": result.total_orders,
        "total_positions": result.total_positions,
        "backtest_start": result.backtest_start.map(|value| value.as_u64().to_string()),
        "backtest_end": result.backtest_end.map(|value| value.as_u64().to_string()),
        "pnl_stats": pnl_stats,
        "return_stats": return_stats,
        "general_stats": general_stats,
    })
}

fn mvp_ema_quotes(instrument_id: InstrumentId) -> Vec<Data> {
    let spread = 0.00020;
    let base_ts: u64 = 1_735_689_600_000_000_000;
    let interval: u64 = 1_000_000_000;
    (0..120)
        .map(|tick| {
            let cycle = tick as f64 / 12.0;
            let mid = 0.65000 + (cycle.sin() * 0.00400) + ((tick % 40) as f64 * 0.00008);
            let bid = format!("{mid:.5}");
            let ask = format!("{:.5}", mid + spread);
            Data::Quote(QuoteTick::new(
                instrument_id,
                Price::from(bid),
                Price::from(ask),
                Quantity::from("100000"),
                Quantity::from("100000"),
                (base_ts + tick as u64 * interval).into(),
                (base_ts + tick as u64 * interval).into(),
            ))
        })
        .collect()
}

fn canonical_float(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.12}")
    } else {
        value.to_string()
    }
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
