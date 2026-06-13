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

use std::{cell::RefCell, rc::Rc};

use ahash::AHashMap;
use nautilus_common::{
    cache::Cache,
    clock::TestClock,
    messages::execution::{SubmitOrder, TradingCommand},
    msgbus::{
        self, MessagingSwitchboard,
        stubs::{TypedIntoMessageSavingHandler, get_typed_into_message_saving_handler},
    },
    throttler::RateLimit,
};
use nautilus_core::UUID4;
use nautilus_model::{
    enums::{OrderSide, OrderType, TradingState},
    events::OrderEventAny,
    identifiers::{ClientId, ClientOrderId, StrategyId, TraderId},
    instruments::{Instrument, InstrumentAny, stubs::currency_pair_btcusdt},
    orders::{Order, OrderTestBuilder},
    types::Quantity,
};
use nautilus_portfolio::Portfolio;
use nautilus_risk::{
    RiskEngine,
    engine::config::RiskEngineConfig,
    v04_rejection::{
        V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID, V04_BINANCE_RISK_REJECTION_REASON,
        V04_BINANCE_RISK_REJECTION_SMOKE_ID, v04_binance_risk_rejection_summary,
    },
};
use ustr::Ustr;

#[test]
fn v04_binance_sandbox_risk_rejection_smoke_is_deterministic() {
    let process_handler = register_process_handler();
    let execute_handler = register_execute_handler();
    let instrument = InstrumentAny::CurrencyPair(currency_pair_btcusdt());
    let mut risk_engine = risk_engine_with_instrument(instrument.clone());
    let order = OrderTestBuilder::new(OrderType::Market)
        .instrument_id(instrument.id())
        .client_order_id(ClientOrderId::from(
            V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID,
        ))
        .side(OrderSide::Buy)
        .quantity(Quantity::from("0.030"))
        .build();

    risk_engine
        .cache()
        .borrow_mut()
        .add_order(order.clone(), None, Some(ClientId::from("BINANCE")), false)
        .unwrap();

    let submit_order = SubmitOrder::new(
        TraderId::from("TRADER-001"),
        Some(ClientId::from("BINANCE")),
        StrategyId::from("S-V04-001"),
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

    risk_engine.set_trading_state(TradingState::Halted);
    risk_engine.execute(TradingCommand::SubmitOrder(submit_order));

    let denied_events = process_handler.get_messages();
    let forwarded_commands = execute_handler.get_messages();
    assert_eq!(denied_events.len(), 1);
    assert_eq!(forwarded_commands.len(), 0);

    let summary =
        v04_binance_risk_rejection_summary(&denied_events[0], !forwarded_commands.is_empty())
            .unwrap();
    assert_eq!(summary.smoke_id, V04_BINANCE_RISK_REJECTION_SMOKE_ID);
    assert_eq!(summary.lifecycle_id, "v04-binance-mock-order-lifecycle");
    assert_eq!(summary.instrument_id, "BTCUSDT.BINANCE");
    assert_eq!(
        summary.client_order_id,
        V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID
    );
    assert_eq!(summary.fixture_reason, "mock_reject_requested");
    assert_eq!(summary.risk_reason, V04_BINANCE_RISK_REJECTION_REASON);
    assert_eq!(summary.order_status, "denied");
    assert!(!summary.forwarded_to_execution);
    assert!(!summary.external_adapter);
    assert!(!summary.real_exchange_connection);
    assert!(!summary.real_orders_submitted);
    assert_eq!(summary.checksum, "60b0dc50f47caea8");

    let artifact = summary.summary_artifact();
    assert!(artifact.contains("command=binance.risk_rejection_smoke"));
    assert!(artifact.contains("status=ok"));
    assert!(artifact.contains("risk_reason=TradingState::HALTED"));
    assert!(artifact.contains("order_status=denied"));
    assert!(artifact.contains("forwarded_to_execution=false"));
    assert!(artifact.contains("runtime_status=risk_rejection_smoke_ready"));
}

fn risk_engine_with_instrument(instrument: InstrumentAny) -> RiskEngine {
    let mut cache = Cache::new(None, None);
    cache.add_instrument(instrument).unwrap();

    let cache = Rc::new(RefCell::new(cache));
    let clock = Rc::new(RefCell::new(TestClock::new()));
    let portfolio = Portfolio::new(cache.clone(), clock.clone(), None);

    RiskEngine::new(
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
    )
}

fn register_process_handler() -> TypedIntoMessageSavingHandler<OrderEventAny> {
    let (handler, saving_handler) = get_typed_into_message_saving_handler::<OrderEventAny>(Some(
        Ustr::from("V04-009.exec_engine_process"),
    ));
    msgbus::register_order_event_endpoint(MessagingSwitchboard::exec_engine_process(), handler);
    saving_handler
}

fn register_execute_handler() -> TypedIntoMessageSavingHandler<TradingCommand> {
    let (handler, saving_handler) = get_typed_into_message_saving_handler::<TradingCommand>(Some(
        Ustr::from("V04-009.exec_engine_queue_execute"),
    ));
    msgbus::register_trading_command_endpoint(
        MessagingSwitchboard::exec_engine_queue_execute(),
        handler,
    );
    saving_handler
}
