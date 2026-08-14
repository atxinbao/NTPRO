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

//! 策略工作台首批产品资源的只读 HTTP 合同。

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering as AtomicOrdering},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json,
    extract::{Path as AxumPath, RawQuery, State, rejection::PathRejection},
    http::{HeaderValue, StatusCode, header::ALLOW},
    response::{IntoResponse, Response},
};
use nautilus_live::status::{
    LifecycleStatus, NODE_STATUS_SCHEMA_VERSION, NodeStatus, SnapshotAvailability, SnapshotValue,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    artifacts::atomic_write_json,
    mvp_contract::{
        MVP_IDENTITY_CONTRACT_PATH, MVP_IDENTITY_CONTRACT_SCHEMA_VERSION, MVP_STATUS_CONTRACT_PATH,
        MVP_STATUS_CONTRACT_SCHEMA_VERSION, MvpIdentityContract, MvpStatusContract,
    },
    supervisor::{
        NODE_METRICS_SCHEMA_VERSION, NodeMetrics, RegistryArtifactState,
        SUPERVISOR_REGISTRY_SCHEMA_VERSION, SupervisorNodeRecord, SupervisorProcessState,
        SupervisorRegistry, SupervisorRegistryStore,
    },
};

use super::{ApiResult, DashboardServerState};

mod live_admission;
mod live_run;
pub(crate) mod live_run_anchor;
mod run;
mod strategy_version;
#[cfg(test)]
mod tests;

pub(super) use live_admission::{live_account_refresh_api, live_admission_api};
pub(super) use live_run::{
    live_execution_cancel_operator_approval_api, live_execution_cancel_owner_approval_api,
    live_execution_operator_approval_api, live_execution_owner_approval_api,
    live_execution_risk_approval_api, live_run_candidate_action_api, live_run_candidate_create_api,
    live_run_candidate_detail_api, live_run_candidate_list_api,
};
pub(crate) use run::shutdown_active_demo_run;
pub(super) use run::{
    demo_run_action_api, demo_run_create_api, demo_run_snapshot_api, run_analysis_api,
    run_comparison_api, run_create_api, run_detail_api, run_list_api, run_metrics_api,
    run_report_api, run_reproduce_api, run_reproduction_proof_api,
};
pub(super) use strategy_version::{strategy_version_detail_api, strategy_version_list_api};

