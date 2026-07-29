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
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use ahash::AHashMap;
use nautilus_common::{
    cache::Cache,
    clock::TestClock,
    messages::execution::{SubmitOrder, SubmitOrderList, TradingCommand},
    msgbus::{
        self, MessagingSwitchboard,
        stubs::{TypedIntoMessageSavingHandler, get_typed_into_message_saving_handler},
    },
    throttler::RateLimit,
};
use nautilus_core::UUID4;
use nautilus_model::{
    enums::{OmsType, OrderSide, OrderType, TradingState},
    events::{OrderEventAny, OrderEventType},
    identifiers::{ClientId, ClientOrderId, OrderListId, StrategyId, TraderId},
    instruments::{
        Instrument, InstrumentAny,
        stubs::{audusd_sim, crypto_perpetual_ethusdt},
    },
    orders::{Order, OrderList, OrderTestBuilder},
    stubs::{stub_position_long, stub_position_short},
    types::Quantity,
};
use nautilus_portfolio::Portfolio;
use nautilus_risk::engine::{RiskEngine, config::RiskEngineConfig};
use serde_json::{Value, json};
use ustr::Ustr;

const CASE_IDS: [&str; 3] = [
    "risk.rejection.trading_halted.001",
    "risk.rejection.reducing_buy_order_list_long.001",
    "risk.rejection.reducing_sell_order_list_short.001",
];

