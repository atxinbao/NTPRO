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

//! v0.28 Trader Terminal 只读合同的 HTTP 响应投影。

use axum::{Json, http::StatusCode};
use serde_json::{Map, Value, json};

use super::{ApiResult, generated_at_now};

const API_CONTRACT_VERSION: &str = "ntpro.v280.trader_terminal_backend_api_contract_handoff.v1";
const BACKEND_CLOSURE_SOURCE_REF: &str =
    "docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json";
const PROVENANCE_SOURCE_REF: &str =
    "docs/rust-cutover/release/v0_28_0_admin_workbench_backend_state_bridge_artifact.json";
const AUDIT_SOURCE_REF: &str =
    "docs/rust-cutover/release/v0_28_0_persistent_audit_storage_runtime_artifact.json";
const TELEMETRY_SOURCE_REF: &str =
    "docs/rust-cutover/release/v0_28_0_telemetry_slo_ingestion_runtime_artifact.json";
const PERMISSION_SOURCE_REF: &str =
    "docs/rust-cutover/release/v0_28_0_identity_permission_runtime_artifact.json";
const DEPLOYMENT_SOURCE_REF: &str =
    "docs/rust-cutover/release/v0_28_0_deployment_orchestration_runtime_artifact.json";

const BACKEND_CLOSURE_SOURCE: &str = include_str!(
    "../../../../docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json"
);
const PROVENANCE_SOURCE: &str = include_str!(
    "../../../../docs/rust-cutover/release/v0_28_0_admin_workbench_backend_state_bridge_artifact.json"
);
const AUDIT_SOURCE: &str = include_str!(
    "../../../../docs/rust-cutover/release/v0_28_0_persistent_audit_storage_runtime_artifact.json"
);
const TELEMETRY_SOURCE: &str = include_str!(
    "../../../../docs/rust-cutover/release/v0_28_0_telemetry_slo_ingestion_runtime_artifact.json"
);
const PERMISSION_SOURCE: &str = include_str!(
    "../../../../docs/rust-cutover/release/v0_28_0_identity_permission_runtime_artifact.json"
);
const DEPLOYMENT_SOURCE: &str = include_str!(
    "../../../../docs/rust-cutover/release/v0_28_0_deployment_orchestration_runtime_artifact.json"
);
#[cfg(test)]
const API_CONTRACT_SOURCE: &str = include_str!(
    "../../../../docs/rust-cutover/release/v0_28_0_trader_terminal_backend_api_contract_artifact.json"
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectionErrorKind {
    Malformed,
    Unredacted,
    ForbiddenControls,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProjectionError {
    kind: ProjectionErrorKind,
    source_ref: &'static str,
    field: String,
}

type ProjectionResult<T> = Result<T, ProjectionError>;

pub(super) async fn backend_closure_status_api() -> ApiResult<Value> {
    api_response(project_backend_closure_status(BACKEND_CLOSURE_SOURCE))
}

pub(super) async fn provenance_drilldown_api() -> ApiResult<Value> {
    api_response(project_provenance_drilldown(PROVENANCE_SOURCE))
}

pub(super) async fn audit_entries_api() -> ApiResult<Value> {
    api_response(project_audit_entries(AUDIT_SOURCE))
}

pub(super) async fn telemetry_health_api() -> ApiResult<Value> {
    api_response(project_telemetry_health(TELEMETRY_SOURCE))
}

pub(super) async fn permission_snapshot_api() -> ApiResult<Value> {
    api_response(project_permission_snapshot(PERMISSION_SOURCE))
}

pub(super) async fn deployment_state_api() -> ApiResult<Value> {
    api_response(project_deployment_state(DEPLOYMENT_SOURCE))
}

fn api_response(result: ProjectionResult<Value>) -> ApiResult<Value> {
    result
        .map(Json)
        .map_err(|error| projection_error_response(&error))
}

fn projection_error_response(error: &ProjectionError) -> (StatusCode, Json<Value>) {
    let error_code = match error.kind {
        ProjectionErrorKind::Malformed => "fail_closed_malformed_response",
        ProjectionErrorKind::Unredacted => "fail_closed_unredacted_payload",
        ProjectionErrorKind::ForbiddenControls => "fail_closed_forbidden_controls",
    };
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error_code": error_code,
            "message": "Trader Terminal 只读响应源未通过合同校验",
            "source_ref": error.source_ref,
            "field": error.field,
            "operation_controls_enabled": false,
            "trading_controls_enabled": false,
            "order_ticket_enabled": false,
            "manual_operation_submit_allowed": false
        })),
    )
}

