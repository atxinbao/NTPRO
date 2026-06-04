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

//! Rust live-node initialization and shutdown smoke for RHARD-005.
//!
//! Run with:
//! `cargo run -p nautilus-live --no-default-features --features node --example live-init-smoke`

use nautilus_common::enums::Environment;
use nautilus_live::node::{LiveNode, NodeState};
use nautilus_model::{
    identifiers::{AccountId, ClientId, TraderId, Venue},
    types::{Currency, Money},
};
use nautilus_sandbox::{SandboxExecutionClientConfig, SandboxExecutionClientFactory};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let trader_id = TraderId::from("LIVE-INIT-001");
    let account_id = AccountId::from("SANDBOX-001");
    let venue = Venue::from("SANDBOX");
    let client_id = ClientId::from("SANDBOX");
    let config = SandboxExecutionClientConfig {
        trader_id,
        account_id,
        venue,
        starting_balances: vec![Money::new(100_000.0, Currency::USDT())],
        ..Default::default()
    };

    let mut node = LiveNode::builder(trader_id, Environment::Sandbox)?
        .with_name("LiveInitSmoke")
        .with_reconciliation(false)
        .with_load_state(false)
        .with_save_state(false)
        .with_timeout_connection(5)
        .with_timeout_disconnection_secs(5)
        .with_delay_post_stop_secs(0)
        .add_simulated_exec_client(
            None,
            Box::new(SandboxExecutionClientFactory::new()),
            Box::new(config),
        )?
        .build()?;
    let handle = node.handle();

    assert_eq!(node.environment(), Environment::Sandbox);
    assert_eq!(handle.state(), NodeState::Idle);
    assert!(
        node.kernel()
            .exec_engine
            .borrow()
            .client_ids()
            .contains(&client_id)
    );
    assert!(node.kernel().exec_engine.borrow().check_disconnected());

    println!("phase=build_node status=ok node_name=LiveInitSmoke");
    println!("phase=register_adapter status=ok adapter=sandbox client_id={client_id}");
    println!(
        "phase=pre_start state={:?} exec_connected={} real_orders_submitted=false external_venue_connection=false",
        handle.state(),
        node.kernel().exec_engine.borrow().check_connected(),
    );

    node.start().await?;

    let account_cached = node
        .kernel()
        .cache
        .borrow()
        .account_owned(&account_id)
        .is_some();
    println!(
        "phase=start status=ok state={:?} running={} exec_connected={} account_cached={account_cached}",
        handle.state(),
        handle.is_running(),
        node.kernel().exec_engine.borrow().check_connected(),
    );
    assert_eq!(handle.state(), NodeState::Running);
    assert!(handle.is_running());
    assert!(node.kernel().exec_engine.borrow().check_connected());
    assert!(account_cached);

    node.stop().await?;

    println!(
        "phase=stop status=ok state={:?} running={} exec_disconnected={} real_orders_submitted=false external_venue_connection=false rust_only_runtime=true",
        handle.state(),
        handle.is_running(),
        node.kernel().exec_engine.borrow().check_disconnected(),
    );
    assert_eq!(handle.state(), NodeState::Stopped);
    assert!(!handle.is_running());
    assert!(node.kernel().exec_engine.borrow().check_disconnected());

    Ok(())
}
