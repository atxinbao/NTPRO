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
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Map, Value, json};

const REQUIRED_CASES: &[ReplayCase] = &[
    ReplayCase {
        trace: "read_model_contract_schema.jsonl",
        case_id: "read_model.contract.healthy_minimal.001",
        family: "contract",
    },
    ReplayCase {
        trace: "read_model_contract_schema.jsonl",
        case_id: "read_model.contract.fail_closed_missing_lineage_source_freshness.001",
        family: "contract",
    },
    ReplayCase {
        trace: "read_model_account_snapshot_schema.jsonl",
        case_id: "read_model.account_snapshot.fresh.001",
        family: "account",
    },
    ReplayCase {
        trace: "read_model_account_snapshot_schema.jsonl",
        case_id: "read_model.account_snapshot.stale.001",
        family: "account",
    },
    ReplayCase {
        trace: "read_model_account_snapshot_schema.jsonl",
        case_id: "read_model.account_snapshot.missing_provenance.001",
        family: "account",
    },
    ReplayCase {
        trace: "read_model_account_snapshot_schema.jsonl",
        case_id: "read_model.account_snapshot.redaction_breach.001",
        family: "account",
    },
    ReplayCase {
        trace: "read_model_position_schema.jsonl",
        case_id: "read_model.position.long.001",
        family: "position",
    },
    ReplayCase {
        trace: "read_model_position_schema.jsonl",
        case_id: "read_model.position.short.001",
        family: "position",
    },
    ReplayCase {
        trace: "read_model_position_schema.jsonl",
        case_id: "read_model.position.flat.001",
        family: "position",
    },
    ReplayCase {
        trace: "read_model_position_schema.jsonl",
        case_id: "read_model.position.precision_mismatch.001",
        family: "position",
    },
    ReplayCase {
        trace: "read_model_position_schema.jsonl",
        case_id: "read_model.position.stale_source.001",
        family: "position",
    },
    ReplayCase {
        trace: "read_model_position_schema.jsonl",
        case_id: "read_model.position.account_mismatch.001",
        family: "position",
    },
    ReplayCase {
        trace: "read_model_order_lifecycle_schema.jsonl",
        case_id: "read_model.order_lifecycle.matched.001",
        family: "order",
    },
    ReplayCase {
        trace: "read_model_order_lifecycle_schema.jsonl",
        case_id: "read_model.order_lifecycle.unknown_response.001",
        family: "order",
    },
    ReplayCase {
        trace: "read_model_order_lifecycle_schema.jsonl",
        case_id: "read_model.order_lifecycle.readback_mismatch.001",
        family: "order",
    },
    ReplayCase {
        trace: "read_model_order_lifecycle_schema.jsonl",
        case_id: "read_model.order_lifecycle.duplicate_attempt.001",
        family: "order",
    },
    ReplayCase {
        trace: "read_model_order_lifecycle_schema.jsonl",
        case_id: "read_model.order_lifecycle.missing_ledger.001",
        family: "order",
    },
    ReplayCase {
        trace: "read_model_fill_execution_schema.jsonl",
        case_id: "read_model.fill_execution.reconciled.001",
        family: "fill",
    },
    ReplayCase {
        trace: "read_model_fill_execution_schema.jsonl",
        case_id: "read_model.fill_execution.partial_fill.001",
        family: "fill",
    },
    ReplayCase {
        trace: "read_model_fill_execution_schema.jsonl",
        case_id: "read_model.fill_execution.duplicate_fill.001",
        family: "fill",
    },
    ReplayCase {
        trace: "read_model_fill_execution_schema.jsonl",
        case_id: "read_model.fill_execution.missing_order_linkage.001",
        family: "fill",
    },
    ReplayCase {
        trace: "read_model_fill_execution_schema.jsonl",
        case_id: "read_model.fill_execution.stale_source.001",
        family: "fill",
    },
    ReplayCase {
        trace: "read_model_fill_execution_schema.jsonl",
        case_id: "read_model.fill_execution.ambiguous_source.001",
        family: "fill",
    },
    ReplayCase {
        trace: "read_model_risk_state_schema.jsonl",
        case_id: "read_model.risk_state.healthy.001",
        family: "risk",
    },
    ReplayCase {
        trace: "read_model_risk_state_schema.jsonl",
        case_id: "read_model.risk_state.risk_visible.001",
        family: "risk",
    },
    ReplayCase {
        trace: "read_model_risk_state_schema.jsonl",
        case_id: "read_model.risk_state.manual_review.001",
        family: "risk",
    },
    ReplayCase {
        trace: "read_model_risk_state_schema.jsonl",
        case_id: "read_model.risk_state.halted.001",
        family: "risk",
    },
    ReplayCase {
        trace: "read_model_risk_state_schema.jsonl",
        case_id: "read_model.risk_state.stale.001",
        family: "risk",
    },
    ReplayCase {
        trace: "read_model_risk_state_schema.jsonl",
        case_id: "read_model.risk_state.mismatch.001",
        family: "risk",
    },
    ReplayCase {
        trace: "read_model_dashboard_schema.jsonl",
        case_id: "read_model.dashboard.readonly_complete.001",
        family: "dashboard",
    },
    ReplayCase {
        trace: "read_model_dashboard_schema.jsonl",
        case_id: "read_model.dashboard.missing_evidence_degraded.001",
        family: "dashboard",
    },
    ReplayCase {
        trace: "read_model_dashboard_schema.jsonl",
        case_id: "read_model.dashboard.forbidden_controls_blocked.001",
        family: "dashboard",
    },
];
const VISIBLE_PANELS: &[&str] = &[
    "accounts",
    "positions",
    "orders",
    "fills",
    "risk",
    "audit_provenance_diagnostics",
];
const DISABLED_CONTROLS: &[&str] = &[
    "submit", "approval", "cancel", "retry", "replace", "amend", "flatten",
];
const CONTRACT_COMPONENTS: &[&str] = &[
    "account",
    "positions",
    "orders",
    "fills",
    "risk",
    "lifecycle_status",
];
const ACCOUNT_PARTITION_COMPONENTS: &[&str] = &[
    "account",
    "positions",
    "orders",
    "fills",
    "risk",
    "alerts",
    "audit",
    "provenance",
];
const STRATEGY_SUPERVISOR_COMPONENTS: &[&str] = &[
    "supervisor_state",
    "runtime_state",
    "risk_state",
    "events",
    "audit",
    "provenance",
];
const VENUE_NODE_LIFECYCLE_COMPONENTS: &[&str] = &[
    "lifecycle_state",
    "adapter_status",
    "connection_state",
    "risk_state",
    "audit",
    "provenance",
];
const ORCHESTRATION_CONTROL_PLANE_COMPONENTS: &[&str] = &[
    "routing",
    "control_intent",
    "owner_approval",
    "risk_gate",
    "audit_gate",
    "provenance",
];
const DASHBOARD_OBSERVABILITY_COMPONENTS: &[&str] = &[
    "account",
    "strategy",
    "venue_node",
    "risk",
    "alerts",
    "audit",
    "provenance",
];
const V230_ACCOUNT_PARTITION_CASES: &[ReplayCase] = &[
    ReplayCase {
        trace: "read_model_account_partition_schema.jsonl",
        case_id: "read_model.account_partition.isolated_accounts.001",
        family: "account_partition",
    },
    ReplayCase {
        trace: "read_model_account_partition_schema.jsonl",
        case_id: "read_model.account_partition.cross_account_mismatch.001",
        family: "account_partition",
    },
    ReplayCase {
        trace: "read_model_account_partition_schema.jsonl",
        case_id: "read_model.account_partition.missing_account_key.001",
        family: "account_partition",
    },
];
const V230_STRATEGY_SUPERVISOR_CASES: &[ReplayCase] = &[
    ReplayCase {
        trace: "read_model_strategy_supervisor_schema.jsonl",
        case_id: "read_model.strategy_supervisor.isolated_strategies.001",
        family: "strategy_supervisor",
    },
    ReplayCase {
        trace: "read_model_strategy_supervisor_schema.jsonl",
        case_id: "read_model.strategy_supervisor.cross_strategy_mismatch.001",
        family: "strategy_supervisor",
    },
    ReplayCase {
        trace: "read_model_strategy_supervisor_schema.jsonl",
        case_id: "read_model.strategy_supervisor.missing_strategy_key.001",
        family: "strategy_supervisor",
    },
];
const V230_VENUE_NODE_LIFECYCLE_CASES: &[ReplayCase] = &[
    ReplayCase {
        trace: "read_model_venue_node_lifecycle_schema.jsonl",
        case_id: "read_model.venue_node_lifecycle.isolated_nodes.001",
        family: "venue_node_lifecycle",
    },
    ReplayCase {
        trace: "read_model_venue_node_lifecycle_schema.jsonl",
        case_id: "read_model.venue_node_lifecycle.cross_node_mismatch.001",
        family: "venue_node_lifecycle",
    },
    ReplayCase {
        trace: "read_model_venue_node_lifecycle_schema.jsonl",
        case_id: "read_model.venue_node_lifecycle.missing_venue_node_key.001",
        family: "venue_node_lifecycle",
    },
];
const V230_ORCHESTRATION_CONTROL_PLANE_CASES: &[ReplayCase] = &[
    ReplayCase {
        trace: "read_model_orchestration_control_plane_schema.jsonl",
        case_id: "read_model.orchestration_control_plane.scoped_intents_ready.001",
        family: "orchestration_control_plane",
    },
    ReplayCase {
        trace: "read_model_orchestration_control_plane_schema.jsonl",
        case_id: "read_model.orchestration_control_plane.cross_scope_route_mismatch.001",
        family: "orchestration_control_plane",
    },
    ReplayCase {
        trace: "read_model_orchestration_control_plane_schema.jsonl",
        case_id: "read_model.orchestration_control_plane.shared_approval_blocked.001",
        family: "orchestration_control_plane",
    },
    ReplayCase {
        trace: "read_model_orchestration_control_plane_schema.jsonl",
        case_id: "read_model.orchestration_control_plane.missing_scope_key.001",
        family: "orchestration_control_plane",
    },
];
const V230_DASHBOARD_OBSERVABILITY_CASES: &[ReplayCase] = &[
    ReplayCase {
        trace: "read_model_dashboard_observability_schema.jsonl",
        case_id: "read_model.dashboard_observability.multi_scope_readonly.001",
        family: "dashboard_observability",
    },
    ReplayCase {
        trace: "read_model_dashboard_observability_schema.jsonl",
        case_id: "read_model.dashboard_observability.filtered_drilldown_isolated.001",
        family: "dashboard_observability",
    },
    ReplayCase {
        trace: "read_model_dashboard_observability_schema.jsonl",
        case_id: "read_model.dashboard_observability.cross_scope_label_mismatch.001",
        family: "dashboard_observability",
    },
    ReplayCase {
        trace: "read_model_dashboard_observability_schema.jsonl",
        case_id: "read_model.dashboard_observability.missing_identity_degraded.001",
        family: "dashboard_observability",
    },
];

