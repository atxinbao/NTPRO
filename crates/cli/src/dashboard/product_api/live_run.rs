//! Live Run 候选、启动前检查与人工停止产品合同。

use std::{
    collections::BTreeSet,
    fs,
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
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
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    dashboard::ApiStatusResult,
    process::process_start_time_secs,
    supervisor::{
        RegisterNodeRequest, StartNodeRequest, StopNodeRequest, SupervisorProcessState,
        SupervisorRegistryStore, SupervisorRunOwnership, SupervisorRunTerminalAnchor,
    },
};

use super::{
    live_admission::{
        LiveExecutionRiskPolicy, LiveRunCreationAdmission, LiveRunPreflightAdmission,
        LiveSizingPreflight, evaluate_live_execution_risk_policy,
        evaluate_live_run_creation_admission, evaluate_live_run_preflight_admission,
    },
    live_run_anchor::{
        LIVE_EXECUTION_RUNTIME_CLAIM_FILE, LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE,
        LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION, LiveExecutionRuntimeClaimArtifact,
        LiveRunAnchorAppendRequest, LiveRunAnchorReceipt, LiveRunAnchorRevision,
        anchor_config_refs,
    },
    run::{
        PromotableStrategyOrderIntent, load_promotable_demo_order_intent,
        open_absolute_directory_nofollow, publish_new_run_file, write_new_run_file,
    },
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
const LIVE_RUN_PREFLIGHT_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_preflight.v2";
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
const LIVE_RUN_MUTATION_LOCK_SCHEMA_VERSION: &str = "ntpro.live_run_mutation_lock.v1";
const LIVE_RUN_STATE_COMMIT_DIRECTORY: &str = "live-run-state-commits";
const LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE: &str = "live-run-audit-anchor-head.json";
const LIVE_RUN_WORKSPACE_ANCHOR_HEAD_NEXT_FILE: &str = ".live-run-audit-anchor-head.next.json";
const LIVE_RUN_ARTIFACT_MAX_BYTES: u64 = 64 * 1024;
const LIVE_RUN_STOP_CLOCK_SKEW_MS: u64 = 5_000;
const LIVE_MARKET_DATA_STARTUP_TIMEOUT_MS: u64 = 20_000;
const LIVE_EXECUTION_ADMISSION_SCHEMA_VERSION: &str =
    "ntpro.product_api.live_execution_admission.v2";
const LIVE_EXECUTION_ADMISSION_FILE: &str = "execution-admission.json";
const LIVE_STRATEGY_INTENT_SCHEMA_VERSION: &str = "ntpro.s3.live_strategy_order_intent.v1";
const LIVE_STRATEGY_INTENT_FILE: &str = "strategy-order-intent.json";
const LIVE_SIZING_DECISION_SCHEMA_VERSION: &str = "ntpro.s3.live_sizing_decision.v1";
const LIVE_SIZING_DECISION_FILE: &str = "live-sizing-decision.json";
const LIVE_EXECUTION_APPROVAL_SCHEMA_VERSION: &str = "ntpro.product_api.live_execution_approval.v2";
const LIVE_EXECUTION_OWNER_APPROVAL_FILE: &str = "execution-owner-approval.json";
const LIVE_EXECUTION_RISK_APPROVAL_FILE: &str = "execution-risk-approval.json";
const LIVE_EXECUTION_OPERATOR_APPROVAL_FILE: &str = "execution-operator-approval.json";
const LIVE_EXECUTION_OWNER_APPROVAL_RECEIPT_FILE: &str = "execution-owner-approval-receipt.json";
const LIVE_EXECUTION_RISK_APPROVAL_RECEIPT_FILE: &str = "execution-risk-approval-receipt.json";
const LIVE_EXECUTION_OPERATOR_APPROVAL_RECEIPT_FILE: &str =
    "execution-operator-approval-receipt.json";
const LIVE_EXECUTION_OWNER_APPROVAL_STAGE_FILE: &str = ".execution-owner-approval-publication.json";
const LIVE_EXECUTION_RISK_APPROVAL_STAGE_FILE: &str = ".execution-risk-approval-publication.json";
const LIVE_EXECUTION_OPERATOR_APPROVAL_STAGE_FILE: &str =
    ".execution-operator-approval-publication.json";
const LIVE_EXECUTION_ORDER_STATE_SCHEMA_VERSION: &str = "ntpro.s3.live_execution_order_state.v4";
const LIVE_EXECUTION_ORDER_STATE_FILE: &str = "execution-order-state.json";
const LIVE_EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION: &str =
    "ntpro.s3.live_execution_control_request.v1";
const LIVE_EXECUTION_CONTROL_RESULT_SCHEMA_VERSION: &str =
    "ntpro.s3.live_execution_control_result.v1";
const LIVE_EXECUTION_RECONCILE_REQUEST_FILE: &str = "execution-reconcile-request.json";
const LIVE_EXECUTION_RECONCILE_SOURCE_ORDER_FILE: &str =
    "execution-reconcile-source-order-state.json";
const LIVE_EXECUTION_RECONCILE_RECEIPT_FILE: &str = "execution-reconcile-request-receipt.json";
const LIVE_EXECUTION_RECONCILE_RESULT_FILE: &str = "execution-reconcile-result.json";
const LIVE_EXECUTION_RECONCILE_RESULT_RECEIPT_FILE: &str =
    "execution-reconcile-result-receipt.json";
const LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE: &str = "execution-cancel-owner-approval.json";
const LIVE_EXECUTION_CANCEL_OWNER_RECEIPT_FILE: &str =
    "execution-cancel-owner-approval-receipt.json";
const LIVE_EXECUTION_CANCEL_OPERATOR_APPROVAL_FILE: &str =
    "execution-cancel-operator-approval.json";
const LIVE_EXECUTION_CANCEL_OPERATOR_RECEIPT_FILE: &str =
    "execution-cancel-operator-approval-receipt.json";
const LIVE_EXECUTION_CANCEL_OWNER_STAGE_FILE: &str = ".execution-cancel-owner-publication.json";
const LIVE_EXECUTION_CANCEL_OPERATOR_STAGE_FILE: &str =
    ".execution-cancel-operator-publication.json";
const LIVE_EXECUTION_CANCEL_REQUEST_FILE: &str = "execution-cancel-request.json";
const LIVE_EXECUTION_CANCEL_SOURCE_ORDER_FILE: &str = "execution-cancel-source-order-state.json";
const LIVE_EXECUTION_CANCEL_RESULT_FILE: &str = "execution-cancel-result.json";
const LIVE_EXECUTION_CANCEL_RESULT_RECEIPT_FILE: &str = "execution-cancel-result-receipt.json";

const LIVE_RUN_GATE_CREATE: &str = "NTPRO_S3_LIVE_RUN_CANDIDATE_CREATE";
const LIVE_RUN_GATE_OWNER_APPROVED: &str = "NTPRO_S3_LIVE_RUN_OWNER_APPROVED";
const LIVE_RUN_GATE_NO_ORDER_SEND: &str = "NTPRO_S3_LIVE_RUN_NO_ORDER_SEND";
const LIVE_RUN_GATE_MANUAL_STOP: &str = "NTPRO_S3_LIVE_RUN_MANUAL_STOP";
const LIVE_RUN_GATE_RISK_APPROVED: &str = "NTPRO_S3_LIVE_RUN_RISK_APPROVED";
const LIVE_RUN_GATE_EXECUTION_SINGLE_SHOT: &str = "NTPRO_S3_LIVE_RUN_EXECUTION_SINGLE_SHOT";
const LIVE_RUN_GATE_ORDER_CONTROL: &str = "NTPRO_S3_LIVE_ORDER_CONTROL";

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
    StartExecution,
    ReconcileOrder,
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
pub(in crate::dashboard) struct LiveExecutionAdmissionRequest {
    run_id: String,
    strategy_version_id: String,
    account_ref: String,
    venue_ref: String,
    admission_id: String,
    source_demo_run_id: String,
    strategy_intent_id: String,
    instrument_id: String,
    side: String,
    order_type: String,
    time_in_force: String,
    price: String,
    quantity: String,
    max_notional: String,
    expires_at_unix_ms: u64,
    user_confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveStrategyOrderIntentArtifact {
    schema_version: String,
    source_demo_run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    intent_id: String,
    instrument_id: String,
    side: String,
    source_order_type: String,
    quantity: String,
    source_signal: String,
    confidence: String,
    market_event_seq: u64,
    created_at_unix_ms: u64,
    source_manifest_sha256: String,
    source_result_sha256: String,
}

impl From<PromotableStrategyOrderIntent> for LiveStrategyOrderIntentArtifact {
    fn from(value: PromotableStrategyOrderIntent) -> Self {
        Self {
            schema_version: LIVE_STRATEGY_INTENT_SCHEMA_VERSION.to_string(),
            source_demo_run_id: value.source_run_id,
            strategy_id: value.strategy_id,
            strategy_version_id: value.strategy_version_id,
            intent_id: value.intent_id,
            instrument_id: value.instrument_id,
            side: value.side,
            source_order_type: value.source_order_type,
            quantity: value.quantity,
            source_signal: value.source_signal,
            confidence: value.confidence,
            market_event_seq: value.market_event_seq,
            created_at_unix_ms: value.created_at_unix_ms,
            source_manifest_sha256: value.source_manifest_sha256,
            source_result_sha256: value.source_result_sha256,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::dashboard) struct LiveExecutionCancelRequest {
    run_id: String,
    request_id: String,
    client_order_id: String,
    source_order_state_sha256: String,
    expires_at_unix_ms: u64,
    user_confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionControlRequestArtifact {
    schema_version: String,
    request_id: String,
    action: String,
    run_id: String,
    admission_id: String,
    strategy_version_id: String,
    instrument_id: String,
    client_order_id: String,
    source_order_state_sha256: String,
    owner_confirmed: bool,
    operator_confirmed: bool,
    requested_at_unix_ms: u64,
    expires_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionCancelApprovalArtifact {
    schema_version: String,
    role: LiveExecutionApprovalRole,
    proposal_sha256: String,
    source_manifest_sha256: String,
    run_id: String,
    admission_id: String,
    strategy_version_id: String,
    instrument_id: String,
    client_order_id: String,
    source_order_state_sha256: String,
    authority_ref: String,
    approved_at_unix_ms: u64,
    expires_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionCancelPublicationStage {
    schema_version: String,
    role: LiveExecutionApprovalRole,
    request: LiveExecutionCancelRequest,
    source_order_raw: String,
    approval: LiveExecutionCancelApprovalArtifact,
    control_request: Option<LiveExecutionControlRequestArtifact>,
    run_revision: u64,
    manifest_sha256: String,
    previous_workspace_receipt: LiveRunAnchorReceipt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CancelPublicationStep {
    SourceOrder,
    ExternalAnchor,
    Approval,
    Receipt,
    WorkspaceHead,
    ControlRequest,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::dashboard) enum LiveExecutionApprovalRole {
    Owner,
    Risk,
    Operator,
}

impl LiveExecutionApprovalRole {
    const fn artifact_file(self) -> &'static str {
        match self {
            Self::Owner => LIVE_EXECUTION_OWNER_APPROVAL_FILE,
            Self::Risk => LIVE_EXECUTION_RISK_APPROVAL_FILE,
            Self::Operator => LIVE_EXECUTION_OPERATOR_APPROVAL_FILE,
        }
    }

    const fn receipt_file(self) -> &'static str {
        match self {
            Self::Owner => LIVE_EXECUTION_OWNER_APPROVAL_RECEIPT_FILE,
            Self::Risk => LIVE_EXECUTION_RISK_APPROVAL_RECEIPT_FILE,
            Self::Operator => LIVE_EXECUTION_OPERATOR_APPROVAL_RECEIPT_FILE,
        }
    }

    const fn publication_stage_file(self) -> &'static str {
        match self {
            Self::Owner => LIVE_EXECUTION_OWNER_APPROVAL_STAGE_FILE,
            Self::Risk => LIVE_EXECUTION_RISK_APPROVAL_STAGE_FILE,
            Self::Operator => LIVE_EXECUTION_OPERATOR_APPROVAL_STAGE_FILE,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionApprovalArtifact {
    schema_version: String,
    role: LiveExecutionApprovalRole,
    proposal_sha256: String,
    strategy_intent_sha256: String,
    sizing_decision_sha256: String,
    source_manifest_sha256: String,
    run_id: String,
    strategy_version_id: String,
    admission_id: String,
    authority_ref: String,
    risk_policy_ref: String,
    approved_at_unix_ms: u64,
    expires_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionApprovalPublicationStage {
    schema_version: String,
    role: LiveExecutionApprovalRole,
    request: LiveExecutionAdmissionRequest,
    strategy_intent: LiveStrategyOrderIntentArtifact,
    sizing_decision: LiveSizingDecisionArtifact,
    approval: LiveExecutionApprovalArtifact,
    admission: Option<LiveExecutionAdmissionArtifact>,
    admission_state: Option<LiveRunCandidateState>,
    base_state: LiveRunCandidateState,
    base_state_sha256: String,
    manifest_sha256: String,
    previous_workspace_receipt: LiveRunAnchorReceipt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ApprovalPublicationStep {
    ExternalAnchor,
    Approval,
    Receipt,
    WorkspaceHead,
    Admission,
    StateExternalAnchor,
    State,
    StateCommit,
    StateReceipt,
    StateHead,
    StateWorkspaceHead,
}

struct LiveExecutionApprovalRecord {
    role: LiveExecutionApprovalRole,
    artifact: LiveExecutionApprovalArtifact,
    artifact_raw: Vec<u8>,
    receipt: LiveRunAnchorReceipt,
}

struct LiveExecutionApprovalBinding<'a> {
    role: LiveExecutionApprovalRole,
    manifest_sha256: &'a str,
    run_id: &'a str,
    strategy_version_id: &'a str,
    admission_id: &'a str,
    proposal_sha256: &'a str,
    strategy_intent_sha256: &'a str,
    sizing_decision_sha256: &'a str,
    risk_policy: &'a LiveExecutionRiskPolicy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionAdmissionArtifact {
    schema_version: String,
    request_sha256: String,
    source_manifest_sha256: String,
    run_id: String,
    strategy_version_id: String,
    account_ref: String,
    venue_ref: String,
    admission_id: String,
    source_demo_run_id: String,
    strategy_intent_id: String,
    strategy_intent_sha256: String,
    sizing_decision_sha256: String,
    instrument_id: String,
    side: String,
    order_type: String,
    time_in_force: String,
    price: String,
    source_quantity: String,
    quantity: String,
    max_notional: String,
    risk_policy_max_notional: String,
    risk_policy_ref: String,
    owner_authority_ref: String,
    risk_authority_ref: String,
    operator_authority_ref: String,
    authorized_at_unix_ms: u64,
    expires_at_unix_ms: u64,
    owner_confirmed: bool,
    risk_confirmed: bool,
    operator_confirmed: bool,
    kill_switch_active: bool,
    single_shot: bool,
    consumed: bool,
    cancel_order_allowed: bool,
    replace_order_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_recovery_allowed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRunCandidateManifest {
    schema_version: String,
    request_sha256: String,
    strategy_version_content_hash: String,
    data_symbols: Vec<String>,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    environment: String,
    account_ref: String,
    venue_ref: String,
    created_at_unix_ms: u64,
    source_refs: Vec<String>,
}

#[derive(Clone, Copy)]
struct LiveRunStrategyVersionBinding<'a> {
    content_hash: &'a str,
    data_symbols: &'a [String],
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
    sizing: LiveSizingPreflight,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveSizingDecisionArtifact {
    schema_version: String,
    run_id: String,
    source_manifest_sha256: String,
    source_preflight_sha256: String,
    strategy_intent_sha256: String,
    instrument_id: String,
    side: String,
    price: String,
    price_tick: String,
    source_quantity: String,
    approved_quantity: String,
    quantity_step: String,
    min_quantity: String,
    max_quantity: String,
    min_notional: String,
    max_account_budget_fraction: String,
    order_notional: String,
    account_budget_notional: String,
    request_max_notional: String,
    risk_policy_max_notional: String,
    sizing_source_ref: String,
    evaluated_at_unix_ms: u64,
    evidence_expires_at_unix_ms: u64,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    execution_order_sha256: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    execution_admission_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    execution_runtime_config_sha256: Option<String>,
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
    file: fs::File,
    _process_reservation: LiveRunProcessMutationReservation,
}

struct LiveRunProcessMutationReservation {
    artifact_root: PathBuf,
}

static LIVE_RUN_PROCESS_MUTATION_LOCKS: std::sync::LazyLock<std::sync::Mutex<BTreeSet<PathBuf>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(BTreeSet::new()));

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRunMutationLockArtifact {
    schema_version: String,
    pid: u32,
    process_start_time_secs: u64,
    acquired_at_unix_ms: u64,
}

impl Drop for LiveRunMutationLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

impl Drop for LiveRunProcessMutationReservation {
    fn drop(&mut self) {
        if let Ok(mut locks) = LIVE_RUN_PROCESS_MUTATION_LOCKS.lock() {
            locks.remove(&self.artifact_root);
        }
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
    owner_approved: bool,
    risk_approved: bool,
    operator_approved: bool,
    blockers: Vec<String>,
}

impl LiveOrderAdmissionSnapshot {
    #[cfg(test)]
    fn blocked() -> Self {
        Self::blocked_with_approvals(false, false, false)
    }

    fn blocked_with_approvals(
        owner_approved: bool,
        risk_approved: bool,
        operator_approved: bool,
    ) -> Self {
        Self {
            status: "blocked".to_string(),
            submit: "blocked".to_string(),
            cancel: "blocked".to_string(),
            replace: "blocked".to_string(),
            fill_reconciliation: "blocked".to_string(),
            owner_approved,
            risk_approved,
            operator_approved,
            blockers: vec![
                "production_order_authority_not_granted".to_string(),
                "execution_adapter_send_not_enabled".to_string(),
                "fill_reconciliation_not_enabled".to_string(),
            ],
        }
    }

    fn authorized() -> Self {
        Self {
            status: "authorized_single_shot".to_string(),
            submit: "authorized_single_shot".to_string(),
            cancel: "blocked".to_string(),
            replace: "blocked".to_string(),
            fill_reconciliation: "runtime_event_projection".to_string(),
            owner_approved: true,
            risk_approved: true,
            operator_approved: true,
            blockers: vec![
                "additional_orders_blocked".to_string(),
                "cancel_not_scoped".to_string(),
                "replace_not_scoped".to_string(),
            ],
        }
    }

    fn consumed() -> Self {
        Self {
            status: "consumed_single_shot".to_string(),
            submit: "blocked".to_string(),
            cancel: "blocked".to_string(),
            replace: "blocked".to_string(),
            fill_reconciliation: "runtime_event_projection".to_string(),
            owner_approved: true,
            risk_approved: true,
            operator_approved: true,
            blockers: vec![
                "single_shot_admission_consumed".to_string(),
                "additional_orders_blocked".to_string(),
                "manual_review_required_for_follow_up".to_string(),
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
    strategy_intent: Option<LiveStrategyOrderIntentArtifact>,
    strategy_intent_sha256: Option<String>,
    sizing_decision: Option<LiveSizingDecisionArtifact>,
    sizing_decision_sha256: Option<String>,
    execution_order: Option<LiveExecutionOrderSnapshot>,
    execution_order_state_sha256: Option<String>,
    execution_control: Option<LiveExecutionControlSnapshot>,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionOrderSnapshot {
    schema_version: String,
    admission_id: String,
    source_demo_run_id: String,
    strategy_intent_id: String,
    strategy_intent_sha256: String,
    sizing_decision_sha256: String,
    strategy_version_id: String,
    instrument_id: String,
    client_order_id: Option<String>,
    venue_order_id: Option<String>,
    original_quantity: String,
    filled_quantity: String,
    remaining_quantity: String,
    status: String,
    terminal: bool,
    new_orders_blocked: bool,
    actual_submission_attempted: bool,
    automatic_retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    last_error: Option<String>,
    updated_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionControlSnapshot {
    schema_version: String,
    request_sha256: String,
    request_id: String,
    action: String,
    run_id: String,
    admission_id: String,
    strategy_version_id: String,
    instrument_id: String,
    client_order_id: String,
    venue_order_id: Option<String>,
    status: String,
    exchange_order_status: Option<String>,
    original_quantity: Option<String>,
    filled_quantity: Option<String>,
    remaining_quantity: Option<String>,
    query_attempted: bool,
    cancel_attempted: bool,
    cancel_confirmed: bool,
    automatic_retry_attempted: bool,
    manual_review_required: bool,
    error_code: Option<String>,
    completed_at_unix_ms: u64,
}

struct LiveExecutionStopActivity {
    endpoint_access_attempted: bool,
    adapter_send_attempted: bool,
    real_order_submitted: bool,
    order_sha256: Option<String>,
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
    const fn enforced(
        order_authorized: bool,
        order_control_allowed: bool,
        execution_adapter_send_attempted: bool,
        real_orders_submitted: bool,
    ) -> Self {
        Self {
            candidate_creation_allowed: true,
            explicit_preflight_allowed: true,
            manual_stop_allowed: true,
            live_runtime_start_allowed: true,
            external_market_data_connection_allowed: true,
            order_endpoint_access_allowed: order_authorized,
            order_submission_allowed: order_authorized,
            cancel_order_allowed: order_control_allowed,
            replace_order_allowed: false,
            fill_reconciliation_allowed: order_control_allowed,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            automatic_recovery_allowed: false,
            execution_adapter_send_attempted,
            real_orders_submitted,
            trading_controls_enabled: order_authorized,
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
    execution_single_shot: bool,
    order_control: bool,
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
            execution_single_shot: reader(LIVE_RUN_GATE_EXECUTION_SINGLE_SHOT),
            order_control: reader(LIVE_RUN_GATE_ORDER_CONTROL),
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
            LIVE_RUN_GATE_ORDER_CONTROL.to_string(),
            LIVE_RUN_GATE_RISK_APPROVED.to_string(),
            LIVE_RUN_GATE_EXECUTION_SINGLE_SHOT.to_string(),
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
        create_live_run_candidate_with_symbols(
            &worker_state,
            request,
            &worker_request_id,
            now,
            admission,
            LiveRunGateState::from_environment(),
            LiveRunStrategyVersionBinding {
                content_hash: version.content_hash(),
                data_symbols: version.data_symbols(),
            },
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
        let _guard = state.lifecycle_action_lock.lock().map_err(|_| {
            product_error(ProductErrorKind::LiveConflict, "live_candidate_action_lock")
        })?;
        let _workspace_lock = acquire_live_run_mutation_lock(&state)?;
        if let Some(pointer) = load_active_live_run_pointer(&state)? {
            reconcile_exited_live_market_data_runtime(&state, &pointer.run_id)?;
        }
        let active = load_active_live_run_candidates(&state)?;
        for (candidate, _, _) in &active {
            reconcile_exited_live_market_data_runtime(&state, &candidate.run_id)?;
        }
        let active = load_active_live_run_candidates(&state)?;
        for (_, manifest, _) in &active {
            validate_live_candidate_against_current_source(&state, manifest)?;
        }
        let data = active
            .into_iter()
            .map(|(candidate, _, _)| candidate)
            .collect::<Vec<_>>();
        let order_authorized = data
            .iter()
            .any(|candidate| candidate.order_admission.status == "authorized_single_shot");
        let execution_adapter_send_attempted = data.iter().any(|candidate| {
            candidate
                .execution_order
                .as_ref()
                .is_some_and(|order| order.actual_submission_attempted)
        });
        let real_orders_submitted = data.iter().any(|candidate| {
            candidate
                .execution_order
                .as_ref()
                .is_some_and(execution_order_has_confirmed_submission)
        });
        let order_control_allowed = LiveRunGateState::from_environment().order_control
            && data.iter().any(|candidate| {
                candidate.lifecycle == LiveRunCandidateLifecycle::MarketDataRunning
                    && candidate
                        .execution_order
                        .as_ref()
                        .is_some_and(|order| !order.terminal && order.client_order_id.is_some())
            });
        Ok(LiveRunCandidateListResponse {
            schema_version: LIVE_RUN_CANDIDATE_LIST_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data,
            runtime_gate_refs: LiveRunGateState::refs(),
            audit_anchor_config_refs: anchor_config_refs(),
            boundaries: LiveRunCandidateBoundaries::enforced(
                order_authorized,
                order_control_allowed,
                execution_adapter_send_attempted,
                real_orders_submitted,
            ),
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
        let _guard = state.lifecycle_action_lock.lock().map_err(|_| {
            product_error(ProductErrorKind::LiveConflict, "live_candidate_action_lock")
        })?;
        let _workspace_lock = acquire_live_run_mutation_lock(&state)?;
        validate_identifier("run_id", &run_id)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "run_id"))?;
        reconcile_exited_live_market_data_runtime(&state, &run_id)?;
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
    if request.action == LiveRunCandidateAction::StartExecution
        && !LiveRunGateState::from_environment().execution_single_shot
    {
        return Err(product_error_response(
            &product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_runtime_gate",
            ),
            &request_id,
        ));
    }
    if request.action == LiveRunCandidateAction::ReconcileOrder
        && !LiveRunGateState::from_environment().order_control
    {
        return Err(product_error_response(
            &product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_order_control_gate",
            ),
            &request_id,
        ));
    }
    let worker_state = state.clone();
    tokio::task::spawn_blocking(move || {
        let _guard = worker_state.lifecycle_action_lock.lock().map_err(|_| {
            product_error(ProductErrorKind::LiveConflict, "live_candidate_action_lock")
        })?;
        run_live_candidate_action_api_guarded(&worker_state, &run_id, &request)
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

pub(in crate::dashboard) async fn live_execution_owner_approval_api(
    state: State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<LiveExecutionAdmissionRequest>, JsonRejection>,
) -> ApiResult<LiveRunCandidateResponse> {
    live_execution_approval_api(LiveExecutionApprovalRole::Owner, state, run_path, payload).await
}

pub(in crate::dashboard) async fn live_execution_risk_approval_api(
    state: State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<LiveExecutionAdmissionRequest>, JsonRejection>,
) -> ApiResult<LiveRunCandidateResponse> {
    live_execution_approval_api(LiveExecutionApprovalRole::Risk, state, run_path, payload).await
}

pub(in crate::dashboard) async fn live_execution_operator_approval_api(
    state: State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<LiveExecutionAdmissionRequest>, JsonRejection>,
) -> ApiResult<LiveRunCandidateResponse> {
    live_execution_approval_api(
        LiveExecutionApprovalRole::Operator,
        state,
        run_path,
        payload,
    )
    .await
}

pub(in crate::dashboard) async fn live_execution_cancel_owner_approval_api(
    state: State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<LiveExecutionCancelRequest>, JsonRejection>,
) -> ApiResult<LiveRunCandidateResponse> {
    live_execution_cancel_approval_api(LiveExecutionApprovalRole::Owner, state, run_path, payload)
        .await
}

pub(in crate::dashboard) async fn live_execution_cancel_operator_approval_api(
    state: State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<LiveExecutionCancelRequest>, JsonRejection>,
) -> ApiResult<LiveRunCandidateResponse> {
    live_execution_cancel_approval_api(
        LiveExecutionApprovalRole::Operator,
        state,
        run_path,
        payload,
    )
    .await
}

async fn live_execution_cancel_approval_api(
    role: LiveExecutionApprovalRole,
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<LiveExecutionCancelRequest>, JsonRejection>,
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
    if !LiveRunGateState::from_environment().order_control {
        return Err(product_error_response(
            &product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_order_control_gate",
            ),
            &request_id,
        ));
    }
    let worker_state = state.clone();
    tokio::task::spawn_blocking(move || {
        let _guard = worker_state.lifecycle_action_lock.lock().map_err(|_| {
            product_error(ProductErrorKind::LiveConflict, "live_candidate_action_lock")
        })?;
        authorize_live_execution_cancel(&worker_state, &run_id, &request, role)
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

async fn live_execution_approval_api(
    role: LiveExecutionApprovalRole,
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<LiveExecutionAdmissionRequest>, JsonRejection>,
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
        authorize_live_execution(&worker_state, &run_id, &request, role)
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

fn authorize_live_execution(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveExecutionAdmissionRequest,
    role: LiveExecutionApprovalRole,
) -> Result<LiveRunCandidate, ProductError> {
    if !LiveRunGateState::from_environment().execution_single_shot {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_runtime_gate",
        ));
    }
    authorize_live_execution_with_source_validator(state, path_run_id, request, role, |manifest| {
        validate_live_candidate_against_current_source(state, manifest)?;
        let source = load_product_source(state, unix_time_ms())?;
        let risk = evaluate_live_execution_risk_policy(&source)?;
        let intent = load_promotable_demo_order_intent(
            state,
            &request.source_demo_run_id,
            &manifest.strategy_id,
            &manifest.strategy_version_id,
        )?;
        Ok((risk, intent))
    })
}

fn authorize_live_execution_with_source_validator<F>(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveExecutionAdmissionRequest,
    role: LiveExecutionApprovalRole,
    source_validator: F,
) -> Result<LiveRunCandidate, ProductError>
where
    F: FnOnce(
        &LiveRunCandidateManifest,
    )
        -> Result<(LiveExecutionRiskPolicy, PromotableStrategyOrderIntent), ProductError>,
{
    authorize_live_execution_with_publication_failure(
        state,
        path_run_id,
        request,
        role,
        source_validator,
        None,
    )
}

fn authorize_live_execution_with_publication_failure<F>(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveExecutionAdmissionRequest,
    role: LiveExecutionApprovalRole,
    source_validator: F,
    fail_after: Option<ApprovalPublicationStep>,
) -> Result<LiveRunCandidate, ProductError>
where
    F: FnOnce(
        &LiveRunCandidateManifest,
    )
        -> Result<(LiveExecutionRiskPolicy, PromotableStrategyOrderIntent), ProductError>,
{
    let _workspace_lock = acquire_live_run_mutation_lock(state)?;
    validate_identifier("run_id", path_run_id)?;
    validate_identifier("execution_admission_id", &request.admission_id)?;
    validate_identifier("source_demo_run_id", &request.source_demo_run_id)?;
    validate_identifier("strategy_intent_id", &request.strategy_intent_id)?;
    let candidate_root = canonical_live_run_root(state, false)?.join(path_run_id);
    if candidate_root.join(role.publication_stage_file()).exists() {
        let (manifest, _) = load_live_run_manifest(state, path_run_id)?;
        let (risk_policy, promoted) = source_validator(&manifest)?;
        let intent = LiveStrategyOrderIntentArtifact::from(promoted);
        resume_live_execution_approval_publication(
            state,
            path_run_id,
            request,
            role,
            &risk_policy,
            &intent,
            fail_after,
        )?;
        return load_live_run_candidate(state, path_run_id);
    }
    let (candidate, manifest, manifest_raw) = load_live_run_candidate_snapshot(state, path_run_id)?;
    if request.run_id != path_run_id
        || request.strategy_version_id != manifest.strategy_version_id
        || request.account_ref != manifest.account_ref
        || request.venue_ref != manifest.venue_ref
        || candidate.lifecycle != LiveRunCandidateLifecycle::PreflightReady
        || !request.user_confirmed
        || request.order_type != "LIMIT"
        || request.time_in_force != "GTC"
        || !matches!(request.side.as_str(), "BUY" | "SELL")
        || !manifest.data_symbols.contains(&request.instrument_id)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_admission",
        ));
    }
    let now = unix_time_ms();
    if request.expires_at_unix_ms <= now
        || request.expires_at_unix_ms > now.saturating_add(15 * 60 * 1_000)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_admission_expiry",
        ));
    }
    let (risk_policy, promoted) = source_validator(&manifest)?;
    let intent = LiveStrategyOrderIntentArtifact::from(promoted);
    ensure_strategy_intent_is_unconsumed(state, path_run_id, &intent)?;
    let intent_raw = serde_json::to_vec_pretty(&intent).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_strategy_intent",
        )
    })?;
    let preflight_raw =
        read_live_run_artifact_bytes(&candidate_root.join("preflight.json"), "live_preflight")?;
    let preflight: LiveRunPreflightArtifact = serde_json::from_slice(&preflight_raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_preflight"))?;
    if now >= preflight.sizing.evidence_expires_at_unix_ms {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_sizing_decision",
        ));
    }
    if request.strategy_intent_id != intent.intent_id
        || request.instrument_id != intent.instrument_id
        || request.side != intent.side
        || request.quantity != intent.quantity
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_strategy_intent_binding",
        ));
    }
    let manifest_sha256 = sha256_ref(&manifest_raw);
    let sizing_decision = evaluate_live_sizing_decision(
        path_run_id,
        &manifest_sha256,
        &sha256_ref(&preflight_raw),
        &sha256_ref(&intent_raw),
        request,
        &preflight.sizing,
        &risk_policy,
        preflight.evaluated_at_unix_ms,
    )?;
    let sizing_raw = serde_json::to_vec_pretty(&sizing_decision).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_sizing_decision",
        )
    })?;
    let sizing_sha256 = sha256_ref(&sizing_raw);
    let (current_state, current_state_raw) =
        load_live_run_state(state, path_run_id, &manifest_sha256)?;
    if current_state.execution_admission_sha256.is_some() {
        return Err(product_error(
            ProductErrorKind::LiveConflict,
            "live_execution_admission",
        ));
    }
    let raw_request = serde_json::to_vec(request).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_execution_admission",
        )
    })?;
    let proposal_sha256 = sha256_ref(&raw_request);
    let authority_ref = match role {
        LiveExecutionApprovalRole::Owner => &risk_policy.owner_authority_ref,
        LiveExecutionApprovalRole::Risk => &risk_policy.risk_authority_ref,
        LiveExecutionApprovalRole::Operator => &risk_policy.operator_authority_ref,
    };
    let approval = LiveExecutionApprovalArtifact {
        schema_version: LIVE_EXECUTION_APPROVAL_SCHEMA_VERSION.to_string(),
        role,
        proposal_sha256: proposal_sha256.clone(),
        strategy_intent_sha256: sha256_ref(&intent_raw),
        sizing_decision_sha256: sizing_sha256.clone(),
        source_manifest_sha256: manifest_sha256.clone(),
        run_id: path_run_id.to_string(),
        strategy_version_id: manifest.strategy_version_id.clone(),
        admission_id: request.admission_id.clone(),
        authority_ref: authority_ref.clone(),
        risk_policy_ref: risk_policy.source_ref.clone(),
        approved_at_unix_ms: now,
        expires_at_unix_ms: request.expires_at_unix_ms,
    };
    let directory = open_absolute_directory_nofollow(&candidate_root)?;
    if let Some((existing, existing_raw)) =
        read_optional_artifact_with_raw::<LiveStrategyOrderIntentArtifact>(
            &candidate_root.join(LIVE_STRATEGY_INTENT_FILE),
            "live_strategy_intent",
        )?
    {
        validate_live_strategy_intent_artifact(&existing, &manifest)?;
        if existing_raw != intent_raw {
            return Err(product_error(
                ProductErrorKind::LiveConflict,
                "live_strategy_intent",
            ));
        }
    } else {
        write_new_run_file(&directory, LIVE_STRATEGY_INTENT_FILE, &intent_raw)?;
    }
    if candidate_root.join(role.artifact_file()).exists()
        || candidate_root.join(role.receipt_file()).exists()
    {
        return Err(product_error(
            ProductErrorKind::LiveConflict,
            "live_execution_approval",
        ));
    }
    let mut approvals = Vec::new();
    for existing_role in [
        LiveExecutionApprovalRole::Owner,
        LiveExecutionApprovalRole::Risk,
        LiveExecutionApprovalRole::Operator,
    ] {
        if let Some((existing, _)) = read_optional_artifact_with_raw::<LiveExecutionApprovalArtifact>(
            &candidate_root.join(existing_role.artifact_file()),
            "live_execution_approval",
        )? {
            validate_live_execution_approval(
                &existing,
                &LiveExecutionApprovalBinding {
                    role: existing_role,
                    manifest_sha256: &manifest_sha256,
                    run_id: path_run_id,
                    strategy_version_id: &manifest.strategy_version_id,
                    admission_id: &request.admission_id,
                    proposal_sha256: &proposal_sha256,
                    strategy_intent_sha256: &sha256_ref(&intent_raw),
                    sizing_decision_sha256: &sizing_sha256,
                    risk_policy: &risk_policy,
                },
            )?;
            approvals.push((existing_role, existing));
        }
    }
    approvals.push((role, approval.clone()));
    let all_roles_approved = approvals.len() == 3;
    let authorized_at = approvals
        .iter()
        .map(|(_, approval)| approval.approved_at_unix_ms)
        .max()
        .unwrap_or(now);
    let admission = all_roles_approved.then(|| LiveExecutionAdmissionArtifact {
        schema_version: LIVE_EXECUTION_ADMISSION_SCHEMA_VERSION.to_string(),
        request_sha256: proposal_sha256.clone(),
        source_manifest_sha256: manifest_sha256.clone(),
        run_id: path_run_id.to_string(),
        strategy_version_id: manifest.strategy_version_id.clone(),
        account_ref: manifest.account_ref.clone(),
        venue_ref: manifest.venue_ref.clone(),
        admission_id: request.admission_id.clone(),
        source_demo_run_id: request.source_demo_run_id.clone(),
        strategy_intent_id: request.strategy_intent_id.clone(),
        strategy_intent_sha256: sha256_ref(&intent_raw),
        sizing_decision_sha256: sizing_sha256.clone(),
        instrument_id: request.instrument_id.clone(),
        side: request.side.clone(),
        order_type: request.order_type.clone(),
        time_in_force: request.time_in_force.clone(),
        price: request.price.clone(),
        source_quantity: request.quantity.clone(),
        quantity: sizing_decision.approved_quantity.clone(),
        max_notional: request.max_notional.clone(),
        risk_policy_max_notional: risk_policy.max_order_notional.clone(),
        risk_policy_ref: risk_policy.source_ref.clone(),
        owner_authority_ref: risk_policy.owner_authority_ref.clone(),
        risk_authority_ref: risk_policy.risk_authority_ref.clone(),
        operator_authority_ref: risk_policy.operator_authority_ref.clone(),
        authorized_at_unix_ms: authorized_at,
        expires_at_unix_ms: request.expires_at_unix_ms,
        owner_confirmed: true,
        risk_confirmed: true,
        operator_confirmed: true,
        kill_switch_active: false,
        single_shot: true,
        consumed: false,
        cancel_order_allowed: false,
        replace_order_allowed: false,
        automatic_retry_allowed: false,
        automatic_recovery_allowed: false,
    });
    let admission_raw = admission
        .as_ref()
        .map(serde_json::to_vec_pretty)
        .transpose()
        .map_err(|_| {
            product_error(
                ProductErrorKind::LiveExecutionFailed,
                "live_execution_admission",
            )
        })?;
    let admission_state = admission_raw.as_ref().map(|raw| LiveRunCandidateState {
        schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
        run_id: path_run_id.to_string(),
        source_manifest_sha256: manifest_sha256.clone(),
        revision: current_state.revision + 1,
        previous_state_sha256: Some(sha256_ref(&current_state_raw)),
        lifecycle: LiveRunCandidateLifecycle::PreflightReady,
        preflight_sha256: current_state.preflight_sha256.clone(),
        execution_admission_sha256: Some(sha256_ref(raw)),
        execution_runtime_config_sha256: None,
        stop_sha256: None,
        updated_at_unix_ms: authorized_at,
    });
    let previous_workspace_receipt = validate_workspace_anchor_head(state)?.ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_head",
        )
    })?;
    let stage = LiveExecutionApprovalPublicationStage {
        schema_version: "ntpro.s3.live_execution_approval_publication.v1".to_string(),
        role,
        request: request.clone(),
        strategy_intent: intent.clone(),
        sizing_decision,
        approval,
        admission,
        admission_state,
        base_state: current_state,
        base_state_sha256: sha256_ref(&current_state_raw),
        manifest_sha256,
        previous_workspace_receipt,
    };
    publish_new_run_file(
        &directory,
        role.publication_stage_file(),
        &serde_json::to_vec_pretty(&stage).map_err(|_| {
            product_error(
                ProductErrorKind::LiveExecutionFailed,
                "live_execution_approval_publication",
            )
        })?,
    )?;
    complete_live_execution_approval_publication(
        state,
        path_run_id,
        &stage,
        &risk_policy,
        &intent,
        fail_after,
    )?;
    load_live_run_candidate(state, path_run_id)
}