const PRODUCT_API_CONTRACT_VERSION: &str = "ntpro.product_api.v1";
const STRATEGY_LIST_SCHEMA_VERSION: &str = "ntpro.product_api.strategy_list.response.v1";
const STRATEGY_DETAIL_SCHEMA_VERSION: &str = "ntpro.product_api.strategy_detail.response.v1";
const PRODUCT_ERROR_SCHEMA_VERSION: &str = "ntpro.product_api.error.v1";
const MAX_CLOCK_SKEW_MS: u64 = 5_000;
const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;
const CURSOR_PREFIX: &str = "strategy-v1-";
static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
const PRODUCT_OPENAPI_SOURCE: &str =
    include_str!("../../../../docs/product/api/ntpro_product_v1.openapi.json");

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct ProductStrategy {
    strategy_id: String,
    name: String,
    description: String,
    owner: String,
    lifecycle: StrategyLifecycle,
    default_version_id: String,
    created_at_unix_ms: u64,
    updated_at_unix_ms: u64,
    source: ProductSource,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StrategyLifecycle {
    Draft,
    Active,
    Archived,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct ProductSource {
    source_type: String,
    freshness_status: String,
    source_refs: Vec<String>,
}

#[derive(Debug)]
struct ValidatedProductSource {
    strategy: ProductStrategy,
    raw_config: String,
    identity: MvpIdentityContract,
    config_name: String,
    runtime_record: SupervisorNodeRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ProductReadOnlyBoundaries {
    read_only: bool,
    strategy_mutation_allowed: bool,
    run_mutation_allowed: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

impl ProductReadOnlyBoundaries {
    const fn enforced() -> Self {
        Self {
            read_only: true,
            strategy_mutation_allowed: false,
            run_mutation_allowed: false,
            external_venue_connection: false,
            order_submission_allowed: false,
            order_mutation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            real_orders_submitted: false,
            trading_controls_enabled: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct StrategyListResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: Vec<ProductStrategy>,
    page: ProductPage,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct StrategyDetailResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: ProductStrategy,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ProductPage {
    limit: usize,
    returned_count: usize,
    next_cursor: Option<String>,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
struct ProductConfigProjection {
    strategy: ProductStrategyConfig,
    mvp: ProductMvpConfig,
}

#[derive(Debug, Deserialize)]
struct ProductStrategyConfig {
    strategy_id: String,
    display_name: String,
    description: String,
    owner: String,
    lifecycle: StrategyLifecycle,
    created_at_unix_ms: u64,
    updated_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
struct ProductMvpConfig {
    strategy_version: String,
}

#[derive(Clone, Copy)]
struct RuntimeArtifactContract<'a> {
    path: &'a Path,
    artifact_state: RegistryArtifactState,
    process_state: SupervisorProcessState,
    expected_node_id: &'a str,
    field: &'a str,
    expected_schema_version: &'a str,
    now_unix_ms: u64,
    freshness_max_age_ms: u64,
    enforce_freshness: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProductErrorKind {
    BadRequest,
    Conflict,
    DemoConflict,
    ExecutionFailed,
    DemoExecutionFailed,
    LiveConflict,
    LiveExecutionFailed,
    Forbidden,
    MethodNotAllowed,
    NotFound,
    VersionNotFound,
    RunNotFound,
    SourceUnavailable,
    SourceInvalid,
    SourceStale,
    BoundaryViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProductError {
    kind: ProductErrorKind,
    field: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StrategyListQuery {
    limit: usize,
    cursor: Option<String>,
    sort: StrategySort,
    order: SortOrder,
    lifecycle: Option<StrategyLifecycle>,
    owner: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StrategySort {
    StrategyId,
    Name,
    UpdatedAt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortOrder {
    Asc,
    Desc,
}

pub(super) async fn strategy_list_api(
    State(state): State<DashboardServerState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<StrategyListResponse> {
    let request_id = product_request_id();
    let result = parse_strategy_list_query(raw_query.as_deref()).and_then(|query| {
        let strategy = load_product_catalog_strategy(&state, unix_time_ms())?;
        project_strategy_list(strategy, &query, request_id.clone())
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(super) async fn strategy_detail_api(
    State(state): State<DashboardServerState>,
    strategy_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<StrategyDetailResponse> {
    let request_id = product_request_id();
    let strategy_id = strategy_path
        .map(|AxumPath(strategy_id)| strategy_id)
        .map_err(|_| {
            product_error_response(
                &product_error(ProductErrorKind::BadRequest, "strategy_id"),
                &request_id,
            )
        })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_identifier("strategy_id", &strategy_id)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "strategy_id"))?;
        let strategy = load_product_catalog_strategy(&state, unix_time_ms())?;
        if strategy.strategy_id != strategy_id {
            return Err(product_error(ProductErrorKind::NotFound, "strategy_id"));
        }
        Ok(StrategyDetailResponse {
            schema_version: STRATEGY_DETAIL_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: strategy,
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(super) fn product_access_denied_response() -> Response {
    let request_id = product_request_id();
    product_error_response(
        &product_error(ProductErrorKind::Forbidden, "portal_access"),
        &request_id,
    )
    .into_response()
}

pub(super) async fn product_method_not_allowed() -> Response {
    let request_id = product_request_id();
    let mut response = product_error_response(
        &product_error(ProductErrorKind::MethodNotAllowed, "method"),
        &request_id,
    )
    .into_response();
    response
        .headers_mut()
        .insert(ALLOW, HeaderValue::from_static("GET"));
    response
}

pub(super) async fn product_run_method_not_allowed() -> Response {
    let request_id = product_request_id();
    let mut response = product_error_response(
        &product_error(ProductErrorKind::MethodNotAllowed, "method"),
        &request_id,
    )
    .into_response();
    response
        .headers_mut()
        .insert(ALLOW, HeaderValue::from_static("GET, POST"));
    response
}

pub(super) async fn product_command_method_not_allowed() -> Response {
    let request_id = product_request_id();
    let mut response = product_error_response(
        &product_error(ProductErrorKind::MethodNotAllowed, "method"),
        &request_id,
    )
    .into_response();
    response
        .headers_mut()
        .insert(ALLOW, HeaderValue::from_static("POST"));
    response
}

#[cfg(test)]
fn load_product_strategy(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<ProductStrategy, ProductError> {
    Ok(load_product_source(state, now_unix_ms)?.strategy)
}

fn load_product_catalog_strategy(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<ProductStrategy, ProductError> {
    Ok(load_product_catalog_source(state, now_unix_ms)?.strategy)
}

fn load_product_source(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<ValidatedProductSource, ProductError> {
    let source = load_product_catalog_source(state, now_unix_ms)?;
    let record = &source.runtime_record;
    let workspace = mvp_workspace_root(&state.registry_path)?;
    let identity_path = workspace.join(MVP_IDENTITY_CONTRACT_PATH);
    let enforce_runtime_freshness =
        !runtime_snapshot_is_stationary(state, record, &source.identity, now_unix_ms)?;
    let freshness_max_age_ms = validate_product_status_contract(
        &workspace,
        &identity_path,
        &state.registry_path,
        record,
        &source.identity,
        now_unix_ms,
        enforce_runtime_freshness,
    )?;
    validate_runtime_boundaries(
        record,
        &source.identity.identities.node_id,
        now_unix_ms,
        freshness_max_age_ms,
        enforce_runtime_freshness,
    )?;
    Ok(source)
}

fn load_product_catalog_source(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<ValidatedProductSource, ProductError> {
    let workspace = mvp_workspace_root(&state.registry_path)?;
    let identity_path = workspace.join(MVP_IDENTITY_CONTRACT_PATH);
    let identity: MvpIdentityContract = load_json(&identity_path, "identity_contract")?;
    validate_product_identity(&identity, now_unix_ms)?;

    let registry = load_product_registry(&state.registry_path)?;
    if registry.schema_version != SUPERVISOR_REGISTRY_SCHEMA_VERSION {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "registry_schema_version",
        ));
    }
    let record = registry
        .nodes
        .get(&identity.identities.node_id)
        .ok_or_else(|| {
            if state.registry_path.is_file() {
                product_error(ProductErrorKind::SourceInvalid, "node_id")
            } else {
                product_error(ProductErrorKind::SourceUnavailable, "registry")
            }
        })?;
    if record.node_id != identity.identities.node_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "registry_node_id",
        ));
    }
    validate_product_runtime_provenance(
        &workspace,
        &state.registry_path,
        record,
        &identity.identities.node_id,
    )?;
    let identity_config_path = PathBuf::from(&identity.provenance.config_path);
    if canonical_path(&identity_config_path, "config_path")?
        != canonical_path(&record.config_path, "registry_config_path")?
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "config_identity",
        ));
    }
    let raw = read_stable_config_snapshot(
        &identity_config_path,
        &identity_path,
        identity.provenance.generated_at_unix_ms,
    )?;
    let projected_identity =
        MvpIdentityContract::load_from_str(&record.config_path, &raw, &identity.identities.node_id)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "config_projection"))?;
    if projected_identity.contract_id != identity.contract_id
        || projected_identity.identities != identity.identities
        || projected_identity.boundaries != identity.boundaries
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "config_projection",
        ));
    }
    let config: ProductConfigProjection = toml::from_str(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "strategy_config"))?;
    validate_product_config(&config, &identity, now_unix_ms)?;
    let config_name = identity_config_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "config_name"))?;
    let strategy = ProductStrategy {
        strategy_id: config.strategy.strategy_id,
        name: config.strategy.display_name,
        description: config.strategy.description,
        owner: config.strategy.owner,
        lifecycle: config.strategy.lifecycle,
        default_version_id: strategy_version_resource_id(
            &identity.identities.strategy_id,
            &config.mvp.strategy_version,
        ),
        created_at_unix_ms: config.strategy.created_at_unix_ms,
        updated_at_unix_ms: config.strategy.updated_at_unix_ms,
        source: ProductSource {
            source_type: "mvp_identity_config".to_string(),
            freshness_status: "fresh".to_string(),
            source_refs: vec![
                MVP_IDENTITY_CONTRACT_PATH.to_string(),
                format!("node-config:{config_name}"),
            ],
        },
    };
    Ok(ValidatedProductSource {
        strategy,
        raw_config: raw,
        identity,
        config_name: config_name.to_string(),
        runtime_record: record.clone(),
    })
}

fn validate_product_runtime_provenance(
    workspace: &Path,
    registry_path: &Path,
    record: &SupervisorNodeRecord,
    node_id: &str,
) -> Result<(), ProductError> {
    let canonical_workspace = canonical_path(workspace, "workspace")?;
    let canonical_registry = canonical_path(registry_path, "registry")?;
    if canonical_registry != canonical_workspace.join("supervisor/registry.json") {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "workspace_containment",
        ));
    }

    let canonical_artifact_root = canonical_path(&record.artifact_root, "artifact_root")?;
    let expected_artifact_root = canonical_workspace.join("nodes").join(node_id);
    if canonical_artifact_root != expected_artifact_root {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "artifact_root_containment",
        ));
    }

    for (field, actual, file_name) in [
        (
            "node_status_path_containment",
            &record.status_path,
            "status.json",
        ),
        (
            "node_metrics_path_containment",
            &record.metrics_path,
            "metrics.json",
        ),
    ] {
        if actual != &record.artifact_root.join(file_name)
            || canonical_contract_path(actual, field)? != canonical_artifact_root.join(file_name)
        {
            return Err(product_error(ProductErrorKind::SourceInvalid, field));
        }
    }
    Ok(())
}

fn validate_product_status_contract(
    workspace: &Path,
    identity_path: &Path,
    registry_path: &Path,
    record: &SupervisorNodeRecord,
    identity: &MvpIdentityContract,
    now_unix_ms: u64,
    enforce_freshness: bool,
) -> Result<u64, ProductError> {
    let status_path = workspace.join(MVP_STATUS_CONTRACT_PATH);
    let status: MvpStatusContract = load_json(&status_path, "status_contract")?;
    if status.schema_version != MVP_STATUS_CONTRACT_SCHEMA_VERSION
        || status.identity_contract_id != identity.contract_id
        || !status.provenance.identity_contract_available
        || status.provenance.freshness_max_age_ms == 0
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "status_contract",
        ));
    }
    for (field, actual, expected) in [
        (
            "status_identity_path",
            status.provenance.identity_contract_path.as_str(),
            identity_path,
        ),
        (
            "status_registry_path",
            status.provenance.supervisor_registry_path.as_str(),
            registry_path,
        ),
        (
            "status_node_path",
            status.provenance.node_status_path.as_str(),
            record.status_path.as_path(),
        ),
        (
            "status_metrics_path",
            status.provenance.node_metrics_path.as_str(),
            record.metrics_path.as_path(),
        ),
    ] {
        if canonical_contract_path(Path::new(actual), field)?
            != canonical_contract_path(expected, field)?
        {
            return Err(product_error(ProductErrorKind::SourceInvalid, field));
        }
    }
    let generated_at = status.provenance.generated_at_unix_ms;
    if generated_at == 0 || generated_at > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "status_contract_timestamp",
        ));
    }
    if enforce_freshness
        && now_unix_ms.saturating_sub(generated_at) > status.provenance.freshness_max_age_ms
    {
        return Err(product_error(
            ProductErrorKind::SourceStale,
            "status_contract_timestamp",
        ));
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
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "status_contract_boundaries",
        ));
    }
    Ok(status.provenance.freshness_max_age_ms)
}

