//! Live Run 候选、启动前检查与人工停止产品合同。

use std::{
    collections::BTreeSet,
    fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    time::Duration,
};

use aws_lc_rs::digest::{SHA256, digest};
use axum::{
    Json,
    extract::{
        Path as AxumPath, RawQuery, State,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use nautilus_live::status::{ConnectionStatus, LifecycleStatus};
use serde::{Deserialize, Serialize};

use crate::{
    dashboard::ApiStatusResult,
    supervisor::{
        RegisterNodeRequest, StartNodeRequest, StopNodeRequest, SupervisorProcessState,
        SupervisorRegistryStore, SupervisorRunOwnership, SupervisorRunTerminalAnchor,
    },
};

use super::{
    live_admission::{
        LiveRunCreationAdmission, LiveRunPreflightAdmission, evaluate_live_run_creation_admission,
        evaluate_live_run_preflight_admission,
    },
    live_run_anchor::{
        LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION, LiveRunAnchorAppendRequest, LiveRunAnchorReceipt,
        LiveRunAnchorRevision, anchor_config_refs,
    },
    run::{open_absolute_directory_nofollow, write_new_run_file},
    *,
};

const LIVE_RUN_CANDIDATE_MANIFEST_SCHEMA_VERSION: &str =
    "ntpro.product_api.live_run_candidate_manifest.v1";
const LIVE_RUN_CANDIDATE_CREATE_SCHEMA_VERSION: &str =
    "ntpro.product_api.live_run_candidate_create.response.v1";
const LIVE_RUN_CANDIDATE_DETAIL_SCHEMA_VERSION: &str =
    "ntpro.product_api.live_run_candidate_detail.response.v1";
const LIVE_RUN_CANDIDATE_LIST_SCHEMA_VERSION: &str =
    "ntpro.product_api.live_run_candidate_list.response.v1";
const LIVE_RUN_CANDIDATE_ACTION_SCHEMA_VERSION: &str =
    "ntpro.product_api.live_run_candidate_action.response.v1";
const LIVE_RUN_PREFLIGHT_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_preflight.v1";
const LIVE_RUN_STOP_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_stop.v1";
const LIVE_MARKET_DATA_NODE_CONFIG_FILE: &str = "live-market-data-node.toml";
const LIVE_RUN_STATE_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_state.v2";
const LIVE_RUN_STATE_HEAD_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_state_head.v2";
const LIVE_RUN_ACTIVE_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_active.v1";
const LIVE_RUN_STATE_COMMIT_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_state_commit.v1";
const LIVE_RUN_ACTIVE_FILE: &str = ".active-candidate.json";
const LIVE_RUN_STATE_HEAD_FILE: &str = "state-head.json";
const LIVE_RUN_STATE_HEAD_NEXT_FILE: &str = ".state-head.next.json";
const LIVE_RUN_MUTATION_LOCK_FILE: &str = ".live-run-mutation.lock";
const LIVE_RUN_STATE_COMMIT_DIRECTORY: &str = "live-run-state-commits";
const LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE: &str = "live-run-audit-anchor-head.json";
const LIVE_RUN_WORKSPACE_ANCHOR_HEAD_NEXT_FILE: &str = ".live-run-audit-anchor-head.next.json";
const LIVE_RUN_ARTIFACT_MAX_BYTES: u64 = 64 * 1024;
const LIVE_MARKET_DATA_STARTUP_TIMEOUT_MS: u64 = 20_000;

const LIVE_RUN_GATE_CREATE: &str = "NTPRO_S3_LIVE_RUN_CANDIDATE_CREATE";
const LIVE_RUN_GATE_OWNER_APPROVED: &str = "NTPRO_S3_LIVE_RUN_OWNER_APPROVED";
const LIVE_RUN_GATE_NO_ORDER_SEND: &str = "NTPRO_S3_LIVE_RUN_NO_ORDER_SEND";
const LIVE_RUN_GATE_MANUAL_STOP: &str = "NTPRO_S3_LIVE_RUN_MANUAL_STOP";
const LIVE_RUN_GATE_RISK_APPROVED: &str = "NTPRO_S3_LIVE_RUN_RISK_APPROVED";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LiveRunCandidateLifecycle {
    Created,
    PreflightReady,
    Starting,
    MarketDataRunning,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LiveRunCandidateAction {
    Preflight,
    StartMarketData,
    Stop,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::dashboard) struct CreateLiveRunCandidateRequest {
    strategy_id: String,
    strategy_version_id: String,
    environment: String,
    account_ref: String,
    venue_ref: String,
    user_confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::dashboard) struct LiveRunCandidateActionRequest {
    run_id: String,
    action: LiveRunCandidateAction,
    user_confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRunCandidateManifest {
    schema_version: String,
    request_sha256: String,
    strategy_version_content_hash: String,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    environment: String,
    account_ref: String,
    venue_ref: String,
    created_at_unix_ms: u64,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRunPreflightArtifact {
    schema_version: String,
    run_id: String,
    source_manifest_sha256: String,
    evaluated_at_unix_ms: u64,
    account_connected: bool,
    account_can_trade_verified: bool,
    runtime_gates_verified: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_send_attempted: bool,
    real_orders_submitted: bool,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRunStopArtifact {
    schema_version: String,
    run_id: String,
    source_manifest_sha256: String,
    source_preflight_sha256: Option<String>,
    stopped_at_unix_ms: u64,
    manual_stop: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_send_attempted: bool,
    real_orders_submitted: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRunCandidateState {
    schema_version: String,
    run_id: String,
    source_manifest_sha256: String,
    revision: u64,
    previous_state_sha256: Option<String>,
    lifecycle: LiveRunCandidateLifecycle,
    preflight_sha256: Option<String>,
    stop_sha256: Option<String>,
    updated_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRunStateCommit {
    schema_version: String,
    run_id: String,
    revision: u64,
    state_sha256: String,
    previous_commit_sha256: Option<String>,
    committed_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRunStateHead {
    schema_version: String,
    run_id: String,
    revision: u64,
    state_sha256: String,
    commit_sha256: String,
    anchor_receipt_sha256: String,
    updated_at_unix_ms: u64,
}

struct LiveRunMutationLock {
    artifact_root: cap_std::fs::Dir,
}

impl Drop for LiveRunMutationLock {
    fn drop(&mut self) {
        let _ = self.artifact_root.remove_file(LIVE_RUN_MUTATION_LOCK_FILE);
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct ActiveLiveRunCandidate {
    schema_version: String,
    run_id: String,
    source_manifest_sha256: String,
    claimed_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveOrderAdmissionSnapshot {
    status: String,
    submit: String,
    cancel: String,
    replace: String,
    fill_reconciliation: String,
    blockers: Vec<String>,
}

impl LiveOrderAdmissionSnapshot {
    fn blocked() -> Self {
        Self {
            status: "blocked".to_string(),
            submit: "blocked".to_string(),
            cancel: "blocked".to_string(),
            replace: "blocked".to_string(),
            fill_reconciliation: "blocked".to_string(),
            blockers: vec![
                "production_order_authority_not_granted".to_string(),
                "execution_adapter_send_not_enabled".to_string(),
                "fill_reconciliation_not_enabled".to_string(),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveRunCandidate {
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    environment: String,
    account_ref: String,
    venue_ref: String,
    lifecycle: LiveRunCandidateLifecycle,
    created_at_unix_ms: u64,
    preflight_at_unix_ms: Option<u64>,
    stopped_at_unix_ms: Option<u64>,
    account_connected: bool,
    account_can_trade_verified: bool,
    runtime_started: bool,
    market_data_connected: bool,
    runtime_node_id: Option<String>,
    runtime_process_state: String,
    runtime_error: Option<String>,
    audit_anchor: LiveRunAuditAnchorSnapshot,
    order_admission: LiveOrderAdmissionSnapshot,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveRunAuditAnchorSnapshot {
    status: String,
    namespace: String,
    revision: u64,
    workspace_revision: u64,
    receipt_ref: String,
    key_id: String,
    anchored_at_unix_ms: u64,
    workspace_snapshot_rollback_detectable: bool,
    trading_authority_granted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveRunCandidateBoundaries {
    candidate_creation_allowed: bool,
    explicit_preflight_allowed: bool,
    manual_stop_allowed: bool,
    live_runtime_start_allowed: bool,
    external_market_data_connection_allowed: bool,
    order_endpoint_access_allowed: bool,
    order_submission_allowed: bool,
    cancel_order_allowed: bool,
    replace_order_allowed: bool,
    fill_reconciliation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    automatic_recovery_allowed: bool,
    execution_adapter_send_attempted: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

impl LiveRunCandidateBoundaries {
    const fn enforced() -> Self {
        Self {
            candidate_creation_allowed: true,
            explicit_preflight_allowed: true,
            manual_stop_allowed: true,
            live_runtime_start_allowed: true,
            external_market_data_connection_allowed: true,
            order_endpoint_access_allowed: false,
            order_submission_allowed: false,
            cancel_order_allowed: false,
            replace_order_allowed: false,
            fill_reconciliation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            automatic_recovery_allowed: false,
            execution_adapter_send_attempted: false,
            real_orders_submitted: false,
            trading_controls_enabled: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct LiveRunCandidateResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: LiveRunCandidate,
    runtime_gate_refs: Vec<String>,
    audit_anchor_config_refs: Vec<String>,
    boundaries: LiveRunCandidateBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct LiveRunCandidateListResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: Vec<LiveRunCandidate>,
    runtime_gate_refs: Vec<String>,
    audit_anchor_config_refs: Vec<String>,
    boundaries: LiveRunCandidateBoundaries,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
struct LiveRunGateState {
    candidate_create: bool,
    owner_approved: bool,
    no_order_send: bool,
    manual_stop: bool,
    risk_approved: bool,
}

impl LiveRunGateState {
    fn from_environment() -> Self {
        Self::from_reader(|name| std::env::var(name).ok().as_deref() == Some("1"))
    }

    fn from_reader<F>(mut reader: F) -> Self
    where
        F: FnMut(&str) -> bool,
    {
        Self {
            candidate_create: reader(LIVE_RUN_GATE_CREATE),
            owner_approved: reader(LIVE_RUN_GATE_OWNER_APPROVED),
            no_order_send: reader(LIVE_RUN_GATE_NO_ORDER_SEND),
            manual_stop: reader(LIVE_RUN_GATE_MANUAL_STOP),
            risk_approved: reader(LIVE_RUN_GATE_RISK_APPROVED),
        }
    }

    const fn all_open(&self) -> bool {
        self.candidate_create
            && self.owner_approved
            && self.no_order_send
            && self.manual_stop
            && self.risk_approved
    }

    fn refs() -> Vec<String> {
        vec![
            LIVE_RUN_GATE_CREATE.to_string(),
            LIVE_RUN_GATE_OWNER_APPROVED.to_string(),
            LIVE_RUN_GATE_NO_ORDER_SEND.to_string(),
            LIVE_RUN_GATE_MANUAL_STOP.to_string(),
            LIVE_RUN_GATE_RISK_APPROVED.to_string(),
        ]
    }
}

pub(in crate::dashboard) async fn live_run_candidate_create_api(
    State(state): State<DashboardServerState>,
    payload: Result<Json<CreateLiveRunCandidateRequest>, JsonRejection>,
) -> ApiStatusResult<LiveRunCandidateResponse> {
    let request_id = product_request_id();
    let Json(request) = payload.map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "request_body"),
            &request_id,
        )
    })?;
    let worker_state = state.clone();
    let worker_request_id = request_id.clone();
    tokio::task::spawn_blocking(move || {
        let _guard = worker_state.lifecycle_action_lock.lock().map_err(|_| {
            product_error(ProductErrorKind::LiveConflict, "live_candidate_action_lock")
        })?;
        let now = unix_time_ms();
        let source = load_product_source(&worker_state, now)?;
        let version = strategy_version::load_product_strategy_version(&source, now)?;
        if version.strategy_id() != request.strategy_id
            || version.strategy_version_id() != request.strategy_version_id
        {
            return Err(product_error(
                ProductErrorKind::BadRequest,
                "live_candidate_identity",
            ));
        }
        let admission = evaluate_live_run_creation_admission(
            &source,
            &request.strategy_id,
            &request.strategy_version_id,
            now,
        )?;
        create_live_run_candidate(
            &worker_state,
            request,
            &worker_request_id,
            now,
            admission,
            LiveRunGateState::from_environment(),
            version.content_hash(),
        )
    })
    .await
    .map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_candidate_worker",
        )
    })
    .and_then(|result| result)
    .map(|data| {
        (
            StatusCode::CREATED,
            Json(response(
                LIVE_RUN_CANDIDATE_CREATE_SCHEMA_VERSION,
                request_id.clone(),
                data,
            )),
        )
    })
    .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn live_run_candidate_list_api(
    State(state): State<DashboardServerState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<LiveRunCandidateListResponse> {
    let request_id = product_request_id();
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        let active = load_active_live_run_candidates(&state)?;
        for (_, manifest, _) in &active {
            validate_live_candidate_against_current_source(&state, manifest)?;
        }
        Ok(LiveRunCandidateListResponse {
            schema_version: LIVE_RUN_CANDIDATE_LIST_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: active
                .into_iter()
                .map(|(candidate, _, _)| candidate)
                .collect(),
            runtime_gate_refs: LiveRunGateState::refs(),
            audit_anchor_config_refs: anchor_config_refs(),
            boundaries: LiveRunCandidateBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn live_run_candidate_detail_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<LiveRunCandidateResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(value)| value).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "run_id"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_identifier("run_id", &run_id)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "run_id"))?;
        let (candidate, manifest, _) = load_live_run_candidate_snapshot(&state, &run_id)?;
        validate_live_candidate_against_current_source(&state, &manifest)?;
        Ok(candidate)
    });
    result
        .map(|data| {
            Json(response(
                LIVE_RUN_CANDIDATE_DETAIL_SCHEMA_VERSION,
                request_id.clone(),
                data,
            ))
        })
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn live_run_candidate_action_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<LiveRunCandidateActionRequest>, JsonRejection>,
) -> ApiResult<LiveRunCandidateResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(value)| value).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "run_id"),
            &request_id,
        )
    })?;
    let Json(request) = payload.map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "request_body"),
            &request_id,
        )
    })?;
    let worker_state = state.clone();
    tokio::task::spawn_blocking(move || {
        let _guard = worker_state.lifecycle_action_lock.lock().map_err(|_| {
            product_error(ProductErrorKind::LiveConflict, "live_candidate_action_lock")
        })?;
        run_live_candidate_action(&worker_state, &run_id, &request)
    })
    .await
    .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_action_worker"))
    .and_then(|result| result)
    .map(|data| {
        Json(response(
            LIVE_RUN_CANDIDATE_ACTION_SCHEMA_VERSION,
            request_id.clone(),
            data,
        ))
    })
    .map_err(|error| product_error_response(&error, &request_id))
}

fn response(
    schema_version: &str,
    request_id: String,
    data: LiveRunCandidate,
) -> LiveRunCandidateResponse {
    LiveRunCandidateResponse {
        schema_version: schema_version.to_string(),
        contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
        request_id,
        data,
        runtime_gate_refs: LiveRunGateState::refs(),
        audit_anchor_config_refs: anchor_config_refs(),
        boundaries: LiveRunCandidateBoundaries::enforced(),
    }
}

fn create_live_run_candidate(
    state: &DashboardServerState,
    request: CreateLiveRunCandidateRequest,
    request_id: &str,
    now: u64,
    admission: LiveRunCreationAdmission,
    gates: LiveRunGateState,
    strategy_version_content_hash: &str,
) -> Result<LiveRunCandidate, ProductError> {
    validate_create_request(&request, &admission, &gates)?;
    validate_sha256_hash(
        "strategy_version_content_hash",
        strategy_version_content_hash,
    )?;
    let _workspace_lock = acquire_live_run_mutation_lock(state)?;
    if !load_active_live_run_candidates(state)?.is_empty() {
        return Err(product_error(
            ProductErrorKind::LiveConflict,
            "active_live_run_candidate",
        ));
    }
    let run_id = request_id.replacen("product-", "live-candidate-", 1);
    validate_identifier("run_id", &run_id)?;
    let request_raw = serde_json::to_vec_pretty(&request)
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_request"))?;
    let mut source_refs = admission.source_refs;
    source_refs.sort();
    source_refs.dedup();
    let manifest = LiveRunCandidateManifest {
        schema_version: LIVE_RUN_CANDIDATE_MANIFEST_SCHEMA_VERSION.to_string(),
        request_sha256: sha256_ref(&request_raw),
        strategy_version_content_hash: strategy_version_content_hash.to_string(),
        run_id: run_id.clone(),
        strategy_id: request.strategy_id,
        strategy_version_id: request.strategy_version_id,
        environment: "live".to_string(),
        account_ref: request.account_ref,
        venue_ref: request.venue_ref,
        created_at_unix_ms: now,
        source_refs,
    };
    let manifest_raw = serde_json::to_vec_pretty(&manifest)
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_manifest"))?;
    let manifest_sha256 = sha256_ref(&manifest_raw);
    let directory = create_live_run_candidate_directory(state, &run_id)?;
    if let Err(error) = write_new_run_file(&directory, "request.json", &request_raw)
        .and_then(|()| write_new_run_file(&directory, "run-manifest.json", &manifest_raw))
        .and_then(|()| {
            write_initial_live_run_state(
                state,
                &run_id,
                &manifest_sha256,
                manifest.created_at_unix_ms,
            )
        })
        .and_then(|()| claim_active_live_run_candidate(state, &run_id, &manifest_sha256, now))
    {
        drop(directory);
        cleanup_unpublished_live_run_candidate(state, &run_id);
        return Err(error);
    }
    load_live_run_candidate(state, &run_id)
}

fn cleanup_unpublished_live_run_candidate(state: &DashboardServerState, run_id: &str) {
    if let Ok(root) = canonical_live_run_root(state, false) {
        let _ = fs::remove_dir_all(root.join(run_id));
    }
    if let Ok(directory) = open_live_run_state_commit_directory(state, false) {
        let _ = directory.remove_file(live_run_state_commit_file_name(run_id, 0));
    }
}

fn validate_create_request(
    request: &CreateLiveRunCandidateRequest,
    admission: &LiveRunCreationAdmission,
    gates: &LiveRunGateState,
) -> Result<(), ProductError> {
    validate_identifier("strategy_id", &request.strategy_id)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, "strategy_id"))?;
    strategy_version::validate_requested_version_id(
        "strategy_version_id",
        &request.strategy_version_id,
    )?;
    if request.environment != "live"
        || !request.user_confirmed
        || request.account_ref != admission.account_ref
        || request.venue_ref != format!("venue://live/{}", admission.venue_id)
    {
        return Err(product_error(
            ProductErrorKind::BadRequest,
            "live_candidate_identity",
        ));
    }
    if !admission.ready || !admission.risk_ready || !gates.all_open() {
        return Err(product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_candidate_admission",
        ));
    }
    Ok(())
}

fn run_live_candidate_action(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveRunCandidateActionRequest,
) -> Result<LiveRunCandidate, ProductError> {
    run_live_candidate_action_with_preflight(state, path_run_id, request, |manifest| {
        evaluate_current_live_candidate_preflight(state, manifest)
    })
}

fn run_live_candidate_action_with_preflight<F>(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveRunCandidateActionRequest,
    preflight_evaluator: F,
) -> Result<LiveRunCandidate, ProductError>
where
    F: FnOnce(&LiveRunCandidateManifest) -> Result<LiveRunPreflightAdmission, ProductError>,
{
    let _workspace_lock = acquire_live_run_mutation_lock(state)?;
    validate_identifier("run_id", path_run_id)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, "run_id"))?;
    if request.run_id != path_run_id || !request.user_confirmed {
        return Err(product_error(
            ProductErrorKind::BadRequest,
            "live_action_identity",
        ));
    }
    let (current, manifest, manifest_raw) = load_live_run_candidate_snapshot(state, path_run_id)?;
    let manifest_sha256 = sha256_ref(&manifest_raw);
    let (current_state, current_state_raw) =
        load_live_run_state(state, path_run_id, &manifest_sha256)?;
    let directory = open_absolute_directory_nofollow(
        &canonical_live_run_root(state, false)?.join(path_run_id),
    )?;
    match request.action {
        LiveRunCandidateAction::Preflight
            if current.lifecycle == LiveRunCandidateLifecycle::Created =>
        {
            let preflight = preflight_evaluator(&manifest)?;
            if sorted_refs(preflight.source_refs.clone()) != manifest.source_refs {
                return Err(product_error(
                    ProductErrorKind::BoundaryViolation,
                    "live_candidate_source_refs",
                ));
            }
            let preflight_raw =
                write_preflight_artifact(&directory, path_run_id, &manifest_sha256, &preflight)?;
            write_live_run_state(
                state,
                path_run_id,
                &LiveRunCandidateState {
                    schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                    run_id: path_run_id.to_string(),
                    source_manifest_sha256: manifest_sha256,
                    revision: current_state.revision + 1,
                    previous_state_sha256: Some(sha256_ref(&current_state_raw)),
                    lifecycle: LiveRunCandidateLifecycle::PreflightReady,
                    preflight_sha256: Some(sha256_ref(&preflight_raw)),
                    stop_sha256: None,
                    updated_at_unix_ms: preflight.evaluated_at_unix_ms,
                },
            )?;
        }
        LiveRunCandidateAction::StartMarketData
            if current.lifecycle == LiveRunCandidateLifecycle::PreflightReady =>
        {
            let starting_at = unix_time_ms();
            write_live_market_data_node_config(&directory, path_run_id)?;
            let starting_result = write_live_run_state(
                state,
                path_run_id,
                &LiveRunCandidateState {
                    schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                    run_id: path_run_id.to_string(),
                    source_manifest_sha256: manifest_sha256.clone(),
                    revision: current_state.revision + 1,
                    previous_state_sha256: Some(sha256_ref(&current_state_raw)),
                    lifecycle: LiveRunCandidateLifecycle::Starting,
                    preflight_sha256: current_state.preflight_sha256,
                    stop_sha256: None,
                    updated_at_unix_ms: starting_at,
                },
            );
            if let Err(error) = starting_result {
                directory
                    .remove_file(LIVE_MARKET_DATA_NODE_CONFIG_FILE)
                    .map_err(|_| {
                        product_error(
                            ProductErrorKind::LiveExecutionFailed,
                            "live_runtime_config_cleanup",
                        )
                    })?;
                return Err(error);
            }
            let store = SupervisorRegistryStore::new(&state.registry_path);
            let runtime_root = live_market_data_runtime_root(state, path_run_id)?;
            if store
                .register_node(RegisterNodeRequest {
                    node_id: path_run_id.to_string(),
                    config_path: canonical_live_run_root(state, false)?
                        .join(path_run_id)
                        .join(LIVE_MARKET_DATA_NODE_CONFIG_FILE),
                    artifact_root: Some(runtime_root),
                })
                .is_err()
            {
                transition_live_market_data_runtime_failed(
                    state,
                    path_run_id,
                    &manifest_sha256,
                    None,
                )?;
                return Err(product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_runtime_register",
                ));
            }
            if store
                .claim_run_ownership(
                    path_run_id,
                    SupervisorRunOwnership {
                        run_id: path_run_id.to_string(),
                        manifest_sha256: manifest_sha256.clone(),
                        claimed_at_unix_ms: starting_at,
                        terminal: None,
                    },
                )
                .is_err()
            {
                store.remove_node(path_run_id).map_err(|_| {
                    product_error(
                        ProductErrorKind::LiveExecutionFailed,
                        "live_runtime_cleanup",
                    )
                })?;
                transition_live_market_data_runtime_failed(
                    state,
                    path_run_id,
                    &manifest_sha256,
                    None,
                )?;
                return Err(product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_runtime_claim",
                ));
            }
            let started = store.start_node_process_for_run(
                &StartNodeRequest {
                    node_id: path_run_id.to_string(),
                    ntpro_node_bin: state.ntpro_node_bin.clone(),
                    startup_timeout: Duration::from_millis(LIVE_MARKET_DATA_STARTUP_TIMEOUT_MS),
                    node_max_runtime: Duration::from_millis(3_600_000),
                    node_heartbeat_interval: Duration::from_millis(1_000),
                    node_parent_pid: Some(std::process::id()),
                    node_shutdown_timeout: Duration::from_millis(5_000),
                },
                path_run_id,
                &manifest_sha256,
            );
            if started.is_err() {
                transition_live_market_data_runtime_failed(
                    state,
                    path_run_id,
                    &manifest_sha256,
                    Some(&store),
                )?;
                return Err(product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_runtime_start",
                ));
            }
            let (starting_state, starting_raw) =
                load_live_run_state(state, path_run_id, &manifest_sha256)?;
            let running_state = LiveRunCandidateState {
                schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                run_id: path_run_id.to_string(),
                source_manifest_sha256: manifest_sha256.clone(),
                revision: starting_state.revision + 1,
                previous_state_sha256: Some(sha256_ref(&starting_raw)),
                lifecycle: LiveRunCandidateLifecycle::MarketDataRunning,
                preflight_sha256: starting_state.preflight_sha256,
                stop_sha256: None,
                updated_at_unix_ms: unix_time_ms(),
            };
            if write_live_run_state(state, path_run_id, &running_state).is_err() {
                let stop_result = store.stop_node_process_for_run(
                    &StopNodeRequest {
                        node_id: path_run_id.to_string(),
                        stop_timeout: Duration::from_millis(
                            super::super::DASHBOARD_ACTION_TIMEOUT_MS,
                        ),
                    },
                    path_run_id,
                    &manifest_sha256,
                );
                if stop_result.is_err()
                    || transition_live_market_data_runtime_failed(
                        state,
                        path_run_id,
                        &manifest_sha256,
                        Some(&store),
                    )
                    .is_err()
                {
                    return Err(product_error(
                        ProductErrorKind::LiveExecutionFailed,
                        "live_runtime_cleanup",
                    ));
                }
                return Err(product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_runtime_state_publish",
                ));
            }
        }
        LiveRunCandidateAction::Stop
            if matches!(
                current.lifecycle,
                LiveRunCandidateLifecycle::Created | LiveRunCandidateLifecycle::PreflightReady
            ) =>
        {
            let stop = LiveRunStopArtifact {
                schema_version: LIVE_RUN_STOP_SCHEMA_VERSION.to_string(),
                run_id: path_run_id.to_string(),
                source_manifest_sha256: manifest_sha256.clone(),
                source_preflight_sha256: current_state.preflight_sha256.clone(),
                stopped_at_unix_ms: unix_time_ms(),
                manual_stop: true,
                order_endpoint_access_attempted: false,
                execution_adapter_send_attempted: false,
                real_orders_submitted: false,
            };
            let raw = serde_json::to_vec_pretty(&stop)
                .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_stop"))?;
            write_new_run_file(&directory, "stop.json", &raw)?;
            write_live_run_state(
                state,
                path_run_id,
                &LiveRunCandidateState {
                    schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                    run_id: path_run_id.to_string(),
                    source_manifest_sha256: manifest_sha256,
                    revision: current_state.revision + 1,
                    previous_state_sha256: Some(sha256_ref(&current_state_raw)),
                    lifecycle: LiveRunCandidateLifecycle::Stopped,
                    preflight_sha256: current_state.preflight_sha256,
                    stop_sha256: Some(sha256_ref(&raw)),
                    updated_at_unix_ms: stop.stopped_at_unix_ms,
                },
            )?;
            release_active_live_run_candidate(state, path_run_id)?;
        }
        LiveRunCandidateAction::Stop
            if current.lifecycle == LiveRunCandidateLifecycle::MarketDataRunning =>
        {
            let stopping_at = unix_time_ms();
            write_live_run_state(
                state,
                path_run_id,
                &LiveRunCandidateState {
                    schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                    run_id: path_run_id.to_string(),
                    source_manifest_sha256: manifest_sha256.clone(),
                    revision: current_state.revision + 1,
                    previous_state_sha256: Some(sha256_ref(&current_state_raw)),
                    lifecycle: LiveRunCandidateLifecycle::Stopping,
                    preflight_sha256: current_state.preflight_sha256,
                    stop_sha256: None,
                    updated_at_unix_ms: stopping_at,
                },
            )?;
            let store = SupervisorRegistryStore::new(&state.registry_path);
            store
                .stop_node_process_for_run(
                    &StopNodeRequest {
                        node_id: path_run_id.to_string(),
                        stop_timeout: Duration::from_millis(
                            super::super::DASHBOARD_ACTION_TIMEOUT_MS,
                        ),
                    },
                    path_run_id,
                    &manifest_sha256,
                )
                .map_err(|_| {
                    product_error(ProductErrorKind::LiveExecutionFailed, "live_runtime_stop")
                })?;
            let (stopping_state, stopping_raw) =
                load_live_run_state(state, path_run_id, &manifest_sha256)?;
            let stop = LiveRunStopArtifact {
                schema_version: LIVE_RUN_STOP_SCHEMA_VERSION.to_string(),
                run_id: path_run_id.to_string(),
                source_manifest_sha256: manifest_sha256.clone(),
                source_preflight_sha256: stopping_state.preflight_sha256.clone(),
                stopped_at_unix_ms: unix_time_ms(),
                manual_stop: true,
                order_endpoint_access_attempted: false,
                execution_adapter_send_attempted: false,
                real_orders_submitted: false,
            };
            let stop_raw = serde_json::to_vec_pretty(&stop)
                .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_stop"))?;
            write_new_run_file(&directory, "stop.json", &stop_raw)?;
            let stopped_state = LiveRunCandidateState {
                schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                run_id: path_run_id.to_string(),
                source_manifest_sha256: manifest_sha256.clone(),
                revision: stopping_state.revision + 1,
                previous_state_sha256: Some(sha256_ref(&stopping_raw)),
                lifecycle: LiveRunCandidateLifecycle::Stopped,
                preflight_sha256: stopping_state.preflight_sha256,
                stop_sha256: Some(sha256_ref(&stop_raw)),
                updated_at_unix_ms: stop.stopped_at_unix_ms,
            };
            write_live_run_state(state, path_run_id, &stopped_state)?;
            let (_, stopped_raw) = load_live_run_state(state, path_run_id, &manifest_sha256)?;
            store
                .anchor_run_terminal(
                    path_run_id,
                    path_run_id,
                    &manifest_sha256,
                    SupervisorRunTerminalAnchor {
                        lifecycle: "stopped".to_string(),
                        terminal_state_sha256: sha256_ref(&stopped_raw),
                        completed_at_unix_ms: stop.stopped_at_unix_ms,
                    },
                )
                .map_err(|_| {
                    product_error(
                        ProductErrorKind::LiveExecutionFailed,
                        "live_runtime_terminal",
                    )
                })?;
            release_active_live_run_candidate(state, path_run_id)?;
        }
        _ => {
            return Err(product_error(
                ProductErrorKind::LiveConflict,
                "live_candidate_transition",
            ));
        }
    }
    load_live_run_candidate(state, path_run_id)
}