fn ensure_strategy_intent_is_unconsumed(
    state: &DashboardServerState,
    current_run_id: &str,
    intent: &LiveStrategyOrderIntentArtifact,
) -> Result<(), ProductError> {
    let root = canonical_live_run_root(state, false)?;
    if !root.exists() {
        return Ok(());
    }
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
        if run_id == current_run_id {
            continue;
        }
        let path = entry.path().join(LIVE_STRATEGY_INTENT_FILE);
        if !path.exists() {
            continue;
        }
        let existing_raw = read_live_run_artifact_bytes(&path, "live_strategy_intent")?;
        let existing: LiveStrategyOrderIntentArtifact = serde_json::from_slice(&existing_raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_strategy_intent"))?;
        if existing.source_demo_run_id == intent.source_demo_run_id
            && existing.intent_id == intent.intent_id
            && existing.source_result_sha256 == intent.source_result_sha256
        {
            return Err(product_error(
                ProductErrorKind::LiveConflict,
                "live_strategy_intent_consumed",
            ));
        }
    }
    Ok(())
}

fn validate_live_execution_approval(
    approval: &LiveExecutionApprovalArtifact,
    binding: &LiveExecutionApprovalBinding<'_>,
) -> Result<(), ProductError> {
    let expected_authority = match binding.role {
        LiveExecutionApprovalRole::Owner => &binding.risk_policy.owner_authority_ref,
        LiveExecutionApprovalRole::Risk => &binding.risk_policy.risk_authority_ref,
        LiveExecutionApprovalRole::Operator => &binding.risk_policy.operator_authority_ref,
    };
    if approval.schema_version != LIVE_EXECUTION_APPROVAL_SCHEMA_VERSION
        || approval.role != binding.role
        || approval.proposal_sha256 != binding.proposal_sha256
        || approval.strategy_intent_sha256 != binding.strategy_intent_sha256
        || approval.sizing_decision_sha256 != binding.sizing_decision_sha256
        || approval.source_manifest_sha256 != binding.manifest_sha256
        || approval.run_id != binding.run_id
        || approval.strategy_version_id != binding.strategy_version_id
        || approval.admission_id != binding.admission_id
        || approval.authority_ref != *expected_authority
        || approval.risk_policy_ref != binding.risk_policy.source_ref
        || approval.approved_at_unix_ms == 0
        || approval.approved_at_unix_ms > approval.expires_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_approval",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn evaluate_live_sizing_decision(
    run_id: &str,
    manifest_sha256: &str,
    preflight_sha256: &str,
    strategy_intent_sha256: &str,
    request: &LiveExecutionAdmissionRequest,
    sizing: &LiveSizingPreflight,
    risk_policy: &LiveExecutionRiskPolicy,
    evaluated_at_unix_ms: u64,
) -> Result<LiveSizingDecisionArtifact, ProductError> {
    let boundary = |field: &str| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            format!("live_sizing_decision.{field}"),
        )
    };
    let decimal =
        |value: &str, field: &str| Decimal::from_str_exact(value).map_err(|_| boundary(field));
    let price = decimal(&request.price, "price")?;
    let source_quantity = decimal(&request.quantity, "source_quantity")?;
    let price_tick = decimal(&sizing.price_tick, "price_tick")?;
    let quantity_step = decimal(&sizing.quantity_step, "quantity_step")?;
    let min_quantity = decimal(&sizing.min_quantity, "min_quantity")?;
    let max_quantity = decimal(&sizing.max_quantity, "max_quantity")?;
    let min_notional = decimal(&sizing.min_notional, "min_notional")?;
    let quote_free = decimal(&sizing.quote_free, "account_balance")?;
    let base_free = decimal(&sizing.base_free, "account_balance")?;
    let budget_fraction = decimal(
        &sizing.max_account_budget_fraction,
        "account_budget_fraction",
    )?;
    let request_max = decimal(&request.max_notional, "request_max_notional")?;
    let risk_max = decimal(&risk_policy.max_order_notional, "risk_policy_max_notional")?;
    if evaluated_at_unix_ms >= sizing.evidence_expires_at_unix_ms {
        return Err(boundary("evidence_expired"));
    }
    if request.instrument_id != sizing.instrument_id {
        return Err(boundary("instrument_id"));
    }
    if price <= Decimal::ZERO || price_tick <= Decimal::ZERO || price % price_tick != Decimal::ZERO
    {
        return Err(boundary("price_tick"));
    }
    if source_quantity <= Decimal::ZERO || quantity_step <= Decimal::ZERO {
        return Err(boundary("quantity_step"));
    }
    if request_max <= Decimal::ZERO || request_max > risk_max {
        return Err(boundary("request_max_notional"));
    }
    if risk_max <= Decimal::ZERO {
        return Err(boundary("risk_policy_max_notional"));
    }
    let approved_quantity = (source_quantity / quantity_step).floor() * quantity_step;
    let order_notional = price * approved_quantity;
    let account_budget_notional = match request.side.as_str() {
        "BUY" => quote_free * budget_fraction,
        "SELL" => base_free * price * budget_fraction,
        _ => return Err(boundary("side")),
    };
    if approved_quantity > source_quantity || approved_quantity % quantity_step != Decimal::ZERO {
        return Err(boundary("quantity_step"));
    }
    if approved_quantity <= Decimal::ZERO || approved_quantity < min_quantity {
        return Err(boundary("min_quantity"));
    }
    if approved_quantity > max_quantity {
        return Err(boundary("max_quantity"));
    }
    if request.side == "SELL" && approved_quantity > base_free {
        return Err(boundary("account_balance"));
    }
    if order_notional < min_notional {
        return Err(boundary("min_notional"));
    }
    if order_notional > account_budget_notional {
        return Err(boundary("account_budget"));
    }
    if order_notional > request_max {
        return Err(boundary("request_max_notional"));
    }
    if order_notional > risk_max {
        return Err(boundary("risk_policy_max_notional"));
    }
    Ok(LiveSizingDecisionArtifact {
        schema_version: LIVE_SIZING_DECISION_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        source_manifest_sha256: manifest_sha256.to_string(),
        source_preflight_sha256: preflight_sha256.to_string(),
        strategy_intent_sha256: strategy_intent_sha256.to_string(),
        instrument_id: request.instrument_id.clone(),
        side: request.side.clone(),
        price: request.price.clone(),
        price_tick: sizing.price_tick.clone(),
        source_quantity: request.quantity.clone(),
        approved_quantity: approved_quantity.normalize().to_string(),
        quantity_step: sizing.quantity_step.clone(),
        min_quantity: sizing.min_quantity.clone(),
        max_quantity: sizing.max_quantity.clone(),
        min_notional: sizing.min_notional.clone(),
        max_account_budget_fraction: sizing.max_account_budget_fraction.clone(),
        order_notional: order_notional.normalize().to_string(),
        account_budget_notional: account_budget_notional.normalize().to_string(),
        request_max_notional: request.max_notional.clone(),
        risk_policy_max_notional: risk_policy.max_order_notional.clone(),
        sizing_source_ref: sizing.source_ref.clone(),
        evaluated_at_unix_ms,
        evidence_expires_at_unix_ms: sizing.evidence_expires_at_unix_ms,
    })
}

fn authorize_live_execution_cancel(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveExecutionCancelRequest,
    role: LiveExecutionApprovalRole,
) -> Result<LiveRunCandidate, ProductError> {
    authorize_live_execution_cancel_with_failure(state, path_run_id, request, role, None)
}

fn authorize_live_execution_cancel_with_failure(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveExecutionCancelRequest,
    role: LiveExecutionApprovalRole,
    fail_after: Option<CancelPublicationStep>,
) -> Result<LiveRunCandidate, ProductError> {
    if role == LiveExecutionApprovalRole::Risk {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_role",
        ));
    }
    let _workspace_lock = acquire_live_run_mutation_lock(state)?;
    validate_identifier("run_id", path_run_id)?;
    validate_identifier("execution_cancel_request_id", &request.request_id)?;
    let candidate_root = canonical_live_run_root(state, false)?.join(path_run_id);
    let stage_file = cancel_publication_stage_file(role)?;
    if candidate_root.join(stage_file).exists() {
        resume_live_execution_cancel_publication(state, path_run_id, request, role, fail_after)?;
        return load_live_run_candidate(state, path_run_id);
    }
    let (candidate, _manifest, manifest_raw) =
        load_live_run_candidate_snapshot(state, path_run_id)?;
    if candidate.lifecycle != LiveRunCandidateLifecycle::MarketDataRunning
        || request.run_id != path_run_id
        || !request.user_confirmed
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_request",
        ));
    }
    let now = unix_time_ms();
    if request.expires_at_unix_ms <= now
        || request.expires_at_unix_ms > now.saturating_add(5 * 60 * 1_000)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_expiry",
        ));
    }
    let (admission, _) = read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
        &candidate_root.join(LIVE_EXECUTION_ADMISSION_FILE),
        "live_execution_admission",
    )?
    .ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_admission",
        )
    })?;
    let runtime_root = live_market_data_runtime_root(state, path_run_id)?;
    let (order, order_raw) = read_optional_artifact_with_raw::<LiveExecutionOrderSnapshot>(
        &runtime_root.join(LIVE_EXECUTION_ORDER_STATE_FILE),
        "live_execution_order_state",
    )?
    .ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_order",
        )
    })?;
    validate_execution_order_snapshot(&order, &admission)?;
    let (source_order, source_order_raw) = if role == LiveExecutionApprovalRole::Operator {
        read_optional_artifact_with_raw::<LiveExecutionOrderSnapshot>(
            &candidate_root.join(LIVE_EXECUTION_CANCEL_SOURCE_ORDER_FILE),
            "live_execution_cancel_source_order",
        )?
        .ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_source_order",
            )
        })?
    } else {
        (order.clone(), order_raw)
    };
    validate_execution_cancel_request_order(
        request,
        &source_order,
        &source_order_raw,
        &order,
        &admission,
    )?;
    let manifest_sha256 = sha256_ref(&manifest_raw);
    let (current_state, _) = load_live_run_state(state, path_run_id, &manifest_sha256)?;
    let proposal_raw = serde_json::to_vec(request).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_execution_cancel_request",
        )
    })?;
    let proposal_sha256 = sha256_ref(&proposal_raw);
    let authority_ref = match role {
        LiveExecutionApprovalRole::Owner => &admission.owner_authority_ref,
        LiveExecutionApprovalRole::Operator => &admission.operator_authority_ref,
        LiveExecutionApprovalRole::Risk => unreachable!(),
    };
    let approval = LiveExecutionCancelApprovalArtifact {
        schema_version: "ntpro.s3.live_execution_cancel_approval.v1".to_string(),
        role,
        proposal_sha256: proposal_sha256.clone(),
        source_manifest_sha256: manifest_sha256.clone(),
        run_id: path_run_id.to_string(),
        admission_id: admission.admission_id.clone(),
        strategy_version_id: admission.strategy_version_id.clone(),
        instrument_id: admission.instrument_id.clone(),
        client_order_id: request.client_order_id.clone(),
        source_order_state_sha256: request.source_order_state_sha256.clone(),
        authority_ref: authority_ref.clone(),
        approved_at_unix_ms: now,
        expires_at_unix_ms: request.expires_at_unix_ms,
    };
    let (approval_file, receipt_file) = match role {
        LiveExecutionApprovalRole::Owner => (
            LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE,
            LIVE_EXECUTION_CANCEL_OWNER_RECEIPT_FILE,
        ),
        LiveExecutionApprovalRole::Operator => (
            LIVE_EXECUTION_CANCEL_OPERATOR_APPROVAL_FILE,
            LIVE_EXECUTION_CANCEL_OPERATOR_RECEIPT_FILE,
        ),
        LiveExecutionApprovalRole::Risk => unreachable!(),
    };
    if candidate_root.join(approval_file).exists()
        || candidate_root.join(receipt_file).exists()
        || candidate_root
            .join(LIVE_EXECUTION_CANCEL_RESULT_FILE)
            .exists()
    {
        return Err(product_error(
            ProductErrorKind::LiveConflict,
            "live_execution_cancel_approval",
        ));
    }
    if role == LiveExecutionApprovalRole::Operator {
        let (owner, _) = read_optional_artifact_with_raw::<LiveExecutionCancelApprovalArtifact>(
            &candidate_root.join(LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE),
            "live_execution_cancel_owner_approval",
        )?
        .ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_owner_approval",
            )
        })?;
        if owner.schema_version != "ntpro.s3.live_execution_cancel_approval.v1"
            || owner.role != LiveExecutionApprovalRole::Owner
            || owner.proposal_sha256 != proposal_sha256
            || owner.source_manifest_sha256 != manifest_sha256
            || owner.run_id != path_run_id
            || owner.admission_id != admission.admission_id
            || owner.strategy_version_id != admission.strategy_version_id
            || owner.instrument_id != admission.instrument_id
            || owner.client_order_id != request.client_order_id
            || owner.source_order_state_sha256 != request.source_order_state_sha256
            || owner.authority_ref != admission.owner_authority_ref
            || owner.expires_at_unix_ms != request.expires_at_unix_ms
            || owner.expires_at_unix_ms <= now
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_owner_approval",
            ));
        }
    }
    let control_request = if role == LiveExecutionApprovalRole::Operator {
        Some(LiveExecutionControlRequestArtifact {
            schema_version: LIVE_EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION.to_string(),
            request_id: request.request_id.clone(),
            action: "cancel".to_string(),
            run_id: path_run_id.to_string(),
            admission_id: admission.admission_id,
            strategy_version_id: admission.strategy_version_id,
            instrument_id: admission.instrument_id,
            client_order_id: request.client_order_id.clone(),
            source_order_state_sha256: request.source_order_state_sha256.clone(),
            owner_confirmed: true,
            operator_confirmed: true,
            requested_at_unix_ms: now,
            expires_at_unix_ms: request.expires_at_unix_ms,
        })
    } else {
        None
    };
    let previous_workspace_receipt = validate_workspace_anchor_head(state)?.ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_head",
        )
    })?;
    let stage = LiveExecutionCancelPublicationStage {
        schema_version: "ntpro.s3.live_execution_cancel_publication.v1".to_string(),
        role,
        request: request.clone(),
        source_order_raw: String::from_utf8(source_order_raw).map_err(|_| {
            product_error(
                ProductErrorKind::SourceInvalid,
                "live_execution_cancel_source_order",
            )
        })?,
        approval,
        control_request,
        run_revision: current_state.revision,
        manifest_sha256,
        previous_workspace_receipt,
    };
    let stage_raw = serde_json::to_vec_pretty(&stage).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_execution_cancel_publication",
        )
    })?;
    let directory = open_absolute_directory_nofollow(&candidate_root)?;
    publish_new_run_file(&directory, stage_file, &stage_raw)?;
    complete_live_execution_cancel_publication(state, path_run_id, &stage, fail_after)?;
    load_live_run_candidate(state, path_run_id)
}

fn cancel_publication_stage_file(
    role: LiveExecutionApprovalRole,
) -> Result<&'static str, ProductError> {
    match role {
        LiveExecutionApprovalRole::Owner => Ok(LIVE_EXECUTION_CANCEL_OWNER_STAGE_FILE),
        LiveExecutionApprovalRole::Operator => Ok(LIVE_EXECUTION_CANCEL_OPERATOR_STAGE_FILE),
        LiveExecutionApprovalRole::Risk => Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_role",
        )),
    }
}

fn validate_execution_cancel_request_order(
    request: &LiveExecutionCancelRequest,
    source: &LiveExecutionOrderSnapshot,
    source_raw: &[u8],
    current: &LiveExecutionOrderSnapshot,
    admission: &LiveExecutionAdmissionArtifact,
) -> Result<(), ProductError> {
    validate_execution_cancel_order_progression(request, source, source_raw, current, admission)?;
    if current.terminal {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_order",
        ));
    }
    Ok(())
}

fn validate_execution_cancel_order_progression(
    request: &LiveExecutionCancelRequest,
    source: &LiveExecutionOrderSnapshot,
    source_raw: &[u8],
    current: &LiveExecutionOrderSnapshot,
    admission: &LiveExecutionAdmissionArtifact,
) -> Result<(), ProductError> {
    validate_execution_order_snapshot(source, admission)?;
    validate_execution_order_snapshot(current, admission)?;
    validate_execution_order_progression(source, current)?;
    if source.client_order_id.as_deref() != Some(request.client_order_id.as_str())
        || request.source_order_state_sha256 != sha256_ref(source_raw)
        || current.replace_attempted
        || !execution_order_cancel_attempt_is_valid(current)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_order",
        ));
    }
    Ok(())
}

fn resume_live_execution_cancel_publication(
    state: &DashboardServerState,
    run_id: &str,
    request: &LiveExecutionCancelRequest,
    role: LiveExecutionApprovalRole,
    fail_after: Option<CancelPublicationStep>,
) -> Result<(), ProductError> {
    let candidate_root = canonical_live_run_root(state, false)?.join(run_id);
    let stage_file = cancel_publication_stage_file(role)?;
    let (stage, _) = read_optional_artifact_with_raw::<LiveExecutionCancelPublicationStage>(
        &candidate_root.join(stage_file),
        "live_execution_cancel_publication",
    )?
    .ok_or_else(|| {
        product_error(
            ProductErrorKind::SourceUnavailable,
            "live_execution_cancel_publication",
        )
    })?;
    if stage.schema_version != "ntpro.s3.live_execution_cancel_publication.v1"
        || stage.role != role
        || stage.request != *request
        || stage.request.run_id != run_id
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_publication",
        ));
    }
    complete_live_execution_cancel_publication(state, run_id, &stage, fail_after)
}

fn complete_live_execution_cancel_publication(
    state: &DashboardServerState,
    run_id: &str,
    stage: &LiveExecutionCancelPublicationStage,
    fail_after: Option<CancelPublicationStep>,
) -> Result<(), ProductError> {
    let candidate_root = canonical_live_run_root(state, false)?.join(run_id);
    let directory = open_absolute_directory_nofollow(&candidate_root)?;
    let (_manifest, manifest_raw) = load_live_run_manifest(state, run_id)?;
    if sha256_ref(&manifest_raw) != stage.manifest_sha256 {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_publication",
        ));
    }
    let (current_state, _) = load_live_run_state(state, run_id, &stage.manifest_sha256)?;
    if current_state.revision != stage.run_revision
        || current_state.lifecycle != LiveRunCandidateLifecycle::MarketDataRunning
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_publication",
        ));
    }
    let (admission, _) = read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
        &candidate_root.join(LIVE_EXECUTION_ADMISSION_FILE),
        "live_execution_cancel_admission",
    )?
    .ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_admission",
        )
    })?;
    let source_raw = stage.source_order_raw.as_bytes();
    let source_order: LiveExecutionOrderSnapshot =
        serde_json::from_slice(source_raw).map_err(|_| {
            product_error(
                ProductErrorKind::SourceInvalid,
                "live_execution_cancel_source_order",
            )
        })?;
    let runtime_root = live_market_data_runtime_root(state, run_id)?;
    let (current_order, _) = read_optional_artifact_with_raw::<LiveExecutionOrderSnapshot>(
        &runtime_root.join(LIVE_EXECUTION_ORDER_STATE_FILE),
        "live_execution_cancel_order",
    )?
    .ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_order",
        )
    })?;
    validate_execution_cancel_order_progression(
        &stage.request,
        &source_order,
        source_raw,
        &current_order,
        &admission,
    )?;
    let proposal_raw = serde_json::to_vec(&stage.request).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_cancel_publication",
        )
    })?;
    if stage.approval.proposal_sha256 != sha256_ref(&proposal_raw)
        || stage.approval.source_order_state_sha256 != stage.request.source_order_state_sha256
        || stage.approval.source_manifest_sha256 != stage.manifest_sha256
        || stage.approval.run_id != run_id
        || stage.approval.role != stage.role
        || stage.approval.client_order_id != stage.request.client_order_id
        || stage.approval.admission_id != admission.admission_id
        || stage.approval.strategy_version_id != admission.strategy_version_id
        || stage.approval.instrument_id != admission.instrument_id
        || stage.approval.approved_at_unix_ms == 0
        || stage.approval.approved_at_unix_ms > stage.approval.expires_at_unix_ms
        || stage.approval.expires_at_unix_ms != stage.request.expires_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_publication",
        ));
    }
    let authority_ref = match stage.role {
        LiveExecutionApprovalRole::Owner => &admission.owner_authority_ref,
        LiveExecutionApprovalRole::Operator => &admission.operator_authority_ref,
        LiveExecutionApprovalRole::Risk => unreachable!(),
    };
    if stage.approval.authority_ref != *authority_ref {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_publication",
        ));
    }
    if stage.role == LiveExecutionApprovalRole::Operator {
        let (owner, _) = read_optional_artifact_with_raw::<LiveExecutionCancelApprovalArtifact>(
            &candidate_root.join(LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE),
            "live_execution_cancel_owner_approval",
        )?
        .ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_owner_approval",
            )
        })?;
        if owner.proposal_sha256 != stage.approval.proposal_sha256
            || owner.source_order_state_sha256 != stage.approval.source_order_state_sha256
            || owner.expires_at_unix_ms != stage.approval.expires_at_unix_ms
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_owner_approval",
            ));
        }
        let control = stage.control_request.as_ref().ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_request",
            )
        })?;
        if control.schema_version != LIVE_EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION
            || control.request_id != stage.request.request_id
            || control.action != "cancel"
            || control.run_id != run_id
            || control.admission_id != admission.admission_id
            || control.strategy_version_id != admission.strategy_version_id
            || control.instrument_id != admission.instrument_id
            || control.client_order_id != stage.request.client_order_id
            || control.source_order_state_sha256 != stage.request.source_order_state_sha256
            || !control.owner_confirmed
            || !control.operator_confirmed
            || control.requested_at_unix_ms != stage.approval.approved_at_unix_ms
            || control.expires_at_unix_ms != stage.request.expires_at_unix_ms
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_request",
            ));
        }
    } else if stage.control_request.is_some() {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_request",
        ));
    }
    let approval_raw = serde_json::to_vec_pretty(&stage.approval).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_execution_cancel_approval",
        )
    })?;
    let (approval_file, receipt_file) = match stage.role {
        LiveExecutionApprovalRole::Owner => (
            LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE,
            LIVE_EXECUTION_CANCEL_OWNER_RECEIPT_FILE,
        ),
        LiveExecutionApprovalRole::Operator => (
            LIVE_EXECUTION_CANCEL_OPERATOR_APPROVAL_FILE,
            LIVE_EXECUTION_CANCEL_OPERATOR_RECEIPT_FILE,
        ),
        LiveExecutionApprovalRole::Risk => unreachable!(),
    };
    if stage.role == LiveExecutionApprovalRole::Owner {
        write_same_or_new_run_file(
            &directory,
            LIVE_EXECUTION_CANCEL_SOURCE_ORDER_FILE,
            source_raw,
            "live_execution_cancel_source_order",
        )?;
        fail_cancel_publication_after(fail_after, CancelPublicationStep::SourceOrder)?;
    } else {
        let persisted_source = read_run_file_from_directory(
            &directory,
            LIVE_EXECUTION_CANCEL_SOURCE_ORDER_FILE,
            "live_execution_cancel_source_order",
        )?;
        if persisted_source != source_raw {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_source_order",
            ));
        }
    }
    let anchor_request = LiveRunAnchorAppendRequest::new(
        state.live_run_audit_anchor.namespace()?,
        run_id,
        LiveRunAnchorRevision::new(
            stage.run_revision,
            stage.previous_workspace_receipt.workspace_revision + 1,
        ),
        sha256_ref(&approval_raw),
        stage.manifest_sha256.clone(),
        Some(stage.previous_workspace_receipt.sha256()),
        stage.approval.approved_at_unix_ms,
    );
    let latest = state.live_run_audit_anchor.latest()?.ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_latest",
        )
    })?;
    let receipt = if latest == stage.previous_workspace_receipt {
        state.live_run_audit_anchor.append(&anchor_request)?
    } else {
        latest
    };
    state
        .live_run_audit_anchor
        .validate_receipt(&receipt, &anchor_request)?;
    fail_cancel_publication_after(fail_after, CancelPublicationStep::ExternalAnchor)?;
    let receipt_raw = serde_json::to_vec_pretty(&receipt).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_execution_cancel_receipt",
        )
    })?;
    write_same_or_new_run_file(
        &directory,
        approval_file,
        &approval_raw,
        "live_execution_cancel_approval",
    )?;
    fail_cancel_publication_after(fail_after, CancelPublicationStep::Approval)?;
    write_same_or_new_run_file(
        &directory,
        receipt_file,
        &receipt_raw,
        "live_execution_cancel_receipt",
    )?;
    fail_cancel_publication_after(fail_after, CancelPublicationStep::Receipt)?;
    recover_workspace_anchor_head_for_publication(
        state,
        &stage.previous_workspace_receipt,
        &receipt,
    )?;
    fail_cancel_publication_after(fail_after, CancelPublicationStep::WorkspaceHead)?;
    if let Some(control) = &stage.control_request {
        let control_raw = serde_json::to_vec_pretty(control).map_err(|_| {
            product_error(
                ProductErrorKind::LiveExecutionFailed,
                "live_execution_cancel_request",
            )
        })?;
        write_same_or_new_run_file(
            &directory,
            LIVE_EXECUTION_CANCEL_REQUEST_FILE,
            &control_raw,
            "live_execution_cancel_request",
        )?;
        fail_cancel_publication_after(fail_after, CancelPublicationStep::ControlRequest)?;
    }
    directory
        .remove_file(cancel_publication_stage_file(stage.role)?)
        .map_err(|_| {
            product_error(
                ProductErrorKind::SourceUnavailable,
                "live_execution_cancel_publication",
            )
        })?;
    if let Ok(parent) = directory.open(".") {
        let _ = parent.sync_all();
    }
    Ok(())
}