fn refresh_product_status_contract(
    state: &DashboardServerState,
    node_id: &str,
) -> Result<(), ProductError> {
    let workspace = mvp_workspace_root(&state.registry_path)?;
    let identity_path = workspace.join(MVP_IDENTITY_CONTRACT_PATH);
    let status_path = workspace.join(MVP_STATUS_CONTRACT_PATH);
    let identity: MvpIdentityContract = load_json(&identity_path, "identity_contract")?;
    let previous: MvpStatusContract = load_json(&status_path, "status_contract")?;
    let freshness_max_age_ms = previous.provenance.freshness_max_age_ms;
    if freshness_max_age_ms == 0 || identity.identities.node_id != node_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "status_contract",
        ));
    }

    let store = SupervisorRegistryStore::new(&state.registry_path);
    let status_error = store
        .refresh_status_from_artifact(node_id)
        .err()
        .map(|error| error.to_string());
    let metrics_result = store.node_metrics(node_id);
    let metrics_error = metrics_result.as_ref().err().map(ToString::to_string);
    let registry = store
        .load()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "registry"))?;
    let record = registry
        .nodes
        .get(node_id)
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "node_id"))?;
    let contract = MvpStatusContract::from_runtime(
        &identity,
        &identity_path,
        &state.registry_path,
        record,
        metrics_result.as_ref().ok(),
        status_error.as_deref(),
        metrics_error.as_deref(),
        None,
        freshness_max_age_ms,
    );
    atomic_write_json(&status_path, &contract)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "status_contract"))
}