#[derive(Clone, Copy)]
struct ReplayCase {
    trace: &'static str,
    case_id: &'static str,
    family: &'static str,
}

#[test]
fn rust_cli_read_model_projection_replays_v211_required_paths() -> Result<(), Box<dyn Error>> {
    let mut covered_families = BTreeSet::new();
    let mut covered_cases = BTreeSet::new();

    for replay_case in REQUIRED_CASES {
        let case = load_case(replay_case.trace, replay_case.case_id)?;
        let input_event = single_event(&case, "input", replay_case.case_id)?;
        let expected_event = single_event(&case, "expected", replay_case.case_id)?;
        let actual_event = project_read_model_event(replay_case.case_id, input_event)?;

        if actual_event != *expected_event {
            return Err(format!(
                "{} Rust read-model projection replay mismatch\nexpected={}\nactual={}",
                replay_case.case_id, expected_event, actual_event
            )
            .into());
        }

        covered_families.insert(replay_case.family);
        covered_cases.insert(replay_case.case_id);
    }

    if covered_cases.len() != REQUIRED_CASES.len() {
        return Err(format!(
            "V221-003 must keep {} read_model projection cases, got {}",
            REQUIRED_CASES.len(),
            covered_cases.len()
        )
        .into());
    }
    assert_contains_all(
        &covered_families,
        &[
            "contract",
            "account",
            "position",
            "order",
            "fill",
            "risk",
            "dashboard",
        ],
        "read_model family",
    )?;

    Ok(())
}

#[test]
fn rust_cli_read_model_projection_replays_v230_account_partition_paths()
-> Result<(), Box<dyn Error>> {
    let mut covered_families = BTreeSet::new();
    let mut covered_cases = BTreeSet::new();

    for replay_case in V230_ACCOUNT_PARTITION_CASES {
        let case = load_case(replay_case.trace, replay_case.case_id)?;
        let input_event = single_event(&case, "input", replay_case.case_id)?;
        let expected_event = single_event(&case, "expected", replay_case.case_id)?;
        let actual_event = project_read_model_event(replay_case.case_id, input_event)?;

        if actual_event != *expected_event {
            return Err(format!(
                "{} Rust account-partition projection replay mismatch\nexpected={}\nactual={}",
                replay_case.case_id, expected_event, actual_event
            )
            .into());
        }

        covered_families.insert(replay_case.family);
        covered_cases.insert(replay_case.case_id);
    }

    if covered_cases.len() != V230_ACCOUNT_PARTITION_CASES.len() {
        return Err(format!(
            "V230-002 must keep {} account-partition projection cases, got {}",
            V230_ACCOUNT_PARTITION_CASES.len(),
            covered_cases.len()
        )
        .into());
    }
    assert_contains_all(
        &covered_families,
        &["account_partition"],
        "read_model account partition family",
    )?;

    Ok(())
}

#[test]
fn rust_cli_read_model_projection_replays_v230_strategy_supervisor_paths()
-> Result<(), Box<dyn Error>> {
    let mut covered_families = BTreeSet::new();
    let mut covered_cases = BTreeSet::new();

    for replay_case in V230_STRATEGY_SUPERVISOR_CASES {
        let case = load_case(replay_case.trace, replay_case.case_id)?;
        let input_event = single_event(&case, "input", replay_case.case_id)?;
        let expected_event = single_event(&case, "expected", replay_case.case_id)?;
        let actual_event = project_read_model_event(replay_case.case_id, input_event)?;

        if actual_event != *expected_event {
            return Err(format!(
                "{} Rust strategy-supervisor projection replay mismatch\nexpected={}\nactual={}",
                replay_case.case_id, expected_event, actual_event
            )
            .into());
        }

        covered_families.insert(replay_case.family);
        covered_cases.insert(replay_case.case_id);
    }

    if covered_cases.len() != V230_STRATEGY_SUPERVISOR_CASES.len() {
        return Err(format!(
            "V230-003 must keep {} strategy-supervisor projection cases, got {}",
            V230_STRATEGY_SUPERVISOR_CASES.len(),
            covered_cases.len()
        )
        .into());
    }
    assert_contains_all(
        &covered_families,
        &["strategy_supervisor"],
        "read_model strategy supervisor family",
    )?;

    Ok(())
}

#[test]
fn rust_cli_read_model_projection_replays_v230_venue_node_lifecycle_paths()
-> Result<(), Box<dyn Error>> {
    let mut covered_families = BTreeSet::new();
    let mut covered_cases = BTreeSet::new();

    for replay_case in V230_VENUE_NODE_LIFECYCLE_CASES {
        let case = load_case(replay_case.trace, replay_case.case_id)?;
        let input_event = single_event(&case, "input", replay_case.case_id)?;
        let expected_event = single_event(&case, "expected", replay_case.case_id)?;
        let actual_event = project_read_model_event(replay_case.case_id, input_event)?;

        if actual_event != *expected_event {
            return Err(format!(
                "{} Rust venue-node lifecycle projection replay mismatch\nexpected={}\nactual={}",
                replay_case.case_id, expected_event, actual_event
            )
            .into());
        }

        covered_families.insert(replay_case.family);
        covered_cases.insert(replay_case.case_id);
    }

    if covered_cases.len() != V230_VENUE_NODE_LIFECYCLE_CASES.len() {
        return Err(format!(
            "V230-004 must keep {} venue-node lifecycle projection cases, got {}",
            V230_VENUE_NODE_LIFECYCLE_CASES.len(),
            covered_cases.len()
        )
        .into());
    }
    assert_contains_all(
        &covered_families,
        &["venue_node_lifecycle"],
        "read_model venue node lifecycle family",
    )?;

    Ok(())
}

#[test]
fn rust_cli_read_model_projection_replays_v230_orchestration_control_plane_paths()
-> Result<(), Box<dyn Error>> {
    let mut covered_families = BTreeSet::new();
    let mut covered_cases = BTreeSet::new();

    for replay_case in V230_ORCHESTRATION_CONTROL_PLANE_CASES {
        let case = load_case(replay_case.trace, replay_case.case_id)?;
        let input_event = single_event(&case, "input", replay_case.case_id)?;
        let expected_event = single_event(&case, "expected", replay_case.case_id)?;
        let actual_event = project_read_model_event(replay_case.case_id, input_event)?;

        if actual_event != *expected_event {
            return Err(format!(
                "{} Rust orchestration control-plane projection replay mismatch\nexpected={}\nactual={}",
                replay_case.case_id, expected_event, actual_event
            )
            .into());
        }

        covered_families.insert(replay_case.family);
        covered_cases.insert(replay_case.case_id);
    }

    if covered_cases.len() != V230_ORCHESTRATION_CONTROL_PLANE_CASES.len() {
        return Err(format!(
            "V230-005 must keep {} orchestration control-plane projection cases, got {}",
            V230_ORCHESTRATION_CONTROL_PLANE_CASES.len(),
            covered_cases.len()
        )
        .into());
    }
    assert_contains_all(
        &covered_families,
        &["orchestration_control_plane"],
        "read_model orchestration control-plane family",
    )?;

    Ok(())
}