fn transition_live_market_data_runtime_failed(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
    supervisor: Option<&SupervisorRegistryStore>,
) -> Result<(), ProductError> {
    let (current, current_raw) = load_live_run_state(state, run_id, manifest_sha256)?;
    if !matches!(
        current.lifecycle,
        LiveRunCandidateLifecycle::Starting | LiveRunCandidateLifecycle::MarketDataRunning
    ) {
        return Err(product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_cleanup",
        ));
    }
    let failed_at = unix_time_ms();
    let failed = LiveRunCandidateState {
        schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        source_manifest_sha256: manifest_sha256.to_string(),
        revision: current.revision + 1,
        previous_state_sha256: Some(sha256_ref(&current_raw)),
        lifecycle: LiveRunCandidateLifecycle::Failed,
        preflight_sha256: current.preflight_sha256,
        stop_sha256: None,
        updated_at_unix_ms: failed_at,
    };
    write_live_run_state(state, run_id, &failed)?;
    if let Some(store) = supervisor {
        let (_, failed_raw) = load_live_run_state(state, run_id, manifest_sha256)?;
        store
            .anchor_run_terminal(
                run_id,
                run_id,
                manifest_sha256,
                SupervisorRunTerminalAnchor {
                    lifecycle: "failed".to_string(),
                    terminal_state_sha256: sha256_ref(&failed_raw),
                    completed_at_unix_ms: failed_at,
                },
            )
            .map_err(|_| {
                product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_runtime_cleanup",
                )
            })?;
    }
    release_active_live_run_candidate(state, run_id)
}