fn project_backend_closure_status(raw: &str) -> ProjectionResult<Value> {
    let source = parse_source(raw, BACKEND_CLOSURE_SOURCE_REF)?;
    let boundaries = required_false_object(
        &source,
        "/required_false_boundary_flags",
        BACKEND_CLOSURE_SOURCE_REF,
    )?;
    with_read_only_boundary(
        json!({
            "schema_version": "ntpro.v280.trader_terminal.backend_closure_status.response.v1",
            "contract_version": API_CONTRACT_VERSION,
            "backend_closure_status": "runtime_closed",
            "module_counts": clone_pointer(
                &source,
                "/expected_counts",
                BACKEND_CLOSURE_SOURCE_REF
            )?,
            "required_false_boundary_flags": boundaries,
            "generated_at": generated_at_now(),
            "source_refs": [
                BACKEND_CLOSURE_SOURCE_REF,
                "docs/rust-cutover/evidence/V280-001.md"
            ],
            "freshness_status": "fresh",
            "redaction_status": "redacted"
        }),
        BACKEND_CLOSURE_SOURCE_REF,
    )
}

fn project_provenance_drilldown(raw: &str) -> ProjectionResult<Value> {
    let source = parse_source(raw, PROVENANCE_SOURCE_REF)?;
    let boundaries = required_false_object(&source, "/boundary_flags", PROVENANCE_SOURCE_REF)?;
    let components = object_pointer(&source, "/component_states", PROVENANCE_SOURCE_REF)?;
    let mut entries = Vec::with_capacity(components.len());
    for (module_id, component) in components {
        require_redacted(
            component,
            "/redaction_status",
            PROVENANCE_SOURCE_REF,
            &format!("/component_states/{module_id}/redaction_status"),
        )?;
        entries.push(json!({
            "module_id": module_id,
            "source_ref": string_pointer(
                component,
                "/source_ref",
                PROVENANCE_SOURCE_REF
            )?,
            "evidence_path": string_pointer(
                component,
                "/evidence_path",
                PROVENANCE_SOURCE_REF
            )?,
            "verification_command": string_pointer(
                component,
                "/verification_command",
                PROVENANCE_SOURCE_REF
            )?,
            "provenance_status": string_pointer(
                component,
                "/provenance_status",
                PROVENANCE_SOURCE_REF
            )?,
            "freshness_status": string_pointer(
                component,
                "/freshness_status",
                PROVENANCE_SOURCE_REF
            )?,
            "redaction_status": "redacted"
        }));
    }
    with_read_only_boundary(
        json!({
            "schema_version": "ntpro.v280.trader_terminal.provenance_drilldown.response.v1",
            "contract_version": API_CONTRACT_VERSION,
            "module_id": string_pointer(&source, "/backend_module", PROVENANCE_SOURCE_REF)?,
            "source_ref": PROVENANCE_SOURCE_REF,
            "evidence_path": "docs/rust-cutover/evidence/V280-006.md",
            "verification_command":
                "scripts/ai/verify_release.sh v28-admin-workbench-backend-state-bridge-closure",
            "provenance_status": "verified",
            "freshness_status": "fresh",
            "redaction_status": "redacted",
            "entries": entries,
            "required_false_boundary_flags": boundaries
        }),
        PROVENANCE_SOURCE_REF,
    )
}