#[test]
fn rust_cli_read_model_projection_replays_v230_dashboard_observability_paths()
-> Result<(), Box<dyn Error>> {
    let mut covered_families = BTreeSet::new();
    let mut covered_cases = BTreeSet::new();

    for replay_case in V230_DASHBOARD_OBSERVABILITY_CASES {
        let case = load_case(replay_case.trace, replay_case.case_id)?;
        let input_event = single_event(&case, "input", replay_case.case_id)?;
        let expected_event = single_event(&case, "expected", replay_case.case_id)?;
        let actual_event = project_read_model_event(replay_case.case_id, input_event)?;

        if actual_event != *expected_event {
            return Err(format!(
                "{} Rust dashboard observability projection replay mismatch\nexpected={}\nactual={}",
                replay_case.case_id, expected_event, actual_event
            )
            .into());
        }

        covered_families.insert(replay_case.family);
        covered_cases.insert(replay_case.case_id);
    }

    if covered_cases.len() != V230_DASHBOARD_OBSERVABILITY_CASES.len() {
        return Err(format!(
            "V230-006 must keep {} dashboard observability projection cases, got {}",
            V230_DASHBOARD_OBSERVABILITY_CASES.len(),
            covered_cases.len()
        )
        .into());
    }
    assert_contains_all(
        &covered_families,
        &["dashboard_observability"],
        "read_model dashboard observability family",
    )?;

    Ok(())
}

fn project_read_model_event(case_id: &str, input_event: &Value) -> Result<Value, Box<dyn Error>> {
    let mut event = Map::new();
    let event_type = if case_id.starts_with("read_model.contract.") {
        "read_model.contract.validated".to_string()
    } else {
        string_field(input_event, "event_type")?.replace(".input", ".validated")
    };
    event.insert("event_type".to_string(), Value::String(event_type));

    for key in [
        "ts_event",
        "ts_init",
        "instrument_id",
        "venue",
        "correlation_id",
    ] {
        let value = input_event
            .get(key)
            .ok_or_else(|| format!("{case_id}: input event missing {key}"))?;
        event.insert(key.to_string(), value.clone());
    }

    let input_payload = payload(input_event)?;
    let snapshot = object_field(input_payload, "snapshot")?;
    let projected_payload = if case_id.starts_with("read_model.contract.") {
        project_contract_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.dashboard_observability.") {
        project_dashboard_observability_payload(case_id, input_payload, snapshot)?
    } else if case_id.starts_with("read_model.orchestration_control_plane.") {
        project_orchestration_control_plane_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.venue_node_lifecycle.") {
        project_venue_node_lifecycle_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.strategy_supervisor.") {
        project_strategy_supervisor_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.account_partition.") {
        project_account_partition_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.account_snapshot.") {
        project_account_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.position.") {
        project_position_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.order_lifecycle.") {
        project_order_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.fill_execution.") {
        project_fill_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.risk_state.") {
        project_risk_payload(case_id, snapshot)?
    } else if case_id.starts_with("read_model.dashboard.") {
        project_dashboard_payload(case_id, input_payload, snapshot)?
    } else {
        return Err(format!("{case_id}: unsupported read-model replay case").into());
    };
    event.insert("payload".to_string(), projected_payload);

    Ok(Value::Object(event))
}

fn project_contract_payload(case_id: &str, snapshot: &Value) -> Result<Value, Box<dyn Error>> {
    let components = object_field(snapshot, "components")?;
    let boundary = object_field(snapshot, "capability_boundary")?;
    let mut blocking_reasons = Vec::new();
    let mut all_healthy = true;
    let mut any_fail_closed = false;

    for name in CONTRACT_COMPONENTS {
        let component = object_field(components, name)?;
        let status = string_field(component, "component_status")?;
        all_healthy &= status == "healthy";
        any_fail_closed |= status == "fail_closed";

        let lineage = object_field(component, "lineage")?;
        let input_refs = array_field(lineage, "input_refs")?;
        let transform = string_field(lineage, "transform")?;
        if input_refs.is_empty() || transform.starts_with("missing:") {
            push_reason(
                &mut blocking_reasons,
                format!("missing_component_lineage:{name}"),
            );
        }

        let source = object_field(component, "source_provenance")?;
        if string_field(source, "source_type")? == "unavailable"
            || string_field(source, "source_ref")?.starts_with("missing:")
        {
            push_reason(
                &mut blocking_reasons,
                format!("missing_component_source_provenance:{name}"),
            );
        }

        match string_field(object_field(component, "freshness")?, "status")? {
            "missing" => push_reason(
                &mut blocking_reasons,
                format!("missing_component_freshness:{name}"),
            ),
            "stale" => push_reason(
                &mut blocking_reasons,
                format!("stale_component_freshness:{name}"),
            ),
            "fresh" => {}
            status => {
                return Err(
                    format!("{case_id}: unsupported {name} freshness status {status}").into(),
                );
            }
        }
    }

    let health_status = if !blocking_reasons.is_empty() || any_fail_closed {
        "fail_closed"
    } else if all_healthy {
        "healthy"
    } else {
        "degraded"
    };
    let claimed_health_status = string_field(snapshot, "health_status")?;
    if health_status != claimed_health_status {
        return Err(format!(
            "{case_id}: derived health status {health_status} does not match snapshot claim {claimed_health_status}"
        )
        .into());
    }
    let claimed_blocking_reasons = string_array(snapshot, "blocking_reasons")?;
    if blocking_reasons != claimed_blocking_reasons {
        return Err(format!(
            "{case_id}: derived blocking reasons {blocking_reasons:?} do not match snapshot claim {claimed_blocking_reasons:?}"
        )
        .into());
    }

    let mut payload = Map::new();
    insert_string(&mut payload, "case_id", case_id);
    insert_string(
        &mut payload,
        "contract_version",
        string_field(snapshot, "contract_version")?,
    );
    insert_string(&mut payload, "health_status", health_status);
    if health_status == "healthy" {
        payload.insert(
            "components".to_string(),
            Value::Array(
                CONTRACT_COMPONENTS
                    .iter()
                    .map(|name| Value::String((*name).to_string()))
                    .collect(),
            ),
        );
    }
    payload.insert(
        "blocking_reasons".to_string(),
        Value::Array(blocking_reasons.into_iter().map(Value::String).collect()),
    );
    for key in [
        "new_submit_capability",
        "dashboard_order_controls_enabled",
        "product_grade_trading_terminal_claim",
    ] {
        payload.insert(key.to_string(), Value::Bool(bool_field(boundary, key)?));
    }

    Ok(Value::Object(payload))
}