fn evaluate_current_live_candidate_preflight(
    state: &DashboardServerState,
    manifest: &LiveRunCandidateManifest,
) -> Result<LiveRunPreflightAdmission, ProductError> {
    let now = unix_time_ms();
    let source = load_product_source(state, now)?;
    let creation = validate_live_candidate_source(&source, manifest, now)?;
    if !creation.ready || !LiveRunGateState::from_environment().all_open() {
        return Err(product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_candidate_admission",
        ));
    }
    let preflight = evaluate_live_run_preflight_admission(
        &source,
        &manifest.strategy_id,
        &manifest.strategy_version_id,
        now,
    )?;
    Ok(preflight)
}

fn validate_live_candidate_against_current_source(
    state: &DashboardServerState,
    manifest: &LiveRunCandidateManifest,
) -> Result<(), ProductError> {
    let now = unix_time_ms();
    let source = load_product_source(state, now)?;
    validate_live_candidate_source(&source, manifest, now).map(|_| ())
}

fn validate_live_candidate_source(
    source: &ValidatedProductSource,
    manifest: &LiveRunCandidateManifest,
    now: u64,
) -> Result<LiveRunCreationAdmission, ProductError> {
    let version = strategy_version::load_product_strategy_version(source, now)?;
    if source.strategy.strategy_id != manifest.strategy_id
        || version.strategy_id() != manifest.strategy_id
        || version.strategy_version_id() != manifest.strategy_version_id
        || version.content_hash() != manifest.strategy_version_content_hash
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_candidate_strategy_version",
        ));
    }
    let creation = evaluate_live_run_creation_admission(
        source,
        &manifest.strategy_id,
        &manifest.strategy_version_id,
        now,
    )?;
    if !creation.risk_ready
        || creation.account_ref != manifest.account_ref
        || format!("venue://live/{}", creation.venue_id) != manifest.venue_ref
        || sorted_refs(creation.source_refs.clone()) != manifest.source_refs
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_candidate_source_binding",
        ));
    }
    Ok(creation)
}

fn write_preflight_artifact(
    directory: &cap_std::fs::Dir,
    run_id: &str,
    manifest_sha256: &str,
    admission: &LiveRunPreflightAdmission,
) -> Result<Vec<u8>, ProductError> {
    if !admission.connected || !admission.can_trade {
        return Err(product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_account_not_trade_ready",
        ));
    }
    let artifact = LiveRunPreflightArtifact {
        schema_version: LIVE_RUN_PREFLIGHT_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        source_manifest_sha256: manifest_sha256.to_string(),
        evaluated_at_unix_ms: admission.evaluated_at_unix_ms,
        account_connected: true,
        account_can_trade_verified: true,
        runtime_gates_verified: true,
        order_endpoint_access_attempted: false,
        execution_adapter_send_attempted: false,
        real_orders_submitted: false,
        source_refs: admission.source_refs.clone(),
    };
    let raw = serde_json::to_vec_pretty(&artifact)
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_preflight"))?;
    write_new_run_file(directory, "preflight.json", &raw)?;
    Ok(raw)
}

fn write_live_market_data_node_config(
    directory: &cap_std::fs::Dir,
    run_id: &str,
) -> Result<(), ProductError> {
    let raw = format!(
        "[live_market_data]\n\
         schema_version = \"ntpro.live_market_data_node.v1\"\n\
         mode = \"production-market-data\"\n\
         environment = \"live\"\n\
         node_id = \"{run_id}\"\n\
         trader_id = \"TRADER-001\"\n\
         venue = \"BINANCE\"\n\
         product_type = \"spot\"\n\
         api_key_env = \"NTPRO_BINANCE_LIVE_API_KEY\"\n\
         api_secret_env = \"NTPRO_BINANCE_LIVE_API_SECRET\"\n\
         execution_client_enabled = false\n\
         order_endpoint_access_allowed = false\n\
         order_submission_allowed = false\n\
         automatic_reconnect_allowed = false\n\n\
         [shutdown]\n\
         mode = \"start-stop\"\n\
         post_stop_delay_secs = 0\n\
         connection_timeout_secs = 10\n\
         disconnection_timeout_secs = 10\n"
    );
    write_new_run_file(directory, LIVE_MARKET_DATA_NODE_CONFIG_FILE, raw.as_bytes())
}

fn live_market_data_runtime_root(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<PathBuf, ProductError> {
    let root = mvp_workspace_root(&state.registry_path)?
        .join("artifacts/live-market-data-runtime")
        .join(run_id);
    fs::create_dir_all(&root)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_runtime_root"))?;
    canonical_path(&root, "live_runtime_root")
}

fn load_live_run_candidate(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<LiveRunCandidate, ProductError> {
    load_live_run_candidate_snapshot(state, run_id).map(|(candidate, _, _)| candidate)
}

fn load_live_run_candidate_snapshot(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<(LiveRunCandidate, LiveRunCandidateManifest, Vec<u8>), ProductError> {
    validate_workspace_anchor_head(state)?;
    let (manifest, manifest_raw) = load_live_run_manifest(state, run_id)?;
    let root = canonical_live_run_root(state, false)?.join(run_id);
    let manifest_sha256 = sha256_ref(&manifest_raw);
    let state_history = load_live_run_state_history(state, run_id, &manifest_sha256)?;
    let (candidate_state, _) = state_history
        .last()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "live_run_state"))?;
    let preflight = read_optional_artifact_with_raw::<LiveRunPreflightArtifact>(
        &root.join("preflight.json"),
        "live_preflight",
    )?;
    let stop = read_optional_artifact_with_raw::<LiveRunStopArtifact>(
        &root.join("stop.json"),
        "live_stop",
    )?;
    validate_action_artifacts(
        &manifest,
        &manifest_sha256,
        preflight.as_ref().map(|(value, _)| value),
        stop.as_ref().map(|(value, _)| value),
    )?;
    validate_live_run_state(
        &manifest,
        candidate_state,
        preflight.as_ref(),
        stop.as_ref(),
    )?;
    validate_candidate_directory_entries(&root, &state_history)?;
    let receipt = load_live_run_anchor_receipt(state, run_id, candidate_state.revision)?;
    let candidate = project_candidate(
        state,
        &manifest,
        candidate_state,
        preflight.as_ref().map(|(value, _)| value),
        stop.as_ref().map(|(value, _)| value),
        &receipt,
    );
    Ok((candidate, manifest, manifest_raw))
}

fn load_live_run_manifest(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<(LiveRunCandidateManifest, Vec<u8>), ProductError> {
    let root = canonical_live_run_root(state, false)?.join(run_id);
    let manifest_raw =
        read_live_run_artifact_bytes(&root.join("run-manifest.json"), "live_manifest")
            .map_err(|_| product_error(ProductErrorKind::RunNotFound, "live_run_candidate"))?;
    let request_raw = read_live_run_artifact_bytes(&root.join("request.json"), "live_request")?;
    let manifest: LiveRunCandidateManifest = serde_json::from_slice(&manifest_raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_manifest"))?;
    validate_identifier("run_id", &manifest.run_id)?;
    validate_identifier("strategy_id", &manifest.strategy_id)?;
    strategy_version::validate_requested_version_id(
        "strategy_version_id",
        &manifest.strategy_version_id,
    )?;
    validate_sha256_hash(
        "strategy_version_content_hash",
        &manifest.strategy_version_content_hash,
    )?;
    if manifest.schema_version != LIVE_RUN_CANDIDATE_MANIFEST_SCHEMA_VERSION
        || manifest.run_id != run_id
        || manifest.environment != "live"
        || manifest.account_ref != "account://live/binance/primary"
        || manifest.venue_ref != "venue://live/BINANCE"
        || manifest.request_sha256 != sha256_ref(&request_raw)
        || manifest.created_at_unix_ms == 0
        || manifest.source_refs.len() != 4
        || manifest.source_refs != sorted_refs(manifest.source_refs.clone())
        || !manifest
            .source_refs
            .contains(&manifest.strategy_version_content_hash)
        || manifest
            .source_refs
            .iter()
            .filter(|value| value.starts_with("node-config:") && value.ends_with("#live_admission"))
            .count()
            != 1
        || manifest
            .source_refs
            .iter()
            .filter(|value| {
                value
                    .strip_prefix("risk-config-sha256:")
                    .is_some_and(|hash| {
                        hash.len() == 64
                            && hash
                                .bytes()
                                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    })
            })
            .count()
            != 1
        || manifest
            .source_refs
            .iter()
            .filter(|value| value.starts_with("node-config:") && value.ends_with("#risk"))
            .count()
            != 1
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_manifest",
        ));
    }
    let request: CreateLiveRunCandidateRequest = serde_json::from_slice(&request_raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_request"))?;
    if request.strategy_id != manifest.strategy_id
        || request.strategy_version_id != manifest.strategy_version_id
        || request.environment != manifest.environment
        || request.account_ref != manifest.account_ref
        || request.venue_ref != manifest.venue_ref
        || !request.user_confirmed
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_request",
        ));
    }
    Ok((manifest, manifest_raw))
}

fn validate_action_artifacts(
    manifest: &LiveRunCandidateManifest,
    manifest_sha256: &str,
    preflight: Option<&LiveRunPreflightArtifact>,
    stop: Option<&LiveRunStopArtifact>,
) -> Result<(), ProductError> {
    if preflight.is_some_and(|value| {
        value.schema_version != LIVE_RUN_PREFLIGHT_SCHEMA_VERSION
            || value.run_id != manifest.run_id
            || value.source_manifest_sha256 != manifest_sha256
            || value.evaluated_at_unix_ms < manifest.created_at_unix_ms
            || !value.account_connected
            || !value.account_can_trade_verified
            || !value.runtime_gates_verified
            || value.order_endpoint_access_attempted
            || value.execution_adapter_send_attempted
            || value.real_orders_submitted
            || value.source_refs != manifest.source_refs
    }) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_preflight",
        ));
    }
    if let Some(value) = stop {
        let earliest = preflight.map_or(manifest.created_at_unix_ms, |item| {
            item.evaluated_at_unix_ms
        });
        if value.schema_version != LIVE_RUN_STOP_SCHEMA_VERSION
            || value.run_id != manifest.run_id
            || value.source_manifest_sha256 != manifest_sha256
            || value.stopped_at_unix_ms < earliest
            || !value.manual_stop
            || value.order_endpoint_access_attempted
            || value.execution_adapter_send_attempted
            || value.real_orders_submitted
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_stop",
            ));
        }
    }
    Ok(())
}

fn project_candidate(
    server_state: &DashboardServerState,
    manifest: &LiveRunCandidateManifest,
    candidate_state: &LiveRunCandidateState,
    preflight: Option<&LiveRunPreflightArtifact>,
    stop: Option<&LiveRunStopArtifact>,
    receipt: &LiveRunAnchorReceipt,
) -> LiveRunCandidate {
    let lifecycle = candidate_state.lifecycle;
    let runtime = project_live_market_data_runtime(server_state, manifest, lifecycle);
    LiveRunCandidate {
        run_id: manifest.run_id.clone(),
        strategy_id: manifest.strategy_id.clone(),
        strategy_version_id: manifest.strategy_version_id.clone(),
        environment: manifest.environment.clone(),
        account_ref: manifest.account_ref.clone(),
        venue_ref: manifest.venue_ref.clone(),
        lifecycle,
        created_at_unix_ms: manifest.created_at_unix_ms,
        preflight_at_unix_ms: preflight.map(|value| value.evaluated_at_unix_ms),
        stopped_at_unix_ms: stop.map(|value| value.stopped_at_unix_ms),
        account_connected: preflight.is_some(),
        account_can_trade_verified: preflight.is_some(),
        runtime_started: runtime.0,
        market_data_connected: runtime.1,
        runtime_node_id: runtime.2,
        runtime_process_state: runtime.3,
        runtime_error: runtime.4,
        audit_anchor: LiveRunAuditAnchorSnapshot {
            status: "verified_external_monotonic_anchor".to_string(),
            namespace: receipt.namespace.clone(),
            revision: receipt.revision,
            workspace_revision: receipt.workspace_revision,
            receipt_ref: receipt.sha256(),
            key_id: receipt.key_id.clone(),
            anchored_at_unix_ms: receipt.anchored_at_unix_ms,
            workspace_snapshot_rollback_detectable: true,
            trading_authority_granted: false,
        },
        order_admission: LiveOrderAdmissionSnapshot::blocked(),
        source_refs: manifest.source_refs.clone(),
    }
}