fn project_audit_entries(raw: &str) -> ProjectionResult<Value> {
    let source = parse_source(raw, AUDIT_SOURCE_REF)?;
    let boundaries = required_false_object(&source, "/boundary_flags", AUDIT_SOURCE_REF)?;
    let records = array_pointer(&source, "/persistent_records", AUDIT_SOURCE_REF)?;
    let mut entries = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        require_redacted(
            record,
            "/redaction_status",
            AUDIT_SOURCE_REF,
            &format!("/persistent_records/{index}/redaction_status"),
        )?;
        entries.push(project_audit_entry(record)?);
    }
    let latest = entries.last().cloned().ok_or_else(|| {
        malformed(
            AUDIT_SOURCE_REF,
            "/persistent_records must contain at least one entry",
        )
    })?;
    with_read_only_boundary(
        json!({
            "schema_version": "ntpro.v280.trader_terminal.audit_entries.response.v1",
            "contract_version": API_CONTRACT_VERSION,
            "audit_entry_id": latest["audit_entry_id"],
            "event_type": latest["event_type"],
            "actor_role": latest["actor_role"],
            "redaction_status": latest["redaction_status"],
            "append_only_sequence": latest["append_only_sequence"],
            "source_ref": AUDIT_SOURCE_REF,
            "freshness_status": string_pointer(
                &source,
                "/audit_sink/freshness_status",
                AUDIT_SOURCE_REF
            )?,
            "entries": entries,
            "required_false_boundary_flags": boundaries
        }),
        AUDIT_SOURCE_REF,
    )
}

fn project_audit_entry(record: &Value) -> ProjectionResult<Value> {
    Ok(json!({
        "audit_entry_id": string_pointer(record, "/record_id", AUDIT_SOURCE_REF)?,
        "event_type": string_pointer(record, "/audit_event_type", AUDIT_SOURCE_REF)?,
        "actor_role": string_pointer(record, "/actor/role", AUDIT_SOURCE_REF)?,
        "redaction_status": string_pointer(record, "/redaction_status", AUDIT_SOURCE_REF)?,
        "append_only_sequence": clone_pointer(record, "/sequence", AUDIT_SOURCE_REF)?,
        "source_ref": string_pointer(record, "/lineage/source_ref", AUDIT_SOURCE_REF)?
    }))
}

fn project_telemetry_health(raw: &str) -> ProjectionResult<Value> {
    let source = parse_source(raw, TELEMETRY_SOURCE_REF)?;
    let boundaries = required_false_object(&source, "/boundary_flags", TELEMETRY_SOURCE_REF)?;
    require_redacted(
        &source,
        "/payload_policy/redaction_status",
        TELEMETRY_SOURCE_REF,
        "/payload_policy/redaction_status",
    )?;
    with_read_only_boundary(
        json!({
            "schema_version": "ntpro.v280.trader_terminal.telemetry_health.response.v1",
            "contract_version": API_CONTRACT_VERSION,
            "slo_state": string_pointer(&source, "/slo_rollup/status", TELEMETRY_SOURCE_REF)?,
            "ingestion_freshness": string_pointer(
                &source,
                "/telemetry_source/freshness_status",
                TELEMETRY_SOURCE_REF
            )?,
            "lineage_status": string_pointer(
                &source,
                "/telemetry_source/lineage_status",
                TELEMETRY_SOURCE_REF
            )?,
            "sampling_policy": clone_pointer(
                &source,
                "/sampling_window",
                TELEMETRY_SOURCE_REF
            )?,
            "source_ref": string_pointer(
                &source,
                "/telemetry_source/source_ref",
                TELEMETRY_SOURCE_REF
            )?,
            "redaction_status": "redacted",
            "required_false_boundary_flags": boundaries
        }),
        TELEMETRY_SOURCE_REF,
    )
}