fn project_dashboard_observability_payload(
    case_id: &str,
    input_payload: &Value,
    snapshot: &Value,
) -> Result<Value, Box<dyn Error>> {
    let rows = array_field(snapshot, "dashboard_rows")?;
    let boundary = object_field(snapshot, "capability_boundary")?;
    let request = input_payload
        .get("dashboard_request")
        .unwrap_or(&Value::Null);
    let account_filter = request.get("account_key").and_then(Value::as_str);
    let strategy_filter = request.get("strategy_key").and_then(Value::as_str);
    let venue_node_filter = request.get("venue_node_key").and_then(Value::as_str);
    let requested_controls = request
        .get("requested_controls")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let filter_applied =
        account_filter.is_some() || strategy_filter.is_some() || venue_node_filter.is_some();
    let mut account_keys = BTreeSet::new();
    let mut strategy_keys = BTreeSet::new();
    let mut venue_node_keys = BTreeSet::new();
    let mut isolation_scope_keys = BTreeSet::new();
    let mut displayed_account_keys = BTreeSet::new();
    let mut displayed_strategy_keys = BTreeSet::new();
    let mut displayed_venue_node_keys = BTreeSet::new();
    let mut displayed_isolation_scope_keys = BTreeSet::new();
    let mut displayed_row_count = 0usize;
    let mut blocking_reasons = string_array(snapshot, "blocking_reasons")?;
    let mut identity_labels_present = true;
    let mut cross_scope_contamination_detected = false;
    let mut read_path_preserves_provenance = true;
    let mut filter_scope_isolated = true;

    for (row_index, row) in rows.iter().enumerate() {
        let account_key = optional_non_empty_string(row, "account_key");
        let strategy_key = optional_non_empty_string(row, "strategy_key");
        let venue_node_key = optional_non_empty_string(row, "venue_node_key");
        let isolation_scope_key = optional_non_empty_string(row, "isolation_scope_key");

        match account_key {
            Some(key) => {
                account_keys.insert(key.to_string());
            }
            None => {
                identity_labels_present = false;
                read_path_preserves_provenance = false;
                filter_scope_isolated = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_account_key:row:{row_index}"),
                );
            }
        }
        match strategy_key {
            Some(key) => {
                strategy_keys.insert(key.to_string());
            }
            None => {
                identity_labels_present = false;
                read_path_preserves_provenance = false;
                filter_scope_isolated = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_strategy_key:row:{row_index}"),
                );
            }
        }
        match venue_node_key {
            Some(key) => {
                venue_node_keys.insert(key.to_string());
            }
            None => {
                identity_labels_present = false;
                read_path_preserves_provenance = false;
                filter_scope_isolated = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_venue_node_key:row:{row_index}"),
                );
            }
        }
        match isolation_scope_key {
            Some(key) => {
                isolation_scope_keys.insert(key.to_string());
            }
            None => {
                identity_labels_present = false;
                read_path_preserves_provenance = false;
                filter_scope_isolated = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_isolation_scope_key:row:{row_index}"),
                );
            }
        }

        let row_matches_filter = account_filter.is_none_or(|filter| account_key == Some(filter))
            && strategy_filter.is_none_or(|filter| strategy_key == Some(filter))
            && venue_node_filter.is_none_or(|filter| venue_node_key == Some(filter));
        if row_matches_filter {
            displayed_row_count += 1;
            if let Some(key) = account_key {
                displayed_account_keys.insert(key.to_string());
            }
            if let Some(key) = strategy_key {
                displayed_strategy_keys.insert(key.to_string());
            }
            if let Some(key) = venue_node_key {
                displayed_venue_node_keys.insert(key.to_string());
            }
            if let Some(key) = isolation_scope_key {
                displayed_isolation_scope_keys.insert(key.to_string());
            }
        }

        let Some(row_account_key) = account_key else {
            continue;
        };
        let Some(row_strategy_key) = strategy_key else {
            continue;
        };
        let Some(row_venue_node_key) = venue_node_key else {
            continue;
        };
        let Some(row_scope_key) = isolation_scope_key else {
            continue;
        };

        let visible_labels = object_field(row, "visible_labels")?;
        validate_scoped_component_identity(
            visible_labels,
            "visible_labels",
            row_account_key,
            row_strategy_key,
            row_venue_node_key,
            row_scope_key,
            &mut identity_labels_present,
            &mut cross_scope_contamination_detected,
            &mut read_path_preserves_provenance,
            &mut blocking_reasons,
        );

        let components = object_field(row, "components")?;
        for component_name in DASHBOARD_OBSERVABILITY_COMPONENTS {
            let component = object_field(components, component_name)?;
            validate_scoped_component_identity(
                component,
                component_name,
                row_account_key,
                row_strategy_key,
                row_venue_node_key,
                row_scope_key,
                &mut identity_labels_present,
                &mut cross_scope_contamination_detected,
                &mut read_path_preserves_provenance,
                &mut blocking_reasons,
            );
        }
    }

    if filter_applied && displayed_isolation_scope_keys.len() != 1 {
        filter_scope_isolated = false;
        push_reason(
            &mut blocking_reasons,
            "filter_did_not_resolve_single_isolation_scope".to_string(),
        );
    }
    if cross_scope_contamination_detected {
        filter_scope_isolated = false;
    }

    let mut dashboard_has_no_operation_controls = true;
    for key in [
        "new_submit_capability",
        "dashboard_order_controls_enabled",
        "dashboard_approval_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "dashboard_retry_controls_enabled",
        "dashboard_submit_controls_enabled",
        "dashboard_replace_controls_enabled",
        "dashboard_amend_controls_enabled",
        "dashboard_flatten_controls_enabled",
        "trader_terminal_order_ticket_enabled",
        "manual_operation_entry_enabled",
        "manual_operation_submit_allowed",
        "manual_operation_cancel_allowed",
        "manual_operation_retry_allowed",
        "manual_operation_replace_allowed",
        "manual_operation_amend_allowed",
        "manual_operation_flatten_allowed",
        "automatic_operation_action_allowed",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "product_grade_trading_terminal_claim",
    ] {
        if bool_field(boundary, key)? {
            dashboard_has_no_operation_controls = false;
            push_reason(
                &mut blocking_reasons,
                format!("forbidden_dashboard_control_enabled:{key}"),
            );
        }
    }

    let missing_identity_degraded_unavailable = !identity_labels_present;
    let drilldown_preserves_scope = !filter_applied || filter_scope_isolated;
    let dashboard_observability_status = if cross_scope_contamination_detected {
        "fail_closed"
    } else if missing_identity_degraded_unavailable || !read_path_preserves_provenance {
        "degraded_unavailable"
    } else if filter_applied {
        "filtered_readonly"
    } else {
        "read_only_aggregated"
    };

    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "dashboard_observability_status",
        dashboard_observability_status,
    );
    payload.insert("dashboard_row_count".to_string(), json!(rows.len()));
    payload.insert(
        "displayed_row_count".to_string(),
        json!(displayed_row_count),
    );
    payload.insert("account_keys".to_string(), string_set_value(account_keys));
    payload.insert("strategy_keys".to_string(), string_set_value(strategy_keys));
    payload.insert(
        "venue_node_keys".to_string(),
        string_set_value(venue_node_keys),
    );
    payload.insert(
        "isolation_scope_keys".to_string(),
        string_set_value(isolation_scope_keys),
    );
    payload.insert(
        "displayed_account_keys".to_string(),
        string_set_value(displayed_account_keys),
    );
    payload.insert(
        "displayed_strategy_keys".to_string(),
        string_set_value(displayed_strategy_keys),
    );
    payload.insert(
        "displayed_venue_node_keys".to_string(),
        string_set_value(displayed_venue_node_keys),
    );
    payload.insert(
        "displayed_isolation_scope_keys".to_string(),
        string_set_value(displayed_isolation_scope_keys),
    );
    payload.insert("filter_applied".to_string(), Value::Bool(filter_applied));
    payload.insert(
        "filter_scope_isolated".to_string(),
        Value::Bool(filter_scope_isolated),
    );
    payload.insert(
        "drilldown_preserves_scope".to_string(),
        Value::Bool(drilldown_preserves_scope),
    );
    payload.insert(
        "identity_labels_present".to_string(),
        Value::Bool(identity_labels_present),
    );
    payload.insert(
        "cross_scope_contamination_detected".to_string(),
        Value::Bool(cross_scope_contamination_detected),
    );
    payload.insert(
        "missing_identity_degraded_unavailable".to_string(),
        Value::Bool(missing_identity_degraded_unavailable),
    );
    payload.insert(
        "read_path_preserves_provenance".to_string(),
        Value::Bool(read_path_preserves_provenance),
    );
    payload.insert(
        "dashboard_has_no_operation_controls".to_string(),
        Value::Bool(dashboard_has_no_operation_controls),
    );
    payload.insert(
        "forbidden_controls_absent".to_string(),
        Value::Bool(dashboard_has_no_operation_controls),
    );
    payload.insert(
        "blocked_controls".to_string(),
        Value::Array(requested_controls),
    );
    payload.insert(
        "blocking_reasons".to_string(),
        Value::Array(
            blocking_reasons
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>(),
        ),
    );

    Ok(Value::Object(payload))
}