fn runtime_snapshot_is_stationary(
    state: &DashboardServerState,
    record: &SupervisorNodeRecord,
    identity: &MvpIdentityContract,
    now_unix_ms: u64,
) -> Result<bool, ProductError> {
    let prepared_without_runtime_artifacts = record.process.state
        == SupervisorProcessState::NotStarted
        && record.last_known_status.lifecycle_state == LifecycleStatus::Stopped
        && record.status_artifact == RegistryArtifactState::Missing
        && record.metrics_artifact == RegistryArtifactState::Missing;
    let stopped_runtime = record.process.state == SupervisorProcessState::Stopped
        && record.last_known_status.lifecycle_state == LifecycleStatus::Stopped;
    if !prepared_without_runtime_artifacts && !stopped_runtime {
        return Ok(false);
    }

    let mut active_ownership = None;
    for (run_id, ownership) in &record.run_ownership {
        if ownership.run_id != *run_id
            || ownership.claimed_at_unix_ms == 0
            || ownership.claimed_at_unix_ms > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS)
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_run_ownership",
            ));
        }
        validate_identifier("demo_run_ownership", run_id)?;
        validate_sha256_hash("demo_run_manifest_sha256", &ownership.manifest_sha256)?;
        if let Some(terminal) = &ownership.terminal {
            if !matches!(terminal.lifecycle.as_str(), "stopped" | "failed")
                || terminal.completed_at_unix_ms < ownership.claimed_at_unix_ms
                || terminal.completed_at_unix_ms > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS)
            {
                return Err(product_error(
                    ProductErrorKind::SourceInvalid,
                    "demo_run_terminal",
                ));
            }
            validate_sha256_hash(
                "demo_run_terminal_state_sha256",
                &terminal.terminal_state_sha256,
            )?;
        } else if active_ownership.replace(ownership).is_some() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_run_ownership",
            ));
        }
    }
    let pending_unstarted_ownership = if let Some(ownership) = active_ownership {
        if record.process.pid.value.is_some()
            || snapshot_is_at_or_after_claim(
                &record.last_known_status.started_at,
                ownership.claimed_at_unix_ms,
            )
            || snapshot_is_at_or_after_claim(
                &record.last_known_status.stopped_at,
                ownership.claimed_at_unix_ms,
            )
        {
            false
        } else {
            run::validate_unstarted_demo_ownership(state, identity, record, ownership)?;
            true
        }
    } else {
        false
    };
    Ok((prepared_without_runtime_artifacts || stopped_runtime)
        && (active_ownership.is_none() || pending_unstarted_ownership))
}

fn snapshot_is_at_or_after_claim(value: &SnapshotValue<String>, claimed_at_unix_ms: u64) -> bool {
    value.value.as_deref().is_some_and(|raw| {
        raw.parse::<u64>()
            .map_or(true, |timestamp| timestamp >= claimed_at_unix_ms)
    })
}

fn validate_product_identity(
    identity: &MvpIdentityContract,
    now_unix_ms: u64,
) -> Result<(), ProductError> {
    if identity.schema_version != MVP_IDENTITY_CONTRACT_SCHEMA_VERSION {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "identity_schema_version",
        ));
    }
    validate_identifier("strategy_id", &identity.identities.strategy_id)?;
    validate_text(
        "strategy_version",
        &identity.identities.strategy_version,
        256,
    )?;
    validate_identifier("node_id", &identity.identities.node_id)?;
    validate_identifier(
        "strategy_instance_id",
        &identity.identities.strategy_instance_id,
    )?;
    if identity.identities.node_id == identity.identities.strategy_instance_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "identity_ownership",
        ));
    }
    if identity.provenance.config_path.trim().is_empty()
        || identity.provenance.generated_at_unix_ms == 0
        || identity.provenance.generated_at_unix_ms > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS)
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "identity_provenance",
        ));
    }
    let expected_contract_id = format!(
        "{}:{}:{}",
        identity.identities.node_id,
        identity.identities.strategy_id,
        identity.identities.strategy_instance_id
    );
    if identity.contract_id != expected_contract_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "identity_contract_id",
        ));
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
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "identity_boundaries",
        ));
    }
    Ok(())
}

