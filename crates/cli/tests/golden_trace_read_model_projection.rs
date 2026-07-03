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
    collections::BTreeSet,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Map, Value, json};

const REQUIRED_CASES: &[ReplayCase] = &[
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
        &["account", "position", "order", "fill", "risk", "dashboard"],
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

fn project_read_model_event(case_id: &str, input_event: &Value) -> Result<Value, Box<dyn Error>> {
    let mut event = Map::new();
    let event_type = string_field(input_event, "event_type")?.replace(".input", ".validated");
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
    let projected_payload = if case_id.starts_with("read_model.strategy_supervisor.") {
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
    payload.insert(
        "blocking_reasons".to_string(),
        clone_field(snapshot, "blocking_reasons")?,
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