fn project_orchestration_control_plane_payload(
    case_id: &str,
    snapshot: &Value,
) -> Result<Value, Box<dyn Error>> {
    let intents = array_field(snapshot, "orchestration_intents")?;
    let boundary = object_field(snapshot, "control_boundary")?;
    let mut account_keys = BTreeSet::new();
    let mut strategy_keys = BTreeSet::new();
    let mut venue_node_keys = BTreeSet::new();
    let mut isolation_scope_keys = BTreeSet::new();
    let mut approval_reference_ids = BTreeSet::new();
    let mut approval_scopes_by_reference = BTreeMap::<String, BTreeSet<String>>::new();
    let mut blocking_reasons = string_array(snapshot, "blocking_reasons")?;
    let mut identity_keys_present = true;
    let mut control_gate_contract_present = true;
    let mut cross_scope_contamination_detected = false;
    let mut shared_approval_consumption_detected = false;
    let mut approval_consumption_single_scope_only = true;
    let mut read_path_preserves_provenance = true;

    for (intent_index, intent) in intents.iter().enumerate() {
        let account_key = optional_non_empty_string(intent, "account_key");
        let strategy_key = optional_non_empty_string(intent, "strategy_key");
        let venue_node_key = optional_non_empty_string(intent, "venue_node_key");
        let isolation_scope_key = optional_non_empty_string(intent, "isolation_scope_key");
        let approval_reference_id = optional_non_empty_string(intent, "approval_reference_id");
        let approval_consumption_scope_key =
            optional_non_empty_string(intent, "approval_consumption_scope_key");

        match account_key {
            Some(key) => {
                account_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_account_key:partition:{intent_index}"),
                );
            }
        }
        match strategy_key {
            Some(key) => {
                strategy_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_strategy_key:partition:{intent_index}"),
                );
            }
        }
        match venue_node_key {
            Some(key) => {
                venue_node_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_venue_node_key:partition:{intent_index}"),
                );
            }
        }
        match isolation_scope_key {
            Some(key) => {
                isolation_scope_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_isolation_scope_key:partition:{intent_index}"),
                );
            }
        }

        match approval_reference_id {
            Some(approval_reference_id) => {
                approval_reference_ids.insert(approval_reference_id.to_string());
                if let Some(scope_key) = isolation_scope_key {
                    approval_scopes_by_reference
                        .entry(approval_reference_id.to_string())
                        .or_default()
                        .insert(scope_key.to_string());
                }
            }
            None => {
                control_gate_contract_present = false;
                approval_consumption_single_scope_only = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_approval_reference_id:partition:{intent_index}"),
                );
            }
        }

        match (approval_consumption_scope_key, isolation_scope_key) {
            (Some(approval_scope_key), Some(scope_key)) if approval_scope_key == scope_key => {}
            (Some(_), Some(_)) => {
                control_gate_contract_present = false;
                cross_scope_contamination_detected = true;
                approval_consumption_single_scope_only = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("approval_scope_mismatch:partition:{intent_index}"),
                );
            }
            _ => {
                control_gate_contract_present = false;
                approval_consumption_single_scope_only = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_approval_consumption_scope_key:partition:{intent_index}"),
                );
            }
        }

        let Some(intent_account_key) = account_key else {
            continue;
        };
        let Some(intent_strategy_key) = strategy_key else {
            continue;
        };
        let Some(intent_venue_node_key) = venue_node_key else {
            continue;
        };
        let Some(intent_scope_key) = isolation_scope_key else {
            continue;
        };

        let components = object_field(intent, "components")?;
        for component_name in ORCHESTRATION_CONTROL_PLANE_COMPONENTS {
            let component = object_field(components, component_name)?;
            validate_scoped_component_identity(
                component,
                component_name,
                intent_account_key,
                intent_strategy_key,
                intent_venue_node_key,
                intent_scope_key,
                &mut identity_keys_present,
                &mut cross_scope_contamination_detected,
                &mut read_path_preserves_provenance,
                &mut blocking_reasons,
            );
        }
    }

    for (approval_reference_id, scope_keys) in approval_scopes_by_reference {
        if scope_keys.len() > 1 {
            shared_approval_consumption_detected = true;
            approval_consumption_single_scope_only = false;
            push_reason(
                &mut blocking_reasons,
                format!("shared_approval_consumption:{approval_reference_id}"),
            );
        }
    }

    for key in [
        "owner_approval_gate_required",
        "risk_gate_required",
        "audit_gate_required",
    ] {
        if !bool_field(boundary, key)? {
            control_gate_contract_present = false;
            push_reason(
                &mut blocking_reasons,
                format!("missing_required_gate:{key}"),
            );
        }
    }
    for key in [
        "implicit_cross_account_operation_allowed",
        "implicit_cross_strategy_operation_allowed",
        "implicit_cross_venue_operation_allowed",
        "implicit_cross_node_operation_allowed",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "ungated_submit_cancel_retry_replace_amend_flatten_allowed",
        "dashboard_operation_controls_enabled",
        "production_order_submission_allowed",
    ] {
        if bool_field(boundary, key)? {
            control_gate_contract_present = false;
            push_reason(
                &mut blocking_reasons,
                format!("forbidden_boundary_enabled:{key}"),
            );
        }
    }

    let missing_identity_key_fail_closed = !identity_keys_present;
    let missing_isolation_scope_key_fail_closed = intents
        .iter()
        .any(|intent| optional_non_empty_string(intent, "isolation_scope_key").is_none());
    let control_plane_gate_status = if missing_identity_key_fail_closed
        || !control_gate_contract_present
        || cross_scope_contamination_detected
        || shared_approval_consumption_detected
    {
        "fail_closed"
    } else {
        "gated_readonly"
    };

    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "control_plane_gate_status",
        control_plane_gate_status,
    );
    payload.insert(
        "orchestration_intent_count".to_string(),
        json!(intents.len()),
    );
    payload.insert("account_keys".to_string(), string_set_value(account_keys));
    payload.insert("strategy_keys".to_string(), string_set_value(strategy_keys));
    payload.insert(
        "venue_node_keys".to_string(),
        string_set_value(venue_node_keys),
    );
    payload.insert(
        "isolation_scope_keys".to_string(),
        string_set_value(isolation_scope_keys),
    );
    payload.insert(
        "approval_reference_ids".to_string(),
        string_set_value(approval_reference_ids),
    );
    payload.insert(
        "identity_keys_present".to_string(),
        Value::Bool(identity_keys_present),
    );
    payload.insert(
        "control_gate_contract_present".to_string(),
        Value::Bool(control_gate_contract_present),
    );
    payload.insert(
        "cross_scope_contamination_detected".to_string(),
        Value::Bool(cross_scope_contamination_detected),
    );
    payload.insert(
        "shared_approval_consumption_detected".to_string(),
        Value::Bool(shared_approval_consumption_detected),
    );
    payload.insert(
        "approval_consumption_single_scope_only".to_string(),
        Value::Bool(approval_consumption_single_scope_only),
    );
    payload.insert(
        "missing_identity_key_fail_closed".to_string(),
        Value::Bool(missing_identity_key_fail_closed),
    );
    payload.insert(
        "missing_isolation_scope_key_fail_closed".to_string(),
        Value::Bool(missing_isolation_scope_key_fail_closed),
    );
    payload.insert(
        "read_path_preserves_provenance".to_string(),
        Value::Bool(read_path_preserves_provenance),
    );
    for key in [
        "owner_approval_gate_required",
        "risk_gate_required",
        "audit_gate_required",
        "implicit_cross_account_operation_allowed",
        "implicit_cross_strategy_operation_allowed",
        "implicit_cross_venue_operation_allowed",
        "implicit_cross_node_operation_allowed",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "ungated_submit_cancel_retry_replace_amend_flatten_allowed",
        "dashboard_operation_controls_enabled",
        "production_order_submission_allowed",
    ] {
        payload.insert(key.to_string(), Value::Bool(bool_field(boundary, key)?));
    }
    payload.insert(
        "blocking_reasons".to_string(),
        Value::Array(
            blocking_reasons
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>(),
        ),
    );

    Ok(Value::Object(payload))
}

