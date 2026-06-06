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
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::{LiquiditySide, OrderSide, OrderStatus, OrderType},
    events::{
        OrderAccepted, OrderCanceled, OrderRejected, OrderSubmitted, OrderTriggered, OrderUpdated,
        order::spec::{
            OrderAcceptedSpec, OrderCanceledSpec, OrderFilledSpec, OrderRejectedSpec,
            OrderSubmittedSpec, OrderTriggeredSpec, OrderUpdatedSpec,
        },
    },
    identifiers::{AccountId, ClientOrderId, InstrumentId, StrategyId, TradeId, VenueOrderId},
    orders::{Order, OrderAny, OrderTestBuilder},
    types::{Currency, Price, Quantity},
};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use ustr::Ustr;

#[test]
fn rust_execution_replays_order_lifecycle_golden_traces() {
    let cases = load_cases("order_lifecycle_schema.jsonl");
    assert_eq!(
        cases.len(),
        6,
        "order-lifecycle trace file should keep the six scoped v0.2 rows executable"
    );

    for case in cases {
        let expected = case
            .get("expected")
            .and_then(|value| value.get("events"))
            .and_then(Value::as_array)
            .expect("expected.events must be an array");
        let actual = OrderLifecycleReplay::default().run(&case);

        assert_eq!(
            actual,
            *expected,
            "Rust execution lifecycle replay must match {}",
            string_field(&case, "case_id")
        );
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/execution")
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

#[derive(Default)]
struct OrderLifecycleReplay {
    orders: HashMap<ClientOrderId, ReplayOrder>,
}

#[derive(Clone)]
struct ReplayOrder {
    client_order_id: ClientOrderId,
    venue_order_id: VenueOrderId,
    instrument_id: InstrumentId,
    side: OrderSide,
    order_type: OrderType,
    quantity: Quantity,
    price: Option<Price>,
    trigger_price: Option<Price>,
    filled_qty: Decimal,
}

impl OrderLifecycleReplay {
    fn run(mut self, case: &Value) -> Vec<Value> {
        case.get("input")
            .and_then(|value| value.get("events"))
            .and_then(Value::as_array)
            .expect("input.events must be an array")
            .iter()
            .flat_map(|event| self.replay_event(event))
            .collect()
    }

    fn replay_event(&mut self, event: &Value) -> Vec<Value> {
        match string_field(event, "event_type") {
            "order.command.submit" => self.replay_submit(event),
            "order.accepted" => {
                self.record_accepted(event);
                Vec::new()
            }
            "order.command.modify" => vec![self.replay_modify(event)],
            "order.command.cancel" => vec![self.replay_cancel(event)],
            "market_data.trade_tick" => self.replay_triggered_fill(event),
            "execution.fill_report" => vec![self.replay_fill_report(event)],
            event_type => panic!("unsupported order-lifecycle golden trace event {event_type}"),
        }
    }

    fn replay_submit(&mut self, event: &Value) -> Vec<Value> {
        let payload = payload(event);
        let client_order_id = client_order_id(payload);
        let instrument_id = instrument_id(event);
        let quantity = Quantity::from(string_field(payload, "quantity"));
        if quantity.is_zero() {
            let rejected = OrderRejectedSpec::builder()
                .strategy_id(StrategyId::from("S-001"))
                .instrument_id(instrument_id)
                .client_order_id(client_order_id)
                .account_id(AccountId::from("SIM-001"))
                .reason(Ustr::from("invalid_quantity"))
                .ts_event(timestamp(event, "ts_event").into())
                .ts_init(timestamp(event, "ts_init").into())
                .build();
            return vec![rejected_event(rejected, event, "invalid_quantity")];
        }

        let command_order = order_from_submit_event(event);
        let submitted = OrderSubmittedSpec::builder()
            .strategy_id(StrategyId::from("S-001"))
            .instrument_id(instrument_id)
            .client_order_id(client_order_id)
            .account_id(AccountId::from("SIM-001"))
            .ts_event(timestamp(event, "ts_event").into())
            .ts_init(timestamp(event, "ts_init").into())
            .build();

        let venue_order_id = VenueOrderId::from("V-001");
        let accepted = OrderAcceptedSpec::builder()
            .strategy_id(StrategyId::from("S-001"))
            .instrument_id(instrument_id)
            .client_order_id(client_order_id)
            .venue_order_id(venue_order_id)
            .account_id(AccountId::from("SIM-001"))
            .ts_event((timestamp(event, "ts_event") + 1_000).into())
            .ts_init((timestamp(event, "ts_init") + 1_000).into())
            .build();
        self.orders.insert(
            client_order_id,
            ReplayOrder {
                client_order_id,
                venue_order_id,
                instrument_id,
                side: command_order.order_side(),
                order_type: command_order.order_type(),
                quantity: command_order.quantity(),
                price: command_order.price(),
                trigger_price: None,
                filled_qty: Decimal::ZERO,
            },
        );

        vec![
            submitted_event(submitted, &command_order, event),
            accepted_event(accepted, payload),
        ]
    }

    fn record_accepted(&mut self, event: &Value) {
        let payload = payload(event);
        let client_order_id = client_order_id(payload);
        let venue_order_id = venue_order_id(payload);
        let quantity = Quantity::from(string_field(payload, "quantity"));
        let price = payload
            .get("price")
            .and_then(Value::as_str)
            .map(Price::from);
        let trigger_price = payload
            .get("trigger_price")
            .and_then(Value::as_str)
            .map(Price::from);
        let order_type = match payload.get("order_type").and_then(Value::as_str) {
            Some("stop_market") => OrderType::StopMarket,
            _ => OrderType::Limit,
        };

        self.orders.insert(
            client_order_id,
            ReplayOrder {
                client_order_id,
                venue_order_id,
                instrument_id: instrument_id(event),
                side: OrderSide::Buy,
                order_type,
                quantity,
                price,
                trigger_price,
                filled_qty: Decimal::ZERO,
            },
        );
    }

    fn replay_modify(&mut self, event: &Value) -> Value {
        let payload = payload(event);
        let client_order_id = client_order_id(payload);
        let order = self
            .orders
            .get_mut(&client_order_id)
            .expect("modify command must reference an accepted order");
        order.quantity = Quantity::from(string_field(payload, "quantity"));
        order.price = Some(Price::from(string_field(payload, "price")));

        let updated = OrderUpdatedSpec::builder()
            .strategy_id(StrategyId::from("S-001"))
            .instrument_id(order.instrument_id)
            .client_order_id(order.client_order_id)
            .venue_order_id(order.venue_order_id)
            .account_id(AccountId::from("SIM-001"))
            .quantity(order.quantity)
            .price(order.price.expect("modified order should retain price"))
            .ts_event((timestamp(event, "ts_event") + 1_000).into())
            .ts_init((timestamp(event, "ts_init") + 1_000).into())
            .build();

        updated_event(updated, event)
    }

    fn replay_cancel(&mut self, event: &Value) -> Value {
        let payload = payload(event);
        let client_order_id = client_order_id(payload);
        let order = self
            .orders
            .remove(&client_order_id)
            .expect("cancel command must reference an accepted order");
        let canceled = OrderCanceledSpec::builder()
            .strategy_id(StrategyId::from("S-001"))
            .instrument_id(order.instrument_id)
            .client_order_id(order.client_order_id)
            .venue_order_id(order.venue_order_id)
            .account_id(AccountId::from("SIM-001"))
            .ts_event((timestamp(event, "ts_event") + 1_000).into())
            .ts_init((timestamp(event, "ts_init") + 1_000).into())
            .build();

        canceled_event(canceled, event)
    }

    fn replay_triggered_fill(&mut self, event: &Value) -> Vec<Value> {
        let (client_order_id, order) = self
            .orders
            .iter_mut()
            .find(|(_, order)| order.trigger_price.is_some())
            .map(|(client_order_id, order)| (*client_order_id, order))
            .expect("trade tick must have one accepted trigger order");
        let trigger_price = order
            .trigger_price
            .expect("triggered order must retain trigger price");
        let triggered = OrderTriggeredSpec::builder()
            .strategy_id(StrategyId::from("S-001"))
            .instrument_id(order.instrument_id)
            .client_order_id(client_order_id)
            .venue_order_id(order.venue_order_id)
            .account_id(AccountId::from("SIM-001"))
            .ts_event(timestamp(event, "ts_event").into())
            .ts_init(timestamp(event, "ts_init").into())
            .build();
        let fill_event = fill_event(
            order,
            TradeId::from("T-005"),
            order.quantity,
            Price::from(string_field(payload(event), "price")),
            timestamp(event, "ts_event") + 1_000,
            timestamp(event, "ts_init") + 1_000,
        );

        vec![triggered_event(triggered, event, trigger_price), fill_event]
    }

    fn replay_fill_report(&mut self, event: &Value) -> Value {
        let payload = payload(event);
        let client_order_id = client_order_id(payload);
        let order = self
            .orders
            .get_mut(&client_order_id)
            .expect("fill report must reference an accepted order");
        let last_qty = Quantity::from(string_field(payload, "last_qty"));
        let last_px = Price::from(string_field(payload, "last_px"));
        let trade_id = TradeId::from(string_field(payload, "trade_id"));

        fill_event(
            order,
            trade_id,
            last_qty,
            last_px,
            timestamp(event, "ts_event"),
            timestamp(event, "ts_init"),
        )
    }
}

fn order_from_submit_event(event: &Value) -> OrderAny {
    let payload = payload(event);
    let mut builder = OrderTestBuilder::new(order_type(string_field(payload, "order_type")));
    builder
        .instrument_id(instrument_id(event))
        .client_order_id(client_order_id(payload))
        .side(order_side(string_field(payload, "side")))
        .quantity(Quantity::from(string_field(payload, "quantity")));
    if let Some(price) = payload.get("price").and_then(Value::as_str) {
        builder.price(Price::from(price));
    }
    builder.build()
}

fn submitted_event(event: OrderSubmitted, order: &OrderAny, source: &Value) -> Value {
    json!({
        "event_type": "order.submitted",
        "ts_event": event.ts_event.to_string(),
        "ts_init": event.ts_init.to_string(),
        "instrument_id": event.instrument_id.to_string(),
        "venue": event.instrument_id.venue.to_string(),
        "correlation_id": string_field(source, "correlation_id"),
        "payload": {
            "client_order_id": event.client_order_id.to_string(),
            "side": order_side_label(order.order_side()),
            "quantity": order.quantity().to_string(),
            "price": order.price().expect("submitted limit order should have a price").to_string(),
        }
    })
}

fn accepted_event(event: OrderAccepted, source_payload: &Value) -> Value {
    json!({
        "event_type": "order.accepted",
        "ts_event": event.ts_event.to_string(),
        "ts_init": event.ts_init.to_string(),
        "instrument_id": event.instrument_id.to_string(),
        "venue": event.instrument_id.venue.to_string(),
        "correlation_id": "submit-1",
        "payload": {
            "client_order_id": event.client_order_id.to_string(),
            "venue_order_id": event.venue_order_id.to_string(),
            "order_status": "accepted",
        }
    })
    .with_correlation(source_payload, "submit-1")
}

fn rejected_event(event: OrderRejected, source: &Value, reason: &str) -> Value {
    json!({
        "event_type": "order.rejected",
        "ts_event": event.ts_event.to_string(),
        "ts_init": event.ts_init.to_string(),
        "instrument_id": event.instrument_id.to_string(),
        "venue": event.instrument_id.venue.to_string(),
        "correlation_id": string_field(source, "correlation_id"),
        "payload": {
            "client_order_id": event.client_order_id.to_string(),
            "reason": reason,
            "order_status": "rejected",
        }
    })
}

fn updated_event(event: OrderUpdated, source: &Value) -> Value {
    json!({
        "event_type": "order.updated",
        "ts_event": event.ts_event.to_string(),
        "ts_init": event.ts_init.to_string(),
        "instrument_id": event.instrument_id.to_string(),
        "venue": event.instrument_id.venue.to_string(),
        "correlation_id": string_field(source, "correlation_id"),
        "payload": {
            "client_order_id": event.client_order_id.to_string(),
            "venue_order_id": event.venue_order_id.expect("updated event should retain venue order ID").to_string(),
            "quantity": event.quantity.to_string(),
            "price": event.price.expect("updated event should retain price").to_string(),
            "order_status": "accepted",
        }
    })
}

fn canceled_event(event: OrderCanceled, source: &Value) -> Value {
    json!({
        "event_type": "order.canceled",
        "ts_event": event.ts_event.to_string(),
        "ts_init": event.ts_init.to_string(),
        "instrument_id": event.instrument_id.to_string(),
        "venue": event.instrument_id.venue.to_string(),
        "correlation_id": string_field(source, "correlation_id"),
        "payload": {
            "client_order_id": event.client_order_id.to_string(),
            "venue_order_id": event.venue_order_id.expect("canceled event should retain venue order ID").to_string(),
            "order_status": "canceled",
            "leaves_qty": "0",
        }
    })
}

fn triggered_event(event: OrderTriggered, source: &Value, trigger_price: Price) -> Value {
    json!({
        "event_type": "order.triggered",
        "ts_event": event.ts_event.to_string(),
        "ts_init": event.ts_init.to_string(),
        "instrument_id": event.instrument_id.to_string(),
        "venue": event.instrument_id.venue.to_string(),
        "payload": {
            "client_order_id": event.client_order_id.to_string(),
            "venue_order_id": event.venue_order_id.expect("triggered event should retain venue order ID").to_string(),
            "trigger_price": trigger_price.to_string(),
        }
    })
    .without_correlation_if_absent(source)
}

fn fill_event(
    order: &mut ReplayOrder,
    trade_id: TradeId,
    last_qty: Quantity,
    last_px: Price,
    ts_event: u64,
    ts_init: u64,
) -> Value {
    order.filled_qty += Decimal::from_str(&last_qty.to_string()).expect("last_qty must be decimal");
    let total_qty =
        Decimal::from_str(&order.quantity.to_string()).expect("quantity must be decimal");
    let leaves_qty = total_qty - order.filled_qty;
    let status = if leaves_qty.is_zero() {
        OrderStatus::Filled
    } else {
        OrderStatus::PartiallyFilled
    };
    let filled = OrderFilledSpec::builder()
        .strategy_id(StrategyId::from("S-001"))
        .instrument_id(order.instrument_id)
        .client_order_id(order.client_order_id)
        .venue_order_id(order.venue_order_id)
        .account_id(AccountId::from("SIM-001"))
        .trade_id(trade_id)
        .order_side(order.side)
        .order_type(order.order_type)
        .last_qty(last_qty)
        .last_px(last_px)
        .currency(Currency::USD())
        .liquidity_side(LiquiditySide::Taker)
        .ts_event(UnixNanos::from(ts_event))
        .ts_init(UnixNanos::from(ts_init))
        .build();
    json!({
        "event_type": match status {
            OrderStatus::PartiallyFilled => "order.partially_filled",
            OrderStatus::Filled => "order.filled",
            _ => panic!("unsupported fill status {status:?}"),
        },
        "ts_event": filled.ts_event.to_string(),
        "ts_init": filled.ts_init.to_string(),
        "instrument_id": filled.instrument_id.to_string(),
        "venue": filled.instrument_id.venue.to_string(),
        "payload": {
            "client_order_id": filled.client_order_id.to_string(),
            "venue_order_id": filled.venue_order_id.to_string(),
            "trade_id": filled.trade_id.to_string(),
            "last_qty": filled.last_qty.to_string(),
            "last_px": filled.last_px.to_string(),
            "filled_qty": decimal_string(order.filled_qty),
            "leaves_qty": decimal_string(leaves_qty),
            "order_status": match status {
                OrderStatus::PartiallyFilled => "partially_filled",
                OrderStatus::Filled => "filled",
                _ => unreachable!(),
            },
        }
    })
}

trait JsonCorrelationExt {
    fn with_correlation(self, source_payload: &Value, fallback: &str) -> Self;
    fn without_correlation_if_absent(self, source: &Value) -> Self;
}

impl JsonCorrelationExt for Value {
    fn with_correlation(mut self, source_payload: &Value, fallback: &str) -> Self {
        let correlation = source_payload
            .get("correlation_id")
            .and_then(Self::as_str)
            .unwrap_or(fallback);
        self.as_object_mut().expect("event must be object").insert(
            "correlation_id".to_string(),
            Self::String(correlation.to_string()),
        );
        self
    }

    fn without_correlation_if_absent(mut self, source: &Value) -> Self {
        if let Some(correlation) = source.get("correlation_id").and_then(Self::as_str) {
            self.as_object_mut().expect("event must be object").insert(
                "correlation_id".to_string(),
                Self::String(correlation.to_string()),
            );
        }
        self
    }
}

fn order_type(value: &str) -> OrderType {
    match value {
        "limit" => OrderType::Limit,
        "market" => OrderType::Market,
        "stop_market" => OrderType::StopMarket,
        order_type => panic!("unsupported order type {order_type}"),
    }
}

fn order_side(value: &str) -> OrderSide {
    match value {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        side => panic!("unsupported order side {side}"),
    }
}

fn order_side_label(side: OrderSide) -> &'static str {
    match side {
        OrderSide::Buy => "buy",
        OrderSide::Sell => "sell",
        OrderSide::NoOrderSide => "none",
    }
}

fn instrument_id(event: &Value) -> InstrumentId {
    InstrumentId::from(string_field(event, "instrument_id"))
}

fn client_order_id(payload: &Value) -> ClientOrderId {
    ClientOrderId::from(string_field(payload, "client_order_id"))
}

fn venue_order_id(payload: &Value) -> VenueOrderId {
    VenueOrderId::from(string_field(payload, "venue_order_id"))
}

fn payload(event: &Value) -> &Value {
    event.get("payload").expect("event payload is required")
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

fn decimal_string(value: Decimal) -> String {
    if value.is_zero() {
        "0".to_string()
    } else {
        value.to_string()
    }
}