fn validate_product_config(
    config: &ProductConfigProjection,
    identity: &MvpIdentityContract,
    now_unix_ms: u64,
) -> Result<(), ProductError> {
    validate_identifier("strategy_id", &config.strategy.strategy_id)?;
    if config.strategy.strategy_id != identity.identities.strategy_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_identity",
        ));
    }
    validate_text("display_name", &config.strategy.display_name, 120)?;
    validate_text("description", &config.strategy.description, 1_000)?;
    validate_text("owner", &config.strategy.owner, 120)?;
    validate_text("strategy_version", &config.mvp.strategy_version, 256)?;
    if config.mvp.strategy_version != identity.identities.strategy_version {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "default_version_identity",
        ));
    }
    if config.strategy.created_at_unix_ms == 0
        || config.strategy.updated_at_unix_ms < config.strategy.created_at_unix_ms
        || config.strategy.updated_at_unix_ms > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS)
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_timestamps",
        ));
    }
    Ok(())
}

fn validate_runtime_boundaries(
    record: &SupervisorNodeRecord,
    expected_node_id: &str,
    now_unix_ms: u64,
    freshness_max_age_ms: u64,
    enforce_freshness: bool,
) -> Result<(), ProductError> {
    validate_runtime_boundary_values(
        &record.last_known_status.node_id,
        record.last_known_status.external_venue_connection,
        record.last_known_status.real_orders_submitted,
        expected_node_id,
        "registry_status",
    )?;
    validate_optional_runtime_artifact::<NodeStatus>(
        RuntimeArtifactContract {
            path: &record.status_path,
            artifact_state: record.status_artifact,
            process_state: record.process.state,
            expected_node_id,
            field: "node_status",
            expected_schema_version: NODE_STATUS_SCHEMA_VERSION,
            now_unix_ms,
            freshness_max_age_ms,
            enforce_freshness,
        },
        |status| {
            (
                status.node_id.as_str(),
                status.schema_version.as_str(),
                &status.generated_at,
                status.external_venue_connection,
                status.real_orders_submitted,
            )
        },
        |_| Ok(()),
    )?;
    validate_optional_runtime_artifact::<NodeMetrics>(
        RuntimeArtifactContract {
            path: &record.metrics_path,
            artifact_state: record.metrics_artifact,
            process_state: record.process.state,
            expected_node_id,
            field: "node_metrics",
            expected_schema_version: NODE_METRICS_SCHEMA_VERSION,
            now_unix_ms,
            freshness_max_age_ms,
            enforce_freshness,
        },
        |metrics| {
            (
                metrics.node_id.as_str(),
                metrics.schema_version.as_str(),
                &metrics.generated_at,
                metrics.external_venue_connection,
                metrics.real_orders_submitted,
            )
        },
        validate_metrics_boundaries,
    )
}

fn validate_optional_runtime_artifact<T>(
    contract: RuntimeArtifactContract<'_>,
    fields: impl for<'a> Fn(&'a T) -> (&'a str, &'a str, &'a SnapshotValue<String>, bool, bool),
    validate_extra: impl Fn(&T) -> Result<(), ProductError>,
) -> Result<(), ProductError>
where
    T: serde::de::DeserializeOwned,
{
    let exists = contract.path.exists();
    match (contract.artifact_state, exists) {
        (RegistryArtifactState::Missing, false)
            if matches!(
                contract.process_state,
                SupervisorProcessState::NotStarted | SupervisorProcessState::Stopped
            ) =>
        {
            return Ok(());
        }
        (RegistryArtifactState::Missing, false) => {
            let kind = if contract.process_state == SupervisorProcessState::Stale {
                ProductErrorKind::SourceStale
            } else {
                ProductErrorKind::SourceUnavailable
            };
            return Err(product_error(kind, contract.field));
        }
        (RegistryArtifactState::Available, true) => {}
        (RegistryArtifactState::Available, false) => {
            return Err(product_error(
                ProductErrorKind::SourceUnavailable,
                contract.field,
            ));
        }
        (RegistryArtifactState::Stale, _) => {
            return Err(product_error(ProductErrorKind::SourceStale, contract.field));
        }
        (RegistryArtifactState::Invalid | RegistryArtifactState::Unknown, _)
        | (RegistryArtifactState::Missing, true) => {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                contract.field,
            ));
        }
    }
    let raw = fs::read_to_string(contract.path)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, contract.field))?;
    let artifact: T = serde_json::from_str(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, contract.field))?;
    let (node_id, schema_version, generated_at, external_venue_connection, real_orders_submitted) =
        fields(&artifact);
    if schema_version != contract.expected_schema_version {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            contract.field,
        ));
    }
    validate_runtime_artifact_freshness(
        generated_at,
        contract.now_unix_ms,
        contract.freshness_max_age_ms,
        contract.field,
        contract.enforce_freshness,
    )?;
    validate_runtime_boundary_values(
        node_id,
        external_venue_connection,
        real_orders_submitted,
        contract.expected_node_id,
        contract.field,
    )?;
    validate_extra(&artifact)
}