fn project_venue_node_lifecycle_payload(
    case_id: &str,
    snapshot: &Value,
) -> Result<Value, Box<dyn Error>> {
    let partitions = array_field(snapshot, "venue_node_partitions")?;
    let boundary = object_field(snapshot, "control_boundary")?;
    let mut account_keys = BTreeSet::new();
    let mut strategy_keys = BTreeSet::new();
    let mut venue_node_keys = BTreeSet::new();
    let mut isolation_scope_keys = BTreeSet::new();
    let mut adapter_instance_ids = BTreeSet::new();
    let mut blocking_reasons = string_array(snapshot, "blocking_reasons")?;
    let mut identity_keys_present = true;
    let mut node_registry_contract_present = true;
    let mut cross_node_contamination_detected = false;
    let mut read_path_preserves_provenance = true;

    for (partition_index, partition) in partitions.iter().enumerate() {
        let account_key = optional_non_empty_string(partition, "account_key");
        let strategy_key = optional_non_empty_string(partition, "strategy_key");
        let venue_node_key = optional_non_empty_string(partition, "venue_node_key");
        let isolation_scope_key = optional_non_empty_string(partition, "isolation_scope_key");

        match account_key {
            Some(key) => {
                account_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_account_key:partition:{partition_index}"),
                );
            }
        }
        match strategy_key {
            Some(key) => {
                strategy_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_strategy_key:partition:{partition_index}"),
                );
            }
        }
        match venue_node_key {
            Some(key) => {
                venue_node_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                node_registry_contract_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_venue_node_key:partition:{partition_index}"),
                );
            }
        }
        match isolation_scope_key {
            Some(key) => {
                isolation_scope_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_isolation_scope_key:partition:{partition_index}"),
                );
            }
        }

        let Some(partition_venue_node_key) = venue_node_key else {
            continue;
        };

        let registry_entry = object_field(partition, "registry_entry")?;
        match optional_non_empty_string(registry_entry, "venue_node_key") {
            Some(registry_venue_node_key)
                if registry_venue_node_key == partition_venue_node_key => {}
            Some(_) => {
                node_registry_contract_present = false;
                cross_node_contamination_detected = true;
                push_reason(
                    &mut blocking_reasons,
                    "registry_venue_node_mismatch".to_string(),
                );
            }
            None => {
                identity_keys_present = false;
                node_registry_contract_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    "missing_registry_venue_node_key".to_string(),
                );
            }
        }
        match optional_non_empty_string(registry_entry, "adapter_instance_id") {
            Some(adapter_instance_id) => {
                adapter_instance_ids.insert(adapter_instance_id.to_string());
            }
            None => {
                node_registry_contract_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    "missing_adapter_instance_id".to_string(),
                );
            }
        }
        if optional_non_empty_string(registry_entry, "source_provenance").is_none() {
            node_registry_contract_present = false;
            read_path_preserves_provenance = false;
            push_reason(
                &mut blocking_reasons,
                "missing_registry_source_provenance".to_string(),
            );
        }

        let components = object_field(partition, "components")?;
        for component_name in VENUE_NODE_LIFECYCLE_COMPONENTS {
            let component = object_field(components, component_name)?;
            match optional_non_empty_string(component, "venue_node_key") {
                Some(component_venue_node_key)
                    if component_venue_node_key == partition_venue_node_key => {}
                Some(_) => {
                    cross_node_contamination_detected = true;
                    push_reason(
                        &mut blocking_reasons,
                        format!("cross_node_component_mismatch:{component_name}"),
                    );
                }
                None => {
                    identity_keys_present = false;
                    read_path_preserves_provenance = false;
                    push_reason(
                        &mut blocking_reasons,
                        format!("missing_component_venue_node_key:{component_name}"),
                    );
                }
            }

            if optional_non_empty_string(component, "source_provenance").is_none() {
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_component_source_provenance:{component_name}"),
                );
            }
        }
    }

    let missing_venue_node_key_fail_closed = !identity_keys_present;
    let venue_node_lifecycle_status =
        if cross_node_contamination_detected || missing_venue_node_key_fail_closed {
            "fail_closed"
        } else {
            "isolated"
        };

    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "venue_node_lifecycle_status",
        venue_node_lifecycle_status,
    );
    payload.insert(
        "venue_node_partition_count".to_string(),
        json!(partitions.len()),
    );
    payload.insert("account_keys".to_string(), string_set_value(account_keys));
    payload.insert("strategy_keys".to_string(), string_set_value(strategy_keys));
    payload.insert(
        "venue_node_keys".to_string(),
        string_set_value(venue_node_keys),
    );
    payload.insert(
        "isolation_scope_keys".to_string(),
        string_set_value(isolation_scope_keys),
    );
    payload.insert(
        "adapter_instance_ids".to_string(),
        string_set_value(adapter_instance_ids),
    );
    payload.insert(
        "identity_keys_present".to_string(),
        Value::Bool(identity_keys_present),
    );
    payload.insert(
        "node_registry_contract_present".to_string(),
        Value::Bool(node_registry_contract_present),
    );
    payload.insert(
        "cross_node_contamination_detected".to_string(),
        Value::Bool(cross_node_contamination_detected),
    );
    payload.insert(
        "missing_venue_node_key_fail_closed".to_string(),
        Value::Bool(missing_venue_node_key_fail_closed),
    );
    payload.insert(
        "read_path_preserves_provenance".to_string(),
        Value::Bool(read_path_preserves_provenance),
    );
    for key in [
        "credential_handling_expansion_allowed",
        "production_order_mutation_allowed",
        "production_order_submission_allowed",
        "dashboard_operation_controls_enabled",
        "cross_venue_implicit_operation_allowed",
        "lifecycle_control_requires_owner_approval",
    ] {
        payload.insert(key.to_string(), Value::Bool(bool_field(boundary, key)?));
    }
    payload.insert(
        "blocking_reasons".to_string(),
        Value::Array(
            blocking_reasons
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>(),
        ),
    );

    Ok(Value::Object(payload))
}

fn project_strategy_supervisor_payload(
    case_id: &str,
    snapshot: &Value,
) -> Result<Value, Box<dyn Error>> {
    let partitions = array_field(snapshot, "strategy_partitions")?;
    let boundary = object_field(snapshot, "control_boundary")?;
    let mut account_keys = BTreeSet::new();
    let mut strategy_keys = BTreeSet::new();
    let mut venue_node_keys = BTreeSet::new();
    let mut isolation_scope_keys = BTreeSet::new();
    let mut blocking_reasons = string_array(snapshot, "blocking_reasons")?;
    let mut identity_keys_present = true;
    let mut cross_strategy_contamination_detected = false;
    let mut read_path_preserves_provenance = true;

    for (partition_index, partition) in partitions.iter().enumerate() {
        let account_key = optional_non_empty_string(partition, "account_key");
        let strategy_key = optional_non_empty_string(partition, "strategy_key");
        let venue_node_key = optional_non_empty_string(partition, "venue_node_key");
        let isolation_scope_key = optional_non_empty_string(partition, "isolation_scope_key");

        match account_key {
            Some(key) => {
                account_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_account_key:partition:{partition_index}"),
                );
            }
        }
        match strategy_key {
            Some(key) => {
                strategy_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_strategy_key:partition:{partition_index}"),
                );
            }
        }
        match venue_node_key {
            Some(key) => {
                venue_node_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_venue_node_key:partition:{partition_index}"),
                );
            }
        }
        match isolation_scope_key {
            Some(key) => {
                isolation_scope_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_isolation_scope_key:partition:{partition_index}"),
                );
            }
        }

        let Some(partition_strategy_key) = strategy_key else {
            continue;
        };
        let components = object_field(partition, "components")?;
        for component_name in STRATEGY_SUPERVISOR_COMPONENTS {
            let component = object_field(components, component_name)?;
            match optional_non_empty_string(component, "strategy_key") {
                Some(component_strategy_key)
                    if component_strategy_key == partition_strategy_key => {}
                Some(_) => {
                    cross_strategy_contamination_detected = true;
                    push_reason(
                        &mut blocking_reasons,
                        format!("cross_strategy_component_mismatch:{component_name}"),
                    );
                }
                None => {
                    identity_keys_present = false;
                    read_path_preserves_provenance = false;
                    push_reason(
                        &mut blocking_reasons,
                        format!("missing_component_strategy_key:{component_name}"),
                    );
                }
            }

            if optional_non_empty_string(component, "source_provenance").is_none() {
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_component_source_provenance:{component_name}"),
                );
            }
        }
    }

    let missing_strategy_key_fail_closed = !identity_keys_present;
    let strategy_supervisor_status =
        if cross_strategy_contamination_detected || missing_strategy_key_fail_closed {
            "fail_closed"
        } else {
            "isolated"
        };

    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "strategy_supervisor_status",
        strategy_supervisor_status,
    );
    payload.insert(
        "strategy_partition_count".to_string(),
        json!(partitions.len()),
    );
    payload.insert("account_keys".to_string(), string_set_value(account_keys));
    payload.insert("strategy_keys".to_string(), string_set_value(strategy_keys));
    payload.insert(
        "venue_node_keys".to_string(),
        string_set_value(venue_node_keys),
    );
    payload.insert(
        "isolation_scope_keys".to_string(),
        string_set_value(isolation_scope_keys),
    );
    payload.insert(
        "identity_keys_present".to_string(),
        Value::Bool(identity_keys_present),
    );
    payload.insert(
        "cross_strategy_contamination_detected".to_string(),
        Value::Bool(cross_strategy_contamination_detected),
    );
    payload.insert(
        "missing_strategy_key_fail_closed".to_string(),
        Value::Bool(missing_strategy_key_fail_closed),
    );
    payload.insert(
        "read_path_preserves_provenance".to_string(),
        Value::Bool(read_path_preserves_provenance),
    );
    for key in [
        "owner_approval_gate_required",
        "approval_consumption_single_scope_only",
        "strategy_driven_production_execution_allowed",
        "dashboard_operation_controls_enabled",
        "production_order_submission_allowed",
    ] {
        payload.insert(key.to_string(), Value::Bool(bool_field(boundary, key)?));
    }
    payload.insert(
        "blocking_reasons".to_string(),
        Value::Array(
            blocking_reasons
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>(),
        ),
    );

    Ok(Value::Object(payload))
}

