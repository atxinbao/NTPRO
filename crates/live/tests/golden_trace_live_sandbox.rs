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
    sync::{Arc, Mutex},
    time::Duration,
};

use nautilus_common::enums::Environment;
use nautilus_live::{
    builder::LiveNodeBuilder,
    config::{LiveExecEngineConfig, LiveNodeConfig},
    node::NodeState,
};
use nautilus_model::identifiers::TraderId;
use serde_json::{Value, json};

const CASE_ID: &str = "backtest_live.sandbox_lifecycle_stop.001";

#[tokio::test(flavor = "current_thread")]
async fn rust_sandbox_live_node_replays_lifecycle_golden_trace() {
    let case = load_case(CASE_ID);
    let build_event = event_by_type(&case, "input", "live.node.build");
    let expected = case
        .get("expected")
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .expect("expected.events must be an array");

    let actual = run_sandbox_lifecycle(build_event).await;

    assert_eq!(
        actual, *expected,
        "Rust sandbox LiveNode lifecycle must match the golden trace"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/live")
}

fn load_case(case_id: &str) -> Value {
    let trace = repository_root().join("tests/golden/live_sandbox_lifecycle_schema.jsonl");
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

async fn run_sandbox_lifecycle(build_event: &Value) -> Vec<Value> {
    let payload = build_event
        .get("payload")
        .expect("build event payload is required");
    let node_name = string_field(payload, "node_name");
    let trader_id = TraderId::from(string_field(payload, "trader_id"));
    assert_eq!(string_field(payload, "environment"), "Sandbox");

    let config = LiveNodeConfig {
        environment: Environment::Sandbox,
        trader_id,
        exec_engine: LiveExecEngineConfig {
            reconciliation: false,
            ..Default::default()
        },
        delay_post_stop: Duration::from_millis(10),
        ..Default::default()
    };
    let mut node = LiveNodeBuilder::from_config(config)
        .unwrap()
        .with_name(node_name)
        .build()
        .unwrap();
    let handle = node.handle();
    let observed = Arc::new(Mutex::new(vec![state_event(
        node_name,
        Environment::Sandbox,
        NodeState::Idle,
        0,
    )]));
    let stop_observed = Arc::clone(&observed);
    let stop_handle = handle.clone();
    let stop_node_name = node_name.to_string();

    tokio::spawn(async move {
        for _ in 0..500 {
            if stop_handle.is_running() {
                stop_observed.lock().unwrap().push(state_event(
                    &stop_node_name,
                    Environment::Sandbox,
                    NodeState::Running,
                    1,
                ));
                stop_handle.stop();
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        stop_handle.stop();
    });

    tokio::time::timeout(Duration::from_secs(5), node.run())
        .await
        .expect("sandbox lifecycle run should finish before timeout")
        .expect("sandbox lifecycle run should succeed");

    observed.lock().unwrap().push(state_event(
        node_name,
        Environment::Sandbox,
        NodeState::Stopped,
        2,
    ));

    Arc::try_unwrap(observed)
        .expect("lifecycle observations should not be shared after run")
        .into_inner()
        .expect("lifecycle observations mutex should not be poisoned")
}

fn state_event(
    node_name: &str,
    environment: Environment,
    state: NodeState,
    sequence: u64,
) -> Value {
    json!({
        "event_type": "live.node.state",
        "ts_event": sequence.to_string(),
        "ts_init": sequence.to_string(),
        "actor_id": node_name,
        "payload": {
            "environment": format!("{environment:?}"),
            "state": format!("{state:?}"),
        }
    })
}

fn string_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a string"))
}
