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

//! Integration tests for LiveNode lifecycle and handle control.
//!
//! These tests use global logging state (one logger per process).
//! Run with cargo-nextest for process isolation, or use --test-threads=1.

use std::{any::Any, cell::RefCell, fmt::Debug, rc::Rc, time::Duration};

use nautilus_common::{
    actor::{DataActor, DataActorCore, data_actor::DataActorConfig},
    cache::CacheView,
    clients::{DataClient, ExecutionClient},
    clock::Clock,
    enums::Environment,
    factories::{ClientConfig, DataClientFactory, ExecutionClientFactory},
    messages::system::ShutdownSystem,
    msgbus::{self, MessagingSwitchboard},
    nautilus_actor,
    testing::wait_until_async,
};
use nautilus_core::{UUID4, UnixNanos};
use nautilus_live::{
    config::{LiveExecEngineConfig, LiveNodeConfig},
    node::{LiveNode, LiveNodeHandle, NodeState},
};
use nautilus_model::{
    accounts::AccountAny,
    enums::OmsType,
    identifiers::{AccountId, ClientId, ExecAlgorithmId, TraderId, Venue},
    orders::OrderAny,
    types::{AccountBalance, MarginBalance},
};
use nautilus_trading::{
    ExecutionAlgorithm, ExecutionAlgorithmConfig, ExecutionAlgorithmCore, nautilus_strategy,
    strategy::{StrategyConfig, StrategyCore},
};
use rstest::rstest;

#[derive(Debug)]
struct TestActor {
    core: DataActorCore,
}

impl TestActor {
    fn new(config: DataActorConfig) -> Self {
        Self {
            core: DataActorCore::new(config),
        }
    }
}

impl DataActor for TestActor {}

nautilus_actor!(TestActor);

#[derive(Debug)]
struct TestStrategy {
    core: StrategyCore,
}

impl TestStrategy {
    fn new(config: StrategyConfig) -> Self {
        Self {
            core: StrategyCore::new(config),
        }
    }
}

impl DataActor for TestStrategy {}

nautilus_strategy!(TestStrategy);

#[derive(Debug)]
struct TestExecAlgorithm {
    core: ExecutionAlgorithmCore,
}

impl TestExecAlgorithm {
    fn new(config: ExecutionAlgorithmConfig) -> Self {
        Self {
            core: ExecutionAlgorithmCore::new(config),
        }
    }
}

impl DataActor for TestExecAlgorithm {}

nautilus_actor!(TestExecAlgorithm);

impl ExecutionAlgorithm for TestExecAlgorithm {
    fn core_mut(&mut self) -> &mut ExecutionAlgorithmCore {
        &mut self.core
    }

    fn on_order(&mut self, _order: OrderAny) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct TestClientConfig;

impl ClientConfig for TestClientConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
struct TestDataClient {
    client_id: ClientId,
    venue: Venue,
    connected: bool,
}

impl TestDataClient {
    fn new(client_id: ClientId, venue: Venue) -> Self {
        Self {
            client_id,
            venue,
            connected: false,
        }
    }
}

impl DataClient for TestDataClient {
    fn client_id(&self) -> ClientId {
        self.client_id
    }

    fn venue(&self) -> Option<Venue> {
        Some(self.venue)
    }

    fn start(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn reset(&mut self) -> anyhow::Result<()> {
        self.connected = false;
        Ok(())
    }

    fn dispose(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn is_disconnected(&self) -> bool {
        !self.connected
    }
}

#[derive(Debug)]
struct TestDataClientFactory;

impl DataClientFactory for TestDataClientFactory {
    fn create(
        &self,
        name: &str,
        config: &dyn ClientConfig,
        _cache: CacheView,
        _clock: Rc<RefCell<dyn Clock>>,
    ) -> anyhow::Result<Box<dyn DataClient>> {
        if config.as_any().downcast_ref::<TestClientConfig>().is_none() {
            anyhow::bail!("invalid test data client config");
        }

        Ok(Box::new(TestDataClient::new(
            ClientId::from(name),
            Venue::from("SIM"),
        )))
    }

    fn name(&self) -> &'static str {
        "TEST-DATA"
    }

    fn config_type(&self) -> &'static str {
        "TestClientConfig"
    }
}

#[derive(Debug)]
struct TestExecutionClient {
    client_id: ClientId,
    account_id: AccountId,
    venue: Venue,
    connected: bool,
}

impl TestExecutionClient {
    fn new(client_id: ClientId, venue: Venue) -> Self {
        Self {
            client_id,
            account_id: AccountId::from("SIM-001"),
            venue,
            connected: false,
        }
    }
}

impl ExecutionClient for TestExecutionClient {
    fn is_connected(&self) -> bool {
        self.connected
    }