fn project_live_market_data_runtime(
    state: &DashboardServerState,
    manifest: &LiveRunCandidateManifest,
    lifecycle: LiveRunCandidateLifecycle,
) -> (bool, bool, Option<String>, String, Option<String>) {
    let runtime_config_exists = canonical_live_run_root(state, false).is_ok_and(|root| {
        root.join(&manifest.run_id)
            .join(LIVE_MARKET_DATA_NODE_CONFIG_FILE)
            .is_file()
    });
    if !matches!(
        lifecycle,
        LiveRunCandidateLifecycle::Starting
            | LiveRunCandidateLifecycle::MarketDataRunning
            | LiveRunCandidateLifecycle::Stopping
            | LiveRunCandidateLifecycle::Stopped
            | LiveRunCandidateLifecycle::Failed
    ) || !runtime_config_exists
    {
        return (false, false, None, "not_started".to_string(), None);
    }
    let node_id = manifest.run_id.clone();
    let store = SupervisorRegistryStore::new(&state.registry_path);
    let record = match store.refresh_status_from_artifact(&node_id) {
        Ok(record) => record,
        Err(_) => {
            return (
                false,
                false,
                Some(node_id),
                "unavailable".to_string(),
                Some("live_market_data_runtime_status_unavailable".to_string()),
            );
        }
    };
    let running = record.process.state == SupervisorProcessState::Running
        && record.last_known_status.lifecycle_state == LifecycleStatus::Running;
    let data_connected = running
        && record.last_known_status.data_connection == ConnectionStatus::Connected
        && record.last_known_status.external_venue_connection;
    let execution_disabled = record.last_known_status.execution_connection
        == ConnectionStatus::NotConfigured
        && !record
            .last_known_status
            .execution
            .started
            .value
            .unwrap_or(true)
        && !record.last_known_status.real_orders_submitted;
    let lifecycle_exposes_runtime = matches!(
        lifecycle,
        LiveRunCandidateLifecycle::MarketDataRunning | LiveRunCandidateLifecycle::Stopping
    );
    let error = if matches!(lifecycle, LiveRunCandidateLifecycle::MarketDataRunning)
        && (!running || !data_connected || !execution_disabled)
    {
        Some("live_market_data_runtime_boundary_violation".to_string())
    } else if record.last_known_status.last_error.is_some() {
        Some("live_market_data_runtime_reported_error".to_string())
    } else {
        None
    };
    (
        lifecycle_exposes_runtime && running && execution_disabled,
        lifecycle_exposes_runtime && data_connected && execution_disabled,
        Some(node_id),
        supervisor_process_state_label(record.process.state).to_string(),
        error,
    )
}

const fn supervisor_process_state_label(state: SupervisorProcessState) -> &'static str {
    match state {
        SupervisorProcessState::NotStarted => "not_started",
        SupervisorProcessState::Running => "running",
        SupervisorProcessState::Stopped => "stopped",
        SupervisorProcessState::Stale => "stale",
        SupervisorProcessState::Unknown => "unknown",
    }
}

fn create_live_run_candidate_directory(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<cap_std::fs::Dir, ProductError> {
    let root_path = canonical_live_run_root(state, true)?;
    let root = open_absolute_directory_nofollow(&root_path)?;
    root.create_dir(run_id).map_err(|error| {
        if error.kind() == ErrorKind::AlreadyExists {
            product_error(ProductErrorKind::LiveConflict, "run_id")
        } else {
            product_error(ProductErrorKind::SourceUnavailable, "live_run_root")
        }
    })?;
    root.open_dir_nofollow(run_id)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_root_containment"))
}

fn write_initial_live_run_state(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
    created_at_unix_ms: u64,
) -> Result<(), ProductError> {
    write_live_run_state(
        state,
        run_id,
        &LiveRunCandidateState {
            schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
            run_id: run_id.to_string(),
            source_manifest_sha256: manifest_sha256.to_string(),
            revision: 0,
            previous_state_sha256: None,
            lifecycle: LiveRunCandidateLifecycle::Created,
            preflight_sha256: None,
            stop_sha256: None,
            updated_at_unix_ms: created_at_unix_ms,
        },
    )
}

fn write_live_run_state(
    server_state: &DashboardServerState,
    run_id: &str,
    candidate_state: &LiveRunCandidateState,
) -> Result<(), ProductError> {
    let run_root = canonical_live_run_root(server_state, false)?.join(run_id);
    let run_directory = open_absolute_directory_nofollow(&run_root)?;
    let state_raw = serde_json::to_vec_pretty(candidate_state)
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_run_state"))?;
    let previous_commit_sha256 = if candidate_state.revision == 0 {
        None
    } else {
        Some(sha256_ref(&load_live_run_state_commit_raw(
            server_state,
            run_id,
            candidate_state.revision - 1,
        )?))
    };
    let commit = LiveRunStateCommit {
        schema_version: LIVE_RUN_STATE_COMMIT_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        revision: candidate_state.revision,
        state_sha256: sha256_ref(&state_raw),
        previous_commit_sha256: previous_commit_sha256.clone(),
        committed_at_unix_ms: candidate_state.updated_at_unix_ms,
    };
    let commit_raw = serde_json::to_vec_pretty(&commit).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_run_state_commit",
        )
    })?;
    let previous_workspace_receipt = load_local_workspace_anchor_head(server_state)?;
    let workspace_revision = previous_workspace_receipt
        .as_ref()
        .map_or(0, |receipt| receipt.workspace_revision + 1);
    let previous_receipt_sha256 = previous_workspace_receipt
        .as_ref()
        .map(LiveRunAnchorReceipt::sha256);
    let anchor_request = LiveRunAnchorAppendRequest::new(
        server_state.live_run_audit_anchor.namespace()?,
        run_id,
        LiveRunAnchorRevision::new(candidate_state.revision, workspace_revision),
        sha256_ref(&state_raw),
        sha256_ref(&commit_raw),
        previous_receipt_sha256,
        candidate_state.updated_at_unix_ms,
    );
    let receipt = server_state.live_run_audit_anchor.append(&anchor_request)?;
    server_state
        .live_run_audit_anchor
        .validate_receipt(&receipt, &anchor_request)?;
    let receipt_raw = serde_json::to_vec_pretty(&receipt).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_run_audit_anchor_receipt",
        )
    })?;
    write_new_run_file(
        &run_directory,
        &live_run_state_file_name(candidate_state.revision),
        &state_raw,
    )?;
    let commit_directory = open_live_run_state_commit_directory(server_state, true)?;
    write_new_run_file(
        &commit_directory,
        &live_run_state_commit_file_name(run_id, candidate_state.revision),
        &commit_raw,
    )?;
    write_new_run_file(
        &run_directory,
        &live_run_anchor_receipt_file_name(candidate_state.revision),
        &receipt_raw,
    )?;
    publish_live_run_state_head(
        &run_directory,
        candidate_state,
        &state_raw,
        &commit_raw,
        &receipt,
        previous_commit_sha256.as_deref(),
    )?;
    publish_workspace_anchor_head(server_state, &receipt)?;

    let (persisted, persisted_raw) = load_live_run_state(
        server_state,
        run_id,
        &candidate_state.source_manifest_sha256,
    )?;
    if persisted != *candidate_state || persisted_raw != state_raw {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_state",
        ));
    }
    Ok(())
}

fn publish_live_run_state_head(
    directory: &cap_std::fs::Dir,
    state: &LiveRunCandidateState,
    state_raw: &[u8],
    commit_raw: &[u8],
    receipt: &LiveRunAnchorReceipt,
    previous_commit_sha256: Option<&str>,
) -> Result<(), ProductError> {
    let head = LiveRunStateHead {
        schema_version: LIVE_RUN_STATE_HEAD_SCHEMA_VERSION.to_string(),
        run_id: state.run_id.clone(),
        revision: state.revision,
        state_sha256: sha256_ref(state_raw),
        commit_sha256: sha256_ref(commit_raw),
        anchor_receipt_sha256: receipt.sha256(),
        updated_at_unix_ms: state.updated_at_unix_ms,
    };
    let raw = serde_json::to_vec_pretty(&head)
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_run_state_head"))?;
    if state.revision == 0 {
        write_new_run_file(directory, LIVE_RUN_STATE_HEAD_FILE, &raw)?;
    } else {
        let current_raw = read_run_file_from_directory(
            directory,
            LIVE_RUN_STATE_HEAD_FILE,
            "live_run_state_head",
        )?;
        let current: LiveRunStateHead = serde_json::from_slice(&current_raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_state_head"))?;
        if current.schema_version != LIVE_RUN_STATE_HEAD_SCHEMA_VERSION
            || current.run_id != state.run_id
            || current.revision + 1 != state.revision
            || Some(current.state_sha256.as_str()) != state.previous_state_sha256.as_deref()
            || Some(current.commit_sha256.as_str()) != previous_commit_sha256
            || current.anchor_receipt_sha256
                != receipt.previous_receipt_sha256.clone().unwrap_or_default()
            || current.updated_at_unix_ms > state.updated_at_unix_ms
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_run_state_head",
            ));
        }
        write_new_run_file(directory, LIVE_RUN_STATE_HEAD_NEXT_FILE, &raw)?;
        directory
            .rename(
                LIVE_RUN_STATE_HEAD_NEXT_FILE,
                directory,
                LIVE_RUN_STATE_HEAD_FILE,
            )
            .map_err(|_| {
                product_error(ProductErrorKind::SourceUnavailable, "live_run_state_head")
            })?;
    }
    let persisted =
        read_run_file_from_directory(directory, LIVE_RUN_STATE_HEAD_FILE, "live_run_state_head")?;
    if persisted != raw {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_state_head",
        ));
    }
    Ok(())
}

fn load_live_run_state(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
) -> Result<(LiveRunCandidateState, Vec<u8>), ProductError> {
    let history = load_live_run_state_history(state, run_id, manifest_sha256)?;
    history
        .last()
        .cloned()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "live_run_state"))
}

fn load_live_run_state_history(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
) -> Result<Vec<(LiveRunCandidateState, Vec<u8>)>, ProductError> {
    let commit_root = canonical_live_run_state_commit_root(state, false)?;
    if !commit_root.exists() {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_state_commit",
        ));
    }
    let mut revisions = Vec::new();
    for entry in fs::read_dir(&commit_root)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_state_commit"))?
    {
        let entry = entry.map_err(|_| {
            product_error(ProductErrorKind::SourceUnavailable, "live_run_state_commit")
        })?;
        let file_type = entry.file_type().map_err(|_| {
            product_error(ProductErrorKind::SourceUnavailable, "live_run_state_commit")
        })?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "live_run_state_commit",
            ));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_state_commit"))?;
        let (commit_run_id, raw_revision) = parse_live_run_state_commit_file_name(&name)?;
        if commit_run_id == run_id {
            revisions.push(raw_revision);
        }
    }
    revisions.sort_unstable();
    revisions.dedup();
    if revisions.is_empty()
        || revisions
            .iter()
            .enumerate()
            .any(|(index, revision)| *revision != index as u64)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_state_commit",
        ));
    }

    let run_root = canonical_live_run_root(state, false)?.join(run_id);
    let mut history = Vec::with_capacity(revisions.len());
    let mut previous_state_sha256: Option<String> = None;
    let mut previous_commit_sha256: Option<String> = None;
    for revision in revisions {
        let state_raw = read_live_run_artifact_bytes(
            &run_root.join(live_run_state_file_name(revision)),
            "live_run_state",
        )?;
        let candidate_state: LiveRunCandidateState = serde_json::from_slice(&state_raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_state"))?;
        let commit_raw = load_live_run_state_commit_raw(state, run_id, revision)?;
        let commit: LiveRunStateCommit = serde_json::from_slice(&commit_raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_state_commit"))?;
        let receipt = load_live_run_anchor_receipt(state, run_id, revision)?;
        let anchor_request = LiveRunAnchorAppendRequest::new(
            state.live_run_audit_anchor.namespace()?,
            run_id,
            LiveRunAnchorRevision::new(revision, receipt.workspace_revision),
            sha256_ref(&state_raw),
            sha256_ref(&commit_raw),
            receipt.previous_receipt_sha256.clone(),
            candidate_state.updated_at_unix_ms,
        );
        state
            .live_run_audit_anchor
            .validate_receipt(&receipt, &anchor_request)?;
        if candidate_state.schema_version != LIVE_RUN_STATE_SCHEMA_VERSION
            || candidate_state.run_id != run_id
            || candidate_state.source_manifest_sha256 != manifest_sha256
            || candidate_state.revision != revision
            || candidate_state.previous_state_sha256 != previous_state_sha256
            || candidate_state.updated_at_unix_ms == 0
            || commit.schema_version != LIVE_RUN_STATE_COMMIT_SCHEMA_VERSION
            || commit.run_id != run_id
            || commit.revision != revision
            || commit.state_sha256 != sha256_ref(&state_raw)
            || commit.previous_commit_sha256 != previous_commit_sha256
            || commit.committed_at_unix_ms != candidate_state.updated_at_unix_ms
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_run_state_commit",
            ));
        }
        if let Some((previous, _)) = history.last() {
            validate_live_run_state_transition(previous, &candidate_state)?;
        } else if revision != 0 || candidate_state.lifecycle != LiveRunCandidateLifecycle::Created {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_run_state",
            ));
        }
        previous_state_sha256 = Some(sha256_ref(&state_raw));
        previous_commit_sha256 = Some(sha256_ref(&commit_raw));
        history.push((candidate_state, state_raw));
    }
    let (latest_state, latest_state_raw) = history
        .last()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "live_run_state"))?;
    let head_raw = read_live_run_artifact_bytes(
        &run_root.join(LIVE_RUN_STATE_HEAD_FILE),
        "live_run_state_head",
    )?;
    let head: LiveRunStateHead = serde_json::from_slice(&head_raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_state_head"))?;
    if head.schema_version != LIVE_RUN_STATE_HEAD_SCHEMA_VERSION
        || head.run_id != run_id
        || head.revision != latest_state.revision
        || head.state_sha256 != sha256_ref(latest_state_raw)
        || Some(head.commit_sha256.as_str()) != previous_commit_sha256.as_deref()
        || head.anchor_receipt_sha256
            != load_live_run_anchor_receipt(state, run_id, latest_state.revision)?.sha256()
        || head.updated_at_unix_ms != latest_state.updated_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_state_head",
        ));
    }
    Ok(history)
}