fn validate_runtime_artifact_freshness(
    generated_at: &SnapshotValue<String>,
    now_unix_ms: u64,
    freshness_max_age_ms: u64,
    field: &str,
    enforce_freshness: bool,
) -> Result<(), ProductError> {
    if generated_at.availability == SnapshotAvailability::Stale {
        return Err(product_error(ProductErrorKind::SourceStale, field));
    }
    if generated_at.availability != SnapshotAvailability::Available {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    let timestamp = generated_at
        .value
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, field))?;
    if timestamp > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS) {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    if enforce_freshness && now_unix_ms.saturating_sub(timestamp) > freshness_max_age_ms {
        return Err(product_error(ProductErrorKind::SourceStale, field));
    }
    Ok(())
}

fn validate_metrics_boundaries(metrics: &NodeMetrics) -> Result<(), ProductError> {
    let kill_switch = &metrics.kill_switch_dry_run;
    if !snapshot_is_explicit_false(&kill_switch.production_order_submission_allowed)
        || !snapshot_is_explicit_false(&kill_switch.production_order_mutation_allowed)
        || !snapshot_is_explicit_false(&kill_switch.dashboard_order_controls_enabled)
        || !snapshot_is_explicit_false(&kill_switch.real_orders_submitted)
        || !snapshot_is_explicit_zero(&kill_switch.production_orders_submitted)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "node_metrics",
        ));
    }
    Ok(())
}

fn snapshot_is_explicit_false(value: &SnapshotValue<bool>) -> bool {
    value.availability == SnapshotAvailability::Available && value.value == Some(false)
}

fn snapshot_is_explicit_zero(value: &SnapshotValue<u64>) -> bool {
    value.availability == SnapshotAvailability::Available && value.value == Some(0)
}

fn validate_runtime_boundary_values(
    node_id: &str,
    external_venue_connection: bool,
    real_orders_submitted: bool,
    expected_node_id: &str,
    field: &str,
) -> Result<(), ProductError> {
    if node_id != expected_node_id {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    if external_venue_connection || real_orders_submitted {
        return Err(product_error(ProductErrorKind::BoundaryViolation, field));
    }
    Ok(())
}

fn read_stable_config_snapshot(
    config_path: &Path,
    identity_path: &Path,
    identity_generated_at_unix_ms: u64,
) -> Result<String, ProductError> {
    let identity_modified = fs::metadata(identity_path)
        .and_then(|metadata| metadata.modified())
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "identity_metadata"))?;
    let before = fs::metadata(config_path)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "config_metadata"))?;
    let before_modified = before
        .modified()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "config_metadata"))?;
    let raw = fs::read_to_string(config_path)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "strategy_config"))?;
    let after = fs::metadata(config_path)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "config_metadata"))?;
    let after_modified = after
        .modified()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "config_metadata"))?;
    if before.len() != after.len()
        || before_modified != after_modified
        || after_modified > identity_modified
        || system_time_ms(after_modified)? > identity_generated_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::SourceStale,
            "strategy_config",
        ));
    }
    Ok(raw)
}

fn project_strategy_list(
    strategy: ProductStrategy,
    query: &StrategyListQuery,
    request_id: String,
) -> Result<StrategyListResponse, ProductError> {
    let mut strategies = vec![strategy];
    strategies.retain(|item| {
        query
            .lifecycle
            .is_none_or(|lifecycle| item.lifecycle == lifecycle)
            && query
                .owner
                .as_ref()
                .is_none_or(|owner| &item.owner == owner)
    });
    strategies.sort_by(|left, right| strategy_comparison(left, right, query.sort, query.order));

    let start = if let Some(cursor) = query.cursor.as_deref() {
        let cursor_id = decode_cursor(cursor)?;
        strategies
            .iter()
            .position(|item| item.strategy_id == cursor_id)
            .map(|position| position + 1)
            .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "cursor"))?
    } else {
        0
    };
    let end = start.saturating_add(query.limit).min(strategies.len());
    let data = strategies[start..end].to_vec();
    let has_more = end < strategies.len();
    let next_cursor = has_more
        .then(|| data.last().map(|item| encode_cursor(&item.strategy_id)))
        .flatten();
    Ok(StrategyListResponse {
        schema_version: STRATEGY_LIST_SCHEMA_VERSION.to_string(),
        contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
        request_id,
        page: ProductPage {
            limit: query.limit,
            returned_count: data.len(),
            next_cursor,
            has_more,
        },
        data,
        boundaries: ProductReadOnlyBoundaries::enforced(),
    })
}

fn strategy_comparison(
    left: &ProductStrategy,
    right: &ProductStrategy,
    sort: StrategySort,
    order: SortOrder,
) -> Ordering {
    let comparison = match sort {
        StrategySort::StrategyId => left.strategy_id.cmp(&right.strategy_id),
        StrategySort::Name => left.name.cmp(&right.name),
        StrategySort::UpdatedAt => left.updated_at_unix_ms.cmp(&right.updated_at_unix_ms),
    };
    match order {
        SortOrder::Asc => comparison,
        SortOrder::Desc => comparison.reverse(),
    }
}

