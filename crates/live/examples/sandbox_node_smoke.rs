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

//! Rust sandbox node smoke for the Rust-first product surface.
//!
//! Run with:
//! `cargo run -p nautilus-live --no-default-features --features node --example sandbox-node-smoke`

use nautilus_common::enums::Environment;
use nautilus_live::node::{LiveNode, NodeState};
use nautilus_model::identifiers::TraderId;

fn main() -> anyhow::Result<()> {
    let trader_id = TraderId::from("SANDBOX-SMOKE-001");
    let node = LiveNode::builder(trader_id, Environment::Sandbox)?
        .with_name("SandboxNodeSmoke")
        .with_load_state(false)
        .with_save_state(false)
        .build()?;

    assert_eq!(node.environment(), Environment::Sandbox);
    assert_eq!(node.state(), NodeState::Idle);
    assert!(!node.is_running());

    println!("node_name=SandboxNodeSmoke");
    println!("trader_id={}", node.trader_id());
    println!("environment={:?}", node.environment());
    println!("state={:?}", node.state());
    println!("running={}", node.is_running());
    println!("python_required=false");
    println!("external_venue_connection=false");

    Ok(())
}