fn project_permission_snapshot(raw: &str) -> ProjectionResult<Value> {
    let source = parse_source(raw, PERMISSION_SOURCE_REF)?;
    let boundaries = required_false_object(
        &source,
        "/required_false_permissions",
        PERMISSION_SOURCE_REF,
    )?;
    require_false(
        &source,
        "/live_operation_authorization",
        PERMISSION_SOURCE_REF,
    )?;
    require_false(
        &source,
        "/production_trading_authorization",
        PERMISSION_SOURCE_REF,
    )?;
    require_redacted(
        &source,
        "/identity_source/redaction_status",
        PERMISSION_SOURCE_REF,
        "/identity_source/redaction_status",
    )?;
    require_redacted(
        &source,
        "/permission_mapping/redaction_status",
        PERMISSION_SOURCE_REF,
        "/permission_mapping/redaction_status",
    )?;
    let roles = object_pointer(&source, "/permission_mapping/roles", PERMISSION_SOURCE_REF)?;
    let principal_roles = roles.keys().cloned().collect::<Vec<_>>();
    let read_scope = roles
        .iter()
        .filter(|(_, role)| {
            role.pointer("/allowed_permissions")
                .and_then(Value::as_array)
                .is_some_and(|permissions| {
                    permissions
                        .iter()
                        .any(|permission| permission.as_str() == Some("dashboard_read"))
                })
        })
        .map(|(role, _)| role.clone())
        .collect::<Vec<_>>();
    let admin_scope = roles
        .iter()
        .filter(|(_, role)| {
            role.pointer("/scope_prefix")
                .and_then(Value::as_str)
                .is_some_and(|scope| scope == "admin:")
        })
        .map(|(role, _)| role.clone())
        .collect::<Vec<_>>();
    with_read_only_boundary(
        json!({
            "schema_version": "ntpro.v280.trader_terminal.permission_snapshot.response.v1",
            "contract_version": API_CONTRACT_VERSION,
            "principal_role": principal_roles,
            "permission_set": roles,
            "read_scope": read_scope,
            "admin_scope": admin_scope,
            "trading_permission_allowed": false,
            "source_ref": PERMISSION_SOURCE_REF,
            "freshness_status": string_pointer(
                &source,
                "/permission_mapping/freshness_status",
                PERMISSION_SOURCE_REF
            )?,
            "redaction_status": "redacted",
            "required_false_boundary_flags": boundaries
        }),
        PERMISSION_SOURCE_REF,
    )
}

fn project_deployment_state(raw: &str) -> ProjectionResult<Value> {
    let source = parse_source(raw, DEPLOYMENT_SOURCE_REF)?;
    let boundaries = required_false_object(&source, "/boundary_flags", DEPLOYMENT_SOURCE_REF)?;
    let transitions = array_pointer(
        &source,
        "/orchestration_plan/state_transitions",
        DEPLOYMENT_SOURCE_REF,
    )?;
    let preview_state = transitions
        .last()
        .ok_or_else(|| {
            malformed(
                DEPLOYMENT_SOURCE_REF,
                "/orchestration_plan/state_transitions must not be empty",
            )
        })
        .and_then(|transition| string_pointer(transition, "/to_status", DEPLOYMENT_SOURCE_REF))?;
    let rollback_state = transitions
        .iter()
        .find(|transition| {
            transition
                .pointer("/operation")
                .and_then(Value::as_str)
                .is_some_and(|operation| operation == "rollback")
        })
        .ok_or_else(|| {
            malformed(
                DEPLOYMENT_SOURCE_REF,
                "/orchestration_plan/state_transitions missing rollback",
            )
        })
        .and_then(|transition| string_pointer(transition, "/to_status", DEPLOYMENT_SOURCE_REF))?;
    with_read_only_boundary(
        json!({
            "schema_version": "ntpro.v280.trader_terminal.deployment_state.response.v1",
            "contract_version": API_CONTRACT_VERSION,
            "deployment_state": string_pointer(
                &source,
                "/backend_module_status",
                DEPLOYMENT_SOURCE_REF
            )?,
            "preview_state": preview_state,
            "approval_status": string_pointer(
                &source,
                "/owner_approval/status",
                DEPLOYMENT_SOURCE_REF
            )?,
            "rollback_state": rollback_state,
            "source_ref": DEPLOYMENT_SOURCE_REF,
            "freshness_status": string_pointer(
                &source,
                "/runbook_provenance/freshness_status",
                DEPLOYMENT_SOURCE_REF
            )?,
            "redaction_status": "redacted",
            "required_false_boundary_flags": boundaries
        }),
        DEPLOYMENT_SOURCE_REF,
    )
}