fn parse_strategy_list_query(raw_query: Option<&str>) -> Result<StrategyListQuery, ProductError> {
    let values = parse_query_values(raw_query)?;
    for key in values.keys() {
        if !matches!(
            key.as_str(),
            "limit" | "cursor" | "sort" | "order" | "lifecycle" | "owner"
        ) {
            return Err(product_error(ProductErrorKind::BadRequest, key));
        }
    }
    let limit = values
        .get("limit")
        .map_or(Ok(DEFAULT_PAGE_LIMIT), |value| {
            value
                .parse::<usize>()
                .ok()
                .filter(|value| (1..=MAX_PAGE_LIMIT).contains(value))
                .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "limit"))
        })?;
    let cursor = values.get("cursor").cloned();
    if let Some(cursor) = cursor.as_deref() {
        decode_cursor(cursor)?;
    }
    let sort = match values.get("sort").map(String::as_str) {
        None | Some("strategy_id") => StrategySort::StrategyId,
        Some("name") => StrategySort::Name,
        Some("updated_at") => StrategySort::UpdatedAt,
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "sort")),
    };
    let order = match values.get("order").map(String::as_str) {
        None | Some("asc") => SortOrder::Asc,
        Some("desc") => SortOrder::Desc,
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "order")),
    };
    let lifecycle = match values.get("lifecycle").map(String::as_str) {
        None => None,
        Some("draft") => Some(StrategyLifecycle::Draft),
        Some("active") => Some(StrategyLifecycle::Active),
        Some("archived") => Some(StrategyLifecycle::Archived),
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "lifecycle")),
    };
    let owner = values.get("owner").cloned();
    if let Some(owner) = owner.as_deref() {
        validate_text("owner", owner, 120)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "owner"))?;
    }
    Ok(StrategyListQuery {
        limit,
        cursor,
        sort,
        order,
        lifecycle,
        owner,
    })
}

fn reject_detail_query(raw_query: Option<&str>) -> Result<(), ProductError> {
    if raw_query.is_some_and(|query| !query.is_empty()) {
        let values = parse_query_values(raw_query)?;
        let field = values
            .keys()
            .next()
            .map_or_else(|| "query".to_string(), Clone::clone);
        return Err(product_error(ProductErrorKind::BadRequest, field));
    }
    Ok(())
}

fn parse_query_values(raw_query: Option<&str>) -> Result<BTreeMap<String, String>, ProductError> {
    let Some(raw_query) = raw_query.filter(|value| !value.is_empty()) else {
        return Ok(BTreeMap::new());
    };
    let mut values = BTreeMap::new();
    for pair in raw_query.split('&') {
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode_query_component(raw_key)?;
        let value = decode_query_component(raw_value)?;
        if key.is_empty() || values.insert(key, value).is_some() {
            return Err(product_error(ProductErrorKind::BadRequest, "query"));
        }
    }
    Ok(values)
}

fn decode_query_component(value: &str) -> Result<String, ProductError> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(product_error(ProductErrorKind::BadRequest, "query"));
                }
                let high = decode_hex_digit(bytes[index + 1])
                    .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "query"))?;
                let low = decode_hex_digit(bytes[index + 2])
                    .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "query"))?;
                decoded.push((high << 4) | low);
                index += 3;
            }
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).map_err(|_| product_error(ProductErrorKind::BadRequest, "query"))
}

const fn decode_hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn encode_cursor(strategy_id: &str) -> String {
    let mut encoded = String::with_capacity(CURSOR_PREFIX.len() + strategy_id.len() * 2);
    encoded.push_str(CURSOR_PREFIX);
    for byte in strategy_id.bytes() {
        use std::fmt::Write as _;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn decode_cursor(cursor: &str) -> Result<String, ProductError> {
    let encoded = cursor
        .strip_prefix(CURSOR_PREFIX)
        .filter(|value| !value.is_empty() && value.len() % 2 == 0)
        .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "cursor"))?;
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let pair = std::str::from_utf8(pair)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "cursor"))?;
        bytes.push(
            u8::from_str_radix(pair, 16)
                .map_err(|_| product_error(ProductErrorKind::BadRequest, "cursor"))?,
        );
    }
    let strategy_id = String::from_utf8(bytes)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, "cursor"))?;
    validate_identifier("cursor", &strategy_id)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, "cursor"))?;
    Ok(strategy_id)
}

fn strategy_version_resource_id(strategy_id: &str, strategy_version: &str) -> String {
    format!("{strategy_id}@{strategy_version}")
}

fn validate_identifier(field: &str, value: &str) -> Result<(), ProductError> {
    if value.is_empty()
        || value.len() > 128
        || matches!(value, "." | "..")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    Ok(())
}

fn validate_text(field: &str, value: &str, max_chars: usize) -> Result<(), ProductError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > max_chars || trimmed != value {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    Ok(())
}

fn validate_sha256_hash(field: &str, value: &str) -> Result<(), ProductError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    Ok(())
}

fn mvp_workspace_root(registry_path: &Path) -> Result<PathBuf, ProductError> {
    registry_path
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "supervisor"))
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| product_error(ProductErrorKind::SourceUnavailable, "workspace_layout"))
}

fn load_json<T: serde::de::DeserializeOwned>(path: &Path, field: &str) -> Result<T, ProductError> {
    let raw = fs::read_to_string(path)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, field))?;
    serde_json::from_str(&raw).map_err(|_| product_error(ProductErrorKind::SourceInvalid, field))
}