fn validate_live_run_state_transition(
    previous: &LiveRunCandidateState,
    current: &LiveRunCandidateState,
) -> Result<(), ProductError> {
    let lifecycle_fields_valid = match (previous.lifecycle, current.lifecycle) {
        (LiveRunCandidateLifecycle::Created, LiveRunCandidateLifecycle::PreflightReady) => {
            previous.preflight_sha256.is_none()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256.is_some()
                && current.stop_sha256.is_none()
        }
        (LiveRunCandidateLifecycle::Created, LiveRunCandidateLifecycle::Stopped) => {
            previous.preflight_sha256.is_none()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256.is_none()
                && current.stop_sha256.is_some()
        }
        (LiveRunCandidateLifecycle::PreflightReady, LiveRunCandidateLifecycle::Stopped) => {
            previous.preflight_sha256.is_some()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.stop_sha256.is_some()
        }
        (LiveRunCandidateLifecycle::PreflightReady, LiveRunCandidateLifecycle::Starting)
        | (
            LiveRunCandidateLifecycle::Starting,
            LiveRunCandidateLifecycle::MarketDataRunning | LiveRunCandidateLifecycle::Failed,
        )
        | (
            LiveRunCandidateLifecycle::MarketDataRunning,
            LiveRunCandidateLifecycle::Stopping | LiveRunCandidateLifecycle::Failed,
        ) => {
            previous.preflight_sha256.is_some()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.stop_sha256.is_none()
        }
        (LiveRunCandidateLifecycle::Stopping, LiveRunCandidateLifecycle::Stopped) => {
            previous.preflight_sha256.is_some()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.stop_sha256.is_some()
        }
        _ => false,
    };
    let allowed = lifecycle_fields_valid
        && current.revision == previous.revision + 1
        && current.updated_at_unix_ms >= previous.updated_at_unix_ms
        && current.source_manifest_sha256 == previous.source_manifest_sha256;
    if !allowed {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_state_transition",
        ));
    }
    Ok(())
}

fn live_run_state_file_name(revision: u64) -> String {
    format!("state-{revision:020}.json")
}

fn live_run_anchor_receipt_file_name(revision: u64) -> String {
    format!("anchor-receipt-{revision:020}.json")
}

fn load_live_run_anchor_receipt(
    state: &DashboardServerState,
    run_id: &str,
    revision: u64,
) -> Result<LiveRunAnchorReceipt, ProductError> {
    let raw = read_live_run_artifact_bytes(
        &canonical_live_run_root(state, false)?
            .join(run_id)
            .join(live_run_anchor_receipt_file_name(revision)),
        "live_run_audit_anchor_receipt",
    )?;
    let receipt: LiveRunAnchorReceipt = serde_json::from_slice(&raw).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_audit_anchor_receipt",
        )
    })?;
    if receipt.schema_version != LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION
        || receipt.run_id != run_id
        || receipt.revision != revision
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_audit_anchor_receipt",
        ));
    }
    Ok(receipt)
}

fn live_run_state_commit_file_name(run_id: &str, revision: u64) -> String {
    format!("{run_id}.state.{revision:020}.json")
}

fn parse_live_run_state_commit_file_name(name: &str) -> Result<(&str, u64), ProductError> {
    let (run_id, suffix) = name
        .rsplit_once(".state.")
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "live_run_state_commit"))?;
    validate_identifier("live_run_state_commit", run_id)?;
    let raw_revision = suffix
        .strip_suffix(".json")
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "live_run_state_commit"))?;
    let revision = raw_revision
        .parse::<u64>()
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_state_commit"))?;
    if raw_revision.len() != 20 || suffix != format!("{revision:020}.json") {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_state_commit",
        ));
    }
    Ok((run_id, revision))
}

fn load_live_run_state_commit_raw(
    state: &DashboardServerState,
    run_id: &str,
    revision: u64,
) -> Result<Vec<u8>, ProductError> {
    read_live_run_artifact_bytes(
        &canonical_live_run_state_commit_root(state, false)?
            .join(live_run_state_commit_file_name(run_id, revision)),
        "live_run_state_commit",
    )
}

fn validate_live_run_state(
    manifest: &LiveRunCandidateManifest,
    state: &LiveRunCandidateState,
    preflight: Option<&(LiveRunPreflightArtifact, Vec<u8>)>,
    stop: Option<&(LiveRunStopArtifact, Vec<u8>)>,
) -> Result<(), ProductError> {
    let preflight_sha = preflight.map(|(_, raw)| sha256_ref(raw));
    let stop_sha = stop.map(|(_, raw)| sha256_ref(raw));
    let valid = match state.lifecycle {
        LiveRunCandidateLifecycle::Created => {
            state.revision == 0
                && state.updated_at_unix_ms == manifest.created_at_unix_ms
                && state.preflight_sha256.is_none()
                && state.stop_sha256.is_none()
                && preflight.is_none()
                && stop.is_none()
        }
        LiveRunCandidateLifecycle::PreflightReady => {
            let evaluated_at = preflight.map(|(value, _)| value.evaluated_at_unix_ms);
            state.revision == 1
                && state.preflight_sha256 == preflight_sha
                && state.stop_sha256.is_none()
                && stop.is_none()
                && Some(state.updated_at_unix_ms) == evaluated_at
        }
        LiveRunCandidateLifecycle::Starting => {
            state.revision == 2
                && state.preflight_sha256 == preflight_sha
                && state.stop_sha256.is_none()
                && preflight.is_some()
                && stop.is_none()
        }
        LiveRunCandidateLifecycle::MarketDataRunning => {
            state.revision == 3
                && state.preflight_sha256 == preflight_sha
                && state.stop_sha256.is_none()
                && preflight.is_some()
                && stop.is_none()
        }
        LiveRunCandidateLifecycle::Stopping => {
            state.revision == 4
                && state.preflight_sha256 == preflight_sha
                && state.stop_sha256.is_none()
                && preflight.is_some()
                && stop.is_none()
        }
        LiveRunCandidateLifecycle::Failed => {
            matches!(state.revision, 3 | 4)
                && state.preflight_sha256 == preflight_sha
                && state.stop_sha256.is_none()
                && preflight.is_some()
                && stop.is_none()
        }
        LiveRunCandidateLifecycle::Stopped => {
            let stopped_at = stop.map(|(value, _)| value.stopped_at_unix_ms);
            let valid_revision = if preflight.is_some() {
                matches!(state.revision, 2 | 5)
            } else {
                state.revision == 1
            };
            valid_revision
                && state.preflight_sha256 == preflight_sha
                && state.stop_sha256 == stop_sha
                && Some(state.updated_at_unix_ms) == stopped_at
                && stop
                    .as_ref()
                    .is_some_and(|(value, _)| value.source_preflight_sha256 == preflight_sha)
        }
    };
    if !valid {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_state",
        ));
    }
    Ok(())
}

fn validate_candidate_directory_entries(
    root: &Path,
    state_history: &[(LiveRunCandidateState, Vec<u8>)],
) -> Result<(), ProductError> {
    let mut expected = BTreeSet::from([
        "request.json".to_string(),
        "run-manifest.json".to_string(),
        LIVE_RUN_STATE_HEAD_FILE.to_string(),
    ]);
    for (state, _) in state_history {
        expected.insert(live_run_state_file_name(state.revision));
        expected.insert(live_run_anchor_receipt_file_name(state.revision));
    }
    let state = state_history
        .last()
        .map(|(state, _)| state)
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "live_run_state"))?;
    if state.preflight_sha256.is_some() {
        expected.insert("preflight.json".to_string());
    }
    if state.stop_sha256.is_some() {
        expected.insert("stop.json".to_string());
    }
    if state_history.iter().any(|(state, _)| {
        matches!(
            state.lifecycle,
            LiveRunCandidateLifecycle::Starting
                | LiveRunCandidateLifecycle::MarketDataRunning
                | LiveRunCandidateLifecycle::Stopping
                | LiveRunCandidateLifecycle::Failed
        )
    }) {
        expected.insert(LIVE_MARKET_DATA_NODE_CONFIG_FILE.to_string());
    }
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(root)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_candidate"))?
    {
        let entry = entry.map_err(|_| {
            product_error(ProductErrorKind::SourceUnavailable, "live_run_candidate")
        })?;
        let file_type = entry.file_type().map_err(|_| {
            product_error(ProductErrorKind::SourceUnavailable, "live_run_candidate")
        })?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "live_run_candidate_artifacts",
            ));
        }
        actual.insert(
            entry.file_name().into_string().map_err(|_| {
                product_error(ProductErrorKind::SourceInvalid, "live_run_candidate")
            })?,
        );
    }
    if actual != expected {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_candidate_artifacts",
        ));
    }
    Ok(())
}

fn live_run_workspace_anchor_head_path(
    state: &DashboardServerState,
) -> Result<PathBuf, ProductError> {
    let live_run_root = canonical_live_run_root(state, false)?;
    let artifacts = live_run_root
        .parent()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "artifact_root"))?;
    let canonical_artifacts = canonical_path(artifacts, "artifact_root")?;
    Ok(canonical_artifacts.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE))
}

fn load_local_workspace_anchor_head(
    state: &DashboardServerState,
) -> Result<Option<LiveRunAnchorReceipt>, ProductError> {
    let head_path = live_run_workspace_anchor_head_path(state)?;
    let next_path = head_path.with_file_name(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_NEXT_FILE);
    if next_path.exists() {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_head",
        ));
    }
    read_optional_artifact_with_raw::<LiveRunAnchorReceipt>(
        &head_path,
        "live_run_workspace_anchor_head",
    )
    .map(|value| value.map(|(receipt, _)| receipt))
}

fn publish_workspace_anchor_head(
    state: &DashboardServerState,
    receipt: &LiveRunAnchorReceipt,
) -> Result<(), ProductError> {
    let head_path = live_run_workspace_anchor_head_path(state)?;
    let artifacts = open_absolute_directory_nofollow(
        head_path
            .parent()
            .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "artifact_root"))?,
    )?;
    let previous = load_local_workspace_anchor_head(state)?;
    let expected_revision = previous
        .as_ref()
        .map_or(0, |value| value.workspace_revision + 1);
    let expected_previous = previous.as_ref().map(LiveRunAnchorReceipt::sha256);
    if receipt.workspace_revision != expected_revision
        || receipt.previous_receipt_sha256 != expected_previous
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_head",
        ));
    }
    let raw = serde_json::to_vec_pretty(receipt).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_run_workspace_anchor_head",
        )
    })?;
    if previous.is_none() {
        write_new_run_file(&artifacts, LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE, &raw)?;
    } else {
        write_new_run_file(&artifacts, LIVE_RUN_WORKSPACE_ANCHOR_HEAD_NEXT_FILE, &raw)?;
        artifacts
            .rename(
                LIVE_RUN_WORKSPACE_ANCHOR_HEAD_NEXT_FILE,
                &artifacts,
                LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE,
            )
            .map_err(|_| {
                product_error(
                    ProductErrorKind::SourceUnavailable,
                    "live_run_workspace_anchor_head",
                )
            })?;
    }
    let persisted = read_run_file_from_directory(
        &artifacts,
        LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE,
        "live_run_workspace_anchor_head",
    )?;
    if persisted != raw {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_workspace_anchor_head",
        ));
    }
    Ok(())
}

fn parse_live_run_anchor_receipt_file_name(name: &str) -> Result<u64, ProductError> {
    let raw_revision = name
        .strip_prefix("anchor-receipt-")
        .and_then(|value| value.strip_suffix(".json"))
        .ok_or_else(|| {
            product_error(
                ProductErrorKind::SourceInvalid,
                "live_run_audit_anchor_receipt",
            )
        })?;
    let revision = raw_revision.parse::<u64>().map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_audit_anchor_receipt",
        )
    })?;
    if raw_revision.len() != 20 || name != live_run_anchor_receipt_file_name(revision) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_audit_anchor_receipt",
        ));
    }
    Ok(revision)
}

fn load_workspace_anchor_receipts(
    state: &DashboardServerState,
) -> Result<Vec<LiveRunAnchorReceipt>, ProductError> {
    let root = canonical_live_run_root(state, false)?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut receipts = Vec::new();
    for entry in fs::read_dir(&root)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_root"))?
    {
        let entry = entry
            .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_root"))?;
        let file_type = entry
            .file_type()
            .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_root"))?;
        if !file_type.is_dir() || file_type.is_symlink() {
            continue;
        }
        let run_id = entry
            .file_name()
            .into_string()
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_root"))?;
        validate_identifier("run_id", &run_id)?;
        for artifact in fs::read_dir(entry.path()).map_err(|_| {
            product_error(
                ProductErrorKind::SourceUnavailable,
                "live_run_audit_anchor_receipt",
            )
        })? {
            let artifact = artifact.map_err(|_| {
                product_error(
                    ProductErrorKind::SourceUnavailable,
                    "live_run_audit_anchor_receipt",
                )
            })?;
            let name = artifact.file_name().into_string().map_err(|_| {
                product_error(
                    ProductErrorKind::SourceInvalid,
                    "live_run_audit_anchor_receipt",
                )
            })?;
            if !name.starts_with("anchor-receipt-") {
                continue;
            }
            let file_type = artifact.file_type().map_err(|_| {
                product_error(
                    ProductErrorKind::SourceUnavailable,
                    "live_run_audit_anchor_receipt",
                )
            })?;
            if !file_type.is_file() || file_type.is_symlink() {
                return Err(product_error(
                    ProductErrorKind::SourceInvalid,
                    "live_run_audit_anchor_receipt",
                ));
            }
            let revision = parse_live_run_anchor_receipt_file_name(&name)?;
            let receipt = load_live_run_anchor_receipt(state, &run_id, revision)?;
            receipts.push(receipt);
        }
    }
    receipts.sort_by_key(|receipt| receipt.workspace_revision);
    Ok(receipts)
}

fn validate_workspace_anchor_head(
    state: &DashboardServerState,
) -> Result<Option<LiveRunAnchorReceipt>, ProductError> {
    let local = load_local_workspace_anchor_head(state)?;
    let remote = state.live_run_audit_anchor.latest()?;
    if local != remote {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_latest",
        ));
    }
    let Some(latest) = local else {
        if !load_workspace_anchor_receipts(state)?.is_empty() {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_run_workspace_anchor_head",
            ));
        }
        return Ok(None);
    };
    let receipts = load_workspace_anchor_receipts(state)?;
    if receipts.len() != latest.workspace_revision as usize + 1 {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_history",
        ));
    }
    let mut previous: Option<&LiveRunAnchorReceipt> = None;
    for (index, receipt) in receipts.iter().enumerate() {
        let request = LiveRunAnchorAppendRequest::new(
            state.live_run_audit_anchor.namespace()?,
            &receipt.run_id,
            LiveRunAnchorRevision::new(receipt.revision, receipt.workspace_revision),
            receipt.state_sha256.clone(),
            receipt.commit_sha256.clone(),
            receipt.previous_receipt_sha256.clone(),
            receipt.anchored_at_unix_ms,
        );
        state
            .live_run_audit_anchor
            .validate_receipt(receipt, &request)?;
        if receipt.workspace_revision != index as u64
            || receipt.previous_receipt_sha256 != previous.map(LiveRunAnchorReceipt::sha256)
            || previous.is_some_and(|value| receipt.anchored_at_unix_ms < value.anchored_at_unix_ms)
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_run_workspace_anchor_history",
            ));
        }
        previous = Some(receipt);
    }
    if receipts.last() != Some(&latest) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_latest",
        ));
    }
    Ok(Some(latest))
}