fn project_account_partition_payload(
    case_id: &str,
    snapshot: &Value,
) -> Result<Value, Box<dyn Error>> {
    let partitions = array_field(snapshot, "account_partitions")?;
    let boundary = object_field(snapshot, "capability_boundary")?;
    let mut account_keys = BTreeSet::new();
    let mut isolation_scope_keys = BTreeSet::new();
    let mut blocking_reasons = string_array(snapshot, "blocking_reasons")?;
    let mut identity_keys_present = true;
    let mut cross_account_contamination_detected = false;
    let mut read_path_preserves_provenance = true;

    for (partition_index, partition) in partitions.iter().enumerate() {
        let account_key = optional_non_empty_string(partition, "account_key");
        let isolation_scope_key = optional_non_empty_string(partition, "isolation_scope_key");

        match account_key {
            Some(key) => {
                account_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_account_key:partition:{partition_index}"),
                );
            }
        }

        match isolation_scope_key {
            Some(key) => {
                isolation_scope_keys.insert(key.to_string());
            }
            None => {
                identity_keys_present = false;
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_isolation_scope_key:partition:{partition_index}"),
                );
            }
        }

        let Some(partition_account_key) = account_key else {
            continue;
        };
        let components = object_field(partition, "components")?;
        for component_name in ACCOUNT_PARTITION_COMPONENTS {
            let component = object_field(components, component_name)?;
            match optional_non_empty_string(component, "account_key") {
                Some(component_account_key) if component_account_key == partition_account_key => {}
                Some(_) => {
                    cross_account_contamination_detected = true;
                    push_reason(
                        &mut blocking_reasons,
                        format!("cross_account_component_mismatch:{component_name}"),
                    );
                }
                None => {
                    identity_keys_present = false;
                    read_path_preserves_provenance = false;
                    push_reason(
                        &mut blocking_reasons,
                        format!("missing_component_account_key:{component_name}"),
                    );
                }
            }

            if optional_non_empty_string(component, "source_provenance").is_none() {
                read_path_preserves_provenance = false;
                push_reason(
                    &mut blocking_reasons,
                    format!("missing_component_source_provenance:{component_name}"),
                );
            }
        }
    }

    let missing_account_key_fail_closed = !identity_keys_present;
    let account_partition_status =
        if cross_account_contamination_detected || missing_account_key_fail_closed {
            "fail_closed"
        } else {
            "isolated"
        };

    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "account_partition_status",
        account_partition_status,
    );
    payload.insert(
        "account_partition_count".to_string(),
        json!(partitions.len()),
    );
    payload.insert("account_keys".to_string(), string_set_value(account_keys));
    payload.insert(
        "isolation_scope_keys".to_string(),
        string_set_value(isolation_scope_keys),
    );
    payload.insert(
        "identity_keys_present".to_string(),
        Value::Bool(identity_keys_present),
    );
    payload.insert(
        "cross_account_contamination_detected".to_string(),
        Value::Bool(cross_account_contamination_detected),
    );
    payload.insert(
        "missing_account_key_fail_closed".to_string(),
        Value::Bool(missing_account_key_fail_closed),
    );
    payload.insert(
        "read_path_preserves_provenance".to_string(),
        Value::Bool(read_path_preserves_provenance),
    );
    payload.insert(
        "dashboard_operation_controls_enabled".to_string(),
        Value::Bool(bool_field(
            boundary,
            "dashboard_operation_controls_enabled",
        )?),
    );
    payload.insert(
        "production_order_submission_allowed".to_string(),
        Value::Bool(bool_field(boundary, "production_order_submission_allowed")?),
    );
    payload.insert(
        "blocking_reasons".to_string(),
        Value::Array(
            blocking_reasons
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>(),
        ),
    );

    Ok(Value::Object(payload))
}

fn project_account_payload(case_id: &str, snapshot: &Value) -> Result<Value, Box<dyn Error>> {
    let account = component(snapshot, "account")?;
    let account_data = object_field(account, "data")?;
    let boundary = object_field(snapshot, "capability_boundary")?;
    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "account_component_status",
        string_field(account, "component_status")?,
    );
    insert_string(
        &mut payload,
        "risk_state",
        string_field(account_data, "risk_state")?,
    );
    let blocking_reasons = if case_id.ends_with(".missing_provenance.001") {
        let source = object_field(account, "source_provenance")?;
        if string_field(source, "source_type")? != "unavailable"
            || !string_field(source, "source_ref")?.starts_with("missing:")
        {
            return Err(format!("{case_id}: account source provenance must be unavailable").into());
        }
        vec!["missing_account_source_provenance".to_string()]
    } else if case_id.ends_with(".redaction_breach.001") {
        let redaction = object_field(account, "redaction")?;
        if string_field(redaction, "status")? != "fail_closed" {
            return Err(format!("{case_id}: account redaction must fail closed").into());
        }
        string_array(account, "diagnostics")?
            .into_iter()
            .filter(|reason| {
                reason == "unredacted_sensitive_field" || reason == "raw_account_payload_persisted"
            })
            .collect()
    } else {
        string_array(snapshot, "blocking_reasons")?
    };
    if blocking_reasons != string_array(snapshot, "blocking_reasons")? {
        return Err(
            format!("{case_id}: derived account blocking reasons do not match snapshot").into(),
        );
    }
    payload.insert(
        "blocking_reasons".to_string(),
        Value::Array(blocking_reasons.into_iter().map(Value::String).collect()),
    );
    payload.insert(
        "dashboard_account_state_visible".to_string(),
        Value::Bool(true),
    );
    payload.insert(
        "dashboard_operation_controls_enabled".to_string(),
        Value::Bool(bool_field(boundary, "dashboard_order_controls_enabled")?),
    );

    if string_field(account, "component_status")? == "healthy" {
        insert_string(
            &mut payload,
            "account_status",
            string_field(account_data, "account_status")?,
        );
        payload.insert(
            "balance_entry_count".to_string(),
            clone_field(account_data, "balance_entry_count")?,
        );
    }

    Ok(Value::Object(payload))
}

fn project_position_payload(case_id: &str, snapshot: &Value) -> Result<Value, Box<dyn Error>> {
    let positions = component(snapshot, "positions")?;
    let data = object_field(positions, "data")?;
    let risk_projection = object_field(data, "risk_projection_input")?;
    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "position_component_status",
        string_field(positions, "component_status")?,
    );
    insert_string(
        &mut payload,
        "net_position_side",
        string_field(data, "net_position_side")?,
    );
    payload.insert(
        "blocking_reasons".to_string(),
        clone_field(risk_projection, "blocking_reasons")?,
    );
    insert_string(
        &mut payload,
        "risk_state",
        string_field(risk_projection, "risk_state")?,
    );
    payload.insert(
        "auto_flatten_position_allowed".to_string(),
        Value::Bool(bool_field(
            risk_projection,
            "auto_flatten_position_allowed",
        )?),
    );
    payload.insert(
        "automatic_position_repair_allowed".to_string(),
        Value::Bool(bool_field(
            risk_projection,
            "automatic_position_repair_allowed",
        )?),
    );
    Ok(Value::Object(payload))
}

fn project_order_payload(case_id: &str, snapshot: &Value) -> Result<Value, Box<dyn Error>> {
    let orders = component(snapshot, "orders")?;
    let data = object_field(orders, "data")?;
    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "order_component_status",
        string_field(orders, "component_status")?,
    );
    for key in ["lifecycle_status", "readback_status"] {
        insert_string(&mut payload, key, string_field(data, key)?);
    }
    payload.insert(
        "blocking_reasons".to_string(),
        clone_field(snapshot, "blocking_reasons")?,
    );
    for key in [
        "no_retry",
        "automatic_remediation_allowed",
        "dashboard_readonly_visible",
    ] {
        payload.insert(key.to_string(), Value::Bool(bool_field(data, key)?));
    }
    Ok(Value::Object(payload))
}

fn project_fill_payload(case_id: &str, snapshot: &Value) -> Result<Value, Box<dyn Error>> {
    let fills = component(snapshot, "fills")?;
    let data = object_field(fills, "data")?;
    let risk_projection = object_field(data, "risk_projection_input")?;
    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "fill_component_status",
        string_field(fills, "component_status")?,
    );
    for key in ["reconciliation_status", "order_linkage_status"] {
        insert_string(&mut payload, key, string_field(data, key)?);
    }
    payload.insert(
        "blocking_reasons".to_string(),
        clone_field(risk_projection, "blocking_reasons")?,
    );
    payload.insert(
        "execution_algorithm_allowed".to_string(),
        Value::Bool(bool_field(risk_projection, "execution_algorithm_allowed")?),
    );
    payload.insert(
        "automatic_reconciliation_repair_allowed".to_string(),
        Value::Bool(bool_field(
            risk_projection,
            "automatic_reconciliation_repair_allowed",
        )?),
    );
    Ok(Value::Object(payload))
}