    fn client_id(&self) -> ClientId {
        self.client_id
    }

    fn account_id(&self) -> AccountId {
        self.account_id
    }

    fn venue(&self) -> Venue {
        self.venue
    }

    fn oms_type(&self) -> OmsType {
        OmsType::Netting
    }

    fn get_account(&self) -> Option<AccountAny> {
        None
    }

    fn generate_account_state(
        &self,
        _balances: Vec<AccountBalance>,
        _margins: Vec<MarginBalance>,
        _reported: bool,
        _ts_event: UnixNanos,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn start(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct TestExecutionClientFactory;

impl ExecutionClientFactory for TestExecutionClientFactory {
    fn create(
        &self,
        name: &str,
        config: &dyn ClientConfig,
        _cache: CacheView,
    ) -> anyhow::Result<Box<dyn ExecutionClient>> {
        if config.as_any().downcast_ref::<TestClientConfig>().is_none() {
            anyhow::bail!("invalid test execution client config");
        }

        Ok(Box::new(TestExecutionClient::new(
            ClientId::from(name),
            Venue::from("SIM"),
        )))
    }

    fn name(&self) -> &'static str {
        "TEST-EXEC"
    }

    fn config_type(&self) -> &'static str {
        "TestClientConfig"
    }
}

#[rstest]
fn test_handle_initial_state() {
    let handle = LiveNodeHandle::new();

    assert_eq!(handle.state(), NodeState::Idle);
    assert!(!handle.should_stop());
    assert!(!handle.is_running());
}

#[rstest]
fn test_handle_stop_sets_flag() {
    let handle = LiveNodeHandle::new();

    handle.stop();

    assert!(handle.should_stop());
}

#[rstest]
fn test_handle_clone_shares_state() {
    let handle1 = LiveNodeHandle::new();
    let handle2 = handle1.clone();

    handle1.stop();

    assert!(handle2.should_stop());
}

#[rstest]
fn test_node_state_values() {
    assert_eq!(NodeState::Idle.as_u8(), 0);
    assert_eq!(NodeState::Starting.as_u8(), 1);
    assert_eq!(NodeState::Running.as_u8(), 2);
    assert_eq!(NodeState::ShuttingDown.as_u8(), 3);
    assert_eq!(NodeState::Stopped.as_u8(), 4);
}

#[rstest]
fn test_node_state_is_running() {
    assert!(!NodeState::Idle.is_running());
    assert!(!NodeState::Starting.is_running());
    assert!(NodeState::Running.is_running());
    assert!(!NodeState::ShuttingDown.is_running());
    assert!(!NodeState::Stopped.is_running());
}

#[rstest]
fn test_builder_rejects_backtest_environment() {
    let result = LiveNode::builder(TraderId::from("TESTER-001"), Environment::Backtest);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Backtest"),
        "Expected Backtest error, was: {err}"
    );
}

#[rstest]
fn test_builder_accepts_sandbox() {
    let result = LiveNode::builder(TraderId::from("TESTER-001"), Environment::Sandbox);

    assert!(result.is_ok());
}

#[rstest]
fn test_builder_accepts_live() {
    let result = LiveNode::builder(TraderId::from("TESTER-001"), Environment::Live);

    assert!(result.is_ok());
}

// -- LiveNode construction tests (require process isolation via nextest) --------------------------
// These tests initialize global logging state and require isolated processes.
// Run with: cargo nextest run -p nautilus-live --test node

mod serial_tests {
    use super::*;