fn load_active_live_run_candidates(
    state: &DashboardServerState,
) -> Result<Vec<(LiveRunCandidate, LiveRunCandidateManifest, Vec<u8>)>, ProductError> {
    validate_workspace_anchor_head(state)?;
    let root = canonical_live_run_root(state, false)?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    let pointer = load_active_live_run_pointer(state)?;
    let mut active = Vec::new();
    for entry in fs::read_dir(&root)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_root"))?
    {
        let entry = entry
            .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_root"))?;
        let file_type = entry
            .file_type()
            .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_root"))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_root"))?;
        if file_type.is_symlink() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "live_run_root_containment",
            ));
        }
        if file_type.is_file() && name == LIVE_RUN_ACTIVE_FILE {
            continue;
        }
        if !file_type.is_dir() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "live_run_root_artifacts",
            ));
        }
        let snapshot = load_live_run_candidate_snapshot(state, &name)?;
        if !matches!(
            snapshot.0.lifecycle,
            LiveRunCandidateLifecycle::Stopped | LiveRunCandidateLifecycle::Failed
        ) {
            active.push(snapshot);
        }
    }
    match (pointer, active.len()) {
        (None, 0) => Ok(Vec::new()),
        (Some(pointer), 0) => {
            let snapshot = load_live_run_candidate_snapshot(state, &pointer.run_id)?;
            if !matches!(
                snapshot.0.lifecycle,
                LiveRunCandidateLifecycle::Stopped | LiveRunCandidateLifecycle::Failed
            ) || sha256_ref(&snapshot.2) != pointer.source_manifest_sha256
            {
                return Err(product_error(
                    ProductErrorKind::SourceInvalid,
                    "active_live_run_candidate",
                ));
            }
            Ok(Vec::new())
        }
        (Some(pointer), 1)
            if active[0].0.run_id == pointer.run_id
                && sha256_ref(&active[0].2) == pointer.source_manifest_sha256 =>
        {
            Ok(active)
        }
        _ => Err(product_error(
            ProductErrorKind::SourceInvalid,
            "active_live_run_candidate",
        )),
    }
}

fn load_active_live_run_pointer(
    state: &DashboardServerState,
) -> Result<Option<ActiveLiveRunCandidate>, ProductError> {
    let path = canonical_live_run_root(state, false)?.join(LIVE_RUN_ACTIVE_FILE);
    let Some((pointer, _)) = read_optional_artifact_with_raw::<ActiveLiveRunCandidate>(
        &path,
        "active_live_run_candidate",
    )?
    else {
        return Ok(None);
    };
    validate_identifier("run_id", &pointer.run_id)?;
    validate_sha256_hash("source_manifest_sha256", &pointer.source_manifest_sha256)?;
    if pointer.schema_version != LIVE_RUN_ACTIVE_SCHEMA_VERSION || pointer.claimed_at_unix_ms == 0 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "active_live_run_candidate",
        ));
    }
    Ok(Some(pointer))
}

fn claim_active_live_run_candidate(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
    now: u64,
) -> Result<(), ProductError> {
    let root_path = canonical_live_run_root(state, false)?;
    let root = open_absolute_directory_nofollow(&root_path)?;
    let pointer = ActiveLiveRunCandidate {
        schema_version: LIVE_RUN_ACTIVE_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        source_manifest_sha256: manifest_sha256.to_string(),
        claimed_at_unix_ms: now,
    };
    let raw = serde_json::to_vec_pretty(&pointer).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "active_live_run_candidate",
        )
    })?;
    match write_new_run_file(&root, LIVE_RUN_ACTIVE_FILE, &raw) {
        Ok(()) => Ok(()),
        Err(error) if error.kind == ProductErrorKind::Conflict => {
            let existing = load_active_live_run_pointer(state)?.ok_or_else(|| {
                product_error(ProductErrorKind::SourceInvalid, "active_live_run_candidate")
            })?;
            let snapshot = load_live_run_candidate_snapshot(state, &existing.run_id)?;
            if matches!(
                snapshot.0.lifecycle,
                LiveRunCandidateLifecycle::Stopped | LiveRunCandidateLifecycle::Failed
            ) && sha256_ref(&snapshot.2) == existing.source_manifest_sha256
            {
                root.remove_file(LIVE_RUN_ACTIVE_FILE).map_err(|_| {
                    product_error(
                        ProductErrorKind::SourceUnavailable,
                        "active_live_run_candidate",
                    )
                })?;
                write_new_run_file(&root, LIVE_RUN_ACTIVE_FILE, &raw).map_err(|_| {
                    product_error(ProductErrorKind::LiveConflict, "active_live_run_candidate")
                })
            } else {
                Err(product_error(
                    ProductErrorKind::LiveConflict,
                    "active_live_run_candidate",
                ))
            }
        }
        Err(error) => Err(error),
    }
}

fn release_active_live_run_candidate(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<(), ProductError> {
    let pointer = load_active_live_run_pointer(state)?.ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "active_live_run_candidate",
        )
    })?;
    let (_, manifest_raw) = load_live_run_manifest(state, run_id)?;
    if pointer.run_id != run_id || pointer.source_manifest_sha256 != sha256_ref(&manifest_raw) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "active_live_run_candidate",
        ));
    }
    let root = open_absolute_directory_nofollow(&canonical_live_run_root(state, false)?)?;
    root.remove_file(LIVE_RUN_ACTIVE_FILE).map_err(|_| {
        product_error(
            ProductErrorKind::SourceUnavailable,
            "active_live_run_candidate",
        )
    })
}

fn canonical_live_run_root(
    state: &DashboardServerState,
    create: bool,
) -> Result<PathBuf, ProductError> {
    let workspace = mvp_workspace_root(&state.registry_path)?;
    let canonical_workspace = canonical_path(&workspace, "workspace")?;
    let artifacts = canonical_path(&workspace.join("artifacts"), "artifact_root")?;
    if artifacts != canonical_workspace.join("artifacts") {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "artifact_root_containment",
        ));
    }
    let candidate = workspace.join("artifacts/live-runs");
    if create && !candidate.exists() {
        let root = open_absolute_directory_nofollow(&artifacts)?;
        match root.create_dir("live-runs") {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(_) => {
                return Err(product_error(
                    ProductErrorKind::SourceUnavailable,
                    "live_run_root",
                ));
            }
        }
    }
    if !candidate.exists() {
        return Ok(candidate);
    }
    let root = canonical_path(&candidate, "live_run_root")?;
    if root != canonical_workspace.join("artifacts/live-runs") {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_root_containment",
        ));
    }
    Ok(root)
}

fn canonical_live_run_state_commit_root(
    state: &DashboardServerState,
    create: bool,
) -> Result<PathBuf, ProductError> {
    let live_run_root = canonical_live_run_root(state, create)?;
    let artifacts = live_run_root
        .parent()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "live_run_state_commit"))?;
    let candidate = artifacts.join(LIVE_RUN_STATE_COMMIT_DIRECTORY);
    if create && !candidate.exists() {
        let root = open_absolute_directory_nofollow(artifacts)?;
        match root.create_dir(LIVE_RUN_STATE_COMMIT_DIRECTORY) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(_) => {
                return Err(product_error(
                    ProductErrorKind::SourceUnavailable,
                    "live_run_state_commit",
                ));
            }
        }
    }
    if !candidate.exists() {
        return Ok(candidate);
    }
    let canonical = canonical_path(&candidate, "live_run_state_commit")?;
    if canonical != artifacts.join(LIVE_RUN_STATE_COMMIT_DIRECTORY) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_state_commit",
        ));
    }
    Ok(canonical)
}

fn open_live_run_state_commit_directory(
    state: &DashboardServerState,
    create: bool,
) -> Result<cap_std::fs::Dir, ProductError> {
    open_absolute_directory_nofollow(&canonical_live_run_state_commit_root(state, create)?)
}

fn acquire_live_run_mutation_lock(
    state: &DashboardServerState,
) -> Result<LiveRunMutationLock, ProductError> {
    let live_run_root = canonical_live_run_root(state, true)?;
    let artifact_root_path = live_run_root
        .parent()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "live_run_mutation_lock"))?;
    let artifact_root = open_absolute_directory_nofollow(artifact_root_path)?;
    let owner = format!("pid={}\n", std::process::id());
    write_new_run_file(
        &artifact_root,
        LIVE_RUN_MUTATION_LOCK_FILE,
        owner.as_bytes(),
    )
    .map_err(|error| {
        if error.kind == ProductErrorKind::Conflict {
            product_error(ProductErrorKind::LiveConflict, "live_run_mutation_lock")
        } else {
            error
        }
    })?;
    Ok(LiveRunMutationLock { artifact_root })
}

fn read_optional_artifact_with_raw<T>(
    path: &Path,
    field: &str,
) -> Result<Option<(T, Vec<u8>)>, ProductError>
where
    T: serde::de::DeserializeOwned,
{
    if !path.exists() {
        return Ok(None);
    }
    let raw = read_live_run_artifact_bytes(path, field)?;
    serde_json::from_slice(&raw)
        .map(|value| Some((value, raw)))
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, field))
}

fn read_live_run_artifact_bytes(path: &Path, field: &str) -> Result<Vec<u8>, ProductError> {
    let parent = path
        .parent()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, field))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, field))?;
    let directory = open_absolute_directory_nofollow(parent)?;
    read_run_file_from_directory(&directory, file_name, field)
}

fn read_run_file_from_directory(
    directory: &cap_std::fs::Dir,
    file_name: impl AsRef<Path>,
    field: &str,
) -> Result<Vec<u8>, ProductError> {
    use cap_std::fs::OpenOptions;

    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    let file = directory
        .open_with(file_name.as_ref(), &options)
        .map_err(|error| {
            if error.kind() == ErrorKind::NotFound {
                product_error(ProductErrorKind::SourceUnavailable, field)
            } else {
                product_error(ProductErrorKind::SourceInvalid, field)
            }
        })?;
    let metadata = file
        .metadata()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, field))?;
    if !metadata.is_file() || metadata.len() > LIVE_RUN_ARTIFACT_MAX_BYTES {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    let mut raw = Vec::with_capacity(metadata.len() as usize);
    file.into_std()
        .take(LIVE_RUN_ARTIFACT_MAX_BYTES + 1)
        .read_to_end(&mut raw)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, field))?;
    if raw.len() as u64 > LIVE_RUN_ARTIFACT_MAX_BYTES {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    Ok(raw)
}

fn sha256_ref(bytes: &[u8]) -> String {
    let hash = digest(&SHA256, bytes);
    let mut value = String::with_capacity(71);
    value.push_str("sha256:");
    for byte in hash.as_ref() {
        value.push_str(&format!("{byte:02x}"));
    }
    value
}

fn sorted_refs(mut refs: Vec<String>) -> Vec<String> {
    refs.sort();
    refs.dedup();
    refs
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use nautilus_live::status::{
        ExecutionStatus, NodeStatus, ProcessMode, RiskTradingState, SnapshotValue,
    };

    use super::*;

    struct LiveRunFixture {
        root: PathBuf,
        state: DashboardServerState,
    }

    impl LiveRunFixture {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "ntpro-s3-lv-004-{name}-{}-{}",
                std::process::id(),
                unix_time_ms()
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(root.join("supervisor")).unwrap();
            fs::create_dir_all(root.join("artifacts")).unwrap();
            Self {
                state: DashboardServerState {
                    registry_path: root.join("supervisor/registry.json"),
                    workflow_root: None,
                    ntpro_node_bin: PathBuf::from("missing-ntpro-node"),
                    lifecycle_action_lock: std::sync::Arc::new(std::sync::Mutex::new(())),
                    backtest_creation_gate: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
                    live_run_audit_anchor: std::sync::Arc::new(
                        super::live_run_anchor::LiveRunAuditAnchorClient::memory_for_test(),
                    ),
                },
                root,
            }
        }

        fn independent_state(&self) -> DashboardServerState {
            DashboardServerState {
                registry_path: self.state.registry_path.clone(),
                workflow_root: None,
                ntpro_node_bin: PathBuf::from("missing-ntpro-node"),
                lifecycle_action_lock: std::sync::Arc::new(std::sync::Mutex::new(())),
                backtest_creation_gate: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
                live_run_audit_anchor: self.state.live_run_audit_anchor.clone(),
            }
        }

        #[cfg(unix)]
        fn install_live_market_data_node(&mut self, run_id: &str) {
            let canonical_root = fs::canonicalize(&self.root).unwrap();
            let runtime_root = canonical_root
                .join("artifacts/live-market-data-runtime")
                .join(run_id);
            let config_path = canonical_root
                .join("artifacts/live-runs")
                .join(run_id)
                .join(LIVE_MARKET_DATA_NODE_CONFIG_FILE);
            let mut running = NodeStatus::unknown(run_id);
            running.process_mode = ProcessMode::SpawnedProcess;
            running.config_path = SnapshotValue::available(config_path.display().to_string());
            running.artifact_root = SnapshotValue::available(runtime_root.display().to_string());
            running.lifecycle_state = LifecycleStatus::Running;
            running.previous_lifecycle_state = LifecycleStatus::Starting;
            running.data_connection = ConnectionStatus::Connected;
            running.execution_connection = ConnectionStatus::NotConfigured;
            running.execution = ExecutionStatus {
                gateway_id: SnapshotValue::not_configured(),
                connection: ConnectionStatus::NotConfigured,
                started: SnapshotValue::available(false),
                account_ref: SnapshotValue::not_configured(),
                orders_open: SnapshotValue::available(0),
                orders_inflight: SnapshotValue::available(0),
                orders_closed: SnapshotValue::available(0),
                last_report_at: SnapshotValue::not_configured(),
                last_reconciliation_at: SnapshotValue::not_configured(),
                last_error: None,
            };
            running.risk.trading_state = RiskTradingState::Halted;
            running.external_venue_connection = true;
            running.real_orders_submitted = false;
            let mut stopped = running.clone();
            stopped.lifecycle_state = LifecycleStatus::Stopped;
            stopped.previous_lifecycle_state = LifecycleStatus::Running;
            stopped.data_connection = ConnectionStatus::Disconnected;
            stopped.external_venue_connection = false;
            fs::write(
                self.root.join("live-market-data-running.json"),
                serde_json::to_vec_pretty(&running).unwrap(),
            )
            .unwrap();
            fs::write(
                self.root.join("live-market-data-stopped.json"),
                serde_json::to_vec_pretty(&stopped).unwrap(),
            )
            .unwrap();
            let path = self.root.join("fixture-live-market-data-node.sh");
            fs::write(
                &path,
                r#"#!/bin/sh
set -eu
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
output=""
stop_file=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output) output="$2"; shift 2 ;;
    --stop-file) stop_file="$2"; shift 2 ;;
    --config|--run-id|--max-runtime-ms|--heartbeat-interval-ms|--parent-pid|--shutdown-timeout-ms) shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$output/logs"