fn write_same_or_new_run_file(
    directory: &cap_std::fs::Dir,
    name: &str,
    raw: &[u8],
    field: &'static str,
) -> Result<(), ProductError> {
    match publish_new_run_file(directory, name, raw) {
        Ok(()) => Ok(()),
        Err(error) if error.kind == ProductErrorKind::Conflict => {
            let existing = read_run_file_from_directory(directory, name, field)?;
            if existing == raw {
                Ok(())
            } else {
                Err(product_error(ProductErrorKind::BoundaryViolation, field))
            }
        }
        Err(error) => Err(error),
    }
}

fn recover_workspace_anchor_head_for_publication(
    state: &DashboardServerState,
    previous: &LiveRunAnchorReceipt,
    receipt: &LiveRunAnchorReceipt,
) -> Result<(), ProductError> {
    let head_path = live_run_workspace_anchor_head_path(state)?;
    let next_path = head_path.with_file_name(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_NEXT_FILE);
    if next_path.exists() {
        let next_raw = read_live_run_artifact_bytes(&next_path, "live_run_workspace_anchor_head")?;
        let next: LiveRunAnchorReceipt = serde_json::from_slice(&next_raw).map_err(|_| {
            product_error(
                ProductErrorKind::SourceInvalid,
                "live_run_workspace_anchor_head",
            )
        })?;
        if next != *receipt {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_run_workspace_anchor_head",
            ));
        }
        let artifacts = open_absolute_directory_nofollow(
            head_path
                .parent()
                .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "artifact_root"))?,
        )?;
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
    let local = load_local_workspace_anchor_head(state)?;
    if local.as_ref() == Some(receipt) {
        return Ok(());
    }
    if local.as_ref() != Some(previous) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_head",
        ));
    }
    publish_workspace_anchor_head(state, receipt)
}

fn fail_cancel_publication_after(
    fail_after: Option<CancelPublicationStep>,
    completed: CancelPublicationStep,
) -> Result<(), ProductError> {
    if fail_after == Some(completed) {
        Err(product_error(
            ProductErrorKind::SourceUnavailable,
            "live_execution_cancel_publication_injected_failure",
        ))
    } else {
        Ok(())
    }
}

fn fail_approval_publication_after(
    fail_after: Option<ApprovalPublicationStep>,
    completed: ApprovalPublicationStep,
) -> Result<(), ProductError> {
    if fail_after == Some(completed) {
        Err(product_error(
            ProductErrorKind::SourceUnavailable,
            "live_execution_approval_publication_injected_failure",
        ))
    } else {
        Ok(())
    }
}

fn resume_live_execution_approval_publication(
    state: &DashboardServerState,
    run_id: &str,
    request: &LiveExecutionAdmissionRequest,
    role: LiveExecutionApprovalRole,
    risk_policy: &LiveExecutionRiskPolicy,
    strategy_intent: &LiveStrategyOrderIntentArtifact,
    fail_after: Option<ApprovalPublicationStep>,
) -> Result<(), ProductError> {
    let root = canonical_live_run_root(state, false)?.join(run_id);
    let (stage, _) = read_optional_artifact_with_raw::<LiveExecutionApprovalPublicationStage>(
        &root.join(role.publication_stage_file()),
        "live_execution_approval_publication",
    )?
    .ok_or_else(|| {
        product_error(
            ProductErrorKind::SourceUnavailable,
            "live_execution_approval_publication",
        )
    })?;
    if stage.schema_version != "ntpro.s3.live_execution_approval_publication.v1"
        || stage.role != role
        || stage.request != *request
        || stage.request.run_id != run_id
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_approval_publication",
        ));
    }
    complete_live_execution_approval_publication(
        state,
        run_id,
        &stage,
        risk_policy,
        strategy_intent,
        fail_after,
    )
}

fn complete_live_execution_approval_publication(
    state: &DashboardServerState,
    run_id: &str,
    stage: &LiveExecutionApprovalPublicationStage,
    risk_policy: &LiveExecutionRiskPolicy,
    strategy_intent: &LiveStrategyOrderIntentArtifact,
    fail_after: Option<ApprovalPublicationStep>,
) -> Result<(), ProductError> {
    let root = canonical_live_run_root(state, false)?.join(run_id);
    let directory = open_absolute_directory_nofollow(&root)?;
    let (manifest, manifest_raw) = load_live_run_manifest(state, run_id)?;
    let intent_raw = serde_json::to_vec_pretty(&stage.strategy_intent).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_approval_publication",
        )
    })?;
    let sizing_raw = serde_json::to_vec_pretty(&stage.sizing_decision).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_approval_publication",
        )
    })?;
    let request_raw = serde_json::to_vec(&stage.request).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_approval_publication",
        )
    })?;
    let preflight_raw =
        read_live_run_artifact_bytes(&root.join("preflight.json"), "live_preflight")?;
    let preflight: LiveRunPreflightArtifact = serde_json::from_slice(&preflight_raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_preflight"))?;
    let recomputed_sizing = evaluate_live_sizing_decision(
        run_id,
        &stage.manifest_sha256,
        &sha256_ref(&preflight_raw),
        &sha256_ref(&intent_raw),
        &stage.request,
        &preflight.sizing,
        risk_policy,
        stage.sizing_decision.evaluated_at_unix_ms,
    )?;
    if sha256_ref(&manifest_raw) != stage.manifest_sha256
        || stage.base_state.run_id != run_id
        || stage.base_state.source_manifest_sha256 != stage.manifest_sha256
        || stage.base_state.lifecycle != LiveRunCandidateLifecycle::PreflightReady
        || stage.base_state.execution_admission_sha256.is_some()
        || stage.base_state_sha256
            != sha256_ref(&read_live_run_artifact_bytes(
                &root.join(live_run_state_file_name(stage.base_state.revision)),
                "live_run_state",
            )?)
        || stage.strategy_intent.intent_id != stage.request.strategy_intent_id
        || stage.strategy_intent.source_demo_run_id != stage.request.source_demo_run_id
        || stage.strategy_intent.instrument_id != stage.request.instrument_id
        || stage.strategy_intent.side != stage.request.side
        || stage.strategy_intent.quantity != stage.request.quantity
        || stage.strategy_intent != *strategy_intent
        || stage.sizing_decision.run_id != run_id
        || stage.sizing_decision.source_manifest_sha256 != stage.manifest_sha256
        || stage.sizing_decision.strategy_intent_sha256 != sha256_ref(&intent_raw)
        || stage.sizing_decision.instrument_id != stage.request.instrument_id
        || stage.sizing_decision.side != stage.request.side
        || stage.sizing_decision.source_quantity != stage.request.quantity
        || stage.sizing_decision != recomputed_sizing
        || unix_time_ms() >= stage.sizing_decision.evidence_expires_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_approval_publication",
        ));
    }
    validate_live_execution_approval(
        &stage.approval,
        &LiveExecutionApprovalBinding {
            role: stage.role,
            manifest_sha256: &stage.manifest_sha256,
            run_id,
            strategy_version_id: &manifest.strategy_version_id,
            admission_id: &stage.request.admission_id,
            proposal_sha256: &sha256_ref(&request_raw),
            strategy_intent_sha256: &sha256_ref(&intent_raw),
            sizing_decision_sha256: &sha256_ref(&sizing_raw),
            risk_policy,
        },
    )?;
    let approval_raw = serde_json::to_vec_pretty(&stage.approval).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_execution_approval",
        )
    })?;
    write_same_or_new_run_file(
        &directory,
        LIVE_SIZING_DECISION_FILE,
        &sizing_raw,
        "live_sizing_decision",
    )?;
    let request = LiveRunAnchorAppendRequest::new(
        state.live_run_audit_anchor.namespace()?,
        run_id,
        LiveRunAnchorRevision::new(
            stage.base_state.revision,
            stage.previous_workspace_receipt.workspace_revision + 1,
        ),
        sha256_ref(&approval_raw),
        stage.manifest_sha256.clone(),
        Some(stage.previous_workspace_receipt.sha256()),
        stage.approval.approved_at_unix_ms,
    );
    let receipt = if let Some((persisted, _)) =
        read_optional_artifact_with_raw::<LiveRunAnchorReceipt>(
            &root.join(stage.role.receipt_file()),
            "live_execution_approval_receipt",
        )? {
        persisted
    } else {
        let latest = state.live_run_audit_anchor.latest()?.ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_run_workspace_anchor_latest",
            )
        })?;
        if latest == stage.previous_workspace_receipt {
            state.live_run_audit_anchor.append(&request)?
        } else {
            latest
        }
    };
    state
        .live_run_audit_anchor
        .validate_receipt(&receipt, &request)?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::ExternalAnchor)?;
    let receipt_raw = serde_json::to_vec_pretty(&receipt).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_execution_approval_receipt",
        )
    })?;
    write_same_or_new_run_file(
        &directory,
        stage.role.artifact_file(),
        &approval_raw,
        "live_execution_approval",
    )?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::Approval)?;
    write_same_or_new_run_file(
        &directory,
        stage.role.receipt_file(),
        &receipt_raw,
        "live_execution_approval_receipt",
    )?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::Receipt)?;
    let local_head = load_local_workspace_anchor_head(state)?;
    if local_head.as_ref() == Some(&stage.previous_workspace_receipt)
        || local_head.as_ref() == Some(&receipt)
    {
        recover_workspace_anchor_head_for_publication(
            state,
            &stage.previous_workspace_receipt,
            &receipt,
        )?;
    }
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::WorkspaceHead)?;

    if let (Some(admission), Some(candidate_state)) = (&stage.admission, &stage.admission_state) {
        let admission_raw = serde_json::to_vec_pretty(admission).map_err(|_| {
            product_error(
                ProductErrorKind::LiveExecutionFailed,
                "live_execution_admission",
            )
        })?;
        validate_execution_admission_artifact(
            &manifest,
            &stage.manifest_sha256,
            admission,
            strategy_intent,
            &intent_raw,
            &stage.sizing_decision,
            &sizing_raw,
            candidate_state.lifecycle,
        )?;
        if admission.risk_policy_max_notional != risk_policy.max_order_notional
            || admission.risk_policy_ref != risk_policy.source_ref
            || admission.owner_authority_ref != risk_policy.owner_authority_ref
            || admission.risk_authority_ref != risk_policy.risk_authority_ref
            || admission.operator_authority_ref != risk_policy.operator_authority_ref
            || candidate_state.execution_admission_sha256.as_deref()
                != Some(sha256_ref(&admission_raw).as_str())
            || candidate_state.previous_state_sha256.as_deref()
                != Some(stage.base_state_sha256.as_str())
            || candidate_state.revision != stage.base_state.revision + 1
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_approval_publication",
            ));
        }
        write_same_or_new_run_file(
            &directory,
            LIVE_EXECUTION_ADMISSION_FILE,
            &admission_raw,
            "live_execution_admission",
        )?;
        fail_approval_publication_after(fail_after, ApprovalPublicationStep::Admission)?;
        write_live_run_state_recoverable(
            state,
            run_id,
            candidate_state,
            &stage.base_state,
            &stage.base_state_sha256,
            &receipt,
            fail_after,
        )?;
    } else if stage.admission.is_some() || stage.admission_state.is_some() {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_approval_publication",
        ));
    }
    directory
        .remove_file(stage.role.publication_stage_file())
        .map_err(|_| {
            product_error(
                ProductErrorKind::SourceUnavailable,
                "live_execution_approval_publication",
            )
        })?;
    Ok(())
}

#[expect(clippy::too_many_arguments)]
fn write_and_anchor_execution_control_artifact(
    state: &DashboardServerState,
    run_id: &str,
    run_revision: u64,
    manifest_sha256: &str,
    artifact_raw: &[u8],
    created_at_unix_ms: u64,
    artifact_file: &str,
    receipt_file: &str,
    directory: &cap_std::fs::Dir,
) -> Result<(), ProductError> {
    let previous = validate_workspace_anchor_head(state)?.ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_workspace_anchor_head",
        )
    })?;
    let request = LiveRunAnchorAppendRequest::new(
        state.live_run_audit_anchor.namespace()?,
        run_id,
        LiveRunAnchorRevision::new(run_revision, previous.workspace_revision + 1),
        sha256_ref(artifact_raw),
        manifest_sha256.to_string(),
        Some(previous.sha256()),
        created_at_unix_ms,
    );
    let receipt = state.live_run_audit_anchor.append(&request)?;
    state
        .live_run_audit_anchor
        .validate_receipt(&receipt, &request)?;
    if state.live_run_audit_anchor.latest()?.as_ref() != Some(&receipt) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_control_receipt",
        ));
    }
    let receipt_raw = serde_json::to_vec_pretty(&receipt).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_execution_control_receipt",
        )
    })?;
    write_new_run_file(directory, artifact_file, artifact_raw)?;
    write_new_run_file(directory, receipt_file, &receipt_raw)?;
    publish_workspace_anchor_head(state, &receipt)
}

fn response(
    schema_version: &str,
    request_id: String,
    data: LiveRunCandidate,
) -> LiveRunCandidateResponse {
    let order_authorized = data.order_admission.status == "authorized_single_shot";
    let execution_adapter_send_attempted = data
        .execution_order
        .as_ref()
        .is_some_and(|order| order.actual_submission_attempted);
    let real_orders_submitted = data
        .execution_order
        .as_ref()
        .is_some_and(execution_order_has_confirmed_submission);
    let order_control_allowed = LiveRunGateState::from_environment().order_control
        && data.lifecycle == LiveRunCandidateLifecycle::MarketDataRunning
        && data
            .execution_order
            .as_ref()
            .is_some_and(|order| !order.terminal && order.client_order_id.is_some());
    LiveRunCandidateResponse {
        schema_version: schema_version.to_string(),
        contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
        request_id,
        data,
        runtime_gate_refs: LiveRunGateState::refs(),
        audit_anchor_config_refs: anchor_config_refs(),
        boundaries: LiveRunCandidateBoundaries::enforced(
            order_authorized,
            order_control_allowed,
            execution_adapter_send_attempted,
            real_orders_submitted,
        ),
    }
}

fn execution_order_has_confirmed_submission(order: &LiveExecutionOrderSnapshot) -> bool {
    order.actual_submission_attempted
        && ((order.status == "submission_requested" && order.cancel_attempted)
            || matches!(
                order.status.as_str(),
                "submitted"
                    | "accepted"
                    | "rejected"
                    | "expired"
                    | "partially_filled"
                    | "filled"
                    | "canceled"
            ))
}

fn create_live_run_candidate_with_symbols(
    state: &DashboardServerState,
    request: CreateLiveRunCandidateRequest,
    request_id: &str,
    now: u64,
    admission: LiveRunCreationAdmission,
    gates: LiveRunGateState,
    strategy_version: LiveRunStrategyVersionBinding<'_>,
) -> Result<LiveRunCandidate, ProductError> {
    validate_create_request(&request, &admission, &gates)?;
    validate_sha256_hash(
        "strategy_version_content_hash",
        strategy_version.content_hash,
    )?;
    validate_live_market_data_symbols(strategy_version.data_symbols)?;
    let _workspace_lock = acquire_live_run_mutation_lock(state)?;
    if let Some(pointer) = load_active_live_run_pointer(state)? {
        reconcile_exited_live_market_data_runtime(state, &pointer.run_id)?;
    }
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
        strategy_version_content_hash: strategy_version.content_hash.to_string(),
        data_symbols: strategy_version.data_symbols.to_vec(),
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

#[cfg(test)]
fn create_live_run_candidate(
    state: &DashboardServerState,
    request: CreateLiveRunCandidateRequest,
    request_id: &str,
    now: u64,
    admission: LiveRunCreationAdmission,
    gates: LiveRunGateState,
    strategy_version_content_hash: &str,
) -> Result<LiveRunCandidate, ProductError> {
    create_live_run_candidate_with_symbols(
        state,
        request,
        request_id,
        now,
        admission,
        gates,
        LiveRunStrategyVersionBinding {
            content_hash: strategy_version_content_hash,
            data_symbols: &["BTCUSDT.BINANCE".to_string()],
        },
    )
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

fn validate_execution_control_snapshot(
    result: &LiveExecutionControlSnapshot,
    request: &LiveExecutionControlRequestArtifact,
    request_raw: &[u8],
    manifest: &LiveRunCandidateManifest,
    admission: Option<&LiveExecutionAdmissionArtifact>,
    order: Option<&LiveExecutionOrderSnapshot>,
    cancel_venue_attempted: bool,
) -> Result<(), ProductError> {
    let admission = admission.ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_control_admission",
        )
    })?;
    let status_valid = execution_control_status_is_valid(result);
    let quantities_valid = match (
        result.original_quantity.as_deref(),
        result.filled_quantity.as_deref(),
        result.remaining_quantity.as_deref(),
    ) {
        (Some(original), Some(filled), Some(remaining)) => {
            let original = Decimal::from_str_exact(original).ok();
            let filled = Decimal::from_str_exact(filled).ok();
            let remaining = Decimal::from_str_exact(remaining).ok();
            let admitted = Decimal::from_str_exact(&admission.quantity).ok();
            matches!((original, filled, remaining, admitted), (Some(o), Some(f), Some(r), Some(a)) if o == a && f >= Decimal::ZERO && r >= Decimal::ZERO && f + r == o)
        }
        (None, None, None) => result.manual_review_required,
        _ => false,
    };
    let monotonic_with_runtime = order.is_some_and(|order| {
        let local_original = Decimal::from_str_exact(&order.original_quantity).ok();
        let local_filled = Decimal::from_str_exact(&order.filled_quantity).ok();
        match (
            local_original,
            local_filled,
            result.original_quantity.as_deref(),
            result.filled_quantity.as_deref(),
        ) {
            (Some(local_original), Some(local_filled), Some(original), Some(filled)) => {
                Decimal::from_str_exact(original).ok() == Some(local_original)
                    && Decimal::from_str_exact(filled).is_ok_and(|value| value >= local_filled)
            }
            (_, _, None, None) => result.manual_review_required,
            _ => false,
        }
    });
    let has_exchange_identity = result.original_quantity.is_some();
    let venue_identity_valid = order.is_some_and(|order| {
        order
            .venue_order_id
            .as_ref()
            .is_none_or(|venue_order_id| result.venue_order_id.as_ref() == Some(venue_order_id))
    });
    if result.schema_version != LIVE_EXECUTION_CONTROL_RESULT_SCHEMA_VERSION
        || request.schema_version != LIVE_EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION
        || result.request_sha256 != sha256_ref(request_raw)
        || result.request_id != request.request_id
        || result.action != request.action
        || result.run_id != request.run_id
        || result.admission_id != request.admission_id
        || result.strategy_version_id != request.strategy_version_id
        || result.instrument_id != request.instrument_id
        || result.client_order_id != request.client_order_id
        || result.run_id != manifest.run_id
        || result.admission_id != admission.admission_id
        || result.strategy_version_id != admission.strategy_version_id
        || result.instrument_id != admission.instrument_id
        || !status_valid
        || result.cancel_attempted != cancel_venue_attempted
        || has_exchange_identity != result.venue_order_id.is_some()
        || has_exchange_identity != result.exchange_order_status.is_some()
        || !venue_identity_valid
        || result.automatic_retry_attempted
        || !quantities_valid
        || !monotonic_with_runtime
        || result.completed_at_unix_ms < request.requested_at_unix_ms
        || result.completed_at_unix_ms > unix_time_ms()
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_control_result",
        ));
    }
    Ok(())
}

fn execution_control_status_is_valid(result: &LiveExecutionControlSnapshot) -> bool {
    match (result.action.as_str(), result.status.as_str()) {
        ("reconcile", "reconciled") => {
            result.query_attempted
                && !result.cancel_attempted
                && !result.cancel_confirmed
                && !result.manual_review_required
                && result.error_code.is_none()
        }
        ("reconcile", "unknown_manual_review") => {
            !result.cancel_attempted
                && !result.cancel_confirmed
                && result.manual_review_required
                && result.error_code.is_some()
        }
        ("cancel", "cancel_confirmed") => {
            result.query_attempted
                && result.cancel_attempted
                && result.cancel_confirmed
                && !result.manual_review_required
                && result.error_code.is_none()
                && result.exchange_order_status.as_deref() == Some("canceled")
        }
        ("cancel", "cancel_sent_readback_pending") => {
            result.query_attempted
                && result.cancel_attempted
                && !result.cancel_confirmed
                && result.manual_review_required
                && result.error_code.is_none()
        }
        ("cancel", "cancel_not_required_terminal_or_pending") => {
            result.query_attempted
                && !result.cancel_attempted
                && !result.cancel_confirmed
                && !result.manual_review_required
                && result.error_code.is_none()
                && matches!(
                    result.exchange_order_status.as_deref(),
                    Some(
                        "filled"
                            | "canceled"
                            | "expired"
                            | "rejected"
                            | "pending_cancel"
                            | "pending_update"
                    )
                )
        }
        ("cancel", "unknown_manual_review") => {
            !result.cancel_confirmed && result.manual_review_required && result.error_code.is_some()
        }
        _ => false,
    }
}

#[expect(clippy::too_many_arguments)]
fn validate_execution_control_request_artifact(
    request: &LiveExecutionControlRequestArtifact,
    request_raw: &[u8],
    receipt_file: Option<&Path>,
    expected_action: &str,
    manifest: &LiveRunCandidateManifest,
    manifest_sha256: &str,
    admission: &LiveExecutionAdmissionArtifact,
    order: &LiveExecutionOrderSnapshot,
    order_raw: &[u8],
) -> Result<(), ProductError> {
    let roles_valid = match expected_action {
        "reconcile" => request.owner_confirmed && !request.operator_confirmed,
        "cancel" => request.owner_confirmed && request.operator_confirmed,
        _ => false,
    };
    if request.schema_version != LIVE_EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION
        || request.action != expected_action
        || request.run_id != manifest.run_id
        || request.admission_id != admission.admission_id
        || request.strategy_version_id != admission.strategy_version_id
        || request.instrument_id != admission.instrument_id
        || request.client_order_id != order.client_order_id.as_deref().unwrap_or_default()
        || request.source_order_state_sha256 != sha256_ref(order_raw)
        || !roles_valid
        || request.requested_at_unix_ms < manifest.created_at_unix_ms
        || request.expires_at_unix_ms <= request.requested_at_unix_ms
        || request.expires_at_unix_ms > request.requested_at_unix_ms.saturating_add(5 * 60 * 1_000)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_control_request",
        ));
    }
    if let Some(receipt_file) = receipt_file {
        validate_execution_control_receipt(
            receipt_file,
            request_raw,
            &manifest.run_id,
            manifest_sha256,
            request.requested_at_unix_ms,
        )?;
    }
    Ok(())
}

fn validate_execution_control_receipt(
    receipt_file: &Path,
    artifact_raw: &[u8],
    run_id: &str,
    expected_commit_sha256: &str,
    created_at_unix_ms: u64,
) -> Result<(), ProductError> {
    let receipt_raw = read_live_run_artifact_bytes(receipt_file, "live_execution_control_receipt")?;
    let receipt: LiveRunAnchorReceipt = serde_json::from_slice(&receipt_raw).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_control_receipt",
        )
    })?;
    if receipt.schema_version != LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION
        || receipt.run_id != run_id
        || receipt.state_sha256 != sha256_ref(artifact_raw)
        || receipt.commit_sha256 != expected_commit_sha256
        || receipt.anchored_at_unix_ms < created_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_control_receipt",
        ));
    }
    Ok(())
}

fn validate_execution_cancel_approval_artifacts(
    root: &Path,
    manifest: &LiveRunCandidateManifest,
    manifest_sha256: &str,
    admission: &LiveExecutionAdmissionArtifact,
    order: &LiveExecutionOrderSnapshot,
    order_raw: &[u8],
    cancel_request: Option<(&LiveExecutionControlRequestArtifact, &[u8])>,
) -> Result<(), ProductError> {
    let owner = read_optional_artifact_with_raw::<LiveExecutionCancelApprovalArtifact>(
        &root.join(LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE),
        "live_execution_cancel_owner_approval",
    )?;
    let operator = read_optional_artifact_with_raw::<LiveExecutionCancelApprovalArtifact>(
        &root.join(LIVE_EXECUTION_CANCEL_OPERATOR_APPROVAL_FILE),
        "live_execution_cancel_operator_approval",
    )?;
    let Some((owner, owner_raw)) = owner else {
        if operator.is_some() || cancel_request.is_some() {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_approval",
            ));
        }
        return Ok(());
    };
    validate_execution_cancel_approval(
        &owner,
        LiveExecutionApprovalRole::Owner,
        manifest,
        manifest_sha256,
        admission,
        order,
        order_raw,
    )?;
    validate_execution_control_receipt(
        &root.join(LIVE_EXECUTION_CANCEL_OWNER_RECEIPT_FILE),
        &owner_raw,
        &manifest.run_id,
        manifest_sha256,
        owner.approved_at_unix_ms,
    )?;
    if let Some((operator, operator_raw)) = operator {
        validate_execution_cancel_approval(
            &operator,
            LiveExecutionApprovalRole::Operator,
            manifest,
            manifest_sha256,
            admission,
            order,
            order_raw,
        )?;
        if operator.proposal_sha256 != owner.proposal_sha256
            || operator.client_order_id != owner.client_order_id
            || operator.source_order_state_sha256 != owner.source_order_state_sha256
            || operator.expires_at_unix_ms != owner.expires_at_unix_ms
            || operator.approved_at_unix_ms < owner.approved_at_unix_ms
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_approval",
            ));
        }
        validate_execution_control_receipt(
            &root.join(LIVE_EXECUTION_CANCEL_OPERATOR_RECEIPT_FILE),
            &operator_raw,
            &manifest.run_id,
            manifest_sha256,
            operator.approved_at_unix_ms,
        )?;
        let request = cancel_request.ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_request",
            )
        })?;
        if request.0.request_id.is_empty()
            || request.0.client_order_id != operator.client_order_id
            || request.0.source_order_state_sha256 != operator.source_order_state_sha256
            || request.0.expires_at_unix_ms != operator.expires_at_unix_ms
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_request",
            ));
        }
    } else if cancel_request.is_some() {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_request",
        ));
    }
    Ok(())
}

fn validate_execution_cancel_approval(
    approval: &LiveExecutionCancelApprovalArtifact,
    role: LiveExecutionApprovalRole,
    manifest: &LiveRunCandidateManifest,
    manifest_sha256: &str,
    admission: &LiveExecutionAdmissionArtifact,
    order: &LiveExecutionOrderSnapshot,
    order_raw: &[u8],
) -> Result<(), ProductError> {
    let authority = match role {
        LiveExecutionApprovalRole::Owner => &admission.owner_authority_ref,
        LiveExecutionApprovalRole::Operator => &admission.operator_authority_ref,
        LiveExecutionApprovalRole::Risk => {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_approval",
            ));
        }
    };
    if approval.schema_version != "ntpro.s3.live_execution_cancel_approval.v1"
        || approval.role != role
        || approval.source_manifest_sha256 != manifest_sha256
        || approval.run_id != manifest.run_id
        || approval.admission_id != admission.admission_id
        || approval.strategy_version_id != admission.strategy_version_id
        || approval.instrument_id != admission.instrument_id
        || approval.client_order_id != order.client_order_id.as_deref().unwrap_or_default()
        || approval.source_order_state_sha256 != sha256_ref(order_raw)
        || approval.authority_ref != *authority
        || approval.approved_at_unix_ms < manifest.created_at_unix_ms
        || approval.approved_at_unix_ms > approval.expires_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_approval",
        ));
    }
    Ok(())
}

#[cfg(test)]
fn run_live_candidate_action(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveRunCandidateActionRequest,
) -> Result<LiveRunCandidate, ProductError> {
    run_live_candidate_action_with_preflight(state, path_run_id, request, |manifest| {
        evaluate_current_live_candidate_preflight(state, manifest)
    })
}

fn run_live_candidate_action_api_guarded(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveRunCandidateActionRequest,
) -> Result<LiveRunCandidate, ProductError> {
    run_live_candidate_action_with_preflight_policy(state, path_run_id, request, true, |manifest| {
        evaluate_current_live_candidate_preflight(state, manifest)
    })
}

#[cfg(test)]
fn run_live_candidate_action_with_preflight<F>(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveRunCandidateActionRequest,
    preflight_evaluator: F,
) -> Result<LiveRunCandidate, ProductError>
where
    F: FnOnce(&LiveRunCandidateManifest) -> Result<LiveRunPreflightAdmission, ProductError>,
{
    run_live_candidate_action_with_preflight_policy(
        state,
        path_run_id,
        request,
        false,
        preflight_evaluator,
    )
}