fn project_risk_payload(case_id: &str, snapshot: &Value) -> Result<Value, Box<dyn Error>> {
    let risk = component(snapshot, "risk")?;
    let data = object_field(risk, "data")?;
    let mut payload = base_payload(case_id, snapshot);
    insert_string(
        &mut payload,
        "risk_component_status",
        string_field(risk, "component_status")?,
    );
    insert_string(
        &mut payload,
        "risk_state",
        string_field(data, "risk_state")?,
    );
    payload.insert(
        "blocking_reasons".to_string(),
        clone_field(snapshot, "blocking_reasons")?,
    );
    payload.insert(
        "audit_closed_allowed".to_string(),
        Value::Bool(bool_field(data, "audit_closed_allowed")?),
    );
    payload.insert(
        "automatic_trading_action_allowed".to_string(),
        Value::Bool(bool_field(data, "automatic_trading_action_allowed")?),
    );
    Ok(Value::Object(payload))
}

fn project_dashboard_payload(
    case_id: &str,
    input_payload: &Value,
    snapshot: &Value,
) -> Result<Value, Box<dyn Error>> {
    let components = object_field(snapshot, "components")?;
    let boundary = object_field(snapshot, "capability_boundary")?;
    let health_status = string_field(snapshot, "health_status")?;
    let mut evidence_sections = Map::new();
    let component_to_panel = [
        ("account", "accounts"),
        ("positions", "positions"),
        ("orders", "orders"),
        ("fills", "fills"),
        ("risk", "risk"),
        ("lifecycle_status", "audit_provenance_diagnostics"),
    ];
    let mut missing_evidence = Vec::new();

    for (component_name, panel_name) in component_to_panel {
        let component = object_field(components, component_name)?;
        let status = string_field(component, "component_status")?;
        evidence_sections.insert(panel_name.to_string(), Value::String(status.to_string()));
        if status == "unavailable" {
            missing_evidence.push(Value::String(component_name.to_string()));
        }
    }

    let control_flags = DISABLED_CONTROLS
        .iter()
        .map(|control| {
            let boundary_key = format!("dashboard_{control}_controls_enabled");
            let enabled = boundary
                .get(&boundary_key)
                .and_then(Value::as_bool)
                .unwrap_or(false);
            ((*control).to_string(), Value::Bool(enabled))
        })
        .collect::<Map<String, Value>>();

    let requested_controls = input_payload
        .get("dashboard_request")
        .and_then(|request| request.get("requested_controls"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let terminal_status = match health_status {
        "healthy" => "foundation_only_readonly",
        "fail_closed" => "blocked_forbidden_controls",
        _ => "degraded_missing_evidence",
    };

    let mut payload = base_payload(case_id, snapshot);
    insert_string(&mut payload, "terminal_status", terminal_status);
    payload.insert("foundation_only".to_string(), Value::Bool(true));
    payload.insert("read_only".to_string(), Value::Bool(true));
    payload.insert("no_submit_controls".to_string(), Value::Bool(true));
    payload.insert("visible_panels".to_string(), json!(VISIBLE_PANELS));
    payload.insert(
        "evidence_sections".to_string(),
        Value::Object(evidence_sections),
    );
    payload.insert(
        "missing_evidence".to_string(),
        Value::Array(missing_evidence),
    );
    payload.insert(
        "blocked_controls".to_string(),
        Value::Array(if health_status == "fail_closed" {
            requested_controls
        } else {
            Vec::new()
        }),
    );
    payload.insert("disabled_controls".to_string(), json!(DISABLED_CONTROLS));
    payload.insert("control_flags".to_string(), Value::Object(control_flags));
    insert_string(&mut payload, "display_claim", "read_only_foundation");
    payload.insert(
        "product_grade_trading_terminal_claim".to_string(),
        Value::Bool(bool_field(
            boundary,
            "product_grade_trading_terminal_claim",
        )?),
    );
    insert_string(&mut payload, "behavior_impact", "display_only");
    payload.insert(
        "blocking_reasons".to_string(),
        clone_field(snapshot, "blocking_reasons")?,
    );
    Ok(Value::Object(payload))
}

#[allow(clippy::too_many_arguments)]
fn validate_scoped_component_identity(
    component: &Value,
    component_name: &str,
    expected_account_key: &str,
    expected_strategy_key: &str,
    expected_venue_node_key: &str,
    expected_isolation_scope_key: &str,
    identity_keys_present: &mut bool,
    cross_scope_contamination_detected: &mut bool,
    read_path_preserves_provenance: &mut bool,
    blocking_reasons: &mut Vec<String>,
) {
    for (field_name, expected_value) in [
        ("account_key", expected_account_key),
        ("strategy_key", expected_strategy_key),
        ("venue_node_key", expected_venue_node_key),
        ("isolation_scope_key", expected_isolation_scope_key),
    ] {
        match optional_non_empty_string(component, field_name) {
            Some(component_value) if component_value == expected_value => {}
            Some(_) => {
                *cross_scope_contamination_detected = true;
                push_reason(
                    blocking_reasons,
                    format!("cross_scope_component_mismatch:{component_name}:{field_name}"),
                );
            }
            None => {
                *identity_keys_present = false;
                *read_path_preserves_provenance = false;
                push_reason(
                    blocking_reasons,
                    format!("missing_component_{field_name}:{component_name}"),
                );
            }
        }
    }

    if optional_non_empty_string(component, "source_provenance").is_none() {
        *read_path_preserves_provenance = false;
        push_reason(
            blocking_reasons,
            format!("missing_component_source_provenance:{component_name}"),
        );
    }
}

fn base_payload(case_id: &str, snapshot: &Value) -> Map<String, Value> {
    let mut payload = Map::new();
    payload.insert("case_id".to_string(), Value::String(case_id.to_string()));
    if let Some(status) = snapshot.get("health_status") {
        payload.insert("health_status".to_string(), status.clone());
    }
    payload
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/cli")
}

fn load_case(trace_file: &str, case_id: &str) -> Result<Value, Box<dyn Error>> {
    let trace = repository_root().join("tests/golden").join(trace_file);
    for line in fs::read_to_string(&trace)?
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
    {
        let row = serde_json::from_str::<Value>(line)?;
        if row.get("case_id").and_then(Value::as_str) == Some(case_id) {
            return Ok(row);
        }
    }
    Err(format!("case {case_id} not found in {}", trace.display()).into())
}

fn single_event<'a>(
    case: &'a Value,
    section: &str,
    case_id: &str,
) -> Result<&'a Value, Box<dyn Error>> {
    let events = case
        .get(section)
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{case_id}: {section}.events must be an array"))?;
    if events.len() != 1 {
        return Err(format!("{case_id}: {section}.events must contain one event").into());
    }
    Ok(&events[0])
}

fn payload(event: &Value) -> Result<&Value, Box<dyn Error>> {
    object_field(event, "payload")
}

fn component<'a>(snapshot: &'a Value, name: &str) -> Result<&'a Value, Box<dyn Error>> {
    object_field(object_field(snapshot, "components")?, name)
}

fn object_field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, Box<dyn Error>> {
    value
        .get(key)
        .filter(|field| field.is_object())
        .ok_or_else(|| format!("missing object field {key}").into())
}

fn array_field<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing array field {key}").into())
}

fn string_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}").into())
}

fn optional_non_empty_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).and_then(|field| {
        let trimmed = field.trim();
        (!trimmed.is_empty() && trimmed != "unknown" && trimmed != "unavailable").then_some(field)
    })
}

fn string_array(value: &Value, key: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let values = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing array field {key}"))?;
    values
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{key} must contain only strings").into())
        })
        .collect()
}

fn bool_field(value: &Value, key: &str) -> Result<bool, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing bool field {key}").into())
}

fn clone_field(value: &Value, key: &str) -> Result<Value, Box<dyn Error>> {
    value
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing field {key}").into())
}

fn insert_string(payload: &mut Map<String, Value>, key: &str, value: &str) {
    payload.insert(key.to_string(), Value::String(value.to_string()));
}

fn push_reason(blocking_reasons: &mut Vec<String>, reason: String) {
    if !blocking_reasons.iter().any(|existing| existing == &reason) {
        blocking_reasons.push(reason);
    }
}

fn string_set_value(values: BTreeSet<String>) -> Value {
    Value::Array(values.into_iter().map(Value::String).collect::<Vec<_>>())
}

fn assert_contains_all(
    actual: &BTreeSet<&str>,
    required: &[&str],
    label: &str,
) -> Result<(), Box<dyn Error>> {
    let missing = required
        .iter()
        .copied()
        .filter(|value| !actual.contains(value))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("{label} coverage missing {missing:?}").into())
    }
}