fn load_product_registry(path: &Path) -> Result<SupervisorRegistry, ProductError> {
    if !path.exists() {
        return Err(product_error(
            ProductErrorKind::SourceUnavailable,
            "registry",
        ));
    }
    let raw = fs::read_to_string(path)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "registry"))?;
    serde_json::from_str(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "registry"))
}

fn canonical_path(path: &Path, field: &str) -> Result<PathBuf, ProductError> {
    fs::canonicalize(path).map_err(|_| product_error(ProductErrorKind::SourceUnavailable, field))
}

fn canonical_contract_path(path: &Path, field: &str) -> Result<PathBuf, ProductError> {
    if path.exists() {
        return canonical_path(path, field);
    }
    let parent = path
        .parent()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, field))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, field))?;
    Ok(canonical_path(parent, field)?.join(file_name))
}

fn product_error(kind: ProductErrorKind, field: impl Into<String>) -> ProductError {
    ProductError {
        kind,
        field: field.into(),
    }
}

fn product_error_response(error: &ProductError, request_id: &str) -> (StatusCode, Json<Value>) {
    let (status, error_code, summary, retryable) = match error.kind {
        ProductErrorKind::BadRequest => (
            StatusCode::BAD_REQUEST,
            "product_query_invalid",
            "查询条件不符合产品 API 合同",
            false,
        ),
        ProductErrorKind::Conflict => (
            StatusCode::CONFLICT,
            "backtest_run_conflict",
            "回测运行与已有不可变记录冲突",
            false,
        ),
        ProductErrorKind::DemoConflict => (
            StatusCode::CONFLICT,
            "demo_run_conflict",
            "Demo 运行与当前节点生命周期冲突",
            false,
        ),
        ProductErrorKind::ExecutionFailed => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "backtest_execution_failed",
            "回测引擎执行或运行记录处理失败",
            false,
        ),
        ProductErrorKind::DemoExecutionFailed => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "demo_execution_failed",
            "Demo 节点执行或运行记录处理失败",
            false,
        ),
        ProductErrorKind::LiveConflict => (
            StatusCode::CONFLICT,
            "live_run_candidate_conflict",
            "Live Run 候选与当前准入生命周期冲突",
            false,
        ),
        ProductErrorKind::LiveExecutionFailed => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "live_run_preflight_failed",
            "Live Run 启动前检查未通过",
            false,
        ),
        ProductErrorKind::Forbidden => (
            StatusCode::FORBIDDEN,
            "product_access_denied",
            "当前账号无权访问策略产品资源",
            false,
        ),
        ProductErrorKind::MethodNotAllowed => (
            StatusCode::METHOD_NOT_ALLOWED,
            "product_method_not_allowed",
            "当前策略产品 API 路径不支持该请求方法",
            false,
        ),
        ProductErrorKind::NotFound => (
            StatusCode::NOT_FOUND,
            "strategy_not_found",
            "未找到指定策略",
            false,
        ),
        ProductErrorKind::VersionNotFound => (
            StatusCode::NOT_FOUND,
            "strategy_version_not_found",
            "未找到指定策略版本",
            false,
        ),
        ProductErrorKind::RunNotFound => (
            StatusCode::NOT_FOUND,
            "run_not_found",
            "未找到指定运行记录",
            false,
        ),
        ProductErrorKind::SourceUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "product_source_unavailable",
            "策略产品数据源暂时不可用",
            true,
        ),
        ProductErrorKind::SourceInvalid => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "product_source_invalid",
            "策略产品数据源未通过合同校验",
            false,
        ),
        ProductErrorKind::SourceStale => (
            StatusCode::SERVICE_UNAVAILABLE,
            "product_source_stale",
            "策略产品数据源已过期，需要刷新对应来源后重试",
            true,
        ),
        ProductErrorKind::BoundaryViolation => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "product_boundary_violation",
            "策略产品只读边界未通过校验",
            false,
        ),
    };
    (
        status,
        Json(json!({
            "schema_version": PRODUCT_ERROR_SCHEMA_VERSION,
            "contract_version": PRODUCT_API_CONTRACT_VERSION,
            "request_id": request_id,
            "error": {
                "code": error_code,
                "summary": summary,
                "retryable": retryable,
                "field": error.field,
            },
            "boundaries": ProductReadOnlyBoundaries::enforced(),
        })),
    )
}

fn demo_product_error_response(
    error: &ProductError,
    request_id: &str,
) -> (StatusCode, Json<Value>) {
    let mut scoped = error.clone();
    scoped.kind = match scoped.kind {
        ProductErrorKind::Conflict => ProductErrorKind::DemoConflict,
        ProductErrorKind::ExecutionFailed => ProductErrorKind::DemoExecutionFailed,
        kind => kind,
    };
    product_error_response(&scoped, request_id)
}

fn product_request_id() -> String {
    let sequence = REQUEST_SEQUENCE.fetch_add(1, AtomicOrdering::Relaxed);
    format!("product-{0:016x}-{sequence:016x}", unix_time_ms())
}

fn system_time_ms(value: SystemTime) -> Result<u64, ProductError> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "timestamp"))
}

fn unix_time_ms() -> u64 {
    system_time_ms(SystemTime::now()).unwrap_or(0)
}