fn run_live_candidate_action_with_preflight_policy<F>(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &LiveRunCandidateActionRequest,
    revalidate_start: bool,
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
    reconcile_exited_live_market_data_runtime(state, path_run_id)?;
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
                    execution_admission_sha256: None,
                    execution_runtime_config_sha256: None,
                    stop_sha256: None,
                    updated_at_unix_ms: preflight.evaluated_at_unix_ms,
                },
            )?;
        }
        LiveRunCandidateAction::StartMarketData | LiveRunCandidateAction::StartExecution
            if current.lifecycle == LiveRunCandidateLifecycle::PreflightReady =>
        {
            let starting_at = unix_time_ms();
            if request.action == LiveRunCandidateAction::StartMarketData
                && current_state.execution_admission_sha256.is_some()
            {
                return Err(product_error(
                    ProductErrorKind::LiveConflict,
                    "live_execution_admission_requires_execution_start",
                ));
            }
            let execution_admission = if request.action == LiveRunCandidateAction::StartExecution {
                let root = canonical_live_run_root(state, false)?.join(path_run_id);
                let (admission, admission_raw) =
                    read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
                        &root.join(LIVE_EXECUTION_ADMISSION_FILE),
                        "live_execution_admission",
                    )?
                    .ok_or_else(|| {
                        product_error(
                            ProductErrorKind::BoundaryViolation,
                            "live_execution_admission",
                        )
                    })?;
                let (intent, intent_raw) =
                    read_optional_artifact_with_raw::<LiveStrategyOrderIntentArtifact>(
                        &root.join(LIVE_STRATEGY_INTENT_FILE),
                        "live_strategy_intent",
                    )?
                    .ok_or_else(|| {
                        product_error(ProductErrorKind::BoundaryViolation, "live_strategy_intent")
                    })?;
                let (sizing, sizing_raw) =
                    read_optional_artifact_with_raw::<LiveSizingDecisionArtifact>(
                        &root.join(LIVE_SIZING_DECISION_FILE),
                        "live_sizing_decision",
                    )?
                    .ok_or_else(|| {
                        product_error(ProductErrorKind::BoundaryViolation, "live_sizing_decision")
                    })?;
                validate_live_strategy_intent_artifact(&intent, &manifest)?;
                validate_execution_admission_artifact(
                    &manifest,
                    &manifest_sha256,
                    &admission,
                    &intent,
                    &intent_raw,
                    &sizing,
                    &sizing_raw,
                    current.lifecycle,
                )?;
                if admission.expires_at_unix_ms <= starting_at || admission.consumed {
                    return Err(product_error(
                        ProductErrorKind::BoundaryViolation,
                        "live_execution_admission_expiry",
                    ));
                }
                if current_state.execution_admission_sha256.as_deref()
                    != Some(sha256_ref(&admission_raw).as_str())
                {
                    return Err(product_error(
                        ProductErrorKind::BoundaryViolation,
                        "live_execution_admission_anchor",
                    ));
                }
                Some(admission)
            } else {
                if current_state.execution_admission_sha256.is_some() {
                    return Err(product_error(
                        ProductErrorKind::BoundaryViolation,
                        "live_execution_admission",
                    ));
                }
                None
            };
            if revalidate_start {
                let refreshed_preflight = preflight_evaluator(&manifest)?;
                if !refreshed_preflight.connected
                    || !refreshed_preflight.can_trade
                    || sorted_refs(refreshed_preflight.source_refs) != manifest.source_refs
                {
                    return Err(product_error(
                        ProductErrorKind::BoundaryViolation,
                        "live_execution_current_preflight",
                    ));
                }
            }
            let runtime_config_sha256 =
                write_live_node_config(state, &directory, &manifest, execution_admission.as_ref())?;
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
                    execution_admission_sha256: current_state.execution_admission_sha256,
                    execution_runtime_config_sha256: execution_admission
                        .as_ref()
                        .map(|_| runtime_config_sha256),
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
                        process_generation_at_claim: 0,
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
                execution_admission_sha256: starting_state.execution_admission_sha256,
                execution_runtime_config_sha256: starting_state.execution_runtime_config_sha256,
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
        LiveRunCandidateAction::ReconcileOrder
            if current.lifecycle == LiveRunCandidateLifecycle::MarketDataRunning =>
        {
            let (admission, _) = read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
                &canonical_live_run_root(state, false)?
                    .join(path_run_id)
                    .join(LIVE_EXECUTION_ADMISSION_FILE),
                "live_execution_admission",
            )?
            .ok_or_else(|| {
                product_error(
                    ProductErrorKind::BoundaryViolation,
                    "live_execution_reconcile_admission",
                )
            })?;
            let runtime_root = live_market_data_runtime_root(state, path_run_id)?;
            let (order, order_raw) = read_optional_artifact_with_raw::<LiveExecutionOrderSnapshot>(
                &runtime_root.join(LIVE_EXECUTION_ORDER_STATE_FILE),
                "live_execution_order_state",
            )?
            .ok_or_else(|| {
                product_error(
                    ProductErrorKind::BoundaryViolation,
                    "live_execution_reconcile_order",
                )
            })?;
            validate_execution_order_snapshot(&order, &admission)?;
            let client_order_id = order.client_order_id.ok_or_else(|| {
                product_error(
                    ProductErrorKind::BoundaryViolation,
                    "live_execution_reconcile_order",
                )
            })?;
            let candidate_root = canonical_live_run_root(state, false)?.join(path_run_id);
            if candidate_root
                .join(LIVE_EXECUTION_RECONCILE_REQUEST_FILE)
                .exists()
                || candidate_root
                    .join(LIVE_EXECUTION_RECONCILE_SOURCE_ORDER_FILE)
                    .exists()
                || candidate_root
                    .join(LIVE_EXECUTION_RECONCILE_RESULT_FILE)
                    .exists()
            {
                return Err(product_error(
                    ProductErrorKind::LiveConflict,
                    "live_execution_reconcile_request",
                ));
            }
            let now = unix_time_ms();
            let control = LiveExecutionControlRequestArtifact {
                schema_version: LIVE_EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION.to_string(),
                request_id: format!("reconcile-{now}"),
                action: "reconcile".to_string(),
                run_id: path_run_id.to_string(),
                admission_id: admission.admission_id,
                strategy_version_id: admission.strategy_version_id,
                instrument_id: admission.instrument_id,
                client_order_id,
                source_order_state_sha256: sha256_ref(&order_raw),
                owner_confirmed: true,
                operator_confirmed: false,
                requested_at_unix_ms: now,
                expires_at_unix_ms: now.saturating_add(5 * 60 * 1_000),
            };
            let control_raw = serde_json::to_vec_pretty(&control).map_err(|_| {
                product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_execution_reconcile_request",
                )
            })?;
            write_new_run_file(
                &directory,
                LIVE_EXECUTION_RECONCILE_SOURCE_ORDER_FILE,
                &order_raw,
            )?;
            write_and_anchor_execution_control_artifact(
                state,
                path_run_id,
                current_state.revision,
                &manifest_sha256,
                &control_raw,
                now,
                LIVE_EXECUTION_RECONCILE_REQUEST_FILE,
                LIVE_EXECUTION_RECONCILE_RECEIPT_FILE,
                &directory,
            )?;
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
                execution_order_sha256: None,
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
                    execution_admission_sha256: current_state.execution_admission_sha256,
                    execution_runtime_config_sha256: current_state.execution_runtime_config_sha256,
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
                    execution_admission_sha256: current_state.execution_admission_sha256,
                    execution_runtime_config_sha256: current_state.execution_runtime_config_sha256,
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
            let activity = load_live_execution_stop_activity(state, path_run_id)?;
            let stop = LiveRunStopArtifact {
                schema_version: LIVE_RUN_STOP_SCHEMA_VERSION.to_string(),
                run_id: path_run_id.to_string(),
                source_manifest_sha256: manifest_sha256.clone(),
                source_preflight_sha256: stopping_state.preflight_sha256.clone(),
                stopped_at_unix_ms: unix_time_ms(),
                manual_stop: true,
                order_endpoint_access_attempted: activity.endpoint_access_attempted,
                execution_adapter_send_attempted: activity.adapter_send_attempted,
                real_orders_submitted: activity.real_order_submitted,
                execution_order_sha256: activity.order_sha256,
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
                execution_admission_sha256: stopping_state.execution_admission_sha256,
                execution_runtime_config_sha256: stopping_state.execution_runtime_config_sha256,
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
        LiveRunCandidateLifecycle::Starting
            | LiveRunCandidateLifecycle::MarketDataRunning
            | LiveRunCandidateLifecycle::Failed
    ) {
        return Err(product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_cleanup",
        ));
    }
    let failed = if current.lifecycle == LiveRunCandidateLifecycle::Failed {
        current
    } else {
        let activity = load_live_execution_stop_activity(state, run_id)?;
        let failed_at = unix_time_ms();
        let candidate_root = canonical_live_run_root(state, false)?.join(run_id);
        let directory = open_absolute_directory_nofollow(&candidate_root)?;
        let (stop, stop_raw) = load_or_create_failed_stop_artifact(FailedStopArtifactContext {
            candidate_root: &candidate_root,
            directory: &directory,
            run_id,
            manifest_sha256,
            preflight_sha256: current.preflight_sha256.as_deref(),
            current_updated_at_unix_ms: current.updated_at_unix_ms,
            failed_at_unix_ms: failed_at,
            activity: &activity,
        })?;
        let failed = LiveRunCandidateState {
            schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
            run_id: run_id.to_string(),
            source_manifest_sha256: manifest_sha256.to_string(),
            revision: current.revision + 1,
            previous_state_sha256: Some(sha256_ref(&current_raw)),
            lifecycle: LiveRunCandidateLifecycle::Failed,
            preflight_sha256: current.preflight_sha256,
            execution_admission_sha256: current.execution_admission_sha256,
            execution_runtime_config_sha256: current.execution_runtime_config_sha256,
            stop_sha256: Some(sha256_ref(&stop_raw)),
            updated_at_unix_ms: stop.stopped_at_unix_ms,
        };
        write_live_run_state(state, run_id, &failed)?;
        failed
    };
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
                    completed_at_unix_ms: failed.updated_at_unix_ms,
                },
            )
            .map_err(|_| {
                product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_runtime_cleanup",
                )
            })?;
    }
    release_active_live_run_candidate_if_present(state, run_id)
}

#[derive(Clone, Copy)]
struct FailedStopArtifactContext<'a> {
    candidate_root: &'a Path,
    directory: &'a cap_std::fs::Dir,
    run_id: &'a str,
    manifest_sha256: &'a str,
    preflight_sha256: Option<&'a str>,
    current_updated_at_unix_ms: u64,
    failed_at_unix_ms: u64,
    activity: &'a LiveExecutionStopActivity,
}

fn load_or_create_failed_stop_artifact(
    context: FailedStopArtifactContext<'_>,
) -> Result<(LiveRunStopArtifact, Vec<u8>), ProductError> {
    let FailedStopArtifactContext {
        candidate_root,
        directory,
        run_id,
        manifest_sha256,
        preflight_sha256,
        current_updated_at_unix_ms,
        failed_at_unix_ms,
        activity,
    } = context;
    if let Some((stop, raw)) = read_optional_artifact_with_raw::<LiveRunStopArtifact>(
        &candidate_root.join("stop.json"),
        "live_runtime_cleanup",
    )? {
        if stop.schema_version != LIVE_RUN_STOP_SCHEMA_VERSION
            || stop.run_id != run_id
            || stop.source_manifest_sha256 != manifest_sha256
            || stop.source_preflight_sha256.as_deref() != preflight_sha256
            || stop.stopped_at_unix_ms < current_updated_at_unix_ms
            || stop.stopped_at_unix_ms
                > failed_at_unix_ms.saturating_add(LIVE_RUN_STOP_CLOCK_SKEW_MS)
            || stop.manual_stop
            || stop.order_endpoint_access_attempted != activity.endpoint_access_attempted
            || stop.execution_adapter_send_attempted != activity.adapter_send_attempted
            || stop.real_orders_submitted != activity.real_order_submitted
            || stop.execution_order_sha256 != activity.order_sha256
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_runtime_cleanup",
            ));
        }
        return Ok((stop, raw));
    }

    let stop = LiveRunStopArtifact {
        schema_version: LIVE_RUN_STOP_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        source_manifest_sha256: manifest_sha256.to_string(),
        source_preflight_sha256: preflight_sha256.map(str::to_string),
        stopped_at_unix_ms: failed_at_unix_ms,
        manual_stop: false,
        order_endpoint_access_attempted: activity.endpoint_access_attempted,
        execution_adapter_send_attempted: activity.adapter_send_attempted,
        real_orders_submitted: activity.real_order_submitted,
        execution_order_sha256: activity.order_sha256.clone(),
    };
    let raw = serde_json::to_vec_pretty(&stop).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_cleanup",
        )
    })?;
    write_new_run_file(directory, "stop.json", &raw)?;
    Ok((stop, raw))
}

fn reconcile_exited_live_market_data_runtime(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<(), ProductError> {
    let (_, manifest_raw) = load_live_run_manifest(state, run_id)?;
    let manifest_sha256 = sha256_ref(&manifest_raw);
    let (candidate_state, _) = load_live_run_state(state, run_id, &manifest_sha256)?;
    if matches!(
        candidate_state.lifecycle,
        LiveRunCandidateLifecycle::Created | LiveRunCandidateLifecycle::PreflightReady
    ) {
        return Ok(());
    }
    ensure_active_live_run_pointer_matches(
        state,
        run_id,
        &manifest_sha256,
        candidate_state.lifecycle,
    )?;
    let store = SupervisorRegistryStore::new(&state.registry_path);
    match candidate_state.lifecycle {
        LiveRunCandidateLifecycle::Starting => {
            reconcile_starting_live_market_data_runtime(state, run_id, &manifest_sha256, &store)?;
        }
        LiveRunCandidateLifecycle::MarketDataRunning => {
            let record = store.refresh_process_state(run_id).map_err(|_| {
                product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_runtime_reconcile",
                )
            })?;
            if matches!(
                record.process.state,
                SupervisorProcessState::Stopped | SupervisorProcessState::Stale
            ) {
                transition_live_market_data_runtime_failed(
                    state,
                    run_id,
                    &manifest_sha256,
                    Some(&store),
                )?;
            } else if record.process.state == SupervisorProcessState::Unknown {
                return Err(product_error(
                    ProductErrorKind::LiveExecutionFailed,
                    "live_runtime_reconcile",
                ));
            }
        }
        LiveRunCandidateLifecycle::Stopping | LiveRunCandidateLifecycle::Stopped => {
            complete_live_market_data_runtime_stop(state, run_id, &manifest_sha256, &store)?;
        }
        LiveRunCandidateLifecycle::Failed => {
            if load_active_live_run_pointer(state)?.is_some() {
                transition_live_market_data_runtime_failed(
                    state,
                    run_id,
                    &manifest_sha256,
                    Some(&store),
                )?;
            }
        }
        LiveRunCandidateLifecycle::Created | LiveRunCandidateLifecycle::PreflightReady => {}
    }
    Ok(())
}

fn ensure_active_live_run_pointer_matches(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
    lifecycle: LiveRunCandidateLifecycle,
) -> Result<(), ProductError> {
    match load_active_live_run_pointer(state)? {
        Some(pointer)
            if pointer.run_id != run_id || pointer.source_manifest_sha256 != manifest_sha256 =>
        {
            Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "active_live_run_candidate",
            ))
        }
        None if matches!(
            lifecycle,
            LiveRunCandidateLifecycle::Starting
                | LiveRunCandidateLifecycle::MarketDataRunning
                | LiveRunCandidateLifecycle::Stopping
        ) =>
        {
            Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "active_live_run_candidate",
            ))
        }
        _ => Ok(()),
    }
}

fn reconcile_starting_live_market_data_runtime(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
    store: &SupervisorRegistryStore,
) -> Result<(), ProductError> {
    let registry = store.load().map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_reconcile",
        )
    })?;
    let Some(record) = registry.nodes.get(run_id) else {
        return transition_live_market_data_runtime_failed(state, run_id, manifest_sha256, None);
    };
    let Some(ownership) = record.run_ownership.get(run_id) else {
        if record.process.state == SupervisorProcessState::Running
            || !record.run_ownership.is_empty()
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_runtime_ownership",
            ));
        }
        store.remove_node(run_id).map_err(|_| {
            product_error(
                ProductErrorKind::LiveExecutionFailed,
                "live_runtime_cleanup",
            )
        })?;
        return transition_live_market_data_runtime_failed(state, run_id, manifest_sha256, None);
    };
    if ownership.manifest_sha256 != manifest_sha256 || ownership.terminal.is_some() {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_runtime_ownership",
        ));
    }
    let refreshed = store.refresh_process_state(run_id).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_reconcile",
        )
    })?;
    if refreshed.process.state == SupervisorProcessState::Running {
        stop_owned_live_market_data_process(store, run_id, manifest_sha256)?;
    } else if refreshed.process.state == SupervisorProcessState::Unknown {
        return Err(product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_reconcile",
        ));
    }
    transition_live_market_data_runtime_failed(state, run_id, manifest_sha256, Some(store))
}

fn stop_owned_live_market_data_process(
    store: &SupervisorRegistryStore,
    run_id: &str,
    manifest_sha256: &str,
) -> Result<(), ProductError> {
    store
        .stop_node_process_for_run(
            &StopNodeRequest {
                node_id: run_id.to_string(),
                stop_timeout: Duration::from_millis(super::super::DASHBOARD_ACTION_TIMEOUT_MS),
            },
            run_id,
            manifest_sha256,
        )
        .map(|_| ())
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_runtime_stop"))
}

fn complete_live_market_data_runtime_stop(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
    store: &SupervisorRegistryStore,
) -> Result<(), ProductError> {
    let (current, current_raw) = load_live_run_state(state, run_id, manifest_sha256)?;
    if current.lifecycle == LiveRunCandidateLifecycle::Stopped && current.revision != 5 {
        return release_active_live_run_candidate_if_present(state, run_id);
    }
    let registry = store.load().map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_reconcile",
        )
    })?;
    let record = registry.nodes.get(run_id).ok_or_else(|| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_ownership",
        )
    })?;
    let ownership = record.run_ownership.get(run_id).ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_runtime_ownership",
        )
    })?;
    if ownership.manifest_sha256 != manifest_sha256 {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_runtime_ownership",
        ));
    }
    let refreshed = store.refresh_process_state(run_id).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_reconcile",
        )
    })?;
    if refreshed.process.state == SupervisorProcessState::Running {
        stop_owned_live_market_data_process(store, run_id, manifest_sha256)?;
    } else if refreshed.process.state == SupervisorProcessState::Unknown {
        return Err(product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_runtime_reconcile",
        ));
    }
    if current.lifecycle == LiveRunCandidateLifecycle::Stopping {
        complete_stopping_live_run_state(state, run_id, manifest_sha256, &current, &current_raw)?;
    }
    let (stopped, stopped_raw) = load_live_run_state(state, run_id, manifest_sha256)?;
    store
        .anchor_run_terminal(
            run_id,
            run_id,
            manifest_sha256,
            SupervisorRunTerminalAnchor {
                lifecycle: "stopped".to_string(),
                terminal_state_sha256: sha256_ref(&stopped_raw),
                completed_at_unix_ms: stopped.updated_at_unix_ms,
            },
        )
        .map_err(|_| {
            product_error(
                ProductErrorKind::LiveExecutionFailed,
                "live_runtime_terminal",
            )
        })?;
    release_active_live_run_candidate_if_present(state, run_id)
}

fn complete_stopping_live_run_state(
    state: &DashboardServerState,
    run_id: &str,
    manifest_sha256: &str,
    current: &LiveRunCandidateState,
    current_raw: &[u8],
) -> Result<(), ProductError> {
    let root = canonical_live_run_root(state, false)?.join(run_id);
    let directory = open_absolute_directory_nofollow(&root)?;
    let (stop, stop_raw) = if let Some(existing) = read_optional_artifact_with_raw::<
        LiveRunStopArtifact,
    >(&root.join("stop.json"), "live_stop")?
    {
        existing
    } else {
        let activity = load_live_execution_stop_activity(state, run_id)?;
        let stop = LiveRunStopArtifact {
            schema_version: LIVE_RUN_STOP_SCHEMA_VERSION.to_string(),
            run_id: run_id.to_string(),
            source_manifest_sha256: manifest_sha256.to_string(),
            source_preflight_sha256: current.preflight_sha256.clone(),
            stopped_at_unix_ms: unix_time_ms(),
            manual_stop: true,
            order_endpoint_access_attempted: activity.endpoint_access_attempted,
            execution_adapter_send_attempted: activity.adapter_send_attempted,
            real_orders_submitted: activity.real_order_submitted,
            execution_order_sha256: activity.order_sha256,
        };
        let raw = serde_json::to_vec_pretty(&stop)
            .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_stop"))?;
        write_new_run_file(&directory, "stop.json", &raw)?;
        (stop, raw)
    };
    let (manifest, _) = load_live_run_manifest(state, run_id)?;
    let preflight = read_optional_artifact_with_raw::<LiveRunPreflightArtifact>(
        &root.join("preflight.json"),
        "live_preflight",
    )?;
    validate_action_artifacts(
        &manifest,
        manifest_sha256,
        LiveRunCandidateLifecycle::Stopped,
        preflight.as_ref().map(|(value, _)| value),
        Some(&stop),
    )?;
    write_live_run_state(
        state,
        run_id,
        &LiveRunCandidateState {
            schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
            run_id: run_id.to_string(),
            source_manifest_sha256: manifest_sha256.to_string(),
            revision: current.revision + 1,
            previous_state_sha256: Some(sha256_ref(current_raw)),
            lifecycle: LiveRunCandidateLifecycle::Stopped,
            preflight_sha256: current.preflight_sha256.clone(),
            execution_admission_sha256: current.execution_admission_sha256.clone(),
            execution_runtime_config_sha256: current.execution_runtime_config_sha256.clone(),
            stop_sha256: Some(sha256_ref(&stop_raw)),
            updated_at_unix_ms: stop.stopped_at_unix_ms,
        },
    )
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
        || version.data_symbols() != manifest.data_symbols
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
        sizing: admission.sizing.clone(),
    };
    let raw = serde_json::to_vec_pretty(&artifact)
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_preflight"))?;
    write_new_run_file(directory, "preflight.json", &raw)?;
    Ok(raw)
}

#[cfg(test)]
fn write_live_market_data_node_config(
    _state: &DashboardServerState,
    directory: &cap_std::fs::Dir,
    manifest: &LiveRunCandidateManifest,
) -> Result<(), ProductError> {
    write_live_node_config(_state, directory, manifest, None).map(|_| ())
}

fn write_live_node_config(
    state: &DashboardServerState,
    directory: &cap_std::fs::Dir,
    manifest: &LiveRunCandidateManifest,
    execution: Option<&LiveExecutionAdmissionArtifact>,
) -> Result<String, ProductError> {
    validate_live_market_data_symbols(&manifest.data_symbols)?;
    let symbols = manifest
        .data_symbols
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let run_id = &manifest.run_id;
    let runtime_artifact_root = live_market_data_runtime_root(state, run_id)?;
    let control_artifact_root = canonical_live_run_root(state, false)?.join(run_id);
    let execution_admission_sha256 = execution
        .map(|_| {
            read_run_file_from_directory(
                directory,
                LIVE_EXECUTION_ADMISSION_FILE,
                "live_execution_admission",
            )
            .map(|raw| sha256_ref(&raw))
        })
        .transpose()?;
    let execution_section = execution.map_or_else(String::new, |admission| {
        format!(
            "\n[live_execution]\n\
             schema_version = \"ntpro.s3.live_execution_node.v2\"\n\
             source_manifest_sha256 = \"{}\"\n\
             execution_admission_sha256 = \"{}\"\n\
             sizing_decision_sha256 = \"{}\"\n\
             runtime_artifact_root = \"{}\"\n\
             control_artifact_root = \"{}\"\n\
             risk_policy_ref = \"{}\"\n\
             owner_authority_ref = \"{}\"\n\
             risk_authority_ref = \"{}\"\n\
             operator_authority_ref = \"{}\"\n\
             admission_id = \"{}\"\n\
             source_demo_run_id = \"{}\"\n\
             strategy_intent_id = \"{}\"\n\
             strategy_intent_sha256 = \"{}\"\n\
             strategy_version_id = \"{}\"\n\
             account_id = \"BINANCE-001\"\n\
             instrument_id = \"{}\"\n\
             side = \"{}\"\n\
             order_type = \"LIMIT\"\n\
             time_in_force = \"GTC\"\n\
             price = \"{}\"\n\
             source_quantity = \"{}\"\n\
             quantity = \"{}\"\n\
             max_notional = \"{}\"\n\
             risk_policy_max_notional = \"{}\"\n\
             expires_at_unix_ms = {}\n\
             api_key_env = \"NTPRO_BINANCE_LIVE_API_KEY\"\n\
             api_secret_env = \"NTPRO_BINANCE_LIVE_API_SECRET\"\n\
             owner_confirmed = true\n\
             risk_confirmed = true\n\
             operator_confirmed = true\n\
             kill_switch_active = false\n\
             single_shot = true\n\
             cancel_order_allowed = false\n\
             replace_order_allowed = false\n\
             automatic_retry_allowed = false\n\
             automatic_recovery_allowed = false\n",
            admission.source_manifest_sha256,
            execution_admission_sha256.as_deref().unwrap_or_default(),
            admission.sizing_decision_sha256,
            runtime_artifact_root.display(),
            control_artifact_root.display(),
            admission.risk_policy_ref,
            admission.owner_authority_ref,
            admission.risk_authority_ref,
            admission.operator_authority_ref,
            admission.admission_id,
            admission.source_demo_run_id,
            admission.strategy_intent_id,
            admission.strategy_intent_sha256,
            admission.strategy_version_id,
            admission.instrument_id,
            admission.side,
            admission.price,
            admission.source_quantity,
            admission.quantity,
            admission.max_notional,
            admission.risk_policy_max_notional,
            admission.expires_at_unix_ms,
        )
    });
    let raw = format!(
        "[live_market_data]\n\
         schema_version = \"ntpro.live_market_data_node.v1\"\n\
         mode = \"production-market-data\"\n\
         environment = \"live\"\n\
         node_id = \"{run_id}\"\n\
         trader_id = \"TRADER-001\"\n\
         venue = \"BINANCE\"\n\
         product_type = \"spot\"\n\
         symbols = [{symbols}]\n\
         api_key_env = \"NTPRO_BINANCE_LIVE_API_KEY\"\n\
         api_secret_env = \"NTPRO_BINANCE_LIVE_API_SECRET\"\n\
         execution_client_enabled = false\n\
         order_endpoint_access_allowed = false\n\
         order_submission_allowed = false\n\
         automatic_reconnect_allowed = false\n\
         {execution_section}\n\
         [shutdown]\n\
         mode = \"start-stop\"\n\
         post_stop_delay_secs = 0\n\
         connection_timeout_secs = 10\n\
         disconnection_timeout_secs = 10\n"
    );
    write_new_run_file(directory, LIVE_MARKET_DATA_NODE_CONFIG_FILE, raw.as_bytes())?;
    Ok(sha256_ref(raw.as_bytes()))
}

fn validate_live_market_data_symbols(symbols: &[String]) -> Result<(), ProductError> {
    let unique = symbols.iter().collect::<BTreeSet<_>>();
    if symbols.is_empty()
        || symbols.len() > 32
        || unique.len() != symbols.len()
        || symbols
            .iter()
            .any(|value| !value.ends_with(".BINANCE") || value.contains(['"', '\n', '\r']))
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_market_data_symbols",
        ));
    }
    Ok(())
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