cp "$script_dir/live-market-data-running.json" "$output/status.json"
printf '%s\n' 'phase=start status=ok environment=live data_connection=connected execution_connection=not_configured order_endpoint_access=false real_orders_submitted=false' > "$output/logs/events.log"
while [ ! -f "$stop_file" ]; do sleep 1; done
cp "$script_dir/live-market-data-stopped.json" "$output/status.json"
printf '%s\n' 'phase=stop status=ok real_orders_submitted=false' >> "$output/logs/events.log"
"#,
            )
            .unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            self.state.ntpro_node_bin = path;
        }
    }

    impl Drop for LiveRunFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn request() -> CreateLiveRunCandidateRequest {
        CreateLiveRunCandidateRequest {
            strategy_id: "ema-cross".to_string(),
            strategy_version_id: "ema-cross@v1".to_string(),
            environment: "live".to_string(),
            account_ref: "account://live/binance/primary".to_string(),
            venue_ref: "venue://live/BINANCE".to_string(),
            user_confirmed: true,
        }
    }

    fn admission() -> LiveRunCreationAdmission {
        LiveRunCreationAdmission {
            account_ref: "account://live/binance/primary".to_string(),
            venue_id: "BINANCE".to_string(),
            ready: true,
            risk_ready: true,
            source_refs: vec![
                "node-config:node.toml#live_admission".to_string(),
                "node-config:node.toml#risk".to_string(),
                "risk-config-sha256:8a92f596c7f51574c25979022b59358cfd6807ec3470ef7b21301fb133d4c1ac"
                    .to_string(),
                VERSION_HASH.to_string(),
            ],
        }
    }

    fn gates() -> LiveRunGateState {
        LiveRunGateState {
            candidate_create: true,
            owner_approved: true,
            no_order_send: true,
            manual_stop: true,
            risk_approved: true,
        }
    }

    const VERSION_HASH: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn live_run_candidate_requires_every_independent_gate() {
        for missing in 0..5 {
            let mut values = [true; 5];
            values[missing] = false;
            let mut index = 0;
            let state = LiveRunGateState::from_reader(|_| {
                let value = values[index];
                index += 1;
                value
            });
            assert!(!state.all_open());
        }
        assert!(gates().all_open());
    }

    #[test]
    fn live_runtime_failure_transition_accepts_pre_start_and_post_start_cleanup() {
        let state = |revision, lifecycle| LiveRunCandidateState {
            schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
            run_id: "live-candidate-transition".to_string(),
            source_manifest_sha256: VERSION_HASH.to_string(),
            revision,
            previous_state_sha256: (revision > 0).then(|| VERSION_HASH.to_string()),
            lifecycle,
            preflight_sha256: Some(VERSION_HASH.to_string()),
            stop_sha256: None,
            updated_at_unix_ms: revision + 1,
        };
        assert!(
            validate_live_run_state_transition(
                &state(2, LiveRunCandidateLifecycle::Starting),
                &state(3, LiveRunCandidateLifecycle::Failed),
            )
            .is_ok()
        );
        assert!(
            validate_live_run_state_transition(
                &state(3, LiveRunCandidateLifecycle::MarketDataRunning),
                &state(4, LiveRunCandidateLifecycle::Failed),
            )
            .is_ok()
        );
    }

    #[test]
    fn live_run_candidate_moves_created_preflight_ready_stopped_without_order_send() {
        let fixture = LiveRunFixture::new("lifecycle");
        let created = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000001-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        assert_eq!(created.lifecycle, LiveRunCandidateLifecycle::Created);
        assert!(!created.runtime_started);

        let source_drift = run_live_candidate_action_with_preflight(
            &fixture.state,
            &created.run_id,
            &LiveRunCandidateActionRequest {
                run_id: created.run_id.clone(),
                action: LiveRunCandidateAction::Preflight,
                user_confirmed: true,
            },
            |_| {
                Ok(LiveRunPreflightAdmission {
                    connected: true,
                    can_trade: true,
                    evaluated_at_unix_ms: 101,
                    source_refs: vec!["node-config:other.toml#live_admission".to_string()],
                })
            },
        )
        .unwrap_err();
        assert_eq!(source_drift.kind, ProductErrorKind::BoundaryViolation);
        assert_eq!(source_drift.field, "live_candidate_source_refs");

        let ready = run_live_candidate_action_with_preflight(
            &fixture.state,
            &created.run_id,
            &LiveRunCandidateActionRequest {
                run_id: created.run_id.clone(),
                action: LiveRunCandidateAction::Preflight,
                user_confirmed: true,
            },
            |_| {
                Ok(LiveRunPreflightAdmission {
                    connected: true,
                    can_trade: true,
                    evaluated_at_unix_ms: 101,
                    source_refs: admission().source_refs,
                })
            },
        )
        .unwrap();
        assert_eq!(ready.lifecycle, LiveRunCandidateLifecycle::PreflightReady);
        assert!(ready.account_connected);
        assert!(ready.account_can_trade_verified);
        assert_eq!(ready.order_admission.status, "blocked");
        assert!(!ready.runtime_started);

        let stopped = run_live_candidate_action(
            &fixture.state,
            &created.run_id,
            &LiveRunCandidateActionRequest {
                run_id: created.run_id.clone(),
                action: LiveRunCandidateAction::Stop,
                user_confirmed: true,
            },
        )
        .unwrap();
        assert_eq!(stopped.lifecycle, LiveRunCandidateLifecycle::Stopped);
        assert!(!stopped.runtime_started);
    }

    fn create_preflight_ready_candidate(
        fixture: &LiveRunFixture,
        run_id: &str,
    ) -> LiveRunCandidate {
        let created = create_live_run_candidate(
            &fixture.state,
            request(),
            run_id,
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        run_live_candidate_action_with_preflight(
            &fixture.state,
            &created.run_id,
            &LiveRunCandidateActionRequest {
                run_id: created.run_id.clone(),
                action: LiveRunCandidateAction::Preflight,
                user_confirmed: true,
            },
            |_| {
                Ok(LiveRunPreflightAdmission {
                    connected: true,
                    can_trade: true,
                    evaluated_at_unix_ms: 101,
                    source_refs: admission().source_refs,
                })
            },
        )
        .unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn live_market_data_runtime_starts_and_stops_without_execution_capability() {
        let mut fixture = LiveRunFixture::new("market-data-runtime");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000002");
        let run_id = ready.run_id.clone();
        fixture.install_live_market_data_node(&run_id);

        let running = run_live_candidate_action(
            &fixture.state,
            &run_id,
            &LiveRunCandidateActionRequest {
                run_id: run_id.clone(),
                action: LiveRunCandidateAction::StartMarketData,
                user_confirmed: true,
            },
        )
        .unwrap();
        assert_eq!(ready.lifecycle, LiveRunCandidateLifecycle::PreflightReady);
        assert_eq!(
            running.lifecycle,
            LiveRunCandidateLifecycle::MarketDataRunning
        );
        assert!(running.runtime_started, "{running:?}");
        assert!(running.market_data_connected, "{running:?}");
        assert_eq!(running.runtime_node_id.as_deref(), Some(run_id.as_str()));
        assert_eq!(running.runtime_process_state, "running");
        assert_eq!(running.runtime_error, None);
        assert_eq!(
            running.order_admission,
            LiveOrderAdmissionSnapshot::blocked()
        );

        let config = fs::read_to_string(
            fixture
                .root
                .join("artifacts/live-runs")
                .join(&run_id)
                .join(LIVE_MARKET_DATA_NODE_CONFIG_FILE),
        )
        .unwrap();
        assert!(config.contains("environment = \"live\""));
        assert!(config.contains("execution_client_enabled = false"));
        assert!(config.contains("order_endpoint_access_allowed = false"));
        assert!(config.contains("order_submission_allowed = false"));
        assert!(config.contains("automatic_reconnect_allowed = false"));
        assert!(!config.contains("api_key ="));
        assert!(!config.contains("api_secret ="));

        let stopped = run_live_candidate_action(
            &fixture.state,
            &run_id,
            &LiveRunCandidateActionRequest {
                run_id: run_id.clone(),
                action: LiveRunCandidateAction::Stop,
                user_confirmed: true,
            },
        )
        .unwrap();
        assert_eq!(stopped.lifecycle, LiveRunCandidateLifecycle::Stopped);
        assert!(!stopped.runtime_started);
        assert!(!stopped.market_data_connected);
        assert!(
            load_active_live_run_candidates(&fixture.state)
                .unwrap()
                .is_empty()
        );
        let registry = SupervisorRegistryStore::new(&fixture.state.registry_path)
            .load()
            .unwrap();
        let ownership = registry.nodes[&run_id].run_ownership.get(&run_id).unwrap();
        assert_eq!(
            ownership
                .terminal
                .as_ref()
                .map(|value| value.lifecycle.as_str()),
            Some("stopped")
        );
    }

    #[test]
    fn live_market_data_runtime_start_failure_is_anchored_and_releases_active_candidate() {
        let fixture = LiveRunFixture::new("market-data-runtime-failure");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000003");
        let run_id = ready.run_id;

        let error = run_live_candidate_action(
            &fixture.state,
            &run_id,
            &LiveRunCandidateActionRequest {
                run_id: run_id.clone(),
                action: LiveRunCandidateAction::StartMarketData,
                user_confirmed: true,
            },
        )
        .unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::LiveExecutionFailed);
        assert_eq!(error.field, "live_runtime_start");
        let failed = load_live_run_candidate(&fixture.state, &run_id).unwrap();
        assert_eq!(failed.lifecycle, LiveRunCandidateLifecycle::Failed);
        assert!(!failed.runtime_started);
        assert!(!failed.market_data_connected);
        assert!(
            load_active_live_run_candidates(&fixture.state)
                .unwrap()
                .is_empty()
        );
        let registry = SupervisorRegistryStore::new(&fixture.state.registry_path)
            .load()
            .unwrap();
        let ownership = registry.nodes[&run_id].run_ownership.get(&run_id).unwrap();
        assert_eq!(
            ownership
                .terminal
                .as_ref()
                .map(|value| value.lifecycle.as_str()),
            Some("failed")
        );
    }

    #[test]
    fn live_run_candidate_fails_closed_for_admission_identity_and_trade_state() {
        let fixture = LiveRunFixture::new("boundaries");
        let mut blocked = admission();
        blocked.ready = false;
        assert_eq!(
            create_live_run_candidate(
                &fixture.state,
                request(),
                "product-0000000000000002-0000000000000001",
                100,
                blocked,
                gates(),
                VERSION_HASH,
            )
            .unwrap_err()
            .kind,
            ProductErrorKind::LiveExecutionFailed
        );

        let mut risk_blocked = admission();
        risk_blocked.risk_ready = false;
        assert_eq!(
            create_live_run_candidate(
                &fixture.state,
                request(),
                "product-0000000000000002-0000000000000002",
                100,
                risk_blocked,
                gates(),
                VERSION_HASH,
            )
            .unwrap_err()
            .kind,
            ProductErrorKind::LiveExecutionFailed
        );

        let mut wrong_identity = request();
        wrong_identity.account_ref = "account://live/binance/other".to_string();
        assert_eq!(
            create_live_run_candidate(
                &fixture.state,
                wrong_identity,
                "product-0000000000000003-0000000000000001",
                100,
                admission(),
                gates(),
                VERSION_HASH,
            )
            .unwrap_err()
            .kind,
            ProductErrorKind::BadRequest
        );

        let created = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000004-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        let (_, raw) = load_live_run_manifest(&fixture.state, &created.run_id).unwrap();
        let directory = open_absolute_directory_nofollow(
            &canonical_live_run_root(&fixture.state, false)
                .unwrap()
                .join(&created.run_id),
        )
        .unwrap();
        assert_eq!(
            write_preflight_artifact(
                &directory,
                &created.run_id,
                &sha256_ref(&raw),
                &LiveRunPreflightAdmission {
                    connected: true,
                    can_trade: false,
                    evaluated_at_unix_ms: 101,
                    source_refs: vec!["source".to_string()],
                },
            )
            .unwrap_err()
            .kind,
            ProductErrorKind::LiveExecutionFailed
        );
        assert!(
            !fixture
                .root
                .join(format!(
                    "artifacts/live-runs/{}/preflight.json",
                    created.run_id
                ))
                .exists()
        );
    }

    #[test]
    fn live_run_candidate_rejects_mutating_or_tampered_artifacts() {
        let fixture = LiveRunFixture::new("tamper");
        let created = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000005-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        let (_, raw) = load_live_run_manifest(&fixture.state, &created.run_id).unwrap();
        let artifact = LiveRunPreflightArtifact {
            schema_version: LIVE_RUN_PREFLIGHT_SCHEMA_VERSION.to_string(),
            run_id: created.run_id.clone(),
            source_manifest_sha256: sha256_ref(&raw),
            evaluated_at_unix_ms: 101,
            account_connected: true,
            account_can_trade_verified: true,
            runtime_gates_verified: true,
            order_endpoint_access_attempted: true,
            execution_adapter_send_attempted: false,
            real_orders_submitted: false,
            source_refs: vec!["source".to_string()],
        };
        fs::write(
            fixture.root.join(format!(
                "artifacts/live-runs/{}/preflight.json",
                created.run_id
            )),
            serde_json::to_vec_pretty(&artifact).unwrap(),
        )
        .unwrap();
        assert_eq!(
            load_live_run_candidate(&fixture.state, &created.run_id)
                .unwrap_err()
                .kind,
            ProductErrorKind::BoundaryViolation
        );
    }

    #[test]
    fn live_run_candidate_allows_a_new_candidate_only_after_manual_stop() {
        let fixture = LiveRunFixture::new("single-active");
        let first = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000006-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        assert_eq!(
            create_live_run_candidate(
                &fixture.state,
                request(),
                "product-0000000000000007-0000000000000001",
                101,
                admission(),
                gates(),
                VERSION_HASH,
            )
            .unwrap_err()
            .kind,
            ProductErrorKind::LiveConflict
        );
        run_live_candidate_action(
            &fixture.state,
            &first.run_id,
            &LiveRunCandidateActionRequest {
                run_id: first.run_id.clone(),
                action: LiveRunCandidateAction::Stop,
                user_confirmed: true,
            },
        )
        .unwrap();
        let second = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000008-0000000000000001",
            102,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        assert_eq!(second.lifecycle, LiveRunCandidateLifecycle::Created);
    }

    #[test]
    fn live_run_candidate_rejects_deleted_or_rolled_back_lifecycle_artifacts() {
        let fixture = LiveRunFixture::new("state-rollback");
        let created = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000009-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        let created_state_path = fixture.root.join(format!(
            "artifacts/live-runs/{}/{}",
            created.run_id,
            live_run_state_file_name(0)
        ));
        let created_state = fs::read(&created_state_path).unwrap();
        run_live_candidate_action_with_preflight(
            &fixture.state,
            &created.run_id,
            &LiveRunCandidateActionRequest {
                run_id: created.run_id.clone(),
                action: LiveRunCandidateAction::Preflight,
                user_confirmed: true,
            },
            |_| {
                Ok(LiveRunPreflightAdmission {
                    connected: true,
                    can_trade: true,
                    evaluated_at_unix_ms: 101,
                    source_refs: admission().source_refs,
                })
            },
        )
        .unwrap();

        let preflight_state_path = fixture.root.join(format!(
            "artifacts/live-runs/{}/{}",
            created.run_id,
            live_run_state_file_name(1)
        ));
        let preflight_state = fs::read(&preflight_state_path).unwrap();
        let preflight_commit_path = fixture
            .root
            .join("artifacts")
            .join(LIVE_RUN_STATE_COMMIT_DIRECTORY)
            .join(live_run_state_commit_file_name(&created.run_id, 1));
        let preflight_commit = fs::read(&preflight_commit_path).unwrap();
        let state_head_path = fixture.root.join(format!(
            "artifacts/live-runs/{}/{}",
            created.run_id, LIVE_RUN_STATE_HEAD_FILE
        ));
        let state_head = fs::read(&state_head_path).unwrap();

        let preflight_path = fixture.root.join(format!(
            "artifacts/live-runs/{}/preflight.json",
            created.run_id
        ));
        let preflight_raw = fs::read(&preflight_path).unwrap();

        fs::remove_file(&preflight_path).unwrap();
        fs::remove_file(&preflight_state_path).unwrap();
        fs::remove_file(&preflight_commit_path).unwrap();
        assert!(load_live_run_candidate(&fixture.state, &created.run_id).is_err());
        fs::write(&preflight_path, &preflight_raw).unwrap();
        fs::write(&preflight_state_path, &preflight_state).unwrap();
        fs::write(&preflight_commit_path, &preflight_commit).unwrap();
        fs::write(&state_head_path, &state_head).unwrap();

        fs::remove_file(&state_head_path).unwrap();
        assert!(load_live_run_candidate(&fixture.state, &created.run_id).is_err());
        fs::write(&state_head_path, &state_head).unwrap();

        fs::remove_file(&preflight_path).unwrap();
        assert!(load_live_run_candidate(&fixture.state, &created.run_id).is_err());

        fs::write(&preflight_path, &preflight_raw).unwrap();
        fs::remove_file(&preflight_state_path).unwrap();
        assert!(load_live_run_candidate(&fixture.state, &created.run_id).is_err());

        fs::write(&preflight_state_path, &created_state).unwrap();
        assert!(load_live_run_candidate(&fixture.state, &created.run_id).is_err());

        fs::write(&preflight_state_path, &preflight_state).unwrap();
        fs::remove_file(&preflight_commit_path).unwrap();
        assert!(load_live_run_candidate(&fixture.state, &created.run_id).is_err());
    }

    #[test]
    fn live_run_candidate_rejects_missing_active_pointer_and_oversized_artifact() {
        let fixture = LiveRunFixture::new("pointer-and-size");
        let created = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000010-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        fs::remove_file(
            fixture
                .root
                .join("artifacts/live-runs")
                .join(LIVE_RUN_ACTIVE_FILE),
        )
        .unwrap();
        assert_eq!(
            load_active_live_run_candidates(&fixture.state)
                .unwrap_err()
                .kind,
            ProductErrorKind::SourceInvalid
        );

        let state_path = fixture.root.join(format!(
            "artifacts/live-runs/{}/{}",
            created.run_id,
            live_run_state_file_name(0)
        ));
        fs::write(
            state_path,
            vec![b' '; (LIVE_RUN_ARTIFACT_MAX_BYTES + 1) as usize],
        )
        .unwrap();
        assert_eq!(
            load_live_run_candidate(&fixture.state, &created.run_id)
                .unwrap_err()
                .kind,
            ProductErrorKind::SourceInvalid
        );
    }

    #[test]
    fn live_run_candidate_single_active_claim_is_atomic_across_server_states() {
        let fixture = LiveRunFixture::new("cross-process-claim");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let results = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (index, state) in [fixture.independent_state(), fixture.independent_state()]
                .into_iter()
                .enumerate()
            {
                let barrier = barrier.clone();
                handles.push(scope.spawn(move || {
                    barrier.wait();
                    create_live_run_candidate(
                        &state,
                        request(),
                        &format!("product-0000000000000011-000000000000000{index}"),
                        100 + index as u64,
                        admission(),
                        gates(),
                        VERSION_HASH,
                    )
                }));
            }
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| result
                    .as_ref()
                    .is_err_and(|error| { error.kind == ProductErrorKind::LiveConflict }))
                .count(),
            1
        );
        assert_eq!(
            load_active_live_run_candidates(&fixture.state)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn stale_active_claim_replacement_is_atomic_across_server_states() {
        let fixture = LiveRunFixture::new("cross-process-stale-claim");
        let stopped = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000013-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        let (_, manifest_raw) = load_live_run_manifest(&fixture.state, &stopped.run_id).unwrap();
        run_live_candidate_action(
            &fixture.state,
            &stopped.run_id,
            &LiveRunCandidateActionRequest {
                run_id: stopped.run_id.clone(),
                action: LiveRunCandidateAction::Stop,
                user_confirmed: true,
            },
        )
        .unwrap();
        claim_active_live_run_candidate(
            &fixture.state,
            &stopped.run_id,
            &sha256_ref(&manifest_raw),
            102,
        )
        .unwrap();

        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let results = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (index, state) in [fixture.independent_state(), fixture.independent_state()]
                .into_iter()
                .enumerate()
            {
                let barrier = barrier.clone();
                handles.push(scope.spawn(move || {
                    barrier.wait();
                    create_live_run_candidate(
                        &state,
                        request(),
                        &format!("product-0000000000000014-000000000000000{index}"),
                        103 + index as u64,
                        admission(),
                        gates(),
                        VERSION_HASH,
                    )
                }));
            }
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| result
                    .as_ref()
                    .is_err_and(|error| { error.kind == ProductErrorKind::LiveConflict }))
                .count(),
            1
        );
        assert_eq!(
            load_active_live_run_candidates(&fixture.state)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn stale_mutation_lock_fails_closed_without_publishing_candidate() {
        let fixture = LiveRunFixture::new("stale-mutation-lock");
        let lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        let error = create_live_run_candidate(
            &fixture.independent_state(),
            request(),
            "product-0000000000000015-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::LiveConflict);
        assert!(
            fs::read_dir(fixture.root.join("artifacts/live-runs"))
                .unwrap()
                .next()
                .is_none()
        );
        drop(lock);
        assert!(
            create_live_run_candidate(
                &fixture.independent_state(),
                request(),
                "product-0000000000000015-0000000000000002",
                101,
                admission(),
                gates(),
                VERSION_HASH,
            )
            .is_ok()
        );
    }

    #[test]
    fn external_anchor_detects_complete_workspace_snapshot_rollback() {
        let fixture = LiveRunFixture::new("complete-workspace-rollback");
        let created = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-complete-workspace-rollback",
            unix_time_ms(),
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        run_live_candidate_action_with_preflight(
            &fixture.state,
            &created.run_id,
            &LiveRunCandidateActionRequest {
                run_id: created.run_id.clone(),
                action: LiveRunCandidateAction::Preflight,
                user_confirmed: true,
            },
            |_| {
                Ok(LiveRunPreflightAdmission {
                    connected: true,
                    can_trade: true,
                    evaluated_at_unix_ms: unix_time_ms(),
                    source_refs: admission().source_refs,
                })
            },
        )
        .unwrap();

        let run_root = fixture
            .root
            .join("artifacts/live-runs")
            .join(&created.run_id);
        let snapshot_files = fs::read_dir(&run_root)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name(), fs::read(entry.path()).unwrap())
            })
            .collect::<Vec<_>>();
        let commit_root = fixture.root.join("artifacts/live-run-state-commits");
        let snapshot_commits = fs::read_dir(&commit_root)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name(), fs::read(entry.path()).unwrap())
            })
            .collect::<Vec<_>>();
        let workspace_head_path = fixture
            .root
            .join("artifacts/live-run-audit-anchor-head.json");
        let workspace_head = fs::read(&workspace_head_path).unwrap();
        let active_pointer = fs::read(
            fixture
                .root
                .join("artifacts/live-runs/.active-candidate.json"),
        )
        .unwrap();

        run_live_candidate_action(
            &fixture.state,
            &created.run_id,
            &LiveRunCandidateActionRequest {
                run_id: created.run_id.clone(),
                action: LiveRunCandidateAction::Stop,
                user_confirmed: true,
            },
        )
        .unwrap();

        fs::remove_dir_all(&run_root).unwrap();
        fs::create_dir(&run_root).unwrap();
        for (name, raw) in snapshot_files {
            fs::write(run_root.join(name), raw).unwrap();
        }
        for entry in fs::read_dir(&commit_root).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
        for (name, raw) in snapshot_commits {
            fs::write(commit_root.join(name), raw).unwrap();
        }
        fs::write(
            fixture
                .root
                .join("artifacts/live-runs/.active-candidate.json"),
            active_pointer,
        )
        .unwrap();
        fs::write(&workspace_head_path, workspace_head).unwrap();

        let error = load_live_run_candidate(&fixture.state, &created.run_id).unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
        assert_eq!(error.field, "live_run_workspace_anchor_latest");
    }

    #[test]
    fn external_anchor_detects_rollback_to_empty_precreation_workspace() {
        let fixture = LiveRunFixture::new("empty-workspace-rollback");
        create_live_run_candidate(
            &fixture.state,
            request(),
            "product-empty-workspace-rollback",
            unix_time_ms(),
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();

        fs::remove_dir_all(fixture.root.join("artifacts/live-runs")).unwrap();
        fs::remove_dir_all(fixture.root.join("artifacts/live-run-state-commits")).unwrap();
        fs::remove_file(
            fixture
                .root
                .join("artifacts/live-run-audit-anchor-head.json"),
        )
        .unwrap();

        let error = load_active_live_run_candidates(&fixture.state).unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
        assert_eq!(error.field, "live_run_workspace_anchor_latest");
    }

    #[test]
    fn external_anchor_detects_append_success_before_local_publication() {
        let fixture = LiveRunFixture::new("append-before-local-publication");
        let anchor_request = LiveRunAnchorAppendRequest::new(
            fixture.state.live_run_audit_anchor.namespace().unwrap(),
            "live-candidate-orphaned-anchor",
            LiveRunAnchorRevision::new(0, 0),
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
            None,
            unix_time_ms(),
        );
        fixture
            .state
            .live_run_audit_anchor
            .append(&anchor_request)
            .unwrap();

        let error = load_active_live_run_candidates(&fixture.state).unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
        assert_eq!(error.field, "live_run_workspace_anchor_latest");
        let create_error = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-after-orphaned-anchor",
            unix_time_ms(),
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap_err();
        assert_eq!(create_error.kind, ProductErrorKind::BoundaryViolation);
        assert!(
            fs::read_dir(fixture.root.join("artifacts/live-runs"))
                .unwrap()
                .all(|entry| !entry.unwrap().file_type().unwrap().is_dir())
        );
    }

    #[test]
    fn unconfigured_anchor_fails_before_candidate_publication() {
        let fixture = LiveRunFixture::new("unconfigured-anchor");
        let mut state = fixture.independent_state();
        state.live_run_audit_anchor =
            std::sync::Arc::new(super::live_run_anchor::LiveRunAuditAnchorClient::Unconfigured);
        let error = create_live_run_candidate(
            &state,
            request(),
            "product-unconfigured-anchor",
            unix_time_ms(),
            admission(),
            gates(),
            &format!("sha256:{}", "1".repeat(64)),
        )
        .unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::LiveExecutionFailed);
        assert_eq!(error.field, "live_run_audit_anchor_config");
        assert!(
            fs::read_dir(fixture.root.join("artifacts/live-runs"))
                .unwrap()
                .all(|entry| !entry.unwrap().file_type().unwrap().is_dir())
        );
        assert!(
            !fixture
                .root
                .join("artifacts/live-run-audit-anchor-head.json")
                .exists()
        );
    }

    #[test]
    fn live_run_candidate_actions_are_atomic_across_server_states() {
        let fixture = LiveRunFixture::new("cross-process-action");
        let created = create_live_run_candidate(
            &fixture.state,
            request(),
            "product-0000000000000012-0000000000000001",
            100,
            admission(),
            gates(),
            VERSION_HASH,
        )
        .unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let run_id = created.run_id;
        let results = std::thread::scope(|scope| {
            let preflight_state = fixture.independent_state();
            let preflight_barrier = barrier.clone();
            let preflight_run_id = run_id.clone();
            let preflight = scope.spawn(move || {
                preflight_barrier.wait();
                run_live_candidate_action_with_preflight(
                    &preflight_state,
                    &preflight_run_id,
                    &LiveRunCandidateActionRequest {
                        run_id: preflight_run_id.clone(),
                        action: LiveRunCandidateAction::Preflight,
                        user_confirmed: true,
                    },
                    |_| {
                        Ok(LiveRunPreflightAdmission {
                            connected: true,
                            can_trade: true,
                            evaluated_at_unix_ms: 101,
                            source_refs: admission().source_refs,
                        })
                    },
                )
            });
            let stop_state = fixture.independent_state();
            let stop_barrier = barrier.clone();
            let stop_run_id = run_id.clone();
            let stop = scope.spawn(move || {
                stop_barrier.wait();
                run_live_candidate_action(
                    &stop_state,
                    &stop_run_id,
                    &LiveRunCandidateActionRequest {
                        run_id: stop_run_id.clone(),
                        action: LiveRunCandidateAction::Stop,
                        user_confirmed: true,
                    },
                )
            });
            vec![preflight.join().unwrap(), stop.join().unwrap()]
        });
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| result
                    .as_ref()
                    .is_err_and(|error| { error.kind == ProductErrorKind::LiveConflict }))
                .count(),
            1
        );
        let candidate = load_live_run_candidate(&fixture.state, &run_id).unwrap();
        assert!(matches!(
            candidate.lifecycle,
            LiveRunCandidateLifecycle::PreflightReady | LiveRunCandidateLifecycle::Stopped
        ));
    }
}