fn with_read_only_boundary(
    mut response: Value,
    source_ref: &'static str,
) -> ProjectionResult<Value> {
    let object = response
        .as_object_mut()
        .ok_or_else(|| malformed(source_ref, "$response"))?;
    object.insert("read_only".to_string(), Value::Bool(true));
    object.insert("operation_controls_enabled".to_string(), Value::Bool(false));
    object.insert("trading_controls_enabled".to_string(), Value::Bool(false));
    object.insert("order_ticket_enabled".to_string(), Value::Bool(false));
    object.insert(
        "manual_operation_submit_allowed".to_string(),
        Value::Bool(false),
    );
    Ok(response)
}

fn parse_source(raw: &str, source_ref: &'static str) -> ProjectionResult<Value> {
    serde_json::from_str(raw).map_err(|_| malformed(source_ref, "$"))
}

fn clone_pointer(
    source: &Value,
    pointer: &str,
    source_ref: &'static str,
) -> ProjectionResult<Value> {
    source
        .pointer(pointer)
        .cloned()
        .ok_or_else(|| malformed(source_ref, pointer))
}

fn string_pointer<'a>(
    source: &'a Value,
    pointer: &str,
    source_ref: &'static str,
) -> ProjectionResult<&'a str> {
    source
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| malformed(source_ref, pointer))
}

fn object_pointer<'a>(
    source: &'a Value,
    pointer: &str,
    source_ref: &'static str,
) -> ProjectionResult<&'a Map<String, Value>> {
    source
        .pointer(pointer)
        .and_then(Value::as_object)
        .ok_or_else(|| malformed(source_ref, pointer))
}

fn array_pointer<'a>(
    source: &'a Value,
    pointer: &str,
    source_ref: &'static str,
) -> ProjectionResult<&'a Vec<Value>> {
    source
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| malformed(source_ref, pointer))
}

fn required_false_object(
    source: &Value,
    pointer: &str,
    source_ref: &'static str,
) -> ProjectionResult<Value> {
    let boundaries = object_pointer(source, pointer, source_ref)?;
    if boundaries.is_empty() {
        return Err(malformed(source_ref, pointer));
    }
    for (field, value) in boundaries {
        match value.as_bool() {
            Some(false) => {}
            Some(true) => {
                return Err(ProjectionError {
                    kind: ProjectionErrorKind::ForbiddenControls,
                    source_ref,
                    field: format!("{pointer}/{field}"),
                });
            }
            None => return Err(malformed(source_ref, &format!("{pointer}/{field}"))),
        }
    }
    Ok(Value::Object(boundaries.clone()))
}

fn require_false(source: &Value, pointer: &str, source_ref: &'static str) -> ProjectionResult<()> {
    match source.pointer(pointer).and_then(Value::as_bool) {
        Some(false) => Ok(()),
        Some(true) => Err(ProjectionError {
            kind: ProjectionErrorKind::ForbiddenControls,
            source_ref,
            field: pointer.to_string(),
        }),
        None => Err(malformed(source_ref, pointer)),
    }
}

fn require_redacted(
    source: &Value,
    pointer: &str,
    source_ref: &'static str,
    response_field: &str,
) -> ProjectionResult<()> {
    match source.pointer(pointer).and_then(Value::as_str) {
        Some("redacted") => Ok(()),
        Some(_) => Err(ProjectionError {
            kind: ProjectionErrorKind::Unredacted,
            source_ref,
            field: response_field.to_string(),
        }),
        None => Err(malformed(source_ref, response_field)),
    }
}

fn malformed(source_ref: &'static str, field: &str) -> ProjectionError {
    ProjectionError {
        kind: ProjectionErrorKind::Malformed,
        source_ref,
        field: field.to_string(),
    }
}

#[cfg(test)]
mod tests;