fn load_live_execution_stop_activity(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<LiveExecutionStopActivity, ProductError> {
    let runtime_root = live_market_data_runtime_root(state, run_id)?;
    let Some((order, raw)) = read_optional_artifact_with_raw::<LiveExecutionOrderSnapshot>(
        &runtime_root.join(LIVE_EXECUTION_ORDER_STATE_FILE),
        "live_execution_order_state",
    )?
    else {
        return Ok(LiveExecutionStopActivity {
            endpoint_access_attempted: false,
            adapter_send_attempted: false,
            real_order_submitted: false,
            order_sha256: None,
        });
    };
    let candidate_root = canonical_live_run_root(state, false)?.join(run_id);
    let (admission, _) = read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
        &candidate_root.join(LIVE_EXECUTION_ADMISSION_FILE),
        "live_execution_admission",
    )?
    .ok_or_else(|| {
        product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_order_state",
        )
    })?;
    validate_execution_order_snapshot(&order, &admission)?;
    let real_order_submitted = order.actual_submission_attempted && order.status != "denied";
    Ok(LiveExecutionStopActivity {
        endpoint_access_attempted: order.actual_submission_attempted,
        adapter_send_attempted: order.actual_submission_attempted,
        real_order_submitted,
        order_sha256: Some(sha256_ref(&raw)),
    })
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
    let execution_admission = read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
        &root.join(LIVE_EXECUTION_ADMISSION_FILE),
        "live_execution_admission",
    )?;
    let strategy_intent = read_optional_artifact_with_raw::<LiveStrategyOrderIntentArtifact>(
        &root.join(LIVE_STRATEGY_INTENT_FILE),
        "live_strategy_intent",
    )?;
    let sizing_decision = read_optional_artifact_with_raw::<LiveSizingDecisionArtifact>(
        &root.join(LIVE_SIZING_DECISION_FILE),
        "live_sizing_decision",
    )?;
    let execution_approvals = load_live_execution_approvals(&root)?;
    validate_action_artifacts(
        &manifest,
        &manifest_sha256,
        candidate_state.lifecycle,
        preflight.as_ref().map(|(value, _)| value),
        stop.as_ref().map(|(value, _)| value),
    )?;
    validate_live_run_state(
        &manifest,
        candidate_state,
        preflight.as_ref(),
        execution_admission.as_ref(),
        stop.as_ref(),
    )?;
    if let Some((intent, _)) = &strategy_intent {
        validate_live_strategy_intent_artifact(intent, &manifest)?;
    }
    if execution_admission.is_some() && (strategy_intent.is_none() || sizing_decision.is_none())
        || !execution_approvals.is_empty()
            && (strategy_intent.is_none() || sizing_decision.is_none())
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_strategy_intent",
        ));
    }
    if let Some((admission, _)) = &execution_admission {
        let (intent, intent_raw) = strategy_intent.as_ref().ok_or_else(|| {
            product_error(ProductErrorKind::BoundaryViolation, "live_strategy_intent")
        })?;
        let (sizing, sizing_raw) = sizing_decision.as_ref().ok_or_else(|| {
            product_error(ProductErrorKind::BoundaryViolation, "live_sizing_decision")
        })?;
        validate_execution_admission_artifact(
            &manifest,
            &manifest_sha256,
            admission,
            intent,
            intent_raw,
            sizing,
            sizing_raw,
            candidate_state.lifecycle,
        )?;
    }
    validate_live_execution_approval_set(
        &execution_approvals,
        &manifest,
        &manifest_sha256,
        strategy_intent.as_ref().map(|(_, raw)| raw.as_slice()),
        sizing_decision.as_ref().map(|(_, raw)| raw.as_slice()),
        execution_admission.as_ref().map(|(value, _)| value),
    )?;
    let execution_order_path = mvp_workspace_root(&state.registry_path)?
        .join("artifacts/live-market-data-runtime")
        .join(run_id)
        .join(LIVE_EXECUTION_ORDER_STATE_FILE);
    let mut execution_order = read_optional_artifact_with_raw::<LiveExecutionOrderSnapshot>(
        &execution_order_path,
        "live_execution_order_state",
    )?;
    if let Some((order, _)) = &execution_order {
        let admission = execution_admission
            .as_ref()
            .map(|(value, _)| value)
            .ok_or_else(|| {
                product_error(
                    ProductErrorKind::BoundaryViolation,
                    "live_execution_order_state",
                )
            })?;
        validate_execution_order_snapshot(order, admission)?;
        validate_execution_order_runtime_context(&root, candidate_state.lifecycle)?;
    }
    let reconcile_request = read_optional_artifact_with_raw::<LiveExecutionControlRequestArtifact>(
        &root.join(LIVE_EXECUTION_RECONCILE_REQUEST_FILE),
        "live_execution_reconcile_request",
    )?;
    let cancel_request = read_optional_artifact_with_raw::<LiveExecutionControlRequestArtifact>(
        &root.join(LIVE_EXECUTION_CANCEL_REQUEST_FILE),
        "live_execution_cancel_request",
    )?;
    let reconcile_source_order = read_optional_artifact_with_raw::<LiveExecutionOrderSnapshot>(
        &root.join(LIVE_EXECUTION_RECONCILE_SOURCE_ORDER_FILE),
        "live_execution_reconcile_source_order",
    )?;
    let cancel_source_order = read_optional_artifact_with_raw::<LiveExecutionOrderSnapshot>(
        &root.join(LIVE_EXECUTION_CANCEL_SOURCE_ORDER_FILE),
        "live_execution_cancel_source_order",
    )?;
    let cancel_approval_present = root
        .join(LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE)
        .exists()
        || root
            .join(LIVE_EXECUTION_CANCEL_OPERATOR_APPROVAL_FILE)
            .exists();
    if reconcile_request.is_some()
        || cancel_request.is_some()
        || reconcile_source_order.is_some()
        || cancel_source_order.is_some()
        || cancel_approval_present
    {
        let admission = execution_admission
            .as_ref()
            .map(|(value, _)| value)
            .ok_or_else(|| {
                product_error(
                    ProductErrorKind::BoundaryViolation,
                    "live_execution_control_admission",
                )
            })?;
        let (order, order_raw) = execution_order.as_ref().ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_control_order",
            )
        })?;
        if let Some((request, request_raw)) = &reconcile_request {
            let (source_order, source_order_raw) =
                reconcile_source_order.as_ref().ok_or_else(|| {
                    product_error(
                        ProductErrorKind::BoundaryViolation,
                        "live_execution_reconcile_source_order",
                    )
                })?;
            validate_execution_order_snapshot(source_order, admission)?;
            validate_execution_order_progression(source_order, order)?;
            validate_execution_control_request_artifact(
                request,
                request_raw,
                Some(&root.join(LIVE_EXECUTION_RECONCILE_RECEIPT_FILE)),
                "reconcile",
                &manifest,
                &manifest_sha256,
                admission,
                source_order,
                source_order_raw,
            )?;
        } else if reconcile_source_order.is_some() {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_reconcile_source_order",
            ));
        }
        if let Some((request, request_raw)) = &cancel_request {
            let (source_order, source_order_raw) =
                cancel_source_order.as_ref().ok_or_else(|| {
                    product_error(
                        ProductErrorKind::BoundaryViolation,
                        "live_execution_cancel_source_order",
                    )
                })?;
            validate_execution_order_snapshot(source_order, admission)?;
            validate_execution_order_progression(source_order, order)?;
            validate_execution_control_request_artifact(
                request,
                request_raw,
                None,
                "cancel",
                &manifest,
                &manifest_sha256,
                admission,
                source_order,
                source_order_raw,
            )?;
        } else if cancel_source_order.is_some() && !cancel_approval_present {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_source_order",
            ));
        }
        let (cancel_source, cancel_source_raw) = cancel_source_order
            .as_ref()
            .map_or((order, order_raw.as_slice()), |(value, raw)| {
                (value, raw.as_slice())
            });
        validate_execution_cancel_approval_artifacts(
            &root,
            &manifest,
            &manifest_sha256,
            admission,
            cancel_source,
            cancel_source_raw,
            cancel_request
                .as_ref()
                .map(|(request, raw)| (request, raw.as_slice())),
        )?;
    }
    let reconcile_result = read_optional_artifact_with_raw::<LiveExecutionControlSnapshot>(
        &root.join(LIVE_EXECUTION_RECONCILE_RESULT_FILE),
        "live_execution_reconcile_result",
    )?;
    let cancel_result = read_optional_artifact_with_raw::<LiveExecutionControlSnapshot>(
        &root.join(LIVE_EXECUTION_CANCEL_RESULT_FILE),
        "live_execution_cancel_result",
    )?;
    let cancel_venue_attempted = if root.join("execution-cancel-venue-attempt.json").exists() {
        let (_, request_raw) = cancel_request.as_ref().ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_venue_attempt",
            )
        })?;
        let attempt_raw = read_live_run_artifact_bytes(
            &root.join("execution-cancel-venue-attempt.json"),
            "live_execution_cancel_venue_attempt",
        )?;
        if attempt_raw != sha256_ref(request_raw).as_bytes() {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_cancel_venue_attempt",
            ));
        }
        true
    } else {
        false
    };
    if !project_execution_cancel_attempt(
        execution_order.as_mut().map(|(order, _)| order),
        cancel_venue_attempted,
    ) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_venue_attempt",
        ));
    }
    if let Some((order, _)) = &execution_order {
        let admission = execution_admission
            .as_ref()
            .map(|(value, _)| value)
            .ok_or_else(|| {
                product_error(
                    ProductErrorKind::BoundaryViolation,
                    "live_execution_cancel_venue_attempt",
                )
            })?;
        validate_execution_order_snapshot(order, admission)?;
    }
    if let Some((control, control_raw)) = &reconcile_result {
        let (request, request_raw) = reconcile_request.as_ref().ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_control_request",
            )
        })?;
        validate_execution_control_snapshot(
            control,
            request,
            request_raw,
            &manifest,
            execution_admission.as_ref().map(|(value, _)| value),
            reconcile_source_order.as_ref().map(|(value, _)| value),
            false,
        )?;
        validate_execution_control_receipt(
            &root.join(LIVE_EXECUTION_RECONCILE_RESULT_RECEIPT_FILE),
            control_raw,
            &manifest.run_id,
            &sha256_ref(request_raw),
            control.completed_at_unix_ms,
        )?;
    }
    if let Some((control, control_raw)) = &cancel_result {
        let (request, request_raw) = cancel_request.as_ref().ok_or_else(|| {
            product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_control_request",
            )
        })?;
        validate_execution_control_snapshot(
            control,
            request,
            request_raw,
            &manifest,
            execution_admission.as_ref().map(|(value, _)| value),
            cancel_source_order.as_ref().map(|(value, _)| value),
            cancel_venue_attempted,
        )?;
        validate_execution_control_receipt(
            &root.join(LIVE_EXECUTION_CANCEL_RESULT_RECEIPT_FILE),
            control_raw,
            &manifest.run_id,
            &sha256_ref(request_raw),
            control.completed_at_unix_ms,
        )?;
    }
    let execution_control = cancel_result.as_ref().or(reconcile_result.as_ref());
    if let Some((stop, _)) = &stop
        && stop.execution_order_sha256.as_deref()
            != execution_order
                .as_ref()
                .map(|(_, raw)| sha256_ref(raw))
                .as_deref()
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_terminal_order_anchor",
        ));
    }
    validate_candidate_directory_entries(&root, &state_history)?;
    let state_receipt = load_live_run_anchor_receipt(state, run_id, candidate_state.revision)?;
    let receipt = validate_workspace_anchor_head(state)?
        .filter(|receipt| receipt.run_id == run_id)
        .unwrap_or(state_receipt);
    let candidate = project_candidate(
        state,
        &manifest,
        candidate_state,
        &LiveRunProjectionArtifacts {
            preflight: preflight.as_ref().map(|(value, _)| value),
            stop: stop.as_ref().map(|(value, _)| value),
            execution_admission: execution_admission.as_ref().map(|(value, _)| value),
            strategy_intent: strategy_intent.as_ref().map(|(value, _)| value),
            strategy_intent_sha256: strategy_intent
                .as_ref()
                .map(|(_, raw)| sha256_ref(raw))
                .as_deref(),
            sizing_decision: sizing_decision.as_ref().map(|(value, _)| value),
            sizing_decision_sha256: sizing_decision
                .as_ref()
                .map(|(_, raw)| sha256_ref(raw))
                .as_deref(),
            execution_approvals: &execution_approvals,
            execution_order: execution_order.as_ref().map(|(value, _)| value),
            execution_order_state_sha256: execution_order
                .as_ref()
                .map(|(_, raw)| sha256_ref(raw))
                .as_deref(),
            execution_control: execution_control.map(|(value, _)| value),
        },
        &receipt,
    );
    Ok((candidate, manifest, manifest_raw))
}

fn project_execution_cancel_attempt(
    order: Option<&mut LiveExecutionOrderSnapshot>,
    cancel_venue_attempted: bool,
) -> bool {
    match (order, cancel_venue_attempted) {
        (None, false) => true,
        (None, true) => false,
        (Some(order), false) => !order.cancel_attempted,
        (Some(order), true) => {
            order.cancel_attempted = true;
            true
        }
    }
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
    validate_live_market_data_symbols(&manifest.data_symbols)?;
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
    lifecycle: LiveRunCandidateLifecycle,
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
            || value.sizing.instrument_id.is_empty()
            || value.sizing.evidence_expires_at_unix_ms <= value.evaluated_at_unix_ms
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
            || (value.manual_stop == (lifecycle == LiveRunCandidateLifecycle::Failed))
            || (value.execution_adapter_send_attempted && !value.order_endpoint_access_attempted)
            || (value.real_orders_submitted && !value.execution_adapter_send_attempted)
            || ((value.order_endpoint_access_attempted
                || value.execution_adapter_send_attempted
                || value.real_orders_submitted)
                && value.execution_order_sha256.is_none())
            || value
                .execution_order_sha256
                .as_ref()
                .is_some_and(|hash| !hash.starts_with("sha256:") || hash.len() != 71)
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_stop",
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_execution_admission_artifact(
    manifest: &LiveRunCandidateManifest,
    manifest_sha256: &str,
    admission: &LiveExecutionAdmissionArtifact,
    intent: &LiveStrategyOrderIntentArtifact,
    intent_raw: &[u8],
    sizing: &LiveSizingDecisionArtifact,
    sizing_raw: &[u8],
    lifecycle: LiveRunCandidateLifecycle,
) -> Result<(), ProductError> {
    let reconstructed_request = LiveExecutionAdmissionRequest {
        run_id: admission.run_id.clone(),
        strategy_version_id: admission.strategy_version_id.clone(),
        account_ref: admission.account_ref.clone(),
        venue_ref: admission.venue_ref.clone(),
        admission_id: admission.admission_id.clone(),
        source_demo_run_id: admission.source_demo_run_id.clone(),
        strategy_intent_id: admission.strategy_intent_id.clone(),
        instrument_id: admission.instrument_id.clone(),
        side: admission.side.clone(),
        order_type: admission.order_type.clone(),
        time_in_force: admission.time_in_force.clone(),
        price: admission.price.clone(),
        quantity: admission.source_quantity.clone(),
        max_notional: admission.max_notional.clone(),
        expires_at_unix_ms: admission.expires_at_unix_ms,
        user_confirmed: true,
    };
    let reconstructed_raw = serde_json::to_vec(&reconstructed_request)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_execution_admission"))?;
    let price = Decimal::from_str_exact(&admission.price)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_execution_admission"))?;
    let quantity = Decimal::from_str_exact(&admission.quantity)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_execution_admission"))?;
    let max_notional = Decimal::from_str_exact(&admission.max_notional)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_execution_admission"))?;
    let risk_policy_max_notional = Decimal::from_str_exact(&admission.risk_policy_max_notional)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_execution_admission"))?;
    if admission.schema_version != LIVE_EXECUTION_ADMISSION_SCHEMA_VERSION
        || admission.request_sha256 != sha256_ref(&reconstructed_raw)
        || admission.source_manifest_sha256 != manifest_sha256
        || admission.run_id != manifest.run_id
        || admission.strategy_version_id != manifest.strategy_version_id
        || admission.account_ref != manifest.account_ref
        || admission.venue_ref != manifest.venue_ref
        || admission.source_demo_run_id != intent.source_demo_run_id
        || admission.strategy_intent_id != intent.intent_id
        || admission.strategy_intent_sha256 != sha256_ref(intent_raw)
        || admission.sizing_decision_sha256 != sha256_ref(sizing_raw)
        || admission.strategy_version_id != intent.strategy_version_id
        || admission.instrument_id != intent.instrument_id
        || admission.side != intent.side
        || admission.source_quantity != intent.quantity
        || sizing.schema_version != LIVE_SIZING_DECISION_SCHEMA_VERSION
        || sizing.run_id != manifest.run_id
        || sizing.source_manifest_sha256 != manifest_sha256
        || sizing.strategy_intent_sha256 != sha256_ref(intent_raw)
        || sizing.instrument_id != admission.instrument_id
        || sizing.side != admission.side
        || sizing.price != admission.price
        || sizing.source_quantity != admission.source_quantity
        || sizing.approved_quantity != admission.quantity
        || sizing.request_max_notional != admission.max_notional
        || sizing.risk_policy_max_notional != admission.risk_policy_max_notional
        || sizing.evaluated_at_unix_ms > admission.authorized_at_unix_ms
        || sizing.evidence_expires_at_unix_ms <= admission.authorized_at_unix_ms
        || !manifest.data_symbols.contains(&admission.instrument_id)
        || !matches!(admission.side.as_str(), "BUY" | "SELL")
        || admission.order_type != "LIMIT"
        || admission.time_in_force != "GTC"
        || !admission.risk_policy_ref.starts_with("risk-config-sha256:")
        || admission.owner_authority_ref.trim().is_empty()
        || admission.risk_authority_ref.trim().is_empty()
        || admission.operator_authority_ref.trim().is_empty()
        || admission.owner_authority_ref == admission.risk_authority_ref
        || admission.owner_authority_ref == admission.operator_authority_ref
        || admission.risk_authority_ref == admission.operator_authority_ref
        || admission.authorized_at_unix_ms < manifest.created_at_unix_ms
        || admission.expires_at_unix_ms <= admission.authorized_at_unix_ms
        || !admission.owner_confirmed
        || !admission.risk_confirmed
        || !admission.operator_confirmed
        || admission.kill_switch_active
        || !admission.single_shot
        || admission.cancel_order_allowed
        || admission.replace_order_allowed
        || admission.automatic_retry_allowed
        || admission.automatic_recovery_allowed
        || price <= Decimal::ZERO
        || quantity <= Decimal::ZERO
        || max_notional <= Decimal::ZERO
        || risk_policy_max_notional <= Decimal::ZERO
        || price * quantity > max_notional
        || max_notional > risk_policy_max_notional
        || (admission.consumed
            && !matches!(
                lifecycle,
                LiveRunCandidateLifecycle::Starting
                    | LiveRunCandidateLifecycle::MarketDataRunning
                    | LiveRunCandidateLifecycle::Stopping
                    | LiveRunCandidateLifecycle::Stopped
                    | LiveRunCandidateLifecycle::Failed
            ))
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_admission",
        ));
    }
    Ok(())
}

fn validate_live_strategy_intent_artifact(
    intent: &LiveStrategyOrderIntentArtifact,
    manifest: &LiveRunCandidateManifest,
) -> Result<(), ProductError> {
    let quantity = Decimal::from_str_exact(&intent.quantity).ok();
    let confidence = Decimal::from_str_exact(&intent.confidence).ok();
    if intent.schema_version != LIVE_STRATEGY_INTENT_SCHEMA_VERSION
        || intent.strategy_id != manifest.strategy_id
        || intent.strategy_version_id != manifest.strategy_version_id
        || !manifest.data_symbols.contains(&intent.instrument_id)
        || !matches!(intent.side.as_str(), "BUY" | "SELL")
        || intent.source_order_type != "market"
        || quantity.is_none_or(|value| value <= Decimal::ZERO)
        || confidence.is_none_or(|value| value < Decimal::ZERO || value > Decimal::ONE)
        || intent.market_event_seq == 0
        || intent.created_at_unix_ms == 0
        || !intent.source_manifest_sha256.starts_with("sha256:")
        || intent.source_manifest_sha256.len() != 71
        || !intent.source_result_sha256.starts_with("sha256:")
        || intent.source_result_sha256.len() != 71
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_strategy_intent",
        ));
    }
    Ok(())
}

fn load_live_execution_approvals(
    root: &Path,
) -> Result<Vec<LiveExecutionApprovalRecord>, ProductError> {
    let mut approvals = Vec::new();
    for role in [
        LiveExecutionApprovalRole::Owner,
        LiveExecutionApprovalRole::Risk,
        LiveExecutionApprovalRole::Operator,
    ] {
        if let Some((artifact, artifact_raw)) =
            read_optional_artifact_with_raw::<LiveExecutionApprovalArtifact>(
                &root.join(role.artifact_file()),
                "live_execution_approval",
            )?
        {
            let receipt_raw = read_live_run_artifact_bytes(
                &root.join(role.receipt_file()),
                "live_execution_approval_receipt",
            )?;
            let receipt: LiveRunAnchorReceipt =
                serde_json::from_slice(&receipt_raw).map_err(|_| {
                    product_error(
                        ProductErrorKind::SourceInvalid,
                        "live_execution_approval_receipt",
                    )
                })?;
            approvals.push(LiveExecutionApprovalRecord {
                role,
                artifact,
                artifact_raw,
                receipt,
            });
        }
    }
    Ok(approvals)
}

fn validate_live_execution_approval_set(
    approvals: &[LiveExecutionApprovalRecord],
    manifest: &LiveRunCandidateManifest,
    manifest_sha256: &str,
    strategy_intent_raw: Option<&[u8]>,
    sizing_decision_raw: Option<&[u8]>,
    admission: Option<&LiveExecutionAdmissionArtifact>,
) -> Result<(), ProductError> {
    let proposal_sha256 = approvals
        .first()
        .map(|record| record.artifact.proposal_sha256.as_str());
    let admission_id = approvals
        .first()
        .map(|record| record.artifact.admission_id.as_str());
    for record in approvals {
        let role = record.role;
        let approval = &record.artifact;
        let receipt = &record.receipt;
        let expected_authority = admission.map(|value| match role {
            LiveExecutionApprovalRole::Owner => value.owner_authority_ref.as_str(),
            LiveExecutionApprovalRole::Risk => value.risk_authority_ref.as_str(),
            LiveExecutionApprovalRole::Operator => value.operator_authority_ref.as_str(),
        });
        if approval.schema_version != LIVE_EXECUTION_APPROVAL_SCHEMA_VERSION
            || approval.role != role
            || approval.source_manifest_sha256 != manifest_sha256
            || approval.run_id != manifest.run_id
            || approval.strategy_version_id != manifest.strategy_version_id
            || strategy_intent_raw
                .is_none_or(|raw| approval.strategy_intent_sha256 != sha256_ref(raw))
            || sizing_decision_raw
                .is_none_or(|raw| approval.sizing_decision_sha256 != sha256_ref(raw))
            || proposal_sha256 != Some(approval.proposal_sha256.as_str())
            || admission_id != Some(approval.admission_id.as_str())
            || !approval.proposal_sha256.starts_with("sha256:")
            || approval.proposal_sha256.len() != 71
            || approval.authority_ref.trim().is_empty()
            || approval.risk_policy_ref.trim().is_empty()
            || approval.approved_at_unix_ms < manifest.created_at_unix_ms
            || approval.approved_at_unix_ms > approval.expires_at_unix_ms
            || record.artifact_raw.len() > LIVE_RUN_ARTIFACT_MAX_BYTES as usize
            || receipt.schema_version != LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION
            || receipt.run_id != manifest.run_id
            || receipt.state_sha256 != sha256_ref(&record.artifact_raw)
            || receipt.commit_sha256 != manifest_sha256
            || receipt.anchored_at_unix_ms < approval.approved_at_unix_ms
            || expected_authority.is_some_and(|value| value != approval.authority_ref)
            || admission.is_some_and(|value| {
                value.request_sha256 != approval.proposal_sha256
                    || value.admission_id != approval.admission_id
                    || value.risk_policy_ref != approval.risk_policy_ref
                    || value.sizing_decision_sha256 != approval.sizing_decision_sha256
            })
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_approval",
            ));
        }
    }
    if admission.is_some() && approvals.len() != 3 {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_approval",
        ));
    }
    Ok(())
}

fn validate_execution_order_snapshot(
    order: &LiveExecutionOrderSnapshot,
    admission: &LiveExecutionAdmissionArtifact,
) -> Result<(), ProductError> {
    let quantities_valid = Decimal::from_str_exact(&order.original_quantity)
        .ok()
        .zip(Decimal::from_str_exact(&order.filled_quantity).ok())
        .zip(Decimal::from_str_exact(&order.remaining_quantity).ok())
        .zip(Decimal::from_str_exact(&admission.quantity).ok())
        .is_some_and(|(((original, filled), remaining), admitted)| {
            original > Decimal::ZERO
                && original == admitted
                && filled >= Decimal::ZERO
                && remaining >= Decimal::ZERO
                && filled + remaining == original
        });
    let terminal_status = matches!(
        order.status.as_str(),
        "rejected" | "denied" | "expired" | "filled" | "canceled" | "submission_failed"
    );
    let status_valid = matches!(
        order.status.as_str(),
        "waiting_for_instrument"
            | "submission_requested"
            | "submitted"
            | "accepted"
            | "rejected"
            | "denied"
            | "expired"
            | "partially_filled"
            | "filled"
            | "canceled"
            | "submission_failed"
    );
    let waiting = order.status == "waiting_for_instrument";
    let denied_before_adapter = order.status == "denied";
    let failed_before_attempt =
        order.status == "submission_failed" && !order.actual_submission_attempted;
    if order.schema_version != LIVE_EXECUTION_ORDER_STATE_SCHEMA_VERSION
        || order.admission_id != admission.admission_id
        || order.source_demo_run_id != admission.source_demo_run_id
        || order.strategy_intent_id != admission.strategy_intent_id
        || order.strategy_intent_sha256 != admission.strategy_intent_sha256
        || order.sizing_decision_sha256 != admission.sizing_decision_sha256
        || order.strategy_version_id != admission.strategy_version_id
        || order.instrument_id != admission.instrument_id
        || !status_valid
        || !quantities_valid
        || order.terminal != terminal_status
        || !order.new_orders_blocked
        || (waiting && order.actual_submission_attempted)
        || (!waiting
            && !denied_before_adapter
            && !failed_before_attempt
            && !order.actual_submission_attempted)
        || ((waiting || failed_before_attempt) && order.client_order_id.is_some())
        || (!waiting && !failed_before_attempt && order.client_order_id.is_none())
        || order.automatic_retry_attempted
        || !execution_order_cancel_attempt_is_valid(order)
        || order.replace_attempted
        || order.updated_at_unix_ms < admission.authorized_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_order_state",
        ));
    }
    Ok(())
}

fn execution_order_cancel_attempt_is_valid(order: &LiveExecutionOrderSnapshot) -> bool {
    !order.cancel_attempted
        || (order.actual_submission_attempted
            && order.client_order_id.is_some()
            && matches!(
                order.status.as_str(),
                "submission_requested"
                    | "submitted"
                    | "accepted"
                    | "partially_filled"
                    | "canceled"
                    | "filled"
                    | "expired"
                    | "rejected"
            ))
}

fn validate_execution_order_progression(
    source: &LiveExecutionOrderSnapshot,
    current: &LiveExecutionOrderSnapshot,
) -> Result<(), ProductError> {
    let source_filled = Decimal::from_str_exact(&source.filled_quantity).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_order_progression",
        )
    })?;
    let current_filled = Decimal::from_str_exact(&current.filled_quantity).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_order_progression",
        )
    })?;
    if source.admission_id != current.admission_id
        || source.strategy_version_id != current.strategy_version_id
        || source.instrument_id != current.instrument_id
        || source.client_order_id != current.client_order_id
        || source.original_quantity != current.original_quantity
        || source
            .venue_order_id
            .as_ref()
            .is_some_and(|source| current.venue_order_id.as_ref() != Some(source))
        || current_filled < source_filled
        || current.updated_at_unix_ms < source.updated_at_unix_ms
        || (source.terminal && !current.terminal)
        || (source.cancel_attempted && !current.cancel_attempted)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_order_progression",
        ));
    }
    Ok(())
}

fn validate_execution_order_runtime_context(
    candidate_root: &Path,
    lifecycle: LiveRunCandidateLifecycle,
) -> Result<(), ProductError> {
    if !matches!(
        lifecycle,
        LiveRunCandidateLifecycle::Starting
            | LiveRunCandidateLifecycle::MarketDataRunning
            | LiveRunCandidateLifecycle::Stopping
            | LiveRunCandidateLifecycle::Stopped
            | LiveRunCandidateLifecycle::Failed
    ) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_order_state",
        ));
    }
    let raw = read_live_run_artifact_bytes(
        &candidate_root.join(LIVE_MARKET_DATA_NODE_CONFIG_FILE),
        "live_execution_runtime_config",
    )?;
    let raw = std::str::from_utf8(&raw).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_runtime_config",
        )
    })?;
    let config: toml::Value = toml::from_str(raw).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_runtime_config",
        )
    })?;
    if config.get("live_execution").is_none() {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_order_state",
        ));
    }
    Ok(())
}

struct LiveRunProjectionArtifacts<'a> {
    preflight: Option<&'a LiveRunPreflightArtifact>,
    stop: Option<&'a LiveRunStopArtifact>,
    execution_admission: Option<&'a LiveExecutionAdmissionArtifact>,
    strategy_intent: Option<&'a LiveStrategyOrderIntentArtifact>,
    strategy_intent_sha256: Option<&'a str>,
    sizing_decision: Option<&'a LiveSizingDecisionArtifact>,
    sizing_decision_sha256: Option<&'a str>,
    execution_approvals: &'a [LiveExecutionApprovalRecord],
    execution_order: Option<&'a LiveExecutionOrderSnapshot>,
    execution_order_state_sha256: Option<&'a str>,
    execution_control: Option<&'a LiveExecutionControlSnapshot>,
}

fn project_candidate(
    server_state: &DashboardServerState,
    manifest: &LiveRunCandidateManifest,
    candidate_state: &LiveRunCandidateState,
    artifacts: &LiveRunProjectionArtifacts<'_>,
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
        preflight_at_unix_ms: artifacts.preflight.map(|value| value.evaluated_at_unix_ms),
        stopped_at_unix_ms: artifacts.stop.map(|value| value.stopped_at_unix_ms),
        account_connected: artifacts.preflight.is_some(),
        account_can_trade_verified: artifacts.preflight.is_some(),
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
        order_admission: if artifacts.execution_admission.is_some() {
            let consumed = canonical_live_run_root(server_state, false).is_ok_and(|root| {
                fs::read_to_string(
                    root.join(&manifest.run_id)
                        .join(LIVE_MARKET_DATA_NODE_CONFIG_FILE),
                )
                .is_ok_and(|raw| raw.contains("[live_execution]"))
            });
            if consumed {
                let mut snapshot = LiveOrderAdmissionSnapshot::consumed();
                snapshot.cancel = "dual_approval_required".to_string();
                snapshot.fill_reconciliation = "explicit_manual_available".to_string();
                snapshot
                    .blockers
                    .retain(|value| value != "manual_review_required_for_follow_up");
                if let Some(control) = artifacts.execution_control {
                    if control.action == "reconcile" {
                        snapshot.fill_reconciliation = control.status.clone();
                    } else if control.action == "cancel" {
                        snapshot.cancel = control.status.clone();
                    }
                }
                snapshot
            } else {
                LiveOrderAdmissionSnapshot::authorized()
            }
        } else {
            LiveOrderAdmissionSnapshot::blocked_with_approvals(
                artifacts
                    .execution_approvals
                    .iter()
                    .any(|record| record.role == LiveExecutionApprovalRole::Owner),
                artifacts
                    .execution_approvals
                    .iter()
                    .any(|record| record.role == LiveExecutionApprovalRole::Risk),
                artifacts
                    .execution_approvals
                    .iter()
                    .any(|record| record.role == LiveExecutionApprovalRole::Operator),
            )
        },
        strategy_intent: artifacts.strategy_intent.cloned(),
        strategy_intent_sha256: artifacts.strategy_intent_sha256.map(str::to_string),
        sizing_decision: artifacts.sizing_decision.cloned(),
        sizing_decision_sha256: artifacts.sizing_decision_sha256.map(str::to_string),
        execution_order: artifacts.execution_order.cloned(),
        execution_order_state_sha256: artifacts.execution_order_state_sha256.map(str::to_string),
        execution_control: artifacts.execution_control.cloned(),
        source_refs: manifest.source_refs.clone(),
    }
}

fn project_live_market_data_runtime(
    state: &DashboardServerState,
    manifest: &LiveRunCandidateManifest,
    lifecycle: LiveRunCandidateLifecycle,
) -> (bool, bool, Option<String>, String, Option<String>) {
    let runtime_config = canonical_live_run_root(state, false).ok().and_then(|root| {
        fs::read_to_string(
            root.join(&manifest.run_id)
                .join(LIVE_MARKET_DATA_NODE_CONFIG_FILE),
        )
        .ok()
    });
    let runtime_config_exists = runtime_config.is_some();
    let execution_expected = runtime_config
        .as_deref()
        .is_some_and(|raw| raw.contains("[live_execution]"));
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
    let execution_boundary_valid = if execution_expected {
        record.last_known_status.execution_connection == ConnectionStatus::Connected
            && record
                .last_known_status
                .execution
                .started
                .value
                .unwrap_or(false)
    } else {
        record.last_known_status.execution_connection == ConnectionStatus::NotConfigured
            && !record
                .last_known_status
                .execution
                .started
                .value
                .unwrap_or(true)
            && !record.last_known_status.real_orders_submitted
    };
    let lifecycle_exposes_runtime = matches!(
        lifecycle,
        LiveRunCandidateLifecycle::MarketDataRunning | LiveRunCandidateLifecycle::Stopping
    );
    let error = if matches!(lifecycle, LiveRunCandidateLifecycle::MarketDataRunning)
        && (!running || !data_connected || !execution_boundary_valid)
    {
        Some("live_market_data_runtime_boundary_violation".to_string())
    } else if record.last_known_status.last_error.is_some() {
        Some("live_market_data_runtime_reported_error".to_string())
    } else {
        None
    };
    (
        lifecycle_exposes_runtime && running && execution_boundary_valid,
        lifecycle_exposes_runtime && data_connected && execution_boundary_valid,
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
            execution_admission_sha256: None,
            execution_runtime_config_sha256: None,
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

fn write_live_run_state_recoverable(
    server_state: &DashboardServerState,
    run_id: &str,
    candidate_state: &LiveRunCandidateState,
    previous_state: &LiveRunCandidateState,
    previous_state_sha256: &str,
    previous_workspace_receipt: &LiveRunAnchorReceipt,
    fail_after: Option<ApprovalPublicationStep>,
) -> Result<(), ProductError> {
    validate_live_run_state_transition(previous_state, candidate_state)?;
    if candidate_state.previous_state_sha256.as_deref() != Some(previous_state_sha256) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_state_transition",
        ));
    }
    let run_root = canonical_live_run_root(server_state, false)?.join(run_id);
    let run_directory = open_absolute_directory_nofollow(&run_root)?;
    let state_raw = serde_json::to_vec_pretty(candidate_state)
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_run_state"))?;
    let previous_commit_raw =
        load_live_run_state_commit_raw(server_state, run_id, previous_state.revision)?;
    let previous_commit_sha256 = sha256_ref(&previous_commit_raw);
    let commit = LiveRunStateCommit {
        schema_version: LIVE_RUN_STATE_COMMIT_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        revision: candidate_state.revision,
        state_sha256: sha256_ref(&state_raw),
        previous_commit_sha256: Some(previous_commit_sha256.clone()),
        committed_at_unix_ms: candidate_state.updated_at_unix_ms,
    };
    let commit_raw = serde_json::to_vec_pretty(&commit).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_run_state_commit",
        )
    })?;
    let anchor_request = LiveRunAnchorAppendRequest::new(
        server_state.live_run_audit_anchor.namespace()?,
        run_id,
        LiveRunAnchorRevision::new(
            candidate_state.revision,
            previous_workspace_receipt.workspace_revision + 1,
        ),
        sha256_ref(&state_raw),
        sha256_ref(&commit_raw),
        Some(previous_workspace_receipt.sha256()),
        candidate_state.updated_at_unix_ms,
    );
    let receipt_path = run_root.join(live_run_anchor_receipt_file_name(candidate_state.revision));
    let receipt = if let Some((persisted, _)) = read_optional_artifact_with_raw::<
        LiveRunAnchorReceipt,
    >(
        &receipt_path, "live_run_audit_anchor_receipt"
    )? {
        persisted
    } else {
        let latest = server_state
            .live_run_audit_anchor
            .latest()?
            .ok_or_else(|| {
                product_error(
                    ProductErrorKind::BoundaryViolation,
                    "live_run_workspace_anchor_latest",
                )
            })?;
        if latest == *previous_workspace_receipt {
            server_state.live_run_audit_anchor.append(&anchor_request)?
        } else {
            latest
        }
    };
    server_state
        .live_run_audit_anchor
        .validate_receipt(&receipt, &anchor_request)?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::StateExternalAnchor)?;
    let receipt_raw = serde_json::to_vec_pretty(&receipt).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_run_audit_anchor_receipt",
        )
    })?;
    write_same_or_new_run_file(
        &run_directory,
        &live_run_state_file_name(candidate_state.revision),
        &state_raw,
        "live_run_state",
    )?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::State)?;
    let commit_directory = open_live_run_state_commit_directory(server_state, true)?;
    write_same_or_new_run_file(
        &commit_directory,
        &live_run_state_commit_file_name(run_id, candidate_state.revision),
        &commit_raw,
        "live_run_state_commit",
    )?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::StateCommit)?;
    write_same_or_new_run_file(
        &run_directory,
        &live_run_anchor_receipt_file_name(candidate_state.revision),
        &receipt_raw,
        "live_run_audit_anchor_receipt",
    )?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::StateReceipt)?;
    recover_live_run_state_head_for_publication(
        &run_directory,
        previous_state,
        candidate_state,
        &state_raw,
        &commit_raw,
        &receipt,
        &previous_commit_sha256,
    )?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::StateHead)?;
    recover_workspace_anchor_head_for_publication(
        server_state,
        previous_workspace_receipt,
        &receipt,
    )?;
    fail_approval_publication_after(fail_after, ApprovalPublicationStep::StateWorkspaceHead)?;
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

