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

//! 单节点 MVP 两个门户共同消费的版本化只读状态投影。

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{Json, extract::State, http::StatusCode};
use nautilus_live::status::HealthStatus;
use serde::Serialize;
use serde_json::{Value, json};

use crate::mvp_contract::{
    MVP_IDENTITY_CONTRACT_PATH, MVP_IDENTITY_CONTRACT_SCHEMA_VERSION, MVP_STATUS_CONTRACT_PATH,
    MVP_STATUS_CONTRACT_SCHEMA_VERSION, MvpIdentityContract, MvpStatusAvailability, MvpStatusAxis,
    MvpStatusContract, MvpStatusFreshness, MvpTechnicalHealth,
};
use crate::supervisor::SupervisorRegistryStore;

use super::{
    ApiResult, DashboardServerState, DashboardValue, TraderTerminalReadModelStatus,
    load_dashboard_snapshot,
};

const MVP_SHARED_STATUS_API_CONTRACT_VERSION: &str = "ntpro.mvp_shared_status_api.v1";
const MVP_SHARED_STATUS_API_SCHEMA_VERSION: &str = "ntpro.mvp_shared_status_api.response.v1";
const MVP_EVENT_CORRELATION_CONTRACT_VERSION: &str = "ntpro.mvp_event_correlation_api.v1";
const MVP_EVENT_CORRELATION_SCHEMA_VERSION: &str = "ntpro.mvp_event_correlation_api.response.v1";
const MAX_CLOCK_SKEW_MS: u64 = 5_000;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct MvpSharedStatusResponse {
    schema_version: String,
    contract_version: String,
    generated_at_unix_ms: u64,
    consumers: Vec<String>,
    identity: MvpIdentityContract,
    status: MvpStatusContract,
    business: MvpBusinessReadModelSummary,
    source_refs: Vec<String>,
    boundaries: MvpSharedStatusBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct MvpEventCorrelationResponse {
    schema_version: String,
    contract_version: String,
    event: MvpCorrelatedStatusEvent,
    links: MvpEventCorrelationLinks,
    boundaries: MvpEventCorrelationBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MvpCorrelatedStatusEvent {
    event_id: String,
    event_kind: String,
    event_source: String,
    identity_contract_id: String,
    node_id: String,
    strategy_instance_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MvpEventCorrelationLinks {
    institution_workbench_path: String,
    control_center_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MvpEventCorrelationBoundaries {
    read_only: bool,
    projected_status_event: bool,
    raw_event_store_exposed: bool,
    raw_event_payload_exposed: bool,
    raw_errors_exposed: bool,
    supervisor_actions_exposed: bool,
    trading_controls_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MvpBusinessReadModelSummary {
    availability: MvpBusinessAvailability,
    health: HealthStatus,
    readiness_status: DashboardValue<String>,
    snapshot_id: DashboardValue<String>,
    schema_version: DashboardValue<String>,
    freshness_status: DashboardValue<String>,
    source_type: DashboardValue<String>,
    source_ref: DashboardValue<String>,
    redaction_state: DashboardValue<String>,
    account: MvpBusinessComponentSummary,
    positions: MvpBusinessComponentSummary,
    orders: MvpBusinessComponentSummary,
    fills: MvpBusinessComponentSummary,
    risk: MvpBusinessComponentSummary,
    lifecycle: MvpBusinessComponentSummary,
    blocking_reasons: DashboardValue<String>,
    diagnostic: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MvpBusinessComponentSummary {
    status: DashboardValue<String>,
    summary: DashboardValue<String>,
    freshness_status: DashboardValue<String>,
    source_ref: DashboardValue<String>,
    redaction_state: DashboardValue<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum MvpBusinessAvailability {
    Available,
    Missing,
    Stale,
    Error,
    IdentityMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MvpSharedStatusBoundaries {
    read_only: bool,
    http_success_implies_technical_health: bool,
    process_alive_implies_technical_health: bool,
    backtest_reference_implies_research_accepted: bool,
    backtest_complete_implies_trading_readiness: bool,
    raw_event_store_exposed: bool,
    raw_venue_payload_exposed: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SharedStatusErrorKind {
    Unavailable,
    Invalid,
    BoundaryViolation,
    IdentityMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SharedStatusError {
    kind: SharedStatusErrorKind,
    source: &'static str,
    field: &'static str,
}

pub(super) async fn mvp_shared_status_api(
    State(state): State<DashboardServerState>,
) -> ApiResult<MvpSharedStatusResponse> {
    project_mvp_shared_status(&state, unix_time_ms())
        .map(Json)
        .map_err(shared_status_error_response)
}

pub(super) async fn mvp_event_correlation_api(
    State(state): State<DashboardServerState>,
) -> ApiResult<MvpEventCorrelationResponse> {
    project_mvp_event_correlation(&state, unix_time_ms())
        .map(Json)
        .map_err(shared_status_error_response)
}

fn project_mvp_event_correlation(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<MvpEventCorrelationResponse, SharedStatusError> {
    let shared = project_mvp_shared_status(state, now_unix_ms)?;
    let identity = &shared.identity.identities;
    let event_id = format!(
        "mvp-status:{}:technical-health",
        shared.identity.contract_id
    );

    Ok(MvpEventCorrelationResponse {
        schema_version: MVP_EVENT_CORRELATION_SCHEMA_VERSION.to_string(),
        contract_version: MVP_EVENT_CORRELATION_CONTRACT_VERSION.to_string(),
        event: MvpCorrelatedStatusEvent {
            event_id,
            event_kind: "technical_health_observation".to_string(),
            event_source: "projected_status_contract".to_string(),
            identity_contract_id: shared.identity.contract_id.clone(),
            node_id: identity.node_id.clone(),
            strategy_instance_id: identity.strategy_instance_id.clone(),
        },
        links: MvpEventCorrelationLinks {
            institution_workbench_path: "/institution-workbench".to_string(),
            control_center_path: "/control-center".to_string(),
        },
        boundaries: MvpEventCorrelationBoundaries {
            read_only: true,
            projected_status_event: true,
            raw_event_store_exposed: false,
            raw_event_payload_exposed: false,
            raw_errors_exposed: false,
            supervisor_actions_exposed: false,
            trading_controls_exposed: false,
        },
    })
}

fn project_mvp_shared_status(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<MvpSharedStatusResponse, SharedStatusError> {
    let workspace = mvp_workspace_root(&state.registry_path)?;
    let identity_path = workspace.join(MVP_IDENTITY_CONTRACT_PATH);
    let status_path = workspace.join(MVP_STATUS_CONTRACT_PATH);
    let identity = load_contract::<MvpIdentityContract>(&identity_path, "identity_contract")?;
    validate_identity_contract(&identity)?;
    let mut status = load_contract::<MvpStatusContract>(&status_path, "status_contract")?;
    validate_status_contract(&status, &identity, &identity_path, &state.registry_path)?;
    validate_runtime_provenance(&status, &identity, &state.registry_path, &workspace)?;
    refresh_contract_freshness(&mut status, now_unix_ms)?;

    let snapshot = load_dashboard_snapshot(state).map_err(|_| SharedStatusError {
        kind: SharedStatusErrorKind::Unavailable,
        source: "supervisor_registry",
        field: "registry",
    })?;
    if !snapshot
        .nodes
        .iter()
        .any(|node| node.node_id == identity.identities.node_id)
    {
        return Err(SharedStatusError {
            kind: SharedStatusErrorKind::IdentityMismatch,
            source: "supervisor_registry",
            field: "node_id",
        });
    }
    let business = snapshot
        .read_model_runtime
        .iter()
        .find(|runtime| runtime.node_id == identity.identities.node_id)
        .map_or_else(
            || MvpBusinessReadModelSummary::missing("read_model_projection_missing"),
            |runtime| business_summary(runtime, &identity, now_unix_ms),
        );

    Ok(MvpSharedStatusResponse {
        schema_version: MVP_SHARED_STATUS_API_SCHEMA_VERSION.to_string(),
        contract_version: MVP_SHARED_STATUS_API_CONTRACT_VERSION.to_string(),
        generated_at_unix_ms: now_unix_ms,
        consumers: vec![
            "institution_workbench".to_string(),
            "control_center".to_string(),
        ],
        identity,
        status,
        business,
        source_refs: vec![
            identity_path.display().to_string(),
            status_path.display().to_string(),
            state.registry_path.display().to_string(),
        ],
        boundaries: MvpSharedStatusBoundaries::read_only(),
    })
}

fn validate_runtime_provenance(
    status: &MvpStatusContract,
    identity: &MvpIdentityContract,
    registry_path: &Path,
    workspace: &Path,
) -> Result<(), SharedStatusError> {
    let registry = SupervisorRegistryStore::new(registry_path)
        .load()
        .map_err(|_| SharedStatusError {
            kind: SharedStatusErrorKind::Unavailable,
            source: "supervisor_registry",
            field: "registry",
        })?;
    let workspace = canonical_path(workspace, "workspace")?;
    let registry_path_canonical = canonical_path(registry_path, "registry")?;
    if registry_path_canonical != workspace.join("supervisor/registry.json") {
        return Err(invalid("supervisor_registry", "workspace_containment"));
    }
    let record = registry
        .nodes
        .get(&identity.identities.node_id)
        .ok_or(SharedStatusError {
            kind: SharedStatusErrorKind::IdentityMismatch,
            source: "supervisor_registry",
            field: "node_id",
        })?;
    let expected_artifact_root = workspace.join("nodes").join(&identity.identities.node_id);
    let canonical_artifact_root = canonical_path(&record.artifact_root, "artifact_root")?;
    if canonical_artifact_root != expected_artifact_root {
        return Err(invalid("supervisor_registry", "artifact_root_containment"));
    }
    let identity_config_path = canonical_path(
        Path::new(&identity.provenance.config_path),
        "identity_config_path",
    )?;
    let registry_config_path = canonical_path(&record.config_path, "registry_config_path")?;
    if identity_config_path != registry_config_path {
        return Err(SharedStatusError {
            kind: SharedStatusErrorKind::IdentityMismatch,
            source: "identity_contract",
            field: "config_path",
        });
    }
    let config_identity =
        MvpIdentityContract::load(&record.config_path, &identity.identities.node_id)
            .map_err(|_| invalid("identity_contract", "config_projection"))?;
    if config_identity.contract_id != identity.contract_id
        || config_identity.identities != identity.identities
        || config_identity.boundaries != identity.boundaries
    {
        return Err(SharedStatusError {
            kind: SharedStatusErrorKind::IdentityMismatch,
            source: "identity_contract",
            field: "config_projection",
        });
    }
    let expected_status = record.artifact_root.join("status.json");
    let expected_metrics = record.artifact_root.join("metrics.json");
    let expected_read_model = record
        .artifact_root
        .join("v0_21/unified_read_model_snapshot.json");
    if record.status_path != expected_status
        || record.metrics_path != expected_metrics
        || Path::new(&status.provenance.node_status_path) != expected_status
        || Path::new(&status.provenance.node_metrics_path) != expected_metrics
        || Path::new(&status.provenance.unified_read_model_path) != expected_read_model
    {
        return Err(invalid("status_contract", "runtime_provenance"));
    }
    validate_artifact_child(
        &expected_status,
        &record.artifact_root,
        &canonical_artifact_root,
        "node_status_path_containment",
    )?;
    validate_artifact_child(
        &expected_metrics,
        &record.artifact_root,
        &canonical_artifact_root,
        "node_metrics_path_containment",
    )?;
    validate_artifact_child(
        &expected_read_model,
        &record.artifact_root,
        &canonical_artifact_root,
        "unified_read_model_path_containment",
    )?;
    Ok(())
}

fn validate_artifact_child(
    path: &Path,
    artifact_root: &Path,
    canonical_artifact_root: &Path,
    field: &'static str,
) -> Result<(), SharedStatusError> {
    let mut existing = path;
    while !existing.exists() {
        existing = existing
            .parent()
            .ok_or_else(|| invalid("supervisor_registry", field))?;
        if !existing.starts_with(artifact_root) {
            return Err(invalid("supervisor_registry", field));
        }
    }
    let relative = existing
        .strip_prefix(artifact_root)
        .map_err(|_| invalid("supervisor_registry", field))?;
    let canonical_existing = canonical_path(existing, field)?;
    if canonical_existing != canonical_artifact_root.join(relative) {
        return Err(invalid("supervisor_registry", field));
    }
    Ok(())
}

fn canonical_path(path: &Path, field: &'static str) -> Result<PathBuf, SharedStatusError> {
    fs::canonicalize(path).map_err(|_| SharedStatusError {
        kind: SharedStatusErrorKind::Unavailable,
        source: "supervisor_registry",
        field,
    })
}

fn mvp_workspace_root(registry_path: &Path) -> Result<PathBuf, SharedStatusError> {
    let registry_dir = registry_path
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "supervisor"));
    registry_dir
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or(SharedStatusError {
            kind: SharedStatusErrorKind::Unavailable,
            source: "supervisor_registry",
            field: "workspace_layout",
        })
}

fn load_contract<T: serde::de::DeserializeOwned>(
    path: &Path,
    source: &'static str,
) -> Result<T, SharedStatusError> {
    let raw = fs::read_to_string(path).map_err(|_| SharedStatusError {
        kind: SharedStatusErrorKind::Unavailable,
        source,
        field: "file",
    })?;
    serde_json::from_str(&raw).map_err(|_| SharedStatusError {
        kind: SharedStatusErrorKind::Invalid,
        source,
        field: "json",
    })
}

fn validate_identity_contract(identity: &MvpIdentityContract) -> Result<(), SharedStatusError> {
    if identity.schema_version != MVP_IDENTITY_CONTRACT_SCHEMA_VERSION {
        return Err(invalid("identity_contract", "schema_version"));
    }
    let identities = &identity.identities;
    if identities.strategy_id.trim().is_empty()
        || identities.strategy_version.trim().is_empty()
        || identities.backtest_run_id.trim().is_empty()
        || identities.backtest_result_ref.trim().is_empty()
        || identities.node_id.trim().is_empty()
        || identities.strategy_instance_id.trim().is_empty()
        || identities.account_id.trim().is_empty()
        || identities.venue_id.trim().is_empty()
        || identity.identities.environment != "sandbox"
        || identities.node_id == identities.strategy_instance_id
        || identity.provenance.config_path.trim().is_empty()
        || identity.provenance.generated_at_unix_ms == 0
    {
        return Err(invalid("identity_contract", "identities"));
    }
    let expected_contract_id = format!(
        "{}:{}:{}",
        identities.node_id, identities.strategy_id, identities.strategy_instance_id
    );
    if identity.contract_id != expected_contract_id {
        return Err(invalid("identity_contract", "contract_id"));
    }
    let boundary = &identity.boundaries;
    if !boundary.read_only_product_contract
        || boundary.external_venue_connection
        || boundary.order_submission_allowed
        || boundary.order_mutation_allowed
        || boundary.automatic_retry_allowed
        || boundary.automatic_remediation_allowed
        || boundary.real_orders_submitted
    {
        return Err(boundary_violation("identity_contract"));
    }
    Ok(())
}

fn validate_status_contract(
    status: &MvpStatusContract,
    identity: &MvpIdentityContract,
    identity_path: &Path,
    registry_path: &Path,
) -> Result<(), SharedStatusError> {
    if status.schema_version != MVP_STATUS_CONTRACT_SCHEMA_VERSION {
        return Err(invalid("status_contract", "schema_version"));
    }
    if status.identity_contract_id != identity.contract_id {
        return Err(SharedStatusError {
            kind: SharedStatusErrorKind::IdentityMismatch,
            source: "status_contract",
            field: "identity_contract_id",
        });
    }
    if Path::new(&status.provenance.identity_contract_path) != identity_path
        || Path::new(&status.provenance.supervisor_registry_path) != registry_path
        || !status.provenance.identity_contract_available
        || status.provenance.freshness_max_age_ms == 0
        || status.provenance.generated_at_unix_ms == 0
    {
        return Err(invalid("status_contract", "provenance"));
    }
    validate_axis(&status.research, "research")?;
    validate_axis(&status.runtime, "runtime")?;
    validate_axis(&status.technical_health, "technical_health")?;
    validate_axis(&status.trading_readiness, "trading_readiness")?;
    if status.technical_health.status == MvpTechnicalHealth::Healthy
        && (status.technical_health.availability != MvpStatusAvailability::Available
            || status.technical_health.freshness != MvpStatusFreshness::Fresh
            || status.technical_health.error.is_some())
    {
        return Err(invalid("status_contract", "technical_health"));
    }
    let boundary = &status.boundaries;
    if !boundary.read_only_product_contract
        || boundary.http_success_implies_technical_health
        || boundary.process_alive_implies_technical_health
        || boundary.backtest_reference_implies_research_accepted
        || boundary.backtest_complete_implies_trading_readiness
        || boundary.external_venue_connection
        || boundary.order_submission_allowed
        || boundary.order_mutation_allowed
        || boundary.automatic_retry_allowed
        || boundary.automatic_remediation_allowed
        || boundary.real_orders_submitted
    {
        return Err(boundary_violation("status_contract"));
    }
    Ok(())
}

fn validate_axis<T>(axis: &MvpStatusAxis<T>, field: &'static str) -> Result<(), SharedStatusError> {
    if axis.source_refs.is_empty()
        || axis.observed_at_unix_ms == 0
        || (axis.error.is_some() && axis.availability != MvpStatusAvailability::Error)
        || (axis.availability == MvpStatusAvailability::Error && axis.error.is_none())
    {
        return Err(invalid("status_contract", field));
    }
    Ok(())
}

fn refresh_contract_freshness(
    status: &mut MvpStatusContract,
    now_unix_ms: u64,
) -> Result<(), SharedStatusError> {
    let generated_at = status.provenance.generated_at_unix_ms;
    if generated_at > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS) {
        return Err(invalid("status_contract", "generated_at_unix_ms"));
    }
    let age_ms = now_unix_ms.saturating_sub(generated_at);
    if age_ms > status.provenance.freshness_max_age_ms {
        mark_axis_stale(
            &mut status.research,
            "shared_api_status_contract_freshness_threshold_exceeded",
        );
        mark_axis_stale(
            &mut status.runtime,
            "shared_api_status_contract_freshness_threshold_exceeded",
        );
        mark_axis_stale(
            &mut status.technical_health,
            "shared_api_status_contract_freshness_threshold_exceeded",
        );
        mark_axis_stale(
            &mut status.trading_readiness,
            "shared_api_status_contract_freshness_threshold_exceeded",
        );
    }
    refresh_axis_freshness(
        &mut status.research,
        now_unix_ms,
        status.provenance.freshness_max_age_ms,
        "research",
    )?;
    refresh_axis_freshness(
        &mut status.runtime,
        now_unix_ms,
        status.provenance.freshness_max_age_ms,
        "runtime",
    )?;
    refresh_axis_freshness(
        &mut status.technical_health,
        now_unix_ms,
        status.provenance.freshness_max_age_ms,
        "technical_health",
    )?;
    refresh_axis_freshness(
        &mut status.trading_readiness,
        now_unix_ms,
        status.provenance.freshness_max_age_ms,
        "trading_readiness",
    )?;
    if status.technical_health.status == MvpTechnicalHealth::Healthy
        && status.technical_health.freshness != MvpStatusFreshness::Fresh
    {
        status.technical_health.status = MvpTechnicalHealth::Degraded;
    }
    Ok(())
}

fn refresh_axis_freshness<T>(
    axis: &mut MvpStatusAxis<T>,
    now_unix_ms: u64,
    max_age_ms: u64,
    field: &'static str,
) -> Result<(), SharedStatusError> {
    if axis.observed_at_unix_ms > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS) {
        return Err(invalid("status_contract", field));
    }
    if now_unix_ms.saturating_sub(axis.observed_at_unix_ms) > max_age_ms {
        mark_axis_stale(axis, "shared_api_axis_freshness_threshold_exceeded");
    }
    Ok(())
}

fn mark_axis_stale<T>(axis: &mut MvpStatusAxis<T>, reason: &str) {
    axis.freshness = MvpStatusFreshness::Stale;
    if !axis.reasons.iter().any(|item| item == reason) {
        axis.reasons.push(reason.to_string());
    }
}

fn business_summary(
    runtime: &TraderTerminalReadModelStatus,
    identity: &MvpIdentityContract,
    now_unix_ms: u64,
) -> MvpBusinessReadModelSummary {
    let mut summary = MvpBusinessReadModelSummary::from_runtime(runtime);
    let Some(path) = runtime.artifact_path.value.as_deref() else {
        return MvpBusinessReadModelSummary::missing("read_model_artifact_path_missing");
    };
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return MvpBusinessReadModelSummary::missing("unified_read_model_missing");
        }
        Err(_) => return summary.fail_closed("unified_read_model_unreadable"),
    };
    let value: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => return summary.fail_closed("unified_read_model_invalid_json"),
    };
    let account_id = value
        .pointer("/snapshot_identity/account_id")
        .and_then(Value::as_str);
    let venue_id = value
        .pointer("/snapshot_identity/venue")
        .and_then(Value::as_str);
    if account_id != Some(identity.identities.account_id.as_str())
        || venue_id != Some(identity.identities.venue_id.as_str())
    {
        return MvpBusinessReadModelSummary::identity_mismatch();
    }

    for component in ["account", "positions"] {
        let component_account_id = value
            .pointer(&format!("/components/{component}/data/account_id"))
            .and_then(Value::as_str);
        if component_account_id != Some(identity.identities.account_id.as_str()) {
            return MvpBusinessReadModelSummary::identity_mismatch();
        }
    }
    let positions_venue = value
        .pointer("/components/positions/data/instrument_identity/venue")
        .and_then(Value::as_str);
    if positions_venue != Some(identity.identities.venue_id.as_str()) {
        return MvpBusinessReadModelSummary::identity_mismatch();
    }
    for component in ["orders", "fills"] {
        let component_venue = value
            .pointer(&format!(
                "/components/{component}/data/instrument_identity/venue"
            ))
            .and_then(Value::as_str);
        if component_venue.is_some_and(|venue| venue != identity.identities.venue_id) {
            return MvpBusinessReadModelSummary::identity_mismatch();
        }
    }

    let top_freshness = match assess_read_model_freshness(&value, "/freshness", now_unix_ms) {
        Ok(assessment) => assessment,
        Err(reason) => return summary.fail_closed(reason),
    };
    if top_freshness == ReadModelFreshnessAssessment::Stale
        && matches!(
            summary.availability,
            MvpBusinessAvailability::Available | MvpBusinessAvailability::Stale
        )
    {
        summary.mark_stale("read_model_freshness_threshold_exceeded");
    }
    for (component, projection) in [
        ("account", BusinessComponent::Account),
        ("positions", BusinessComponent::Positions),
        ("orders", BusinessComponent::Orders),
        ("fills", BusinessComponent::Fills),
        ("risk", BusinessComponent::Risk),
        ("lifecycle_status", BusinessComponent::Lifecycle),
    ] {
        let pointer = format!("/components/{component}/freshness");
        let assessment = match assess_read_model_freshness(&value, &pointer, now_unix_ms) {
            Ok(assessment) => assessment,
            Err(reason) => return summary.fail_closed(reason),
        };
        if assessment == ReadModelFreshnessAssessment::Stale {
            summary.mark_component_stale(projection, component);
        }
    }
    summary
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadModelFreshnessAssessment {
    Fresh,
    Stale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BusinessComponent {
    Account,
    Positions,
    Orders,
    Fills,
    Risk,
    Lifecycle,
}

fn assess_read_model_freshness(
    value: &Value,
    pointer: &str,
    now_unix_ms: u64,
) -> Result<ReadModelFreshnessAssessment, &'static str> {
    let freshness = value
        .pointer(pointer)
        .ok_or("read_model_freshness_missing")?;
    let as_of_unix_ns = freshness
        .get("as_of_unix_ns")
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<u128>().ok())
        .ok_or("read_model_freshness_timestamp_invalid")?;
    let checked_at_unix_ns = freshness
        .get("checked_at_unix_ns")
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<u128>().ok())
        .ok_or("read_model_freshness_checked_at_invalid")?;
    let max_age_ms = freshness
        .get("max_age_ms")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or("read_model_freshness_threshold_invalid")?;
    if checked_at_unix_ns < as_of_unix_ns {
        return Err("read_model_freshness_order_invalid");
    }
    let now_unix_ns = u128::from(now_unix_ms) * 1_000_000;
    let max_future_ns = u128::from(MAX_CLOCK_SKEW_MS) * 1_000_000;
    if as_of_unix_ns > now_unix_ns + max_future_ns
        || checked_at_unix_ns > now_unix_ns + max_future_ns
    {
        return Err("read_model_freshness_timestamp_in_future");
    }
    let age_ms = now_unix_ns.saturating_sub(as_of_unix_ns) / 1_000_000;
    Ok(if age_ms > u128::from(max_age_ms) {
        ReadModelFreshnessAssessment::Stale
    } else {
        ReadModelFreshnessAssessment::Fresh
    })
}

impl MvpBusinessReadModelSummary {
    fn from_runtime(runtime: &TraderTerminalReadModelStatus) -> Self {
        let availability = match runtime.health {
            HealthStatus::Healthy => MvpBusinessAvailability::Available,
            HealthStatus::Stale => MvpBusinessAvailability::Stale,
            HealthStatus::Error => MvpBusinessAvailability::Error,
            HealthStatus::Degraded | HealthStatus::Unknown => {
                if runtime
                    .diagnostic
                    .value
                    .as_deref()
                    .is_some_and(|value| value.contains("missing"))
                {
                    MvpBusinessAvailability::Missing
                } else {
                    MvpBusinessAvailability::Error
                }
            }
        };
        Self {
            availability,
            health: runtime.health,
            readiness_status: runtime.readiness_status.clone(),
            snapshot_id: runtime.snapshot_id.clone(),
            schema_version: runtime.schema_version.clone(),
            freshness_status: runtime.freshness_status.clone(),
            source_type: runtime.source_type.clone(),
            source_ref: runtime.source_ref.clone(),
            redaction_state: runtime.redaction_state.clone(),
            account: MvpBusinessComponentSummary {
                status: runtime.account_status.clone(),
                summary: runtime.account_summary.clone(),
                freshness_status: runtime.account_freshness_status.clone(),
                source_ref: runtime.account_source_ref.clone(),
                redaction_state: runtime.account_redaction_state.clone(),
            },
            positions: MvpBusinessComponentSummary {
                status: runtime.positions_status.clone(),
                summary: runtime.positions_summary.clone(),
                freshness_status: runtime.positions_freshness_status.clone(),
                source_ref: runtime.positions_source_ref.clone(),
                redaction_state: runtime.positions_redaction_state.clone(),
            },
            orders: MvpBusinessComponentSummary {
                status: runtime.orders_status.clone(),
                summary: runtime.orders_summary.clone(),
                freshness_status: runtime.orders_freshness_status.clone(),
                source_ref: runtime.orders_source_ref.clone(),
                redaction_state: runtime.orders_redaction_state.clone(),
            },
            fills: MvpBusinessComponentSummary {
                status: runtime.fills_status.clone(),
                summary: runtime.fills_summary.clone(),
                freshness_status: runtime.fills_freshness_status.clone(),
                source_ref: runtime.fills_source_ref.clone(),
                redaction_state: runtime.fills_redaction_state.clone(),
            },
            risk: MvpBusinessComponentSummary {
                status: runtime.risk_status.clone(),
                summary: runtime.risk_summary.clone(),
                freshness_status: runtime.risk_freshness_status.clone(),
                source_ref: runtime.risk_source_ref.clone(),
                redaction_state: runtime.risk_redaction_state.clone(),
            },
            lifecycle: MvpBusinessComponentSummary {
                status: runtime.lifecycle_status.clone(),
                summary: runtime.lifecycle_summary.clone(),
                freshness_status: runtime.audit_freshness_status.clone(),
                source_ref: runtime.audit_source_ref.clone(),
                redaction_state: runtime.audit_redaction_state.clone(),
            },
            blocking_reasons: runtime.blocking_reasons.clone(),
            diagnostic: runtime.diagnostic.clone(),
        }
    }

    fn missing(reason: &str) -> Self {
        Self {
            availability: MvpBusinessAvailability::Missing,
            health: HealthStatus::Unknown,
            readiness_status: DashboardValue::available("missing_artifact".to_string()),
            snapshot_id: DashboardValue::unknown(),
            schema_version: DashboardValue::unknown(),
            freshness_status: DashboardValue::unknown(),
            source_type: DashboardValue::unknown(),
            source_ref: DashboardValue::unknown(),
            redaction_state: DashboardValue::unknown(),
            account: MvpBusinessComponentSummary::unknown(),
            positions: MvpBusinessComponentSummary::unknown(),
            orders: MvpBusinessComponentSummary::unknown(),
            fills: MvpBusinessComponentSummary::unknown(),
            risk: MvpBusinessComponentSummary::unknown(),
            lifecycle: MvpBusinessComponentSummary::unknown(),
            blocking_reasons: DashboardValue::available(
                "unified_read_model_unavailable".to_string(),
            ),
            diagnostic: DashboardValue::available(reason.to_string()),
        }
    }

    fn identity_mismatch() -> Self {
        let mut summary = Self::missing("unified_read_model_identity_mismatch");
        summary.availability = MvpBusinessAvailability::IdentityMismatch;
        summary.health = HealthStatus::Error;
        summary.readiness_status = DashboardValue::available("fail_closed".to_string());
        summary
    }

    fn fail_closed(mut self, reason: &str) -> Self {
        self.availability = MvpBusinessAvailability::Error;
        self.health = HealthStatus::Error;
        self.readiness_status = DashboardValue::available("fail_closed".to_string());
        self.diagnostic = DashboardValue::available(reason.to_string());
        self
    }

    fn mark_stale(&mut self, reason: &str) {
        self.availability = MvpBusinessAvailability::Stale;
        self.health = HealthStatus::Stale;
        self.readiness_status = DashboardValue::available("stale_artifact".to_string());
        self.freshness_status = DashboardValue::available("stale".to_string());
        self.diagnostic = DashboardValue::available(reason.to_string());
        for component in [
            &mut self.account,
            &mut self.positions,
            &mut self.orders,
            &mut self.fills,
            &mut self.risk,
            &mut self.lifecycle,
        ] {
            component.freshness_status = DashboardValue::available("stale".to_string());
        }
    }

    fn mark_component_stale(&mut self, component: BusinessComponent, name: &str) {
        self.availability = MvpBusinessAvailability::Stale;
        self.health = HealthStatus::Stale;
        self.readiness_status = DashboardValue::available("stale_artifact".to_string());
        self.diagnostic = DashboardValue::available(format!(
            "read_model_component_freshness_threshold_exceeded:{name}"
        ));
        let component = match component {
            BusinessComponent::Account => &mut self.account,
            BusinessComponent::Positions => &mut self.positions,
            BusinessComponent::Orders => &mut self.orders,
            BusinessComponent::Fills => &mut self.fills,
            BusinessComponent::Risk => &mut self.risk,
            BusinessComponent::Lifecycle => &mut self.lifecycle,
        };
        component.freshness_status = DashboardValue::available("stale".to_string());
    }
}

impl MvpBusinessComponentSummary {
    fn unknown() -> Self {
        Self {
            status: DashboardValue::unknown(),
            summary: DashboardValue::unknown(),
            freshness_status: DashboardValue::unknown(),
            source_ref: DashboardValue::unknown(),
            redaction_state: DashboardValue::unknown(),
        }
    }
}

impl MvpSharedStatusBoundaries {
    const fn read_only() -> Self {
        Self {
            read_only: true,
            http_success_implies_technical_health: false,
            process_alive_implies_technical_health: false,
            backtest_reference_implies_research_accepted: false,
            backtest_complete_implies_trading_readiness: false,
            raw_event_store_exposed: false,
            raw_venue_payload_exposed: false,
            external_venue_connection: false,
            order_submission_allowed: false,
            order_mutation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            real_orders_submitted: false,
        }
    }
}

fn invalid(source: &'static str, field: &'static str) -> SharedStatusError {
    SharedStatusError {
        kind: SharedStatusErrorKind::Invalid,
        source,
        field,
    }
}

fn boundary_violation(source: &'static str) -> SharedStatusError {
    SharedStatusError {
        kind: SharedStatusErrorKind::BoundaryViolation,
        source,
        field: "boundaries",
    }
}

fn shared_status_error_response(error: SharedStatusError) -> (StatusCode, Json<Value>) {
    let (status, code) = match error.kind {
        SharedStatusErrorKind::Unavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "mvp_status_source_unavailable",
        ),
        SharedStatusErrorKind::Invalid => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "mvp_status_source_invalid",
        ),
        SharedStatusErrorKind::BoundaryViolation => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "mvp_status_boundary_violation",
        ),
        SharedStatusErrorKind::IdentityMismatch => {
            (StatusCode::CONFLICT, "mvp_status_identity_mismatch")
        }
    };
    (
        status,
        Json(json!({
            "schema_version": "ntpro.mvp_shared_status_api.error.v1",
            "error_code": code,
            "message": "MVP 共享只读状态来源未通过合同校验",
            "source": error.source,
            "field": error.field,
            "read_only": true,
            "raw_event_store_exposed": false,
            "raw_venue_payload_exposed": false,
            "external_venue_connection": false,
            "order_submission_allowed": false,
            "order_mutation_allowed": false,
            "automatic_retry_allowed": false,
            "automatic_remediation_allowed": false,
            "real_orders_submitted": false
        })),
    )
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}
