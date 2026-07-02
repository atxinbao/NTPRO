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
    let projected_payload = if case_id.starts_with("read_model.account_snapshot.") {
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

fn string_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}").into())
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