#[test]
fn rust_risk_engine_replays_rejection_golden_trace() {
    for case_id in CASE_IDS {
        let case = load_case(case_id);
        let input_event = case
            .get("input")
            .and_then(|value| value.get("events"))
            .and_then(Value::as_array)
            .and_then(|events| events.first())
            .expect("input.events must contain an event");
        let expected = case
            .get("expected")
            .and_then(|value| value.get("events"))
            .and_then(Value::as_array)
            .expect("expected.events must be an array");

        let actual = match string_field(input_event, "event_type") {
            "risk.command.submit_order" => run_risk_rejection_replay(input_event),
            "risk.command.submit_order_list" => {
                run_reducing_order_list_rejection_replay(input_event)
            }
            event_type => panic!("unsupported risk input event {event_type}"),
        };

        assert_eq!(
            actual, *expected,
            "Rust RiskEngine rejection path must match golden trace {case_id}"
        );
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/risk")
}

fn load_case(case_id: &str) -> Value {
    let trace = repository_root().join("tests/golden/risk_rejection_schema.jsonl");
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

fn run_risk_rejection_replay(input_event: &Value) -> Vec<Value> {
    let process_handler = register_process_handler();
    let execute_handler = register_execute_handler();
    let instrument = InstrumentAny::CryptoPerpetual(crypto_perpetual_ethusdt());
    assert_eq!(
        string_field(input_event, "instrument_id"),
        instrument.id().to_string()
    );

    let mut cache = Cache::new(None, None);
    cache.add_instrument(instrument.clone()).unwrap();

    let payload = payload(input_event);
    let order = OrderTestBuilder::new(order_type(string_field(payload, "order_type")))
        .instrument_id(instrument.id())
        .client_order_id(ClientOrderId::from(string_field(
            payload,
            "client_order_id",
        )))
        .side(order_side(string_field(payload, "side")))
        .quantity(Quantity::from(string_field(payload, "quantity")))
        .build();
    cache
        .add_order(order.clone(), None, Some(ClientId::from("BINANCE")), false)
        .unwrap();

    let cache = Rc::new(RefCell::new(cache));
    let clock = Rc::new(RefCell::new(TestClock::new()));
    let portfolio = Portfolio::new(cache.clone(), clock.clone(), None);
    let mut risk_engine = RiskEngine::new(
        RiskEngineConfig {
            debug: true,
            bypass: false,
            max_order_submit: RateLimit::new(10, 1_000),
            max_order_modify: RateLimit::new(5, 1_000),
            max_notional_per_order: AHashMap::new(),
        },
        portfolio,
        clock,
        cache,
    );
    risk_engine.set_trading_state(trading_state(string_field(payload, "trading_state")));

    let submit_order = SubmitOrder::new(
        TraderId::from("TRADER-001"),
        Some(ClientId::from("BINANCE")),
        StrategyId::from("S-001"),
        order.instrument_id(),
        order.client_order_id(),
        order.init_event().clone(),
        None,
        None,
        None,
        UUID4::new(),
        risk_engine.clock().borrow().timestamp_ns(),
        None,
    );

    risk_engine.execute(TradingCommand::SubmitOrder(submit_order));

    let denied_events = process_handler.get_messages();
    let forwarded_commands = execute_handler.get_messages();
    assert_eq!(
        denied_events.len(),
        1,
        "halted trading should emit one denial"
    );
    assert_eq!(
        forwarded_commands.len(),
        0,
        "halted trading should not forward the command to execution"
    );

    vec![risk_denied_event(
        &denied_events[0],
        input_event,
        forwarded_commands.is_empty(),
    )]
}

fn run_reducing_order_list_rejection_replay(input_event: &Value) -> Vec<Value> {
    let process_handler = register_process_handler();
    let execute_handler = register_execute_handler();
    let instrument = InstrumentAny::CurrencyPair(audusd_sim());
    assert_eq!(
        string_field(input_event, "instrument_id"),
        instrument.id().to_string()
    );

    let payload = payload(input_event);
    let position = match string_field(payload, "position_side") {
        "long" => stub_position_long(audusd_sim()),
        "short" => stub_position_short(audusd_sim()),
        side => panic!("unsupported position side {side}"),
    };
    assert_eq!(
        position.quantity.to_string(),
        string_field(payload, "position_quantity")
    );

    let mut cache = Cache::new(None, None);
    cache.add_instrument(instrument.clone()).unwrap();
    cache.add_position(&position, OmsType::Hedging).unwrap();

    let client_order_ids = string_array_field(payload, "client_order_ids");
    let orders = client_order_ids
        .iter()
        .map(|client_order_id| {
            OrderTestBuilder::new(order_type(string_field(payload, "order_type")))
                .instrument_id(instrument.id())
                .client_order_id(ClientOrderId::from(client_order_id.as_str()))
                .side(order_side(string_field(payload, "side")))
                .quantity(Quantity::from(string_field(payload, "quantity")))
                .build()
        })
        .collect::<Vec<_>>();
    for order in &orders {
        cache
            .add_order(order.clone(), None, Some(ClientId::from("SIM")), true)
            .unwrap();
    }

    let cache = Rc::new(RefCell::new(cache));
    let clock = Rc::new(RefCell::new(TestClock::new()));
    let portfolio = Portfolio::new(cache.clone(), clock.clone(), None);
    let mut risk_engine = RiskEngine::new(
        RiskEngineConfig {
            debug: true,
            bypass: false,
            max_order_submit: RateLimit::new(10, 1_000),
            max_order_modify: RateLimit::new(5, 1_000),
            max_notional_per_order: AHashMap::new(),
        },
        portfolio,
        clock,
        cache,
    );
    risk_engine.portfolio_mut().initialize_positions();
    risk_engine.set_trading_state(trading_state(string_field(payload, "trading_state")));

    let order_list = OrderList::new(
        OrderListId::new(string_field(payload, "order_list_id")),
        instrument.id(),
        StrategyId::from("S-001"),
        orders.iter().map(|order| order.client_order_id()).collect(),
        risk_engine.clock().borrow().timestamp_ns(),
    );
    let submit_order_list = SubmitOrderList::new(
        TraderId::from("TRADER-001"),
        Some(ClientId::from("SIM")),
        StrategyId::from("S-001"),
        order_list,
        orders
            .iter()
            .map(|order| order.init_event().clone())
            .collect(),
        None,
        None,
        None,
        UUID4::new(),
        risk_engine.clock().borrow().timestamp_ns(),
        None,
    );

    risk_engine.execute(TradingCommand::SubmitOrderList(submit_order_list));

    let denied_events = process_handler.get_messages();
    let forwarded_commands = execute_handler.get_messages();
    assert_eq!(
        denied_events.len(),
        orders.len(),
        "every exposure-increasing order-list member must be denied: orders={orders:?}, denied={denied_events:?}, forwarded={forwarded_commands:?}"
    );
    assert!(
        forwarded_commands.is_empty(),
        "rejected order-list must not be forwarded to execution"
    );

    denied_events
        .iter()
        .map(|event| risk_denied_event(event, input_event, true))
        .collect()
}

fn register_process_handler() -> TypedIntoMessageSavingHandler<OrderEventAny> {
    let (handler, saving_handler) = get_typed_into_message_saving_handler::<OrderEventAny>(Some(
        Ustr::from("DRG-009.exec_engine_process"),
    ));
    msgbus::register_order_event_endpoint(MessagingSwitchboard::exec_engine_process(), handler);
    saving_handler
}

fn register_execute_handler() -> TypedIntoMessageSavingHandler<TradingCommand> {
    let (handler, saving_handler) = get_typed_into_message_saving_handler::<TradingCommand>(Some(
        Ustr::from("DRG-009.exec_engine_queue_execute"),
    ));
    msgbus::register_trading_command_endpoint(
        MessagingSwitchboard::exec_engine_queue_execute(),
        handler,
    );
    saving_handler
}

fn risk_denied_event(event: &OrderEventAny, source: &Value, no_forward: bool) -> Value {
    assert_eq!(event.event_type(), OrderEventType::Denied);
    let denied = match event {
        OrderEventAny::Denied(denied) => denied,
        other => panic!("expected OrderDenied event, got {other:?}"),
    };
    json!({
        "event_type": "risk.order_denied",
        "ts_event": denied.ts_event.to_string(),
        "ts_init": denied.ts_init.to_string(),
        "instrument_id": denied.instrument_id.to_string(),
        "venue": denied.instrument_id.venue.to_string(),
        "correlation_id": string_field(source, "correlation_id"),
        "payload": {
            "client_order_id": denied.client_order_id.to_string(),
            "reason": denied.reason.to_string(),
            "order_status": "denied",
            "forwarded_to_execution": (!no_forward).to_string(),
        }
    })
}

fn order_type(value: &str) -> OrderType {
    match value {
        "market" => OrderType::Market,
        "limit" => OrderType::Limit,
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

fn trading_state(value: &str) -> TradingState {
    match value {
        "halted" => TradingState::Halted,
        "active" => TradingState::Active,
        "reducing" => TradingState::Reducing,
        state => panic!("unsupported trading state {state}"),
    }
}

fn payload(event: &Value) -> &Value {
    event.get("payload").expect("event payload is required")
}

fn string_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a string"))
}

fn string_array_field(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{key} must be an array"))
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("{key} entries must be strings"))
                .to_string()
        })
        .collect()
}
