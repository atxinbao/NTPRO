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
use nautilus_model::{
    data::{Bar, BarType, BookOrder, InstrumentStatus, OrderBookDelta, QuoteTick, TradeTick},
    enums::{AggressorSide, BookAction, MarketStatusAction, OrderSide},
    identifiers::{InstrumentId, Symbol, TradeId, Venue},
    types::{Price, Quantity},
};
use serde_json::{Value, json};
use ustr::Ustr;

#[test]
fn rust_model_replays_market_data_golden_traces() {
    let cases = load_cases("market_data_schema.jsonl");
    assert_eq!(
        cases.len(),
        6,
        "market-data trace file should keep the six scoped v0.2 rows executable"
    );

    for case in cases {
        let expected = case
            .get("expected")
            .and_then(|value| value.get("events"))
            .and_then(Value::as_array)
            .expect("expected.events must be an array");
        let actual = replay_market_data_case(&case);

        assert_eq!(
            actual,
            *expected,
            "Rust model replay must match {}",
            string_field(&case, "case_id")
        );
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/model")
}

fn load_cases(file_name: &str) -> Vec<Value> {
    let trace = repository_root().join("tests/golden").join(file_name);
    fs::read_to_string(&trace)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", trace.display()))
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(|line| {
            serde_json::from_str::<Value>(line)
                .unwrap_or_else(|err| panic!("{} invalid JSON: {err}", trace.display()))
        })
        .collect()
}

fn replay_market_data_case(case: &Value) -> Vec<Value> {
    let input = case
        .get("input")
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .expect("input.events must be an array");

    input.iter().map(replay_market_data_event).collect()
}

fn replay_market_data_event(event: &Value) -> Value {
    match string_field(event, "event_type") {
        "market_data.quote_tick" => quote_event(quote_from_event(event)),
        "market_data.trade_tick" => trade_event(trade_from_event(event)),
        "market_data.bar" => bar_event(bar_from_event(event)),
        "market_data.order_book_delta" => {
            order_book_delta_event(order_book_delta_from_event(event))
        }
        "market_data.instrument_status" => {
            instrument_status_event(event, instrument_status_from_event(event))
        }
        "market_data.instrument_definition" => {
            instrument_definition_event(instrument_definition_from_event(event))
        }
        event_type => panic!("unsupported market-data golden trace event {event_type}"),
    }
}

fn quote_from_event(event: &Value) -> QuoteTick {
    let payload = payload(event);
    QuoteTick::new(
        instrument_id(event),
        Price::from(string_field(payload, "bid")),
        Price::from(string_field(payload, "ask")),
        Quantity::from(string_field(payload, "bid_size")),
        Quantity::from(string_field(payload, "ask_size")),
        timestamp(event, "ts_event").into(),
        timestamp(event, "ts_init").into(),
    )
}

fn trade_from_event(event: &Value) -> TradeTick {
    let payload = payload(event);
    TradeTick::new(
        instrument_id(event),
        Price::from(string_field(payload, "price")),
        Quantity::from(string_field(payload, "size")),
        aggressor_side(string_field(payload, "aggressor_side")),
        TradeId::from(string_field(payload, "trade_id")),
        timestamp(event, "ts_event").into(),
        timestamp(event, "ts_init").into(),
    )
}

fn bar_from_event(event: &Value) -> Bar {
    let payload = payload(event);
    Bar::new(
        BarType::from(string_field(payload, "bar_type")),
        Price::from(string_field(payload, "open")),
        Price::from(string_field(payload, "high")),
        Price::from(string_field(payload, "low")),
        Price::from(string_field(payload, "close")),
        Quantity::from(string_field(payload, "volume")),
        timestamp(event, "ts_event").into(),
        timestamp(event, "ts_init").into(),
    )
}

fn order_book_delta_from_event(event: &Value) -> OrderBookDelta {
    let payload = payload(event);
    let order = BookOrder::new(
        order_side(string_field(payload, "side")),
        Price::from(string_field(payload, "price")),
        Quantity::from(string_field(payload, "size")),
        sequence(payload),
    );
    OrderBookDelta::new(
        instrument_id(event),
        book_action(string_field(payload, "action")),
        order,
        0,
        sequence(payload),
        timestamp(event, "ts_event").into(),
        timestamp(event, "ts_init").into(),
    )
}

fn instrument_status_from_event(event: &Value) -> InstrumentStatus {
    let payload = payload(event);
    let action = match string_field(payload, "status") {
        "open" => MarketStatusAction::Trading,
        status => panic!("unsupported instrument status {status}"),
    };
    InstrumentStatus::new(
        instrument_id(event),
        action,
        timestamp(event, "ts_event").into(),
        timestamp(event, "ts_init").into(),
        Some(Ustr::from(string_field(payload, "reason"))),
        Some(Ustr::from(string_field(payload, "action"))),
        Some(matches!(action, MarketStatusAction::Trading)),
        Some(matches!(action, MarketStatusAction::Trading)),
        None,
    )
}

fn instrument_definition_from_event(event: &Value) -> CatalogInstrumentDefinition {
    let payload = payload(event);
    let venue = Venue::from(string_field(event, "venue"));
    let symbol = Symbol::from(string_field(payload, "symbol"));
    let instrument_id = InstrumentId { symbol, venue };
    assert_eq!(
        instrument_id,
        instrument_id_from_event(event),
        "instrument definition must agree with event instrument_id"
    );

    CatalogInstrumentDefinition {
        instrument_id,
        symbol,
        price_precision: string_field(payload, "price_precision").to_string(),
        size_precision: string_field(payload, "size_precision").to_string(),
        ts_event: timestamp(event, "ts_event").into(),
        ts_init: timestamp(event, "ts_init").into(),
    }
}

#[derive(Debug)]
struct CatalogInstrumentDefinition {
    instrument_id: InstrumentId,
    symbol: Symbol,
    price_precision: String,
    size_precision: String,
    ts_event: UnixNanos,
    ts_init: UnixNanos,
}

fn quote_event(quote: QuoteTick) -> Value {
    json!({
        "event_type": "market_data.quote_tick",
        "ts_event": quote.ts_event.to_string(),
        "ts_init": quote.ts_init.to_string(),
        "instrument_id": quote.instrument_id.to_string(),
        "venue": quote.instrument_id.venue.to_string(),
        "payload": {
            "bid": quote.bid_price.to_string(),
            "ask": quote.ask_price.to_string(),
            "bid_size": quote.bid_size.to_string(),
            "ask_size": quote.ask_size.to_string(),
        }
    })
}

fn trade_event(trade: TradeTick) -> Value {
    json!({
        "event_type": "market_data.trade_tick",
        "ts_event": trade.ts_event.to_string(),
        "ts_init": trade.ts_init.to_string(),
        "instrument_id": trade.instrument_id.to_string(),
        "venue": trade.instrument_id.venue.to_string(),
        "payload": {
            "price": trade.price.to_string(),
            "size": trade.size.to_string(),
            "aggressor_side": match trade.aggressor_side {
                AggressorSide::Buyer => "buyer",
                AggressorSide::Seller => "seller",
                AggressorSide::NoAggressor => "none",
            },
            "trade_id": trade.trade_id.to_string(),
        }
    })
}

fn bar_event(bar: Bar) -> Value {
    json!({
        "event_type": "market_data.bar",
        "ts_event": bar.ts_event.to_string(),
        "ts_init": bar.ts_init.to_string(),
        "instrument_id": bar.instrument_id().to_string(),
        "venue": bar.instrument_id().venue.to_string(),
        "payload": {
            "bar_type": bar.bar_type.to_string(),
            "open": bar.open.to_string(),
            "high": bar.high.to_string(),
            "low": bar.low.to_string(),
            "close": bar.close.to_string(),
            "volume": bar.volume.to_string(),
        }
    })
}

fn order_book_delta_event(delta: OrderBookDelta) -> Value {
    json!({
        "event_type": "market_data.order_book_delta",
        "ts_event": delta.ts_event.to_string(),
        "ts_init": delta.ts_init.to_string(),
        "instrument_id": delta.instrument_id.to_string(),
        "venue": delta.instrument_id.venue.to_string(),
        "payload": {
            "action": match delta.action {
                BookAction::Update => "update",
                BookAction::Add => "add",
                BookAction::Delete => "delete",
                BookAction::Clear => "clear",
            },
            "side": match delta.order.side {
                OrderSide::Buy => "bid",
                OrderSide::Sell => "ask",
                OrderSide::NoOrderSide => "none",
            },
            "price": delta.order.price.to_string(),
            "size": delta.order.size.to_string(),
            "sequence": delta.sequence.to_string(),
        }
    })
}

fn instrument_status_event(source_event: &Value, status: InstrumentStatus) -> Value {
    let payload = payload(source_event);
    json!({
        "event_type": "market_data.instrument_status",
        "ts_event": status.ts_event.to_string(),
        "ts_init": status.ts_init.to_string(),
        "instrument_id": status.instrument_id.to_string(),
        "venue": status.instrument_id.venue.to_string(),
        "payload": {
            "status": match status.action {
                MarketStatusAction::Trading => "open",
                _ => "unknown",
            },
            "action": status.trading_event.map_or_else(|| string_field(payload, "action").to_string(), |value| value.to_string()),
            "reason": status.reason.map_or_else(|| string_field(payload, "reason").to_string(), |value| value.to_string()),
        }
    })
}

fn instrument_definition_event(definition: CatalogInstrumentDefinition) -> Value {
    json!({
        "event_type": "market_data.instrument_definition",
        "ts_event": definition.ts_event.to_string(),
        "ts_init": definition.ts_init.to_string(),
        "instrument_id": definition.instrument_id.to_string(),
        "venue": definition.instrument_id.venue.to_string(),
        "payload": {
            "symbol": definition.symbol.to_string(),
            "price_precision": definition.price_precision,
            "size_precision": definition.size_precision,
        }
    })
}

fn aggressor_side(value: &str) -> AggressorSide {
    match value {
        "buyer" => AggressorSide::Buyer,
        "seller" => AggressorSide::Seller,
        "none" => AggressorSide::NoAggressor,
        side => panic!("unsupported aggressor side {side}"),
    }
}

fn book_action(value: &str) -> BookAction {
    match value {
        "add" => BookAction::Add,
        "update" => BookAction::Update,
        "delete" => BookAction::Delete,
        "clear" => BookAction::Clear,
        action => panic!("unsupported book action {action}"),
    }
}

fn order_side(value: &str) -> OrderSide {
    match value {
        "bid" | "buy" => OrderSide::Buy,
        "ask" | "sell" => OrderSide::Sell,
        side => panic!("unsupported order side {side}"),
    }
}

fn instrument_id(event: &Value) -> InstrumentId {
    instrument_id_from_event(event)
}

fn instrument_id_from_event(event: &Value) -> InstrumentId {
    InstrumentId::from(string_field(event, "instrument_id"))
}

fn payload(event: &Value) -> &Value {
    event.get("payload").expect("event payload is required")
}

fn sequence(payload: &Value) -> u64 {
    string_field(payload, "sequence")
        .parse()
        .expect("sequence must parse as u64")
}

fn timestamp(event: &Value, key: &str) -> u64 {
    string_field(event, key)
        .parse()
        .unwrap_or_else(|err| panic!("{key} must parse as u64: {err}"))
}

fn string_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a string"))
}