fn recover_live_run_state_head_for_publication(
    directory: &cap_std::fs::Dir,
    previous_state: &LiveRunCandidateState,
    state: &LiveRunCandidateState,
    state_raw: &[u8],
    commit_raw: &[u8],
    receipt: &LiveRunAnchorReceipt,
    previous_commit_sha256: &str,
) -> Result<(), ProductError> {
    let target = LiveRunStateHead {
        schema_version: LIVE_RUN_STATE_HEAD_SCHEMA_VERSION.to_string(),
        run_id: state.run_id.clone(),
        revision: state.revision,
        state_sha256: sha256_ref(state_raw),
        commit_sha256: sha256_ref(commit_raw),
        anchor_receipt_sha256: receipt.sha256(),
        updated_at_unix_ms: state.updated_at_unix_ms,
    };
    let target_raw = serde_json::to_vec_pretty(&target)
        .map_err(|_| product_error(ProductErrorKind::LiveExecutionFailed, "live_run_state_head"))?;
    if let Ok(current_raw) =
        read_run_file_from_directory(directory, LIVE_RUN_STATE_HEAD_FILE, "live_run_state_head")
    {
        if current_raw == target_raw {
            return Ok(());
        }
        let current: LiveRunStateHead = serde_json::from_slice(&current_raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_state_head"))?;
        if current.run_id != previous_state.run_id
            || current.revision != previous_state.revision
            || current.state_sha256 != previous_state_sha256_for_head(previous_state)?
            || current.commit_sha256 != previous_commit_sha256
            || current.updated_at_unix_ms > state.updated_at_unix_ms
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_run_state_head",
            ));
        }
    }
    write_same_or_new_run_file(
        directory,
        LIVE_RUN_STATE_HEAD_NEXT_FILE,
        &target_raw,
        "live_run_state_head",
    )?;
    directory
        .rename(
            LIVE_RUN_STATE_HEAD_NEXT_FILE,
            directory,
            LIVE_RUN_STATE_HEAD_FILE,
        )
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "live_run_state_head"))?;
    let persisted =
        read_run_file_from_directory(directory, LIVE_RUN_STATE_HEAD_FILE, "live_run_state_head")?;
    if persisted != target_raw {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_run_state_head",
        ));
    }
    Ok(())
}

fn previous_state_sha256_for_head(state: &LiveRunCandidateState) -> Result<String, ProductError> {
    serde_json::to_vec_pretty(state)
        .map(|raw| sha256_ref(&raw))
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_run_state_head"))
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
                && previous.execution_admission_sha256.is_none()
                && previous.execution_runtime_config_sha256.is_none()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256.is_some()
                && current.execution_admission_sha256.is_none()
                && current.execution_runtime_config_sha256.is_none()
                && current.stop_sha256.is_none()
        }
        (LiveRunCandidateLifecycle::Created, LiveRunCandidateLifecycle::Stopped) => {
            previous.preflight_sha256.is_none()
                && previous.execution_admission_sha256.is_none()
                && previous.execution_runtime_config_sha256.is_none()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256.is_none()
                && current.execution_admission_sha256.is_none()
                && current.execution_runtime_config_sha256.is_none()
                && current.stop_sha256.is_some()
        }
        (LiveRunCandidateLifecycle::PreflightReady, LiveRunCandidateLifecycle::PreflightReady) => {
            previous.preflight_sha256.is_some()
                && previous.execution_admission_sha256.is_none()
                && previous.execution_runtime_config_sha256.is_none()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.execution_admission_sha256.is_some()
                && current.execution_runtime_config_sha256.is_none()
                && current.stop_sha256.is_none()
        }
        (LiveRunCandidateLifecycle::PreflightReady, LiveRunCandidateLifecycle::Stopped) => {
            previous.preflight_sha256.is_some()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.execution_admission_sha256 == previous.execution_admission_sha256
                && current.execution_runtime_config_sha256.is_none()
                && current.stop_sha256.is_some()
        }
        (LiveRunCandidateLifecycle::PreflightReady, LiveRunCandidateLifecycle::Starting) => {
            previous.preflight_sha256.is_some()
                && previous.stop_sha256.is_none()
                && previous.execution_runtime_config_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.execution_admission_sha256 == previous.execution_admission_sha256
                && current.execution_runtime_config_sha256.is_some()
                    == current.execution_admission_sha256.is_some()
                && current.stop_sha256.is_none()
        }
        (LiveRunCandidateLifecycle::Starting, LiveRunCandidateLifecycle::MarketDataRunning)
        | (LiveRunCandidateLifecycle::MarketDataRunning, LiveRunCandidateLifecycle::Stopping) => {
            previous.preflight_sha256.is_some()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.execution_admission_sha256 == previous.execution_admission_sha256
                && current.execution_runtime_config_sha256
                    == previous.execution_runtime_config_sha256
                && current.stop_sha256.is_none()
        }
        (
            LiveRunCandidateLifecycle::Starting | LiveRunCandidateLifecycle::MarketDataRunning,
            LiveRunCandidateLifecycle::Failed,
        ) => {
            previous.preflight_sha256.is_some()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.execution_admission_sha256 == previous.execution_admission_sha256
                && current.execution_runtime_config_sha256
                    == previous.execution_runtime_config_sha256
                && current.stop_sha256.is_some()
        }
        (LiveRunCandidateLifecycle::Stopping, LiveRunCandidateLifecycle::Stopped) => {
            previous.preflight_sha256.is_some()
                && previous.stop_sha256.is_none()
                && current.preflight_sha256 == previous.preflight_sha256
                && current.execution_admission_sha256 == previous.execution_admission_sha256
                && current.execution_runtime_config_sha256
                    == previous.execution_runtime_config_sha256
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
    execution_admission: Option<&(LiveExecutionAdmissionArtifact, Vec<u8>)>,
    stop: Option<&(LiveRunStopArtifact, Vec<u8>)>,
) -> Result<(), ProductError> {
    let preflight_sha = preflight.map(|(_, raw)| sha256_ref(raw));
    let execution_admission_sha = execution_admission.map(|(_, raw)| sha256_ref(raw));
    let stop_sha = stop.map(|(_, raw)| sha256_ref(raw));
    let execution_scoped = execution_admission.is_some();
    let revision_offset = u64::from(execution_scoped);
    let runtime_config_binding_valid = match state.lifecycle {
        LiveRunCandidateLifecycle::Created | LiveRunCandidateLifecycle::PreflightReady => {
            state.execution_runtime_config_sha256.is_none()
        }
        LiveRunCandidateLifecycle::Starting
        | LiveRunCandidateLifecycle::MarketDataRunning
        | LiveRunCandidateLifecycle::Stopping
        | LiveRunCandidateLifecycle::Failed => {
            state.execution_runtime_config_sha256.is_some() == execution_scoped
        }
        LiveRunCandidateLifecycle::Stopped => {
            if execution_scoped && state.revision == 3 {
                state.execution_runtime_config_sha256.is_none()
            } else {
                state.execution_runtime_config_sha256.is_some() == execution_scoped
            }
        }
    };
    let valid = runtime_config_binding_valid
        && match state.lifecycle {
            LiveRunCandidateLifecycle::Created => {
                state.revision == 0
                    && state.updated_at_unix_ms == manifest.created_at_unix_ms
                    && state.preflight_sha256.is_none()
                    && state.execution_admission_sha256.is_none()
                    && state.stop_sha256.is_none()
                    && preflight.is_none()
                    && stop.is_none()
            }
            LiveRunCandidateLifecycle::PreflightReady => {
                let evaluated_at = preflight.map(|(value, _)| value.evaluated_at_unix_ms);
                matches!(state.revision, 1 | 2)
                    && state.preflight_sha256 == preflight_sha
                    && state.execution_admission_sha256 == execution_admission_sha
                    && state.stop_sha256.is_none()
                    && stop.is_none()
                    && if execution_scoped {
                        state.revision == 2
                            && state.updated_at_unix_ms
                                >= evaluated_at.unwrap_or(state.updated_at_unix_ms)
                    } else {
                        state.revision == 1 && Some(state.updated_at_unix_ms) == evaluated_at
                    }
            }
            LiveRunCandidateLifecycle::Starting => {
                state.revision == 2 + revision_offset
                    && state.preflight_sha256 == preflight_sha
                    && state.execution_admission_sha256 == execution_admission_sha
                    && state.stop_sha256.is_none()
                    && preflight.is_some()
                    && stop.is_none()
            }
            LiveRunCandidateLifecycle::MarketDataRunning => {
                state.revision == 3 + revision_offset
                    && state.preflight_sha256 == preflight_sha
                    && state.execution_admission_sha256 == execution_admission_sha
                    && state.stop_sha256.is_none()
                    && preflight.is_some()
                    && stop.is_none()
            }
            LiveRunCandidateLifecycle::Stopping => {
                state.revision == 4 + revision_offset
                    && state.preflight_sha256 == preflight_sha
                    && state.execution_admission_sha256 == execution_admission_sha
                    && state.stop_sha256.is_none()
                    && preflight.is_some()
                    && stop.is_none()
            }
            LiveRunCandidateLifecycle::Failed => {
                let failed_at = stop.map(|(value, _)| value.stopped_at_unix_ms);
                matches!(state.revision, 3..=5)
                    && state.preflight_sha256 == preflight_sha
                    && state.execution_admission_sha256 == execution_admission_sha
                    && state.stop_sha256 == stop_sha
                    && preflight.is_some()
                    && Some(state.updated_at_unix_ms) == failed_at
                    && stop.as_ref().is_some_and(|(value, _)| {
                        !value.manual_stop && value.source_preflight_sha256 == preflight_sha
                    })
            }
            LiveRunCandidateLifecycle::Stopped => {
                let stopped_at = stop.map(|(value, _)| value.stopped_at_unix_ms);
                let valid_revision = if preflight.is_some() {
                    if execution_scoped {
                        matches!(state.revision, 3 | 6)
                    } else {
                        matches!(state.revision, 2 | 5)
                    }
                } else {
                    state.revision == 1
                };
                valid_revision
                    && state.preflight_sha256 == preflight_sha
                    && state.execution_admission_sha256 == execution_admission_sha
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
    if state.execution_admission_sha256.is_some() {
        expected.insert(LIVE_EXECUTION_ADMISSION_FILE.to_string());
    }
    let strategy_intent_exists = root.join(LIVE_STRATEGY_INTENT_FILE).exists();
    if strategy_intent_exists {
        expected.insert(LIVE_STRATEGY_INTENT_FILE.to_string());
    }
    let sizing_decision_exists = root.join(LIVE_SIZING_DECISION_FILE).exists();
    if sizing_decision_exists {
        expected.insert(LIVE_SIZING_DECISION_FILE.to_string());
    }
    let mut approval_count = 0;
    for role in [
        LiveExecutionApprovalRole::Owner,
        LiveExecutionApprovalRole::Risk,
        LiveExecutionApprovalRole::Operator,
    ] {
        let artifact_exists = root.join(role.artifact_file()).exists();
        let receipt_exists = root.join(role.receipt_file()).exists();
        if artifact_exists != receipt_exists {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_approval",
            ));
        }
        if artifact_exists {
            approval_count += 1;
            expected.insert(role.artifact_file().to_string());
            expected.insert(role.receipt_file().to_string());
        }
    }
    if state.execution_admission_sha256.is_some() && approval_count != 3 {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_approval",
        ));
    }
    if approval_count > 0 && (!strategy_intent_exists || !sizing_decision_exists) {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_sizing_decision",
        ));
    }
    let claim_exists = validate_live_execution_runtime_claim(root, state)?;
    let claim_required = state.execution_admission_sha256.is_some()
        && matches!(
            state.lifecycle,
            LiveRunCandidateLifecycle::MarketDataRunning
                | LiveRunCandidateLifecycle::Stopping
                | LiveRunCandidateLifecycle::Stopped
                | LiveRunCandidateLifecycle::Failed
        );
    if claim_required && !claim_exists {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_runtime_claim",
        ));
    }
    if claim_exists {
        expected.insert(LIVE_EXECUTION_RUNTIME_CLAIM_FILE.to_string());
        expected.insert(LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE.to_string());
    }
    let reconcile_request = root.join(LIVE_EXECUTION_RECONCILE_REQUEST_FILE).exists();
    let reconcile_source = root
        .join(LIVE_EXECUTION_RECONCILE_SOURCE_ORDER_FILE)
        .exists();
    let reconcile_receipt = root.join(LIVE_EXECUTION_RECONCILE_RECEIPT_FILE).exists();
    let reconcile_attempt = root.join("execution-reconcile-attempt.json").exists();
    let reconcile_result = root.join(LIVE_EXECUTION_RECONCILE_RESULT_FILE).exists();
    let reconcile_result_receipt = root
        .join(LIVE_EXECUTION_RECONCILE_RESULT_RECEIPT_FILE)
        .exists();
    if reconcile_request != reconcile_receipt
        || reconcile_request != reconcile_source
        || reconcile_attempt && !reconcile_request
        || reconcile_result && !reconcile_attempt
        || reconcile_result != reconcile_result_receipt
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_reconcile_artifacts",
        ));
    }
    if reconcile_request {
        expected.insert(LIVE_EXECUTION_RECONCILE_REQUEST_FILE.to_string());
        expected.insert(LIVE_EXECUTION_RECONCILE_SOURCE_ORDER_FILE.to_string());
        expected.insert(LIVE_EXECUTION_RECONCILE_RECEIPT_FILE.to_string());
    }
    if reconcile_attempt {
        expected.insert("execution-reconcile-attempt.json".to_string());
    }
    if reconcile_result {
        expected.insert(LIVE_EXECUTION_RECONCILE_RESULT_FILE.to_string());
        expected.insert(LIVE_EXECUTION_RECONCILE_RESULT_RECEIPT_FILE.to_string());
    }
    let cancel_owner = root
        .join(LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE)
        .exists();
    let cancel_source = root.join(LIVE_EXECUTION_CANCEL_SOURCE_ORDER_FILE).exists();
    let cancel_owner_receipt = root.join(LIVE_EXECUTION_CANCEL_OWNER_RECEIPT_FILE).exists();
    let cancel_operator = root
        .join(LIVE_EXECUTION_CANCEL_OPERATOR_APPROVAL_FILE)
        .exists();
    let cancel_operator_receipt = root
        .join(LIVE_EXECUTION_CANCEL_OPERATOR_RECEIPT_FILE)
        .exists();
    let cancel_request = root.join(LIVE_EXECUTION_CANCEL_REQUEST_FILE).exists();
    let cancel_attempt = root.join("execution-cancel-attempt.json").exists();
    let cancel_venue_attempt = root.join("execution-cancel-venue-attempt.json").exists();
    let cancel_result = root.join(LIVE_EXECUTION_CANCEL_RESULT_FILE).exists();
    let cancel_result_receipt = root
        .join(LIVE_EXECUTION_CANCEL_RESULT_RECEIPT_FILE)
        .exists();
    if cancel_owner != cancel_owner_receipt
        || cancel_owner != cancel_source
        || cancel_operator != cancel_operator_receipt
        || cancel_operator && !cancel_owner
        || cancel_request != cancel_operator
        || cancel_attempt && !cancel_request
        || cancel_venue_attempt && !cancel_attempt
        || cancel_result && !cancel_attempt
        || cancel_result != cancel_result_receipt
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_cancel_artifacts",
        ));
    }
    for (present, file) in [
        (cancel_owner, LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE),
        (cancel_source, LIVE_EXECUTION_CANCEL_SOURCE_ORDER_FILE),
        (
            cancel_owner_receipt,
            LIVE_EXECUTION_CANCEL_OWNER_RECEIPT_FILE,
        ),
        (
            cancel_operator,
            LIVE_EXECUTION_CANCEL_OPERATOR_APPROVAL_FILE,
        ),
        (
            cancel_operator_receipt,
            LIVE_EXECUTION_CANCEL_OPERATOR_RECEIPT_FILE,
        ),
        (cancel_request, LIVE_EXECUTION_CANCEL_REQUEST_FILE),
        (cancel_attempt, "execution-cancel-attempt.json"),
        (cancel_venue_attempt, "execution-cancel-venue-attempt.json"),
        (cancel_result, LIVE_EXECUTION_CANCEL_RESULT_FILE),
        (
            cancel_result_receipt,
            LIVE_EXECUTION_CANCEL_RESULT_RECEIPT_FILE,
        ),
    ] {
        if present {
            expected.insert(file.to_string());
        }
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

fn validate_live_execution_runtime_claim(
    root: &Path,
    state: &LiveRunCandidateState,
) -> Result<bool, ProductError> {
    let claim = read_optional_artifact_with_raw::<LiveExecutionRuntimeClaimArtifact>(
        &root.join(LIVE_EXECUTION_RUNTIME_CLAIM_FILE),
        "live_execution_runtime_claim",
    )?;
    let receipt = read_optional_artifact_with_raw::<LiveRunAnchorReceipt>(
        &root.join(LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE),
        "live_execution_runtime_claim_receipt",
    )?;
    let (Some((claim, claim_raw)), Some((receipt, _))) = (claim, receipt) else {
        if root.join(LIVE_EXECUTION_RUNTIME_CLAIM_FILE).exists()
            || root
                .join(LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE)
                .exists()
        {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "live_execution_runtime_claim",
            ));
        }
        return Ok(false);
    };
    let valid = claim.schema_version == "ntpro.live_execution.runtime_claim.v1"
        && claim.run_id == state.run_id
        && claim.control_state_revision <= state.revision
        && claim.starting_receipt_sha256.starts_with("sha256:")
        && claim.starting_receipt_sha256.len() == 71
        && claim.source_manifest_sha256 == state.source_manifest_sha256
        && Some(claim.execution_admission_sha256.as_str())
            == state.execution_admission_sha256.as_deref()
        && Some(claim.runtime_config_sha256.as_str())
            == state.execution_runtime_config_sha256.as_deref()
        && Path::new(&claim.runtime_artifact_root).is_absolute()
        && receipt.schema_version == LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION
        && receipt.run_id == state.run_id
        && receipt.revision == claim.control_state_revision
        && receipt.state_sha256 == sha256_ref(&claim_raw)
        && receipt.commit_sha256 == claim.runtime_config_sha256
        && receipt.previous_receipt_sha256.as_deref()
            == Some(claim.starting_receipt_sha256.as_str())
        && receipt.anchored_at_unix_ms >= claim.claimed_at_unix_ms;
    if !valid {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_runtime_claim",
        ));
    }
    Ok(true)
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
            if name == LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE
                || matches!(
                    name.as_str(),
                    LIVE_EXECUTION_OWNER_APPROVAL_RECEIPT_FILE
                        | LIVE_EXECUTION_RISK_APPROVAL_RECEIPT_FILE
                        | LIVE_EXECUTION_OPERATOR_APPROVAL_RECEIPT_FILE
                        | LIVE_EXECUTION_RECONCILE_RECEIPT_FILE
                        | LIVE_EXECUTION_RECONCILE_RESULT_RECEIPT_FILE
                        | LIVE_EXECUTION_CANCEL_OWNER_RECEIPT_FILE
                        | LIVE_EXECUTION_CANCEL_OPERATOR_RECEIPT_FILE
                        | LIVE_EXECUTION_CANCEL_RESULT_RECEIPT_FILE
                )
            {
                let (receipt, _) = read_optional_artifact_with_raw::<LiveRunAnchorReceipt>(
                    &artifact.path(),
                    "live_execution_runtime_claim_receipt",
                )?
                .ok_or_else(|| {
                    product_error(
                        ProductErrorKind::SourceUnavailable,
                        "live_execution_runtime_claim_receipt",
                    )
                })?;
                receipts.push(receipt);
                continue;
            }
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
        Err(error) if error.kind == ProductErrorKind::Conflict => Err(product_error(
            ProductErrorKind::LiveConflict,
            "active_live_run_candidate",
        )),
        Err(error) => Err(error),
    }
}