    #[rstest]
    fn test_live_node_build_with_default_config() {
        let node = LiveNode::build("TestNode".to_string(), None).unwrap();

        assert_eq!(node.state(), NodeState::Idle);
        assert_eq!(node.environment(), Environment::Live);
        assert!(!node.is_running());
    }

    #[rstest]
    fn test_live_node_build_overrides_environment_to_live() {
        let config = LiveNodeConfig {
            environment: Environment::Sandbox,
            trader_id: TraderId::from("TESTER-001"),
            ..Default::default()
        };

        let node = LiveNode::build("TestNode".to_string(), Some(config)).unwrap();

        // Environment is overridden to Live when using build()
        assert_eq!(node.environment(), Environment::Live);
        assert_eq!(node.trader_id(), TraderId::from("TESTER-001"));
    }

    #[rstest]
    fn test_live_node_returns_handle() {
        let node = LiveNode::build("TestNode".to_string(), None).unwrap();
        let handle = node.handle();

        assert_eq!(handle.state(), NodeState::Idle);
        assert!(!handle.should_stop());
    }

    #[rstest]
    fn test_live_node_config_with_disabled_reconciliation() {
        let config = LiveNodeConfig {
            exec_engine: LiveExecEngineConfig {
                reconciliation: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let node = LiveNode::build("TestNode".to_string(), Some(config)).unwrap();

        assert_eq!(node.state(), NodeState::Idle);
    }

    #[rstest]
    fn test_add_actor() {
        let mut node = LiveNode::build("TestNode".to_string(), None).unwrap();

        let actor = TestActor::new(DataActorConfig::default());

        let result = node.add_actor(actor);

        assert!(result.is_ok());
    }

    #[rstest]
    fn test_add_strategy() {
        let mut node = LiveNode::build("TestNode".to_string(), None).unwrap();

        let strategy = TestStrategy::new(StrategyConfig::default());

        let result = node.add_strategy(strategy);

        assert!(result.is_ok());
    }

    #[rstest]
    fn test_add_exec_algorithm() {
        let mut node = LiveNode::build("TestNode".to_string(), None).unwrap();

        let config = ExecutionAlgorithmConfig {
            exec_algorithm_id: Some(ExecAlgorithmId::from("TEST_ALGO")),
            ..Default::default()
        };
        let algo = TestExecAlgorithm::new(config);

        let result = node.add_exec_algorithm(algo);

        assert!(result.is_ok());
    }

    #[rstest]
    fn test_add_exec_algorithm_registers_execute_endpoint() {
        let mut node = LiveNode::build("TestNode".to_string(), None).unwrap();

        let config = ExecutionAlgorithmConfig {
            exec_algorithm_id: Some(ExecAlgorithmId::from("MY_ALGO")),
            ..Default::default()
        };
        let algo = TestExecAlgorithm::new(config);

        node.add_exec_algorithm(algo).unwrap();

        assert!(nautilus_common::msgbus::has_endpoint("MY_ALGO.execute"));
    }

    #[rstest]
    fn test_handle_from_node_shares_state() {
        let node = LiveNode::build("TestNode".to_string(), None).unwrap();
        let handle = node.handle();

        handle.stop();

        assert!(handle.should_stop());
    }

    #[rstest]
    fn test_node_starts_in_idle_state() {
        let node = LiveNode::build("TestNode".to_string(), None).unwrap();

        assert_eq!(node.state(), NodeState::Idle);
    }

    #[rstest]
    fn test_kernel_access() {
        let node = LiveNode::build("TestNode".to_string(), None).unwrap();

        let kernel = node.kernel();

        assert_eq!(kernel.trader_id(), TraderId::from("TRADER-001"));
    }

    #[rstest]
    fn test_exec_manager_access() {
        let node = LiveNode::build("TestNode".to_string(), None).unwrap();

        let _manager = node.exec_manager();
    }

    #[rstest]
    fn test_builder_registers_rust_data_and_exec_client_factories() {
        let node = LiveNode::builder(TraderId::from("TESTER-001"), Environment::Sandbox)
            .unwrap()
            .with_name("RustClientRegistration")
            .add_data_client(
                None,
                Box::new(TestDataClientFactory),
                Box::new(TestClientConfig),
            )
            .unwrap()
            .add_exec_client(
                None,
                Box::new(TestExecutionClientFactory),
                Box::new(TestClientConfig),
            )
            .unwrap()
            .build()
            .unwrap();

        let registered_data_clients = node.kernel().data_engine.borrow().registered_clients();
        let registered_exec_clients = node.kernel().exec_engine.borrow().client_ids();

        assert!(registered_data_clients.contains(&ClientId::from("TEST-DATA")));
        assert!(registered_exec_clients.contains(&ClientId::from("TEST-EXEC")));
    }

    #[rstest]
    #[tokio::test]
    async fn test_stop_when_not_running_returns_error() {
        let mut node = LiveNode::build("TestNode".to_string(), None).unwrap();

        let result = node.stop().await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not running"));
    }

    #[rstest]
    #[tokio::test(flavor = "current_thread")]
    async fn test_rust_builder_start_stop_smoke_without_python() {
        let mut node = LiveNode::builder(TraderId::from("TESTER-001"), Environment::Sandbox)
            .unwrap()
            .with_name("RustLifecycleSmoke")
            .with_reconciliation(false)
            .with_delay_post_stop_secs(0)
            .build()
            .unwrap();
        let handle = node.handle();

        assert_eq!(node.environment(), Environment::Sandbox);
        assert_eq!(handle.state(), NodeState::Idle);

        tokio::time::timeout(Duration::from_secs(5), node.start())
            .await
            .expect("Rust LiveNode start should complete before timeout")
            .expect("Rust LiveNode start should succeed");

        assert_eq!(handle.state(), NodeState::Running);
        assert!(handle.is_running());

        tokio::time::timeout(Duration::from_secs(5), node.stop())
            .await
            .expect("Rust LiveNode stop should complete before timeout")
            .expect("Rust LiveNode stop should succeed");

        assert_eq!(handle.state(), NodeState::Stopped);
        assert!(!handle.is_running());
    }

    #[rstest]
    #[tokio::test]
    async fn test_run_twice_returns_error() {
        let config = LiveNodeConfig {
            exec_engine: LiveExecEngineConfig {
                reconciliation: false,
                ..Default::default()
            },
            delay_post_stop: Duration::from_millis(50),
            ..Default::default()
        };
        let mut node = LiveNode::build("TestNode".to_string(), Some(config)).unwrap();
        let handle = node.handle();

        // Must stop after node enters Running (stop flag is cleared on Running transition)
        let stop_handle = handle.clone();

        tokio::spawn(async move {
            wait_until_async(
                || async { stop_handle.is_running() },
                Duration::from_secs(5),
            )
            .await;
            stop_handle.stop();
        });

        // First run - completes and consumes the runner
        let _ = node.run().await;

        // Second run - should fail because runner is consumed
        let result = node.run().await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Runner already consumed")
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_handle_stop_triggers_graceful_shutdown() {
        let config = LiveNodeConfig {
            exec_engine: LiveExecEngineConfig {
                reconciliation: false,
                ..Default::default()
            },
            delay_post_stop: Duration::from_millis(50),
            ..Default::default()
        };
        let mut node = LiveNode::build("TestNode".to_string(), Some(config)).unwrap();
        let handle = node.handle();

        assert_eq!(handle.state(), NodeState::Idle);

        // Spawn task to stop after node enters Running state
        let stop_handle = handle.clone();

        tokio::spawn(async move {
            wait_until_async(
                || async { stop_handle.is_running() },
                Duration::from_secs(5),
            )
            .await;
            stop_handle.stop();
        });

        // With no clients, run() completes startup immediately and waits for stop signal
        let result = node.run().await;

        assert!(result.is_ok());
        assert_eq!(handle.state(), NodeState::Stopped);
    }

    #[rstest]
    #[tokio::test(flavor = "current_thread")]
    async fn test_shutdown_system_triggers_graceful_shutdown() {
        let config = LiveNodeConfig {
            exec_engine: LiveExecEngineConfig {
                reconciliation: false,
                ..Default::default()
            },
            delay_post_stop: Duration::from_millis(50),
            ..Default::default()
        };
        let mut node = LiveNode::build("TestNode".to_string(), Some(config)).unwrap();
        let handle = node.handle();
        let trader_id = node.kernel().trader_id();
        let ts = node.kernel().generate_timestamp_ns();

        // Publish ShutdownSystem once the node reaches Running. msgbus uses
        // thread-local storage, so the publish must happen on the same thread
        // as node.run(). The test runtime is pinned to current_thread above
        // so tokio::spawn stays on this thread.
        let state_handle = handle.clone();

        tokio::spawn(async move {
            wait_until_async(
                || async { state_handle.is_running() },
                Duration::from_secs(5),
            )
            .await;
            let command = ShutdownSystem::new(
                trader_id,
                ustr::Ustr::from("TestComponent"),
                Some("integration test".to_string()),
                UUID4::new(),
                ts,
                None, // correlation_id
            );
            msgbus::publish_any(
                MessagingSwitchboard::shutdown_system_topic(),
                command.as_any(),
            );
        });

        let result = node.run().await;

        assert!(result.is_ok());
        assert_eq!(handle.state(), NodeState::Stopped);
    }

    #[rstest]
    #[tokio::test]
    async fn test_handle_stop_completes_within_timeout() {
        let config = LiveNodeConfig {
            exec_engine: LiveExecEngineConfig {
                reconciliation: false,
                ..Default::default()
            },
            delay_post_stop: Duration::from_millis(50),
            ..Default::default()
        };
        let mut node = LiveNode::build("TestNode".to_string(), Some(config)).unwrap();
        let handle = node.handle();

        let stop_handle = handle.clone();

        tokio::spawn(async move {
            wait_until_async(
                || async { stop_handle.is_running() },
                Duration::from_secs(5),
            )
            .await;
            stop_handle.stop();
        });

        // The biased select in the event loop prioritizes signals over data,
        // so stop should complete well within 5 seconds even under load
        let result = tokio::time::timeout(Duration::from_secs(5), node.run()).await;

        assert!(
            result.is_ok(),
            "run() should complete within 5 seconds after stop"
        );
        assert_eq!(handle.state(), NodeState::Stopped);
    }

    // The maintenance dispatcher is a single `select!` arm in `LiveNode::run`
    // that fires up to six periodic tasks. With reconciliation disabled, the
    // only sub-second-cadenced task that can fire in a short test window is
    // the own-books audit (interval is `Option<f64>` seconds). Configuring it
    // at 0.1s and holding the node Running for ~250ms guarantees the
    // maintenance arm is polled multiple times and dispatches at least one
    // body. If the dispatcher panics, deadlocks the cache `borrow_mut()`, or
    // otherwise breaks the loop, `run()` will not return cleanly.
    #[rstest]
    #[tokio::test(flavor = "current_thread")]
    async fn test_maintenance_dispatcher_runs_while_running() {
        let config = LiveNodeConfig {
            exec_engine: LiveExecEngineConfig {
                reconciliation: false,
                own_books_audit_interval_secs: Some(0.1),
                ..Default::default()
            },
            delay_post_stop: Duration::from_millis(50),
            ..Default::default()
        };
        let mut node = LiveNode::build("MaintenanceTestNode".to_string(), Some(config)).unwrap();
        let handle = node.handle();

        let stop_handle = handle.clone();

        tokio::spawn(async move {
            wait_until_async(
                || async { stop_handle.is_running() },
                Duration::from_secs(5),
            )
            .await;
            tokio::time::sleep(Duration::from_millis(250)).await;
            stop_handle.stop();
        });

        let result = tokio::time::timeout(Duration::from_secs(5), node.run()).await;

        assert!(result.is_ok(), "run() should complete within timeout");
        assert!(
            result.unwrap().is_ok(),
            "run() should succeed after maintenance dispatcher fires"
        );
        assert_eq!(handle.state(), NodeState::Stopped);
    }
}