fn release_active_live_run_candidate(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<(), ProductError> {
    release_active_live_run_candidate_with_absence(state, run_id, false)
}

fn release_active_live_run_candidate_if_present(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<(), ProductError> {
    release_active_live_run_candidate_with_absence(state, run_id, true)
}

fn release_active_live_run_candidate_with_absence(
    state: &DashboardServerState,
    run_id: &str,
    allow_absent: bool,
) -> Result<(), ProductError> {
    let Some(pointer) = load_active_live_run_pointer(state)? else {
        return if allow_absent {
            Ok(())
        } else {
            Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "active_live_run_candidate",
            ))
        };
    };
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
    let process_reservation = {
        let mut locks = LIVE_RUN_PROCESS_MUTATION_LOCKS.lock().map_err(|_| {
            product_error(
                ProductErrorKind::SourceUnavailable,
                "live_run_mutation_lock",
            )
        })?;
        if !locks.insert(artifact_root_path.to_path_buf()) {
            return Err(product_error(
                ProductErrorKind::LiveConflict,
                "live_run_mutation_lock",
            ));
        }
        LiveRunProcessMutationReservation {
            artifact_root: artifact_root_path.to_path_buf(),
        }
    };
    let artifact_root = open_absolute_directory_nofollow(artifact_root_path)?;
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).write(true).create(true);
    options.follow(FollowSymlinks::No);
    let mut file = artifact_root
        .open_with(LIVE_RUN_MUTATION_LOCK_FILE, &options)
        .map_err(|_| {
            product_error(
                ProductErrorKind::SourceUnavailable,
                "live_run_mutation_lock",
            )
        })?
        .into_std();
    match file.try_lock() {
        Ok(()) => {}
        Err(fs::TryLockError::WouldBlock) => {
            return Err(product_error(
                ProductErrorKind::LiveConflict,
                "live_run_mutation_lock",
            ));
        }
        Err(fs::TryLockError::Error(error))
            if matches!(
                error.kind(),
                ErrorKind::WouldBlock | ErrorKind::ResourceBusy
            ) =>
        {
            return Err(product_error(
                ProductErrorKind::LiveConflict,
                "live_run_mutation_lock",
            ));
        }
        Err(fs::TryLockError::Error(_)) => {
            return Err(product_error(
                ProductErrorKind::SourceUnavailable,
                "live_run_mutation_lock",
            ));
        }
    }
    let pid = std::process::id();
    let process_start_time_secs = process_start_time_secs(pid).ok_or_else(|| {
        product_error(
            ProductErrorKind::SourceUnavailable,
            "live_run_mutation_lock",
        )
    })?;
    let owner = LiveRunMutationLockArtifact {
        schema_version: LIVE_RUN_MUTATION_LOCK_SCHEMA_VERSION.to_string(),
        pid,
        process_start_time_secs,
        acquired_at_unix_ms: unix_time_ms(),
    };
    let mut raw = serde_json::to_vec_pretty(&owner).map_err(|_| {
        product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_run_mutation_lock",
        )
    })?;
    raw.push(b'\n');
    file.set_len(0)
        .and_then(|()| file.seek(SeekFrom::Start(0)).map(|_| ()))
        .and_then(|()| file.write_all(&raw))
        .and_then(|()| file.sync_all())
        .map_err(|_| {
            product_error(
                ProductErrorKind::SourceUnavailable,
                "live_run_mutation_lock",
            )
        })?;
    Ok(LiveRunMutationLock {
        file,
        _process_reservation: process_reservation,
    })
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

    static LIVE_RUNTIME_PROCESS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn lock_live_runtime_process_tests() -> std::sync::MutexGuard<'static, ()> {
        LIVE_RUNTIME_PROCESS_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn execution_control_snapshot(action: &str, status: &str) -> LiveExecutionControlSnapshot {
        LiveExecutionControlSnapshot {
            schema_version: LIVE_EXECUTION_CONTROL_RESULT_SCHEMA_VERSION.to_string(),
            request_sha256: VERSION_HASH.to_string(),
            request_id: "control-001".to_string(),
            action: action.to_string(),
            run_id: "live-candidate-control".to_string(),
            admission_id: "admission-001".to_string(),
            strategy_version_id: "strategy@v1".to_string(),
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            client_order_id: "S3LV007-001".to_string(),
            venue_order_id: Some("1001".to_string()),
            status: status.to_string(),
            exchange_order_status: Some("accepted".to_string()),
            original_quantity: Some("0.01".to_string()),
            filled_quantity: Some("0".to_string()),
            remaining_quantity: Some("0.01".to_string()),
            query_attempted: true,
            cancel_attempted: false,
            cancel_confirmed: false,
            automatic_retry_attempted: false,
            manual_review_required: false,
            error_code: None,
            completed_at_unix_ms: 1,
        }
    }

    #[test]
    fn execution_control_status_matrix_rejects_cross_action_and_flag_drift() {
        let reconcile = execution_control_snapshot("reconcile", "reconciled");
        assert!(execution_control_status_is_valid(&reconcile));

        let mut invalid = reconcile.clone();
        invalid.action = "cancel".to_string();
        assert!(!execution_control_status_is_valid(&invalid));

        invalid = reconcile;
        invalid.manual_review_required = true;
        assert!(!execution_control_status_is_valid(&invalid));

        let mut terminal =
            execution_control_snapshot("cancel", "cancel_not_required_terminal_or_pending");
        terminal.exchange_order_status = Some("filled".to_string());
        assert!(execution_control_status_is_valid(&terminal));

        terminal.action = "reconcile".to_string();
        assert!(!execution_control_status_is_valid(&terminal));

        let mut confirmed = execution_control_snapshot("cancel", "cancel_confirmed");
        confirmed.exchange_order_status = Some("canceled".to_string());
        confirmed.cancel_attempted = true;
        confirmed.cancel_confirmed = true;
        assert!(execution_control_status_is_valid(&confirmed));
        confirmed.manual_review_required = true;
        assert!(!execution_control_status_is_valid(&confirmed));
    }

    #[test]
    fn execution_order_cancel_state_projects_marker_ahead_and_rejects_false_claims() {
        assert!(project_execution_cancel_attempt(None, false));
        assert!(!project_execution_cancel_attempt(None, true));
        let mut order: LiveExecutionOrderSnapshot = serde_json::from_value(serde_json::json!({
            "schema_version": LIVE_EXECUTION_ORDER_STATE_SCHEMA_VERSION,
            "admission_id": "admission-001",
            "source_demo_run_id": "demo-source-001",
            "strategy_intent_id": "intent-001",
            "strategy_intent_sha256": format!("sha256:{}", "5".repeat(64)),
            "sizing_decision_sha256": format!("sha256:{}", "6".repeat(64)),
            "strategy_version_id": "strategy@v1",
            "instrument_id": "BTCUSDT.BINANCE",
            "client_order_id": "S3LV007-001",
            "venue_order_id": "1001",
            "original_quantity": "0.01",
            "filled_quantity": "0",
            "remaining_quantity": "0.01",
            "status": "accepted",
            "terminal": false,
            "new_orders_blocked": true,
            "actual_submission_attempted": true,
            "automatic_retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "last_error": null,
            "updated_at_unix_ms": 1
        }))
        .unwrap();
        assert!(project_execution_cancel_attempt(Some(&mut order), false));
        assert!(project_execution_cancel_attempt(Some(&mut order), true));
        assert!(order.cancel_attempted);
        assert!(!project_execution_cancel_attempt(Some(&mut order), false));
        order.cancel_attempted = true;
        assert!(project_execution_cancel_attempt(Some(&mut order), true));
        for status in ["submission_requested", "submitted"] {
            order.status = status.to_string();
            order.cancel_attempted = false;
            assert!(project_execution_cancel_attempt(Some(&mut order), true));
            assert!(order.cancel_attempted, "{status}");
            assert!(execution_order_cancel_attempt_is_valid(&order), "{status}");
        }
        for status in ["waiting_for_instrument", "denied", "submission_failed"] {
            order.status = status.to_string();
            order.cancel_attempted = true;
            assert!(!execution_order_cancel_attempt_is_valid(&order), "{status}");
        }
    }

    #[test]
    fn execution_order_progression_accepts_partial_fill_and_rejects_regression() {
        let source: LiveExecutionOrderSnapshot = serde_json::from_value(serde_json::json!({
            "schema_version": LIVE_EXECUTION_ORDER_STATE_SCHEMA_VERSION,
            "admission_id": "admission-001",
            "source_demo_run_id": "demo-source-001",
            "strategy_intent_id": "intent-001",
            "strategy_intent_sha256": format!("sha256:{}", "5".repeat(64)),
            "sizing_decision_sha256": format!("sha256:{}", "6".repeat(64)),
            "strategy_version_id": "strategy@v1",
            "instrument_id": "BTCUSDT.BINANCE",
            "client_order_id": "S3LV008-001",
            "venue_order_id": "1001",
            "original_quantity": "0.00001000",
            "filled_quantity": "0.00000400",
            "remaining_quantity": "0.00000600",
            "status": "partially_filled",
            "terminal": false,
            "new_orders_blocked": true,
            "actual_submission_attempted": true,
            "automatic_retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "last_error": null,
            "updated_at_unix_ms": 10
        }))
        .unwrap();
        let mut progressed = source.clone();
        progressed.filled_quantity = "0.00000700".to_string();
        progressed.remaining_quantity = "0.00000300".to_string();
        progressed.updated_at_unix_ms = 11;
        assert!(validate_execution_order_progression(&source, &progressed).is_ok());

        let mut regressed = progressed.clone();
        regressed.filled_quantity = "0.00000300".to_string();
        regressed.remaining_quantity = "0.00000700".to_string();
        regressed.updated_at_unix_ms = 12;
        assert!(validate_execution_order_progression(&source, &regressed).is_err());

        let mut changed_venue = progressed;
        changed_venue.venue_order_id = Some("1002".to_string());
        assert!(validate_execution_order_progression(&source, &changed_venue).is_err());

        let mut missing_venue = source.clone();
        missing_venue.venue_order_id = None;
        missing_venue.updated_at_unix_ms += 1;
        assert!(validate_execution_order_progression(&source, &missing_venue).is_err());
    }

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
                "risk-config-sha256:3311829f7f08266f4f8b706148285292e433d725a57b85ffd3b551f64223968c"
                    .to_string(),
                VERSION_HASH.to_string(),
            ],
        }
    }

    fn sizing_preflight() -> LiveSizingPreflight {
        LiveSizingPreflight {
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            base_asset: "BTC".to_string(),
            quote_asset: "USDT".to_string(),
            base_free: "1.00000000".to_string(),
            quote_free: "1000.00".to_string(),
            price_tick: "0.01".to_string(),
            quantity_step: "0.00001000".to_string(),
            min_quantity: "0.00001000".to_string(),
            max_quantity: "9000.00000000".to_string(),
            min_notional: "0.000001".to_string(),
            max_account_budget_fraction: "0.10".to_string(),
            evidence_expires_at_unix_ms: u64::MAX,
            source_ref: format!("sizing-config-sha256:{}", "2".repeat(64)),
        }
    }

    fn gates() -> LiveRunGateState {
        LiveRunGateState {
            candidate_create: true,
            owner_approved: true,
            no_order_send: true,
            manual_stop: true,
            risk_approved: true,
            execution_single_shot: true,
            order_control: true,
        }
    }

    const VERSION_HASH: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn live_run_candidate_requires_every_independent_gate() {
        for missing in 0..5 {
            let mut values = [true; 7];
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
        let mut execution_blocked = gates();
        execution_blocked.execution_single_shot = false;
        assert!(execution_blocked.all_open());
        assert!(!execution_blocked.execution_single_shot);
    }

    #[test]
    fn live_runtime_failure_transition_accepts_pre_start_and_post_start_cleanup() {
        let state = |revision, lifecycle, stop_sha256| LiveRunCandidateState {
            schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
            run_id: "live-candidate-transition".to_string(),
            source_manifest_sha256: VERSION_HASH.to_string(),
            revision,
            previous_state_sha256: (revision > 0).then(|| VERSION_HASH.to_string()),
            lifecycle,
            preflight_sha256: Some(VERSION_HASH.to_string()),
            execution_admission_sha256: None,
            execution_runtime_config_sha256: None,
            stop_sha256,
            updated_at_unix_ms: revision + 1,
        };
        assert!(
            validate_live_run_state_transition(
                &state(2, LiveRunCandidateLifecycle::Starting, None),
                &state(
                    3,
                    LiveRunCandidateLifecycle::Failed,
                    Some(VERSION_HASH.to_string()),
                ),
            )
            .is_ok()
        );
        assert!(
            validate_live_run_state_transition(
                &state(3, LiveRunCandidateLifecycle::MarketDataRunning, None),
                &state(
                    4,
                    LiveRunCandidateLifecycle::Failed,
                    Some(VERSION_HASH.to_string()),
                ),
            )
            .is_ok()
        );
        assert!(
            validate_live_run_state_transition(
                &state(2, LiveRunCandidateLifecycle::Starting, None),
                &state(3, LiveRunCandidateLifecycle::Failed, None),
            )
            .is_err()
        );
    }

    #[test]
    fn confirmed_submission_projection_is_identical_for_list_and_detail() {
        let mut order = LiveExecutionOrderSnapshot {
            schema_version: LIVE_EXECUTION_ORDER_STATE_SCHEMA_VERSION.to_string(),
            admission_id: "admission-001".to_string(),
            source_demo_run_id: "demo-source-001".to_string(),
            strategy_intent_id: "intent-001".to_string(),
            strategy_intent_sha256: format!("sha256:{}", "5".repeat(64)),
            sizing_decision_sha256: format!("sha256:{}", "6".repeat(64)),
            strategy_version_id: "ema-cross@v1".to_string(),
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            client_order_id: Some("S3LV007-001".to_string()),
            venue_order_id: Some("1001".to_string()),
            original_quantity: "0.01".to_string(),
            filled_quantity: "0".to_string(),
            remaining_quantity: "0.01".to_string(),
            status: "submitted".to_string(),
            terminal: false,
            new_orders_blocked: true,
            actual_submission_attempted: true,
            automatic_retry_attempted: false,
            cancel_attempted: false,
            replace_attempted: false,
            last_error: None,
            updated_at_unix_ms: 1,
        };
        for status in [
            "submitted",
            "accepted",
            "rejected",
            "expired",
            "partially_filled",
            "filled",
            "canceled",
        ] {
            order.status = status.to_string();
            assert!(execution_order_has_confirmed_submission(&order), "{status}");
        }
        order.status = "submission_requested".to_string();
        order.cancel_attempted = true;
        assert!(execution_order_has_confirmed_submission(&order));
        order.cancel_attempted = false;
        for status in [
            "waiting_for_instrument",
            "submission_requested",
            "denied",
            "submission_failed",
        ] {
            order.status = status.to_string();
            assert!(
                !execution_order_has_confirmed_submission(&order),
                "{status}"
            );
        }
    }

    #[test]
    fn live_execution_runtime_claim_binds_artifact_bytes_to_receipt() {
        let fixture = LiveRunFixture::new("runtime-claim-binding");
        let root = fixture
            .root
            .join("artifacts/live-runs/live-candidate-claim-binding");
        fs::create_dir_all(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let manifest_sha256 = format!("sha256:{}", "1".repeat(64));
        let admission_sha256 = format!("sha256:{}", "2".repeat(64));
        let config_sha256 = format!("sha256:{}", "3".repeat(64));
        let starting_receipt_sha256 = format!("sha256:{}", "4".repeat(64));
        let state = LiveRunCandidateState {
            schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
            run_id: "live-candidate-claim-binding".to_string(),
            source_manifest_sha256: manifest_sha256.clone(),
            revision: 4,
            previous_state_sha256: Some(VERSION_HASH.to_string()),
            lifecycle: LiveRunCandidateLifecycle::MarketDataRunning,
            preflight_sha256: Some(VERSION_HASH.to_string()),
            execution_admission_sha256: Some(admission_sha256.clone()),
            execution_runtime_config_sha256: Some(config_sha256.clone()),
            stop_sha256: None,
            updated_at_unix_ms: 20,
        };
        let mut claim = LiveExecutionRuntimeClaimArtifact {
            schema_version: "ntpro.live_execution.runtime_claim.v1".to_string(),
            claim_id: "claim-001".to_string(),
            run_id: state.run_id.clone(),
            control_state_revision: 3,
            starting_receipt_sha256: starting_receipt_sha256.clone(),
            source_manifest_sha256: manifest_sha256,
            execution_admission_sha256: admission_sha256,
            runtime_config_sha256: config_sha256.clone(),
            runtime_artifact_root: fixture.root.display().to_string(),
            claimed_at_unix_ms: 10,
        };
        let claim_raw = serde_json::to_vec_pretty(&claim).unwrap();
        let receipt = LiveRunAnchorReceipt {
            schema_version: LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION.to_string(),
            namespace: "test".to_string(),
            run_id: state.run_id.clone(),
            revision: 3,
            workspace_revision: 4,
            state_sha256: sha256_ref(&claim_raw),
            commit_sha256: config_sha256,
            previous_receipt_sha256: Some(starting_receipt_sha256),
            anchored_at_unix_ms: 10,
            key_id: "test-key".to_string(),
            receipt_id: "receipt-001".to_string(),
            signature_base64: "test-signature".to_string(),
        };
        fs::write(root.join(LIVE_EXECUTION_RUNTIME_CLAIM_FILE), claim_raw).unwrap();
        fs::write(
            root.join(LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
        assert!(validate_live_execution_runtime_claim(&root, &state).unwrap());

        claim.claim_id = "tampered-claim".to_string();
        fs::write(
            root.join(LIVE_EXECUTION_RUNTIME_CLAIM_FILE),
            serde_json::to_vec_pretty(&claim).unwrap(),
        )
        .unwrap();
        let error = validate_live_execution_runtime_claim(&root, &state).unwrap_err();
        assert_eq!(error.field, "live_execution_runtime_claim");
    }

    #[test]
    fn failed_stop_artifact_is_idempotent_across_state_publish_retry() {
        let fixture = LiveRunFixture::new("failed-stop-retry-window");
        let candidate_root = fixture
            .root
            .join("artifacts/live-runs/live-candidate-stop-retry");
        fs::create_dir_all(&candidate_root).unwrap();
        let candidate_root = fs::canonicalize(candidate_root).unwrap();
        let directory = open_absolute_directory_nofollow(&candidate_root).unwrap();
        let activity = LiveExecutionStopActivity {
            endpoint_access_attempted: true,
            adapter_send_attempted: true,
            real_order_submitted: true,
            order_sha256: Some(VERSION_HASH.to_string()),
        };

        let (first, first_raw) = load_or_create_failed_stop_artifact(FailedStopArtifactContext {
            candidate_root: &candidate_root,
            directory: &directory,
            run_id: "live-candidate-stop-retry",
            manifest_sha256: VERSION_HASH,
            preflight_sha256: Some(VERSION_HASH),
            current_updated_at_unix_ms: 10,
            failed_at_unix_ms: 20,
            activity: &activity,
        })
        .unwrap();
        let (retried, retried_raw) =
            load_or_create_failed_stop_artifact(FailedStopArtifactContext {
                candidate_root: &candidate_root,
                directory: &directory,
                run_id: "live-candidate-stop-retry",
                manifest_sha256: VERSION_HASH,
                preflight_sha256: Some(VERSION_HASH),
                current_updated_at_unix_ms: 10,
                failed_at_unix_ms: 30,
                activity: &activity,
            })
            .unwrap();

        assert_eq!(retried, first);
        assert_eq!(retried_raw, first_raw);
        assert_eq!(retried.stopped_at_unix_ms, 20);
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
                    sizing: sizing_preflight(),
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
                    sizing: sizing_preflight(),
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
                    sizing: sizing_preflight(),
                })
            },
        )
        .unwrap()
    }

    fn execution_admission_request(run_id: &str) -> LiveExecutionAdmissionRequest {
        let intent = test_live_strategy_intent();
        LiveExecutionAdmissionRequest {
            run_id: run_id.to_string(),
            strategy_version_id: "ema-cross@v1".to_string(),
            account_ref: "account://live/binance/primary".to_string(),
            venue_ref: "venue://live/BINANCE".to_string(),
            admission_id: "admission-001".to_string(),
            source_demo_run_id: intent.source_demo_run_id,
            strategy_intent_id: intent.intent_id,
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            time_in_force: "GTC".to_string(),
            price: "1.00".to_string(),
            quantity: "0.00001000".to_string(),
            max_notional: "1.00".to_string(),
            expires_at_unix_ms: unix_time_ms() + 60_000,
            user_confirmed: true,
        }
    }

    fn test_live_strategy_intent() -> LiveStrategyOrderIntentArtifact {
        LiveStrategyOrderIntentArtifact {
            schema_version: LIVE_STRATEGY_INTENT_SCHEMA_VERSION.to_string(),
            source_demo_run_id: "demo-source-001".to_string(),
            strategy_id: "ema-cross".to_string(),
            strategy_version_id: "ema-cross@v1".to_string(),
            intent_id: "intent-001".to_string(),
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            side: "BUY".to_string(),
            source_order_type: "market".to_string(),
            quantity: "0.00001000".to_string(),
            source_signal: "long".to_string(),
            confidence: "0.72".to_string(),
            market_event_seq: 1,
            created_at_unix_ms: 100,
            source_manifest_sha256: format!("sha256:{}", "5".repeat(64)),
            source_result_sha256: format!("sha256:{}", "6".repeat(64)),
        }
    }

    fn execution_risk_policy() -> LiveExecutionRiskPolicy {
        LiveExecutionRiskPolicy {
            max_order_notional: "1.00".to_string(),
            owner_authority_ref: "role://institution-owner".to_string(),
            risk_authority_ref: "policy://risk/test-v1".to_string(),
            operator_authority_ref: "role://operations-operator".to_string(),
            source_ref: format!("risk-config-sha256:{}", "3".repeat(64)),
        }
    }

    #[test]
    fn live_sizing_rounds_down_and_binds_budget() {
        let mut request = execution_admission_request("live-sizing-001");
        request.quantity = "0.00001999".to_string();
        request.price = "100.00".to_string();
        let sizing = sizing_preflight();
        let decision = evaluate_live_sizing_decision(
            "live-sizing-001",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &request,
            &sizing,
            &execution_risk_policy(),
            1,
        )
        .unwrap();
        assert_eq!(decision.source_quantity, "0.00001999");
        assert_eq!(decision.approved_quantity, "0.00001");
        assert_eq!(decision.price_tick, "0.01");
        assert_eq!(decision.quantity_step, "0.00001000");
        assert_eq!(decision.min_quantity, "0.00001000");
        assert_eq!(decision.max_quantity, "9000.00000000");
        assert_eq!(decision.min_notional, "0.000001");
        assert_eq!(decision.max_account_budget_fraction, "0.10");
        assert_eq!(decision.order_notional, "0.001");
        assert_eq!(decision.account_budget_notional, "100");
    }

    #[test]
    fn live_sizing_applies_account_budget_fraction_to_sell_inventory() {
        let mut request = execution_admission_request("live-sizing-sell");
        request.side = "SELL".to_string();
        request.price = "100.00".to_string();
        request.quantity = "0.10001000".to_string();
        request.max_notional = "1000.00".to_string();
        let mut risk = execution_risk_policy();
        risk.max_order_notional = "1000.00".to_string();
        let error = evaluate_live_sizing_decision(
            "live-sizing-sell",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &request,
            &sizing_preflight(),
            &risk,
            1,
        )
        .unwrap_err();
        assert_eq!(error.field, "live_sizing_decision.account_budget");

        request.quantity = "0.09999000".to_string();
        let decision = evaluate_live_sizing_decision(
            "live-sizing-sell",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &request,
            &sizing_preflight(),
            &risk,
            1,
        )
        .unwrap();
        assert_eq!(decision.account_budget_notional, "10");
        assert_eq!(decision.order_notional, "9.999");
    }

    #[test]
    fn live_sizing_fails_closed_for_rule_budget_and_freshness_drift() {
        let mut request = execution_admission_request("live-sizing-002");
        request.price = "100.001".to_string();
        let error = evaluate_live_sizing_decision(
            "live-sizing-002",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &request,
            &sizing_preflight(),
            &execution_risk_policy(),
            1,
        )
        .unwrap_err();
        assert_eq!(error.field, "live_sizing_decision.price_tick");

        request.price = "100000.00".to_string();
        request.quantity = "0.00200000".to_string();
        request.max_notional = "1000.00".to_string();
        let mut risk = execution_risk_policy();
        risk.max_order_notional = "1000.00".to_string();
        let error = evaluate_live_sizing_decision(
            "live-sizing-002",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &request,
            &sizing_preflight(),
            &risk,
            1,
        )
        .unwrap_err();
        assert_eq!(error.field, "live_sizing_decision.account_budget");

        let mut below_min = execution_admission_request("live-sizing-002");
        below_min.quantity = "0.00000999".to_string();
        let error = evaluate_live_sizing_decision(
            "live-sizing-002",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &below_min,
            &sizing_preflight(),
            &execution_risk_policy(),
            1,
        )
        .unwrap_err();
        assert_eq!(error.field, "live_sizing_decision.min_quantity");

        let mut above_max = execution_admission_request("live-sizing-002");
        above_max.quantity = "0.00002000".to_string();
        let mut max_rule = sizing_preflight();
        max_rule.max_quantity = "0.00001000".to_string();
        let error = evaluate_live_sizing_decision(
            "live-sizing-002",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &above_max,
            &max_rule,
            &execution_risk_policy(),
            1,
        )
        .unwrap_err();
        assert_eq!(error.field, "live_sizing_decision.max_quantity");

        let mut notional_rule = sizing_preflight();
        notional_rule.min_notional = "0.01".to_string();
        let error = evaluate_live_sizing_decision(
            "live-sizing-002",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &execution_admission_request("live-sizing-002"),
            &notional_rule,
            &execution_risk_policy(),
            1,
        )
        .unwrap_err();
        assert_eq!(error.field, "live_sizing_decision.min_notional");

        let mut expired = sizing_preflight();
        expired.evidence_expires_at_unix_ms = 1;
        let error = evaluate_live_sizing_decision(
            "live-sizing-002",
            VERSION_HASH,
            VERSION_HASH,
            VERSION_HASH,
            &execution_admission_request("live-sizing-002"),
            &expired,
            &execution_risk_policy(),
            1,
        )
        .unwrap_err();
        assert_eq!(error.field, "live_sizing_decision.evidence_expired");
    }

    fn promotable_strategy_intent() -> PromotableStrategyOrderIntent {
        let intent = test_live_strategy_intent();
        PromotableStrategyOrderIntent {
            source_run_id: intent.source_demo_run_id,
            strategy_id: intent.strategy_id,
            strategy_version_id: intent.strategy_version_id,
            intent_id: intent.intent_id,
            instrument_id: intent.instrument_id,
            side: intent.side,
            source_order_type: intent.source_order_type,
            quantity: intent.quantity,
            source_signal: intent.source_signal,
            confidence: intent.confidence,
            market_event_seq: intent.market_event_seq,
            created_at_unix_ms: intent.created_at_unix_ms,
            source_manifest_sha256: intent.source_manifest_sha256,
            source_result_sha256: intent.source_result_sha256,
        }
    }

    fn execution_source() -> (LiveExecutionRiskPolicy, PromotableStrategyOrderIntent) {
        (execution_risk_policy(), promotable_strategy_intent())
    }

    #[test]
    fn strategy_intent_is_single_use_across_live_runs() {
        let fixture = LiveRunFixture::new("strategy-intent-single-use");
        let root = canonical_live_run_root(&fixture.state, true).unwrap();
        let historical = root.join("live-candidate-historical");
        fs::create_dir_all(&historical).unwrap();
        let intent = test_live_strategy_intent();
        fs::write(
            historical.join(LIVE_STRATEGY_INTENT_FILE),
            serde_json::to_vec_pretty(&intent).unwrap(),
        )
        .unwrap();

        ensure_strategy_intent_is_unconsumed(&fixture.state, "live-candidate-historical", &intent)
            .unwrap();
        let error =
            ensure_strategy_intent_is_unconsumed(&fixture.state, "live-candidate-current", &intent)
                .unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::LiveConflict);
        assert_eq!(error.field, "live_strategy_intent_consumed");
    }

    fn setup_cancel_publication(
        name: &str,
    ) -> (
        LiveRunFixture,
        String,
        LiveExecutionAdmissionArtifact,
        LiveExecutionOrderSnapshot,
        Vec<u8>,
    ) {
        let fixture = LiveRunFixture::new(name);
        let ready = create_preflight_ready_candidate(
            &fixture,
            &format!("product-{name}-cancel-publication"),
        );
        let admission_request = execution_admission_request(&ready.run_id);
        for role in [
            LiveExecutionApprovalRole::Owner,
            LiveExecutionApprovalRole::Risk,
            LiveExecutionApprovalRole::Operator,
        ] {
            authorize_live_execution_with_source_validator(
                &fixture.state,
                &ready.run_id,
                &admission_request,
                role,
                |_| Ok(execution_source()),
            )
            .unwrap();
        }
        let candidate_root = canonical_live_run_root(&fixture.state, false)
            .unwrap()
            .join(&ready.run_id);
        let directory = open_absolute_directory_nofollow(&candidate_root).unwrap();
        let (manifest, manifest_raw) =
            load_live_run_manifest(&fixture.state, &ready.run_id).unwrap();
        let manifest_sha256 = sha256_ref(&manifest_raw);
        let (preflight_ready, preflight_ready_raw) =
            load_live_run_state(&fixture.state, &ready.run_id, &manifest_sha256).unwrap();
        write_live_node_config(
            &fixture.state,
            &directory,
            &manifest,
            read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
                &candidate_root.join(LIVE_EXECUTION_ADMISSION_FILE),
                "live_execution_admission",
            )
            .unwrap()
            .as_ref()
            .map(|(value, _)| value),
        )
        .unwrap();
        let runtime_config_raw =
            fs::read(candidate_root.join(LIVE_MARKET_DATA_NODE_CONFIG_FILE)).unwrap();
        let starting = LiveRunCandidateState {
            schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
            run_id: ready.run_id.clone(),
            source_manifest_sha256: manifest_sha256.clone(),
            revision: preflight_ready.revision + 1,
            previous_state_sha256: Some(sha256_ref(&preflight_ready_raw)),
            lifecycle: LiveRunCandidateLifecycle::Starting,
            preflight_sha256: preflight_ready.preflight_sha256,
            execution_admission_sha256: preflight_ready.execution_admission_sha256,
            execution_runtime_config_sha256: Some(sha256_ref(&runtime_config_raw)),
            stop_sha256: None,
            updated_at_unix_ms: unix_time_ms(),
        };
        write_live_run_state(&fixture.state, &ready.run_id, &starting).unwrap();
        let (starting, starting_raw) =
            load_live_run_state(&fixture.state, &ready.run_id, &manifest_sha256).unwrap();
        write_live_run_state(
            &fixture.state,
            &ready.run_id,
            &LiveRunCandidateState {
                schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                run_id: ready.run_id.clone(),
                source_manifest_sha256: manifest_sha256,
                revision: starting.revision + 1,
                previous_state_sha256: Some(sha256_ref(&starting_raw)),
                lifecycle: LiveRunCandidateLifecycle::MarketDataRunning,
                preflight_sha256: starting.preflight_sha256,
                execution_admission_sha256: starting.execution_admission_sha256,
                execution_runtime_config_sha256: starting.execution_runtime_config_sha256,
                stop_sha256: None,
                updated_at_unix_ms: unix_time_ms(),
            },
        )
        .unwrap();
        let (admission, _) = read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
            &candidate_root.join(LIVE_EXECUTION_ADMISSION_FILE),
            "live_execution_admission",
        )
        .unwrap()
        .unwrap();
        let order = LiveExecutionOrderSnapshot {
            schema_version: LIVE_EXECUTION_ORDER_STATE_SCHEMA_VERSION.to_string(),
            admission_id: admission.admission_id.clone(),
            source_demo_run_id: admission.source_demo_run_id.clone(),
            strategy_intent_id: admission.strategy_intent_id.clone(),
            strategy_intent_sha256: admission.strategy_intent_sha256.clone(),
            sizing_decision_sha256: admission.sizing_decision_sha256.clone(),
            strategy_version_id: admission.strategy_version_id.clone(),
            instrument_id: admission.instrument_id.clone(),
            client_order_id: Some("S3LV008-001".to_string()),
            venue_order_id: Some("1001".to_string()),
            original_quantity: admission.quantity.clone(),
            filled_quantity: "0".to_string(),
            remaining_quantity: admission.quantity.clone(),
            status: "accepted".to_string(),
            terminal: false,
            new_orders_blocked: true,
            actual_submission_attempted: true,
            automatic_retry_attempted: false,
            cancel_attempted: false,
            replace_attempted: false,
            last_error: None,
            updated_at_unix_ms: unix_time_ms().max(admission.authorized_at_unix_ms),
        };
        let order_raw = serde_json::to_vec_pretty(&order).unwrap();
        let runtime_root = live_market_data_runtime_root(&fixture.state, &ready.run_id).unwrap();
        fs::write(
            runtime_root.join(LIVE_EXECUTION_ORDER_STATE_FILE),
            &order_raw,
        )
        .unwrap();
        (fixture, ready.run_id, admission, order, order_raw)
    }

    fn cancel_request(run_id: &str, order_raw: &[u8]) -> LiveExecutionCancelRequest {
        LiveExecutionCancelRequest {
            run_id: run_id.to_string(),
            request_id: "cancel-001".to_string(),
            client_order_id: "S3LV008-001".to_string(),
            source_order_state_sha256: sha256_ref(order_raw),
            expires_at_unix_ms: unix_time_ms() + 60_000,
            user_confirmed: true,
        }
    }

    fn stage_cancel_publication(
        fixture: &LiveRunFixture,
        run_id: &str,
        request: &LiveExecutionCancelRequest,
        role: LiveExecutionApprovalRole,
        source_order_raw: &[u8],
    ) -> LiveExecutionCancelPublicationStage {
        let candidate_root = canonical_live_run_root(&fixture.state, false)
            .unwrap()
            .join(run_id);
        let (admission, _) = read_optional_artifact_with_raw::<LiveExecutionAdmissionArtifact>(
            &candidate_root.join(LIVE_EXECUTION_ADMISSION_FILE),
            "live_execution_admission",
        )
        .unwrap()
        .unwrap();
        let (_, manifest_raw) = load_live_run_manifest(&fixture.state, run_id).unwrap();
        let manifest_sha256 = sha256_ref(&manifest_raw);
        let (state, _) = load_live_run_state(&fixture.state, run_id, &manifest_sha256).unwrap();
        let approved_at_unix_ms = unix_time_ms();
        let proposal_sha256 = sha256_ref(&serde_json::to_vec(request).unwrap());
        let approval = LiveExecutionCancelApprovalArtifact {
            schema_version: "ntpro.s3.live_execution_cancel_approval.v1".to_string(),
            role,
            proposal_sha256,
            source_manifest_sha256: manifest_sha256.clone(),
            run_id: run_id.to_string(),
            admission_id: admission.admission_id.clone(),
            strategy_version_id: admission.strategy_version_id.clone(),
            instrument_id: admission.instrument_id.clone(),
            client_order_id: request.client_order_id.clone(),
            source_order_state_sha256: request.source_order_state_sha256.clone(),
            authority_ref: match role {
                LiveExecutionApprovalRole::Owner => admission.owner_authority_ref.clone(),
                LiveExecutionApprovalRole::Operator => admission.operator_authority_ref.clone(),
                LiveExecutionApprovalRole::Risk => unreachable!(),
            },
            approved_at_unix_ms,
            expires_at_unix_ms: request.expires_at_unix_ms,
        };
        let control_request = (role == LiveExecutionApprovalRole::Operator).then(|| {
            LiveExecutionControlRequestArtifact {
                schema_version: LIVE_EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION.to_string(),
                request_id: request.request_id.clone(),
                action: "cancel".to_string(),
                run_id: run_id.to_string(),
                admission_id: admission.admission_id,
                strategy_version_id: admission.strategy_version_id,
                instrument_id: admission.instrument_id,
                client_order_id: request.client_order_id.clone(),
                source_order_state_sha256: request.source_order_state_sha256.clone(),
                owner_confirmed: true,
                operator_confirmed: true,
                requested_at_unix_ms: approved_at_unix_ms,
                expires_at_unix_ms: request.expires_at_unix_ms,
            }
        });
        let stage = LiveExecutionCancelPublicationStage {
            schema_version: "ntpro.s3.live_execution_cancel_publication.v1".to_string(),
            role,
            request: request.clone(),
            source_order_raw: String::from_utf8(source_order_raw.to_vec()).unwrap(),
            approval,
            control_request,
            run_revision: state.revision,
            manifest_sha256,
            previous_workspace_receipt: validate_workspace_anchor_head(&fixture.state)
                .unwrap()
                .unwrap(),
        };
        let directory = open_absolute_directory_nofollow(&candidate_root).unwrap();
        publish_new_run_file(
            &directory,
            cancel_publication_stage_file(role).unwrap(),
            &serde_json::to_vec_pretty(&stage).unwrap(),
        )
        .unwrap();
        stage
    }

    #[test]
    fn live_execution_admission_is_fail_closed_single_use_and_externally_anchored() {
        let fixture = LiveRunFixture::new("execution-admission");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000090");
        let mut rejected = execution_admission_request(&ready.run_id);
        rejected.user_confirmed = false;
        let error = authorize_live_execution_with_source_validator(
            &fixture.state,
            &ready.run_id,
            &rejected,
            LiveExecutionApprovalRole::Owner,
            |_| Ok(execution_source()),
        )
        .unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
        assert_eq!(error.field, "live_execution_admission");

        for field in ["side", "quantity", "intent"] {
            let mut drifted = execution_admission_request(&ready.run_id);
            match field {
                "side" => drifted.side = "SELL".to_string(),
                "quantity" => drifted.quantity = "0.00002000".to_string(),
                "intent" => drifted.strategy_intent_id = "intent-002".to_string(),
                _ => unreachable!(),
            }
            let error = authorize_live_execution_with_source_validator(
                &fixture.state,
                &ready.run_id,
                &drifted,
                LiveExecutionApprovalRole::Owner,
                |_| Ok(execution_source()),
            )
            .unwrap_err();
            assert_eq!(error.kind, ProductErrorKind::BoundaryViolation, "{field}");
            assert_eq!(error.field, "live_strategy_intent_binding", "{field}");
        }

        let request = execution_admission_request(&ready.run_id);
        let owner_approved = authorize_live_execution_with_source_validator(
            &fixture.state,
            &ready.run_id,
            &request,
            LiveExecutionApprovalRole::Owner,
            |_| Ok(execution_source()),
        )
        .unwrap();
        assert_eq!(owner_approved.order_admission.status, "blocked");
        assert!(owner_approved.order_admission.owner_approved);
        assert!(!owner_approved.order_admission.risk_approved);
        assert!(!owner_approved.order_admission.operator_approved);
        let risk_approved = authorize_live_execution_with_source_validator(
            &fixture.state,
            &ready.run_id,
            &request,
            LiveExecutionApprovalRole::Risk,
            |_| Ok(execution_source()),
        )
        .unwrap();
        assert_eq!(risk_approved.order_admission.status, "blocked");
        assert!(risk_approved.order_admission.owner_approved);
        assert!(risk_approved.order_admission.risk_approved);
        assert!(!risk_approved.order_admission.operator_approved);
        let authorized = authorize_live_execution_with_source_validator(
            &fixture.state,
            &ready.run_id,
            &request,
            LiveExecutionApprovalRole::Operator,
            |_| Ok(execution_source()),
        )
        .unwrap();
        assert_eq!(
            authorized.lifecycle,
            LiveRunCandidateLifecycle::PreflightReady
        );
        assert_eq!(authorized.order_admission.status, "authorized_single_shot");
        let (manifest, manifest_raw) =
            load_live_run_manifest(&fixture.state, &ready.run_id).unwrap();
        let (anchored_state, _) =
            load_live_run_state(&fixture.state, &ready.run_id, &sha256_ref(&manifest_raw)).unwrap();
        let admission_raw = fs::read(
            fixture
                .root
                .join("artifacts/live-runs")
                .join(&ready.run_id)
                .join(LIVE_EXECUTION_ADMISSION_FILE),
        )
        .unwrap();
        assert_eq!(anchored_state.revision, 2);
        assert_eq!(
            anchored_state.execution_admission_sha256,
            Some(sha256_ref(&admission_raw))
        );
        assert_eq!(manifest.run_id, ready.run_id);
        let admission_artifact: LiveExecutionAdmissionArtifact =
            serde_json::from_slice(&admission_raw).unwrap();
        let mut order = LiveExecutionOrderSnapshot {
            schema_version: LIVE_EXECUTION_ORDER_STATE_SCHEMA_VERSION.to_string(),
            admission_id: admission_artifact.admission_id.clone(),
            source_demo_run_id: admission_artifact.source_demo_run_id.clone(),
            strategy_intent_id: admission_artifact.strategy_intent_id.clone(),
            strategy_intent_sha256: admission_artifact.strategy_intent_sha256.clone(),
            sizing_decision_sha256: admission_artifact.sizing_decision_sha256.clone(),
            strategy_version_id: admission_artifact.strategy_version_id.clone(),
            instrument_id: admission_artifact.instrument_id.clone(),
            client_order_id: Some("S3LV007-001".to_string()),
            venue_order_id: Some("1001".to_string()),
            original_quantity: admission_artifact.quantity.clone(),
            filled_quantity: "0".to_string(),
            remaining_quantity: admission_artifact.quantity.clone(),
            status: "accepted".to_string(),
            terminal: false,
            new_orders_blocked: true,
            actual_submission_attempted: true,
            automatic_retry_attempted: false,
            cancel_attempted: false,
            replace_attempted: false,
            last_error: None,
            updated_at_unix_ms: admission_artifact.authorized_at_unix_ms + 1,
        };
        validate_execution_order_snapshot(&order, &admission_artifact).unwrap();
        order.automatic_retry_attempted = true;
        assert!(validate_execution_order_snapshot(&order, &admission_artifact).is_err());
        order.automatic_retry_attempted = false;
        order.original_quantity = "0.02".to_string();
        order.remaining_quantity = "0.02".to_string();
        assert!(validate_execution_order_snapshot(&order, &admission_artifact).is_err());
        order.original_quantity = admission_artifact.quantity.clone();
        order.filled_quantity = admission_artifact.quantity.clone();
        order.remaining_quantity = "0".to_string();
        order.cancel_attempted = true;
        for status in ["filled", "expired", "rejected"] {
            order.status = status.to_string();
            order.terminal = true;
            assert!(
                validate_execution_order_snapshot(&order, &admission_artifact).is_ok(),
                "{status}"
            );
        }

        let candidate_root = canonical_live_run_root(&fixture.state, false)
            .unwrap()
            .join(&ready.run_id);
        assert!(
            validate_execution_order_runtime_context(
                &candidate_root,
                LiveRunCandidateLifecycle::PreflightReady,
            )
            .is_err()
        );
        fs::write(
            candidate_root.join(LIVE_MARKET_DATA_NODE_CONFIG_FILE),
            "[live_market_data]\nmode = \"production-market-data\"\n",
        )
        .unwrap();
        assert!(
            validate_execution_order_runtime_context(
                &candidate_root,
                LiveRunCandidateLifecycle::MarketDataRunning,
            )
            .is_err()
        );
        fs::write(
            candidate_root.join(LIVE_MARKET_DATA_NODE_CONFIG_FILE),
            "[live_market_data]\nmode = \"production-market-data\"\n[live_execution]\nsingle_shot = true\n",
        )
        .unwrap();
        validate_execution_order_runtime_context(
            &candidate_root,
            LiveRunCandidateLifecycle::MarketDataRunning,
        )
        .unwrap();
        fs::remove_file(candidate_root.join(LIVE_MARKET_DATA_NODE_CONFIG_FILE)).unwrap();

        let duplicate = authorize_live_execution_with_source_validator(
            &fixture.state,
            &ready.run_id,
            &request,
            LiveExecutionApprovalRole::Owner,
            |_| Ok(execution_source()),
        )
        .unwrap_err();
        assert_eq!(duplicate.kind, ProductErrorKind::LiveConflict);

        let market_only_start = run_live_candidate_action(
            &fixture.state,
            &ready.run_id,
            &LiveRunCandidateActionRequest {
                run_id: ready.run_id.clone(),
                action: LiveRunCandidateAction::StartMarketData,
                user_confirmed: true,
            },
        )
        .unwrap_err();
        assert_eq!(market_only_start.kind, ProductErrorKind::LiveConflict);
        assert_eq!(
            market_only_start.field,
            "live_execution_admission_requires_execution_start"
        );
    }

    #[test]
    fn execution_approval_publication_recovers_every_write_boundary() {
        let cases = [
            (
                LiveExecutionApprovalRole::Owner,
                vec![
                    ApprovalPublicationStep::ExternalAnchor,
                    ApprovalPublicationStep::Approval,
                    ApprovalPublicationStep::Receipt,
                    ApprovalPublicationStep::WorkspaceHead,
                ],
            ),
            (
                LiveExecutionApprovalRole::Risk,
                vec![
                    ApprovalPublicationStep::ExternalAnchor,
                    ApprovalPublicationStep::Approval,
                    ApprovalPublicationStep::Receipt,
                    ApprovalPublicationStep::WorkspaceHead,
                ],
            ),
            (
                LiveExecutionApprovalRole::Operator,
                vec![
                    ApprovalPublicationStep::ExternalAnchor,
                    ApprovalPublicationStep::Approval,
                    ApprovalPublicationStep::Receipt,
                    ApprovalPublicationStep::WorkspaceHead,
                    ApprovalPublicationStep::Admission,
                    ApprovalPublicationStep::StateExternalAnchor,
                    ApprovalPublicationStep::State,
                    ApprovalPublicationStep::StateCommit,
                    ApprovalPublicationStep::StateReceipt,
                    ApprovalPublicationStep::StateHead,
                    ApprovalPublicationStep::StateWorkspaceHead,
                ],
            ),
        ];
        for (role, steps) in cases {
            for step in steps {
                let fixture = LiveRunFixture::new(&format!("approval-recovery-{role:?}-{step:?}"));
                let ready = create_preflight_ready_candidate(
                    &fixture,
                    &format!("product-recovery-{role:?}-{step:?}").to_lowercase(),
                );
                let request = execution_admission_request(&ready.run_id);
                for prior in [
                    LiveExecutionApprovalRole::Owner,
                    LiveExecutionApprovalRole::Risk,
                ] {
                    if prior == role {
                        break;
                    }
                    authorize_live_execution_with_source_validator(
                        &fixture.state,
                        &ready.run_id,
                        &request,
                        prior,
                        |_| Ok(execution_source()),
                    )
                    .unwrap();
                }
                let error = authorize_live_execution_with_publication_failure(
                    &fixture.state,
                    &ready.run_id,
                    &request,
                    role,
                    |_| Ok(execution_source()),
                    Some(step),
                )
                .unwrap_err();
                assert_eq!(
                    error.field, "live_execution_approval_publication_injected_failure",
                    "{role:?} {step:?}"
                );
                let source = execution_source();
                let mut drifted = source.1.clone();
                drifted.quantity = "0.2".to_string();
                let drift_error = authorize_live_execution_with_source_validator(
                    &fixture.state,
                    &ready.run_id,
                    &request,
                    role,
                    |_| Ok((source.0.clone(), drifted)),
                )
                .unwrap_err();
                assert_eq!(
                    drift_error.field, "live_execution_approval_publication",
                    "{role:?} {step:?}"
                );
                let recovered = authorize_live_execution_with_source_validator(
                    &fixture.state,
                    &ready.run_id,
                    &request,
                    role,
                    |_| Ok(execution_source()),
                )
                .unwrap();
                assert!(
                    !fixture
                        .root
                        .join("artifacts/live-runs")
                        .join(&ready.run_id)
                        .join(role.publication_stage_file())
                        .exists(),
                    "{role:?} {step:?}"
                );
                if role == LiveExecutionApprovalRole::Operator {
                    assert_eq!(
                        recovered.order_admission.status, "authorized_single_shot",
                        "{step:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn cancel_operator_approval_accepts_current_order_progress_from_owner_source() {
        let (fixture, run_id, admission, mut order, order_raw) =
            setup_cancel_publication("cancel-progress");
        let request = cancel_request(&run_id, &order_raw);
        let owner_stage = stage_cancel_publication(
            &fixture,
            &run_id,
            &request,
            LiveExecutionApprovalRole::Owner,
            &order_raw,
        );
        complete_live_execution_cancel_publication(&fixture.state, &run_id, &owner_stage, None)
            .unwrap();

        order.status = "partially_filled".to_string();
        order.filled_quantity = "0.00000400".to_string();
        order.remaining_quantity = "0.00000600".to_string();
        order.updated_at_unix_ms += 1;
        fs::write(
            fixture
                .root
                .join("artifacts/live-market-data-runtime")
                .join(&run_id)
                .join(LIVE_EXECUTION_ORDER_STATE_FILE),
            serde_json::to_vec_pretty(&order).unwrap(),
        )
        .unwrap();

        validate_execution_cancel_request_order(
            &request,
            &serde_json::from_slice(&order_raw).unwrap(),
            &order_raw,
            &order,
            &admission,
        )
        .unwrap();
        let operator_stage = stage_cancel_publication(
            &fixture,
            &run_id,
            &request,
            LiveExecutionApprovalRole::Operator,
            &order_raw,
        );
        complete_live_execution_cancel_publication(&fixture.state, &run_id, &operator_stage, None)
            .unwrap();
        let root = fixture.root.join("artifacts/live-runs").join(&run_id);
        assert_eq!(
            fs::read(root.join(LIVE_EXECUTION_CANCEL_SOURCE_ORDER_FILE)).unwrap(),
            order_raw
        );
        assert!(root.join(LIVE_EXECUTION_CANCEL_REQUEST_FILE).is_file());
    }

    #[test]
    fn cancel_publication_recovers_every_owner_and_operator_write_boundary() {
        for role in [
            LiveExecutionApprovalRole::Owner,
            LiveExecutionApprovalRole::Operator,
        ] {
            let steps: &[CancelPublicationStep] = if role == LiveExecutionApprovalRole::Owner {
                &[
                    CancelPublicationStep::SourceOrder,
                    CancelPublicationStep::ExternalAnchor,
                    CancelPublicationStep::Approval,
                    CancelPublicationStep::Receipt,
                    CancelPublicationStep::WorkspaceHead,
                ]
            } else {
                &[
                    CancelPublicationStep::ExternalAnchor,
                    CancelPublicationStep::Approval,
                    CancelPublicationStep::Receipt,
                    CancelPublicationStep::WorkspaceHead,
                    CancelPublicationStep::ControlRequest,
                ]
            };
            for step in steps {
                let name = format!("cancel-recovery-{role:?}-{step:?}");
                let (fixture, run_id, _admission, _order, order_raw) =
                    setup_cancel_publication(&name);
                let request = cancel_request(&run_id, &order_raw);
                if role == LiveExecutionApprovalRole::Operator {
                    let owner_stage = stage_cancel_publication(
                        &fixture,
                        &run_id,
                        &request,
                        LiveExecutionApprovalRole::Owner,
                        &order_raw,
                    );
                    complete_live_execution_cancel_publication(
                        &fixture.state,
                        &run_id,
                        &owner_stage,
                        None,
                    )
                    .unwrap();
                }
                let stage = stage_cancel_publication(&fixture, &run_id, &request, role, &order_raw);
                let error = complete_live_execution_cancel_publication(
                    &fixture.state,
                    &run_id,
                    &stage,
                    Some(*step),
                )
                .unwrap_err();
                assert_eq!(
                    error.field, "live_execution_cancel_publication_injected_failure",
                    "{role:?} {step:?}"
                );
                resume_live_execution_cancel_publication(
                    &fixture.state,
                    &run_id,
                    &request,
                    role,
                    None,
                )
                .unwrap();
                let root = fixture.root.join("artifacts/live-runs").join(&run_id);
                assert!(
                    !root
                        .join(cancel_publication_stage_file(role).unwrap())
                        .exists(),
                    "{role:?} {step:?}"
                );
                assert!(
                    root.join(match role {
                        LiveExecutionApprovalRole::Owner => {
                            LIVE_EXECUTION_CANCEL_OWNER_APPROVAL_FILE
                        }
                        LiveExecutionApprovalRole::Operator => {
                            LIVE_EXECUTION_CANCEL_OPERATOR_APPROVAL_FILE
                        }
                        LiveExecutionApprovalRole::Risk => unreachable!(),
                    })
                    .is_file(),
                    "{role:?} {step:?}"
                );
                if role == LiveExecutionApprovalRole::Operator {
                    assert!(root.join(LIVE_EXECUTION_CANCEL_REQUEST_FILE).is_file());
                }
            }
        }
    }

    #[test]
    fn cancel_publication_recovery_completes_after_order_becomes_terminal() {
        let (fixture, run_id, _admission, mut order, order_raw) =
            setup_cancel_publication("cancel-terminal-recovery");
        let request = cancel_request(&run_id, &order_raw);
        let stage = stage_cancel_publication(
            &fixture,
            &run_id,
            &request,
            LiveExecutionApprovalRole::Owner,
            &order_raw,
        );
        complete_live_execution_cancel_publication(
            &fixture.state,
            &run_id,
            &stage,
            Some(CancelPublicationStep::ExternalAnchor),
        )
        .unwrap_err();
        order.status = "filled".to_string();
        order.filled_quantity = order.original_quantity.clone();
        order.remaining_quantity = "0".to_string();
        order.terminal = true;
        order.updated_at_unix_ms += 1;
        fs::write(
            live_market_data_runtime_root(&fixture.state, &run_id)
                .unwrap()
                .join(LIVE_EXECUTION_ORDER_STATE_FILE),
            serde_json::to_vec_pretty(&order).unwrap(),
        )
        .unwrap();
        resume_live_execution_cancel_publication(
            &fixture.state,
            &run_id,
            &request,
            LiveExecutionApprovalRole::Owner,
            None,
        )
        .unwrap();
        assert!(
            !fixture
                .root
                .join("artifacts/live-runs")
                .join(&run_id)
                .join(LIVE_EXECUTION_CANCEL_OWNER_STAGE_FILE)
                .exists()
        );
    }

    #[test]
    fn live_execution_start_requires_an_independently_anchored_admission() {
        let fixture = LiveRunFixture::new("execution-start-without-admission");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000091");
        let error = run_live_candidate_action(
            &fixture.state,
            &ready.run_id,
            &LiveRunCandidateActionRequest {
                run_id: ready.run_id.clone(),
                action: LiveRunCandidateAction::StartExecution,
                user_confirmed: true,
            },
        )
        .unwrap_err();
        assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
        assert_eq!(error.field, "live_execution_admission");
        assert!(
            !fixture
                .root
                .join("artifacts/live-runs")
                .join(ready.run_id)
                .join(LIVE_MARKET_DATA_NODE_CONFIG_FILE)
                .exists()
        );
    }

    fn persist_starting_live_market_data_runtime(fixture: &LiveRunFixture, run_id: &str) -> String {
        let (manifest, manifest_raw) = load_live_run_manifest(&fixture.state, run_id).unwrap();
        let manifest_sha256 = sha256_ref(&manifest_raw);
        let (current, current_raw) =
            load_live_run_state(&fixture.state, run_id, &manifest_sha256).unwrap();
        let directory = open_absolute_directory_nofollow(
            &canonical_live_run_root(&fixture.state, false)
                .unwrap()
                .join(run_id),
        )
        .unwrap();
        write_live_market_data_node_config(&fixture.state, &directory, &manifest).unwrap();
        write_live_run_state(
            &fixture.state,
            run_id,
            &LiveRunCandidateState {
                schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                run_id: run_id.to_string(),
                source_manifest_sha256: manifest_sha256.clone(),
                revision: current.revision + 1,
                previous_state_sha256: Some(sha256_ref(&current_raw)),
                lifecycle: LiveRunCandidateLifecycle::Starting,
                preflight_sha256: current.preflight_sha256,
                execution_admission_sha256: current.execution_admission_sha256,
                execution_runtime_config_sha256: current.execution_runtime_config_sha256,
                stop_sha256: None,
                updated_at_unix_ms: unix_time_ms(),
            },
        )
        .unwrap();
        manifest_sha256
    }

    fn register_live_market_data_runtime(
        fixture: &LiveRunFixture,
        run_id: &str,
    ) -> SupervisorRegistryStore {
        let store = SupervisorRegistryStore::new(&fixture.state.registry_path);
        store
            .register_node(RegisterNodeRequest {
                node_id: run_id.to_string(),
                config_path: canonical_live_run_root(&fixture.state, false)
                    .unwrap()
                    .join(run_id)
                    .join(LIVE_MARKET_DATA_NODE_CONFIG_FILE),
                artifact_root: Some(live_market_data_runtime_root(&fixture.state, run_id).unwrap()),
            })
            .unwrap();
        store
    }

    fn claim_live_market_data_runtime(
        store: &SupervisorRegistryStore,
        run_id: &str,
        manifest_sha256: &str,
    ) {
        let process_generation = store.load().unwrap().nodes[run_id].process_generation;
        store
            .claim_run_ownership(
                run_id,
                SupervisorRunOwnership {
                    run_id: run_id.to_string(),
                    manifest_sha256: manifest_sha256.to_string(),
                    claimed_at_unix_ms: unix_time_ms(),
                    process_generation_at_claim: process_generation,
                    terminal: None,
                },
            )
            .unwrap();
    }

    fn persist_stopping_live_market_data_runtime(
        fixture: &LiveRunFixture,
        run_id: &str,
        manifest_sha256: &str,
    ) {
        let (current, current_raw) =
            load_live_run_state(&fixture.state, run_id, manifest_sha256).unwrap();
        write_live_run_state(
            &fixture.state,
            run_id,
            &LiveRunCandidateState {
                schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                run_id: run_id.to_string(),
                source_manifest_sha256: manifest_sha256.to_string(),
                revision: current.revision + 1,
                previous_state_sha256: Some(sha256_ref(&current_raw)),
                lifecycle: LiveRunCandidateLifecycle::Stopping,
                preflight_sha256: current.preflight_sha256,
                execution_admission_sha256: current.execution_admission_sha256,
                execution_runtime_config_sha256: current.execution_runtime_config_sha256,
                stop_sha256: None,
                updated_at_unix_ms: unix_time_ms(),
            },
        )
        .unwrap();
    }

    fn assert_failed_runtime_recovery(
        fixture: &LiveRunFixture,
        run_id: &str,
        terminal_expected: bool,
    ) {
        let failed = load_live_run_candidate(&fixture.state, run_id).unwrap();
        assert_eq!(failed.lifecycle, LiveRunCandidateLifecycle::Failed);
        assert!(!failed.runtime_started);
        assert!(
            load_active_live_run_pointer(&fixture.state)
                .unwrap()
                .is_none()
        );
        let registry = SupervisorRegistryStore::new(&fixture.state.registry_path)
            .load()
            .unwrap();
        let terminal = registry
            .nodes
            .get(run_id)
            .and_then(|record| record.run_ownership.get(run_id))
            .and_then(|ownership| ownership.terminal.as_ref());
        assert_eq!(terminal.is_some(), terminal_expected);
    }

    #[test]
    fn starting_runtime_interruption_before_registration_fails_and_releases_candidate() {
        let fixture = LiveRunFixture::new("starting-before-registration");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000006");
        let run_id = ready.run_id;
        persist_starting_live_market_data_runtime(&fixture, &run_id);

        let _workspace_lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        assert_failed_runtime_recovery(&fixture, &run_id, false);
    }

    #[test]
    fn starting_runtime_interruption_after_registration_removes_unowned_node() {
        let fixture = LiveRunFixture::new("starting-after-registration");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000007");
        let run_id = ready.run_id;
        persist_starting_live_market_data_runtime(&fixture, &run_id);
        register_live_market_data_runtime(&fixture, &run_id);

        let _workspace_lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        assert_failed_runtime_recovery(&fixture, &run_id, false);
        assert!(
            !SupervisorRegistryStore::new(&fixture.state.registry_path)
                .load()
                .unwrap()
                .nodes
                .contains_key(&run_id)
        );
    }

    #[cfg(unix)]
    #[test]
    fn starting_runtime_interruption_after_process_start_stops_and_anchors_failure() {
        let _test_lock = lock_live_runtime_process_tests();
        let mut fixture = LiveRunFixture::new("starting-after-process-start");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000008");
        let run_id = ready.run_id;
        fixture.install_live_market_data_node(&run_id);
        let manifest_sha256 = persist_starting_live_market_data_runtime(&fixture, &run_id);
        let store = register_live_market_data_runtime(&fixture, &run_id);
        claim_live_market_data_runtime(&store, &run_id, &manifest_sha256);
        store
            .start_node_process_for_run(
                &StartNodeRequest {
                    node_id: run_id.clone(),
                    ntpro_node_bin: fixture.state.ntpro_node_bin.clone(),
                    startup_timeout: Duration::from_millis(LIVE_MARKET_DATA_STARTUP_TIMEOUT_MS),
                    node_max_runtime: Duration::from_millis(3_600_000),
                    node_heartbeat_interval: Duration::from_millis(1_000),
                    node_parent_pid: Some(std::process::id()),
                    node_shutdown_timeout: Duration::from_millis(5_000),
                },
                &run_id,
                &manifest_sha256,
            )
            .unwrap();

        let _workspace_lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        assert_failed_runtime_recovery(&fixture, &run_id, true);
    }

    #[cfg(unix)]
    #[test]
    fn live_market_data_runtime_starts_and_stops_without_execution_capability() {
        let _test_lock = lock_live_runtime_process_tests();
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

    #[cfg(unix)]
    #[test]
    fn stopping_runtime_interruption_is_stopped_anchored_and_released() {
        let _test_lock = lock_live_runtime_process_tests();
        let mut fixture = LiveRunFixture::new("stopping-interruption");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000009");
        let run_id = ready.run_id;
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
        assert_eq!(
            running.lifecycle,
            LiveRunCandidateLifecycle::MarketDataRunning
        );
        let manifest_sha256 = SupervisorRegistryStore::new(&fixture.state.registry_path)
            .load()
            .unwrap()
            .nodes[&run_id]
            .run_ownership[&run_id]
            .manifest_sha256
            .clone();
        persist_stopping_live_market_data_runtime(&fixture, &run_id, &manifest_sha256);

        let _workspace_lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        let stopped = load_live_run_candidate(&fixture.state, &run_id).unwrap();
        assert_eq!(stopped.lifecycle, LiveRunCandidateLifecycle::Stopped);
        assert!(!stopped.runtime_started);
        assert!(
            load_active_live_run_pointer(&fixture.state)
                .unwrap()
                .is_none()
        );
        let registry = SupervisorRegistryStore::new(&fixture.state.registry_path)
            .load()
            .unwrap();
        assert_eq!(
            registry.nodes[&run_id].run_ownership[&run_id]
                .terminal
                .as_ref()
                .map(|terminal| terminal.lifecycle.as_str()),
            Some("stopped")
        );
    }

    #[cfg(unix)]
    #[test]
    fn stopped_runtime_interruption_retries_terminal_anchor_and_pointer_cleanup() {
        let _test_lock = lock_live_runtime_process_tests();
        let mut fixture = LiveRunFixture::new("stopped-interruption");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000010");
        let run_id = ready.run_id;
        fixture.install_live_market_data_node(&run_id);
        run_live_candidate_action(
            &fixture.state,
            &run_id,
            &LiveRunCandidateActionRequest {
                run_id: run_id.clone(),
                action: LiveRunCandidateAction::StartMarketData,
                user_confirmed: true,
            },
        )
        .unwrap();
        let store = SupervisorRegistryStore::new(&fixture.state.registry_path);
        let manifest_sha256 = store.load().unwrap().nodes[&run_id].run_ownership[&run_id]
            .manifest_sha256
            .clone();
        persist_stopping_live_market_data_runtime(&fixture, &run_id, &manifest_sha256);
        store
            .stop_node_process_for_run(
                &StopNodeRequest {
                    node_id: run_id.clone(),
                    stop_timeout: Duration::from_secs(5),
                },
                &run_id,
                &manifest_sha256,
            )
            .unwrap();
        let (stopping, stopping_raw) =
            load_live_run_state(&fixture.state, &run_id, &manifest_sha256).unwrap();
        complete_stopping_live_run_state(
            &fixture.state,
            &run_id,
            &manifest_sha256,
            &stopping,
            &stopping_raw,
        )
        .unwrap();
        assert!(
            store.load().unwrap().nodes[&run_id].run_ownership[&run_id]
                .terminal
                .is_none()
        );
        assert!(
            load_active_live_run_pointer(&fixture.state)
                .unwrap()
                .is_some()
        );

        let _workspace_lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        assert!(
            load_active_live_run_pointer(&fixture.state)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store.load().unwrap().nodes[&run_id].run_ownership[&run_id]
                .terminal
                .as_ref()
                .map(|terminal| terminal.lifecycle.as_str()),
            Some("stopped")
        );
    }

    #[cfg(unix)]
    #[test]
    fn live_market_data_runtime_external_exit_is_failed_anchored_and_released() {
        let _test_lock = lock_live_runtime_process_tests();
        let mut fixture = LiveRunFixture::new("market-data-runtime-external-exit");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000004");
        let run_id = ready.run_id;
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
        assert_eq!(
            running.lifecycle,
            LiveRunCandidateLifecycle::MarketDataRunning
        );

        let store = SupervisorRegistryStore::new(&fixture.state.registry_path);
        let pid = store.load().unwrap().nodes[&run_id]
            .process
            .pid
            .value
            .expect("running fixture should publish its process ID");
        if crate::process::process_is_alive(pid) {
            crate::process::send_kill(pid).unwrap();
        }
        assert!(crate::process::wait_for_process_exit(
            pid,
            Duration::from_secs(10)
        ));
        let _workspace_lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();

        let failed = load_live_run_candidate(&fixture.state, &run_id).unwrap();
        assert_eq!(failed.lifecycle, LiveRunCandidateLifecycle::Failed);
        assert!(!failed.runtime_started);
        assert!(!failed.market_data_connected);
        assert!(
            load_active_live_run_candidates(&fixture.state)
                .unwrap()
                .is_empty()
        );
        let registry = store.load().unwrap();
        let terminal = registry.nodes[&run_id].run_ownership[&run_id]
            .terminal
            .as_ref()
            .unwrap();
        assert_eq!(terminal.lifecycle, "failed");
    }

    #[cfg(unix)]
    #[test]
    fn failed_live_runtime_retries_terminal_anchor_and_pointer_cleanup() {
        let _test_lock = lock_live_runtime_process_tests();
        let mut fixture = LiveRunFixture::new("market-data-runtime-failed-retry");
        let ready =
            create_preflight_ready_candidate(&fixture, "product-0000000000000001-0000000000000005");
        let run_id = ready.run_id;
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
        let store = SupervisorRegistryStore::new(&fixture.state.registry_path);
        let registry = store.load().unwrap();
        let manifest_sha256 = registry.nodes[&run_id].run_ownership[&run_id]
            .manifest_sha256
            .clone();
        store
            .stop_node_process_for_run(
                &StopNodeRequest {
                    node_id: run_id.clone(),
                    stop_timeout: Duration::from_secs(5),
                },
                &run_id,
                &manifest_sha256,
            )
            .unwrap();
        let (current, current_raw) =
            load_live_run_state(&fixture.state, &run_id, &manifest_sha256).unwrap();
        let stop = LiveRunStopArtifact {
            schema_version: LIVE_RUN_STOP_SCHEMA_VERSION.to_string(),
            run_id: run_id.clone(),
            source_manifest_sha256: manifest_sha256.clone(),
            source_preflight_sha256: current.preflight_sha256.clone(),
            stopped_at_unix_ms: unix_time_ms(),
            manual_stop: false,
            order_endpoint_access_attempted: false,
            execution_adapter_send_attempted: false,
            real_orders_submitted: false,
            execution_order_sha256: None,
        };
        let stop_raw = serde_json::to_vec_pretty(&stop).unwrap();
        fs::write(
            canonical_live_run_root(&fixture.state, false)
                .unwrap()
                .join(&run_id)
                .join("stop.json"),
            &stop_raw,
        )
        .unwrap();
        write_live_run_state(
            &fixture.state,
            &run_id,
            &LiveRunCandidateState {
                schema_version: LIVE_RUN_STATE_SCHEMA_VERSION.to_string(),
                run_id: run_id.clone(),
                source_manifest_sha256: manifest_sha256,
                revision: current.revision + 1,
                previous_state_sha256: Some(sha256_ref(&current_raw)),
                lifecycle: LiveRunCandidateLifecycle::Failed,
                preflight_sha256: current.preflight_sha256,
                execution_admission_sha256: current.execution_admission_sha256,
                execution_runtime_config_sha256: current.execution_runtime_config_sha256,
                stop_sha256: Some(sha256_ref(&stop_raw)),
                updated_at_unix_ms: unix_time_ms(),
            },
        )
        .unwrap();
        assert_eq!(
            running.lifecycle,
            LiveRunCandidateLifecycle::MarketDataRunning
        );
        assert!(
            load_active_live_run_pointer(&fixture.state)
                .unwrap()
                .is_some()
        );
        assert!(
            store.load().unwrap().nodes[&run_id].run_ownership[&run_id]
                .terminal
                .is_none()
        );

        let _workspace_lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();

        assert!(
            load_active_live_run_pointer(&fixture.state)
                .unwrap()
                .is_none()
        );
        let registry = store.load().unwrap();
        assert_eq!(
            registry.nodes[&run_id].run_ownership[&run_id]
                .terminal
                .as_ref()
                .map(|terminal| terminal.lifecycle.as_str()),
            Some("failed")
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
        let _workspace_lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        reconcile_exited_live_market_data_runtime(&fixture.state, &run_id).unwrap();
        assert_eq!(
            load_live_run_candidate(&fixture.state, &run_id)
                .unwrap()
                .lifecycle,
            LiveRunCandidateLifecycle::Failed
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
                    sizing: sizing_preflight(),
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
            sizing: sizing_preflight(),
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
                    sizing: sizing_preflight(),
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
    fn live_mutation_lock_refuses_an_active_owner_without_publishing_candidate() {
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
    fn live_mutation_lock_rewrites_stale_diagnostics_after_lock_release() {
        let fixture = LiveRunFixture::new("dead-mutation-lock");
        canonical_live_run_root(&fixture.state, true).unwrap();
        let canonical_root = fs::canonicalize(&fixture.root).unwrap();
        let artifact_root =
            open_absolute_directory_nofollow(&canonical_root.join("artifacts")).unwrap();
        let stale = LiveRunMutationLockArtifact {
            schema_version: LIVE_RUN_MUTATION_LOCK_SCHEMA_VERSION.to_string(),
            pid: std::process::id(),
            process_start_time_secs: u64::MAX,
            acquired_at_unix_ms: 1,
        };
        write_new_run_file(
            &artifact_root,
            LIVE_RUN_MUTATION_LOCK_FILE,
            &serde_json::to_vec_pretty(&stale).unwrap(),
        )
        .unwrap();
        let lock = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        let current: LiveRunMutationLockArtifact = serde_json::from_slice(
            &fs::read(
                fixture
                    .root
                    .join("artifacts")
                    .join(LIVE_RUN_MUTATION_LOCK_FILE),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(current.pid, std::process::id());
        assert_ne!(current.process_start_time_secs, u64::MAX);
        assert_eq!(
            process_start_time_secs(current.pid),
            Some(current.process_start_time_secs)
        );
        drop(lock);
        assert!(
            fixture
                .root
                .join("artifacts")
                .join(LIVE_RUN_MUTATION_LOCK_FILE)
                .exists()
        );
    }

    #[test]
    fn live_mutation_lock_crash_helper() {
        let Ok(root) = std::env::var("NTPRO_LIVE_RUN_CRASH_HELPER_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        fs::create_dir_all(root.join("supervisor")).unwrap();
        fs::create_dir_all(root.join("artifacts")).unwrap();
        let state = DashboardServerState {
            registry_path: root.join("supervisor/registry.json"),
            workflow_root: None,
            ntpro_node_bin: PathBuf::from("missing-ntpro-node"),
            lifecycle_action_lock: std::sync::Arc::new(std::sync::Mutex::new(())),
            backtest_creation_gate: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
            live_run_audit_anchor: std::sync::Arc::new(
                super::live_run_anchor::LiveRunAuditAnchorClient::memory_for_test(),
            ),
        };
        let live_root = canonical_live_run_root(&state, true).unwrap();
        let _lock = acquire_live_run_mutation_lock(&state).unwrap();
        let run_root = live_root.join("crash-recovery-run");
        fs::create_dir(&run_root).unwrap();
        fs::write(
            run_root.join(LIVE_EXECUTION_OWNER_APPROVAL_STAGE_FILE),
            b"persisted-before-crash",
        )
        .unwrap();
        fs::write(root.join("crash-helper-ready"), b"ready").unwrap();
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    #[test]
    fn live_mutation_lock_recovers_after_forced_process_termination() {
        let fixture = LiveRunFixture::new("mutation-lock-process-crash");
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("dashboard::product_api::live_run::tests::live_mutation_lock_crash_helper")
            .arg("--nocapture")
            .env("NTPRO_LIVE_RUN_CRASH_HELPER_ROOT", &fixture.root)
            .spawn()
            .unwrap();
        let ready = fixture.root.join("crash-helper-ready");
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while !ready.exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
        }
        assert!(ready.exists());
        assert!(crate::process::process_is_alive(child.id()));
        crate::process::send_kill(child.id()).unwrap();
        assert!(crate::process::wait_for_process_exit(
            child.id(),
            Duration::from_secs(10)
        ));
        let _ = child.wait();
        let stage = fixture
            .root
            .join("artifacts/live-runs/crash-recovery-run")
            .join(LIVE_EXECUTION_OWNER_APPROVAL_STAGE_FILE);
        assert_eq!(fs::read(&stage).unwrap(), b"persisted-before-crash");
        let recovered = acquire_live_run_mutation_lock(&fixture.state).unwrap();
        assert!(stage.exists());
        drop(recovered);
    }

    #[test]
    fn live_mutation_lock_allows_only_one_concurrent_recovery_writer() {
        let fixture = LiveRunFixture::new("mutation-lock-concurrent-recovery");
        canonical_live_run_root(&fixture.state, true).unwrap();
        let state = std::sync::Arc::new(fixture.independent_state());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let release = std::sync::Arc::new(std::sync::Barrier::new(2));
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut handles = Vec::new();
        for _ in 0..2 {
            let state = state.clone();
            let barrier = barrier.clone();
            let release = release.clone();
            let sender = sender.clone();
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                match acquire_live_run_mutation_lock(&state) {
                    Ok(lock) => {
                        sender.send(true).unwrap();
                        release.wait();
                        drop(lock);
                    }
                    Err(error) => {
                        assert_eq!(error.kind, ProductErrorKind::LiveConflict);
                        sender.send(false).unwrap();
                    }
                }
            }));
        }
        barrier.wait();
        let first = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        let second = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(
            [first, second].into_iter().filter(|value| *value).count(),
            1
        );
        release.wait();
        for handle in handles {
            handle.join().unwrap();
        }
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
                    sizing: sizing_preflight(),
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
                            sizing: sizing_preflight(),
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
