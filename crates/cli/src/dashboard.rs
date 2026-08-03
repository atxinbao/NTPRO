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

//! Dashboard read-model DTOs and local HTTP server for local Rust-only artifacts.

use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path as FsPath, PathBuf},
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, ensure};
use aws_lc_rs::digest;
use axum::{Json, http::StatusCode};
use nautilus_live::status::{
    ConnectionStatus, HealthStatus, LifecycleStatus, NodeStatus, ProcessMode, RiskTradingState,
    SnapshotAvailability, SnapshotValue,
};
use nautilus_trading::strategy::v04_smoke::{
    V04_BINANCE_EMA_MOCK_LIFECYCLE_ID, V04_BINANCE_EMA_RISK_SMOKE_ID, v04_ema_smoke_from_csv,
    v04_rsi_smoke_from_csv,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    strategy_session::{StrategySessionArtifactAuditHealth, audit_strategy_session_artifacts},
    supervisor::{
        NodeMetrics, PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION,
        RegistryArtifactState, StartNodeRequest, StopNodeRequest, SupervisorNodeRecord,
        SupervisorProcessState, SupervisorRegistry, SupervisorRegistryStore,
    },
    workflow_contract::{
        TESTNET_AUTHENTICATED_READONLY_PROBE_ARTIFACT_PATH,
        TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH, TESTNET_HTTP_CONNECTIVITY_PROBE_ARTIFACT_PATH,
        TESTNET_WEBSOCKET_PROBE_ARTIFACT_PATH, WorkflowManifest, WorkflowManifestArtifact,
    },
};

mod institution_workbench;
mod mvp_status_api;
mod rendering;
mod server;
mod trader_terminal_api;

use institution_workbench::{
    INSTITUTION_WORKBENCH_CSS, INSTITUTION_WORKBENCH_HTML, INSTITUTION_WORKBENCH_JS,
};
use rendering::{DASHBOARD_CSS, DASHBOARD_HTML, DASHBOARD_JS};
#[cfg(test)]
use server::dashboard_router;
pub(crate) use server::run_dashboard_command;

pub const DASHBOARD_SNAPSHOT_SCHEMA_VERSION: &str = "ntpro.dashboard_snapshot.v1";
const DASHBOARD_DATA_RECONNECT_UNSUPPORTED_MESSAGE: &str =
    "本地沙盒仅记录数据源重连为不支持，不会连接真实交易所或真实 adapter";
const DASHBOARD_EXECUTION_RECONNECT_UNSUPPORTED_MESSAGE: &str =
    "本地沙盒仅记录执行网关重连为不支持，不会连接真实交易所或真实 adapter";
const V04_BINANCE_SPOT_BARS_CSV: &str =
    include_str!("../../adapters/binance/test_data/v04/binance_spot_bars.csv");
const V04_BINANCE_MOCK_ORDER_LIFECYCLE_JSONL: &str =
    include_str!("../../adapters/binance/test_data/v04/mock_order_lifecycle.jsonl");
const V04_BINANCE_MOCK_ORDER_LIFECYCLE_PATH: &str =
    "crates/adapters/binance/test_data/v04/mock_order_lifecycle.jsonl";
const V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID: &str = "O-V04-003";
const V04_BINANCE_RISK_REJECTION_FIXTURE_REASON: &str = "mock_reject_requested";
const V04_BINANCE_RISK_REJECTION_REASON: &str = "TradingState::HALTED";
const PRODUCTION_SHADOW_MANIFEST_SCHEMA_VERSION: &str = "ntpro.v111_production_shadow_manifest.v1";
const PRODUCTION_ACCOUNT_SNAPSHOT_SCHEMA_VERSION: &str =
    "ntpro.v110_authenticated_account_snapshot_contract.v1";
const PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION: &str =
    "ntpro.v120_authenticated_account_snapshot_online_read.v1";
const PRODUCTION_PUBLIC_READ_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v110_production_public_read_probe.v1";
const PRODUCTION_PUBLIC_ONLINE_READ_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v120_production_public_online_read_probe.v1";
const PRODUCTION_SHADOW_INTENT_SCHEMA_VERSION: &str = "ntpro.v110_shadow_execution_intent.v1";
const PRODUCTION_SHADOW_PORTFOLIO_SCHEMA_VERSION: &str = "ntpro.v110_shadow_portfolio_snapshot.v1";
const PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION: &str =
    "ntpro.v120_shadow_portfolio_runtime.v1";
const PRODUCTION_SHADOW_LIFECYCLE_SCHEMA_VERSION: &str = "ntpro.v110_order_lifecycle_state.v1";
const PRODUCTION_SHADOW_RECONCILIATION_SCHEMA_VERSION: &str = "ntpro.v110_reconciliation_event.v1";
const PRODUCTION_SHADOW_STRATEGY_SESSION_EVENT_SCHEMA_VERSION: &str =
    "ntpro.v120_shadow_strategy_session_event.v1";
const PRODUCTION_READONLY_RECONCILIATION_EVENT_SCHEMA_VERSION: &str =
    "ntpro.v120_readonly_reconciliation_event.v1";
const PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_RELATIVE_PATH: &str =
    "v0_13/kill_switch_approval_artifact.json";
const LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION: &str =
    "ntpro.v140_live_alpha_dry_run_order_gate.v1";
const LIVE_ALPHA_RISK_PREFLIGHT_SCHEMA_VERSION: &str = "ntpro.v140_live_alpha_risk_preflight.v1";
const PRODUCTION_ORDER_STATE_READONLY_PROOF_SCHEMA_VERSION: &str =
    "ntpro.v140_production_order_state_readonly_proof.v1";
const LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_SCHEMA_VERSION: &str =
    "ntpro.v150_live_alpha_manual_approval_lifecycle.v1";
const LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION: &str =
    "ntpro.v150_live_alpha_order_request_preview.v1";
const LIVE_ALPHA_EXECUTION_DRY_RUN_SCHEMA_VERSION: &str =
    "ntpro.v150_live_alpha_execution_dry_run.v1";
const LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION: &str =
    "ntpro.v150_live_alpha_kill_switch_runtime_gate.v1";
const PRODUCTION_MUTATION_RUNTIME_GATE_SCHEMA_VERSION: &str =
    "ntpro.v160_production_mutation_runtime_gate.v1";
const PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION: &str =
    "ntpro.v160_production_mutation_signing_approval.v1";
const PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION: &str =
    "ntpro.v160_production_mutation_request_builder.v1";
const PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION: &str =
    "ntpro.v160_production_mutation_guarded_send.v1";
const PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION: &str =
    "ntpro.v160_production_mutation_response_redaction.v1";
const PRODUCTION_MUTATION_ORDER_STATE_READBACK_SCHEMA_VERSION: &str =
    "ntpro.v160_production_mutation_order_state_readback.v1";
const PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION: &str =
    "ntpro.v160_production_mutation_audit_trail.v1";
const PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION: &str =
    "ntpro.v160_production_mutation_failure_semantics.v1";
const PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_SCHEMA_VERSION: &str =
    "ntpro.v170_production_mutation_local_order_ledger.v1";
const PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION: &str =
    "ntpro.v170_production_mutation_exchange_readback_mapper.v1";
const PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION: &str =
    "ntpro.v170_production_mutation_reconciliation_classifier.v1";
const PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_SCHEMA_VERSION: &str =
    "ntpro.v170_production_mutation_orphan_order_detector.v1";
const PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_SCHEMA_VERSION: &str =
    "ntpro.v180_cancel_request_preview.v1";
const PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION: &str = "ntpro.v180_cancel_risk_gate.v1";
const PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION: &str =
    "ntpro.v180_manual_owner_approval_lifecycle.v1";
const PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION: &str =
    "ntpro.v180_cancel_response_redaction.v1";
const PRODUCTION_MUTATION_POST_CANCEL_READBACK_SCHEMA_VERSION: &str =
    "ntpro.v180_post_cancel_readback.v1";
const PRODUCTION_MUTATION_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_SCHEMA_VERSION: &str =
    "ntpro.v180_cancel_recovery_incident_audit_closeout.v1";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_owner_approval_lifecycle.v1";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_single_shot.v1";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_readback_reconciliation.v1";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_FAILURE_EVIDENCE_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_failure_evidence.v1";
const LIVE_ALPHA_DRY_RUN_ORDER_GATE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_14/live_alpha_dry_run_order_gate.json";
const LIVE_ALPHA_RISK_PREFLIGHT_ARTIFACT_RELATIVE_PATH: &str =
    "v0_14/live_alpha_risk_preflight.json";
const PRODUCTION_ORDER_STATE_READONLY_PROOF_ARTIFACT_RELATIVE_PATH: &str =
    "v0_14/production_order_state_readonly_proof.json";
const LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_15/manual_approval_lifecycle.json";
const LIVE_ALPHA_ORDER_REQUEST_PREVIEW_ARTIFACT_RELATIVE_PATH: &str =
    "v0_15/live_alpha_order_request_preview.json";
const LIVE_ALPHA_EXECUTION_DRY_RUN_ARTIFACT_RELATIVE_PATH: &str =
    "v0_15/live_alpha_execution_dry_run.json";
const LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_15/kill_switch_runtime_gate.json";
const PRODUCTION_MUTATION_RUNTIME_GATE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_16/production_mutation_runtime_gate.json";
const PRODUCTION_MUTATION_SIGNING_APPROVAL_ARTIFACT_RELATIVE_PATH: &str =
    "v0_16/production_mutation_signing_approval.json";
const PRODUCTION_MUTATION_REQUEST_BUILDER_ARTIFACT_RELATIVE_PATH: &str =
    "v0_16/production_mutation_request_builder.json";
const PRODUCTION_MUTATION_GUARDED_SEND_ARTIFACT_RELATIVE_PATH: &str =
    "v0_16/production_mutation_guarded_send.json";
const PRODUCTION_MUTATION_RESPONSE_REDACTION_ARTIFACT_RELATIVE_PATH: &str =
    "v0_16/production_mutation_response_redaction.json";
const PRODUCTION_MUTATION_ORDER_STATE_READBACK_ARTIFACT_RELATIVE_PATH: &str =
    "v0_16/production_mutation_order_state_readback.json";
const PRODUCTION_MUTATION_AUDIT_TRAIL_ARTIFACT_RELATIVE_PATH: &str =
    "v0_16/production_mutation_audit_trail.json";
const PRODUCTION_MUTATION_FAILURE_SEMANTICS_ARTIFACT_RELATIVE_PATH: &str =
    "v0_16/production_mutation_failure_semantics.json";
const PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_ARTIFACT_RELATIVE_PATH: &str =
    "v0_17/production_mutation_local_order_ledger.json";
const PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_ARTIFACT_RELATIVE_PATH: &str =
    "v0_17/production_mutation_exchange_readback_mapper.json";
const PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_ARTIFACT_RELATIVE_PATH: &str =
    "v0_17/production_mutation_reconciliation_classifier.json";
const PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_ARTIFACT_RELATIVE_PATH: &str =
    "v0_17/production_mutation_orphan_order_detector.json";
const PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_ARTIFACT_RELATIVE_PATH: &str =
    "v0_18/cancel_request_preview.json";
const PRODUCTION_MUTATION_CANCEL_RISK_GATE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_18/cancel_risk_gate.json";
const PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_18/manual_owner_approval_lifecycle.json";
const PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_ARTIFACT_RELATIVE_PATH: &str =
    "v0_18/cancel_response_redaction.json";
const PRODUCTION_MUTATION_POST_CANCEL_READBACK_ARTIFACT_RELATIVE_PATH: &str =
    "v0_18/post_cancel_readback.json";
const PRODUCTION_MUTATION_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_ARTIFACT_RELATIVE_PATH: &str =
    "v0_18/cancel_recovery_incident_audit_closeout.json";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_19/actual_cancel_owner_approval_lifecycle.json";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_ARTIFACT_RELATIVE_PATH: &str =
    "v0_19/actual_cancel_single_shot.json";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_ARTIFACT_RELATIVE_PATH: &str =
    "v0_19/actual_cancel_readback_reconciliation.json";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_FAILURE_EVIDENCE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_19/actual_cancel_failure_evidence.json";
const PRODUCTION_ORDER_LIFECYCLE_SUBMIT_CANDIDATE_ARTIFACT_RELATIVE_PATH: &str =
    "v0_20/guarded_submit_candidate.json";
const PRODUCTION_ORDER_LIFECYCLE_RESPONSE_REDACTION_ARTIFACT_RELATIVE_PATH: &str =
    "v0_20/submit_response_redaction.json";
const PRODUCTION_ORDER_LIFECYCLE_READBACK_RECONCILIATION_ARTIFACT_RELATIVE_PATH: &str =
    "v0_20/submit_readback_reconciliation.json";
const PRODUCTION_ORDER_LIFECYCLE_FAILURE_NO_RETRY_ARTIFACT_RELATIVE_PATH: &str =
    "v0_20/failure_no_retry_evidence.json";
const PRODUCTION_ORDER_LIFECYCLE_AUDIT_CLOSEOUT_ARTIFACT_RELATIVE_PATH: &str =
    "v0_20/order_lifecycle_audit_closeout.json";
const PRODUCTION_ORDER_LIFECYCLE_SUBMIT_CANDIDATE_SCHEMA_VERSION: &str =
    "ntpro.v200_guarded_single_shot_submit_candidate.v1";
const PRODUCTION_ORDER_LIFECYCLE_RESPONSE_REDACTION_SCHEMA_VERSION: &str =
    "ntpro.v200_submit_response_redaction.v1";
const PRODUCTION_ORDER_LIFECYCLE_READBACK_RECONCILIATION_SCHEMA_VERSION: &str =
    "ntpro.v200_submit_readback_reconciliation.v1";
const PRODUCTION_ORDER_LIFECYCLE_FAILURE_NO_RETRY_SCHEMA_VERSION: &str =
    "ntpro.v200_failure_no_retry_evidence.v1";
const PRODUCTION_ORDER_LIFECYCLE_AUDIT_CLOSEOUT_SCHEMA_VERSION: &str =
    "ntpro.v200_order_lifecycle_audit_closeout.v1";
const TRADER_TERMINAL_READ_MODEL_ARTIFACT_RELATIVE_PATH: &str =
    "v0_21/unified_read_model_snapshot.json";
const UNIFIED_READ_MODEL_CONTRACT_VERSION: &str = "ntpro.v210.unified_read_model.v1";
const UNIFIED_READ_MODEL_SCHEMA_VERSION: &str = "ntpro.v210.unified_read_model.schema.v1";
const V24_ORDER_CONTROL_PREVIEW_COMPONENT: &str = "v24_order_control_preview";
const V25_MONITORING_OBSERVABILITY_COMPONENT: &str = "v25_monitoring_observability";
const V25_ALERT_TAXONOMY_COMPONENT: &str = "v25_alert_taxonomy_routing";
const V25_INCIDENT_LIFECYCLE_COMPONENT: &str = "v25_incident_lifecycle";
const V25_RUNBOOK_AUDIT_COMPONENT: &str = "v25_runbook_audit";
const V25_DR_PREVIEW_COMPONENT: &str = "v25_dr_preview_drill";
const V25_DASHBOARD_SURFACE_COMPONENTS: [&str; 5] = [
    V25_MONITORING_OBSERVABILITY_COMPONENT,
    V25_ALERT_TAXONOMY_COMPONENT,
    V25_INCIDENT_LIFECYCLE_COMPONENT,
    V25_RUNBOOK_AUDIT_COMPONENT,
    V25_DR_PREVIEW_COMPONENT,
];
const V26_PERMISSION_BOUNDARY_COMPONENT: &str = "v26_permission_boundary";
const V26_OPERATION_AUDIT_COMPONENT: &str = "v26_operation_audit";
const V26_DEPLOYMENT_PROVENANCE_COMPONENT: &str = "v26_deployment_provenance";
const V26_UPGRADE_ROLLBACK_COMPONENT: &str = "v26_upgrade_rollback";
const V26_STABILITY_SLO_COMPONENT: &str = "v26_stability_slo";
const V26_DASHBOARD_ADMIN_COMPONENTS: [&str; 5] = [
    V26_PERMISSION_BOUNDARY_COMPONENT,
    V26_OPERATION_AUDIT_COMPONENT,
    V26_DEPLOYMENT_PROVENANCE_COMPONENT,
    V26_UPGRADE_ROLLBACK_COMPONENT,
    V26_STABILITY_SLO_COMPONENT,
];
const TRADER_TERMINAL_READ_MODEL_REQUIRED_COMPONENTS: [&str; 17] = [
    "account",
    "positions",
    "orders",
    "fills",
    "risk",
    "lifecycle_status",
    "operation_entry",
    V25_MONITORING_OBSERVABILITY_COMPONENT,
    V25_ALERT_TAXONOMY_COMPONENT,
    V25_INCIDENT_LIFECYCLE_COMPONENT,
    V25_RUNBOOK_AUDIT_COMPONENT,
    V25_DR_PREVIEW_COMPONENT,
    V26_PERMISSION_BOUNDARY_COMPONENT,
    V26_OPERATION_AUDIT_COMPONENT,
    V26_DEPLOYMENT_PROVENANCE_COMPONENT,
    V26_UPGRADE_ROLLBACK_COMPONENT,
    V26_STABILITY_SLO_COMPONENT,
];

#[derive(Clone, Debug)]
struct DashboardServerState {
    registry_path: PathBuf,
    workflow_root: Option<PathBuf>,
    ntpro_node_bin: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DashboardServerMetadata {
    registry_path: String,
    workflow_root: Option<String>,
    local_only: bool,
}

const DASHBOARD_ACTION_TIMEOUT_MS: u64 = 5_000;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<Value>)>;
type ApiStatusResult<T> = Result<(StatusCode, Json<T>), (StatusCode, Json<Value>)>;
type SnapshotLoadResult = Result<DashboardSnapshot, (StatusCode, Json<Value>)>;

fn control_action_response(
    state: &DashboardServerState,
    node_id: &str,
    action: &str,
) -> ApiStatusResult<ControlActionResponse> {
    let started_at = generated_at_now();
    let snapshot = load_dashboard_snapshot(state)?;
    let Some(node) = snapshot.nodes.iter().find(|node| node.node_id == node_id) else {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state: LifecycleStatus::Unknown,
                current_state: LifecycleStatus::Unknown,
                started_at,
                error_code: DashboardValue::available("node_not_found".to_string()),
                message: DashboardValue::available("本地监督器注册表中没有找到该节点".to_string()),
            })),
        ));
    };
    let previous_state = node.lifecycle_state;

    match action {
        "start" if previous_state != LifecycleStatus::Stopped => Ok((
            StatusCode::CONFLICT,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available("invalid_lifecycle_state".to_string()),
                message: DashboardValue::available("只有已停止的节点可以启动".to_string()),
            })),
        )),
        "stop"
            if !matches!(
                previous_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            ) =>
        {
            Ok((
                StatusCode::CONFLICT,
                Json(action_response(ControlActionResponseParts {
                    action,
                    node_id,
                    status: ControlActionStatus::Rejected,
                    previous_state,
                    current_state: previous_state,
                    started_at,
                    error_code: DashboardValue::available("invalid_lifecycle_state".to_string()),
                    message: DashboardValue::available(
                        "只有运行中或已暂停的节点可以停止".to_string(),
                    ),
                })),
            ))
        }
        "pause" if previous_state != LifecycleStatus::Running => Ok((
            StatusCode::CONFLICT,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available("invalid_lifecycle_state".to_string()),
                message: DashboardValue::available("只有运行中的节点可以暂停".to_string()),
            })),
        )),
        "resume" if previous_state != LifecycleStatus::Paused => Ok((
            StatusCode::CONFLICT,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available("invalid_lifecycle_state".to_string()),
                message: DashboardValue::available("只有已暂停的节点可以恢复".to_string()),
            })),
        )),
        "reconnect_data" | "reconnect_execution"
            if !matches!(
                previous_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            ) =>
        {
            Ok((
                StatusCode::CONFLICT,
                Json(action_response(ControlActionResponseParts {
                    action,
                    node_id,
                    status: ControlActionStatus::Rejected,
                    previous_state,
                    current_state: previous_state,
                    started_at,
                    error_code: DashboardValue::available("invalid_lifecycle_state".to_string()),
                    message: DashboardValue::available(
                        "只有运行中或已暂停的节点可以执行重连控制".to_string(),
                    ),
                })),
            ))
        }
        "start" => Ok(run_start_action(state, node_id, previous_state, started_at)),
        "stop" => Ok(run_stop_action(state, node_id, previous_state, started_at)),
        "pause" => Ok(run_pause_action(state, node_id, previous_state, started_at)),
        "resume" => Ok(run_resume_action(
            state,
            node_id,
            previous_state,
            started_at,
        )),
        "reconnect_data" => Ok(run_reconnect_data_action(
            state,
            node_id,
            previous_state,
            started_at,
        )),
        "reconnect_execution" => Ok(run_reconnect_execution_action(
            state,
            node_id,
            previous_state,
            started_at,
        )),
        _ => Ok((
            StatusCode::NOT_IMPLEMENTED,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available("unsupported_control_action".to_string()),
                message: DashboardValue::available("当前本地控制台暂不支持该控制动作".to_string()),
            })),
        )),
    }
}

fn run_start_action(
    state: &DashboardServerState,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
) -> (StatusCode, Json<ControlActionResponse>) {
    let store = SupervisorRegistryStore::new(state.registry_path.clone());
    let result = store.start_node_process(&StartNodeRequest {
        node_id: node_id.to_string(),
        ntpro_node_bin: state.ntpro_node_bin.clone(),
        startup_timeout: Duration::from_millis(DASHBOARD_ACTION_TIMEOUT_MS),
        node_max_runtime: Duration::from_millis(3_600_000),
        node_heartbeat_interval: Duration::from_millis(1_000),
        node_parent_pid: Some(std::process::id()),
        node_shutdown_timeout: Duration::from_millis(5_000),
    });
    match result {
        Ok(record) => (
            StatusCode::OK,
            Json(action_response(ControlActionResponseParts {
                action: "start",
                node_id,
                status: ControlActionStatus::Succeeded,
                previous_state,
                current_state: record.last_known_status.lifecycle_state,
                started_at,
                error_code: DashboardValue::unknown(),
                message: DashboardValue::available("已通过本地监督器完成启动".to_string()),
            })),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(action_response(ControlActionResponseParts {
                action: "start",
                node_id,
                status: ControlActionStatus::Failed,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available(control_error_code(&error)),
                message: DashboardValue::available("启动失败，详细信息已脱敏".to_string()),
            })),
        ),
    }
}

fn run_stop_action(
    state: &DashboardServerState,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
) -> (StatusCode, Json<ControlActionResponse>) {
    let store = SupervisorRegistryStore::new(state.registry_path.clone());
    let result = store.stop_node_process(&StopNodeRequest {
        node_id: node_id.to_string(),
        stop_timeout: Duration::from_millis(DASHBOARD_ACTION_TIMEOUT_MS),
    });
    match result {
        Ok(record) => (
            StatusCode::OK,
            Json(action_response(ControlActionResponseParts {
                action: "stop",
                node_id,
                status: ControlActionStatus::Succeeded,
                previous_state,
                current_state: record.last_known_status.lifecycle_state,
                started_at,
                error_code: DashboardValue::unknown(),
                message: DashboardValue::available("已通过本地监督器完成停止".to_string()),
            })),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(action_response(ControlActionResponseParts {
                action: "stop",
                node_id,
                status: ControlActionStatus::Failed,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available(control_error_code(&error)),
                message: DashboardValue::available("停止失败，详细信息已脱敏".to_string()),
            })),
        ),
    }
}

fn run_pause_action(
    state: &DashboardServerState,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
) -> (StatusCode, Json<ControlActionResponse>) {
    let store = SupervisorRegistryStore::new(state.registry_path.clone());
    let result = store.pause_node(node_id);
    match result {
        Ok(record) => (
            StatusCode::OK,
            Json(action_response(ControlActionResponseParts {
                action: "pause",
                node_id,
                status: ControlActionStatus::Succeeded,
                previous_state,
                current_state: record.last_known_status.lifecycle_state,
                started_at,
                error_code: DashboardValue::unknown(),
                message: DashboardValue::available("已通过本地监督器完成暂停".to_string()),
            })),
        ),
        Err(error) => failed_control_response("pause", node_id, previous_state, started_at, &error),
    }
}

fn run_resume_action(
    state: &DashboardServerState,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
) -> (StatusCode, Json<ControlActionResponse>) {
    let store = SupervisorRegistryStore::new(state.registry_path.clone());
    let result = store.resume_node(node_id);
    match result {
        Ok(record) => (
            StatusCode::OK,
            Json(action_response(ControlActionResponseParts {
                action: "resume",
                node_id,
                status: ControlActionStatus::Succeeded,
                previous_state,
                current_state: record.last_known_status.lifecycle_state,
                started_at,
                error_code: DashboardValue::unknown(),
                message: DashboardValue::available("已通过本地监督器完成恢复".to_string()),
            })),
        ),
        Err(error) => {
            failed_control_response("resume", node_id, previous_state, started_at, &error)
        }
    }
}

fn run_reconnect_data_action(
    state: &DashboardServerState,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
) -> (StatusCode, Json<ControlActionResponse>) {
    let store = SupervisorRegistryStore::new(state.registry_path.clone());
    let result = store.reconnect_data_source(node_id);
    match result {
        Ok(record) => (
            StatusCode::OK,
            Json(action_response(ControlActionResponseParts {
                action: "reconnect_data",
                node_id,
                status: ControlActionStatus::NotSupported,
                previous_state,
                current_state: record.last_known_status.lifecycle_state,
                started_at,
                error_code: DashboardValue::available(
                    "sandbox_reconnect_not_supported".to_string(),
                ),
                message: DashboardValue::available(
                    DASHBOARD_DATA_RECONNECT_UNSUPPORTED_MESSAGE.to_string(),
                ),
            })),
        ),
        Err(error) => failed_control_response(
            "reconnect_data",
            node_id,
            previous_state,
            started_at,
            &error,
        ),
    }
}

fn run_reconnect_execution_action(
    state: &DashboardServerState,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
) -> (StatusCode, Json<ControlActionResponse>) {
    let store = SupervisorRegistryStore::new(state.registry_path.clone());
    let result = store.reconnect_execution_gateway(node_id);
    match result {
        Ok(record) => (
            StatusCode::OK,
            Json(action_response(ControlActionResponseParts {
                action: "reconnect_execution",
                node_id,
                status: ControlActionStatus::NotSupported,
                previous_state,
                current_state: record.last_known_status.lifecycle_state,
                started_at,
                error_code: DashboardValue::available(
                    "sandbox_reconnect_not_supported".to_string(),
                ),
                message: DashboardValue::available(
                    DASHBOARD_EXECUTION_RECONNECT_UNSUPPORTED_MESSAGE.to_string(),
                ),
            })),
        ),
        Err(error) => failed_control_response(
            "reconnect_execution",
            node_id,
            previous_state,
            started_at,
            &error,
        ),
    }
}

fn failed_control_response(
    action: &'static str,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
    error: &anyhow::Error,
) -> (StatusCode, Json<ControlActionResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(action_response(ControlActionResponseParts {
            action,
            node_id,
            status: ControlActionStatus::Failed,
            previous_state,
            current_state: previous_state,
            started_at,
            error_code: DashboardValue::available(control_error_code(error)),
            message: DashboardValue::available(format!(
                "{}失败，详细信息已脱敏",
                control_action_display_name(action)
            )),
        })),
    )
}

fn control_action_display_name(action: &str) -> &'static str {
    match action {
        "start" => "启动",
        "stop" => "停止",
        "pause" => "暂停",
        "resume" => "恢复",
        "reconnect_data" => "记录数据源重连不支持",
        "reconnect_execution" => "记录执行网关重连不支持",
        _ => "控制动作",
    }
}

#[derive(Debug)]
struct ControlActionResponseParts<'a> {
    action: &'a str,
    node_id: &'a str,
    status: ControlActionStatus,
    previous_state: LifecycleStatus,
    current_state: LifecycleStatus,
    started_at: String,
    error_code: DashboardValue<String>,
    message: DashboardValue<String>,
}

fn action_response(parts: ControlActionResponseParts<'_>) -> ControlActionResponse {
    let finished_at = generated_at_now();
    ControlActionResponse {
        action_id: format!("{}:{}:{}", parts.action, parts.node_id, parts.started_at),
        action: format!("{}:{}", parts.action, parts.node_id),
        status: parts.status,
        previous_state: parts.previous_state,
        current_state: parts.current_state,
        started_at: DashboardValue::available(parts.started_at),
        finished_at: DashboardValue::available(finished_at),
        error_code: parts.error_code,
        message: parts.message,
        observability_ref: DashboardValue::available(format!("registry:{}", parts.node_id)),
    }
}

fn control_error_code(error: &anyhow::Error) -> String {
    let message = error.to_string();
    if message.contains("already running") || message.contains("not running") {
        "invalid_lifecycle_state".to_string()
    } else if message.contains("ntpro-node binary") {
        "ntpro_node_binary_unavailable".to_string()
    } else if message.contains("timed out") {
        "lifecycle_timeout".to_string()
    } else if message.contains("not registered") {
        "node_not_found".to_string()
    } else {
        "supervisor_action_failed".to_string()
    }
}

fn load_dashboard_snapshot(state: &DashboardServerState) -> SnapshotLoadResult {
    snapshot_from_supervisor_artifacts_with_workflow_root(
        &state.registry_path,
        state.workflow_root.as_deref(),
        generated_at_now(),
    )
    .map_err(|error| {
        let message = error.to_string();
        server_error_response("snapshot_load_failed", &message)
    })
}

fn not_found_response(error_code: &str, node_id: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error_code": error_code,
            "node_id": node_id,
            "message": "本地监督器注册表中没有找到该节点"
        })),
    )
}

fn server_error_response(error_code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error_code": error_code,
            "message": message
        })),
    )
}

fn generated_at_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    format!("unix_seconds:{seconds}")
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardAvailability {
    Available,
    NotConfigured,
    NotSupported,
    Stale,
    Redacted,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardValue<T> {
    pub availability: DashboardAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,
}

impl<T> DashboardValue<T> {
    #[must_use]
    pub const fn available(value: T) -> Self {
        Self {
            availability: DashboardAvailability::Available,
            value: Some(value),
        }
    }

    #[must_use]
    pub const fn not_configured() -> Self {
        Self {
            availability: DashboardAvailability::NotConfigured,
            value: None,
        }
    }

    #[must_use]
    pub const fn not_supported() -> Self {
        Self {
            availability: DashboardAvailability::NotSupported,
            value: None,
        }
    }

    #[must_use]
    pub const fn stale() -> Self {
        Self {
            availability: DashboardAvailability::Stale,
            value: None,
        }
    }

    #[must_use]
    pub const fn redacted() -> Self {
        Self {
            availability: DashboardAvailability::Redacted,
            value: None,
        }
    }

    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            availability: DashboardAvailability::Unknown,
            value: None,
        }
    }
}

impl<T> Default for DashboardValue<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub schema_version: String,
    pub generated_at: DashboardValue<String>,
    pub overview: DashboardOverview,
    pub nodes: Vec<DashboardNodeSummary>,
    pub data_sources: Vec<DataSourceStatus>,
    pub execution_gateways: Vec<ExecutionGatewayStatus>,
    pub risk: RiskStatus,
    pub sandbox_business: SandboxBusinessStatus,
    pub workflow_artifacts: Vec<WorkflowArtifactStatus>,
    pub read_model_runtime: Vec<TraderTerminalReadModelStatus>,
    pub strategy_runtime: Vec<StrategyRuntimeStatus>,
    pub production_shadow: Vec<ProductionShadowStatus>,
    pub preflight_readiness: Vec<PreflightReadinessStatus>,
    pub live_alpha_dry_run: Vec<LiveAlphaDryRunStatus>,
    pub production_mutation_evidence: Vec<ProductionMutationEvidenceStatus>,
    pub production_reconciliation_orphan: Vec<ProductionReconciliationOrphanStatus>,
    pub production_cancel_recovery: Vec<ProductionCancelRecoveryStatus>,
    pub production_actual_cancel_audit: Vec<ProductionActualCancelAuditStatus>,
    pub production_order_lifecycle_audit: Vec<ProductionOrderLifecycleAuditStatus>,
    pub runtime_modules: Vec<RuntimeModuleStatus>,
    pub logs: Vec<LogStatus>,
    pub metrics: Vec<MetricStatus>,
    pub alerts: AlertSummary,
    pub controls: Vec<ControlStatus>,
    pub gaps: Vec<DashboardGap>,
}

impl DashboardSnapshot {
    #[must_use]
    pub fn empty(generated_at: impl Into<String>) -> Self {
        Self {
            schema_version: DASHBOARD_SNAPSHOT_SCHEMA_VERSION.to_string(),
            generated_at: DashboardValue::available(generated_at.into()),
            overview: DashboardOverview::default(),
            nodes: Vec::new(),
            data_sources: Vec::new(),
            execution_gateways: Vec::new(),
            risk: RiskStatus::unknown(),
            sandbox_business: sandbox_business_status_from_v04_evidence(),
            workflow_artifacts: Vec::new(),
            read_model_runtime: Vec::new(),
            strategy_runtime: Vec::new(),
            production_shadow: Vec::new(),
            preflight_readiness: Vec::new(),
            live_alpha_dry_run: Vec::new(),
            production_mutation_evidence: Vec::new(),
            production_reconciliation_orphan: Vec::new(),
            production_cancel_recovery: Vec::new(),
            production_actual_cancel_audit: Vec::new(),
            production_order_lifecycle_audit: Vec::new(),
            runtime_modules: Vec::new(),
            logs: Vec::new(),
            metrics: Vec::new(),
            alerts: AlertSummary::default(),
            controls: Vec::new(),
            gaps: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_nodes(generated_at: impl Into<String>, nodes: Vec<DashboardNodeSummary>) -> Self {
        let overview = DashboardOverview::from_nodes(&nodes);
        Self {
            overview,
            nodes,
            ..Self::empty(generated_at)
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub node_count: u64,
    pub running_nodes: u64,
    pub stopped_nodes: u64,
    pub error_nodes: u64,
    pub unknown_nodes: u64,
    pub health: HealthStatus,
    pub sandbox_only: bool,
    pub external_venue_connection: bool,
    pub production_venue_connection: bool,
    pub testnet_public_network_connection: bool,
    pub external_network_attempted: bool,
    pub real_orders_submitted: bool,
    pub latest_transition_at: DashboardValue<String>,
    pub latest_error: Option<String>,
}

impl DashboardOverview {
    #[must_use]
    pub fn from_nodes(nodes: &[DashboardNodeSummary]) -> Self {
        let mut overview = Self {
            node_count: nodes.len() as u64,
            sandbox_only: true,
            health: HealthStatus::Unknown,
            ..Self::default()
        };

        for node in nodes {
            match node.lifecycle_state {
                LifecycleStatus::Running => overview.running_nodes += 1,
                LifecycleStatus::Stopped => overview.stopped_nodes += 1,
                LifecycleStatus::Error => overview.error_nodes += 1,
                LifecycleStatus::Unknown => overview.unknown_nodes += 1,
                LifecycleStatus::Starting
                | LifecycleStatus::Pausing
                | LifecycleStatus::Paused
                | LifecycleStatus::Resuming
                | LifecycleStatus::Stopping => {}
            }
            overview.external_venue_connection |= node.external_venue_connection;
            overview.real_orders_submitted |= node.real_orders_submitted;
            if overview.latest_error.is_none() {
                overview.latest_error.clone_from(&node.last_error);
            }
        }

        overview.health = derive_overview_health(&overview);
        overview
    }

    fn apply_workflow_artifacts(&mut self, workflow_artifacts: &[WorkflowArtifactStatus]) {
        if workflow_artifacts.is_empty() {
            self.health = derive_overview_health(self);
            return;
        }
        let all_workflows_sandbox_only = workflow_artifacts
            .iter()
            .all(|workflow| workflow.sandbox_only);
        self.sandbox_only = if self.node_count == 0 {
            all_workflows_sandbox_only
        } else {
            self.sandbox_only && all_workflows_sandbox_only
        };
        for workflow in workflow_artifacts {
            self.external_venue_connection |= workflow.external_venue_connection;
            self.production_venue_connection |= workflow.production_venue_connection;
            self.testnet_public_network_connection |= workflow.testnet_public_network_connection;
            self.external_network_attempted |= workflow.external_network_attempted;
            self.real_orders_submitted |= workflow.real_orders_submitted;
        }
        self.health = derive_overview_health(self);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardNodeSummary {
    pub node_id: String,
    pub lifecycle_state: LifecycleStatus,
    pub process_mode: ProcessMode,
    pub process_state: SupervisorProcessState,
    pub pid: SnapshotValue<u32>,
    pub health: HealthStatus,
    pub config_path: SnapshotValue<String>,
    pub artifact_root: SnapshotValue<String>,
    pub generated_at: SnapshotValue<String>,
    pub started_at: SnapshotValue<String>,
    pub stopped_at: SnapshotValue<String>,
    pub last_transition_at: SnapshotValue<String>,
    pub last_error: Option<String>,
    pub external_venue_connection: bool,
    pub real_orders_submitted: bool,
    pub gaps: Vec<DashboardGap>,
}

impl DashboardNodeSummary {
    #[must_use]
    pub fn from_status(status: &NodeStatus) -> Self {
        Self {
            node_id: status.node_id.clone(),
            lifecycle_state: status.lifecycle_state,
            process_mode: status.process_mode,
            process_state: SupervisorProcessState::Unknown,
            pid: SnapshotValue::unknown(),
            health: derive_node_health(status),
            config_path: status.config_path.clone(),
            artifact_root: status.artifact_root.clone(),
            generated_at: status.generated_at.clone(),
            started_at: status.started_at.clone(),
            stopped_at: status.stopped_at.clone(),
            last_transition_at: status.last_transition_at.clone(),
            last_error: status.last_error.clone(),
            external_venue_connection: status.external_venue_connection,
            real_orders_submitted: status.real_orders_submitted,
            gaps: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSourceStatus {
    pub source_id: String,
    pub source_kind: DashboardValue<String>,
    pub provider: DashboardValue<String>,
    pub connection: ConnectionStatus,
    pub freshness: DashboardValue<String>,
    pub lag_ms: DashboardValue<u64>,
    pub health: HealthStatus,
    pub last_error: DashboardValue<String>,
}

impl DataSourceStatus {
    #[must_use]
    pub fn unknown(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            source_kind: DashboardValue::unknown(),
            provider: DashboardValue::unknown(),
            connection: ConnectionStatus::Unknown,
            freshness: DashboardValue::unknown(),
            lag_ms: DashboardValue::unknown(),
            health: HealthStatus::Unknown,
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderCountSummary {
    pub open: DashboardValue<u64>,
    pub inflight: DashboardValue<u64>,
    pub closed: DashboardValue<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionGatewayStatus {
    pub gateway_id: String,
    pub venue: DashboardValue<String>,
    pub connection: ConnectionStatus,
    pub started: DashboardValue<bool>,
    pub account_ref: DashboardValue<String>,
    pub order_counts: OrderCountSummary,
    pub last_report_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl ExecutionGatewayStatus {
    #[must_use]
    pub fn unknown(gateway_id: impl Into<String>) -> Self {
        Self {
            gateway_id: gateway_id.into(),
            venue: DashboardValue::unknown(),
            connection: ConnectionStatus::Unknown,
            started: DashboardValue::unknown(),
            account_ref: DashboardValue::redacted(),
            order_counts: OrderCountSummary::default(),
            last_report_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectionSummary {
    pub reason: DashboardValue<String>,
    pub last_rejected_at: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskStatus {
    pub availability: DashboardAvailability,
    pub trading_state: RiskTradingState,
    pub health: HealthStatus,
    pub command_count: DashboardValue<u64>,
    pub event_count: DashboardValue<u64>,
    pub rejections_total: DashboardValue<u64>,
    pub last_rejection: DashboardValue<RejectionSummary>,
    pub last_error: DashboardValue<String>,
}

impl RiskStatus {
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            availability: DashboardAvailability::Unknown,
            trading_state: RiskTradingState::Unknown,
            health: HealthStatus::Unknown,
            command_count: DashboardValue::unknown(),
            event_count: DashboardValue::unknown(),
            rejections_total: DashboardValue::unknown(),
            last_rejection: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxBusinessStatus {
    pub availability: DashboardAvailability,
    pub exchange: SandboxExchangePanel,
    pub strategies: Vec<SandboxStrategyPanel>,
    pub order: SandboxOrderPanel,
    pub risk: SandboxRiskPanel,
    pub diagnostic: DashboardValue<String>,
}

impl SandboxBusinessStatus {
    #[must_use]
    pub fn unknown(diagnostic: impl Into<String>) -> Self {
        Self {
            availability: DashboardAvailability::Unknown,
            exchange: SandboxExchangePanel::unknown(),
            strategies: Vec::new(),
            order: SandboxOrderPanel::unknown(),
            risk: SandboxRiskPanel::unknown(),
            diagnostic: DashboardValue::available(diagnostic.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxExchangePanel {
    pub venue: DashboardValue<String>,
    pub instrument_id: DashboardValue<String>,
    pub bar_type: DashboardValue<String>,
    pub fixture_id: DashboardValue<String>,
    pub fixture_checksum: DashboardValue<String>,
    pub bars_processed: DashboardValue<u64>,
    pub connection_mode: DashboardValue<String>,
    pub external_venue_connection: bool,
}

impl SandboxExchangePanel {
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            venue: DashboardValue::unknown(),
            instrument_id: DashboardValue::unknown(),
            bar_type: DashboardValue::unknown(),
            fixture_id: DashboardValue::unknown(),
            fixture_checksum: DashboardValue::unknown(),
            bars_processed: DashboardValue::unknown(),
            connection_mode: DashboardValue::unknown(),
            external_venue_connection: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxStrategyPanel {
    pub strategy_id: String,
    pub strategy_name: DashboardValue<String>,
    pub smoke_id: DashboardValue<String>,
    pub runtime_status: DashboardValue<String>,
    pub signal_mode: DashboardValue<String>,
    pub bars_processed: DashboardValue<u64>,
    pub signals_emitted: DashboardValue<u64>,
    pub mock_orders_requested: DashboardValue<u64>,
    pub final_signal: DashboardValue<String>,
    pub indicator_value: DashboardValue<String>,
    pub checksum: DashboardValue<String>,
    pub real_orders_submitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxOrderPanel {
    pub lifecycle_id: DashboardValue<String>,
    pub source_path: DashboardValue<String>,
    pub event_count: DashboardValue<u64>,
    pub submitted_count: DashboardValue<u64>,
    pub accepted_count: DashboardValue<u64>,
    pub filled_count: DashboardValue<u64>,
    pub canceled_count: DashboardValue<u64>,
    pub rejected_count: DashboardValue<u64>,
    pub event_types: Vec<String>,
    pub mock_orders_requested: DashboardValue<u64>,
    pub real_orders_submitted: bool,
    pub evidence_source: DashboardValue<String>,
}

impl SandboxOrderPanel {
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            lifecycle_id: DashboardValue::unknown(),
            source_path: DashboardValue::unknown(),
            event_count: DashboardValue::unknown(),
            submitted_count: DashboardValue::unknown(),
            accepted_count: DashboardValue::unknown(),
            filled_count: DashboardValue::unknown(),
            canceled_count: DashboardValue::unknown(),
            rejected_count: DashboardValue::unknown(),
            event_types: Vec::new(),
            mock_orders_requested: DashboardValue::unknown(),
            real_orders_submitted: false,
            evidence_source: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxRiskPanel {
    pub smoke_id: DashboardValue<String>,
    pub lifecycle_id: DashboardValue<String>,
    pub client_order_id: DashboardValue<String>,
    pub fixture_reason: DashboardValue<String>,
    pub risk_reason: DashboardValue<String>,
    pub order_status: DashboardValue<String>,
    pub forwarded_to_execution: bool,
    pub rejection_count: DashboardValue<u64>,
    pub real_orders_submitted: bool,
    pub health: HealthStatus,
}

impl SandboxRiskPanel {
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            smoke_id: DashboardValue::unknown(),
            lifecycle_id: DashboardValue::unknown(),
            client_order_id: DashboardValue::unknown(),
            fixture_reason: DashboardValue::unknown(),
            risk_reason: DashboardValue::unknown(),
            order_status: DashboardValue::unknown(),
            forwarded_to_execution: false,
            rejection_count: DashboardValue::unknown(),
            real_orders_submitted: false,
            health: HealthStatus::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowArtifactStatus {
    pub run_id: String,
    pub workflow: String,
    pub workflow_id: DashboardValue<String>,
    pub schema_version: String,
    pub runtime_status: String,
    pub health: HealthStatus,
    pub manifest_path: String,
    pub artifact_count: u64,
    pub market_fixture_id: DashboardValue<String>,
    pub order_lifecycle_id: DashboardValue<String>,
    pub risk_smoke_id: DashboardValue<String>,
    pub sandbox_only: bool,
    pub fixture_replay: bool,
    pub mock_execution: bool,
    pub external_venue_connection: bool,
    pub production_venue_connection: bool,
    pub testnet_public_network_connection: bool,
    pub external_network_attempted: bool,
    pub real_funds: bool,
    pub production_trading: bool,
    pub real_orders_submitted: bool,
    pub testnet_connection: bool,
    pub network_permission_requested: bool,
    pub network_attempted: bool,
    pub credential_policy: DashboardValue<String>,
    pub connectivity_mode: DashboardValue<String>,
    pub order_submission_mode: DashboardValue<String>,
    pub reconciliation_mode: DashboardValue<String>,
    pub probe_status: DashboardValue<String>,
    pub probe_latency_ms: DashboardValue<u64>,
    pub probe_endpoint_class: DashboardValue<String>,
    pub probe_error_code: DashboardValue<String>,
    pub values_recorded: DashboardValue<bool>,
    pub secrets_redacted: DashboardValue<bool>,
    pub authenticated_probe_status: DashboardValue<String>,
    pub authenticated_endpoint_kind: DashboardValue<String>,
    pub authenticated_request_method: DashboardValue<String>,
    pub authenticated_response_shape: DashboardValue<String>,
    pub authenticated_response_shape_validated: DashboardValue<bool>,
    pub authenticated_api_key_present: DashboardValue<bool>,
    pub authenticated_api_secret_present: DashboardValue<bool>,
    pub authenticated_secrets_redacted: DashboardValue<bool>,
    pub authenticated_account_mutation: DashboardValue<bool>,
    pub authenticated_real_orders_submitted: DashboardValue<bool>,
    pub authenticated_production_venue_connection: DashboardValue<bool>,
    pub order_proof_risk_preflight_status: DashboardValue<String>,
    pub order_proof_order_test_status: DashboardValue<String>,
    pub order_proof_submit_ack_status: DashboardValue<String>,
    pub order_proof_cancel_ack_status: DashboardValue<String>,
    pub order_proof_terminal_status: DashboardValue<String>,
    pub order_proof_reconciliation_status: DashboardValue<String>,
    pub order_proof_manual_submit_cancel_observed: DashboardValue<bool>,
    pub order_proof_testnet_orders_submitted: DashboardValue<u64>,
    pub order_proof_testnet_orders_canceled: DashboardValue<u64>,
    pub order_proof_production_orders_submitted: DashboardValue<u64>,
    pub order_proof_production_orders_canceled: DashboardValue<u64>,
    pub order_proof_dashboard_order_controls: DashboardValue<bool>,
    pub websocket_probe_status: DashboardValue<String>,
    pub websocket_error_code: DashboardValue<String>,
    pub websocket_attempted: bool,
    pub websocket_subscription_attempted: bool,
    pub websocket_message_count: DashboardValue<u64>,
    pub diagnostic: DashboardValue<String>,
}

impl WorkflowArtifactStatus {
    #[must_use]
    pub fn unknown(manifest_path: impl Into<String>, diagnostic: impl Into<String>) -> Self {
        Self {
            run_id: "unknown".to_string(),
            workflow: "unknown".to_string(),
            workflow_id: DashboardValue::unknown(),
            schema_version: "unknown".to_string(),
            runtime_status: "unknown".to_string(),
            health: HealthStatus::Unknown,
            manifest_path: manifest_path.into(),
            artifact_count: 0,
            market_fixture_id: DashboardValue::unknown(),
            order_lifecycle_id: DashboardValue::unknown(),
            risk_smoke_id: DashboardValue::unknown(),
            sandbox_only: false,
            fixture_replay: false,
            mock_execution: false,
            external_venue_connection: false,
            production_venue_connection: false,
            testnet_public_network_connection: false,
            external_network_attempted: false,
            real_funds: false,
            production_trading: false,
            real_orders_submitted: false,
            testnet_connection: false,
            network_permission_requested: false,
            network_attempted: false,
            credential_policy: DashboardValue::unknown(),
            connectivity_mode: DashboardValue::unknown(),
            order_submission_mode: DashboardValue::unknown(),
            reconciliation_mode: DashboardValue::unknown(),
            probe_status: DashboardValue::unknown(),
            probe_latency_ms: DashboardValue::unknown(),
            probe_endpoint_class: DashboardValue::unknown(),
            probe_error_code: DashboardValue::unknown(),
            values_recorded: DashboardValue::unknown(),
            secrets_redacted: DashboardValue::unknown(),
            authenticated_probe_status: DashboardValue::unknown(),
            authenticated_endpoint_kind: DashboardValue::unknown(),
            authenticated_request_method: DashboardValue::unknown(),
            authenticated_response_shape: DashboardValue::unknown(),
            authenticated_response_shape_validated: DashboardValue::unknown(),
            authenticated_api_key_present: DashboardValue::unknown(),
            authenticated_api_secret_present: DashboardValue::unknown(),
            authenticated_secrets_redacted: DashboardValue::unknown(),
            authenticated_account_mutation: DashboardValue::unknown(),
            authenticated_real_orders_submitted: DashboardValue::unknown(),
            authenticated_production_venue_connection: DashboardValue::unknown(),
            order_proof_risk_preflight_status: DashboardValue::unknown(),
            order_proof_order_test_status: DashboardValue::unknown(),
            order_proof_submit_ack_status: DashboardValue::unknown(),
            order_proof_cancel_ack_status: DashboardValue::unknown(),
            order_proof_terminal_status: DashboardValue::unknown(),
            order_proof_reconciliation_status: DashboardValue::unknown(),
            order_proof_manual_submit_cancel_observed: DashboardValue::unknown(),
            order_proof_testnet_orders_submitted: DashboardValue::unknown(),
            order_proof_testnet_orders_canceled: DashboardValue::unknown(),
            order_proof_production_orders_submitted: DashboardValue::unknown(),
            order_proof_production_orders_canceled: DashboardValue::unknown(),
            order_proof_dashboard_order_controls: DashboardValue::unknown(),
            websocket_probe_status: DashboardValue::unknown(),
            websocket_error_code: DashboardValue::unknown(),
            websocket_attempted: false,
            websocket_subscription_attempted: false,
            websocket_message_count: DashboardValue::unknown(),
            diagnostic: DashboardValue::available(diagnostic.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyRuntimeStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub diagnostic: DashboardValue<String>,
    pub session_id: DashboardValue<String>,
    pub session_state: DashboardValue<String>,
    pub strategy_id: DashboardValue<String>,
    pub symbol: DashboardValue<String>,
    pub market_stream_status: DashboardValue<String>,
    pub signal_count: DashboardValue<u64>,
    pub latest_signal: DashboardValue<String>,
    pub latest_order_intent: DashboardValue<String>,
    pub latest_risk_decision: DashboardValue<String>,
    pub rejection_reason: DashboardValue<String>,
    pub order_submission_mode: DashboardValue<String>,
    pub actual_submission_count: DashboardValue<u64>,
    pub session_status_path: DashboardValue<String>,
    pub signal_artifact_path: DashboardValue<String>,
    pub order_intent_artifact_path: DashboardValue<String>,
    pub risk_decision_artifact_path: DashboardValue<String>,
    pub summary_artifact_path: DashboardValue<String>,
    pub manifest_path: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionShadowStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub diagnostic: DashboardValue<String>,
    pub artifact_version: DashboardValue<String>,
    pub public_read_status: DashboardValue<String>,
    pub public_read_endpoint_class: DashboardValue<String>,
    pub response_shape_status: DashboardValue<String>,
    pub response_shape_validated: DashboardValue<bool>,
    pub manifest_status: DashboardValue<String>,
    pub manifest_artifact_count: DashboardValue<u64>,
    pub account_snapshot_status: DashboardValue<String>,
    pub account_snapshot_endpoint_class: DashboardValue<String>,
    pub shadow_intent_status: DashboardValue<String>,
    pub shadow_intents_created: DashboardValue<u64>,
    pub portfolio_snapshot_status: DashboardValue<String>,
    pub portfolio_exposure_status: DashboardValue<String>,
    pub portfolio_pnl_status: DashboardValue<String>,
    pub lifecycle_status: DashboardValue<String>,
    pub lifecycle_events_created: DashboardValue<u64>,
    pub shadow_strategy_session_status: DashboardValue<String>,
    pub shadow_strategy_session_heartbeats: DashboardValue<u64>,
    pub reconciliation_status: DashboardValue<String>,
    pub reconciliation_classification: DashboardValue<String>,
    pub reconciliation_recommended_action: DashboardValue<String>,
    pub reconciliation_events_created: DashboardValue<u64>,
    pub kill_switch_status: DashboardValue<String>,
    pub kill_switch_active: DashboardValue<bool>,
    pub kill_switch_dry_run: DashboardValue<bool>,
    pub kill_switch_manual_approval_recorded: DashboardValue<bool>,
    pub kill_switch_approval_state: DashboardValue<String>,
    pub kill_switch_production_order_submission_allowed: DashboardValue<bool>,
    pub kill_switch_production_order_mutation_allowed: DashboardValue<bool>,
    pub kill_switch_production_order_state_reads_allowed: DashboardValue<bool>,
    pub kill_switch_listen_key_lifecycle_allowed: DashboardValue<bool>,
    pub risk_halted: DashboardValue<bool>,
    pub manual_review_required: DashboardValue<bool>,
    pub new_orders_blocked: DashboardValue<bool>,
    pub actual_submission_count: DashboardValue<u64>,
    pub production_order_submissions_attempted: DashboardValue<u64>,
    pub production_orders_submitted: DashboardValue<u64>,
    pub production_order_mutations_attempted: DashboardValue<u64>,
    pub production_order_state_reads_attempted: DashboardValue<u64>,
    pub listen_key_lifecycle_attempted: DashboardValue<u64>,
    pub automatic_correction_orders_submitted: DashboardValue<u64>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub real_orders_submitted: DashboardValue<bool>,
    pub order_state_values_are_exchange_truth: DashboardValue<bool>,
    pub shadow_values_are_exchange_truth: DashboardValue<bool>,
    pub portfolio_values_are_exchange_truth: DashboardValue<bool>,
    pub values_are_exchange_truth: DashboardValue<bool>,
    pub manifest_path: DashboardValue<String>,
    pub public_read_probe_path: DashboardValue<String>,
    pub account_snapshot_path: DashboardValue<String>,
    pub response_shape_path: DashboardValue<String>,
    pub shadow_intent_path: DashboardValue<String>,
    pub portfolio_snapshot_path: DashboardValue<String>,
    pub shadow_strategy_session_path: DashboardValue<String>,
    pub lifecycle_path: DashboardValue<String>,
    pub reconciliation_path: DashboardValue<String>,
    pub kill_switch_approval_artifact_path: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReadinessStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub readiness_status: DashboardValue<String>,
    pub owner_proof_pack_status: DashboardValue<String>,
    pub kill_switch_artifact_status: DashboardValue<String>,
    pub bounded_shadow_preflight_status: DashboardValue<String>,
    pub decimal_boundary_status: DashboardValue<String>,
    pub no_production_mutation_gate_status: DashboardValue<String>,
    pub production_order_submission_allowed: DashboardValue<bool>,
    pub production_order_mutation_allowed: DashboardValue<bool>,
    pub production_order_state_reads_allowed: DashboardValue<bool>,
    pub listen_key_lifecycle_allowed: DashboardValue<bool>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub real_orders_submitted: DashboardValue<bool>,
    pub order_state_values_are_exchange_truth: DashboardValue<bool>,
    pub shadow_values_are_exchange_truth: DashboardValue<bool>,
    pub portfolio_values_are_exchange_truth: DashboardValue<bool>,
    pub values_are_exchange_truth: DashboardValue<bool>,
    pub diagnostic: DashboardValue<String>,
    pub evidence_source: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveAlphaDryRunStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub readiness_status: DashboardValue<String>,
    pub diagnostic: DashboardValue<String>,
    pub gate_status: DashboardValue<String>,
    pub gate_ready: DashboardValue<bool>,
    pub missing_gate_flags: DashboardValue<String>,
    pub dry_run_order_intent_recorded: DashboardValue<bool>,
    pub order_submission_mode: DashboardValue<String>,
    pub risk_preflight_status: DashboardValue<String>,
    pub risk_decision: DashboardValue<String>,
    pub execution_decision: DashboardValue<String>,
    pub risk_reasons: DashboardValue<String>,
    pub kill_switch_active: DashboardValue<bool>,
    pub manual_approval_status: DashboardValue<String>,
    pub manual_approval_state: DashboardValue<String>,
    pub manual_approval_valid: DashboardValue<bool>,
    pub manual_approval_issues: DashboardValue<String>,
    pub manual_approval_recorded: DashboardValue<bool>,
    pub manual_approval_one_time: DashboardValue<bool>,
    pub manual_approval_used: DashboardValue<bool>,
    pub manual_approval_expires_at_unix_ms: DashboardValue<u64>,
    pub request_preview_status: DashboardValue<String>,
    pub request_preview_allowed: DashboardValue<bool>,
    pub request_preview_built: DashboardValue<bool>,
    pub request_sent: DashboardValue<bool>,
    pub request_method: DashboardValue<String>,
    pub request_target: DashboardValue<String>,
    pub endpoint_class: DashboardValue<String>,
    pub endpoint_decision: DashboardValue<String>,
    pub query_shape_without_signature: DashboardValue<String>,
    pub signature_preflight: DashboardValue<String>,
    pub secrets_redacted: DashboardValue<bool>,
    pub signed_request_memory_only: DashboardValue<bool>,
    pub execution_dry_run_status: DashboardValue<String>,
    pub dry_run_execution_adapter_called: DashboardValue<bool>,
    pub dry_run_execution_adapter_wrote_artifact: DashboardValue<bool>,
    pub dry_run_adapter_artifact_only: DashboardValue<bool>,
    pub production_adapter_called: DashboardValue<bool>,
    pub production_adapter_instantiated: DashboardValue<bool>,
    pub strategy_intent_recorded: DashboardValue<bool>,
    pub strategy_intent_reaches_risk_preflight: DashboardValue<bool>,
    pub strategy_intent_reaches_dry_run_adapter: DashboardValue<bool>,
    pub strategy_intent_reaches_production_adapter: DashboardValue<bool>,
    pub kill_switch_runtime_gate_status: DashboardValue<String>,
    pub runtime_gate_decision: DashboardValue<String>,
    pub runtime_gate_open: DashboardValue<bool>,
    pub runtime_gate_reasons: DashboardValue<String>,
    pub order_state_readable: DashboardValue<bool>,
    pub order_state_read_status: DashboardValue<String>,
    pub order_state_endpoint: DashboardValue<String>,
    pub order_state_network_attempted: DashboardValue<bool>,
    pub order_state_read_attempted: DashboardValue<bool>,
    pub order_state_shape_validated: DashboardValue<bool>,
    pub order_state_age_ms: DashboardValue<u64>,
    pub max_order_state_age_ms: DashboardValue<u64>,
    pub open_order_count: DashboardValue<u64>,
    pub max_open_orders: DashboardValue<u64>,
    pub non_empty_order_state_observed: DashboardValue<bool>,
    pub order_lifecycle_readiness: DashboardValue<bool>,
    pub order_state_truth_source: DashboardValue<String>,
    pub reconciliation_status: DashboardValue<String>,
    pub production_order_submission_allowed: DashboardValue<bool>,
    pub production_order_mutation_allowed: DashboardValue<bool>,
    pub production_order_state_reads_allowed: DashboardValue<bool>,
    pub listen_key_lifecycle_allowed: DashboardValue<bool>,
    pub production_order_submissions_attempted: DashboardValue<u64>,
    pub production_orders_submitted: DashboardValue<u64>,
    pub production_order_mutations_attempted: DashboardValue<u64>,
    pub production_order_state_reads_attempted: DashboardValue<u64>,
    pub listen_key_lifecycle_attempted: DashboardValue<u64>,
    pub cancel_replace_amend_attempted: DashboardValue<bool>,
    pub order_endpoint_access_attempted: DashboardValue<bool>,
    pub execution_adapter_called: DashboardValue<bool>,
    pub matching_engine_submission: DashboardValue<bool>,
    pub actual_submission_count: DashboardValue<u64>,
    pub automatic_correction_orders_submitted: DashboardValue<u64>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub network_attempted: DashboardValue<bool>,
    pub real_orders_submitted: DashboardValue<bool>,
    pub real_funds: DashboardValue<bool>,
    pub production_trading_enabled: DashboardValue<bool>,
    pub order_state_values_are_exchange_truth: DashboardValue<bool>,
    pub shadow_values_are_exchange_truth: DashboardValue<bool>,
    pub portfolio_values_are_exchange_truth: DashboardValue<bool>,
    pub values_are_exchange_truth: DashboardValue<bool>,
    pub order_gate_path: DashboardValue<String>,
    pub risk_preflight_path: DashboardValue<String>,
    pub order_state_proof_path: DashboardValue<String>,
    pub manual_approval_lifecycle_path: DashboardValue<String>,
    pub request_preview_path: DashboardValue<String>,
    pub execution_dry_run_path: DashboardValue<String>,
    pub kill_switch_runtime_gate_path: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionMutationEvidenceStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub readiness_status: DashboardValue<String>,
    pub diagnostic: DashboardValue<String>,
    pub runtime_gate_status: DashboardValue<String>,
    pub runtime_gate_open: DashboardValue<bool>,
    pub signing_approval_status: DashboardValue<String>,
    pub approval_state: DashboardValue<String>,
    pub manual_approval_recorded: DashboardValue<bool>,
    pub approved_by: DashboardValue<String>,
    pub request_builder_status: DashboardValue<String>,
    pub request_builder_ready: DashboardValue<bool>,
    pub guarded_send_status: DashboardValue<String>,
    pub guarded_send_ready: DashboardValue<bool>,
    pub request_sent: DashboardValue<bool>,
    pub network_attempted: DashboardValue<bool>,
    pub kill_switch_checked_before_send: DashboardValue<bool>,
    pub kill_switch_checked_after_send: DashboardValue<bool>,
    pub response_redaction_status: DashboardValue<String>,
    pub response_redaction_ready: DashboardValue<bool>,
    pub order_state_readback_status: DashboardValue<String>,
    pub readback_contract_ready: DashboardValue<bool>,
    pub order_state_read_attempted: DashboardValue<bool>,
    pub response_shape_validated: DashboardValue<bool>,
    pub audit_trail_status: DashboardValue<String>,
    pub audit_trail_ready: DashboardValue<bool>,
    pub failure_semantics_status: DashboardValue<String>,
    pub failure_semantics_ready: DashboardValue<bool>,
    pub failure_mode: DashboardValue<String>,
    pub terminal_action: DashboardValue<String>,
    pub strategy_continuation_allowed: DashboardValue<bool>,
    pub symbol: DashboardValue<String>,
    pub side: DashboardValue<String>,
    pub order_type: DashboardValue<String>,
    pub time_in_force: DashboardValue<String>,
    pub quantity: DashboardValue<String>,
    pub price: DashboardValue<String>,
    pub order_id: DashboardValue<String>,
    pub production_order_submissions_attempted: DashboardValue<u64>,
    pub production_orders_submitted: DashboardValue<u64>,
    pub production_order_mutations_attempted: DashboardValue<u64>,
    pub production_order_state_reads_attempted: DashboardValue<u64>,
    pub listen_key_lifecycle_attempted: DashboardValue<u64>,
    pub retry_attempted: DashboardValue<bool>,
    pub cancel_attempted: DashboardValue<bool>,
    pub replace_attempted: DashboardValue<bool>,
    pub amend_attempted: DashboardValue<bool>,
    pub correction_attempted: DashboardValue<bool>,
    pub flatten_attempted: DashboardValue<bool>,
    pub remediation_attempted: DashboardValue<bool>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub real_orders_submitted: DashboardValue<bool>,
    pub production_trading_enabled: DashboardValue<bool>,
    pub runtime_gate_path: DashboardValue<String>,
    pub signing_approval_path: DashboardValue<String>,
    pub request_builder_path: DashboardValue<String>,
    pub guarded_send_path: DashboardValue<String>,
    pub response_redaction_path: DashboardValue<String>,
    pub order_state_readback_path: DashboardValue<String>,
    pub audit_trail_path: DashboardValue<String>,
    pub failure_semantics_path: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionReconciliationOrphanStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub readiness_status: DashboardValue<String>,
    pub diagnostic: DashboardValue<String>,
    pub missing_artifacts: DashboardValue<String>,
    pub schema_diagnostics: DashboardValue<String>,
    pub provenance_diagnostics: DashboardValue<String>,
    pub stale_artifacts: DashboardValue<String>,
    pub order_lineage_id: DashboardValue<String>,
    pub local_ledger_status: DashboardValue<String>,
    pub local_order_state: DashboardValue<String>,
    pub local_ledger_ready: DashboardValue<bool>,
    pub restart_readable: DashboardValue<bool>,
    pub exchange_readback_status: DashboardValue<String>,
    pub exchange_readback_mapped: DashboardValue<bool>,
    pub exchange_order_state: DashboardValue<String>,
    pub exchange_order_status: DashboardValue<String>,
    pub open_order_observed: DashboardValue<bool>,
    pub terminal_state_observed: DashboardValue<bool>,
    pub reconciliation_status: DashboardValue<String>,
    pub reconciliation_classified: DashboardValue<bool>,
    pub reconciliation_outcome: DashboardValue<String>,
    pub orphan_status: DashboardValue<String>,
    pub orphan_detection_completed: DashboardValue<bool>,
    pub orphan_detection_outcome: DashboardValue<String>,
    pub orphan_risk_detected: DashboardValue<bool>,
    pub risk_halted: DashboardValue<bool>,
    pub manual_review_required: DashboardValue<bool>,
    pub new_orders_blocked: DashboardValue<bool>,
    pub stale_ledger_restart_required: DashboardValue<bool>,
    pub duplicate_submit_attempted: DashboardValue<bool>,
    pub retry_attempted: DashboardValue<bool>,
    pub cancel_attempted: DashboardValue<bool>,
    pub remediation_attempted: DashboardValue<bool>,
    pub automatic_cancel_allowed: DashboardValue<bool>,
    pub automatic_remediation_allowed: DashboardValue<bool>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub dashboard_cancel_controls_enabled: DashboardValue<bool>,
    pub network_attempted: DashboardValue<bool>,
    pub production_order_submission_allowed: DashboardValue<bool>,
    pub production_order_mutation_allowed: DashboardValue<bool>,
    pub local_order_ledger_path: DashboardValue<String>,
    pub exchange_readback_mapper_path: DashboardValue<String>,
    pub reconciliation_classifier_path: DashboardValue<String>,
    pub orphan_order_detector_path: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionCancelRecoveryStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub readiness_status: DashboardValue<String>,
    pub diagnostic: DashboardValue<String>,
    pub missing_artifacts: DashboardValue<String>,
    pub schema_diagnostics: DashboardValue<String>,
    pub provenance_diagnostics: DashboardValue<String>,
    pub stale_artifacts: DashboardValue<String>,
    pub order_lineage_id: DashboardValue<String>,
    pub cancel_preview_status: DashboardValue<String>,
    pub cancel_request_preview_ready: DashboardValue<bool>,
    pub cancel_reason: DashboardValue<String>,
    pub candidate_count: DashboardValue<u64>,
    pub known_order_id: DashboardValue<String>,
    pub known_client_order_id: DashboardValue<String>,
    pub symbol: DashboardValue<String>,
    pub account_label: DashboardValue<String>,
    pub risk_gate_status: DashboardValue<String>,
    pub risk_gate_result: DashboardValue<String>,
    pub risk_gate_ready: DashboardValue<bool>,
    pub orphan_risk_detected: DashboardValue<bool>,
    pub risk_halted: DashboardValue<bool>,
    pub manual_review_required: DashboardValue<bool>,
    pub new_orders_blocked: DashboardValue<bool>,
    pub approval_lifecycle_status: DashboardValue<String>,
    pub owner_approval_state: DashboardValue<String>,
    pub manual_approval_recorded: DashboardValue<bool>,
    pub approval_lifecycle_valid: DashboardValue<bool>,
    pub approval_consumed: DashboardValue<bool>,
    pub redaction_contract_state: DashboardValue<String>,
    pub cancel_response_redaction_ready: DashboardValue<bool>,
    pub cancel_response_redacted: DashboardValue<bool>,
    pub post_cancel_readback_status: DashboardValue<String>,
    pub post_cancel_readback_ready: DashboardValue<bool>,
    pub readback_state: DashboardValue<String>,
    pub readback_state_class: DashboardValue<String>,
    pub readback_outcome: DashboardValue<String>,
    pub terminal_state_observed: DashboardValue<bool>,
    pub ambiguous_state_observed: DashboardValue<bool>,
    pub incident_closeout_status: DashboardValue<String>,
    pub incident_closeout_ready: DashboardValue<bool>,
    pub audit_trail_ready: DashboardValue<bool>,
    pub audit_traceability_ready: DashboardValue<bool>,
    pub cancel_recovery_lineage_ready: DashboardValue<bool>,
    pub terminal_action_recommendation: DashboardValue<String>,
    pub remaining_risk: DashboardValue<String>,
    pub remaining_risk_requires_manual_review: DashboardValue<bool>,
    pub source_artifact_issues: DashboardValue<String>,
    pub lineage_issues: DashboardValue<String>,
    pub missing_cli_flags: DashboardValue<String>,
    pub actual_cancel_send_allowed: DashboardValue<bool>,
    pub cancel_attempted: DashboardValue<bool>,
    pub cancel_requests_sent: DashboardValue<u64>,
    pub production_order_mutations_attempted: DashboardValue<u64>,
    pub readback_execution_attempted: DashboardValue<bool>,
    pub production_order_state_reads_attempted: DashboardValue<u64>,
    pub network_attempted: DashboardValue<bool>,
    pub network_readback_endpoint_attempted: DashboardValue<bool>,
    pub network_cancel_endpoint_attempted: DashboardValue<bool>,
    pub retry_attempted: DashboardValue<bool>,
    pub remediation_attempted: DashboardValue<bool>,
    pub automatic_cancel_allowed: DashboardValue<bool>,
    pub automatic_remediation_allowed: DashboardValue<bool>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub dashboard_cancel_controls_enabled: DashboardValue<bool>,
    pub dashboard_auto_approval_allowed: DashboardValue<bool>,
    pub dashboard_auto_approval_attempted: DashboardValue<bool>,
    pub cancel_request_preview_path: DashboardValue<String>,
    pub cancel_risk_gate_path: DashboardValue<String>,
    pub manual_owner_approval_lifecycle_path: DashboardValue<String>,
    pub cancel_response_redaction_path: DashboardValue<String>,
    pub post_cancel_readback_path: DashboardValue<String>,
    pub incident_audit_closeout_path: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionActualCancelAuditStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub readiness_status: DashboardValue<String>,
    pub audit_state: DashboardValue<String>,
    pub diagnostic: DashboardValue<String>,
    pub missing_artifacts: DashboardValue<String>,
    pub schema_diagnostics: DashboardValue<String>,
    pub provenance_diagnostics: DashboardValue<String>,
    pub stale_artifacts: DashboardValue<String>,
    pub order_lineage_id: DashboardValue<String>,
    pub approval_lifecycle_status: DashboardValue<String>,
    pub owner_approval_state: DashboardValue<String>,
    pub approval_lifecycle_valid: DashboardValue<bool>,
    pub approval_execution_authorized: DashboardValue<bool>,
    pub risk_gate_status: DashboardValue<String>,
    pub risk_gate_result: DashboardValue<String>,
    pub risk_gate_ready: DashboardValue<bool>,
    pub cancel_attempt_status: DashboardValue<String>,
    pub actual_cancel_command_ready: DashboardValue<bool>,
    pub single_shot_cancel_allowed: DashboardValue<bool>,
    pub request_sent: DashboardValue<bool>,
    pub cancel_attempted: DashboardValue<bool>,
    pub cancel_requests_sent: DashboardValue<u64>,
    pub request_id: DashboardValue<String>,
    pub venue_response_status: DashboardValue<String>,
    pub venue_response_source: DashboardValue<String>,
    pub venue_response_code: DashboardValue<u64>,
    pub venue_response_error_code: DashboardValue<String>,
    pub local_audit_reference: DashboardValue<String>,
    pub readback_status: DashboardValue<String>,
    pub readback_result: DashboardValue<String>,
    pub reconciliation_status: DashboardValue<String>,
    pub readback_state: DashboardValue<String>,
    pub venue_state: DashboardValue<String>,
    pub terminal_state_observed: DashboardValue<bool>,
    pub unknown_observed: DashboardValue<bool>,
    pub dashboard_read_only_consumable: DashboardValue<bool>,
    pub dashboard_audit_view_ready: DashboardValue<bool>,
    pub outcome_status: DashboardValue<String>,
    pub cancel_outcome: DashboardValue<String>,
    pub outcome_category: DashboardValue<String>,
    pub recovered: DashboardValue<bool>,
    pub degraded: DashboardValue<bool>,
    pub failed: DashboardValue<bool>,
    pub partial_success: DashboardValue<bool>,
    pub operator_action_required: DashboardValue<bool>,
    pub residual_risk_visible: DashboardValue<bool>,
    pub request_response_readback_audit_refs_recorded: DashboardValue<bool>,
    pub source_artifact_issues: DashboardValue<String>,
    pub lineage_issues: DashboardValue<String>,
    pub missing_cli_flags: DashboardValue<String>,
    pub actual_cancel_send_allowed: DashboardValue<bool>,
    pub production_order_mutations_attempted: DashboardValue<u64>,
    pub readback_execution_attempted: DashboardValue<bool>,
    pub production_order_state_reads_attempted: DashboardValue<u64>,
    pub network_attempted: DashboardValue<bool>,
    pub network_readback_endpoint_attempted: DashboardValue<bool>,
    pub network_cancel_endpoint_attempted: DashboardValue<bool>,
    pub retry_attempted: DashboardValue<bool>,
    pub remediation_attempted: DashboardValue<bool>,
    pub automatic_cancel_allowed: DashboardValue<bool>,
    pub automatic_remediation_allowed: DashboardValue<bool>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub dashboard_cancel_controls_enabled: DashboardValue<bool>,
    pub bulk_cancel_allowed: DashboardValue<bool>,
    pub second_cancel_attempted: DashboardValue<bool>,
    pub compensation_trade_attempted: DashboardValue<bool>,
    pub cancel_risk_gate_path: DashboardValue<String>,
    pub owner_approval_lifecycle_path: DashboardValue<String>,
    pub actual_cancel_single_shot_path: DashboardValue<String>,
    pub readback_reconciliation_path: DashboardValue<String>,
    pub failure_evidence_path: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionOrderLifecycleAuditStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub readiness_status: DashboardValue<String>,
    pub audit_state: DashboardValue<String>,
    pub risk_visibility: DashboardValue<String>,
    pub diagnostic: DashboardValue<String>,
    pub missing_artifacts: DashboardValue<String>,
    pub schema_diagnostics: DashboardValue<String>,
    pub provenance_diagnostics: DashboardValue<String>,
    pub stale_artifacts: DashboardValue<String>,
    pub source_diagnostics: DashboardValue<String>,
    pub foundation_boundary_status: DashboardValue<String>,
    pub foundation_boundary_diagnostics: DashboardValue<String>,
    pub evidence_source_class: DashboardValue<String>,
    pub adapter_runtime_integrated: DashboardValue<bool>,
    pub foundation_only: DashboardValue<bool>,
    pub exchange_truth_claimed: DashboardValue<bool>,
    pub lifecycle_id: DashboardValue<String>,
    pub attempt_id: DashboardValue<String>,
    pub submit_attempt_state: DashboardValue<String>,
    pub submit_attempt_code: DashboardValue<String>,
    pub owner_approval_state_before_attempt: DashboardValue<String>,
    pub owner_approval_state_after_attempt: DashboardValue<String>,
    pub owner_approval_consumed: DashboardValue<bool>,
    pub production_submit_attempted: DashboardValue<bool>,
    pub readback_required: DashboardValue<bool>,
    pub response_state: DashboardValue<String>,
    pub response_code: DashboardValue<String>,
    pub venue_status: DashboardValue<String>,
    pub venue_order_id: DashboardValue<String>,
    pub client_order_id: DashboardValue<String>,
    pub readback_state: DashboardValue<String>,
    pub readback_code: DashboardValue<String>,
    pub mismatch_fields: DashboardValue<String>,
    pub readback_consistent: DashboardValue<bool>,
    pub readback_missing: DashboardValue<bool>,
    pub readback_failed: DashboardValue<bool>,
    pub failure_category: DashboardValue<String>,
    pub failure_code: DashboardValue<String>,
    pub next_allowed_action: DashboardValue<String>,
    pub no_implicit_retry: DashboardValue<bool>,
    pub unknown_state_visible: DashboardValue<bool>,
    pub audit_closeout_status: DashboardValue<String>,
    pub audit_closed: DashboardValue<bool>,
    pub dashboard_audit_consumable: DashboardValue<bool>,
    pub release_gate_consumable: DashboardValue<bool>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub dashboard_approval_controls_enabled: DashboardValue<bool>,
    pub dashboard_cancel_controls_enabled: DashboardValue<bool>,
    pub retry_attempted: DashboardValue<bool>,
    pub replace_attempted: DashboardValue<bool>,
    pub amend_attempted: DashboardValue<bool>,
    pub flatten_attempted: DashboardValue<bool>,
    pub automatic_cancel_attempted: DashboardValue<bool>,
    pub automatic_remediation_allowed: DashboardValue<bool>,
    pub strategy_continuation_allowed: DashboardValue<bool>,
    pub submit_candidate_path: DashboardValue<String>,
    pub response_redaction_path: DashboardValue<String>,
    pub readback_reconciliation_path: DashboardValue<String>,
    pub failure_no_retry_path: DashboardValue<String>,
    pub audit_closeout_path: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraderTerminalReadModelStatus {
    pub node_id: String,
    pub health: HealthStatus,
    pub readiness_status: DashboardValue<String>,
    pub diagnostic: DashboardValue<String>,
    pub artifact_path: DashboardValue<String>,
    pub contract_version: DashboardValue<String>,
    pub schema_version: DashboardValue<String>,
    pub snapshot_id: DashboardValue<String>,
    pub snapshot_kind: DashboardValue<String>,
    pub snapshot_health_status: DashboardValue<String>,
    pub freshness_status: DashboardValue<String>,
    pub source_type: DashboardValue<String>,
    pub source_ref: DashboardValue<String>,
    pub redaction_state: DashboardValue<String>,
    pub account_status: DashboardValue<String>,
    pub positions_status: DashboardValue<String>,
    pub orders_status: DashboardValue<String>,
    pub fills_status: DashboardValue<String>,
    pub risk_status: DashboardValue<String>,
    pub lifecycle_status: DashboardValue<String>,
    pub operation_entry_status: DashboardValue<String>,
    pub account_summary: DashboardValue<String>,
    pub positions_summary: DashboardValue<String>,
    pub account_freshness_status: DashboardValue<String>,
    pub account_source_type: DashboardValue<String>,
    pub account_source_ref: DashboardValue<String>,
    pub account_redaction_state: DashboardValue<String>,
    pub account_risk_state: DashboardValue<String>,
    pub account_equity: DashboardValue<String>,
    pub account_available_balance: DashboardValue<String>,
    pub account_balance_entry_count: DashboardValue<String>,
    pub positions_freshness_status: DashboardValue<String>,
    pub positions_source_type: DashboardValue<String>,
    pub positions_source_ref: DashboardValue<String>,
    pub positions_redaction_state: DashboardValue<String>,
    pub positions_account_id: DashboardValue<String>,
    pub positions_net_position_side: DashboardValue<String>,
    pub positions_quantity: DashboardValue<String>,
    pub positions_notional: DashboardValue<String>,
    pub positions_precision: DashboardValue<String>,
    pub positions_lineage: DashboardValue<String>,
    pub orders_freshness_status: DashboardValue<String>,
    pub orders_source_type: DashboardValue<String>,
    pub orders_source_ref: DashboardValue<String>,
    pub orders_redaction_state: DashboardValue<String>,
    pub orders_lifecycle_state: DashboardValue<String>,
    pub orders_client_order_id: DashboardValue<String>,
    pub orders_request_digest: DashboardValue<String>,
    pub orders_attempt_id: DashboardValue<String>,
    pub orders_approval_id: DashboardValue<String>,
    pub orders_readback_status: DashboardValue<String>,
    pub orders_audit_state: DashboardValue<String>,
    pub orders_ledger_present: DashboardValue<String>,
    pub orders_duplicate_attempt_detected: DashboardValue<String>,
    pub orders_no_retry: DashboardValue<String>,
    pub orders_diagnostics: DashboardValue<String>,
    pub orders_lineage: DashboardValue<String>,
    pub orders_exchange_truth: DashboardValue<String>,
    pub orders_adapter_runtime_integrated: DashboardValue<String>,
    pub orders_values_are_exchange_truth: DashboardValue<String>,
    pub fills_freshness_status: DashboardValue<String>,
    pub fills_source_type: DashboardValue<String>,
    pub fills_source_ref: DashboardValue<String>,
    pub fills_redaction_state: DashboardValue<String>,
    pub fills_fill_id: DashboardValue<String>,
    pub fills_execution_id: DashboardValue<String>,
    pub fills_order_id: DashboardValue<String>,
    pub fills_client_order_id: DashboardValue<String>,
    pub fills_fill_status: DashboardValue<String>,
    pub fills_order_linkage_status: DashboardValue<String>,
    pub fills_reconciliation_status: DashboardValue<String>,
    pub fills_quantity: DashboardValue<String>,
    pub fills_cumulative_quantity: DashboardValue<String>,
    pub fills_remaining_quantity: DashboardValue<String>,
    pub fills_quantity_precision: DashboardValue<String>,
    pub fills_price: DashboardValue<String>,
    pub fills_price_precision: DashboardValue<String>,
    pub fills_precision_status: DashboardValue<String>,
    pub fills_duplicate_fill_detected: DashboardValue<String>,
    pub fills_partial_fill_detected: DashboardValue<String>,
    pub fills_risk_projection_input: DashboardValue<String>,
    pub fills_diagnostics: DashboardValue<String>,
    pub fills_lineage: DashboardValue<String>,
    pub fills_exchange_truth: DashboardValue<String>,
    pub fills_adapter_runtime_integrated: DashboardValue<String>,
    pub fills_values_are_exchange_truth: DashboardValue<String>,
    pub risk_freshness_status: DashboardValue<String>,
    pub risk_source_type: DashboardValue<String>,
    pub risk_source_ref: DashboardValue<String>,
    pub risk_redaction_state: DashboardValue<String>,
    pub risk_state: DashboardValue<String>,
    pub risk_priority_state: DashboardValue<String>,
    pub risk_visible: DashboardValue<String>,
    pub risk_manual_review_required: DashboardValue<String>,
    pub risk_halted: DashboardValue<String>,
    pub risk_mismatch_detected: DashboardValue<String>,
    pub risk_freshness_rollup: DashboardValue<String>,
    pub risk_critical_evidence_complete: DashboardValue<String>,
    pub risk_alert_severity: DashboardValue<String>,
    pub risk_alert_missing_evidence: DashboardValue<String>,
    pub risk_alert_stale_source: DashboardValue<String>,
    pub risk_alert_schema_mismatch: DashboardValue<String>,
    pub risk_alert_redaction_breach: DashboardValue<String>,
    pub risk_alert_forbidden_control_request: DashboardValue<String>,
    pub risk_alert_summary: DashboardValue<String>,
    pub risk_diagnostics: DashboardValue<String>,
    pub risk_lineage: DashboardValue<String>,
    pub audit_freshness_status: DashboardValue<String>,
    pub audit_source_type: DashboardValue<String>,
    pub audit_source_ref: DashboardValue<String>,
    pub audit_redaction_state: DashboardValue<String>,
    pub audit_state: DashboardValue<String>,
    pub audit_closed: DashboardValue<String>,
    pub audit_required_evidence_complete: DashboardValue<String>,
    pub audit_required_components_complete: DashboardValue<String>,
    pub audit_missing_evidence: DashboardValue<String>,
    pub audit_release_provenance: DashboardValue<String>,
    pub audit_artifact_digest: DashboardValue<String>,
    pub audit_artifact_sha: DashboardValue<String>,
    pub audit_provenance_mismatch: DashboardValue<String>,
    pub audit_diagnostics: DashboardValue<String>,
    pub audit_lineage: DashboardValue<String>,
    pub operation_entry_freshness_status: DashboardValue<String>,
    pub operation_entry_source_type: DashboardValue<String>,
    pub operation_entry_source_ref: DashboardValue<String>,
    pub operation_entry_redaction_state: DashboardValue<String>,
    pub operation_intent_preview: DashboardValue<String>,
    pub operation_owner_approval_ref: DashboardValue<String>,
    pub operation_risk_decision_ref: DashboardValue<String>,
    pub operation_audit_evidence_ref: DashboardValue<String>,
    pub operation_entry_disabled: DashboardValue<String>,
    pub operation_entry_blocked_reason: DashboardValue<String>,
    pub operation_missing_owner_approval: DashboardValue<String>,
    pub operation_missing_risk_gate: DashboardValue<String>,
    pub operation_missing_audit_gate: DashboardValue<String>,
    pub operation_stale_read_model: DashboardValue<String>,
    pub operation_provenance_mismatch: DashboardValue<String>,
    pub operation_gates_complete: DashboardValue<String>,
    pub operation_ungated_attempted: DashboardValue<String>,
    pub operation_attempt_status: DashboardValue<String>,
    pub operation_ungated_attempt_fail_closed: DashboardValue<String>,
    pub v24_order_control_preview_status: DashboardValue<String>,
    pub v24_order_intent_status: DashboardValue<String>,
    pub v24_execution_policy_status: DashboardValue<String>,
    pub v24_rate_limit_status: DashboardValue<String>,
    pub v24_slicing_status: DashboardValue<String>,
    pub v24_cancel_replace_amend_status: DashboardValue<String>,
    pub v24_retry_policy_status: DashboardValue<String>,
    pub v24_readback_audit_status: DashboardValue<String>,
    pub v24_blocked_reasons: DashboardValue<String>,
    pub v24_scope_key: DashboardValue<String>,
    pub v24_source_provenance: DashboardValue<String>,
    pub v24_redaction_state: DashboardValue<String>,
    pub v24_order_intent_ref: DashboardValue<String>,
    pub v24_policy_ref: DashboardValue<String>,
    pub v24_rate_limit_ref: DashboardValue<String>,
    pub v24_slicing_ref: DashboardValue<String>,
    pub v24_cancel_replace_amend_ref: DashboardValue<String>,
    pub v24_retry_policy_ref: DashboardValue<String>,
    pub v24_readback_ref: DashboardValue<String>,
    pub v24_audit_ref: DashboardValue<String>,
    pub v24_provenance_ref: DashboardValue<String>,
    pub v24_dashboard_redacted_ref: DashboardValue<String>,
    pub v24_preview_evidence_present: DashboardValue<String>,
    pub v24_missing_preview_evidence: DashboardValue<String>,
    pub v24_forbidden_control_detected: DashboardValue<String>,
    pub v24_render_smoke_case: DashboardValue<String>,
    pub v25_dashboard_surface_status: DashboardValue<String>,
    pub v25_diagnostics_gate_status: DashboardValue<String>,
    pub v25_slo_status: DashboardValue<String>,
    pub v25_freshness_threshold_status: DashboardValue<String>,
    pub v25_staleness_reasons: DashboardValue<String>,
    pub v25_diagnostic_severity: DashboardValue<String>,
    pub v25_source_truth_status: DashboardValue<String>,
    pub v25_release_provenance_status: DashboardValue<String>,
    pub v25_no_remediation_status: DashboardValue<String>,
    pub v25_monitoring_status: DashboardValue<String>,
    pub v25_monitoring_runtime_health: DashboardValue<String>,
    pub v25_monitoring_effective_status: DashboardValue<String>,
    pub v25_monitoring_freshness_status: DashboardValue<String>,
    pub v25_monitoring_source_ref: DashboardValue<String>,
    pub v25_monitoring_redaction_state: DashboardValue<String>,
    pub v25_alert_status: DashboardValue<String>,
    pub v25_alert_highest_severity: DashboardValue<String>,
    pub v25_alert_route_status: DashboardValue<String>,
    pub v25_alert_dedupe_key: DashboardValue<String>,
    pub v25_incident_status: DashboardValue<String>,
    pub v25_incident_current_state: DashboardValue<String>,
    pub v25_incident_ack_status: DashboardValue<String>,
    pub v25_incident_owner: DashboardValue<String>,
    pub v25_runbook_status: DashboardValue<String>,
    pub v25_runbook_decision_type: DashboardValue<String>,
    pub v25_runbook_decision_status: DashboardValue<String>,
    pub v25_runbook_evidence_ref: DashboardValue<String>,
    pub v25_dr_preview_status: DashboardValue<String>,
    pub v25_dr_scenario: DashboardValue<String>,
    pub v25_dr_recovery_point: DashboardValue<String>,
    pub v25_dr_operator_approval_status: DashboardValue<String>,
    pub v25_dr_snapshot_lineage: DashboardValue<String>,
    pub v25_surface_blocking_reasons: DashboardValue<String>,
    pub v26_dashboard_admin_surface_status: DashboardValue<String>,
    pub v26_permission_boundary_status: DashboardValue<String>,
    pub v26_permission_roles_checked: DashboardValue<String>,
    pub v26_operation_audit_status: DashboardValue<String>,
    pub v26_operation_audit_lineage: DashboardValue<String>,
    pub v26_deployment_provenance_status: DashboardValue<String>,
    pub v26_deployment_environment: DashboardValue<String>,
    pub v26_upgrade_rollback_status: DashboardValue<String>,
    pub v26_upgrade_rollback_preview: DashboardValue<String>,
    pub v26_stability_status: DashboardValue<String>,
    pub v26_stability_degradation_reason: DashboardValue<String>,
    pub v26_admin_surface_blocking_reasons: DashboardValue<String>,
    pub orders_summary: DashboardValue<String>,
    pub fills_summary: DashboardValue<String>,
    pub risk_summary: DashboardValue<String>,
    pub lifecycle_summary: DashboardValue<String>,
    pub missing_components: DashboardValue<String>,
    pub blocking_reasons: DashboardValue<String>,
    pub component_diagnostics: DashboardValue<String>,
    pub new_submit_capability: DashboardValue<bool>,
    pub dashboard_order_controls_enabled: DashboardValue<bool>,
    pub dashboard_approval_controls_enabled: DashboardValue<bool>,
    pub dashboard_cancel_controls_enabled: DashboardValue<bool>,
    pub dashboard_retry_controls_enabled: DashboardValue<bool>,
    pub dashboard_fill_controls_enabled: DashboardValue<bool>,
    pub dashboard_submit_controls_enabled: DashboardValue<bool>,
    pub dashboard_replace_controls_enabled: DashboardValue<bool>,
    pub dashboard_amend_controls_enabled: DashboardValue<bool>,
    pub dashboard_flatten_controls_enabled: DashboardValue<bool>,
    pub trader_terminal_order_ticket_enabled: DashboardValue<bool>,
    pub trader_terminal_live_trading_claim: DashboardValue<bool>,
    pub production_order_submission_allowed: DashboardValue<bool>,
    pub production_order_mutation_allowed: DashboardValue<bool>,
    pub retry_replace_amend_flatten_allowed: DashboardValue<bool>,
    pub order_permission_control_allowed: DashboardValue<bool>,
    pub retry_order_allowed: DashboardValue<bool>,
    pub automatic_cancel_allowed: DashboardValue<bool>,
    pub automatic_order_remediation_allowed: DashboardValue<bool>,
    pub funds_transfer_allowed: DashboardValue<bool>,
    pub account_configuration_mutation_allowed: DashboardValue<bool>,
    pub auto_flatten_position_allowed: DashboardValue<bool>,
    pub automatic_position_repair_allowed: DashboardValue<bool>,
    pub execution_algorithm_allowed: DashboardValue<bool>,
    pub automatic_fill_repair_allowed: DashboardValue<bool>,
    pub automatic_reconciliation_repair_allowed: DashboardValue<bool>,
    pub dashboard_risk_controls_enabled: DashboardValue<bool>,
    pub automatic_risk_action_allowed: DashboardValue<bool>,
    pub automatic_risk_repair_allowed: DashboardValue<bool>,
    pub automatic_alert_action_allowed: DashboardValue<bool>,
    pub automatic_audit_action_allowed: DashboardValue<bool>,
    pub automatic_provenance_repair_allowed: DashboardValue<bool>,
    pub manual_operation_entry_enabled: DashboardValue<bool>,
    pub manual_operation_submit_allowed: DashboardValue<bool>,
    pub manual_operation_cancel_allowed: DashboardValue<bool>,
    pub manual_operation_retry_allowed: DashboardValue<bool>,
    pub manual_operation_replace_allowed: DashboardValue<bool>,
    pub manual_operation_amend_allowed: DashboardValue<bool>,
    pub manual_operation_flatten_allowed: DashboardValue<bool>,
    pub automatic_operation_action_allowed: DashboardValue<bool>,
    pub product_grade_trading_terminal_claim: DashboardValue<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeModuleStatus {
    pub module_name: String,
    pub status: DashboardValue<String>,
    pub health: HealthStatus,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
    pub evidence_source: DashboardValue<String>,
}

impl RuntimeModuleStatus {
    #[must_use]
    pub fn unknown(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
            status: DashboardValue::unknown(),
            health: HealthStatus::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
            evidence_source: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogStatus {
    pub log_id: String,
    pub node_id: DashboardValue<String>,
    pub path: DashboardValue<String>,
    pub availability: DashboardAvailability,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl LogStatus {
    #[must_use]
    pub fn unknown(log_id: impl Into<String>) -> Self {
        Self {
            log_id: log_id.into(),
            node_id: DashboardValue::unknown(),
            path: DashboardValue::unknown(),
            availability: DashboardAvailability::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricStatus {
    pub metric_id: String,
    pub node_id: DashboardValue<String>,
    pub value: DashboardValue<String>,
    pub availability: DashboardAvailability,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl MetricStatus {
    #[must_use]
    pub fn unknown(metric_id: impl Into<String>) -> Self {
        Self {
            metric_id: metric_id.into(),
            node_id: DashboardValue::unknown(),
            value: DashboardValue::unknown(),
            availability: DashboardAvailability::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertSummary {
    pub active_count: u64,
    pub counts_by_severity: BTreeMap<String, u64>,
    pub active: Vec<DashboardAlert>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardAlert {
    pub alert_id: String,
    pub severity: String,
    pub source: String,
    pub message: String,
    pub first_seen_at: DashboardValue<String>,
    pub last_seen_at: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlStatus {
    pub action: String,
    pub availability: DashboardAvailability,
    pub enabled: bool,
    pub reason: DashboardValue<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlActionStatus {
    Accepted,
    Rejected,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    NotSupported,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlActionResponse {
    pub action_id: String,
    pub action: String,
    pub status: ControlActionStatus,
    pub previous_state: LifecycleStatus,
    pub current_state: LifecycleStatus,
    pub started_at: DashboardValue<String>,
    pub finished_at: DashboardValue<String>,
    pub error_code: DashboardValue<String>,
    pub message: DashboardValue<String>,
    pub observability_ref: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardGap {
    pub field_path: String,
    pub reason: DashboardAvailability,
    pub owner_task: DashboardValue<String>,
    pub notes: DashboardValue<String>,
}

impl DashboardGap {
    #[must_use]
    pub fn new(
        field_path: impl Into<String>,
        reason: DashboardAvailability,
        owner_task: impl Into<String>,
        notes: impl Into<String>,
    ) -> Self {
        Self {
            field_path: field_path.into(),
            reason,
            owner_task: DashboardValue::available(owner_task.into()),
            notes: DashboardValue::available(notes.into()),
        }
    }
}

/// Builds a dashboard snapshot from local supervisor registry and node artifacts.
///
/// # Errors
///
/// Returns an error if the registry file exists but cannot be read. Missing or
/// invalid supervisor artifacts are represented as explicit dashboard gaps so
/// callers can still render an owner-visible partial snapshot.
pub fn snapshot_from_supervisor_artifacts(
    registry_path: impl AsRef<FsPath>,
    generated_at: impl Into<String>,
) -> anyhow::Result<DashboardSnapshot> {
    snapshot_from_supervisor_artifacts_with_workflow_root(
        registry_path,
        None::<&FsPath>,
        generated_at,
    )
}

/// Builds a dashboard snapshot from local supervisor artifacts plus an
/// optional explicit workflow artifact root.
///
/// # Errors
///
/// Returns an error if the registry file exists but cannot be read.
pub fn snapshot_from_supervisor_artifacts_with_workflow_root(
    registry_path: impl AsRef<FsPath>,
    workflow_root: Option<&FsPath>,
    generated_at: impl Into<String>,
) -> anyhow::Result<DashboardSnapshot> {
    let registry_path = registry_path.as_ref();
    let mut snapshot = DashboardSnapshot::empty(generated_at);
    if let Some(workflow_root) = workflow_root {
        snapshot.workflow_artifacts =
            workflow_artifacts_from_explicit_root(workflow_root, &mut snapshot.gaps);
        snapshot
            .overview
            .apply_workflow_artifacts(&snapshot.workflow_artifacts);
    }

    if !registry_path.exists() {
        snapshot.gaps.push(DashboardGap::new(
            "supervisor.registry",
            DashboardAvailability::Unknown,
            "V03-004",
            format!("监督器注册表工件 '{}' 缺失", registry_path.display()),
        ));
        return Ok(snapshot);
    }

    let raw = fs::read_to_string(registry_path).with_context(|| {
        format!(
            "failed to read supervisor registry '{}'",
            registry_path.display()
        )
    })?;
    let registry = match serde_json::from_str::<SupervisorRegistry>(&raw) {
        Ok(registry) => registry,
        Err(error) => {
            snapshot.gaps.push(DashboardGap::new(
                "supervisor.registry",
                DashboardAvailability::Unknown,
                "V03-004",
                format!("监督器注册表工件无效：{error}"),
            ));
            return Ok(snapshot);
        }
    };
    snapshot.workflow_artifacts =
        workflow_artifacts_from_paths(registry_path, workflow_root, &mut snapshot.gaps);
    if registry.nodes.is_empty() {
        snapshot.gaps.push(DashboardGap::new(
            "nodes",
            DashboardAvailability::NotConfigured,
            "V03-004",
            "监督器注册表中没有节点",
        ));
        snapshot
            .overview
            .apply_workflow_artifacts(&snapshot.workflow_artifacts);
        return Ok(snapshot);
    }

    let mut nodes = Vec::with_capacity(registry.nodes.len());
    let mut statuses = Vec::with_capacity(registry.nodes.len());

    for record in registry.nodes.values() {
        let ArtifactStatus {
            status,
            status_availability,
            mut gaps,
        } = read_status_artifact(record);

        gaps.extend(process_gaps(record));
        let RuntimeModuleReadout {
            modules,
            gaps: module_gaps,
        } = runtime_modules_from_status(record, &status);
        gaps.extend(module_gaps);
        let mut node = DashboardNodeSummary::from_status(&status);
        node.node_id = record.node_id.clone();
        node.process_state = record.process.state;
        node.pid = record.process.pid.clone();
        node.gaps = gaps.clone();
        snapshot.gaps.extend(gaps);

        snapshot
            .data_sources
            .push(data_source_from_status(record, &status));
        snapshot
            .execution_gateways
            .push(execution_gateway_from_status(record, &status));
        let read_model_runtime = trader_terminal_read_model_status_from_record(record);
        if read_model_runtime.health != HealthStatus::Healthy {
            snapshot.gaps.push(DashboardGap::new(
                format!("read_model_runtime.{}", record.node_id),
                dashboard_availability_from_read_model_health(read_model_runtime.health),
                "V211-005",
                read_model_runtime
                    .diagnostic
                    .value
                    .clone()
                    .unwrap_or_else(|| {
                        "Unified Read Model runtime artifact is not ready".to_string()
                    }),
            ));
        }
        snapshot.read_model_runtime.push(read_model_runtime);
        snapshot.runtime_modules.extend(modules);
        if let Some(strategy_runtime) = strategy_runtime_from_record(record) {
            snapshot.strategy_runtime.push(strategy_runtime);
        }
        if let Some(production_shadow) = production_shadow_from_record(record) {
            if let Some(preflight_readiness) =
                preflight_readiness_from_production_shadow(&production_shadow)
            {
                snapshot.preflight_readiness.push(preflight_readiness);
            }
            snapshot.production_shadow.push(production_shadow);
        }
        if let Some(live_alpha_dry_run) = live_alpha_dry_run_from_record(record) {
            snapshot.live_alpha_dry_run.push(live_alpha_dry_run);
        }
        if let Some(production_mutation_evidence) = production_mutation_evidence_from_record(record)
        {
            snapshot
                .production_mutation_evidence
                .push(production_mutation_evidence);
        }
        if let Some(reconciliation_orphan) = production_reconciliation_orphan_from_record(record) {
            snapshot
                .production_reconciliation_orphan
                .push(reconciliation_orphan);
        }
        if let Some(cancel_recovery) = production_cancel_recovery_from_record(record) {
            snapshot.production_cancel_recovery.push(cancel_recovery);
        }
        if let Some(actual_cancel_audit) = production_actual_cancel_audit_from_record(record) {
            snapshot
                .production_actual_cancel_audit
                .push(actual_cancel_audit);
        }
        if let Some(order_lifecycle_audit) = production_order_lifecycle_audit_from_record(record) {
            snapshot
                .production_order_lifecycle_audit
                .push(order_lifecycle_audit);
        }
        snapshot
            .logs
            .extend(log_statuses_from_record(record, &status));
        snapshot
            .metrics
            .extend(metric_statuses_from_record(record, status_availability));

        nodes.push(node);
        statuses.push(status);
    }

    snapshot.overview = DashboardOverview::from_nodes(&nodes);
    snapshot
        .overview
        .apply_workflow_artifacts(&snapshot.workflow_artifacts);
    snapshot.nodes = nodes;
    snapshot.risk = aggregate_risk_status(&statuses);
    snapshot.controls = control_statuses_from_nodes(&snapshot.nodes);
    snapshot.alerts = alert_summary_from_gaps(&snapshot.gaps);
    Ok(snapshot)
}

struct ArtifactStatus {
    status: NodeStatus,
    status_availability: DashboardAvailability,
    gaps: Vec<DashboardGap>,
}

fn read_status_artifact(record: &SupervisorNodeRecord) -> ArtifactStatus {
    let mut gaps = Vec::new();

    if !record.status_path.exists() {
        let mut status = record.last_known_status.clone();
        status.generated_at = SnapshotValue::stale();
        status.last_error = Some(format!("状态工件 '{}' 缺失", record.status_path.display()));
        gaps.push(DashboardGap::new(
            format!("nodes.{}.status", record.node_id),
            DashboardAvailability::Unknown,
            "V03-004",
            format!("状态工件 '{}' 缺失", record.status_path.display()),
        ));
        return ArtifactStatus {
            status,
            status_availability: DashboardAvailability::Unknown,
            gaps,
        };
    }

    let raw = match fs::read_to_string(&record.status_path) {
        Ok(raw) => raw,
        Err(error) => {
            let mut status = record.last_known_status.clone();
            status.generated_at = SnapshotValue::stale();
            status.last_error = Some(format!("读取状态工件失败：{error}"));
            gaps.push(DashboardGap::new(
                format!("nodes.{}.status", record.node_id),
                DashboardAvailability::Unknown,
                "V03-004",
                format!(
                    "读取状态工件 '{}' 失败：{error}",
                    record.status_path.display()
                ),
            ));
            return ArtifactStatus {
                status,
                status_availability: DashboardAvailability::Unknown,
                gaps,
            };
        }
    };

    match serde_json::from_str::<NodeStatus>(&raw) {
        Ok(status) if status.node_id == record.node_id => {
            let mut availability = availability_from_registry_state(record.status_artifact);
            if status.generated_at.availability == SnapshotAvailability::Stale {
                availability = DashboardAvailability::Stale;
                gaps.push(DashboardGap::new(
                    format!("nodes.{}.status.generated_at", record.node_id),
                    DashboardAvailability::Stale,
                    "V03-004",
                    "状态工件 generated_at 已过期",
                ));
            }
            ArtifactStatus {
                status,
                status_availability: availability,
                gaps,
            }
        }
        Ok(status) => {
            let mut fallback = record.last_known_status.clone();
            fallback.node_id.clone_from(&record.node_id);
            fallback.generated_at = SnapshotValue::stale();
            fallback.last_error = Some(format!(
                "状态节点身份不匹配：注册表节点 '{}' 收到运行时节点 '{}'",
                record.node_id, status.node_id
            ));
            gaps.push(DashboardGap::new(
                format!("nodes.{}.status.node_id", record.node_id),
                DashboardAvailability::Unknown,
                "P0-006",
                fallback.last_error.clone().unwrap(),
            ));
            ArtifactStatus {
                status: fallback,
                status_availability: DashboardAvailability::Unknown,
                gaps,
            }
        }
        Err(error) => {
            let mut status = record.last_known_status.clone();
            status.generated_at = SnapshotValue::stale();
            status.last_error = Some(format!("状态工件无效：{error}"));
            gaps.push(DashboardGap::new(
                format!("nodes.{}.status", record.node_id),
                DashboardAvailability::Unknown,
                "V03-004",
                format!("状态工件 '{}' 无效：{error}", record.status_path.display()),
            ));
            ArtifactStatus {
                status,
                status_availability: DashboardAvailability::Unknown,
                gaps,
            }
        }
    }
}

fn process_gaps(record: &SupervisorNodeRecord) -> Vec<DashboardGap> {
    let mut gaps = Vec::new();
    if record.process.state == SupervisorProcessState::Stale {
        gaps.push(DashboardGap::new(
            format!("nodes.{}.process", record.node_id),
            DashboardAvailability::Stale,
            "V03-004",
            "监督器进程状态已过期",
        ));
    }
    if record.status_artifact == RegistryArtifactState::Stale {
        gaps.push(DashboardGap::new(
            format!("nodes.{}.status", record.node_id),
            DashboardAvailability::Stale,
            "V03-004",
            "注册表将状态工件标记为过期",
        ));
    }
    if record.metrics_artifact == RegistryArtifactState::Stale {
        gaps.push(DashboardGap::new(
            format!("nodes.{}.metrics", record.node_id),
            DashboardAvailability::Stale,
            "V03-004",
            "注册表将指标工件标记为过期",
        ));
    }
    gaps
}

fn data_source_from_status(record: &SupervisorNodeRecord, status: &NodeStatus) -> DataSourceStatus {
    DataSourceStatus {
        source_id: format!("{}:data", record.node_id),
        source_kind: DashboardValue::available("supervisor_artifact".to_string()),
        provider: DashboardValue::available("local".to_string()),
        connection: status.data_connection,
        freshness: dashboard_value_from_snapshot(&status.generated_at),
        lag_ms: DashboardValue::unknown(),
        health: health_from_connection(status.data_connection),
        last_error: optional_dashboard_value(status.last_error.clone()),
    }
}

fn execution_gateway_from_status(
    record: &SupervisorNodeRecord,
    status: &NodeStatus,
) -> ExecutionGatewayStatus {
    ExecutionGatewayStatus {
        gateway_id: status
            .execution
            .gateway_id
            .value
            .clone()
            .unwrap_or_else(|| format!("{}:execution", record.node_id)),
        venue: DashboardValue::unknown(),
        connection: status.execution.connection,
        started: dashboard_value_from_snapshot(&status.execution.started),
        account_ref: DashboardValue::redacted(),
        order_counts: OrderCountSummary {
            open: dashboard_value_from_snapshot(&status.execution.orders_open),
            inflight: dashboard_value_from_snapshot(&status.execution.orders_inflight),
            closed: dashboard_value_from_snapshot(&status.execution.orders_closed),
        },
        last_report_at: dashboard_value_from_snapshot(&status.execution.last_report_at),
        last_error: optional_dashboard_value(status.execution.last_error.clone()),
    }
}

struct RuntimeModuleReadout {
    modules: Vec<RuntimeModuleStatus>,
    gaps: Vec<DashboardGap>,
}

fn runtime_modules_from_status(
    record: &SupervisorNodeRecord,
    status: &NodeStatus,
) -> RuntimeModuleReadout {
    let evidence_source = DashboardValue::available(record.status_path.display().to_string());
    let logging = logging_module_status(record, status);
    let metrics_writer = metrics_writer_module_status(record, status);
    let mut modules = vec![
        RuntimeModuleStatus {
            module_name: module_name(record, "LiveNode"),
            status: DashboardValue::available(json_label(&status.lifecycle_state)),
            health: derive_node_health(status),
            last_seen_at: dashboard_value_from_snapshot(&status.generated_at),
            last_error: redacted_optional_dashboard_error(status.last_error.as_deref()),
            evidence_source: evidence_source.clone(),
        },
        RuntimeModuleStatus {
            module_name: module_name(record, "DataEngine"),
            status: DashboardValue::available(json_label(&status.data_connection)),
            health: health_from_connection(status.data_connection),
            last_seen_at: dashboard_value_from_snapshot(&status.generated_at),
            last_error: DashboardValue::unknown(),
            evidence_source: evidence_source.clone(),
        },
        RuntimeModuleStatus {
            module_name: module_name(record, "ExecutionEngine"),
            status: DashboardValue::available(json_label(&status.execution.connection)),
            health: health_from_connection(status.execution.connection),
            last_seen_at: dashboard_value_from_snapshot(&status.generated_at),
            last_error: redacted_optional_dashboard_error(status.execution.last_error.as_deref()),
            evidence_source: evidence_source.clone(),
        },
        RuntimeModuleStatus {
            module_name: module_name(record, "RiskEngine"),
            status: DashboardValue::available(json_label(&status.risk.trading_state)),
            health: status.risk.health,
            last_seen_at: dashboard_value_from_snapshot(&status.generated_at),
            last_error: redacted_optional_dashboard_error(status.risk.last_error.as_deref()),
            evidence_source,
        },
        logging.clone(),
        metrics_writer.clone(),
        supervisor_module_status(record),
    ];

    let mut gaps = Vec::new();
    if logging.status.availability != DashboardAvailability::Available {
        gaps.push(DashboardGap::new(
            module_gap_path(record, "Logging"),
            logging.status.availability,
            "V03-008",
            "一个或多个日志工件不可用",
        ));
    }
    if metrics_writer.status.availability != DashboardAvailability::Available {
        gaps.push(DashboardGap::new(
            module_gap_path(record, "Metrics writer"),
            metrics_writer.status.availability,
            "V03-008",
            "指标写入工件不可用",
        ));
    }
    for module in ["NautilusKernel", "Portfolio", "Cache", "MessageBus"] {
        let (status, gap) =
            unsupported_runtime_module(record, module, "监督器工件暂未暴露该模块细节");
        modules.push(status);
        gaps.push(gap);
    }

    RuntimeModuleReadout { modules, gaps }
}

fn trader_terminal_read_model_status_from_record(
    record: &SupervisorNodeRecord,
) -> TraderTerminalReadModelStatus {
    let artifact_path = record
        .artifact_root
        .join(TRADER_TERMINAL_READ_MODEL_ARTIFACT_RELATIVE_PATH);
    if !artifact_path.exists() {
        return degraded_trader_terminal_read_model_status(
            record,
            &artifact_path,
            HealthStatus::Degraded,
            "missing_artifact",
            "canonical_unified_read_model_artifact_missing",
        );
    }

    let raw = match fs::read_to_string(&artifact_path) {
        Ok(raw) => raw,
        Err(error) => {
            return degraded_trader_terminal_read_model_status(
                record,
                &artifact_path,
                HealthStatus::Error,
                "artifact_unreadable",
                format!("canonical_unified_read_model_artifact_unreadable:{error}"),
            );
        }
    };
    let value: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            return degraded_trader_terminal_read_model_status(
                record,
                &artifact_path,
                HealthStatus::Error,
                "invalid_artifact",
                format!("canonical_unified_read_model_artifact_invalid_json:{error}"),
            );
        }
    };

    trader_terminal_read_model_status_from_value(record, &artifact_path, &value)
}

fn degraded_trader_terminal_read_model_status(
    record: &SupervisorNodeRecord,
    artifact_path: &FsPath,
    health: HealthStatus,
    readiness_status: impl Into<String>,
    diagnostic: impl Into<String>,
) -> TraderTerminalReadModelStatus {
    TraderTerminalReadModelStatus {
        node_id: record.node_id.clone(),
        health,
        readiness_status: DashboardValue::available(readiness_status.into()),
        diagnostic: DashboardValue::available(diagnostic.into()),
        artifact_path: DashboardValue::available(artifact_path.display().to_string()),
        contract_version: DashboardValue::unknown(),
        schema_version: DashboardValue::unknown(),
        snapshot_id: DashboardValue::unknown(),
        snapshot_kind: DashboardValue::unknown(),
        snapshot_health_status: DashboardValue::unknown(),
        freshness_status: DashboardValue::unknown(),
        source_type: DashboardValue::unknown(),
        source_ref: DashboardValue::unknown(),
        redaction_state: DashboardValue::unknown(),
        account_status: DashboardValue::unknown(),
        positions_status: DashboardValue::unknown(),
        orders_status: DashboardValue::unknown(),
        fills_status: DashboardValue::unknown(),
        risk_status: DashboardValue::unknown(),
        lifecycle_status: DashboardValue::unknown(),
        operation_entry_status: DashboardValue::available(
            "blocked_missing_operation_entry_contract".to_string(),
        ),
        account_summary: DashboardValue::unknown(),
        positions_summary: DashboardValue::unknown(),
        account_freshness_status: DashboardValue::unknown(),
        account_source_type: DashboardValue::unknown(),
        account_source_ref: DashboardValue::unknown(),
        account_redaction_state: DashboardValue::unknown(),
        account_risk_state: DashboardValue::unknown(),
        account_equity: DashboardValue::unknown(),
        account_available_balance: DashboardValue::unknown(),
        account_balance_entry_count: DashboardValue::unknown(),
        positions_freshness_status: DashboardValue::unknown(),
        positions_source_type: DashboardValue::unknown(),
        positions_source_ref: DashboardValue::unknown(),
        positions_redaction_state: DashboardValue::unknown(),
        positions_account_id: DashboardValue::unknown(),
        positions_net_position_side: DashboardValue::unknown(),
        positions_quantity: DashboardValue::unknown(),
        positions_notional: DashboardValue::unknown(),
        positions_precision: DashboardValue::unknown(),
        positions_lineage: DashboardValue::unknown(),
        orders_freshness_status: DashboardValue::unknown(),
        orders_source_type: DashboardValue::unknown(),
        orders_source_ref: DashboardValue::unknown(),
        orders_redaction_state: DashboardValue::unknown(),
        orders_lifecycle_state: DashboardValue::unknown(),
        orders_client_order_id: DashboardValue::unknown(),
        orders_request_digest: DashboardValue::unknown(),
        orders_attempt_id: DashboardValue::unknown(),
        orders_approval_id: DashboardValue::unknown(),
        orders_readback_status: DashboardValue::unknown(),
        orders_audit_state: DashboardValue::unknown(),
        orders_ledger_present: DashboardValue::unknown(),
        orders_duplicate_attempt_detected: DashboardValue::unknown(),
        orders_no_retry: DashboardValue::unknown(),
        orders_diagnostics: DashboardValue::unknown(),
        orders_lineage: DashboardValue::unknown(),
        orders_exchange_truth: DashboardValue::unknown(),
        orders_adapter_runtime_integrated: DashboardValue::unknown(),
        orders_values_are_exchange_truth: DashboardValue::unknown(),
        fills_freshness_status: DashboardValue::unknown(),
        fills_source_type: DashboardValue::unknown(),
        fills_source_ref: DashboardValue::unknown(),
        fills_redaction_state: DashboardValue::unknown(),
        fills_fill_id: DashboardValue::unknown(),
        fills_execution_id: DashboardValue::unknown(),
        fills_order_id: DashboardValue::unknown(),
        fills_client_order_id: DashboardValue::unknown(),
        fills_fill_status: DashboardValue::unknown(),
        fills_order_linkage_status: DashboardValue::unknown(),
        fills_reconciliation_status: DashboardValue::unknown(),
        fills_quantity: DashboardValue::unknown(),
        fills_cumulative_quantity: DashboardValue::unknown(),
        fills_remaining_quantity: DashboardValue::unknown(),
        fills_quantity_precision: DashboardValue::unknown(),
        fills_price: DashboardValue::unknown(),
        fills_price_precision: DashboardValue::unknown(),
        fills_precision_status: DashboardValue::unknown(),
        fills_duplicate_fill_detected: DashboardValue::unknown(),
        fills_partial_fill_detected: DashboardValue::unknown(),
        fills_risk_projection_input: DashboardValue::unknown(),
        fills_diagnostics: DashboardValue::unknown(),
        fills_lineage: DashboardValue::unknown(),
        fills_exchange_truth: DashboardValue::unknown(),
        fills_adapter_runtime_integrated: DashboardValue::unknown(),
        fills_values_are_exchange_truth: DashboardValue::unknown(),
        risk_freshness_status: DashboardValue::unknown(),
        risk_source_type: DashboardValue::unknown(),
        risk_source_ref: DashboardValue::unknown(),
        risk_redaction_state: DashboardValue::unknown(),
        risk_state: DashboardValue::unknown(),
        risk_priority_state: DashboardValue::unknown(),
        risk_visible: DashboardValue::unknown(),
        risk_manual_review_required: DashboardValue::unknown(),
        risk_halted: DashboardValue::unknown(),
        risk_mismatch_detected: DashboardValue::unknown(),
        risk_freshness_rollup: DashboardValue::unknown(),
        risk_critical_evidence_complete: DashboardValue::unknown(),
        risk_alert_severity: DashboardValue::unknown(),
        risk_alert_missing_evidence: DashboardValue::unknown(),
        risk_alert_stale_source: DashboardValue::unknown(),
        risk_alert_schema_mismatch: DashboardValue::unknown(),
        risk_alert_redaction_breach: DashboardValue::unknown(),
        risk_alert_forbidden_control_request: DashboardValue::unknown(),
        risk_alert_summary: DashboardValue::unknown(),
        risk_diagnostics: DashboardValue::unknown(),
        risk_lineage: DashboardValue::unknown(),
        audit_freshness_status: DashboardValue::unknown(),
        audit_source_type: DashboardValue::unknown(),
        audit_source_ref: DashboardValue::unknown(),
        audit_redaction_state: DashboardValue::unknown(),
        audit_state: DashboardValue::unknown(),
        audit_closed: DashboardValue::unknown(),
        audit_required_evidence_complete: DashboardValue::unknown(),
        audit_required_components_complete: DashboardValue::unknown(),
        audit_missing_evidence: DashboardValue::unknown(),
        audit_release_provenance: DashboardValue::unknown(),
        audit_artifact_digest: DashboardValue::unknown(),
        audit_artifact_sha: DashboardValue::unknown(),
        audit_provenance_mismatch: DashboardValue::unknown(),
        audit_diagnostics: DashboardValue::unknown(),
        audit_lineage: DashboardValue::unknown(),
        operation_entry_freshness_status: DashboardValue::unknown(),
        operation_entry_source_type: DashboardValue::unknown(),
        operation_entry_source_ref: DashboardValue::unknown(),
        operation_entry_redaction_state: DashboardValue::unknown(),
        operation_intent_preview: DashboardValue::unknown(),
        operation_owner_approval_ref: DashboardValue::unknown(),
        operation_risk_decision_ref: DashboardValue::unknown(),
        operation_audit_evidence_ref: DashboardValue::unknown(),
        operation_entry_disabled: DashboardValue::available("true".to_string()),
        operation_entry_blocked_reason: DashboardValue::available(
            "missing_operation_entry_contract".to_string(),
        ),
        operation_missing_owner_approval: DashboardValue::available("true".to_string()),
        operation_missing_risk_gate: DashboardValue::available("true".to_string()),
        operation_missing_audit_gate: DashboardValue::available("true".to_string()),
        operation_stale_read_model: DashboardValue::available("true".to_string()),
        operation_provenance_mismatch: DashboardValue::unknown(),
        operation_gates_complete: DashboardValue::available("false".to_string()),
        operation_ungated_attempted: DashboardValue::available("false".to_string()),
        operation_attempt_status: DashboardValue::available(
            "fail_closed_without_contract".to_string(),
        ),
        operation_ungated_attempt_fail_closed: DashboardValue::available("true".to_string()),
        v24_order_control_preview_status: DashboardValue::unknown(),
        v24_order_intent_status: DashboardValue::unknown(),
        v24_execution_policy_status: DashboardValue::unknown(),
        v24_rate_limit_status: DashboardValue::unknown(),
        v24_slicing_status: DashboardValue::unknown(),
        v24_cancel_replace_amend_status: DashboardValue::unknown(),
        v24_retry_policy_status: DashboardValue::unknown(),
        v24_readback_audit_status: DashboardValue::unknown(),
        v24_blocked_reasons: DashboardValue::unknown(),
        v24_scope_key: DashboardValue::unknown(),
        v24_source_provenance: DashboardValue::unknown(),
        v24_redaction_state: DashboardValue::unknown(),
        v24_order_intent_ref: DashboardValue::unknown(),
        v24_policy_ref: DashboardValue::unknown(),
        v24_rate_limit_ref: DashboardValue::unknown(),
        v24_slicing_ref: DashboardValue::unknown(),
        v24_cancel_replace_amend_ref: DashboardValue::unknown(),
        v24_retry_policy_ref: DashboardValue::unknown(),
        v24_readback_ref: DashboardValue::unknown(),
        v24_audit_ref: DashboardValue::unknown(),
        v24_provenance_ref: DashboardValue::unknown(),
        v24_dashboard_redacted_ref: DashboardValue::unknown(),
        v24_preview_evidence_present: DashboardValue::unknown(),
        v24_missing_preview_evidence: DashboardValue::unknown(),
        v24_forbidden_control_detected: DashboardValue::unknown(),
        v24_render_smoke_case: DashboardValue::unknown(),
        v25_dashboard_surface_status: DashboardValue::unknown(),
        v25_diagnostics_gate_status: DashboardValue::unknown(),
        v25_slo_status: DashboardValue::unknown(),
        v25_freshness_threshold_status: DashboardValue::unknown(),
        v25_staleness_reasons: DashboardValue::unknown(),
        v25_diagnostic_severity: DashboardValue::unknown(),
        v25_source_truth_status: DashboardValue::unknown(),
        v25_release_provenance_status: DashboardValue::unknown(),
        v25_no_remediation_status: DashboardValue::unknown(),
        v25_monitoring_status: DashboardValue::unknown(),
        v25_monitoring_runtime_health: DashboardValue::unknown(),
        v25_monitoring_effective_status: DashboardValue::unknown(),
        v25_monitoring_freshness_status: DashboardValue::unknown(),
        v25_monitoring_source_ref: DashboardValue::unknown(),
        v25_monitoring_redaction_state: DashboardValue::unknown(),
        v25_alert_status: DashboardValue::unknown(),
        v25_alert_highest_severity: DashboardValue::unknown(),
        v25_alert_route_status: DashboardValue::unknown(),
        v25_alert_dedupe_key: DashboardValue::unknown(),
        v25_incident_status: DashboardValue::unknown(),
        v25_incident_current_state: DashboardValue::unknown(),
        v25_incident_ack_status: DashboardValue::unknown(),
        v25_incident_owner: DashboardValue::unknown(),
        v25_runbook_status: DashboardValue::unknown(),
        v25_runbook_decision_type: DashboardValue::unknown(),
        v25_runbook_decision_status: DashboardValue::unknown(),
        v25_runbook_evidence_ref: DashboardValue::unknown(),
        v25_dr_preview_status: DashboardValue::unknown(),
        v25_dr_scenario: DashboardValue::unknown(),
        v25_dr_recovery_point: DashboardValue::unknown(),
        v25_dr_operator_approval_status: DashboardValue::unknown(),
        v25_dr_snapshot_lineage: DashboardValue::unknown(),
        v25_surface_blocking_reasons: DashboardValue::unknown(),
        v26_dashboard_admin_surface_status: DashboardValue::unknown(),
        v26_permission_boundary_status: DashboardValue::unknown(),
        v26_permission_roles_checked: DashboardValue::unknown(),
        v26_operation_audit_status: DashboardValue::unknown(),
        v26_operation_audit_lineage: DashboardValue::unknown(),
        v26_deployment_provenance_status: DashboardValue::unknown(),
        v26_deployment_environment: DashboardValue::unknown(),
        v26_upgrade_rollback_status: DashboardValue::unknown(),
        v26_upgrade_rollback_preview: DashboardValue::unknown(),
        v26_stability_status: DashboardValue::unknown(),
        v26_stability_degradation_reason: DashboardValue::unknown(),
        v26_admin_surface_blocking_reasons: DashboardValue::unknown(),
        orders_summary: DashboardValue::unknown(),
        fills_summary: DashboardValue::unknown(),
        risk_summary: DashboardValue::unknown(),
        lifecycle_summary: DashboardValue::unknown(),
        missing_components: DashboardValue::unknown(),
        blocking_reasons: DashboardValue::unknown(),
        component_diagnostics: DashboardValue::unknown(),
        new_submit_capability: DashboardValue::available(false),
        dashboard_order_controls_enabled: DashboardValue::available(false),
        dashboard_approval_controls_enabled: DashboardValue::available(false),
        dashboard_cancel_controls_enabled: DashboardValue::available(false),
        dashboard_retry_controls_enabled: DashboardValue::available(false),
        dashboard_fill_controls_enabled: DashboardValue::available(false),
        dashboard_submit_controls_enabled: DashboardValue::available(false),
        dashboard_replace_controls_enabled: DashboardValue::available(false),
        dashboard_amend_controls_enabled: DashboardValue::available(false),
        dashboard_flatten_controls_enabled: DashboardValue::available(false),
        trader_terminal_order_ticket_enabled: DashboardValue::available(false),
        trader_terminal_live_trading_claim: DashboardValue::available(false),
        production_order_submission_allowed: DashboardValue::available(false),
        production_order_mutation_allowed: DashboardValue::available(false),
        retry_replace_amend_flatten_allowed: DashboardValue::available(false),
        order_permission_control_allowed: DashboardValue::available(false),
        retry_order_allowed: DashboardValue::available(false),
        automatic_cancel_allowed: DashboardValue::available(false),
        automatic_order_remediation_allowed: DashboardValue::available(false),
        funds_transfer_allowed: DashboardValue::available(false),
        account_configuration_mutation_allowed: DashboardValue::available(false),
        auto_flatten_position_allowed: DashboardValue::available(false),
        automatic_position_repair_allowed: DashboardValue::available(false),
        execution_algorithm_allowed: DashboardValue::available(false),
        automatic_fill_repair_allowed: DashboardValue::available(false),
        automatic_reconciliation_repair_allowed: DashboardValue::available(false),
        dashboard_risk_controls_enabled: DashboardValue::available(false),
        automatic_risk_action_allowed: DashboardValue::available(false),
        automatic_risk_repair_allowed: DashboardValue::available(false),
        automatic_alert_action_allowed: DashboardValue::available(false),
        automatic_audit_action_allowed: DashboardValue::available(false),
        automatic_provenance_repair_allowed: DashboardValue::available(false),
        manual_operation_entry_enabled: DashboardValue::available(false),
        manual_operation_submit_allowed: DashboardValue::available(false),
        manual_operation_cancel_allowed: DashboardValue::available(false),
        manual_operation_retry_allowed: DashboardValue::available(false),
        manual_operation_replace_allowed: DashboardValue::available(false),
        manual_operation_amend_allowed: DashboardValue::available(false),
        manual_operation_flatten_allowed: DashboardValue::available(false),
        automatic_operation_action_allowed: DashboardValue::available(false),
        product_grade_trading_terminal_claim: DashboardValue::available(false),
    }
}

fn trader_terminal_read_model_status_from_value(
    record: &SupervisorNodeRecord,
    artifact_path: &FsPath,
    value: &Value,
) -> TraderTerminalReadModelStatus {
    let contract_version = json_string_field(value, "contract_version");
    let schema_version = json_string_field(value, "schema_version");
    let snapshot_kind = json_string_field(value, "snapshot_kind");
    let snapshot_health_status = json_string_field(value, "health_status");
    let freshness_status = nested_json_string_field(value, "freshness", "status");
    let source_type = nested_json_string_field(value, "source_provenance", "source_type");
    let source_ref = nested_json_string_field(value, "source_provenance", "source_ref");
    let redaction_state = nested_json_string_field(value, "redaction", "status");
    let blocking_reasons = read_model_string_array_field(value, "blocking_reasons");
    let mut diagnostics = Vec::new();
    let mut health = match snapshot_health_status.value.as_deref() {
        Some("healthy") => HealthStatus::Healthy,
        Some("degraded") => HealthStatus::Degraded,
        Some("fail_closed") => HealthStatus::Error,
        Some(other) => {
            diagnostics.push(format!("health_status_unexpected:{other}"));
            HealthStatus::Error
        }
        None => {
            diagnostics.push("health_status_missing".to_string());
            HealthStatus::Error
        }
    };

    if contract_version.value.as_deref() != Some(UNIFIED_READ_MODEL_CONTRACT_VERSION) {
        diagnostics.push("contract_version_mismatch".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }
    if schema_version.value.as_deref() != Some(UNIFIED_READ_MODEL_SCHEMA_VERSION) {
        diagnostics.push("schema_version_mismatch".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }
    if !matches!(
        snapshot_kind.value.as_deref(),
        Some("unified_snapshot" | "dashboard_view")
    ) {
        diagnostics.push("snapshot_kind_not_terminal_read_model".to_string());
        health = strongest_health(health, HealthStatus::Degraded);
    }
    match freshness_status.value.as_deref() {
        Some("fresh") => {}
        Some("stale") => {
            diagnostics.push("snapshot_freshness_stale".to_string());
            health = strongest_health(health, HealthStatus::Stale);
        }
        Some(other) => {
            diagnostics.push(format!("snapshot_freshness_{other}"));
            health = strongest_health(health, HealthStatus::Degraded);
        }
        None => {
            diagnostics.push("snapshot_freshness_missing".to_string());
            health = strongest_health(health, HealthStatus::Error);
        }
    }
    if source_type.availability != DashboardAvailability::Available
        || source_ref.availability != DashboardAvailability::Available
    {
        diagnostics.push("source_provenance_missing".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }
    if redaction_state
        .value
        .as_deref()
        .is_none_or(|status| matches!(status, "fail_closed" | "unavailable" | "unknown"))
    {
        diagnostics.push("redaction_state_not_ready".to_string());
        health = strongest_health(health, HealthStatus::Degraded);
    }

    let components = value.get("components").and_then(Value::as_object);
    let mut missing_components = Vec::new();
    let mut component_diagnostics = Vec::new();
    let component_statuses = TRADER_TERMINAL_READ_MODEL_REQUIRED_COMPONENTS
        .iter()
        .map(|component| {
            let status = components
                .and_then(|items| items.get(*component))
                .and_then(|component_value| component_value.get("component_status"))
                .and_then(Value::as_str)
                .map(str::to_string)
                .map_or_else(DashboardValue::unknown, DashboardValue::available);
            if status.availability != DashboardAvailability::Available {
                missing_components.push((*component).to_string());
                component_diagnostics.push(format!("{component}:missing"));
                health = strongest_health(health, HealthStatus::Error);
                return (*component, status);
            }
            match status.value.as_deref() {
                Some("healthy") => {}
                Some("degraded") => {
                    component_diagnostics.push(format!("{component}:degraded"));
                    health = strongest_health(health, HealthStatus::Degraded);
                }
                Some("unavailable") => {
                    component_diagnostics.push(format!("{component}:unavailable"));
                    health = strongest_health(health, HealthStatus::Degraded);
                }
                Some("fail_closed") => {
                    component_diagnostics.push(format!("{component}:fail_closed"));
                    health = strongest_health(health, HealthStatus::Error);
                }
                Some(other) => {
                    component_diagnostics.push(format!("{component}:status_unexpected:{other}"));
                    health = strongest_health(health, HealthStatus::Error);
                }
                None => {}
            }
            let component_freshness = components
                .and_then(|items| items.get(*component))
                .and_then(|component_value| component_value.get("freshness"))
                .and_then(|freshness| freshness.get("status"))
                .and_then(Value::as_str);
            match component_freshness {
                Some("fresh") => {}
                Some("stale") => {
                    component_diagnostics.push(format!("{component}:freshness_stale"));
                    health = strongest_health(health, HealthStatus::Stale);
                }
                Some(other) => {
                    component_diagnostics.push(format!("{component}:freshness_{other}"));
                    health = strongest_health(health, HealthStatus::Degraded);
                }
                None => {
                    component_diagnostics.push(format!("{component}:freshness_missing"));
                    health = strongest_health(health, HealthStatus::Error);
                }
            }
            (*component, status)
        })
        .collect::<BTreeMap<_, _>>();

    for component in TRADER_TERMINAL_READ_MODEL_REQUIRED_COMPONENTS {
        let source_type = read_model_component_source_field(value, component, "source_type");
        let source_ref = read_model_component_source_field(value, component, "source_ref");
        if source_type.availability != DashboardAvailability::Available
            || source_ref.availability != DashboardAvailability::Available
        {
            component_diagnostics.push(format!("{component}:source_provenance_missing"));
            health = strongest_health(health, HealthStatus::Error);
        }

        let component_redaction = read_model_component_redaction_status(value, component);
        if component_redaction
            .value
            .as_deref()
            .is_none_or(|status| matches!(status, "fail_closed" | "unavailable" | "unknown"))
        {
            component_diagnostics.push(format!("{component}:redaction_state_not_ready"));
            health = strongest_health(health, HealthStatus::Degraded);
        }
    }

    for component in ["orders", "fills", "risk", "lifecycle_status"] {
        for diagnostic in read_model_component_diagnostics(value, component) {
            component_diagnostics.push(format!("{component}:diagnostic:{diagnostic}"));
        }

        for (field, claim) in [
            (
                "source_exchange_truth",
                read_model_component_source_field(value, component, "exchange_truth"),
            ),
            (
                "source_adapter_runtime_integrated",
                read_model_component_source_field(value, component, "adapter_runtime_integrated"),
            ),
            (
                "values_are_exchange_truth",
                read_model_component_data_scalar(value, component, "values_are_exchange_truth"),
            ),
        ] {
            if claim.value.as_deref() == Some("true") {
                component_diagnostics.push(format!(
                    "{component}:runtime_exchange_truth_claimed:{field}"
                ));
                health = strongest_health(health, HealthStatus::Error);
            }
        }
    }

    let account_id = read_model_component_data_scalar(value, "account", "account_id");
    let positions_account_id = read_model_component_data_scalar(value, "positions", "account_id");
    if let (Some(account_id), Some(positions_account_id)) = (
        account_id.value.as_deref(),
        positions_account_id.value.as_deref(),
    ) && account_id != positions_account_id
    {
        component_diagnostics.push("account_position_mismatch".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }

    if blocking_reasons.availability == DashboardAvailability::Available {
        diagnostics.push("blocking_reasons_present".to_string());
        health = strongest_health(health, HealthStatus::Degraded);
    }

    let risk_priority_state = read_model_risk_priority_state(value);
    if risk_priority_state.value.as_deref() == Some("mismatch") {
        component_diagnostics.push("risk:mismatch_detected".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }
    let risk_alert_missing_evidence =
        read_model_component_data_nested_scalar(value, "risk", "alerts", "missing_evidence");
    let risk_alert_stale_source =
        read_model_component_data_nested_scalar(value, "risk", "alerts", "stale_source");
    let risk_alert_schema_mismatch =
        read_model_component_data_nested_scalar(value, "risk", "alerts", "schema_mismatch");
    let risk_alert_redaction_breach =
        read_model_component_data_nested_scalar(value, "risk", "alerts", "redaction_breach");
    let risk_alert_forbidden_control_request = read_model_component_data_nested_scalar(
        value,
        "risk",
        "alerts",
        "forbidden_control_request",
    );
    for (field, alert_health, alert) in [
        (
            "missing_evidence",
            HealthStatus::Error,
            &risk_alert_missing_evidence,
        ),
        (
            "stale_source",
            HealthStatus::Stale,
            &risk_alert_stale_source,
        ),
        (
            "schema_mismatch",
            HealthStatus::Error,
            &risk_alert_schema_mismatch,
        ),
        (
            "redaction_breach",
            HealthStatus::Error,
            &risk_alert_redaction_breach,
        ),
        (
            "forbidden_control_request",
            HealthStatus::Error,
            &risk_alert_forbidden_control_request,
        ),
    ] {
        if alert.value.as_deref() == Some("true") {
            component_diagnostics.push(format!("risk:alert:{field}"));
            health = strongest_health(health, alert_health);
        }
    }

    let audit_closed = read_model_component_data_scalar(value, "lifecycle_status", "audit_closed");
    let audit_required_evidence_complete =
        read_model_component_data_scalar(value, "lifecycle_status", "required_evidence_complete");
    let audit_required_components_complete =
        read_model_component_data_scalar(value, "lifecycle_status", "required_components_complete");
    let audit_missing_evidence =
        read_model_component_data_array_field(value, "lifecycle_status", "missing_evidence");
    let audit_release_provenance =
        read_model_component_data_scalar(value, "lifecycle_status", "release_provenance");
    let audit_artifact_digest =
        read_model_component_data_scalar(value, "lifecycle_status", "artifact_digest");
    let audit_artifact_sha =
        read_model_component_data_scalar(value, "lifecycle_status", "artifact_sha");
    let audit_provenance_mismatch =
        read_model_component_data_scalar(value, "lifecycle_status", "provenance_mismatch");
    if audit_closed.value.as_deref() == Some("true")
        && (audit_required_evidence_complete.value.as_deref() != Some("true")
            || audit_required_components_complete.value.as_deref() != Some("true")
            || audit_missing_evidence.availability == DashboardAvailability::Available)
    {
        component_diagnostics
            .push("lifecycle_status:audit_closed_without_complete_evidence".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }
    if audit_closed.value.as_deref() == Some("true")
        && (audit_release_provenance.availability != DashboardAvailability::Available
            || audit_artifact_digest.availability != DashboardAvailability::Available
            || audit_artifact_sha.availability != DashboardAvailability::Available)
    {
        component_diagnostics.push("lifecycle_status:audit_closed_missing_provenance".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }
    if audit_provenance_mismatch.value.as_deref() == Some("true") {
        component_diagnostics.push("lifecycle_status:provenance_mismatch".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }

    let operation_intent_preview =
        read_model_component_data_scalar(value, "operation_entry", "intent_preview");
    let operation_owner_approval_ref =
        read_model_component_data_scalar(value, "operation_entry", "owner_approval_ref");
    let operation_risk_decision_ref =
        read_model_component_data_scalar(value, "operation_entry", "risk_decision_ref");
    let operation_audit_evidence_ref =
        read_model_component_data_scalar(value, "operation_entry", "audit_evidence_ref");
    let operation_missing_owner_approval_flag = read_model_component_data_nested_bool(
        value,
        "operation_entry",
        "blocked_states",
        "missing_owner_approval",
    )
    .unwrap_or(operation_owner_approval_ref.availability != DashboardAvailability::Available);
    let operation_missing_risk_gate_flag = read_model_component_data_nested_bool(
        value,
        "operation_entry",
        "blocked_states",
        "missing_risk_gate",
    )
    .unwrap_or(operation_risk_decision_ref.availability != DashboardAvailability::Available);
    let operation_missing_audit_gate_flag = read_model_component_data_nested_bool(
        value,
        "operation_entry",
        "blocked_states",
        "missing_audit_gate",
    )
    .unwrap_or(operation_audit_evidence_ref.availability != DashboardAvailability::Available);
    let operation_stale_read_model_flag = read_model_component_data_nested_bool(
        value,
        "operation_entry",
        "blocked_states",
        "stale_read_model",
    )
    .unwrap_or(freshness_status.value.as_deref() == Some("stale"));
    let operation_provenance_mismatch_flag = read_model_component_data_nested_bool(
        value,
        "operation_entry",
        "blocked_states",
        "provenance_mismatch",
    )
    .unwrap_or(audit_provenance_mismatch.value.as_deref() == Some("true"));
    let operation_entry_disabled_flag =
        read_model_component_data_bool(value, "operation_entry", "disabled").unwrap_or(true);
    let operation_gates_complete_flag =
        read_model_component_data_bool(value, "operation_entry", "gates_complete").unwrap_or(false);
    let operation_ungated_attempted_flag =
        read_model_component_data_bool(value, "operation_entry", "ungated_operation_attempted")
            .unwrap_or(false);
    let operation_ungated_attempt_fail_closed_flag = read_model_component_data_bool(
        value,
        "operation_entry",
        "ungated_operation_attempt_fail_closed",
    )
    .unwrap_or(true);
    if operation_ungated_attempted_flag {
        component_diagnostics.push("operation_entry:ungated_operation_attempted".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }
    if operation_provenance_mismatch_flag {
        component_diagnostics.push("operation_entry:provenance_mismatch".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }
    if operation_ungated_attempted_flag && !operation_ungated_attempt_fail_closed_flag {
        component_diagnostics.push("operation_entry:ungated_attempt_not_fail_closed".to_string());
        health = strongest_health(health, HealthStatus::Error);
    }

    let operation_entry_blocked_reason = read_model_operation_entry_blocked_reason(
        operation_missing_owner_approval_flag,
        operation_missing_risk_gate_flag,
        operation_missing_audit_gate_flag,
        operation_stale_read_model_flag,
        operation_provenance_mismatch_flag,
    );
    let operation_entry_state = ReadModelOperationEntryState {
        ungated_operation_attempted: operation_ungated_attempted_flag,
        stale_read_model: operation_stale_read_model_flag,
        provenance_mismatch: operation_provenance_mismatch_flag,
        missing_owner_approval: operation_missing_owner_approval_flag,
        missing_risk_gate: operation_missing_risk_gate_flag,
        missing_audit_gate: operation_missing_audit_gate_flag,
        gates_complete: operation_gates_complete_flag,
        disabled: operation_entry_disabled_flag,
    };
    let operation_entry_status = read_model_operation_entry_status(operation_entry_state);

    let v24_preview_component_present = components
        .and_then(|items| items.get(V24_ORDER_CONTROL_PREVIEW_COMPONENT))
        .is_some();
    let v24_component_freshness =
        read_model_component_freshness_status(value, V24_ORDER_CONTROL_PREVIEW_COMPONENT);
    let v24_component_source_type = read_model_component_source_field(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "source_type",
    );
    let v24_component_source_ref =
        read_model_component_source_field(value, V24_ORDER_CONTROL_PREVIEW_COMPONENT, "source_ref");
    let v24_component_redaction =
        read_model_component_redaction_status(value, V24_ORDER_CONTROL_PREVIEW_COMPONENT);
    let v24_order_control_preview_status = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "preview_status",
    );
    let v24_order_intent_status = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "order_intent_status",
    );
    let v24_execution_policy_status = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "execution_policy_status",
    );
    let v24_rate_limit_status = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "rate_limit_status",
    );
    let v24_slicing_status = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "slicing_status",
    );
    let v24_cancel_replace_amend_status = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "cancel_replace_amend_status",
    );
    let v24_retry_policy_status = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "retry_policy_status",
    );
    let v24_readback_audit_status = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "readback_audit_status",
    );
    let v24_blocked_reasons = read_model_component_data_array_field(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "blocked_reasons",
    );
    let v24_scope_key =
        read_model_component_data_scalar(value, V24_ORDER_CONTROL_PREVIEW_COMPONENT, "scope_key");
    let v24_source_provenance = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "source_provenance",
    );
    let v24_redaction_state = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "redaction_state",
    );
    let v24_order_intent_ref = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "order_intent_ref",
    );
    let v24_policy_ref =
        read_model_component_data_scalar(value, V24_ORDER_CONTROL_PREVIEW_COMPONENT, "policy_ref");
    let v24_rate_limit_ref = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "rate_limit_ref",
    );
    let v24_slicing_ref =
        read_model_component_data_scalar(value, V24_ORDER_CONTROL_PREVIEW_COMPONENT, "slicing_ref");
    let v24_cancel_replace_amend_ref = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "cancel_replace_amend_ref",
    );
    let v24_retry_policy_ref = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "retry_policy_ref",
    );
    let v24_readback_ref = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "readback_ref",
    );
    let v24_audit_ref =
        read_model_component_data_scalar(value, V24_ORDER_CONTROL_PREVIEW_COMPONENT, "audit_ref");
    let v24_provenance_ref = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "provenance_ref",
    );
    let v24_dashboard_redacted_ref = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "dashboard_redacted_ref",
    );
    let v24_preview_evidence_present = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "preview_evidence_present",
    );
    let v24_missing_preview_evidence = read_model_component_data_array_field(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "missing_preview_evidence",
    );
    let v24_forbidden_control_detected = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "forbidden_control_detected",
    );
    let v24_render_smoke_case = read_model_component_data_scalar(
        value,
        V24_ORDER_CONTROL_PREVIEW_COMPONENT,
        "render_smoke_case",
    );
    let snapshot_identity_venue = nested_json_string_field(value, "snapshot_identity", "venue");
    if v24_preview_component_present {
        match v24_component_freshness.value.as_deref() {
            Some("fresh") => {}
            Some("stale") => {
                component_diagnostics.push(format!(
                    "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:freshness_stale"
                ));
                health = strongest_health(health, HealthStatus::Stale);
            }
            Some(other) => {
                component_diagnostics.push(format!(
                    "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:freshness_{other}"
                ));
                health = strongest_health(health, HealthStatus::Degraded);
            }
            None => {
                component_diagnostics.push(format!(
                    "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:freshness_missing"
                ));
                health = strongest_health(health, HealthStatus::Error);
            }
        }
        if v24_component_source_type
            .value
            .as_deref()
            .is_none_or(str::is_empty)
            || v24_component_source_ref
                .value
                .as_deref()
                .is_none_or(str::is_empty)
        {
            component_diagnostics.push(format!(
                "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:source_provenance_missing"
            ));
            health = strongest_health(health, HealthStatus::Error);
        }
        if v24_component_redaction.value.as_deref() != Some("redacted") {
            component_diagnostics.push(format!(
                "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:redaction_state_not_ready"
            ));
            health = strongest_health(health, HealthStatus::Error);
        }
        if v24_redaction_state.value.as_deref() != Some("redacted") {
            component_diagnostics.push(format!(
                "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:data_redaction_state_not_ready"
            ));
            health = strongest_health(health, HealthStatus::Error);
        }
        match v24_order_control_preview_status.value.as_deref() {
            Some("ready_preview") => {}
            Some("blocked" | "degraded_unavailable") => {
                component_diagnostics.push(format!(
                    "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:{}",
                    v24_order_control_preview_status
                        .value
                        .as_deref()
                        .unwrap_or("unknown")
                ));
                health = strongest_health(health, HealthStatus::Degraded);
            }
            Some("fail_closed" | "forbidden_control_detected") => {
                component_diagnostics.push(format!(
                    "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:{}",
                    v24_order_control_preview_status
                        .value
                        .as_deref()
                        .unwrap_or("unknown")
                ));
                health = strongest_health(health, HealthStatus::Error);
            }
            Some(other) => {
                component_diagnostics.push(format!(
                    "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:preview_status_unexpected:{other}"
                ));
                health = strongest_health(health, HealthStatus::Error);
            }
            None => {
                component_diagnostics.push(format!(
                    "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:preview_status_missing"
                ));
                health = strongest_health(health, HealthStatus::Degraded);
            }
        }
        if v24_preview_evidence_present.value.as_deref() != Some("true") {
            component_diagnostics.push(format!(
                "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:preview_evidence_missing"
            ));
            health = strongest_health(health, HealthStatus::Degraded);
        }
        if v24_order_control_preview_status.value.as_deref() == Some("ready_preview")
            && v24_missing_preview_evidence.availability == DashboardAvailability::Available
        {
            component_diagnostics.push(format!(
                "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:ready_with_missing_preview_evidence"
            ));
            health = strongest_health(health, HealthStatus::Error);
        }
        if v24_preview_evidence_present.value.as_deref() == Some("true")
            && v24_provenance_ref
                .value
                .as_deref()
                .is_none_or(str::is_empty)
        {
            component_diagnostics.push(format!(
                "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:preview_provenance_missing"
            ));
            health = strongest_health(health, HealthStatus::Error);
        }
        if let (Some(scope_key), Some(account_id)) =
            (v24_scope_key.value.as_deref(), account_id.value.as_deref())
            && !scope_key.contains(account_id)
        {
            component_diagnostics.push(format!(
                "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:scope_mismatch"
            ));
            health = strongest_health(health, HealthStatus::Error);
        }
        if let (Some(scope_key), Some(venue)) = (
            v24_scope_key.value.as_deref(),
            snapshot_identity_venue.value.as_deref(),
        ) {
            let scope_key = scope_key.to_ascii_lowercase();
            let venue = venue.to_ascii_lowercase();
            if !scope_key.contains(&venue) {
                component_diagnostics.push(format!(
                    "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:scope_mismatch"
                ));
                health = strongest_health(health, HealthStatus::Error);
            }
        }
        if v24_forbidden_control_detected.value.as_deref() == Some("true") {
            component_diagnostics.push(format!(
                "{V24_ORDER_CONTROL_PREVIEW_COMPONENT}:forbidden_control_detected"
            ));
            health = strongest_health(health, HealthStatus::Error);
        }
    }

    let mut v25_surface_diagnostics = Vec::new();
    for component in V25_DASHBOARD_SURFACE_COMPONENTS {
        validate_v25_dashboard_surface_component(
            value,
            component,
            &mut component_diagnostics,
            &mut v25_surface_diagnostics,
            &mut health,
        );
    }
    let v25_dashboard_surface_status_value =
        v25_dashboard_surface_status(&v25_surface_diagnostics, health);
    let v25_diagnostics_gate_status = v25_diagnostics_gate_status(&v25_surface_diagnostics, health);
    let v25_slo_status = v25_slo_status(&v25_surface_diagnostics);
    let v25_freshness_threshold_status = v25_freshness_threshold_status(&v25_surface_diagnostics);
    let v25_staleness_reasons = v25_staleness_reasons(&v25_surface_diagnostics);
    let v25_diagnostic_severity = v25_diagnostic_severity(&v25_surface_diagnostics, health);
    let v25_source_truth_status = v25_source_truth_status(&v25_surface_diagnostics);
    let v25_release_provenance_status = v25_release_provenance_status(&v25_surface_diagnostics);
    let v25_no_remediation_status = v25_no_remediation_status(&v25_surface_diagnostics);
    let v25_surface_blocking_reasons = diagnostic_value(&v25_surface_diagnostics);

    let v25_monitoring_status = component_statuses
        .get(V25_MONITORING_OBSERVABILITY_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v25_monitoring_runtime_health = read_model_component_data_scalar(
        value,
        V25_MONITORING_OBSERVABILITY_COMPONENT,
        "runtime_health_status",
    );
    let v25_monitoring_effective_status = read_model_component_data_scalar(
        value,
        V25_MONITORING_OBSERVABILITY_COMPONENT,
        "effective_monitoring_status",
    );
    let v25_monitoring_freshness_status =
        read_model_component_freshness_status(value, V25_MONITORING_OBSERVABILITY_COMPONENT);
    let v25_monitoring_source_ref = read_model_component_source_field(
        value,
        V25_MONITORING_OBSERVABILITY_COMPONENT,
        "source_ref",
    );
    let v25_monitoring_redaction_state =
        read_model_component_redaction_status(value, V25_MONITORING_OBSERVABILITY_COMPONENT);
    let v25_alert_status = component_statuses
        .get(V25_ALERT_TAXONOMY_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v25_alert_highest_severity =
        read_model_component_data_scalar(value, V25_ALERT_TAXONOMY_COMPONENT, "highest_severity");
    let v25_alert_route_status =
        read_model_component_data_scalar(value, V25_ALERT_TAXONOMY_COMPONENT, "route_status");
    let v25_alert_dedupe_key =
        read_model_component_data_scalar(value, V25_ALERT_TAXONOMY_COMPONENT, "dedupe_key");
    let v25_incident_status = component_statuses
        .get(V25_INCIDENT_LIFECYCLE_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v25_incident_current_state =
        read_model_component_data_scalar(value, V25_INCIDENT_LIFECYCLE_COMPONENT, "current_state");
    let v25_incident_ack_status =
        read_model_component_data_scalar(value, V25_INCIDENT_LIFECYCLE_COMPONENT, "ack_status");
    let v25_incident_owner =
        read_model_component_data_scalar(value, V25_INCIDENT_LIFECYCLE_COMPONENT, "owner");
    let v25_runbook_status = component_statuses
        .get(V25_RUNBOOK_AUDIT_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v25_runbook_decision_type =
        read_model_component_data_scalar(value, V25_RUNBOOK_AUDIT_COMPONENT, "decision_type");
    let v25_runbook_decision_status =
        read_model_component_data_scalar(value, V25_RUNBOOK_AUDIT_COMPONENT, "decision_status");
    let v25_runbook_evidence_ref =
        read_model_component_data_scalar(value, V25_RUNBOOK_AUDIT_COMPONENT, "evidence_ref");
    let v25_dr_preview_status = component_statuses
        .get(V25_DR_PREVIEW_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v25_dr_scenario =
        read_model_component_data_scalar(value, V25_DR_PREVIEW_COMPONENT, "scenario");
    let v25_dr_recovery_point =
        read_model_component_data_scalar(value, V25_DR_PREVIEW_COMPONENT, "recovery_point");
    let v25_dr_operator_approval_status = read_model_component_data_scalar(
        value,
        V25_DR_PREVIEW_COMPONENT,
        "operator_approval_status",
    );
    let v25_dr_snapshot_lineage =
        read_model_component_data_scalar(value, V25_DR_PREVIEW_COMPONENT, "snapshot_lineage");

    let mut v26_admin_surface_diagnostics = Vec::new();
    for component in V26_DASHBOARD_ADMIN_COMPONENTS {
        validate_v25_dashboard_surface_component(
            value,
            component,
            &mut component_diagnostics,
            &mut v26_admin_surface_diagnostics,
            &mut health,
        );
    }
    let v26_dashboard_admin_surface_status_value =
        v25_dashboard_surface_status(&v26_admin_surface_diagnostics, health);
    let v26_admin_surface_blocking_reasons = diagnostic_value(&v26_admin_surface_diagnostics);
    let v26_permission_boundary_status = component_statuses
        .get(V26_PERMISSION_BOUNDARY_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v26_permission_roles_checked =
        read_model_component_data_scalar(value, V26_PERMISSION_BOUNDARY_COMPONENT, "roles_checked");
    let v26_operation_audit_status = component_statuses
        .get(V26_OPERATION_AUDIT_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v26_operation_audit_lineage =
        read_model_component_data_scalar(value, V26_OPERATION_AUDIT_COMPONENT, "audit_lineage");
    let v26_deployment_provenance_status = component_statuses
        .get(V26_DEPLOYMENT_PROVENANCE_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v26_deployment_environment =
        read_model_component_data_scalar(value, V26_DEPLOYMENT_PROVENANCE_COMPONENT, "environment");
    let v26_upgrade_rollback_status = component_statuses
        .get(V26_UPGRADE_ROLLBACK_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v26_upgrade_rollback_preview =
        read_model_component_data_scalar(value, V26_UPGRADE_ROLLBACK_COMPONENT, "preview_status");
    let v26_stability_status = component_statuses
        .get(V26_STABILITY_SLO_COMPONENT)
        .cloned()
        .unwrap_or_else(DashboardValue::unknown);
    let v26_stability_degradation_reason =
        read_model_component_data_scalar(value, V26_STABILITY_SLO_COMPONENT, "degradation_reason");

    let boundary = value.get("capability_boundary").unwrap_or(&Value::Null);
    let new_submit_capability = required_read_model_boundary_bool(
        boundary,
        "new_submit_capability",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_order_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_order_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_approval_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_approval_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_cancel_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_cancel_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_retry_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_retry_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_fill_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_fill_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_submit_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_submit_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_replace_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_replace_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_amend_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_amend_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_flatten_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_flatten_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let trader_terminal_order_ticket_enabled = required_read_model_boundary_bool(
        boundary,
        "trader_terminal_order_ticket_enabled",
        &mut diagnostics,
        &mut health,
    );
    let trader_terminal_live_trading_claim = required_read_model_boundary_bool(
        boundary,
        "trader_terminal_live_trading_claim",
        &mut diagnostics,
        &mut health,
    );
    let production_order_submission_allowed = required_read_model_boundary_bool(
        boundary,
        "production_order_submission_allowed",
        &mut diagnostics,
        &mut health,
    );
    let production_order_mutation_allowed = required_read_model_boundary_bool(
        boundary,
        "production_order_mutation_allowed",
        &mut diagnostics,
        &mut health,
    );
    let retry_replace_amend_flatten_allowed = required_read_model_boundary_bool(
        boundary,
        "retry_replace_amend_flatten_allowed",
        &mut diagnostics,
        &mut health,
    );
    let order_permission_control_allowed = required_read_model_boundary_bool(
        boundary,
        "order_permission_control_allowed",
        &mut diagnostics,
        &mut health,
    );
    let retry_order_allowed = required_read_model_boundary_bool(
        boundary,
        "retry_order_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_cancel_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_cancel_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_order_remediation_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_order_remediation_allowed",
        &mut diagnostics,
        &mut health,
    );
    let funds_transfer_allowed = required_read_model_boundary_bool(
        boundary,
        "funds_transfer_allowed",
        &mut diagnostics,
        &mut health,
    );
    let account_configuration_mutation_allowed = required_read_model_boundary_bool(
        boundary,
        "account_configuration_mutation_allowed",
        &mut diagnostics,
        &mut health,
    );
    let auto_flatten_position_allowed = required_read_model_boundary_bool(
        boundary,
        "auto_flatten_position_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_position_repair_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_position_repair_allowed",
        &mut diagnostics,
        &mut health,
    );
    let execution_algorithm_allowed = required_read_model_boundary_bool(
        boundary,
        "execution_algorithm_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_fill_repair_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_fill_repair_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_reconciliation_repair_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_reconciliation_repair_allowed",
        &mut diagnostics,
        &mut health,
    );
    let dashboard_risk_controls_enabled = required_read_model_boundary_bool(
        boundary,
        "dashboard_risk_controls_enabled",
        &mut diagnostics,
        &mut health,
    );
    let automatic_risk_action_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_risk_action_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_risk_repair_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_risk_repair_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_alert_action_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_alert_action_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_audit_action_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_audit_action_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_provenance_repair_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_provenance_repair_allowed",
        &mut diagnostics,
        &mut health,
    );
    let manual_operation_entry_enabled = required_read_model_boundary_bool(
        boundary,
        "manual_operation_entry_enabled",
        &mut diagnostics,
        &mut health,
    );
    let manual_operation_submit_allowed = required_read_model_boundary_bool(
        boundary,
        "manual_operation_submit_allowed",
        &mut diagnostics,
        &mut health,
    );
    let manual_operation_cancel_allowed = required_read_model_boundary_bool(
        boundary,
        "manual_operation_cancel_allowed",
        &mut diagnostics,
        &mut health,
    );
    let manual_operation_retry_allowed = required_read_model_boundary_bool(
        boundary,
        "manual_operation_retry_allowed",
        &mut diagnostics,
        &mut health,
    );
    let manual_operation_replace_allowed = required_read_model_boundary_bool(
        boundary,
        "manual_operation_replace_allowed",
        &mut diagnostics,
        &mut health,
    );
    let manual_operation_amend_allowed = required_read_model_boundary_bool(
        boundary,
        "manual_operation_amend_allowed",
        &mut diagnostics,
        &mut health,
    );
    let manual_operation_flatten_allowed = required_read_model_boundary_bool(
        boundary,
        "manual_operation_flatten_allowed",
        &mut diagnostics,
        &mut health,
    );
    let automatic_operation_action_allowed = required_read_model_boundary_bool(
        boundary,
        "automatic_operation_action_allowed",
        &mut diagnostics,
        &mut health,
    );
    let product_grade_trading_terminal_claim = required_read_model_boundary_bool(
        boundary,
        "product_grade_trading_terminal_claim",
        &mut diagnostics,
        &mut health,
    );

    let readiness_status = trader_terminal_read_model_readiness(
        health,
        schema_version.value.as_deref(),
        &missing_components,
        &component_diagnostics,
        freshness_status.value.as_deref(),
        blocking_reasons.availability == DashboardAvailability::Available,
    );
    if readiness_status != "ready_readonly_artifact" && health == HealthStatus::Healthy {
        health = HealthStatus::Degraded;
    }
    let diagnostic = if diagnostics.is_empty() && component_diagnostics.is_empty() {
        "canonical_unified_read_model_artifact_ready".to_string()
    } else {
        diagnostics
            .iter()
            .chain(component_diagnostics.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join(",")
    };

    TraderTerminalReadModelStatus {
        node_id: record.node_id.clone(),
        health,
        readiness_status: DashboardValue::available(readiness_status),
        diagnostic: DashboardValue::available(diagnostic),
        artifact_path: DashboardValue::available(artifact_path.display().to_string()),
        contract_version,
        schema_version,
        snapshot_id: json_string_field(value, "snapshot_id"),
        snapshot_kind,
        snapshot_health_status,
        freshness_status,
        source_type,
        source_ref,
        redaction_state,
        account_status: component_statuses
            .get("account")
            .cloned()
            .unwrap_or_else(DashboardValue::unknown),
        positions_status: component_statuses
            .get("positions")
            .cloned()
            .unwrap_or_else(DashboardValue::unknown),
        orders_status: component_statuses
            .get("orders")
            .cloned()
            .unwrap_or_else(DashboardValue::unknown),
        fills_status: component_statuses
            .get("fills")
            .cloned()
            .unwrap_or_else(DashboardValue::unknown),
        risk_status: component_statuses
            .get("risk")
            .cloned()
            .unwrap_or_else(DashboardValue::unknown),
        lifecycle_status: component_statuses
            .get("lifecycle_status")
            .cloned()
            .unwrap_or_else(DashboardValue::unknown),
        operation_entry_status,
        account_summary: read_model_component_data_summary(value, "account"),
        positions_summary: read_model_component_data_summary(value, "positions"),
        account_freshness_status: read_model_component_freshness_status(value, "account"),
        account_source_type: read_model_component_source_field(value, "account", "source_type"),
        account_source_ref: read_model_component_source_field(value, "account", "source_ref"),
        account_redaction_state: read_model_component_redaction_status(value, "account"),
        account_risk_state: read_model_component_data_scalar(value, "account", "risk_state"),
        account_equity: read_model_component_data_scalar(value, "account", "equity"),
        account_available_balance: read_model_component_data_scalar(
            value,
            "account",
            "available_balance",
        ),
        account_balance_entry_count: read_model_component_data_scalar(
            value,
            "account",
            "balance_entry_count",
        ),
        positions_freshness_status: read_model_component_freshness_status(value, "positions"),
        positions_source_type: read_model_component_source_field(value, "positions", "source_type"),
        positions_source_ref: read_model_component_source_field(value, "positions", "source_ref"),
        positions_redaction_state: read_model_component_redaction_status(value, "positions"),
        positions_account_id,
        positions_net_position_side: read_model_component_data_scalar(
            value,
            "positions",
            "net_position_side",
        ),
        positions_quantity: read_model_component_data_scalar(value, "positions", "quantity"),
        positions_notional: read_model_component_data_scalar(value, "positions", "notional"),
        positions_precision: read_model_component_data_scalar(value, "positions", "precision"),
        positions_lineage: read_model_component_lineage_summary(value, "positions"),
        orders_freshness_status: read_model_component_freshness_status(value, "orders"),
        orders_source_type: read_model_component_source_field(value, "orders", "source_type"),
        orders_source_ref: read_model_component_source_field(value, "orders", "source_ref"),
        orders_redaction_state: read_model_component_redaction_status(value, "orders"),
        orders_lifecycle_state: read_model_component_data_scalar(
            value,
            "orders",
            "lifecycle_status",
        ),
        orders_client_order_id: read_model_component_data_scalar(
            value,
            "orders",
            "client_order_id",
        ),
        orders_request_digest: read_model_component_data_scalar(value, "orders", "request_digest"),
        orders_attempt_id: read_model_component_data_scalar(value, "orders", "attempt_id"),
        orders_approval_id: read_model_component_data_scalar(value, "orders", "approval_id"),
        orders_readback_status: read_model_component_data_scalar(
            value,
            "orders",
            "readback_status",
        ),
        orders_audit_state: read_model_component_data_scalar(value, "orders", "audit_state"),
        orders_ledger_present: read_model_component_data_scalar(value, "orders", "ledger_present"),
        orders_duplicate_attempt_detected: read_model_component_data_scalar(
            value,
            "orders",
            "duplicate_attempt_detected",
        ),
        orders_no_retry: read_model_component_data_scalar(value, "orders", "no_retry"),
        orders_diagnostics: read_model_component_diagnostics_summary(value, "orders"),
        orders_lineage: read_model_component_lineage_summary(value, "orders"),
        orders_exchange_truth: read_model_component_source_field(value, "orders", "exchange_truth"),
        orders_adapter_runtime_integrated: read_model_component_source_field(
            value,
            "orders",
            "adapter_runtime_integrated",
        ),
        orders_values_are_exchange_truth: read_model_component_data_scalar(
            value,
            "orders",
            "values_are_exchange_truth",
        ),
        fills_freshness_status: read_model_component_freshness_status(value, "fills"),
        fills_source_type: read_model_component_source_field(value, "fills", "source_type"),
        fills_source_ref: read_model_component_source_field(value, "fills", "source_ref"),
        fills_redaction_state: read_model_component_redaction_status(value, "fills"),
        fills_fill_id: read_model_component_data_scalar(value, "fills", "fill_id"),
        fills_execution_id: read_model_component_data_scalar(value, "fills", "execution_id"),
        fills_order_id: read_model_component_data_scalar(value, "fills", "order_id"),
        fills_client_order_id: read_model_component_data_scalar(value, "fills", "client_order_id"),
        fills_fill_status: read_model_component_data_scalar(value, "fills", "fill_status"),
        fills_order_linkage_status: read_model_component_data_scalar(
            value,
            "fills",
            "order_linkage_status",
        ),
        fills_reconciliation_status: read_model_component_data_scalar(
            value,
            "fills",
            "reconciliation_status",
        ),
        fills_quantity: read_model_component_data_scalar(value, "fills", "quantity"),
        fills_cumulative_quantity: read_model_component_data_scalar(
            value,
            "fills",
            "cumulative_quantity",
        ),
        fills_remaining_quantity: read_model_component_data_scalar(
            value,
            "fills",
            "remaining_quantity",
        ),
        fills_quantity_precision: read_model_component_data_scalar(
            value,
            "fills",
            "quantity_precision",
        ),
        fills_price: read_model_component_data_scalar(value, "fills", "price"),
        fills_price_precision: read_model_component_data_scalar(value, "fills", "price_precision"),
        fills_precision_status: read_model_component_data_scalar(
            value,
            "fills",
            "precision_status",
        ),
        fills_duplicate_fill_detected: read_model_component_data_scalar(
            value,
            "fills",
            "duplicate_fill_detected",
        ),
        fills_partial_fill_detected: read_model_component_data_scalar(
            value,
            "fills",
            "partial_fill_detected",
        ),
        fills_risk_projection_input: read_model_component_data_object_summary(
            value,
            "fills",
            "risk_projection_input",
            &[
                "fill_reconciliation_status",
                "realized_fill_quantity",
                "remaining_order_quantity",
                "risk_state",
                "blocking_reasons",
                "automatic_reconciliation_repair_allowed",
                "execution_algorithm_allowed",
            ],
        ),
        fills_diagnostics: read_model_component_diagnostics_summary(value, "fills"),
        fills_lineage: read_model_component_lineage_summary(value, "fills"),
        fills_exchange_truth: read_model_component_source_field(value, "fills", "exchange_truth"),
        fills_adapter_runtime_integrated: read_model_component_source_field(
            value,
            "fills",
            "adapter_runtime_integrated",
        ),
        fills_values_are_exchange_truth: read_model_component_data_scalar(
            value,
            "fills",
            "values_are_exchange_truth",
        ),
        risk_freshness_status: read_model_component_freshness_status(value, "risk"),
        risk_source_type: read_model_component_source_field(value, "risk", "source_type"),
        risk_source_ref: read_model_component_source_field(value, "risk", "source_ref"),
        risk_redaction_state: read_model_component_redaction_status(value, "risk"),
        risk_state: read_model_component_data_scalar(value, "risk", "risk_state"),
        risk_priority_state,
        risk_visible: read_model_component_data_scalar(value, "risk", "risk_visible"),
        risk_manual_review_required: read_model_component_data_scalar(
            value,
            "risk",
            "manual_review_required",
        ),
        risk_halted: read_model_component_data_scalar(value, "risk", "halted"),
        risk_mismatch_detected: read_model_component_data_scalar(
            value,
            "risk",
            "mismatch_detected",
        ),
        risk_freshness_rollup: read_model_component_data_scalar(value, "risk", "freshness_rollup"),
        risk_critical_evidence_complete: read_model_component_data_scalar(
            value,
            "risk",
            "critical_evidence_complete",
        ),
        risk_alert_severity: read_model_alert_severity(value),
        risk_alert_missing_evidence,
        risk_alert_stale_source,
        risk_alert_schema_mismatch,
        risk_alert_redaction_breach,
        risk_alert_forbidden_control_request,
        risk_alert_summary: read_model_component_data_object_summary(
            value,
            "risk",
            "alerts",
            &[
                "highest_severity",
                "missing_evidence",
                "stale_source",
                "schema_mismatch",
                "redaction_breach",
                "forbidden_control_request",
            ],
        ),
        risk_diagnostics: read_model_component_diagnostics_summary(value, "risk"),
        risk_lineage: read_model_component_lineage_summary(value, "risk"),
        audit_freshness_status: read_model_component_freshness_status(value, "lifecycle_status"),
        audit_source_type: read_model_component_source_field(
            value,
            "lifecycle_status",
            "source_type",
        ),
        audit_source_ref: read_model_component_source_field(
            value,
            "lifecycle_status",
            "source_ref",
        ),
        audit_redaction_state: read_model_component_redaction_status(value, "lifecycle_status"),
        audit_state: read_model_component_data_scalar(value, "lifecycle_status", "audit_state"),
        audit_closed,
        audit_required_evidence_complete,
        audit_required_components_complete,
        audit_missing_evidence,
        audit_release_provenance,
        audit_artifact_digest,
        audit_artifact_sha,
        audit_provenance_mismatch,
        audit_diagnostics: read_model_component_diagnostics_summary(value, "lifecycle_status"),
        audit_lineage: read_model_component_lineage_summary(value, "lifecycle_status"),
        operation_entry_freshness_status: read_model_component_freshness_status(
            value,
            "operation_entry",
        ),
        operation_entry_source_type: read_model_component_source_field(
            value,
            "operation_entry",
            "source_type",
        ),
        operation_entry_source_ref: read_model_component_source_field(
            value,
            "operation_entry",
            "source_ref",
        ),
        operation_entry_redaction_state: read_model_component_redaction_status(
            value,
            "operation_entry",
        ),
        operation_intent_preview,
        operation_owner_approval_ref,
        operation_risk_decision_ref,
        operation_audit_evidence_ref,
        operation_entry_disabled: dashboard_bool_string(operation_entry_disabled_flag),
        operation_entry_blocked_reason,
        operation_missing_owner_approval: dashboard_bool_string(
            operation_missing_owner_approval_flag,
        ),
        operation_missing_risk_gate: dashboard_bool_string(operation_missing_risk_gate_flag),
        operation_missing_audit_gate: dashboard_bool_string(operation_missing_audit_gate_flag),
        operation_stale_read_model: dashboard_bool_string(operation_stale_read_model_flag),
        operation_provenance_mismatch: dashboard_bool_string(operation_provenance_mismatch_flag),
        operation_gates_complete: dashboard_bool_string(operation_gates_complete_flag),
        operation_ungated_attempted: dashboard_bool_string(operation_ungated_attempted_flag),
        operation_attempt_status: read_model_component_data_scalar(
            value,
            "operation_entry",
            "attempt_status",
        ),
        operation_ungated_attempt_fail_closed: dashboard_bool_string(
            operation_ungated_attempt_fail_closed_flag,
        ),
        v24_order_control_preview_status,
        v24_order_intent_status,
        v24_execution_policy_status,
        v24_rate_limit_status,
        v24_slicing_status,
        v24_cancel_replace_amend_status,
        v24_retry_policy_status,
        v24_readback_audit_status,
        v24_blocked_reasons,
        v24_scope_key,
        v24_source_provenance,
        v24_redaction_state,
        v24_order_intent_ref,
        v24_policy_ref,
        v24_rate_limit_ref,
        v24_slicing_ref,
        v24_cancel_replace_amend_ref,
        v24_retry_policy_ref,
        v24_readback_ref,
        v24_audit_ref,
        v24_provenance_ref,
        v24_dashboard_redacted_ref,
        v24_preview_evidence_present,
        v24_missing_preview_evidence,
        v24_forbidden_control_detected,
        v24_render_smoke_case,
        v25_dashboard_surface_status: v25_dashboard_surface_status_value,
        v25_diagnostics_gate_status,
        v25_slo_status,
        v25_freshness_threshold_status,
        v25_staleness_reasons,
        v25_diagnostic_severity,
        v25_source_truth_status,
        v25_release_provenance_status,
        v25_no_remediation_status,
        v25_monitoring_status,
        v25_monitoring_runtime_health,
        v25_monitoring_effective_status,
        v25_monitoring_freshness_status,
        v25_monitoring_source_ref,
        v25_monitoring_redaction_state,
        v25_alert_status,
        v25_alert_highest_severity,
        v25_alert_route_status,
        v25_alert_dedupe_key,
        v25_incident_status,
        v25_incident_current_state,
        v25_incident_ack_status,
        v25_incident_owner,
        v25_runbook_status,
        v25_runbook_decision_type,
        v25_runbook_decision_status,
        v25_runbook_evidence_ref,
        v25_dr_preview_status,
        v25_dr_scenario,
        v25_dr_recovery_point,
        v25_dr_operator_approval_status,
        v25_dr_snapshot_lineage,
        v25_surface_blocking_reasons,
        v26_dashboard_admin_surface_status: v26_dashboard_admin_surface_status_value,
        v26_permission_boundary_status,
        v26_permission_roles_checked,
        v26_operation_audit_status,
        v26_operation_audit_lineage,
        v26_deployment_provenance_status,
        v26_deployment_environment,
        v26_upgrade_rollback_status,
        v26_upgrade_rollback_preview,
        v26_stability_status,
        v26_stability_degradation_reason,
        v26_admin_surface_blocking_reasons,
        orders_summary: read_model_component_data_summary(value, "orders"),
        fills_summary: read_model_component_data_summary(value, "fills"),
        risk_summary: read_model_component_data_summary(value, "risk"),
        lifecycle_summary: read_model_component_data_summary(value, "lifecycle_status"),
        missing_components: diagnostic_value(&missing_components),
        blocking_reasons,
        component_diagnostics: diagnostic_value(&component_diagnostics),
        new_submit_capability,
        dashboard_order_controls_enabled,
        dashboard_approval_controls_enabled,
        dashboard_cancel_controls_enabled,
        dashboard_retry_controls_enabled,
        dashboard_fill_controls_enabled,
        dashboard_submit_controls_enabled,
        dashboard_replace_controls_enabled,
        dashboard_amend_controls_enabled,
        dashboard_flatten_controls_enabled,
        trader_terminal_order_ticket_enabled,
        trader_terminal_live_trading_claim,
        production_order_submission_allowed,
        production_order_mutation_allowed,
        retry_replace_amend_flatten_allowed,
        order_permission_control_allowed,
        retry_order_allowed,
        automatic_cancel_allowed,
        automatic_order_remediation_allowed,
        funds_transfer_allowed,
        account_configuration_mutation_allowed,
        auto_flatten_position_allowed,
        automatic_position_repair_allowed,
        execution_algorithm_allowed,
        automatic_fill_repair_allowed,
        automatic_reconciliation_repair_allowed,
        dashboard_risk_controls_enabled,
        automatic_risk_action_allowed,
        automatic_risk_repair_allowed,
        automatic_alert_action_allowed,
        automatic_audit_action_allowed,
        automatic_provenance_repair_allowed,
        manual_operation_entry_enabled,
        manual_operation_submit_allowed,
        manual_operation_cancel_allowed,
        manual_operation_retry_allowed,
        manual_operation_replace_allowed,
        manual_operation_amend_allowed,
        manual_operation_flatten_allowed,
        automatic_operation_action_allowed,
        product_grade_trading_terminal_claim,
    }
}

fn dashboard_bool_string(value: bool) -> DashboardValue<String> {
    DashboardValue::available(value.to_string())
}

fn read_model_operation_entry_blocked_reason(
    missing_owner_approval: bool,
    missing_risk_gate: bool,
    missing_audit_gate: bool,
    stale_read_model: bool,
    provenance_mismatch: bool,
) -> DashboardValue<String> {
    let mut reasons = Vec::new();
    if missing_owner_approval {
        reasons.push("missing_owner_approval");
    }
    if missing_risk_gate {
        reasons.push("missing_risk_gate");
    }
    if missing_audit_gate {
        reasons.push("missing_audit_gate");
    }
    if stale_read_model {
        reasons.push("stale_read_model");
    }
    if provenance_mismatch {
        reasons.push("provenance_mismatch");
    }

    if reasons.is_empty() {
        DashboardValue::available("none".to_string())
    } else {
        DashboardValue::available(reasons.join(","))
    }
}

#[derive(Clone, Copy)]
struct ReadModelOperationEntryState {
    ungated_operation_attempted: bool,
    stale_read_model: bool,
    provenance_mismatch: bool,
    missing_owner_approval: bool,
    missing_risk_gate: bool,
    missing_audit_gate: bool,
    gates_complete: bool,
    disabled: bool,
}

fn read_model_operation_entry_status(
    state: ReadModelOperationEntryState,
) -> DashboardValue<String> {
    let status = if state.ungated_operation_attempted {
        "fail_closed_ungated_operation_attempt"
    } else if state.stale_read_model {
        "blocked_stale_read_model"
    } else if state.provenance_mismatch {
        "fail_closed_provenance_mismatch"
    } else if state.missing_owner_approval {
        "blocked_missing_owner_approval"
    } else if state.missing_risk_gate {
        "blocked_missing_risk_gate"
    } else if state.missing_audit_gate {
        "blocked_missing_audit_gate"
    } else if state.gates_complete && state.disabled {
        "disabled_gated_preview_ready"
    } else if state.gates_complete {
        "gated_preview_ready"
    } else {
        "blocked_missing_gate"
    };
    DashboardValue::available(status.to_string())
}

fn required_read_model_boundary_bool(
    boundary: &Value,
    field: &str,
    diagnostics: &mut Vec<String>,
    health: &mut HealthStatus,
) -> DashboardValue<bool> {
    match boundary.get(field).and_then(Value::as_bool) {
        Some(false) => DashboardValue::available(false),
        Some(true) => {
            diagnostics.push(format!("{field}_true"));
            *health = strongest_health(*health, HealthStatus::Error);
            DashboardValue::available(true)
        }
        None => {
            diagnostics.push(format!("{field}_missing"));
            *health = strongest_health(*health, HealthStatus::Error);
            DashboardValue::unknown()
        }
    }
}

fn validate_v25_dashboard_surface_component(
    snapshot: &Value,
    component: &str,
    component_diagnostics: &mut Vec<String>,
    v25_surface_diagnostics: &mut Vec<String>,
    health: &mut HealthStatus,
) {
    let Some(component_value) = snapshot
        .get("components")
        .and_then(|components| components.get(component))
    else {
        let diagnostic = format!("{component}:component_missing");
        component_diagnostics.push(diagnostic.clone());
        v25_surface_diagnostics.push(diagnostic);
        *health = strongest_health(*health, HealthStatus::Degraded);
        return;
    };

    match component_value
        .get("component_status")
        .and_then(Value::as_str)
    {
        Some("healthy") => {}
        Some("degraded" | "partial" | "unavailable") => {
            let diagnostic = format!("{component}:component_degraded");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Degraded);
        }
        Some("stale") => {
            let diagnostic = format!("{component}:component_stale");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Stale);
        }
        Some("fail_closed" | "error") => {
            let diagnostic = format!("{component}:component_fail_closed");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
        Some(other) => {
            let diagnostic = format!("{component}:component_status_unexpected:{other}");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
        None => {
            let diagnostic = format!("{component}:component_status_missing");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
    }

    let freshness = component_value.get("freshness");
    match freshness
        .and_then(|freshness| freshness.get("status"))
        .and_then(Value::as_str)
    {
        Some("fresh") => {}
        Some("stale") => {
            let diagnostic = format!("{component}:freshness_stale");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Stale);
        }
        Some(other) => {
            let diagnostic = format!("{component}:freshness_{other}");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Degraded);
        }
        None => {
            let diagnostic = format!("{component}:freshness_missing");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
    }
    let observed_age_ms = freshness
        .and_then(|freshness| freshness.get("observed_age_ms"))
        .and_then(read_model_u64_value);
    let max_age_ms = freshness
        .and_then(|freshness| freshness.get("max_age_ms"))
        .and_then(read_model_u64_value);
    match (observed_age_ms, max_age_ms) {
        (Some(observed), Some(max_age)) if observed <= max_age => {}
        (Some(observed), Some(max_age)) => {
            let diagnostic =
                format!("{component}:freshness_threshold_exceeded:{observed}>{max_age}");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Stale);
        }
        _ => {
            let diagnostic = format!("{component}:freshness_threshold_missing");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
    }
    if freshness
        .and_then(|freshness| freshness.get("status"))
        .and_then(Value::as_str)
        .is_some_and(|status| status != "fresh")
    {
        let staleness_reason = freshness
            .and_then(|freshness| freshness.get("staleness_reason"))
            .and_then(read_model_scalar_string);
        match staleness_reason.as_deref() {
            Some(reason) if !reason.trim().is_empty() => {
                let diagnostic = format!("{component}:staleness_reason:{reason}");
                component_diagnostics.push(diagnostic.clone());
                v25_surface_diagnostics.push(diagnostic);
            }
            _ => {
                let diagnostic = format!("{component}:staleness_reason_missing");
                component_diagnostics.push(diagnostic.clone());
                v25_surface_diagnostics.push(diagnostic);
                *health = strongest_health(*health, HealthStatus::Error);
            }
        }
    }

    let source = component_value.get("source_provenance");
    let source_type_missing = source
        .and_then(|source| source.get("source_type"))
        .and_then(read_model_scalar_string)
        .is_none_or(|value| value.is_empty());
    let source_ref_missing = source
        .and_then(|source| source.get("source_ref"))
        .and_then(read_model_scalar_string)
        .is_none_or(|value| value.is_empty());
    if source_type_missing || source_ref_missing {
        let diagnostic = format!("{component}:source_provenance_missing");
        component_diagnostics.push(diagnostic.clone());
        v25_surface_diagnostics.push(diagnostic);
        *health = strongest_health(*health, HealthStatus::Error);
    }

    if component_value
        .get("redaction")
        .and_then(|redaction| redaction.get("status"))
        .and_then(Value::as_str)
        != Some("redacted")
    {
        let diagnostic = format!("{component}:redaction_state_not_ready");
        component_diagnostics.push(diagnostic.clone());
        v25_surface_diagnostics.push(diagnostic);
        *health = strongest_health(*health, HealthStatus::Error);
    }

    let Some(data) = component_value.get("data") else {
        let diagnostic = format!("{component}:data_missing");
        component_diagnostics.push(diagnostic.clone());
        v25_surface_diagnostics.push(diagnostic);
        *health = strongest_health(*health, HealthStatus::Error);
        return;
    };

    if data
        .get("slo_evidence_ref")
        .and_then(read_model_scalar_string)
        .is_none_or(|value| value.trim().is_empty())
    {
        let diagnostic = format!("{component}:slo_evidence_missing");
        component_diagnostics.push(diagnostic.clone());
        v25_surface_diagnostics.push(diagnostic);
        *health = strongest_health(*health, HealthStatus::Error);
    }
    match data
        .get("diagnostic_severity")
        .and_then(read_model_scalar_string)
        .as_deref()
    {
        Some("info" | "ok") => {}
        Some("warning") => {
            let diagnostic = format!("{component}:diagnostic_severity_warning");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Degraded);
        }
        Some("error" | "critical") => {
            let diagnostic = format!("{component}:diagnostic_severity_fail_closed");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
        Some(other) => {
            let diagnostic = format!("{component}:diagnostic_severity_unexpected:{other}");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
        None => {
            let diagnostic = format!("{component}:diagnostic_severity_missing");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
    }
    if data.get("partial_projection").and_then(Value::as_bool) == Some(true) {
        let diagnostic = format!("{component}:partial_projection");
        component_diagnostics.push(diagnostic.clone());
        v25_surface_diagnostics.push(diagnostic);
        *health = strongest_health(*health, HealthStatus::Degraded);
    }
    match data
        .get("source_truth_status")
        .and_then(read_model_scalar_string)
        .as_deref()
    {
        Some("artifact_truth_only") => {}
        Some("unknown") | None => {
            let diagnostic = format!("{component}:unknown_source_truth");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
        Some(other) => {
            let diagnostic = format!("{component}:source_truth_unexpected:{other}");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
    }
    match data
        .get("adapter_truth_status")
        .and_then(read_model_scalar_string)
        .as_deref()
    {
        Some("not_integrated") => {}
        Some("unknown") | None => {
            let diagnostic = format!("{component}:unknown_adapter_truth");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
        Some(other) => {
            let diagnostic = format!("{component}:adapter_truth_unexpected:{other}");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
    }
    match data
        .get("release_provenance_status")
        .and_then(read_model_scalar_string)
        .as_deref()
    {
        Some("matched") => {}
        Some("drift") => {
            let diagnostic = format!("{component}:release_provenance_drift");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
        Some(other) => {
            let diagnostic = format!("{component}:release_provenance_unexpected:{other}");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
        None => {
            let diagnostic = format!("{component}:release_provenance_missing");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
    }
    for field in ["remediation_action_allowed", "trading_action_allowed"] {
        match data.get(field).and_then(Value::as_bool) {
            Some(false) => {}
            Some(true) => {
                let diagnostic = format!("{component}:boundary_true:{field}");
                component_diagnostics.push(diagnostic.clone());
                v25_surface_diagnostics.push(diagnostic);
                *health = strongest_health(*health, HealthStatus::Error);
            }
            None => {
                let diagnostic = format!("{component}:boundary_missing:{field}");
                component_diagnostics.push(diagnostic.clone());
                v25_surface_diagnostics.push(diagnostic);
                *health = strongest_health(*health, HealthStatus::Error);
            }
        }
    }

    for field in [
        "forbidden_control_detected",
        "dashboard_trading_control_allowed",
        "submit_order_allowed",
        "cancel_order_allowed",
        "retry_order_allowed",
        "replace_order_allowed",
        "amend_order_allowed",
        "flatten_position_allowed",
        "order_ticket_enabled",
        "live_exchange_request_allowed",
        "adapter_send_allowed",
        "automatic_remediation_allowed",
        "automatic_actions_allowed",
    ] {
        if data.get(field).and_then(Value::as_bool) == Some(true) {
            let diagnostic = format!("{component}:boundary_true:{field}");
            component_diagnostics.push(diagnostic.clone());
            v25_surface_diagnostics.push(diagnostic);
            *health = strongest_health(*health, HealthStatus::Error);
        }
    }

    if data
        .get("operation_boundary_readonly")
        .and_then(Value::as_bool)
        == Some(false)
    {
        let diagnostic = format!("{component}:operation_boundary_not_readonly");
        component_diagnostics.push(diagnostic.clone());
        v25_surface_diagnostics.push(diagnostic);
        *health = strongest_health(*health, HealthStatus::Error);
    }
}

fn v25_dashboard_surface_status(
    v25_surface_diagnostics: &[String],
    health: HealthStatus,
) -> DashboardValue<String> {
    let status = if v25_surface_diagnostics.is_empty() && health == HealthStatus::Healthy {
        "ready_readonly_surface"
    } else if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("component_missing"))
    {
        "degraded_missing_surface_artifact"
    } else if v25_surface_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("boundary_true")
            || diagnostic.contains("forbidden_control")
            || diagnostic.contains("operation_boundary_not_readonly")
    }) || health == HealthStatus::Error
    {
        "fail_closed_surface_artifact"
    } else if health == HealthStatus::Stale {
        "stale_surface_artifact"
    } else {
        "degraded_surface_artifact"
    };
    DashboardValue::available(status.to_string())
}

fn v25_diagnostics_gate_status(
    v25_surface_diagnostics: &[String],
    health: HealthStatus,
) -> DashboardValue<String> {
    let status = if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("component_missing"))
    {
        "fail_closed_missing_component"
    } else if v25_surface_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("unknown_source_truth") || diagnostic.contains("unknown_adapter_truth")
    }) {
        "fail_closed_unknown_source_truth"
    } else if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("release_provenance"))
    {
        "fail_closed_release_provenance_drift"
    } else if v25_surface_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("boundary_true")
            || diagnostic.contains("forbidden_control")
            || diagnostic.contains("operation_boundary_not_readonly")
    }) {
        "fail_closed_forbidden_action"
    } else if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("partial_projection"))
    {
        "degraded_partial_projection"
    } else if v25_surface_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("freshness_threshold_exceeded")
            || diagnostic.contains("freshness_stale")
            || diagnostic.contains("staleness_reason")
    }) {
        "degraded_stale_source"
    } else if v25_surface_diagnostics.is_empty() && health == HealthStatus::Healthy {
        "ready_slo_freshness_gate"
    } else if health == HealthStatus::Error {
        "fail_closed_diagnostics_gate"
    } else {
        "degraded_diagnostics_gate"
    };
    DashboardValue::available(status.to_string())
}

fn v25_slo_status(v25_surface_diagnostics: &[String]) -> DashboardValue<String> {
    let status = if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("slo_evidence_missing"))
    {
        "fail_closed_missing_slo_evidence"
    } else {
        "slo_evidence_ready"
    };
    DashboardValue::available(status.to_string())
}

fn v25_freshness_threshold_status(v25_surface_diagnostics: &[String]) -> DashboardValue<String> {
    let status = if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("freshness_threshold_missing"))
    {
        "fail_closed_threshold_missing"
    } else if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("freshness_threshold_exceeded"))
    {
        "degraded_threshold_exceeded"
    } else {
        "freshness_thresholds_ready"
    };
    DashboardValue::available(status.to_string())
}

fn v25_staleness_reasons(v25_surface_diagnostics: &[String]) -> DashboardValue<String> {
    let reasons = v25_surface_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.contains("staleness_reason"))
        .cloned()
        .collect::<Vec<_>>();
    if reasons.is_empty() {
        DashboardValue::available("none".to_string())
    } else {
        DashboardValue::available(reasons.join(","))
    }
}

fn v25_diagnostic_severity(
    v25_surface_diagnostics: &[String],
    health: HealthStatus,
) -> DashboardValue<String> {
    let severity = if health == HealthStatus::Error
        || v25_surface_diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("fail_closed")
                || diagnostic.contains("boundary_true")
                || diagnostic.contains("unknown_source_truth")
                || diagnostic.contains("unknown_adapter_truth")
                || diagnostic.contains("release_provenance")
        }) {
        "critical"
    } else if health == HealthStatus::Stale
        || health == HealthStatus::Degraded
        || v25_surface_diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("freshness_threshold_exceeded")
                || diagnostic.contains("partial_projection")
                || diagnostic.contains("diagnostic_severity_warning")
        })
    {
        "warning"
    } else {
        "info"
    };
    DashboardValue::available(severity.to_string())
}

fn v25_source_truth_status(v25_surface_diagnostics: &[String]) -> DashboardValue<String> {
    let status = if v25_surface_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("unknown_source_truth") || diagnostic.contains("unknown_adapter_truth")
    }) {
        "fail_closed_unknown_source_truth"
    } else {
        "artifact_truth_only"
    };
    DashboardValue::available(status.to_string())
}

fn v25_release_provenance_status(v25_surface_diagnostics: &[String]) -> DashboardValue<String> {
    let status = if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("release_provenance_drift"))
    {
        "fail_closed_release_provenance_drift"
    } else if v25_surface_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("release_provenance_missing"))
    {
        "fail_closed_release_provenance_missing"
    } else {
        "matched"
    };
    DashboardValue::available(status.to_string())
}

fn v25_no_remediation_status(v25_surface_diagnostics: &[String]) -> DashboardValue<String> {
    let status = if v25_surface_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("boundary_true:remediation_action_allowed")
            || diagnostic.contains("boundary_true:trading_action_allowed")
            || diagnostic.contains("automatic_remediation_allowed")
            || diagnostic.contains("automatic_actions_allowed")
    }) {
        "fail_closed_forbidden_action"
    } else if v25_surface_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("boundary_missing:remediation_action_allowed")
            || diagnostic.contains("boundary_missing:trading_action_allowed")
    }) {
        "fail_closed_missing_no_action_boundary"
    } else {
        "no_remediation_no_trading_actions"
    };
    DashboardValue::available(status.to_string())
}

fn trader_terminal_read_model_readiness(
    health: HealthStatus,
    schema_version: Option<&str>,
    missing_components: &[String],
    component_diagnostics: &[String],
    freshness_status: Option<&str>,
    has_blocking_reasons: bool,
) -> String {
    if schema_version != Some(UNIFIED_READ_MODEL_SCHEMA_VERSION) {
        "schema_mismatch".to_string()
    } else if freshness_status == Some("stale")
        || component_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.ends_with(":freshness_stale"))
    {
        "stale_artifact".to_string()
    } else if !missing_components.is_empty() {
        "component_missing".to_string()
    } else if component_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.ends_with(":unavailable"))
    {
        "component_unavailable".to_string()
    } else if health == HealthStatus::Error {
        "fail_closed".to_string()
    } else if health == HealthStatus::Stale {
        "stale_artifact".to_string()
    } else if health == HealthStatus::Degraded || has_blocking_reasons {
        "degraded_artifact".to_string()
    } else {
        "ready_readonly_artifact".to_string()
    }
}

fn read_model_string_array_field(value: &Value, field: &str) -> DashboardValue<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(read_model_scalar_string)
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|joined| !joined.is_empty())
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn read_model_component_data_summary(snapshot: &Value, component: &str) -> DashboardValue<String> {
    let Some(data) = snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("data"))
    else {
        return DashboardValue::unknown();
    };
    let fields: &[&str] = match component {
        "account" => &[
            "summary_status",
            "account_id",
            "account_status",
            "risk_state",
            "equity",
            "available_balance",
            "balance_entry_count",
        ],
        "positions" => &[
            "summary_status",
            "account_id",
            "position_count",
            "net_position_side",
            "quantity",
            "net_exposure",
            "notional",
            "precision",
        ],
        "orders" => &[
            "client_order_id",
            "request_digest",
            "attempt_id",
            "approval_id",
            "lifecycle_status",
            "open_order_count",
            "terminal_order_count",
            "submitted",
            "accepted",
            "rejected",
            "readback_status",
            "audit_state",
        ],
        "fills" => &[
            "fill_id",
            "execution_id",
            "fill_status",
            "fill_count",
            "order_linkage_status",
            "reconciliation_status",
            "quantity",
            "price",
            "partial_fill_detected",
            "duplicate_fill_detected",
            "last_fill_id",
        ],
        "risk" => &[
            "risk_state",
            "risk_visible",
            "critical_evidence_complete",
            "manual_review_required",
            "halted",
            "mismatch_detected",
            "freshness_rollup",
        ],
        "lifecycle_status" => &[
            "lifecycle_status",
            "audit_state",
            "audit_closed",
            "required_evidence_complete",
            "required_components_complete",
            "release_provenance",
            "artifact_digest",
            "readback_status",
            "no_retry",
            "ledger_present",
        ],
        _ => &[],
    };
    let summary = fields
        .iter()
        .filter_map(|field| {
            data.get(*field)
                .and_then(read_model_scalar_string)
                .map(|value| format!("{field}={value}"))
        })
        .collect::<Vec<_>>()
        .join("; ");
    if summary.is_empty() {
        DashboardValue::unknown()
    } else {
        DashboardValue::available(summary)
    }
}

fn read_model_component_data_scalar(
    snapshot: &Value,
    component: &str,
    field: &str,
) -> DashboardValue<String> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("data"))
        .and_then(|data| data.get(field))
        .and_then(read_model_scalar_string)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn read_model_component_data_bool(snapshot: &Value, component: &str, field: &str) -> Option<bool> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("data"))
        .and_then(|data| data.get(field))
        .and_then(Value::as_bool)
}

fn read_model_component_data_nested_scalar(
    snapshot: &Value,
    component: &str,
    object_field: &str,
    field: &str,
) -> DashboardValue<String> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("data"))
        .and_then(|data| data.get(object_field))
        .and_then(|object| object.get(field))
        .and_then(read_model_scalar_string)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn read_model_component_data_nested_bool(
    snapshot: &Value,
    component: &str,
    object_field: &str,
    field: &str,
) -> Option<bool> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("data"))
        .and_then(|data| data.get(object_field))
        .and_then(|object| object.get(field))
        .and_then(Value::as_bool)
}

fn read_model_component_data_array_field(
    snapshot: &Value,
    component: &str,
    field: &str,
) -> DashboardValue<String> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("data"))
        .and_then(|data| data.get(field))
        .and_then(read_model_scalar_string)
        .filter(|joined| !joined.is_empty())
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn read_model_risk_priority_state(snapshot: &Value) -> DashboardValue<String> {
    let freshness = read_model_component_freshness_status(snapshot, "risk");
    let freshness_rollup = read_model_component_data_scalar(snapshot, "risk", "freshness_rollup");
    let risk_visible = read_model_component_data_bool(snapshot, "risk", "risk_visible");
    let manual_review_required =
        read_model_component_data_bool(snapshot, "risk", "manual_review_required");
    let halted = read_model_component_data_bool(snapshot, "risk", "halted");
    let mismatch_detected = read_model_component_data_bool(snapshot, "risk", "mismatch_detected");

    let state = if halted == Some(true) {
        "halted"
    } else if mismatch_detected == Some(true) {
        "mismatch"
    } else if freshness.value.as_deref() == Some("stale")
        || freshness_rollup.value.as_deref() == Some("stale")
    {
        "stale"
    } else if manual_review_required == Some(true) {
        "manual_review"
    } else if risk_visible == Some(true) {
        "risk_visible"
    } else {
        "healthy"
    };
    DashboardValue::available(state.to_string())
}

fn read_model_alert_severity(snapshot: &Value) -> DashboardValue<String> {
    let critical = [
        "missing_evidence",
        "schema_mismatch",
        "redaction_breach",
        "forbidden_control_request",
    ]
    .iter()
    .any(|field| {
        read_model_component_data_nested_bool(snapshot, "risk", "alerts", field) == Some(true)
    });
    if critical {
        return DashboardValue::available("critical".to_string());
    }
    if read_model_component_data_nested_bool(snapshot, "risk", "alerts", "stale_source")
        == Some(true)
    {
        return DashboardValue::available("warning".to_string());
    }
    read_model_component_data_nested_scalar(snapshot, "risk", "alerts", "highest_severity")
}

fn read_model_component_data_object_summary(
    snapshot: &Value,
    component: &str,
    object_field: &str,
    fields: &[&str],
) -> DashboardValue<String> {
    let Some(object) = snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("data"))
        .and_then(|data| data.get(object_field))
        .and_then(Value::as_object)
    else {
        return DashboardValue::unknown();
    };
    let summary = fields
        .iter()
        .filter_map(|field| {
            object
                .get(*field)
                .and_then(read_model_scalar_string)
                .map(|value| format!("{field}={value}"))
        })
        .collect::<Vec<_>>()
        .join("; ");
    if summary.is_empty() {
        DashboardValue::unknown()
    } else {
        DashboardValue::available(summary)
    }
}

fn read_model_component_freshness_status(
    snapshot: &Value,
    component: &str,
) -> DashboardValue<String> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("freshness"))
        .and_then(|freshness| freshness.get("status"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn read_model_component_source_field(
    snapshot: &Value,
    component: &str,
    field: &str,
) -> DashboardValue<String> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("source_provenance"))
        .and_then(|source| source.get(field))
        .and_then(read_model_scalar_string)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn read_model_component_redaction_status(
    snapshot: &Value,
    component: &str,
) -> DashboardValue<String> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("redaction"))
        .and_then(|redaction| redaction.get("status"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn read_model_component_diagnostics(snapshot: &Value, component: &str) -> Vec<String> {
    snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("diagnostics"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(read_model_scalar_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn read_model_component_diagnostics_summary(
    snapshot: &Value,
    component: &str,
) -> DashboardValue<String> {
    let diagnostics = read_model_component_diagnostics(snapshot, component);
    if diagnostics.is_empty() {
        DashboardValue::unknown()
    } else {
        DashboardValue::available(diagnostics.join(";"))
    }
}

fn read_model_component_lineage_summary(
    snapshot: &Value,
    component: &str,
) -> DashboardValue<String> {
    let Some(lineage) = snapshot
        .get("components")
        .and_then(|components| components.get(component))
        .and_then(|component| component.get("lineage"))
    else {
        return DashboardValue::unknown();
    };

    let mut parts = Vec::new();
    if let Some(transform) = lineage.get("transform").and_then(read_model_scalar_string) {
        parts.push(format!("transform={transform}"));
    }
    if let Some(input_refs) = lineage.get("input_refs").and_then(read_model_scalar_string) {
        parts.push(format!("input_refs={input_refs}"));
    }
    if parts.is_empty() {
        DashboardValue::unknown()
    } else {
        DashboardValue::available(parts.join(", "))
    }
}

fn read_model_scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Array(items) => {
            let joined = items
                .iter()
                .filter_map(read_model_scalar_string)
                .collect::<Vec<_>>()
                .join(",");
            if joined.is_empty() {
                None
            } else {
                Some(joined)
            }
        }
        Value::Null | Value::Object(_) => None,
    }
}

fn read_model_u64_value(value: &Value) -> Option<u64> {
    match value {
        Value::Number(value) => value.as_u64(),
        Value::String(value) => value.parse::<u64>().ok(),
        Value::Bool(_) | Value::Array(_) | Value::Null | Value::Object(_) => None,
    }
}

fn dashboard_availability_from_read_model_health(health: HealthStatus) -> DashboardAvailability {
    match health {
        HealthStatus::Healthy => DashboardAvailability::Available,
        HealthStatus::Stale => DashboardAvailability::Stale,
        HealthStatus::Degraded | HealthStatus::Error => DashboardAvailability::Unknown,
        HealthStatus::Unknown => DashboardAvailability::Unknown,
    }
}

fn logging_module_status(
    record: &SupervisorNodeRecord,
    status: &NodeStatus,
) -> RuntimeModuleStatus {
    let log_paths = [
        &record.stdout_log_path,
        &record.stderr_log_path,
        &record.events_log_path,
    ];
    let all_logs_present = log_paths.iter().all(|path| path.exists());
    RuntimeModuleStatus {
        module_name: module_name(record, "Logging"),
        status: if all_logs_present {
            DashboardValue::available("logs_available".to_string())
        } else {
            DashboardValue::unknown()
        },
        health: if all_logs_present {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        },
        last_seen_at: if all_logs_present {
            dashboard_value_from_snapshot(&status.generated_at)
        } else {
            DashboardValue::unknown()
        },
        last_error: if all_logs_present {
            DashboardValue::unknown()
        } else {
            DashboardValue::available("present".to_string())
        },
        evidence_source: DashboardValue::available(
            record.artifact_root.join("logs").display().to_string(),
        ),
    }
}

fn metrics_writer_module_status(
    record: &SupervisorNodeRecord,
    status: &NodeStatus,
) -> RuntimeModuleStatus {
    let availability = availability_from_registry_state(record.metrics_artifact);
    RuntimeModuleStatus {
        module_name: module_name(record, "Metrics writer"),
        status: match availability {
            DashboardAvailability::Available => {
                DashboardValue::available("metrics_available".to_string())
            }
            DashboardAvailability::Stale => DashboardValue::stale(),
            DashboardAvailability::NotConfigured => DashboardValue::not_configured(),
            DashboardAvailability::NotSupported => DashboardValue::not_supported(),
            DashboardAvailability::Redacted | DashboardAvailability::Unknown => {
                DashboardValue::unknown()
            }
        },
        health: match availability {
            DashboardAvailability::Available => HealthStatus::Healthy,
            DashboardAvailability::Stale => HealthStatus::Stale,
            DashboardAvailability::Unknown
            | DashboardAvailability::NotConfigured
            | DashboardAvailability::NotSupported
            | DashboardAvailability::Redacted => HealthStatus::Unknown,
        },
        last_seen_at: if record.metrics_path.exists() {
            dashboard_value_from_snapshot(&status.generated_at)
        } else {
            DashboardValue::unknown()
        },
        last_error: if record.metrics_path.exists() {
            DashboardValue::unknown()
        } else {
            DashboardValue::available("present".to_string())
        },
        evidence_source: DashboardValue::available(record.metrics_path.display().to_string()),
    }
}

fn supervisor_module_status(record: &SupervisorNodeRecord) -> RuntimeModuleStatus {
    RuntimeModuleStatus {
        module_name: module_name(record, "Supervisor"),
        status: DashboardValue::available(json_label(&record.process.state)),
        health: health_from_process_state(record.process.state),
        last_seen_at: dashboard_value_from_snapshot(&record.process.updated_at),
        last_error: DashboardValue::unknown(),
        evidence_source: DashboardValue::available(record.pid_path.display().to_string()),
    }
}

fn unsupported_runtime_module(
    record: &SupervisorNodeRecord,
    module: &str,
    notes: &str,
) -> (RuntimeModuleStatus, DashboardGap) {
    (
        RuntimeModuleStatus {
            module_name: module_name(record, module),
            status: DashboardValue::not_supported(),
            health: HealthStatus::Unknown,
            last_seen_at: DashboardValue::not_supported(),
            last_error: DashboardValue::not_supported(),
            evidence_source: DashboardValue::not_supported(),
        },
        DashboardGap::new(
            module_gap_path(record, module),
            DashboardAvailability::NotSupported,
            "V03-008",
            notes,
        ),
    )
}

fn module_name(record: &SupervisorNodeRecord, module: &str) -> String {
    format!("{}:{module}", record.node_id)
}

fn module_gap_path(record: &SupervisorNodeRecord, module: &str) -> String {
    format!(
        "runtime_modules.{}.{}",
        record.node_id,
        module.to_ascii_lowercase().replace(' ', "_")
    )
}

fn log_statuses_from_record(record: &SupervisorNodeRecord, status: &NodeStatus) -> Vec<LogStatus> {
    [
        ("stdout", &record.stdout_log_path),
        ("stderr", &record.stderr_log_path),
        ("events", &record.events_log_path),
    ]
    .into_iter()
    .map(|(kind, path)| {
        let exists = path.exists();
        LogStatus {
            log_id: format!("{}:{kind}", record.node_id),
            node_id: DashboardValue::available(record.node_id.clone()),
            path: DashboardValue::available(path.display().to_string()),
            availability: if exists {
                DashboardAvailability::Available
            } else {
                DashboardAvailability::Unknown
            },
            last_seen_at: if exists {
                dashboard_value_from_snapshot(&status.generated_at)
            } else {
                DashboardValue::unknown()
            },
            last_error: if exists {
                DashboardValue::unknown()
            } else {
                DashboardValue::available(format!("日志工件 '{}' 缺失", path.display()))
            },
        }
    })
    .collect()
}

fn metric_statuses_from_record(
    record: &SupervisorNodeRecord,
    status_availability: DashboardAvailability,
) -> Vec<MetricStatus> {
    if !record.metrics_path.exists() {
        return vec![MetricStatus {
            metric_id: format!("{}:node-metrics", record.node_id),
            node_id: DashboardValue::available(record.node_id.clone()),
            value: DashboardValue::unknown(),
            availability: DashboardAvailability::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::available(format!(
                "指标工件 '{}' 缺失",
                record.metrics_path.display()
            )),
        }];
    }

    let raw = match fs::read_to_string(&record.metrics_path) {
        Ok(raw) => raw,
        Err(error) => {
            return vec![MetricStatus {
                metric_id: format!("{}:node-metrics", record.node_id),
                node_id: DashboardValue::available(record.node_id.clone()),
                value: DashboardValue::unknown(),
                availability: DashboardAvailability::Unknown,
                last_seen_at: DashboardValue::unknown(),
                last_error: DashboardValue::available(format!(
                    "读取指标工件 '{}' 失败：{error}",
                    record.metrics_path.display()
                )),
            }];
        }
    };

    let metrics = match serde_json::from_str::<NodeMetrics>(&raw) {
        Ok(metrics) if metrics.node_id == record.node_id => metrics,
        Ok(metrics) => {
            return vec![MetricStatus {
                metric_id: format!("{}:node-metrics", record.node_id),
                node_id: DashboardValue::available(record.node_id.clone()),
                value: DashboardValue::unknown(),
                availability: DashboardAvailability::Unknown,
                last_seen_at: DashboardValue::unknown(),
                last_error: DashboardValue::available(format!(
                    "指标节点身份不匹配：注册表节点 '{}' 收到运行时节点 '{}'",
                    record.node_id, metrics.node_id
                )),
            }];
        }
        Err(error) => {
            return vec![MetricStatus {
                metric_id: format!("{}:node-metrics", record.node_id),
                node_id: DashboardValue::available(record.node_id.clone()),
                value: DashboardValue::unknown(),
                availability: DashboardAvailability::Unknown,
                last_seen_at: DashboardValue::unknown(),
                last_error: DashboardValue::available(format!(
                    "指标工件 '{}' 无效：{error}",
                    record.metrics_path.display()
                )),
            }];
        }
    };

    let availability = if metrics.generated_at.availability == SnapshotAvailability::Stale
        || record.metrics_artifact == RegistryArtifactState::Stale
        || status_availability == DashboardAvailability::Stale
    {
        DashboardAvailability::Stale
    } else {
        availability_from_registry_state(record.metrics_artifact)
    };

    [
        ("starts_total", metrics.starts_total.to_string()),
        ("stops_total", metrics.stops_total.to_string()),
        (
            "state_transitions_total",
            metrics.state_transitions_total.to_string(),
        ),
        (
            "uptime_ms",
            metrics
                .uptime_ms
                .value
                .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
        ),
    ]
    .into_iter()
    .map(|(name, value)| MetricStatus {
        metric_id: format!("{}:{name}", record.node_id),
        node_id: DashboardValue::available(record.node_id.clone()),
        value: DashboardValue::available(value),
        availability,
        last_seen_at: dashboard_value_from_snapshot(&metrics.generated_at),
        last_error: optional_dashboard_value(metrics.last_error_summary.clone()),
    })
    .collect()
}

fn strategy_runtime_from_record(record: &SupervisorNodeRecord) -> Option<StrategyRuntimeStatus> {
    let strategy_root = record.artifact_root.join("strategy");
    let session_status_path = strategy_root.join("session_status.json");
    let market_status_path = strategy_root.join("market_status.json");
    let signal_path = strategy_root.join("signal.jsonl");
    let order_intent_path = strategy_root.join("order_intent.jsonl");
    let risk_decision_path = strategy_root.join("risk_decision.jsonl");
    let summary_path = strategy_root.join("summary.json");
    let manifest_path = strategy_root.join("manifest.json");

    let has_strategy_artifact = [
        &manifest_path,
        &session_status_path,
        &market_status_path,
        &signal_path,
        &order_intent_path,
        &risk_decision_path,
        &summary_path,
    ]
    .iter()
    .any(|path| path.exists());
    if !has_strategy_artifact {
        return None;
    }

    let session = read_json_file_value(&session_status_path);
    let market = read_json_file_value(&market_status_path);
    let signal = read_latest_jsonl_file_value(&signal_path);
    let order_intent = read_latest_jsonl_file_value(&order_intent_path);
    let risk_decision = read_latest_jsonl_file_value(&risk_decision_path);
    let summary = read_json_file_value(&summary_path);

    let symbol = first_dashboard_string_field([&signal, &order_intent, &risk_decision], "symbol");
    let lifecycle_state = json_label(&record.last_known_status.lifecycle_state);
    let audit = audit_strategy_session_artifacts(&strategy_root, Some(&lifecycle_state));
    let health = match audit.health {
        StrategySessionArtifactAuditHealth::Healthy => HealthStatus::Healthy,
        StrategySessionArtifactAuditHealth::Degraded => HealthStatus::Degraded,
    };

    Some(StrategyRuntimeStatus {
        node_id: record.node_id.clone(),
        health,
        diagnostic: DashboardValue::available(audit.diagnostic_label()),
        session_id: session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "session_id")
            }),
        session_state: session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "state")
            }),
        strategy_id: session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "strategy_id")
            }),
        symbol,
        market_stream_status: market
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field_with_fallback(value, "state", "connection")
            }),
        signal_count: summary
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "signal_count")
            }),
        latest_signal: signal
            .as_ref()
            .map_or_else(DashboardValue::unknown, latest_signal_label),
        latest_order_intent: order_intent
            .as_ref()
            .map_or_else(DashboardValue::unknown, latest_order_intent_label),
        latest_risk_decision: risk_decision
            .as_ref()
            .map_or_else(DashboardValue::unknown, latest_risk_decision_label),
        rejection_reason: risk_decision
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "reasons")
            }),
        order_submission_mode: risk_decision
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "order_submission")
            }),
        actual_submission_count: summary
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "actual_submission_count")
            }),
        session_status_path: dashboard_path_if_exists(&session_status_path),
        signal_artifact_path: dashboard_path_if_exists(&signal_path),
        order_intent_artifact_path: dashboard_path_if_exists(&order_intent_path),
        risk_decision_artifact_path: dashboard_path_if_exists(&risk_decision_path),
        summary_artifact_path: dashboard_path_if_exists(&summary_path),
        manifest_path: dashboard_path_if_exists(&audit.manifest_path),
    })
}

#[derive(Clone, Debug)]
struct ProductionShadowArtifactPaths {
    version_label: &'static str,
    root: PathBuf,
    manifest_path: PathBuf,
    public_read_probe_path: PathBuf,
    account_snapshot_path: PathBuf,
    response_shape_path: PathBuf,
    shadow_intent_path: PathBuf,
    portfolio_snapshot_path: PathBuf,
    shadow_strategy_session_path: PathBuf,
    lifecycle_path: PathBuf,
    reconciliation_path: PathBuf,
    kill_switch_approval_artifact_path: PathBuf,
}

impl ProductionShadowArtifactPaths {
    fn v13(record: &SupervisorNodeRecord) -> Self {
        let root = record.artifact_root.join("v0_13");
        Self {
            version_label: "v0_13",
            manifest_path: root.join("manifest.json"),
            public_read_probe_path: root.join("production_public_online_read_probe.json"),
            account_snapshot_path: root.join("production_account_snapshot_redacted.json"),
            response_shape_path: root.join("production_readonly_response_shape.json"),
            shadow_intent_path: root.join("shadow_execution_intent.jsonl"),
            portfolio_snapshot_path: root.join("shadow_portfolio_runtime.json"),
            shadow_strategy_session_path: root.join("shadow_strategy_session.jsonl"),
            lifecycle_path: root.join("order_lifecycle_state.jsonl"),
            reconciliation_path: root.join("reconciliation_events.jsonl"),
            kill_switch_approval_artifact_path: record
                .artifact_root
                .join(PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_RELATIVE_PATH),
            root,
        }
    }

    fn v12(record: &SupervisorNodeRecord) -> Self {
        let root = record.artifact_root.join("v0_12");
        Self {
            version_label: "v0_12",
            manifest_path: root.join("manifest.json"),
            public_read_probe_path: root.join("production_public_online_read_probe.json"),
            account_snapshot_path: root.join("production_account_snapshot_redacted.json"),
            response_shape_path: root.join("production_readonly_response_shape.json"),
            shadow_intent_path: root.join("shadow_execution_intent.jsonl"),
            portfolio_snapshot_path: root.join("shadow_portfolio_runtime.json"),
            shadow_strategy_session_path: root.join("shadow_strategy_session.jsonl"),
            lifecycle_path: root.join("order_lifecycle_state.jsonl"),
            reconciliation_path: root.join("reconciliation_events.jsonl"),
            kill_switch_approval_artifact_path: record
                .artifact_root
                .join(PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_RELATIVE_PATH),
            root,
        }
    }

    fn v11(record: &SupervisorNodeRecord) -> Self {
        let root = record.artifact_root.join("v0_11");
        Self {
            version_label: "v0_11",
            manifest_path: root.join("manifest.json"),
            public_read_probe_path: root.join("production_public_read_probe.json"),
            account_snapshot_path: root.join("account_snapshot_redacted.json"),
            response_shape_path: root.join("production_readonly_response_shape.json"),
            shadow_intent_path: root.join("shadow_execution_intent.jsonl"),
            portfolio_snapshot_path: root.join("shadow_portfolio_snapshot.json"),
            shadow_strategy_session_path: root.join("shadow_strategy_session.jsonl"),
            lifecycle_path: root.join("order_lifecycle_state.jsonl"),
            reconciliation_path: root.join("reconciliation_events.jsonl"),
            kill_switch_approval_artifact_path: record
                .artifact_root
                .join(PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_RELATIVE_PATH),
            root,
        }
    }

    fn has_any_artifact(&self) -> bool {
        [
            &self.public_read_probe_path,
            &self.account_snapshot_path,
            &self.response_shape_path,
            &self.shadow_intent_path,
            &self.portfolio_snapshot_path,
            &self.shadow_strategy_session_path,
            &self.lifecycle_path,
            &self.reconciliation_path,
            &self.manifest_path,
            &self.kill_switch_approval_artifact_path,
        ]
        .iter()
        .any(|path| path.exists())
    }

    fn has_shadow_artifact(&self) -> bool {
        [
            &self.public_read_probe_path,
            &self.account_snapshot_path,
            &self.response_shape_path,
            &self.shadow_intent_path,
            &self.portfolio_snapshot_path,
            &self.shadow_strategy_session_path,
            &self.lifecycle_path,
            &self.reconciliation_path,
            &self.manifest_path,
        ]
        .iter()
        .any(|path| path.exists())
    }

    fn is_v13_only(&self) -> bool {
        self.version_label == "v0_13"
    }

    fn is_v12(&self) -> bool {
        self.version_label == "v0_12"
    }
}

fn production_shadow_paths(record: &SupervisorNodeRecord) -> Option<ProductionShadowArtifactPaths> {
    let v12_paths = ProductionShadowArtifactPaths::v12(record);
    if v12_paths.has_shadow_artifact() {
        return Some(v12_paths);
    }

    let v11_paths = ProductionShadowArtifactPaths::v11(record);
    if v11_paths.has_shadow_artifact() {
        return Some(v11_paths);
    }

    let v13_paths = ProductionShadowArtifactPaths::v13(record);
    v13_paths.has_any_artifact().then_some(v13_paths)
}

fn production_shadow_from_record(record: &SupervisorNodeRecord) -> Option<ProductionShadowStatus> {
    let paths = production_shadow_paths(record)?;

    let public_read_probe = read_json_file_value(&paths.public_read_probe_path);
    let account_snapshot = read_json_file_value(&paths.account_snapshot_path);
    let response_shape = read_json_file_value(&paths.response_shape_path);
    let shadow_intent = read_latest_jsonl_file_value(&paths.shadow_intent_path);
    let portfolio_snapshot = read_json_file_value(&paths.portfolio_snapshot_path);
    let shadow_strategy_session = read_latest_jsonl_file_value(&paths.shadow_strategy_session_path);
    let lifecycle = read_latest_jsonl_file_value(&paths.lifecycle_path);
    let reconciliation = read_latest_jsonl_file_value(&paths.reconciliation_path);
    let kill_switch_approval = read_json_file_value(&paths.kill_switch_approval_artifact_path);
    let shadow_artifact_audit = if paths.is_v13_only() {
        ProductionShadowArtifactHealthAudit {
            boundary_violation: false,
            diagnostic: None,
        }
    } else if paths.is_v12() {
        audit_production_shadow_v12_artifact_health(&paths)
    } else {
        audit_production_shadow_v11_artifact_health(
            &paths.account_snapshot_path,
            &paths.shadow_intent_path,
            &paths.portfolio_snapshot_path,
            &paths.lifecycle_path,
            &paths.reconciliation_path,
        )
    };
    let kill_switch_artifact_audit = audit_production_kill_switch_artifact_health(
        &paths.kill_switch_approval_artifact_path,
        paths.is_v13_only(),
    );
    let artifact_audit =
        combine_production_shadow_audits(shadow_artifact_audit, kill_switch_artifact_audit);
    let manifest_audit = audit_production_shadow_manifest(&paths.root);

    let actual_submission_count = first_available_u64_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "actual_submission_count")
            }),
        portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "actual_submission_count")
            }),
        lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "actual_submission_count")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "actual_submission_count")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "actual_submission_count")
            }),
    ]);
    let production_order_submissions_attempted = first_available_u64_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_submissions_attempted")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_submissions_attempted")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_submissions_attempted")
            }),
        account_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_as_u64_field(value, "production_order_submission_attempted")
            }),
        public_read_probe
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_as_u64_field(value, "production_order_submission_attempted")
            }),
    ]);
    let production_orders_submitted = first_available_u64_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_orders_submitted")
            }),
        portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_orders_submitted")
            }),
        lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_orders_submitted")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_orders_submitted")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_orders_submitted")
            }),
    ]);
    let production_order_mutations_attempted = first_available_u64_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_mutations_attempted")
            }),
        portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_mutations_attempted")
            }),
        lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_mutations_attempted")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_mutations_attempted")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_mutations_attempted")
            }),
        account_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_as_u64_field(value, "production_order_mutation_attempted")
            }),
        public_read_probe
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_as_u64_field(value, "production_order_mutation_attempted")
            }),
    ]);
    let production_order_state_reads_attempted = first_available_u64_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_state_reads_attempted")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_state_reads_attempted")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "production_order_state_reads_attempted")
            }),
    ]);
    let listen_key_lifecycle_attempted = first_available_u64_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "listen_key_lifecycle_attempted")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "listen_key_lifecycle_attempted")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "listen_key_lifecycle_attempted")
            }),
    ]);
    let automatic_correction_orders_submitted = first_available_u64_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "automatic_correction_orders_submitted")
            }),
        portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "automatic_correction_orders_submitted")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "automatic_correction_orders_submitted")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "automatic_correction_orders_submitted")
            }),
    ]);
    let dashboard_order_controls_enabled = first_available_bool_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "dashboard_order_controls_enabled")
            }),
        portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "dashboard_order_controls_enabled")
            }),
        lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "dashboard_order_controls_enabled")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "dashboard_order_controls_enabled")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "dashboard_order_controls_enabled")
            }),
        account_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "dashboard_order_controls_enabled")
            }),
        public_read_probe
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "dashboard_order_controls_enabled")
            }),
    ]);
    let real_orders_submitted = first_available_bool_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "real_orders_submitted")
            }),
        portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "real_orders_submitted")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "real_orders_submitted")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "real_orders_submitted")
            }),
    ]);
    let order_state_values_are_exchange_truth = first_available_bool_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "order_state_values_are_exchange_truth")
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "order_state_values_are_exchange_truth")
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "order_state_values_are_exchange_truth")
            }),
    ]);
    let shadow_values_are_exchange_truth = any_available_bool_from_values([
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field_with_fallback(
                    value,
                    "shadow_values_are_exchange_truth",
                    "values_are_exchange_truth",
                )
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field_with_fallback(
                    value,
                    "shadow_values_are_exchange_truth",
                    "values_are_exchange_truth",
                )
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field_with_fallback(
                    value,
                    "shadow_values_are_exchange_truth",
                    "values_are_exchange_truth",
                )
            }),
    ]);
    let portfolio_values_are_exchange_truth = first_available_bool_from_values([
        portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                nested_json_bool_field_with_fallback(
                    value,
                    "provenance",
                    "portfolio_values_are_exchange_truth",
                    "values_are_exchange_truth",
                )
            }),
        kill_switch_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field_with_fallback(
                    value,
                    "portfolio_values_are_exchange_truth",
                    "values_are_exchange_truth",
                )
            }),
        reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field_with_fallback(
                    value,
                    "portfolio_values_are_exchange_truth",
                    "values_are_exchange_truth",
                )
            }),
        shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field_with_fallback(
                    value,
                    "portfolio_values_are_exchange_truth",
                    "values_are_exchange_truth",
                )
            }),
    ]);
    let values_are_exchange_truth = any_available_bool_from_values([
        order_state_values_are_exchange_truth.clone(),
        shadow_values_are_exchange_truth.clone(),
        portfolio_values_are_exchange_truth.clone(),
    ]);
    let kill_switch_status = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let kill_switch_active = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "kill_switch_active")
        });
    let kill_switch_dry_run = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "kill_switch_dry_run")
        });
    let kill_switch_manual_approval_recorded = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "manual_approval_recorded")
        });
    let kill_switch_approval_state = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "approval_state")
        });
    let kill_switch_production_order_submission_allowed = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "production_order_submission_allowed")
        });
    let kill_switch_production_order_mutation_allowed = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "production_order_mutation_allowed")
        });
    let kill_switch_production_order_state_reads_allowed = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "production_order_state_reads_allowed")
        });
    let kill_switch_listen_key_lifecycle_allowed = kill_switch_approval
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "listen_key_lifecycle_allowed")
        });

    let boundary_violation = actual_submission_count.value.is_some_and(|value| value > 0)
        || production_order_submissions_attempted
            .value
            .is_some_and(|value| value > 0)
        || production_orders_submitted
            .value
            .is_some_and(|value| value > 0)
        || production_order_mutations_attempted
            .value
            .is_some_and(|value| value > 0)
        || production_order_state_reads_attempted
            .value
            .is_some_and(|value| value > 0)
        || listen_key_lifecycle_attempted
            .value
            .is_some_and(|value| value > 0)
        || automatic_correction_orders_submitted
            .value
            .is_some_and(|value| value > 0)
        || dashboard_order_controls_enabled.value == Some(true)
        || real_orders_submitted.value == Some(true)
        || shadow_values_are_exchange_truth.value == Some(true)
        || portfolio_values_are_exchange_truth.value == Some(true)
        || kill_switch_production_order_submission_allowed.value == Some(true)
        || kill_switch_production_order_mutation_allowed.value == Some(true)
        || kill_switch_production_order_state_reads_allowed.value == Some(true)
        || kill_switch_listen_key_lifecycle_allowed.value == Some(true)
        || artifact_audit.boundary_violation
        || manifest_audit.boundary_violation;

    Some(ProductionShadowStatus {
        node_id: record.node_id.clone(),
        health: if boundary_violation {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        },
        diagnostic: DashboardValue::available(if boundary_violation {
            artifact_audit
                .diagnostic
                .or(manifest_audit.diagnostic)
                .unwrap_or_else(|| "production_shadow_readonly_boundary_violation".to_string())
        } else {
            "production_shadow_readonly_artifacts_ok".to_string()
        }),
        artifact_version: DashboardValue::available(paths.version_label.to_string()),
        public_read_status: public_read_probe
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "status")
            }),
        public_read_endpoint_class: public_read_probe
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "endpoint_class")
            }),
        response_shape_status: response_shape.as_ref().map_or_else(
            || {
                account_snapshot
                    .as_ref()
                    .map_or_else(DashboardValue::unknown, |value| {
                        nested_json_string_field(value, "response_shape_summary", "status")
                    })
            },
            |value| json_string_field_with_fallback(value, "status", "response_shape"),
        ),
        response_shape_validated: response_shape.as_ref().map_or_else(
            || {
                account_snapshot
                    .as_ref()
                    .map_or_else(DashboardValue::unknown, |value| {
                        json_bool_field(value, "response_shape_validated")
                    })
            },
            |value| json_bool_field(value, "response_shape_validated"),
        ),
        manifest_status: manifest_audit.status,
        manifest_artifact_count: manifest_audit.artifact_count,
        account_snapshot_status: account_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field_with_fallback(value, "status", "snapshot_mode")
            }),
        account_snapshot_endpoint_class: account_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "endpoint_class")
            }),
        shadow_intent_status: shadow_intent
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field_with_fallback(value, "submission_status", "mode")
            }),
        shadow_intents_created: jsonl_record_count(&paths.shadow_intent_path),
        portfolio_snapshot_status: portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field_with_fallback(value, "status", "snapshot_mode")
            }),
        portfolio_exposure_status: portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                nested_json_string_field(value, "exposure", "status")
            }),
        portfolio_pnl_status: portfolio_snapshot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                nested_json_string_field(value, "pnl", "status")
            }),
        lifecycle_status: lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field_with_fallback(value, "next_state", "state")
            }),
        lifecycle_events_created: jsonl_record_count(&paths.lifecycle_path),
        shadow_strategy_session_status: shadow_strategy_session
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "state")
            }),
        shadow_strategy_session_heartbeats: jsonl_record_count_matching(
            &paths.shadow_strategy_session_path,
            "event_type",
            "shadow_strategy_session_heartbeat",
        ),
        reconciliation_status: reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field_with_fallback(value, "severity", "event_type")
            }),
        reconciliation_classification: reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "classification")
            }),
        reconciliation_recommended_action: reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "recommended_action")
            }),
        reconciliation_events_created: jsonl_record_count(&paths.reconciliation_path),
        kill_switch_status,
        kill_switch_active: kill_switch_active.clone(),
        kill_switch_dry_run,
        kill_switch_manual_approval_recorded,
        kill_switch_approval_state,
        kill_switch_production_order_submission_allowed,
        kill_switch_production_order_mutation_allowed,
        kill_switch_production_order_state_reads_allowed,
        kill_switch_listen_key_lifecycle_allowed,
        risk_halted: any_available_bool_from_values([
            kill_switch_active,
            reconciliation
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_bool_field(value, "risk_halted")
                }),
            portfolio_snapshot
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    nested_json_bool_field(value, "risk_summary", "risk_halted")
                }),
        ]),
        manual_review_required: any_available_bool_from_values([
            kill_switch_approval
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_bool_field(value, "manual_approval_required")
                }),
            reconciliation
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_bool_field(value, "manual_review_required")
                }),
        ]),
        new_orders_blocked: any_available_bool_from_values([
            kill_switch_approval
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_bool_field(value, "owner_approval_required_before_any_mutation")
                }),
            reconciliation
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_bool_field(value, "new_orders_blocked")
                }),
            portfolio_snapshot
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    nested_json_bool_field(value, "risk_summary", "new_orders_blocked")
                }),
        ]),
        actual_submission_count,
        production_order_submissions_attempted,
        production_orders_submitted,
        production_order_mutations_attempted,
        production_order_state_reads_attempted,
        listen_key_lifecycle_attempted,
        automatic_correction_orders_submitted,
        dashboard_order_controls_enabled,
        real_orders_submitted,
        order_state_values_are_exchange_truth,
        shadow_values_are_exchange_truth,
        portfolio_values_are_exchange_truth,
        values_are_exchange_truth,
        manifest_path: dashboard_path_if_exists(&paths.manifest_path),
        public_read_probe_path: dashboard_path_if_exists(&paths.public_read_probe_path),
        account_snapshot_path: dashboard_path_if_exists(&paths.account_snapshot_path),
        response_shape_path: dashboard_path_if_exists(&paths.response_shape_path),
        shadow_intent_path: dashboard_path_if_exists(&paths.shadow_intent_path),
        portfolio_snapshot_path: dashboard_path_if_exists(&paths.portfolio_snapshot_path),
        shadow_strategy_session_path: dashboard_path_if_exists(&paths.shadow_strategy_session_path),
        lifecycle_path: dashboard_path_if_exists(&paths.lifecycle_path),
        reconciliation_path: dashboard_path_if_exists(&paths.reconciliation_path),
        kill_switch_approval_artifact_path: dashboard_path_if_exists(
            &paths.kill_switch_approval_artifact_path,
        ),
    })
}

fn preflight_readiness_from_production_shadow(
    shadow: &ProductionShadowStatus,
) -> Option<PreflightReadinessStatus> {
    if shadow.artifact_version.value.as_deref() != Some("v0_13") {
        return None;
    }

    let owner_proof_pack_status = if dashboard_string_available(&shadow.public_read_status)
        || dashboard_string_available(&shadow.account_snapshot_status)
        || dashboard_string_available(&shadow.response_shape_status)
    {
        DashboardValue::available("owner_proof_pack_artifacts_observed".to_string())
    } else {
        DashboardValue::available("not_included_default_offline_preflight".to_string())
    };
    let bounded_shadow_preflight_status =
        if dashboard_string_available(&shadow.shadow_strategy_session_status)
            || dashboard_string_available(&shadow.lifecycle_status)
        {
            DashboardValue::available("bounded_shadow_preflight_artifacts_observed".to_string())
        } else {
            DashboardValue::available("bounded_shadow_preflight_contract_only".to_string())
        };
    let no_production_mutation_gate_status =
        if dashboard_bool_is_false(&shadow.kill_switch_production_order_submission_allowed)
            && dashboard_bool_is_false(&shadow.kill_switch_production_order_mutation_allowed)
            && dashboard_bool_is_false(&shadow.kill_switch_production_order_state_reads_allowed)
            && dashboard_bool_is_false(&shadow.kill_switch_listen_key_lifecycle_allowed)
            && dashboard_bool_is_false(&shadow.dashboard_order_controls_enabled)
            && dashboard_bool_is_false(&shadow.real_orders_submitted)
            && dashboard_bool_is_false(&shadow.shadow_values_are_exchange_truth)
            && dashboard_bool_is_false(&shadow.portfolio_values_are_exchange_truth)
        {
            DashboardValue::available("no_production_mutation_boundary_ok".to_string())
        } else {
            DashboardValue::available("no_production_mutation_boundary_violation".to_string())
        };
    let boundary_ok = no_production_mutation_gate_status.value.as_deref()
        == Some("no_production_mutation_boundary_ok")
        && shadow.health == HealthStatus::Healthy;

    Some(PreflightReadinessStatus {
        node_id: shadow.node_id.clone(),
        health: if boundary_ok {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        },
        readiness_status: DashboardValue::available(if boundary_ok {
            "v13_preflight_readiness_ok".to_string()
        } else {
            "v13_preflight_readiness_degraded".to_string()
        }),
        owner_proof_pack_status,
        kill_switch_artifact_status: shadow.kill_switch_status.clone(),
        bounded_shadow_preflight_status,
        decimal_boundary_status: DashboardValue::available(
            "decimal_boundary_contract_present".to_string(),
        ),
        no_production_mutation_gate_status,
        production_order_submission_allowed: shadow
            .kill_switch_production_order_submission_allowed
            .clone(),
        production_order_mutation_allowed: shadow
            .kill_switch_production_order_mutation_allowed
            .clone(),
        production_order_state_reads_allowed: shadow
            .kill_switch_production_order_state_reads_allowed
            .clone(),
        listen_key_lifecycle_allowed: shadow.kill_switch_listen_key_lifecycle_allowed.clone(),
        dashboard_order_controls_enabled: shadow.dashboard_order_controls_enabled.clone(),
        real_orders_submitted: shadow.real_orders_submitted.clone(),
        order_state_values_are_exchange_truth: shadow.order_state_values_are_exchange_truth.clone(),
        shadow_values_are_exchange_truth: shadow.shadow_values_are_exchange_truth.clone(),
        portfolio_values_are_exchange_truth: shadow.portfolio_values_are_exchange_truth.clone(),
        values_are_exchange_truth: shadow.values_are_exchange_truth.clone(),
        diagnostic: DashboardValue::available(if boundary_ok {
            "v13_preflight_readiness_ok".to_string()
        } else {
            "v13_preflight_readiness_degraded".to_string()
        }),
        evidence_source: shadow.kill_switch_approval_artifact_path.clone(),
    })
}

#[derive(Clone, Debug)]
struct LiveAlphaDryRunArtifactPaths {
    order_gate_path: PathBuf,
    risk_preflight_path: PathBuf,
    order_state_proof_path: PathBuf,
    manual_approval_lifecycle_path: PathBuf,
    request_preview_path: PathBuf,
    execution_dry_run_path: PathBuf,
    kill_switch_runtime_gate_path: PathBuf,
}

impl LiveAlphaDryRunArtifactPaths {
    fn v14(record: &SupervisorNodeRecord) -> Self {
        Self {
            order_gate_path: record
                .artifact_root
                .join(LIVE_ALPHA_DRY_RUN_ORDER_GATE_ARTIFACT_RELATIVE_PATH),
            risk_preflight_path: record
                .artifact_root
                .join(LIVE_ALPHA_RISK_PREFLIGHT_ARTIFACT_RELATIVE_PATH),
            order_state_proof_path: record
                .artifact_root
                .join(PRODUCTION_ORDER_STATE_READONLY_PROOF_ARTIFACT_RELATIVE_PATH),
            manual_approval_lifecycle_path: record
                .artifact_root
                .join(LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_ARTIFACT_RELATIVE_PATH),
            request_preview_path: record
                .artifact_root
                .join(LIVE_ALPHA_ORDER_REQUEST_PREVIEW_ARTIFACT_RELATIVE_PATH),
            execution_dry_run_path: record
                .artifact_root
                .join(LIVE_ALPHA_EXECUTION_DRY_RUN_ARTIFACT_RELATIVE_PATH),
            kill_switch_runtime_gate_path: record
                .artifact_root
                .join(LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_ARTIFACT_RELATIVE_PATH),
        }
    }

    fn has_any_artifact(&self) -> bool {
        self.order_gate_path.exists()
            || self.risk_preflight_path.exists()
            || self.order_state_proof_path.exists()
            || self.manual_approval_lifecycle_path.exists()
            || self.request_preview_path.exists()
            || self.execution_dry_run_path.exists()
            || self.kill_switch_runtime_gate_path.exists()
    }
}

fn live_alpha_dry_run_from_record(record: &SupervisorNodeRecord) -> Option<LiveAlphaDryRunStatus> {
    let paths = LiveAlphaDryRunArtifactPaths::v14(record);
    if !paths.has_any_artifact() {
        return None;
    }

    let order_gate = read_json_file_value(&paths.order_gate_path);
    let risk_preflight = read_json_file_value(&paths.risk_preflight_path);
    let order_state_proof = read_json_file_value(&paths.order_state_proof_path);
    let manual_approval_lifecycle = read_json_file_value(&paths.manual_approval_lifecycle_path);
    let request_preview = read_json_file_value(&paths.request_preview_path);
    let execution_dry_run = read_json_file_value(&paths.execution_dry_run_path);
    let kill_switch_runtime_gate = read_json_file_value(&paths.kill_switch_runtime_gate_path);
    let has_v15_artifact = manual_approval_lifecycle.is_some()
        || request_preview.is_some()
        || execution_dry_run.is_some()
        || kill_switch_runtime_gate.is_some();
    let gate_schema_ok = order_gate.as_ref().is_none_or(|_| {
        artifact_schema_matches(&order_gate, LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION)
    });
    let risk_schema_ok = risk_preflight.as_ref().is_none_or(|_| {
        artifact_schema_matches(&risk_preflight, LIVE_ALPHA_RISK_PREFLIGHT_SCHEMA_VERSION)
    });
    let order_state_schema_ok = order_state_proof.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &order_state_proof,
            PRODUCTION_ORDER_STATE_READONLY_PROOF_SCHEMA_VERSION,
        )
    });
    let manual_approval_schema_ok = manual_approval_lifecycle.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &manual_approval_lifecycle,
            LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_SCHEMA_VERSION,
        )
    });
    let request_preview_schema_ok = request_preview.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &request_preview,
            LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION,
        )
    });
    let execution_dry_run_schema_ok = execution_dry_run.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &execution_dry_run,
            LIVE_ALPHA_EXECUTION_DRY_RUN_SCHEMA_VERSION,
        )
    });
    let kill_switch_runtime_gate_schema_ok = kill_switch_runtime_gate.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &kill_switch_runtime_gate,
            LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION,
        )
    });
    let gate_status = order_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let gate_ready = order_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "dry_run_order_gate_ready")
        });
    let risk_preflight_status = risk_preflight
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let order_state_read_status = order_state_proof
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let order_state_endpoint = order_state_proof
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "endpoint")
        });
    let order_state_network_attempted = order_state_proof
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "network_attempted")
        });
    let order_state_read_attempted = order_state_proof
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "order_state_read_attempted")
        });
    let order_state_shape_validated = first_available_bool_from_values([
        order_state_proof
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "endpoint_shape_validated")
            }),
        order_state_proof
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "response_shape_validated")
            }),
    ]);
    let non_empty_order_state_observed = order_state_proof
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "non_empty_order_state_observed")
        });
    let order_lifecycle_readiness = order_state_proof
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "order_lifecycle_readiness")
        });
    let risk_decision = risk_preflight
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "risk_decision")
        });
    let execution_decision = first_available_string_from_values([
        execution_dry_run
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "execution_decision")
            }),
        risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "execution_decision")
            }),
    ]);
    let manual_approval_status = manual_approval_lifecycle
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let manual_approval_state = first_available_string_from_values([
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "manual_approval_lifecycle_state")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "approval_state")
            }),
    ]);
    let manual_approval_valid = first_available_bool_from_values([
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "manual_approval_lifecycle_valid")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "approval_lifecycle_valid")
            }),
    ]);
    let manual_approval_issues = first_available_string_from_values([
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "manual_approval_lifecycle_issues")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "approval_lifecycle_issues")
            }),
    ]);
    let manual_approval_recorded = manual_approval_lifecycle
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "manual_approval_recorded")
        });
    let manual_approval_one_time = first_available_bool_from_values([
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "manual_approval_one_time")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "one_time_approval")
            }),
    ]);
    let manual_approval_used = first_available_bool_from_values([
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "manual_approval_used")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "approval_used")
            }),
    ]);
    let manual_approval_expires_at_unix_ms = first_available_u64_from_values([
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "manual_approval_expires_at_unix_ms")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "expires_at_unix_ms")
            }),
    ]);
    let request_preview_status = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let request_preview_allowed = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "request_preview_allowed")
        });
    let request_preview_built = first_available_bool_from_values([
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "request_preview_built")
            }),
        kill_switch_runtime_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "request_preview_built")
            }),
        execution_dry_run
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "request_preview_built")
            }),
    ]);
    let request_sent = first_available_bool_from_values([
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "request_sent")
            }),
        kill_switch_runtime_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "request_sent")
            }),
        execution_dry_run
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "request_sent")
            }),
    ]);
    let request_method = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "request_method")
        });
    let request_target = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "request_target")
        });
    let endpoint_class = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "endpoint_class")
        });
    let endpoint_decision = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "endpoint_decision")
        });
    let query_shape_without_signature = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "query_shape_without_signature")
        });
    let signature_preflight = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "signature_preflight")
        });
    let secrets_redacted = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "secrets_redacted")
        });
    let signed_request_memory_only = request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "signed_request_memory_only")
        });
    let execution_dry_run_status = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let dry_run_execution_adapter_called = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "dry_run_execution_adapter_called")
        });
    let dry_run_execution_adapter_wrote_artifact = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "dry_run_execution_adapter_wrote_artifact")
        });
    let dry_run_adapter_artifact_only = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "dry_run_adapter_artifact_only")
        });
    let production_adapter_called = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "production_adapter_called")
        });
    let production_adapter_instantiated = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "production_adapter_instantiated")
        });
    let strategy_intent_recorded = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "strategy_intent_recorded")
        });
    let strategy_intent_reaches_risk_preflight = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "strategy_intent_reaches_risk_preflight")
        });
    let strategy_intent_reaches_dry_run_adapter = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "strategy_intent_reaches_dry_run_adapter")
        });
    let strategy_intent_reaches_production_adapter = execution_dry_run
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "strategy_intent_reaches_production_adapter")
        });
    let kill_switch_runtime_gate_status = kill_switch_runtime_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let runtime_gate_decision = kill_switch_runtime_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "runtime_gate_decision")
        });
    let runtime_gate_open = kill_switch_runtime_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "runtime_gate_open")
        });
    let runtime_gate_reasons = kill_switch_runtime_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_array_field(value, "runtime_gate_reasons")
        });
    let production_order_submission_allowed = first_available_bool_from_values([
        execution_dry_run
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_submission_allowed")
            }),
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_submission_allowed")
            }),
        kill_switch_runtime_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_submission_allowed")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_submission_allowed")
            }),
        risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_submission_allowed")
            }),
        order_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_submission_allowed")
            }),
    ]);
    let production_order_mutation_allowed = first_available_bool_from_values([
        execution_dry_run
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_mutation_allowed")
            }),
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_mutation_allowed")
            }),
        kill_switch_runtime_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_mutation_allowed")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_mutation_allowed")
            }),
        risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_mutation_allowed")
            }),
        order_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_mutation_allowed")
            }),
    ]);
    let production_order_state_reads_allowed = first_available_bool_from_values([
        execution_dry_run
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_state_reads_allowed")
            }),
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_state_reads_allowed")
            }),
        kill_switch_runtime_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_state_reads_allowed")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_state_reads_allowed")
            }),
        risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_state_reads_allowed")
            }),
        order_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "production_order_state_reads_allowed")
            }),
    ]);
    let listen_key_lifecycle_allowed = first_available_bool_from_values([
        execution_dry_run
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "listen_key_lifecycle_allowed")
            }),
        request_preview
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "listen_key_lifecycle_allowed")
            }),
        kill_switch_runtime_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "listen_key_lifecycle_allowed")
            }),
        manual_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "listen_key_lifecycle_allowed")
            }),
        risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "listen_key_lifecycle_allowed")
            }),
        order_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "listen_key_lifecycle_allowed")
            }),
    ]);
    let live_alpha_artifacts = [
        &execution_dry_run,
        &request_preview,
        &kill_switch_runtime_gate,
        &manual_approval_lifecycle,
        &risk_preflight,
        &order_gate,
    ];
    let production_order_submissions_attempted = live_alpha_first_available_u64_any(
        &live_alpha_artifacts,
        "production_order_submissions_attempted",
    );
    let production_orders_submitted =
        live_alpha_first_available_u64_any(&live_alpha_artifacts, "production_orders_submitted");
    let production_order_mutations_attempted = live_alpha_first_available_u64_any(
        &live_alpha_artifacts,
        "production_order_mutations_attempted",
    );
    let production_order_state_reads_attempted = live_alpha_first_available_u64_any(
        &live_alpha_artifacts,
        "production_order_state_reads_attempted",
    );
    let listen_key_lifecycle_attempted =
        live_alpha_first_available_u64_any(&live_alpha_artifacts, "listen_key_lifecycle_attempted");
    let actual_submission_count =
        live_alpha_first_available_u64_any(&live_alpha_artifacts, "actual_submission_count");
    let automatic_correction_orders_submitted = live_alpha_first_available_u64_any(
        &live_alpha_artifacts,
        "automatic_correction_orders_submitted",
    );
    let cancel_replace_amend_attempted = live_alpha_first_available_bool_any(
        &live_alpha_artifacts,
        "cancel_replace_amend_attempted",
    );
    let order_endpoint_access_attempted = live_alpha_first_available_bool_any(
        &live_alpha_artifacts,
        "order_endpoint_access_attempted",
    );
    let execution_adapter_called = live_alpha_first_available_bool_any(
        &[
            &risk_preflight,
            &order_gate,
            &request_preview,
            &kill_switch_runtime_gate,
        ],
        "execution_adapter_called",
    );
    let matching_engine_submission =
        live_alpha_first_available_bool_any(&live_alpha_artifacts, "matching_engine_submission");
    let dashboard_order_controls_enabled = live_alpha_any_available_bool_any(
        &live_alpha_artifacts,
        "dashboard_order_controls_enabled",
    );
    let network_attempted =
        live_alpha_first_available_bool_any(&live_alpha_artifacts, "network_attempted");
    let real_orders_submitted =
        live_alpha_first_available_bool_any(&live_alpha_artifacts, "real_orders_submitted");
    let real_funds = live_alpha_first_available_bool_any(&live_alpha_artifacts, "real_funds");
    let production_trading_enabled =
        live_alpha_first_available_bool_any(&live_alpha_artifacts, "production_trading_enabled");
    let order_state_values_are_exchange_truth = first_available_bool_from_values([
        order_state_proof
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "order_state_values_are_exchange_truth")
            }),
        live_alpha_first_available_bool_any(
            &live_alpha_artifacts,
            "order_state_values_are_exchange_truth",
        ),
    ]);
    let shadow_values_are_exchange_truth = live_alpha_first_available_bool_any(
        &live_alpha_artifacts,
        "shadow_values_are_exchange_truth",
    );
    let portfolio_values_are_exchange_truth = live_alpha_first_available_bool_any(
        &live_alpha_artifacts,
        "portfolio_values_are_exchange_truth",
    );
    let legacy_values_are_exchange_truth =
        live_alpha_first_available_bool_any(&live_alpha_artifacts, "values_are_exchange_truth");
    let values_are_exchange_truth = any_available_bool_from_values([
        order_state_values_are_exchange_truth.clone(),
        shadow_values_are_exchange_truth.clone(),
        portfolio_values_are_exchange_truth.clone(),
        legacy_values_are_exchange_truth,
    ]);
    let order_state_proof_boundary_violation =
        order_state_readonly_proof_boundary_violation(&order_state_proof, order_state_schema_ok);
    let live_alpha_v15_boundary_violation = live_alpha_v15_boundary_violation(
        &manual_approval_lifecycle,
        &request_preview,
        &execution_dry_run,
        &kill_switch_runtime_gate,
    );

    let boundary_violation = !gate_schema_ok
        || !risk_schema_ok
        || !order_state_schema_ok
        || !manual_approval_schema_ok
        || !request_preview_schema_ok
        || !execution_dry_run_schema_ok
        || !kill_switch_runtime_gate_schema_ok
        || order_state_proof_boundary_violation
        || live_alpha_v15_boundary_violation
        || dashboard_u64_gt_zero(&production_order_submissions_attempted)
        || dashboard_u64_gt_zero(&production_orders_submitted)
        || dashboard_u64_gt_zero(&production_order_mutations_attempted)
        || dashboard_u64_gt_zero(&production_order_state_reads_attempted)
        || dashboard_u64_gt_zero(&listen_key_lifecycle_attempted)
        || dashboard_u64_gt_zero(&actual_submission_count)
        || dashboard_u64_gt_zero(&automatic_correction_orders_submitted)
        || production_order_submission_allowed.value == Some(true)
        || production_order_mutation_allowed.value == Some(true)
        || production_order_state_reads_allowed.value == Some(true)
        || listen_key_lifecycle_allowed.value == Some(true)
        || cancel_replace_amend_attempted.value == Some(true)
        || order_endpoint_access_attempted.value == Some(true)
        || execution_adapter_called.value == Some(true)
        || matching_engine_submission.value == Some(true)
        || dashboard_order_controls_enabled.value == Some(true)
        || network_attempted.value == Some(true)
        || real_orders_submitted.value == Some(true)
        || real_funds.value == Some(true)
        || production_trading_enabled.value == Some(true)
        || shadow_values_are_exchange_truth.value == Some(true)
        || portfolio_values_are_exchange_truth.value == Some(true);
    let ready_v14 = gate_ready.value == Some(true)
        && risk_decision.value.as_deref() == Some("dry_run_approved")
        && execution_decision.value.as_deref() == Some("blocked_no_production_mutation");
    let ready_v15 = manual_approval_valid.value == Some(true)
        && request_preview_built.value == Some(true)
        && request_sent.value == Some(false)
        && runtime_gate_open.value == Some(true)
        && dry_run_execution_adapter_called.value == Some(true)
        && production_adapter_called.value == Some(false)
        && production_adapter_instantiated.value == Some(false)
        && strategy_intent_reaches_production_adapter.value == Some(false);
    let ready = if has_v15_artifact {
        ready_v15
    } else {
        ready_v14
    };
    let readiness_status = if boundary_violation {
        if has_v15_artifact {
            "live_alpha_mutation_preflight_boundary_violation"
        } else {
            "live_alpha_dry_run_boundary_violation"
        }
    } else if ready {
        if has_v15_artifact {
            "live_alpha_mutation_preflight_ready_for_owner_review"
        } else {
            "live_alpha_dry_run_ready"
        }
    } else {
        if has_v15_artifact {
            "live_alpha_mutation_preflight_blocked"
        } else {
            "live_alpha_dry_run_blocked"
        }
    };

    Some(LiveAlphaDryRunStatus {
        node_id: record.node_id.clone(),
        health: if boundary_violation || !ready {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        },
        readiness_status: DashboardValue::available(readiness_status.to_string()),
        diagnostic: live_alpha_diagnostic(
            &order_gate,
            &risk_preflight,
            &order_state_proof,
            LiveAlphaSchemaHealth {
                gate_schema_ok,
                risk_schema_ok,
                order_state_schema_ok,
                manual_approval_schema_ok,
                request_preview_schema_ok,
                execution_dry_run_schema_ok,
                kill_switch_runtime_gate_schema_ok,
            },
            boundary_violation,
            readiness_status,
        ),
        gate_status,
        gate_ready,
        missing_gate_flags: live_alpha_missing_gate_flags(&order_gate, &risk_preflight),
        dry_run_order_intent_recorded: order_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "dry_run_order_intent_recorded")
            }),
        order_submission_mode: order_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "order_submission_mode")
            }),
        risk_preflight_status,
        risk_decision,
        execution_decision,
        risk_reasons: risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "reasons")
            }),
        kill_switch_active: first_available_bool_from_values([
            kill_switch_runtime_gate
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_bool_field(value, "kill_switch_active")
                }),
            risk_preflight
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_bool_field(value, "kill_switch_active")
                }),
        ]),
        manual_approval_status,
        manual_approval_state,
        manual_approval_valid,
        manual_approval_issues,
        manual_approval_recorded,
        manual_approval_one_time,
        manual_approval_used,
        manual_approval_expires_at_unix_ms,
        request_preview_status,
        request_preview_allowed,
        request_preview_built,
        request_sent,
        request_method,
        request_target,
        endpoint_class,
        endpoint_decision,
        query_shape_without_signature,
        signature_preflight,
        secrets_redacted,
        signed_request_memory_only,
        execution_dry_run_status,
        dry_run_execution_adapter_called,
        dry_run_execution_adapter_wrote_artifact,
        dry_run_adapter_artifact_only,
        production_adapter_called,
        production_adapter_instantiated,
        strategy_intent_recorded,
        strategy_intent_reaches_risk_preflight,
        strategy_intent_reaches_dry_run_adapter,
        strategy_intent_reaches_production_adapter,
        kill_switch_runtime_gate_status,
        runtime_gate_decision,
        runtime_gate_open,
        runtime_gate_reasons,
        order_state_readable: first_available_bool_from_values([
            order_state_shape_validated.clone(),
            risk_preflight
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_bool_field(value, "order_state_readable")
                }),
        ]),
        order_state_read_status,
        order_state_endpoint,
        order_state_network_attempted,
        order_state_read_attempted,
        order_state_shape_validated,
        order_state_age_ms: risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "order_state_age_ms")
            }),
        max_order_state_age_ms: risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "max_order_state_age_ms")
            }),
        open_order_count: first_available_u64_from_values([
            order_state_proof
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_u64_field(value, "order_entries_observed")
                }),
            risk_preflight
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_u64_field(value, "open_order_count")
                }),
        ]),
        max_open_orders: risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, "max_open_orders")
            }),
        non_empty_order_state_observed,
        order_lifecycle_readiness,
        order_state_truth_source: live_alpha_order_state_truth_source(
            &order_state_proof,
            &risk_preflight,
        ),
        reconciliation_status: risk_preflight
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "status")
            }),
        production_order_submission_allowed,
        production_order_mutation_allowed,
        production_order_state_reads_allowed,
        listen_key_lifecycle_allowed,
        production_order_submissions_attempted,
        production_orders_submitted,
        production_order_mutations_attempted,
        production_order_state_reads_attempted,
        listen_key_lifecycle_attempted,
        cancel_replace_amend_attempted,
        order_endpoint_access_attempted,
        execution_adapter_called,
        matching_engine_submission,
        actual_submission_count,
        automatic_correction_orders_submitted,
        dashboard_order_controls_enabled,
        network_attempted,
        real_orders_submitted,
        real_funds,
        production_trading_enabled,
        order_state_values_are_exchange_truth,
        shadow_values_are_exchange_truth,
        portfolio_values_are_exchange_truth,
        values_are_exchange_truth,
        order_gate_path: dashboard_path_if_exists(&paths.order_gate_path),
        risk_preflight_path: dashboard_path_if_exists(&paths.risk_preflight_path),
        order_state_proof_path: dashboard_path_if_exists(&paths.order_state_proof_path),
        manual_approval_lifecycle_path: dashboard_path_if_exists(
            &paths.manual_approval_lifecycle_path,
        ),
        request_preview_path: dashboard_path_if_exists(&paths.request_preview_path),
        execution_dry_run_path: dashboard_path_if_exists(&paths.execution_dry_run_path),
        kill_switch_runtime_gate_path: dashboard_path_if_exists(
            &paths.kill_switch_runtime_gate_path,
        ),
    })
}

#[derive(Clone, Debug)]
struct ProductionMutationEvidenceArtifactPaths {
    runtime_gate_path: PathBuf,
    signing_approval_path: PathBuf,
    request_builder_path: PathBuf,
    guarded_send_path: PathBuf,
    response_redaction_path: PathBuf,
    order_state_readback_path: PathBuf,
    audit_trail_path: PathBuf,
    failure_semantics_path: PathBuf,
}

impl ProductionMutationEvidenceArtifactPaths {
    fn v16(record: &SupervisorNodeRecord) -> Self {
        Self {
            runtime_gate_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_RUNTIME_GATE_ARTIFACT_RELATIVE_PATH),
            signing_approval_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_SIGNING_APPROVAL_ARTIFACT_RELATIVE_PATH),
            request_builder_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_REQUEST_BUILDER_ARTIFACT_RELATIVE_PATH),
            guarded_send_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_GUARDED_SEND_ARTIFACT_RELATIVE_PATH),
            response_redaction_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_RESPONSE_REDACTION_ARTIFACT_RELATIVE_PATH),
            order_state_readback_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_ORDER_STATE_READBACK_ARTIFACT_RELATIVE_PATH),
            audit_trail_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_AUDIT_TRAIL_ARTIFACT_RELATIVE_PATH),
            failure_semantics_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_FAILURE_SEMANTICS_ARTIFACT_RELATIVE_PATH),
        }
    }

    fn has_any_artifact(&self) -> bool {
        self.runtime_gate_path.exists()
            || self.signing_approval_path.exists()
            || self.request_builder_path.exists()
            || self.guarded_send_path.exists()
            || self.response_redaction_path.exists()
            || self.order_state_readback_path.exists()
            || self.audit_trail_path.exists()
            || self.failure_semantics_path.exists()
    }
}

fn production_mutation_evidence_from_record(
    record: &SupervisorNodeRecord,
) -> Option<ProductionMutationEvidenceStatus> {
    let paths = ProductionMutationEvidenceArtifactPaths::v16(record);
    if !paths.has_any_artifact() {
        return None;
    }

    let runtime_gate = read_json_file_value(&paths.runtime_gate_path);
    let signing_approval = read_json_file_value(&paths.signing_approval_path);
    let request_builder = read_json_file_value(&paths.request_builder_path);
    let guarded_send = read_json_file_value(&paths.guarded_send_path);
    let response_redaction = read_json_file_value(&paths.response_redaction_path);
    let order_state_readback = read_json_file_value(&paths.order_state_readback_path);
    let audit_trail = read_json_file_value(&paths.audit_trail_path);
    let failure_semantics = read_json_file_value(&paths.failure_semantics_path);
    let artifacts = [
        &runtime_gate,
        &signing_approval,
        &request_builder,
        &guarded_send,
        &response_redaction,
        &order_state_readback,
        &audit_trail,
        &failure_semantics,
    ];

    let schema_ok = runtime_gate.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &runtime_gate,
            PRODUCTION_MUTATION_RUNTIME_GATE_SCHEMA_VERSION,
        )
    }) && signing_approval.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &signing_approval,
            PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION,
        )
    }) && request_builder.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &request_builder,
            PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION,
        )
    }) && guarded_send.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &guarded_send,
            PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION,
        )
    }) && response_redaction.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &response_redaction,
            PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION,
        )
    }) && order_state_readback.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &order_state_readback,
            PRODUCTION_MUTATION_ORDER_STATE_READBACK_SCHEMA_VERSION,
        )
    }) && audit_trail.as_ref().is_none_or(|_| {
        artifact_schema_matches(&audit_trail, PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION)
    }) && failure_semantics.as_ref().is_none_or(|_| {
        artifact_schema_matches(
            &failure_semantics,
            PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION,
        )
    });

    let runtime_gate_status = runtime_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let runtime_gate_open = first_available_bool_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "runtime_gate_open")
            }),
        runtime_gate
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "runtime_gate_open")
            }),
    ]);
    let signing_approval_status = first_available_string_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "signing_approval_status")
            }),
        signing_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "status")
            }),
    ]);
    let approval_state = first_available_string_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "approval_state")
            }),
        signing_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "approval_state")
            }),
    ]);
    let manual_approval_recorded = first_available_bool_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "manual_approval_recorded")
            }),
        signing_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "manual_approval_recorded")
            }),
    ]);
    let approved_by = first_available_string_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "approved_by")
            }),
        signing_approval
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "approved_by")
            }),
    ]);
    let request_builder_status = request_builder
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let request_builder_ready = request_builder
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "request_builder_ready")
        });
    let guarded_send_status = first_available_string_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "guarded_send_status")
            }),
        guarded_send
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "status")
            }),
    ]);
    let guarded_send_ready = guarded_send
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "guarded_send_ready")
        });
    let request_sent = v16_any_available_bool_any(&artifacts, "request_sent");
    let network_attempted = v16_any_available_bool_any(&artifacts, "network_attempted");
    let kill_switch_checked_before_send =
        v16_any_available_bool_any(&artifacts, "kill_switch_checked_before_send");
    let kill_switch_checked_after_send =
        v16_any_available_bool_any(&artifacts, "kill_switch_checked_after_send");
    let response_redaction_status = first_available_string_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "response_redaction_status")
            }),
        response_redaction
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "status")
            }),
    ]);
    let response_redaction_ready = first_available_bool_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "response_redaction_ready")
            }),
        response_redaction
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "response_redaction_ready")
            }),
    ]);
    let order_state_readback_status = first_available_string_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "order_state_readback_status")
            }),
        order_state_readback
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "status")
            }),
    ]);
    let readback_contract_ready = first_available_bool_from_values([
        audit_trail
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "readback_contract_ready")
            }),
        order_state_readback
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "readback_contract_ready")
            }),
    ]);
    let order_state_read_attempted =
        v16_any_available_bool_any(&artifacts, "order_state_read_attempted");
    let response_shape_validated = first_available_bool_from_values([
        order_state_readback
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "response_shape_validated")
            }),
        order_state_readback
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "endpoint_shape_validated")
            }),
    ]);
    let audit_trail_status = audit_trail
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let audit_trail_ready = audit_trail
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "audit_trail_ready")
        });
    let failure_semantics_status = failure_semantics
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let failure_semantics_ready = failure_semantics
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "failure_semantics_ready")
        });
    let failure_mode = failure_semantics
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "failure_mode")
        });
    let terminal_action = failure_semantics
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "terminal_action")
        });
    let strategy_continuation_allowed =
        v16_any_available_bool_any(&artifacts, "strategy_continuation_allowed");
    let symbol = v16_first_available_string_any(&artifacts, "symbol");
    let side = v16_first_available_string_any(&artifacts, "side");
    let order_type = v16_first_available_string_any(&artifacts, "order_type");
    let time_in_force = v16_first_available_string_any(&artifacts, "time_in_force");
    let quantity = v16_first_available_string_any(&artifacts, "quantity");
    let price = v16_first_available_string_any(&artifacts, "price");
    let order_id = v16_first_available_string_any(&artifacts, "order_id");
    let production_order_submissions_attempted =
        v16_max_available_u64_any(&artifacts, "production_order_submissions_attempted");
    let production_orders_submitted =
        v16_max_available_u64_any(&artifacts, "production_orders_submitted");
    let production_order_mutations_attempted =
        v16_max_available_u64_any(&artifacts, "production_order_mutations_attempted");
    let production_order_state_reads_attempted =
        v16_max_available_u64_any(&artifacts, "production_order_state_reads_attempted");
    let listen_key_lifecycle_attempted =
        v16_max_available_u64_any(&artifacts, "listen_key_lifecycle_attempted");
    let retry_attempted = v16_any_available_bool_any(&artifacts, "retry_attempted");
    let cancel_attempted = v16_any_available_bool_any(&artifacts, "cancel_attempted");
    let replace_attempted = v16_any_available_bool_any(&artifacts, "replace_attempted");
    let amend_attempted = v16_any_available_bool_any(&artifacts, "amend_attempted");
    let correction_attempted = v16_any_available_bool_any(&artifacts, "correction_attempted");
    let flatten_attempted = v16_any_available_bool_any(&artifacts, "flatten_attempted");
    let remediation_attempted = v16_any_available_bool_any(&artifacts, "remediation_attempted");
    let dashboard_order_controls_enabled =
        v16_any_available_bool_any(&artifacts, "dashboard_order_controls_enabled");
    let real_orders_submitted = v16_any_available_bool_any(&artifacts, "real_orders_submitted");
    let production_trading_enabled =
        v16_any_available_bool_any(&artifacts, "production_trading_enabled");
    let boundary_violation = !schema_ok
        || dashboard_u64_gt(&production_order_submissions_attempted, 1)
        || dashboard_u64_gt(&production_orders_submitted, 1)
        || dashboard_u64_gt(&production_order_mutations_attempted, 1)
        || dashboard_u64_gt(&production_order_state_reads_attempted, 1)
        || dashboard_u64_gt_zero(&listen_key_lifecycle_attempted)
        || retry_attempted.value == Some(true)
        || cancel_attempted.value == Some(true)
        || replace_attempted.value == Some(true)
        || amend_attempted.value == Some(true)
        || correction_attempted.value == Some(true)
        || flatten_attempted.value == Some(true)
        || remediation_attempted.value == Some(true)
        || dashboard_order_controls_enabled.value == Some(true)
        || real_orders_submitted.value == Some(true)
        || production_trading_enabled.value == Some(true)
        || strategy_continuation_allowed.value == Some(true);
    let ready = audit_trail_ready.value == Some(true)
        && failure_semantics_ready.value == Some(true)
        && !boundary_violation;
    let readiness_status = if boundary_violation {
        "production_mutation_evidence_boundary_violation"
    } else if ready {
        "production_mutation_evidence_ready_for_owner_review"
    } else {
        "production_mutation_evidence_blocked"
    };

    Some(ProductionMutationEvidenceStatus {
        node_id: record.node_id.clone(),
        health: if ready {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        },
        readiness_status: DashboardValue::available(readiness_status.to_string()),
        diagnostic: DashboardValue::available(if !schema_ok {
            "production_mutation_evidence_schema_invalid".to_string()
        } else if boundary_violation {
            "production_mutation_evidence_readonly_boundary_violation".to_string()
        } else {
            readiness_status.to_string()
        }),
        runtime_gate_status,
        runtime_gate_open,
        signing_approval_status,
        approval_state,
        manual_approval_recorded,
        approved_by,
        request_builder_status,
        request_builder_ready,
        guarded_send_status,
        guarded_send_ready,
        request_sent,
        network_attempted,
        kill_switch_checked_before_send,
        kill_switch_checked_after_send,
        response_redaction_status,
        response_redaction_ready,
        order_state_readback_status,
        readback_contract_ready,
        order_state_read_attempted,
        response_shape_validated,
        audit_trail_status,
        audit_trail_ready,
        failure_semantics_status,
        failure_semantics_ready,
        failure_mode,
        terminal_action,
        strategy_continuation_allowed,
        symbol,
        side,
        order_type,
        time_in_force,
        quantity,
        price,
        order_id,
        production_order_submissions_attempted,
        production_orders_submitted,
        production_order_mutations_attempted,
        production_order_state_reads_attempted,
        listen_key_lifecycle_attempted,
        retry_attempted,
        cancel_attempted,
        replace_attempted,
        amend_attempted,
        correction_attempted,
        flatten_attempted,
        remediation_attempted,
        dashboard_order_controls_enabled,
        real_orders_submitted,
        production_trading_enabled,
        runtime_gate_path: dashboard_path_if_exists(&paths.runtime_gate_path),
        signing_approval_path: dashboard_path_if_exists(&paths.signing_approval_path),
        request_builder_path: dashboard_path_if_exists(&paths.request_builder_path),
        guarded_send_path: dashboard_path_if_exists(&paths.guarded_send_path),
        response_redaction_path: dashboard_path_if_exists(&paths.response_redaction_path),
        order_state_readback_path: dashboard_path_if_exists(&paths.order_state_readback_path),
        audit_trail_path: dashboard_path_if_exists(&paths.audit_trail_path),
        failure_semantics_path: dashboard_path_if_exists(&paths.failure_semantics_path),
    })
}

#[derive(Clone, Debug)]
struct ProductionReconciliationOrphanArtifactPaths {
    local_order_ledger_path: PathBuf,
    exchange_readback_mapper_path: PathBuf,
    reconciliation_classifier_path: PathBuf,
    orphan_order_detector_path: PathBuf,
}

impl ProductionReconciliationOrphanArtifactPaths {
    fn v17(record: &SupervisorNodeRecord) -> Self {
        Self {
            local_order_ledger_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_ARTIFACT_RELATIVE_PATH),
            exchange_readback_mapper_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_ARTIFACT_RELATIVE_PATH),
            reconciliation_classifier_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_ARTIFACT_RELATIVE_PATH),
            orphan_order_detector_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_ARTIFACT_RELATIVE_PATH),
        }
    }

    fn has_any_artifact(&self) -> bool {
        self.local_order_ledger_path.exists()
            || self.exchange_readback_mapper_path.exists()
            || self.reconciliation_classifier_path.exists()
            || self.orphan_order_detector_path.exists()
    }
}

fn production_reconciliation_orphan_from_record(
    record: &SupervisorNodeRecord,
) -> Option<ProductionReconciliationOrphanStatus> {
    let paths = ProductionReconciliationOrphanArtifactPaths::v17(record);
    if !paths.has_any_artifact() {
        return None;
    }

    let local_order_ledger = read_json_file_value(&paths.local_order_ledger_path);
    let exchange_readback_mapper = read_json_file_value(&paths.exchange_readback_mapper_path);
    let reconciliation_classifier = read_json_file_value(&paths.reconciliation_classifier_path);
    let orphan_order_detector = read_json_file_value(&paths.orphan_order_detector_path);
    let artifacts = [
        &local_order_ledger,
        &exchange_readback_mapper,
        &reconciliation_classifier,
        &orphan_order_detector,
    ];
    let artifact_specs = [
        (
            "local_order_ledger",
            paths.local_order_ledger_path.as_path(),
            &local_order_ledger,
            PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_SCHEMA_VERSION,
        ),
        (
            "exchange_readback_mapper",
            paths.exchange_readback_mapper_path.as_path(),
            &exchange_readback_mapper,
            PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION,
        ),
        (
            "reconciliation_classifier",
            paths.reconciliation_classifier_path.as_path(),
            &reconciliation_classifier,
            PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION,
        ),
        (
            "orphan_order_detector",
            paths.orphan_order_detector_path.as_path(),
            &orphan_order_detector,
            PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_SCHEMA_VERSION,
        ),
    ];
    let missing_artifacts = v17_missing_artifact_diagnostics(&artifact_specs);
    let schema_diagnostics = v17_schema_diagnostics(&artifact_specs);
    let provenance_diagnostics = v17_provenance_diagnostics(&artifact_specs);
    let stale_artifacts = v17_stale_artifact_diagnostics(&artifact_specs);
    let schema_ok = schema_diagnostics.is_empty();
    let provenance_ok = provenance_diagnostics.is_empty();

    let order_lineage_id = v17_first_available_string_any(&artifacts, "order_lineage_id");
    let local_ledger_status = local_order_ledger
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let local_order_state = local_order_ledger
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "current_local_state")
        });
    let local_ledger_ready = local_order_ledger
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_bool_field(value, "local_ledger_ready")
        });
    let restart_readable = v17_any_available_bool_any(&artifacts, "restart_readable");
    let exchange_readback_status = exchange_readback_mapper
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let exchange_readback_mapped =
        v17_any_available_bool_any(&artifacts, "exchange_readback_mapped");
    let exchange_order_state = exchange_readback_mapper
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "exchange_order_state")
        });
    let exchange_order_status = exchange_readback_mapper
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "exchange_order_status")
        });
    let open_order_observed = v17_any_available_bool_any(&artifacts, "open_order_observed");
    let terminal_state_observed = v17_any_available_bool_any(&artifacts, "terminal_state_observed");
    let reconciliation_status = reconciliation_classifier
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let reconciliation_classified =
        v17_any_available_bool_any(&artifacts, "reconciliation_classified");
    let reconciliation_outcome = reconciliation_classifier
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "reconciliation_outcome")
        });
    let orphan_status = orphan_order_detector
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let orphan_detection_completed =
        v17_any_available_bool_any(&artifacts, "orphan_detection_completed");
    let orphan_detection_outcome = orphan_order_detector
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "orphan_detection_outcome")
        });
    let orphan_risk_detected = v17_any_available_bool_any(&artifacts, "orphan_risk_detected");
    let risk_halted = v17_any_available_bool_any(&artifacts, "risk_halted");
    let manual_review_required = v17_any_available_bool_any(&artifacts, "manual_review_required");
    let new_orders_blocked = v17_any_available_bool_any(&artifacts, "new_orders_blocked");
    let stale_ledger_restart_required =
        v17_any_available_bool_any(&artifacts, "stale_ledger_restart_required");
    let duplicate_submit_attempted =
        v17_any_available_bool_any(&artifacts, "duplicate_submit_attempted");
    let retry_attempted = v17_any_available_bool_any(&artifacts, "retry_attempted");
    let cancel_attempted = v17_any_available_bool_any(&artifacts, "cancel_attempted");
    let remediation_attempted = v17_any_available_bool_any(&artifacts, "remediation_attempted");
    let automatic_cancel_allowed =
        v17_any_available_bool_any(&artifacts, "automatic_cancel_allowed");
    let automatic_remediation_allowed =
        v17_any_available_bool_any(&artifacts, "automatic_remediation_allowed");
    let dashboard_order_controls_enabled =
        v17_any_available_bool_any(&artifacts, "dashboard_order_controls_enabled");
    let dashboard_cancel_controls_enabled =
        v17_any_available_bool_any(&artifacts, "dashboard_cancel_controls_enabled");
    let network_attempted = v17_any_available_bool_any(&artifacts, "network_attempted");
    let production_order_submission_allowed =
        v17_any_available_bool_any(&artifacts, "production_order_submission_allowed");
    let production_order_mutation_allowed =
        v17_any_available_bool_any(&artifacts, "production_order_mutation_allowed");

    let boundary_violation = !schema_ok
        || !provenance_ok
        || duplicate_submit_attempted.value == Some(true)
        || retry_attempted.value == Some(true)
        || cancel_attempted.value == Some(true)
        || remediation_attempted.value == Some(true)
        || automatic_cancel_allowed.value == Some(true)
        || automatic_remediation_allowed.value == Some(true)
        || dashboard_order_controls_enabled.value == Some(true)
        || dashboard_cancel_controls_enabled.value == Some(true)
        || network_attempted.value == Some(true)
        || production_order_submission_allowed.value == Some(true)
        || production_order_mutation_allowed.value == Some(true);
    let review_required = !stale_artifacts.is_empty()
        || orphan_risk_detected.value == Some(true)
        || risk_halted.value == Some(true)
        || manual_review_required.value == Some(true)
        || new_orders_blocked.value == Some(true)
        || stale_ledger_restart_required.value == Some(true);
    let ready = missing_artifacts.is_empty()
        && local_ledger_ready.value == Some(true)
        && exchange_readback_mapped.value == Some(true)
        && reconciliation_classified.value == Some(true)
        && orphan_detection_completed.value == Some(true)
        && !boundary_violation;
    let readiness_status = if boundary_violation {
        "production_reconciliation_orphan_boundary_violation"
    } else if review_required {
        "production_reconciliation_orphan_manual_review_required"
    } else if ready {
        "production_reconciliation_orphan_ready"
    } else {
        "production_reconciliation_orphan_incomplete"
    };

    Some(ProductionReconciliationOrphanStatus {
        node_id: record.node_id.clone(),
        health: if boundary_violation || review_required || !ready {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        },
        readiness_status: DashboardValue::available(readiness_status.to_string()),
        diagnostic: DashboardValue::available(v17_reconciliation_orphan_diagnostic(
            readiness_status,
            &missing_artifacts,
            &schema_diagnostics,
            &provenance_diagnostics,
            &stale_artifacts,
        )),
        missing_artifacts: diagnostic_value(&missing_artifacts),
        schema_diagnostics: diagnostic_value(&schema_diagnostics),
        provenance_diagnostics: diagnostic_value(&provenance_diagnostics),
        stale_artifacts: diagnostic_value(&stale_artifacts),
        order_lineage_id,
        local_ledger_status,
        local_order_state,
        local_ledger_ready,
        restart_readable,
        exchange_readback_status,
        exchange_readback_mapped,
        exchange_order_state,
        exchange_order_status,
        open_order_observed,
        terminal_state_observed,
        reconciliation_status,
        reconciliation_classified,
        reconciliation_outcome,
        orphan_status,
        orphan_detection_completed,
        orphan_detection_outcome,
        orphan_risk_detected,
        risk_halted,
        manual_review_required,
        new_orders_blocked,
        stale_ledger_restart_required,
        duplicate_submit_attempted,
        retry_attempted,
        cancel_attempted,
        remediation_attempted,
        automatic_cancel_allowed,
        automatic_remediation_allowed,
        dashboard_order_controls_enabled,
        dashboard_cancel_controls_enabled,
        network_attempted,
        production_order_submission_allowed,
        production_order_mutation_allowed,
        local_order_ledger_path: dashboard_path_if_exists(&paths.local_order_ledger_path),
        exchange_readback_mapper_path: dashboard_path_if_exists(
            &paths.exchange_readback_mapper_path,
        ),
        reconciliation_classifier_path: dashboard_path_if_exists(
            &paths.reconciliation_classifier_path,
        ),
        orphan_order_detector_path: dashboard_path_if_exists(&paths.orphan_order_detector_path),
    })
}

#[derive(Clone, Debug)]
struct ProductionCancelRecoveryArtifactPaths {
    cancel_request_preview_path: PathBuf,
    cancel_risk_gate_path: PathBuf,
    manual_owner_approval_lifecycle_path: PathBuf,
    cancel_response_redaction_path: PathBuf,
    post_cancel_readback_path: PathBuf,
    incident_audit_closeout_path: PathBuf,
}

impl ProductionCancelRecoveryArtifactPaths {
    fn v18(record: &SupervisorNodeRecord) -> Self {
        Self {
            cancel_request_preview_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_ARTIFACT_RELATIVE_PATH),
            cancel_risk_gate_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_CANCEL_RISK_GATE_ARTIFACT_RELATIVE_PATH),
            manual_owner_approval_lifecycle_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_ARTIFACT_RELATIVE_PATH),
            cancel_response_redaction_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_ARTIFACT_RELATIVE_PATH),
            post_cancel_readback_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_POST_CANCEL_READBACK_ARTIFACT_RELATIVE_PATH),
            incident_audit_closeout_path: record.artifact_root.join(
                PRODUCTION_MUTATION_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_ARTIFACT_RELATIVE_PATH,
            ),
        }
    }

    fn has_any_artifact(&self) -> bool {
        self.cancel_request_preview_path.exists()
            || self.cancel_risk_gate_path.exists()
            || self.manual_owner_approval_lifecycle_path.exists()
            || self.cancel_response_redaction_path.exists()
            || self.post_cancel_readback_path.exists()
            || self.incident_audit_closeout_path.exists()
    }
}

fn production_cancel_recovery_from_record(
    record: &SupervisorNodeRecord,
) -> Option<ProductionCancelRecoveryStatus> {
    let paths = ProductionCancelRecoveryArtifactPaths::v18(record);
    if !paths.has_any_artifact() {
        return None;
    }

    let cancel_request_preview = read_json_file_value(&paths.cancel_request_preview_path);
    let cancel_risk_gate = read_json_file_value(&paths.cancel_risk_gate_path);
    let manual_owner_approval_lifecycle =
        read_json_file_value(&paths.manual_owner_approval_lifecycle_path);
    let cancel_response_redaction = read_json_file_value(&paths.cancel_response_redaction_path);
    let post_cancel_readback = read_json_file_value(&paths.post_cancel_readback_path);
    let incident_audit_closeout = read_json_file_value(&paths.incident_audit_closeout_path);
    let artifacts = [
        &cancel_request_preview,
        &cancel_risk_gate,
        &manual_owner_approval_lifecycle,
        &cancel_response_redaction,
        &post_cancel_readback,
        &incident_audit_closeout,
    ];
    let artifact_specs = [
        (
            "cancel_request_preview",
            paths.cancel_request_preview_path.as_path(),
            &cancel_request_preview,
            PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_SCHEMA_VERSION,
        ),
        (
            "cancel_risk_gate",
            paths.cancel_risk_gate_path.as_path(),
            &cancel_risk_gate,
            PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION,
        ),
        (
            "manual_owner_approval_lifecycle",
            paths.manual_owner_approval_lifecycle_path.as_path(),
            &manual_owner_approval_lifecycle,
            PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION,
        ),
        (
            "cancel_response_redaction",
            paths.cancel_response_redaction_path.as_path(),
            &cancel_response_redaction,
            PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION,
        ),
        (
            "post_cancel_readback",
            paths.post_cancel_readback_path.as_path(),
            &post_cancel_readback,
            PRODUCTION_MUTATION_POST_CANCEL_READBACK_SCHEMA_VERSION,
        ),
        (
            "incident_audit_closeout",
            paths.incident_audit_closeout_path.as_path(),
            &incident_audit_closeout,
            PRODUCTION_MUTATION_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_SCHEMA_VERSION,
        ),
    ];
    let missing_artifacts = v17_missing_artifact_diagnostics(&artifact_specs);
    let schema_diagnostics = v17_schema_diagnostics(&artifact_specs);
    let provenance_diagnostics = v17_provenance_diagnostics(&artifact_specs);
    let stale_artifacts = v17_stale_artifact_diagnostics(&artifact_specs);
    let schema_ok = schema_diagnostics.is_empty();
    let provenance_ok = provenance_diagnostics.is_empty();

    let order_lineage_id = v18_first_available_string_any(&artifacts, "order_lineage_id");
    let cancel_preview_status = cancel_request_preview
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let cancel_request_preview_ready =
        v18_any_available_bool_any(&artifacts, "cancel_request_preview_ready");
    let cancel_reason = v18_first_available_string_any(&artifacts, "cancel_reason");
    let candidate_count = v18_max_available_u64_any(&artifacts, "candidate_count");
    let known_order_id = v18_first_available_string_any(&artifacts, "known_order_id");
    let known_client_order_id = v18_first_available_string_any(&artifacts, "known_client_order_id");
    let symbol = v18_first_available_string_any(&artifacts, "symbol");
    let account_label = v18_first_available_string_any(&artifacts, "account_label");
    let risk_gate_status = cancel_risk_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let risk_gate_result = incident_audit_closeout
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "risk_gate_result")
        });
    let risk_gate_ready = v18_any_available_bool_any(&artifacts, "risk_gate_ready");
    let orphan_risk_detected = v18_any_available_bool_any(&artifacts, "orphan_risk_detected");
    let risk_halted = v18_any_available_bool_any(&artifacts, "risk_halted");
    let manual_review_required = v18_any_available_bool_any(&artifacts, "manual_review_required");
    let new_orders_blocked = v18_any_available_bool_any(&artifacts, "new_orders_blocked");
    let approval_lifecycle_status = manual_owner_approval_lifecycle
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let owner_approval_state = first_available_string_from_values([
        incident_audit_closeout
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "owner_approval_state")
            }),
        manual_owner_approval_lifecycle
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "approval_state")
            }),
    ]);
    let manual_approval_recorded =
        v18_any_available_bool_any(&artifacts, "manual_approval_recorded");
    let approval_lifecycle_valid =
        v18_any_available_bool_any(&artifacts, "approval_lifecycle_valid");
    let approval_consumed = v18_any_available_bool_any(&artifacts, "approval_consumed");
    let redaction_contract_state = incident_audit_closeout
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "redaction_contract_state")
        });
    let cancel_response_redaction_ready = first_available_bool_from_values([
        incident_audit_closeout
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "cancel_response_redaction_ready")
            }),
        cancel_response_redaction
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, "response_redaction_ready")
            }),
    ]);
    let cancel_response_redacted =
        v18_any_available_bool_any(&artifacts, "cancel_response_redacted");
    let post_cancel_readback_status = post_cancel_readback
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let post_cancel_readback_ready =
        v18_any_available_bool_any(&artifacts, "post_cancel_readback_ready");
    let readback_state = v18_first_available_string_any(&artifacts, "readback_state");
    let readback_state_class = v18_first_available_string_any(&artifacts, "readback_state_class");
    let readback_outcome = v18_first_available_string_any(&artifacts, "readback_outcome");
    let terminal_state_observed = v18_any_available_bool_any(&artifacts, "terminal_state_observed");
    let ambiguous_state_observed =
        v18_any_available_bool_any(&artifacts, "ambiguous_state_observed");
    let incident_closeout_status = incident_audit_closeout
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let incident_closeout_ready = v18_any_available_bool_any(&artifacts, "incident_closeout_ready");
    let audit_trail_ready = v18_any_available_bool_any(&artifacts, "audit_trail_ready");
    let audit_traceability_ready =
        v18_any_available_bool_any(&artifacts, "audit_traceability_ready");
    let cancel_recovery_lineage_ready =
        v18_any_available_bool_any(&artifacts, "cancel_recovery_lineage_ready");
    let terminal_action_recommendation =
        v18_first_available_string_any(&artifacts, "terminal_action_recommendation");
    let remaining_risk = v18_first_available_string_any(&artifacts, "remaining_risk");
    let remaining_risk_requires_manual_review =
        v18_any_available_bool_any(&artifacts, "remaining_risk_requires_manual_review");
    let source_artifact_issues = incident_audit_closeout
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_array_field(value, "source_artifact_issues")
        });
    let lineage_issues = incident_audit_closeout
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_array_field(value, "lineage_issues")
        });
    let missing_cli_flags = incident_audit_closeout
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_array_field(value, "missing_cli_flags")
        });
    let actual_cancel_send_allowed =
        v18_any_available_bool_any(&artifacts, "actual_cancel_send_allowed");
    let cancel_attempted = v18_any_available_bool_any(&artifacts, "cancel_attempted");
    let cancel_requests_sent = v18_max_available_u64_any(&artifacts, "cancel_requests_sent");
    let production_order_mutations_attempted =
        v18_max_available_u64_any(&artifacts, "production_order_mutations_attempted");
    let readback_execution_attempted =
        v18_any_available_bool_any(&artifacts, "readback_execution_attempted");
    let production_order_state_reads_attempted =
        v18_max_available_u64_any(&artifacts, "production_order_state_reads_attempted");
    let network_attempted = v18_any_available_bool_any(&artifacts, "network_attempted");
    let network_readback_endpoint_attempted =
        v18_any_available_bool_any(&artifacts, "network_readback_endpoint_attempted");
    let network_cancel_endpoint_attempted =
        v18_any_available_bool_any(&artifacts, "network_cancel_endpoint_attempted");
    let retry_attempted = v18_any_available_bool_any(&artifacts, "retry_attempted");
    let remediation_attempted = v18_any_available_bool_any(&artifacts, "remediation_attempted");
    let automatic_cancel_allowed =
        v18_any_available_bool_any(&artifacts, "automatic_cancel_allowed");
    let automatic_remediation_allowed =
        v18_any_available_bool_any(&artifacts, "automatic_remediation_allowed");
    let dashboard_order_controls_enabled =
        v18_any_available_bool_any(&artifacts, "dashboard_order_controls_enabled");
    let dashboard_cancel_controls_enabled =
        v18_any_available_bool_any(&artifacts, "dashboard_cancel_controls_enabled");
    let dashboard_auto_approval_allowed =
        v18_any_available_bool_any(&artifacts, "dashboard_auto_approval_allowed");
    let dashboard_auto_approval_attempted =
        v18_any_available_bool_any(&artifacts, "dashboard_auto_approval_attempted");

    let boundary_violation = !schema_ok
        || !provenance_ok
        || !stale_artifacts.is_empty()
        || actual_cancel_send_allowed.value == Some(true)
        || cancel_attempted.value == Some(true)
        || dashboard_u64_gt_zero(&cancel_requests_sent)
        || dashboard_u64_gt_zero(&production_order_mutations_attempted)
        || readback_execution_attempted.value == Some(true)
        || dashboard_u64_gt_zero(&production_order_state_reads_attempted)
        || network_attempted.value == Some(true)
        || network_readback_endpoint_attempted.value == Some(true)
        || network_cancel_endpoint_attempted.value == Some(true)
        || retry_attempted.value == Some(true)
        || remediation_attempted.value == Some(true)
        || automatic_cancel_allowed.value == Some(true)
        || automatic_remediation_allowed.value == Some(true)
        || dashboard_order_controls_enabled.value == Some(true)
        || dashboard_cancel_controls_enabled.value == Some(true)
        || dashboard_auto_approval_allowed.value == Some(true)
        || dashboard_auto_approval_attempted.value == Some(true);
    let closeout_blocked = v18_array_has_items(&incident_audit_closeout, "source_artifact_issues")
        || v18_array_has_items(&incident_audit_closeout, "lineage_issues")
        || v18_array_has_items(&incident_audit_closeout, "missing_cli_flags");
    let ready = missing_artifacts.is_empty()
        && cancel_request_preview_ready.value == Some(true)
        && risk_gate_ready.value == Some(true)
        && approval_lifecycle_valid.value == Some(true)
        && cancel_response_redaction_ready.value == Some(true)
        && cancel_response_redacted.value == Some(true)
        && post_cancel_readback_ready.value == Some(true)
        && incident_closeout_ready.value == Some(true)
        && audit_trail_ready.value == Some(true)
        && audit_traceability_ready.value == Some(true)
        && cancel_recovery_lineage_ready.value == Some(true)
        && !closeout_blocked
        && !boundary_violation;
    let manual_followup_required =
        ready && remaining_risk_requires_manual_review.value == Some(true);
    let readiness_status = if boundary_violation {
        "production_cancel_recovery_boundary_violation"
    } else if manual_followup_required {
        "production_cancel_recovery_manual_review_required"
    } else if ready {
        "production_cancel_recovery_ready"
    } else {
        "production_cancel_recovery_incomplete"
    };

    Some(ProductionCancelRecoveryStatus {
        node_id: record.node_id.clone(),
        health: if ready && !manual_followup_required {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        },
        readiness_status: DashboardValue::available(readiness_status.to_string()),
        diagnostic: DashboardValue::available(v18_cancel_recovery_diagnostic(
            readiness_status,
            &missing_artifacts,
            &schema_diagnostics,
            &provenance_diagnostics,
            &stale_artifacts,
            closeout_blocked,
        )),
        missing_artifacts: diagnostic_value(&missing_artifacts),
        schema_diagnostics: diagnostic_value(&schema_diagnostics),
        provenance_diagnostics: diagnostic_value(&provenance_diagnostics),
        stale_artifacts: diagnostic_value(&stale_artifacts),
        order_lineage_id,
        cancel_preview_status,
        cancel_request_preview_ready,
        cancel_reason,
        candidate_count,
        known_order_id,
        known_client_order_id,
        symbol,
        account_label,
        risk_gate_status,
        risk_gate_result,
        risk_gate_ready,
        orphan_risk_detected,
        risk_halted,
        manual_review_required,
        new_orders_blocked,
        approval_lifecycle_status,
        owner_approval_state,
        manual_approval_recorded,
        approval_lifecycle_valid,
        approval_consumed,
        redaction_contract_state,
        cancel_response_redaction_ready,
        cancel_response_redacted,
        post_cancel_readback_status,
        post_cancel_readback_ready,
        readback_state,
        readback_state_class,
        readback_outcome,
        terminal_state_observed,
        ambiguous_state_observed,
        incident_closeout_status,
        incident_closeout_ready,
        audit_trail_ready,
        audit_traceability_ready,
        cancel_recovery_lineage_ready,
        terminal_action_recommendation,
        remaining_risk,
        remaining_risk_requires_manual_review,
        source_artifact_issues,
        lineage_issues,
        missing_cli_flags,
        actual_cancel_send_allowed,
        cancel_attempted,
        cancel_requests_sent,
        production_order_mutations_attempted,
        readback_execution_attempted,
        production_order_state_reads_attempted,
        network_attempted,
        network_readback_endpoint_attempted,
        network_cancel_endpoint_attempted,
        retry_attempted,
        remediation_attempted,
        automatic_cancel_allowed,
        automatic_remediation_allowed,
        dashboard_order_controls_enabled,
        dashboard_cancel_controls_enabled,
        dashboard_auto_approval_allowed,
        dashboard_auto_approval_attempted,
        cancel_request_preview_path: dashboard_path_if_exists(&paths.cancel_request_preview_path),
        cancel_risk_gate_path: dashboard_path_if_exists(&paths.cancel_risk_gate_path),
        manual_owner_approval_lifecycle_path: dashboard_path_if_exists(
            &paths.manual_owner_approval_lifecycle_path,
        ),
        cancel_response_redaction_path: dashboard_path_if_exists(
            &paths.cancel_response_redaction_path,
        ),
        post_cancel_readback_path: dashboard_path_if_exists(&paths.post_cancel_readback_path),
        incident_audit_closeout_path: dashboard_path_if_exists(&paths.incident_audit_closeout_path),
    })
}

#[derive(Clone, Debug)]
struct ProductionActualCancelAuditArtifactPaths {
    cancel_risk_gate_path: PathBuf,
    owner_approval_lifecycle_path: PathBuf,
    actual_cancel_single_shot_path: PathBuf,
    readback_reconciliation_path: PathBuf,
    failure_evidence_path: PathBuf,
}

impl ProductionActualCancelAuditArtifactPaths {
    fn v19(record: &SupervisorNodeRecord) -> Self {
        Self {
            cancel_risk_gate_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_CANCEL_RISK_GATE_ARTIFACT_RELATIVE_PATH),
            owner_approval_lifecycle_path: record.artifact_root.join(
                PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_ARTIFACT_RELATIVE_PATH,
            ),
            actual_cancel_single_shot_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_ARTIFACT_RELATIVE_PATH),
            readback_reconciliation_path: record.artifact_root.join(
                PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_ARTIFACT_RELATIVE_PATH,
            ),
            failure_evidence_path: record
                .artifact_root
                .join(PRODUCTION_MUTATION_ACTUAL_CANCEL_FAILURE_EVIDENCE_ARTIFACT_RELATIVE_PATH),
        }
    }

    fn has_any_artifact(&self) -> bool {
        self.cancel_risk_gate_path.exists()
            || self.owner_approval_lifecycle_path.exists()
            || self.actual_cancel_single_shot_path.exists()
            || self.readback_reconciliation_path.exists()
            || self.failure_evidence_path.exists()
    }
}

fn production_actual_cancel_audit_from_record(
    record: &SupervisorNodeRecord,
) -> Option<ProductionActualCancelAuditStatus> {
    let paths = ProductionActualCancelAuditArtifactPaths::v19(record);
    if !paths.has_any_artifact() {
        return None;
    }

    let cancel_risk_gate = read_json_file_value(&paths.cancel_risk_gate_path);
    let owner_approval_lifecycle = read_json_file_value(&paths.owner_approval_lifecycle_path);
    let actual_cancel_single_shot = read_json_file_value(&paths.actual_cancel_single_shot_path);
    let readback_reconciliation = read_json_file_value(&paths.readback_reconciliation_path);
    let failure_evidence = read_json_file_value(&paths.failure_evidence_path);
    let artifacts = [
        &cancel_risk_gate,
        &owner_approval_lifecycle,
        &actual_cancel_single_shot,
        &readback_reconciliation,
        &failure_evidence,
    ];
    let artifact_specs = [
        (
            "cancel_risk_gate",
            paths.cancel_risk_gate_path.as_path(),
            &cancel_risk_gate,
            PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION,
        ),
        (
            "actual_cancel_owner_approval_lifecycle",
            paths.owner_approval_lifecycle_path.as_path(),
            &owner_approval_lifecycle,
            PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION,
        ),
        (
            "actual_cancel_single_shot",
            paths.actual_cancel_single_shot_path.as_path(),
            &actual_cancel_single_shot,
            PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_SCHEMA_VERSION,
        ),
        (
            "actual_cancel_readback_reconciliation",
            paths.readback_reconciliation_path.as_path(),
            &readback_reconciliation,
            PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_SCHEMA_VERSION,
        ),
        (
            "actual_cancel_failure_evidence",
            paths.failure_evidence_path.as_path(),
            &failure_evidence,
            PRODUCTION_MUTATION_ACTUAL_CANCEL_FAILURE_EVIDENCE_SCHEMA_VERSION,
        ),
    ];
    let missing_artifacts = v17_missing_artifact_diagnostics(&artifact_specs);
    let schema_diagnostics = v17_schema_diagnostics(&artifact_specs);
    let provenance_diagnostics = v17_provenance_diagnostics(&artifact_specs);
    let stale_artifacts = v17_stale_artifact_diagnostics(&artifact_specs);
    let schema_ok = schema_diagnostics.is_empty();
    let provenance_ok = provenance_diagnostics.is_empty();

    let order_lineage_id = v18_first_available_string_any(&artifacts, "order_lineage_id");
    let approval_lifecycle_status = owner_approval_lifecycle
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let owner_approval_state = owner_approval_lifecycle
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "approval_state")
        });
    let approval_lifecycle_valid =
        v18_any_available_bool_any(&artifacts, "approval_lifecycle_valid");
    let approval_execution_authorized =
        v18_any_available_bool_any(&artifacts, "approval_execution_authorized");
    let risk_gate_status = cancel_risk_gate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let risk_gate_result = v18_first_available_string_any(&artifacts, "risk_gate_result");
    let risk_gate_ready = v18_any_available_bool_any(&artifacts, "risk_gate_ready");
    let cancel_attempt_status = actual_cancel_single_shot
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let actual_cancel_command_ready =
        v18_any_available_bool_any(&artifacts, "actual_cancel_command_ready");
    let single_shot_cancel_allowed =
        v18_any_available_bool_any(&artifacts, "single_shot_cancel_allowed");
    let request_sent = v18_any_available_bool_any(&artifacts, "request_sent");
    let cancel_attempted = v18_any_available_bool_any(&artifacts, "cancel_attempted");
    let cancel_requests_sent = v18_max_available_u64_any(&artifacts, "cancel_requests_sent");
    let request_id = first_available_string_from_values([
        actual_cancel_single_shot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "request_id")
            }),
        readback_reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "actual_cancel_request_id")
            }),
    ]);
    let venue_response_status = actual_cancel_single_shot
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "venue_response_status")
        });
    let venue_response_source = actual_cancel_single_shot
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "venue_response_source")
        });
    let venue_response_code = actual_cancel_single_shot
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_u64_field(value, "venue_response_code")
        });
    let venue_response_error_code = actual_cancel_single_shot
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "venue_response_error_code")
        });
    let local_audit_reference = actual_cancel_single_shot
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "local_audit_reference")
        });
    let readback_status = readback_reconciliation
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let readback_result = first_available_string_from_values([
        failure_evidence
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "readback_result")
            }),
        readback_reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "readback_result")
            }),
    ]);
    let reconciliation_status = v18_first_available_string_any(&artifacts, "reconciliation_status");
    let readback_state = first_available_string_from_values([
        failure_evidence
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "source_readback_state")
            }),
        readback_reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "readback_state")
            }),
    ]);
    let venue_state = first_available_string_from_values([
        failure_evidence
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "source_venue_state")
            }),
        readback_reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, "venue_state")
            }),
    ]);
    let terminal_state_observed = v18_any_available_bool_any(&artifacts, "terminal_state_observed");
    let unknown_observed = v18_any_available_bool_any(&artifacts, "unknown_observed");
    let dashboard_read_only_consumable =
        v18_any_available_bool_any(&artifacts, "dashboard_read_only_consumable");
    let dashboard_audit_view_ready =
        v18_any_available_bool_any(&artifacts, "dashboard_audit_view_ready");
    let outcome_status = failure_evidence
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let cancel_outcome = failure_evidence
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "cancel_outcome")
        });
    let outcome_category = failure_evidence
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "outcome_category")
        });
    let recovered = v18_any_available_bool_any(&artifacts, "recovered");
    let degraded = v18_any_available_bool_any(&artifacts, "degraded");
    let failed = v18_any_available_bool_any(&artifacts, "failed");
    let partial_success = v18_any_available_bool_any(&artifacts, "partial_success");
    let operator_action_required =
        v18_any_available_bool_any(&artifacts, "operator_action_required");
    let residual_risk_visible = v18_any_available_bool_any(&artifacts, "residual_risk_visible");
    let request_response_readback_audit_refs_recorded =
        v18_any_available_bool_any(&artifacts, "request_response_readback_audit_refs_recorded");
    let source_artifact_issues = first_available_string_from_values([
        failure_evidence
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "source_artifact_issues")
            }),
        readback_reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "source_artifact_issues")
            }),
        actual_cancel_single_shot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "source_artifact_issues")
            }),
    ]);
    let lineage_issues = first_available_string_from_values([
        failure_evidence
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "lineage_issues")
            }),
        readback_reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "readback_lineage_issues")
            }),
    ]);
    let missing_cli_flags = first_available_string_from_values([
        failure_evidence
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "missing_cli_flags")
            }),
        readback_reconciliation
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "missing_cli_flags")
            }),
        actual_cancel_single_shot
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_array_field(value, "missing_cli_flags")
            }),
    ]);
    let actual_cancel_send_allowed =
        v18_any_available_bool_any(&artifacts, "actual_cancel_send_allowed");
    let production_order_mutations_attempted =
        v18_max_available_u64_any(&artifacts, "production_order_mutations_attempted");
    let readback_execution_attempted =
        v18_any_available_bool_any(&artifacts, "readback_execution_attempted");
    let production_order_state_reads_attempted =
        v18_max_available_u64_any(&artifacts, "production_order_state_reads_attempted");
    let network_attempted = v18_any_available_bool_any(&artifacts, "network_attempted");
    let network_readback_endpoint_attempted =
        v18_any_available_bool_any(&artifacts, "network_readback_endpoint_attempted");
    let network_cancel_endpoint_attempted =
        v18_any_available_bool_any(&artifacts, "network_cancel_endpoint_attempted");
    let retry_attempted = v18_any_available_bool_any(&artifacts, "retry_attempted");
    let remediation_attempted = v18_any_available_bool_any(&artifacts, "remediation_attempted");
    let automatic_cancel_allowed =
        v18_any_available_bool_any(&artifacts, "automatic_cancel_allowed");
    let automatic_remediation_allowed =
        v18_any_available_bool_any(&artifacts, "automatic_remediation_allowed");
    let dashboard_order_controls_enabled =
        v18_any_available_bool_any(&artifacts, "dashboard_order_controls_enabled");
    let dashboard_cancel_controls_enabled =
        v18_any_available_bool_any(&artifacts, "dashboard_cancel_controls_enabled");
    let bulk_cancel_allowed = v18_any_available_bool_any(&artifacts, "bulk_cancel_allowed");
    let second_cancel_attempted = v18_any_available_bool_any(&artifacts, "second_cancel_attempted");
    let compensation_trade_attempted =
        v18_any_available_bool_any(&artifacts, "compensation_trade_attempted");
    let reconciliation_ready = v18_any_available_bool_any(&artifacts, "reconciliation_ready");
    let readback_reconciliation_complete =
        v18_any_available_bool_any(&artifacts, "readback_reconciliation_complete");
    let evidence_ready = v18_any_available_bool_any(&artifacts, "evidence_ready");
    let failure_evidence_ready = v18_any_available_bool_any(&artifacts, "failure_evidence_ready");

    let issue_artifacts = [
        &actual_cancel_single_shot,
        &readback_reconciliation,
        &failure_evidence,
    ];
    let source_issue_blocked = issue_artifacts.iter().any(|artifact| {
        v18_array_has_items(artifact, "source_artifact_issues")
            || v18_array_has_items(artifact, "lineage_issues")
            || v18_array_has_items(artifact, "readback_lineage_issues")
            || v18_array_has_items(artifact, "missing_cli_flags")
    });
    let boundary_violation = !schema_ok
        || !provenance_ok
        || !stale_artifacts.is_empty()
        || actual_cancel_send_allowed.value == Some(true)
        || retry_attempted.value == Some(true)
        || remediation_attempted.value == Some(true)
        || automatic_cancel_allowed.value == Some(true)
        || automatic_remediation_allowed.value == Some(true)
        || dashboard_order_controls_enabled.value == Some(true)
        || dashboard_cancel_controls_enabled.value == Some(true)
        || bulk_cancel_allowed.value == Some(true)
        || second_cancel_attempted.value == Some(true)
        || compensation_trade_attempted.value == Some(true);
    let unknown_readback = unknown_observed.value == Some(true)
        || readback_result
            .value
            .as_deref()
            .is_some_and(|value| value == "unknown")
        || cancel_outcome
            .value
            .as_deref()
            .is_some_and(|value| value == "unknown");
    let required_ready = missing_artifacts.is_empty()
        && !boundary_violation
        && !source_issue_blocked
        && risk_gate_ready.value == Some(true)
        && approval_lifecycle_valid.value == Some(true)
        && approval_execution_authorized.value == Some(true)
        && actual_cancel_command_ready.value == Some(true)
        && single_shot_cancel_allowed.value == Some(true)
        && request_sent.value == Some(true)
        && cancel_attempted.value == Some(true)
        && dashboard_u64_gt_zero(&cancel_requests_sent)
        && reconciliation_ready.value == Some(true)
        && readback_reconciliation_complete.value == Some(true)
        && dashboard_read_only_consumable.value == Some(true)
        && dashboard_audit_view_ready.value == Some(true)
        && evidence_ready.value == Some(true)
        && failure_evidence_ready.value == Some(true)
        && request_response_readback_audit_refs_recorded.value == Some(true);
    let audit_state = if !required_ready || unknown_readback {
        "unknown"
    } else if outcome_category.value.as_deref() == Some("ready") {
        "ready"
    } else if recovered.value == Some(true)
        || outcome_category.value.as_deref() == Some("recovered")
        || cancel_outcome
            .value
            .as_deref()
            .is_some_and(|value| matches!(value, "cancel_confirmed" | "already_cancelled"))
    {
        "recovered"
    } else if failed.value == Some(true) || outcome_category.value.as_deref() == Some("failed") {
        "failed"
    } else if degraded.value == Some(true)
        || partial_success.value == Some(true)
        || operator_action_required.value == Some(true)
        || outcome_category
            .value
            .as_deref()
            .is_some_and(|value| matches!(value, "degraded" | "partial_success"))
    {
        "degraded"
    } else {
        "unknown"
    };
    let readiness_status = if boundary_violation {
        "production_actual_cancel_audit_boundary_violation".to_string()
    } else if !required_ready {
        "production_actual_cancel_audit_incomplete".to_string()
    } else {
        format!("production_actual_cancel_audit_{audit_state}")
    };
    let health = match audit_state {
        "ready" | "recovered" if readiness_status.ends_with(audit_state) => HealthStatus::Healthy,
        "failed" if readiness_status.ends_with(audit_state) => HealthStatus::Error,
        _ => HealthStatus::Degraded,
    };

    Some(ProductionActualCancelAuditStatus {
        node_id: record.node_id.clone(),
        health,
        readiness_status: DashboardValue::available(readiness_status.clone()),
        audit_state: DashboardValue::available(audit_state.to_string()),
        diagnostic: DashboardValue::available(v19_actual_cancel_audit_diagnostic(
            &readiness_status,
            &missing_artifacts,
            &schema_diagnostics,
            &provenance_diagnostics,
            &stale_artifacts,
            V19ActualCancelAuditDiagnosticFlags {
                boundary_violation,
                unknown_readback,
                source_issue_blocked,
            },
        )),
        missing_artifacts: diagnostic_value(&missing_artifacts),
        schema_diagnostics: diagnostic_value(&schema_diagnostics),
        provenance_diagnostics: diagnostic_value(&provenance_diagnostics),
        stale_artifacts: diagnostic_value(&stale_artifacts),
        order_lineage_id,
        approval_lifecycle_status,
        owner_approval_state,
        approval_lifecycle_valid,
        approval_execution_authorized,
        risk_gate_status,
        risk_gate_result,
        risk_gate_ready,
        cancel_attempt_status,
        actual_cancel_command_ready,
        single_shot_cancel_allowed,
        request_sent,
        cancel_attempted,
        cancel_requests_sent,
        request_id,
        venue_response_status,
        venue_response_source,
        venue_response_code,
        venue_response_error_code,
        local_audit_reference,
        readback_status,
        readback_result,
        reconciliation_status,
        readback_state,
        venue_state,
        terminal_state_observed,
        unknown_observed,
        dashboard_read_only_consumable,
        dashboard_audit_view_ready,
        outcome_status,
        cancel_outcome,
        outcome_category,
        recovered,
        degraded,
        failed,
        partial_success,
        operator_action_required,
        residual_risk_visible,
        request_response_readback_audit_refs_recorded,
        source_artifact_issues,
        lineage_issues,
        missing_cli_flags,
        actual_cancel_send_allowed,
        production_order_mutations_attempted,
        readback_execution_attempted,
        production_order_state_reads_attempted,
        network_attempted,
        network_readback_endpoint_attempted,
        network_cancel_endpoint_attempted,
        retry_attempted,
        remediation_attempted,
        automatic_cancel_allowed,
        automatic_remediation_allowed,
        dashboard_order_controls_enabled,
        dashboard_cancel_controls_enabled,
        bulk_cancel_allowed,
        second_cancel_attempted,
        compensation_trade_attempted,
        cancel_risk_gate_path: dashboard_path_if_exists(&paths.cancel_risk_gate_path),
        owner_approval_lifecycle_path: dashboard_path_if_exists(
            &paths.owner_approval_lifecycle_path,
        ),
        actual_cancel_single_shot_path: dashboard_path_if_exists(
            &paths.actual_cancel_single_shot_path,
        ),
        readback_reconciliation_path: dashboard_path_if_exists(&paths.readback_reconciliation_path),
        failure_evidence_path: dashboard_path_if_exists(&paths.failure_evidence_path),
    })
}

#[derive(Clone, Debug)]
struct ProductionOrderLifecycleAuditArtifactPaths {
    submit_candidate_path: PathBuf,
    response_redaction_path: PathBuf,
    readback_reconciliation_path: PathBuf,
    failure_no_retry_path: PathBuf,
    audit_closeout_path: PathBuf,
}

impl ProductionOrderLifecycleAuditArtifactPaths {
    fn v20(record: &SupervisorNodeRecord) -> Self {
        Self {
            submit_candidate_path: record
                .artifact_root
                .join(PRODUCTION_ORDER_LIFECYCLE_SUBMIT_CANDIDATE_ARTIFACT_RELATIVE_PATH),
            response_redaction_path: record
                .artifact_root
                .join(PRODUCTION_ORDER_LIFECYCLE_RESPONSE_REDACTION_ARTIFACT_RELATIVE_PATH),
            readback_reconciliation_path: record
                .artifact_root
                .join(PRODUCTION_ORDER_LIFECYCLE_READBACK_RECONCILIATION_ARTIFACT_RELATIVE_PATH),
            failure_no_retry_path: record
                .artifact_root
                .join(PRODUCTION_ORDER_LIFECYCLE_FAILURE_NO_RETRY_ARTIFACT_RELATIVE_PATH),
            audit_closeout_path: record
                .artifact_root
                .join(PRODUCTION_ORDER_LIFECYCLE_AUDIT_CLOSEOUT_ARTIFACT_RELATIVE_PATH),
        }
    }

    fn has_any_artifact(&self) -> bool {
        self.submit_candidate_path.exists()
            || self.response_redaction_path.exists()
            || self.readback_reconciliation_path.exists()
            || self.failure_no_retry_path.exists()
            || self.audit_closeout_path.exists()
    }
}

fn production_order_lifecycle_audit_from_record(
    record: &SupervisorNodeRecord,
) -> Option<ProductionOrderLifecycleAuditStatus> {
    let paths = ProductionOrderLifecycleAuditArtifactPaths::v20(record);
    if !paths.has_any_artifact() {
        return None;
    }

    let submit_candidate = read_json_file_value(&paths.submit_candidate_path);
    let response_redaction = read_json_file_value(&paths.response_redaction_path);
    let readback_reconciliation = read_json_file_value(&paths.readback_reconciliation_path);
    let failure_no_retry = read_json_file_value(&paths.failure_no_retry_path);
    let audit_closeout = read_json_file_value(&paths.audit_closeout_path);
    let artifacts = [
        &submit_candidate,
        &response_redaction,
        &readback_reconciliation,
        &failure_no_retry,
        &audit_closeout,
    ];
    let artifact_specs = [
        (
            "guarded_submit_candidate",
            paths.submit_candidate_path.as_path(),
            &submit_candidate,
            PRODUCTION_ORDER_LIFECYCLE_SUBMIT_CANDIDATE_SCHEMA_VERSION,
        ),
        (
            "submit_response_redaction",
            paths.response_redaction_path.as_path(),
            &response_redaction,
            PRODUCTION_ORDER_LIFECYCLE_RESPONSE_REDACTION_SCHEMA_VERSION,
        ),
        (
            "submit_readback_reconciliation",
            paths.readback_reconciliation_path.as_path(),
            &readback_reconciliation,
            PRODUCTION_ORDER_LIFECYCLE_READBACK_RECONCILIATION_SCHEMA_VERSION,
        ),
        (
            "failure_no_retry_evidence",
            paths.failure_no_retry_path.as_path(),
            &failure_no_retry,
            PRODUCTION_ORDER_LIFECYCLE_FAILURE_NO_RETRY_SCHEMA_VERSION,
        ),
        (
            "order_lifecycle_audit_closeout",
            paths.audit_closeout_path.as_path(),
            &audit_closeout,
            PRODUCTION_ORDER_LIFECYCLE_AUDIT_CLOSEOUT_SCHEMA_VERSION,
        ),
    ];
    let missing_artifacts = v17_missing_artifact_diagnostics(&artifact_specs);
    let schema_diagnostics = v17_schema_diagnostics(&artifact_specs);
    let provenance_diagnostics = v17_provenance_diagnostics(&artifact_specs);
    let stale_artifacts = v17_stale_artifact_diagnostics(&artifact_specs);
    let source_diagnostics =
        v20_order_lifecycle_source_diagnostics(&response_redaction, &readback_reconciliation);
    let schema_ok = schema_diagnostics.is_empty();
    let provenance_ok = provenance_diagnostics.is_empty();

    let lifecycle_id = v18_first_available_string_any(&artifacts, "lifecycle_id");
    let attempt_id = v18_first_available_string_any(&artifacts, "attempt_id");
    let submit_attempt_state = submit_candidate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "state")
        });
    let submit_attempt_code = submit_candidate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "code")
        });
    let owner_approval_state_before_attempt = submit_candidate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "owner_approval_state_before_attempt")
        });
    let owner_approval_state_after_attempt = submit_candidate
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "owner_approval_state_after_attempt")
        });
    let owner_approval_consumed = v18_any_available_bool_any(&artifacts, "owner_approval_consumed");
    let production_submit_attempted =
        v18_any_available_bool_any(&artifacts, "production_submit_attempted");
    let readback_required = v18_any_available_bool_any(&artifacts, "readback_required");
    let response_state = response_redaction
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "state")
        });
    let response_code = response_redaction
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "code")
        });
    let venue_status = response_redaction
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "venue_status")
        });
    let venue_order_id = response_redaction
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "order_id")
        });
    let client_order_id = response_redaction
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "client_order_id")
        });
    let readback_state = readback_reconciliation
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "state")
        });
    let readback_code = readback_reconciliation
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "code")
        });
    let mismatch_fields = readback_reconciliation
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_array_field(value, "mismatch_fields")
        });
    let readback_consistent = v18_any_available_bool_any(&artifacts, "readback_consistent");
    let readback_missing = v18_any_available_bool_any(&artifacts, "readback_missing");
    let readback_failed = v18_any_available_bool_any(&artifacts, "readback_failed");
    let source_artifacts = [&response_redaction, &readback_reconciliation];
    let adapter_runtime_integrated =
        v18_any_available_bool_any(&source_artifacts, "adapter_runtime_integrated");
    let exchange_truth_claimed =
        v18_any_available_bool_any(&source_artifacts, "exchange_truth_claimed");
    let foundation_only = DashboardValue::available(
        adapter_runtime_integrated.value != Some(true)
            && exchange_truth_claimed.value != Some(true),
    );
    let evidence_source_class = DashboardValue::available(v20_order_lifecycle_source_class(
        &response_redaction,
        &readback_reconciliation,
        adapter_runtime_integrated.value == Some(true),
    ));
    let foundation_boundary_status =
        DashboardValue::available(v20_order_lifecycle_foundation_boundary_status(
            &source_diagnostics,
            foundation_only.value,
            adapter_runtime_integrated.value,
            exchange_truth_claimed.value,
        ));
    let foundation_boundary_diagnostics = v20_order_lifecycle_foundation_boundary_diagnostics(
        foundation_boundary_status.value.as_deref(),
        evidence_source_class.value.as_deref(),
    );
    let failure_category = failure_no_retry
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "category")
        });
    let failure_code = failure_no_retry
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "code")
        });
    let next_allowed_action = failure_no_retry
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "next_allowed_action")
        });
    let no_implicit_retry = v18_any_available_bool_any(&artifacts, "no_implicit_retry");
    let unknown_state_visible = v18_any_available_bool_any(&artifacts, "unknown_state_visible");
    let audit_closeout_status = audit_closeout
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "status")
        });
    let audit_closed = v18_any_available_bool_any(&artifacts, "audit_closed");
    let dashboard_audit_consumable =
        v18_any_available_bool_any(&artifacts, "dashboard_audit_consumable");
    let release_gate_consumable = v18_any_available_bool_any(&artifacts, "release_gate_consumable");
    let dashboard_order_controls_enabled =
        v18_any_available_bool_any(&artifacts, "dashboard_order_controls_enabled");
    let dashboard_approval_controls_enabled =
        v18_any_available_bool_any(&artifacts, "dashboard_approval_controls_enabled");
    let dashboard_cancel_controls_enabled =
        v18_any_available_bool_any(&artifacts, "dashboard_cancel_controls_enabled");
    let retry_attempted = v18_any_available_bool_any(&artifacts, "retry_attempted");
    let replace_attempted = v18_any_available_bool_any(&artifacts, "replace_attempted");
    let amend_attempted = v18_any_available_bool_any(&artifacts, "amend_attempted");
    let flatten_attempted = v18_any_available_bool_any(&artifacts, "flatten_attempted");
    let automatic_cancel_attempted =
        v18_any_available_bool_any(&artifacts, "automatic_cancel_attempted");
    let automatic_cancel_allowed =
        v18_any_available_bool_any(&artifacts, "automatic_cancel_allowed");
    let automatic_remediation_allowed =
        v18_any_available_bool_any(&artifacts, "automatic_remediation_allowed");
    let strategy_continuation_allowed =
        v18_any_available_bool_any(&artifacts, "strategy_continuation_allowed");

    let boundary_violation = !schema_ok
        || !provenance_ok
        || !source_diagnostics.is_empty()
        || !foundation_boundary_diagnostics.is_empty()
        || !stale_artifacts.is_empty()
        || dashboard_order_controls_enabled.value == Some(true)
        || dashboard_approval_controls_enabled.value == Some(true)
        || dashboard_cancel_controls_enabled.value == Some(true)
        || retry_attempted.value == Some(true)
        || replace_attempted.value == Some(true)
        || amend_attempted.value == Some(true)
        || flatten_attempted.value == Some(true)
        || automatic_cancel_attempted.value == Some(true)
        || automatic_cancel_allowed.value == Some(true)
        || automatic_remediation_allowed.value == Some(true)
        || strategy_continuation_allowed.value == Some(true);
    let risk_visible = unknown_state_visible.value == Some(true)
        || readback_missing.value == Some(true)
        || readback_failed.value == Some(true)
        || readback_state.value.as_deref().is_some_and(|value| {
            matches!(
                value,
                "mismatched" | "missing" | "ambiguous" | "readback_failed"
            )
        })
        || failure_category.value.as_deref().is_some_and(|value| {
            matches!(
                value,
                "response_unknown"
                    | "readback_missing"
                    | "readback_mismatch"
                    | "cancel_required"
                    | "audit_incomplete"
            )
        });
    let ready = missing_artifacts.is_empty()
        && !boundary_violation
        && production_submit_attempted.value == Some(true)
        && readback_required.value == Some(true)
        && dashboard_audit_consumable.value == Some(true)
        && release_gate_consumable.value == Some(true)
        && no_implicit_retry.value == Some(true)
        && audit_closed.value == Some(true);
    let audit_state = if boundary_violation {
        "boundary_violation"
    } else if !ready {
        "incomplete"
    } else if risk_visible {
        "risk_visible"
    } else {
        "audit_closed"
    };
    let readiness_status = format!("production_order_lifecycle_audit_{audit_state}");
    let health = match audit_state {
        "audit_closed" => HealthStatus::Healthy,
        "boundary_violation" => HealthStatus::Error,
        _ => HealthStatus::Degraded,
    };

    Some(ProductionOrderLifecycleAuditStatus {
        node_id: record.node_id.clone(),
        health,
        readiness_status: DashboardValue::available(readiness_status.clone()),
        audit_state: DashboardValue::available(audit_state.to_string()),
        risk_visibility: DashboardValue::available(
            if risk_visible {
                "risk_visible"
            } else {
                "no_risk_visible"
            }
            .to_string(),
        ),
        diagnostic: DashboardValue::available(v20_order_lifecycle_audit_diagnostic(
            &readiness_status,
            &V20OrderLifecycleAuditDiagnostics {
                missing_artifacts: &missing_artifacts,
                schema_diagnostics: &schema_diagnostics,
                provenance_diagnostics: &provenance_diagnostics,
                source_diagnostics: &source_diagnostics,
                foundation_boundary_diagnostics: &foundation_boundary_diagnostics,
                stale_artifacts: &stale_artifacts,
            },
            boundary_violation,
            risk_visible,
        )),
        missing_artifacts: diagnostic_value(&missing_artifacts),
        schema_diagnostics: diagnostic_value(&schema_diagnostics),
        provenance_diagnostics: diagnostic_value(&provenance_diagnostics),
        source_diagnostics: diagnostic_value(&source_diagnostics),
        foundation_boundary_status,
        foundation_boundary_diagnostics: diagnostic_value(&foundation_boundary_diagnostics),
        evidence_source_class,
        adapter_runtime_integrated,
        foundation_only,
        exchange_truth_claimed,
        stale_artifacts: diagnostic_value(&stale_artifacts),
        lifecycle_id,
        attempt_id,
        submit_attempt_state,
        submit_attempt_code,
        owner_approval_state_before_attempt,
        owner_approval_state_after_attempt,
        owner_approval_consumed,
        production_submit_attempted,
        readback_required,
        response_state,
        response_code,
        venue_status,
        venue_order_id,
        client_order_id,
        readback_state,
        readback_code,
        mismatch_fields,
        readback_consistent,
        readback_missing,
        readback_failed,
        failure_category,
        failure_code,
        next_allowed_action,
        no_implicit_retry,
        unknown_state_visible,
        audit_closeout_status,
        audit_closed,
        dashboard_audit_consumable,
        release_gate_consumable,
        dashboard_order_controls_enabled,
        dashboard_approval_controls_enabled,
        dashboard_cancel_controls_enabled,
        retry_attempted,
        replace_attempted,
        amend_attempted,
        flatten_attempted,
        automatic_cancel_attempted,
        automatic_remediation_allowed,
        strategy_continuation_allowed,
        submit_candidate_path: dashboard_path_if_exists(&paths.submit_candidate_path),
        response_redaction_path: dashboard_path_if_exists(&paths.response_redaction_path),
        readback_reconciliation_path: dashboard_path_if_exists(&paths.readback_reconciliation_path),
        failure_no_retry_path: dashboard_path_if_exists(&paths.failure_no_retry_path),
        audit_closeout_path: dashboard_path_if_exists(&paths.audit_closeout_path),
    })
}

fn diagnostic_value(diagnostics: &[String]) -> DashboardValue<String> {
    if diagnostics.is_empty() {
        DashboardValue::unknown()
    } else {
        DashboardValue::available(diagnostics.join(";"))
    }
}

fn v17_missing_artifact_diagnostics(
    artifacts: &[(&str, &FsPath, &Option<Value>, &str)],
) -> Vec<String> {
    artifacts
        .iter()
        .filter(|(_, path, _, _)| !path.exists())
        .map(|(name, _, _, _)| (*name).to_string())
        .collect()
}

fn v17_schema_diagnostics(artifacts: &[(&str, &FsPath, &Option<Value>, &str)]) -> Vec<String> {
    artifacts
        .iter()
        .filter_map(|(name, path, value, expected)| {
            if !path.exists() {
                return None;
            }
            let actual = value
                .as_ref()
                .and_then(|value| value.get("schema_version"))
                .and_then(Value::as_str)
                .unwrap_or(if value.is_some() {
                    "missing_schema"
                } else {
                    "json_invalid"
                });
            (actual != *expected).then(|| format!("{name}:expected={expected},actual={actual}"))
        })
        .collect()
}

fn v17_provenance_diagnostics(artifacts: &[(&str, &FsPath, &Option<Value>, &str)]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for (artifact_name, _, artifact, _) in artifacts {
        let Some(object) = artifact.as_ref().and_then(Value::as_object) else {
            continue;
        };
        for (field, source_ref) in object {
            if !field.ends_with("_ref") || !source_ref.is_object() {
                continue;
            }
            let prefix = format!("{artifact_name}.{field}");
            for required in [
                "sha256",
                "bytes",
                "source_command",
                "source_commit",
                "source_release_tag",
            ] {
                if source_ref.get(required).is_none() {
                    diagnostics.push(format!("{prefix}.{required}_missing"));
                }
            }
            let Some(source_path) = source_ref.get("path").and_then(Value::as_str) else {
                diagnostics.push(format!("{prefix}.path_missing"));
                continue;
            };
            let source_path = FsPath::new(source_path);
            if !source_path.exists() {
                diagnostics.push(format!("{prefix}.source_missing"));
                continue;
            }
            let Ok(bytes) = fs::read(source_path) else {
                diagnostics.push(format!("{prefix}.source_unreadable"));
                continue;
            };
            if source_ref
                .get("sha256")
                .and_then(Value::as_str)
                .is_some_and(|expected| expected != sha256_bytes(&bytes))
            {
                diagnostics.push(format!("{prefix}.sha256_mismatch"));
            }
            if source_ref
                .get("bytes")
                .and_then(Value::as_u64)
                .is_some_and(|expected| expected != bytes.len() as u64)
            {
                diagnostics.push(format!("{prefix}.bytes_mismatch"));
            }
            if let Ok(source_value) = serde_json::from_slice::<Value>(&bytes) {
                match (
                    source_ref.get("source_commit").and_then(Value::as_str),
                    source_value.get("source_commit").and_then(Value::as_str),
                ) {
                    (Some(expected_commit), Some(actual_commit))
                        if expected_commit != actual_commit =>
                    {
                        diagnostics.push(format!("{prefix}.source_commit_mismatch"));
                    }
                    _ => {}
                }
                match (
                    source_ref.get("source_release_tag").and_then(Value::as_str),
                    source_value
                        .get("source_release_tag")
                        .and_then(Value::as_str),
                ) {
                    (Some(expected_tag), Some(actual_tag)) if expected_tag != actual_tag => {
                        diagnostics.push(format!("{prefix}.source_release_tag_mismatch"));
                    }
                    _ => {}
                }
            }
        }
    }
    diagnostics
}

fn v17_stale_artifact_diagnostics(
    artifacts: &[(&str, &FsPath, &Option<Value>, &str)],
) -> Vec<String> {
    artifacts
        .iter()
        .filter_map(|(name, _, value, _)| {
            let value = value.as_ref()?;
            let stale = json_bool(value, "stale_ledger_restart_required")
                || json_bool(value, "stale_evidence")
                || json_bool(value, "regeneration_required")
                || value
                    .get("status")
                    .and_then(Value::as_str)
                    .is_some_and(|status| status.contains("stale"));
            stale.then(|| (*name).to_string())
        })
        .collect()
}

fn v17_reconciliation_orphan_diagnostic(
    readiness_status: &str,
    missing_artifacts: &[String],
    schema_diagnostics: &[String],
    provenance_diagnostics: &[String],
    stale_artifacts: &[String],
) -> String {
    if !missing_artifacts.is_empty() {
        format!(
            "production_reconciliation_orphan_missing_artifacts:{}",
            missing_artifacts.join("|")
        )
    } else if !schema_diagnostics.is_empty() {
        format!(
            "production_reconciliation_orphan_schema_mismatch:{}",
            schema_diagnostics.join("|")
        )
    } else if !provenance_diagnostics.is_empty() {
        format!(
            "production_reconciliation_orphan_provenance_mismatch:{}",
            provenance_diagnostics.join("|")
        )
    } else if !stale_artifacts.is_empty() {
        format!(
            "production_reconciliation_orphan_stale_evidence:{}",
            stale_artifacts.join("|")
        )
    } else {
        readiness_status.to_string()
    }
}

fn v18_cancel_recovery_diagnostic(
    readiness_status: &str,
    missing_artifacts: &[String],
    schema_diagnostics: &[String],
    provenance_diagnostics: &[String],
    stale_artifacts: &[String],
    closeout_blocked: bool,
) -> String {
    if !missing_artifacts.is_empty() {
        format!(
            "production_cancel_recovery_missing_artifacts:{}",
            missing_artifacts.join("|")
        )
    } else if !schema_diagnostics.is_empty() {
        format!(
            "production_cancel_recovery_schema_mismatch:{}",
            schema_diagnostics.join("|")
        )
    } else if !provenance_diagnostics.is_empty() {
        format!(
            "production_cancel_recovery_provenance_mismatch:{}",
            provenance_diagnostics.join("|")
        )
    } else if !stale_artifacts.is_empty() {
        format!(
            "production_cancel_recovery_stale_evidence:{}",
            stale_artifacts.join("|")
        )
    } else if closeout_blocked {
        "production_cancel_recovery_closeout_blocked".to_string()
    } else {
        readiness_status.to_string()
    }
}

#[derive(Clone, Copy)]
struct V19ActualCancelAuditDiagnosticFlags {
    boundary_violation: bool,
    unknown_readback: bool,
    source_issue_blocked: bool,
}

fn v19_actual_cancel_audit_diagnostic(
    readiness_status: &str,
    missing_artifacts: &[String],
    schema_diagnostics: &[String],
    provenance_diagnostics: &[String],
    stale_artifacts: &[String],
    flags: V19ActualCancelAuditDiagnosticFlags,
) -> String {
    if !missing_artifacts.is_empty() {
        format!(
            "production_actual_cancel_audit_missing_evidence:{}",
            missing_artifacts.join("|")
        )
    } else if !schema_diagnostics.is_empty() {
        format!(
            "production_actual_cancel_audit_schema_mismatch:{}",
            schema_diagnostics.join("|")
        )
    } else if !provenance_diagnostics.is_empty() {
        format!(
            "production_actual_cancel_audit_provenance_mismatch:{}",
            provenance_diagnostics.join("|")
        )
    } else if !stale_artifacts.is_empty() {
        format!(
            "production_actual_cancel_audit_stale_evidence:{}",
            stale_artifacts.join("|")
        )
    } else if flags.boundary_violation {
        "production_actual_cancel_audit_boundary_violation".to_string()
    } else if flags.unknown_readback {
        "production_actual_cancel_audit_unknown_readback".to_string()
    } else if flags.source_issue_blocked {
        "production_actual_cancel_audit_source_issue_blocked".to_string()
    } else {
        readiness_status.to_string()
    }
}

struct V20OrderLifecycleAuditDiagnostics<'a> {
    missing_artifacts: &'a [String],
    schema_diagnostics: &'a [String],
    provenance_diagnostics: &'a [String],
    source_diagnostics: &'a [String],
    foundation_boundary_diagnostics: &'a [String],
    stale_artifacts: &'a [String],
}

fn v20_order_lifecycle_audit_diagnostic(
    readiness_status: &str,
    diagnostics: &V20OrderLifecycleAuditDiagnostics<'_>,
    boundary_violation: bool,
    risk_visible: bool,
) -> String {
    if !diagnostics.missing_artifacts.is_empty() {
        format!(
            "production_order_lifecycle_audit_missing_evidence:{}",
            diagnostics.missing_artifacts.join("|")
        )
    } else if !diagnostics.schema_diagnostics.is_empty() {
        format!(
            "production_order_lifecycle_audit_schema_mismatch:{}",
            diagnostics.schema_diagnostics.join("|")
        )
    } else if !diagnostics.provenance_diagnostics.is_empty() {
        format!(
            "production_order_lifecycle_audit_provenance_mismatch:{}",
            diagnostics.provenance_diagnostics.join("|")
        )
    } else if !diagnostics.source_diagnostics.is_empty() {
        format!(
            "production_order_lifecycle_audit_source_mismatch:{}",
            diagnostics.source_diagnostics.join("|")
        )
    } else if !diagnostics.foundation_boundary_diagnostics.is_empty() {
        format!(
            "production_order_lifecycle_audit_foundation_boundary:{}",
            diagnostics.foundation_boundary_diagnostics.join("|")
        )
    } else if !diagnostics.stale_artifacts.is_empty() {
        format!(
            "production_order_lifecycle_audit_stale_evidence:{}",
            diagnostics.stale_artifacts.join("|")
        )
    } else if boundary_violation {
        "production_order_lifecycle_audit_boundary_violation".to_string()
    } else if risk_visible {
        "production_order_lifecycle_audit_risk_visible".to_string()
    } else {
        readiness_status.to_string()
    }
}

fn v20_order_lifecycle_source_diagnostics(
    response_redaction: &Option<Value>,
    readback_reconciliation: &Option<Value>,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    v20_collect_source_diagnostics(
        &mut diagnostics,
        "submit_response_redaction",
        response_redaction,
        false,
    );
    v20_collect_source_diagnostics(
        &mut diagnostics,
        "submit_readback_reconciliation",
        readback_reconciliation,
        true,
    );
    diagnostics
}

fn v20_collect_source_diagnostics(
    diagnostics: &mut Vec<String>,
    artifact_name: &str,
    artifact: &Option<Value>,
    allow_exchange_readback: bool,
) {
    let Some(value) = artifact.as_ref() else {
        return;
    };
    let source = json_string_field(value, "evidence_source");
    let source_value = source.value.as_deref();
    let provenance = json_string_field(value, "source_provenance_id");
    let provenance_valid = json_bool_field(value, "source_provenance_valid");
    let claim_consistent = json_bool_field(value, "source_claim_consistent");
    let exchange_truth_claimed = json_bool_field(value, "exchange_truth_claimed");
    let adapter_runtime_integrated = json_bool_field(value, "adapter_runtime_integrated");
    let foundation_only = json_bool_field(value, "foundation_only");

    match source_value {
        None => diagnostics.push(format!("{artifact_name}_evidence_source_missing")),
        Some("unknown") => diagnostics.push(format!("{artifact_name}_evidence_source_unknown")),
        Some("manual_structured" | "adapter_snapshot") => {}
        Some("exchange_readback") if allow_exchange_readback => {}
        Some(other) => diagnostics.push(format!("{artifact_name}_evidence_source_invalid:{other}")),
    }
    if provenance
        .value
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
        || provenance_valid.value == Some(false)
    {
        diagnostics.push(format!("{artifact_name}_source_provenance_missing"));
    }
    if claim_consistent.value == Some(false) {
        diagnostics.push(format!("{artifact_name}_source_claim_inconsistent"));
    }
    if source_value == Some("manual_structured") && exchange_truth_claimed.value == Some(true) {
        diagnostics.push(format!(
            "{artifact_name}_manual_structured_claims_exchange_truth"
        ));
    }
    if source_value == Some("manual_structured") && adapter_runtime_integrated.value == Some(true) {
        diagnostics.push(format!(
            "{artifact_name}_manual_structured_claims_adapter_runtime"
        ));
    }
    if source_value == Some("exchange_readback") && adapter_runtime_integrated.value != Some(true) {
        diagnostics.push(format!(
            "{artifact_name}_exchange_readback_without_adapter_runtime"
        ));
    }
    if exchange_truth_claimed.value == Some(true) && adapter_runtime_integrated.value != Some(true)
    {
        diagnostics.push(format!(
            "{artifact_name}_exchange_truth_without_adapter_runtime"
        ));
    }
    if foundation_only.value == Some(true)
        && (adapter_runtime_integrated.value == Some(true)
            || exchange_truth_claimed.value == Some(true))
    {
        diagnostics.push(format!("{artifact_name}_foundation_only_claim_conflict"));
    }
}

fn v20_order_lifecycle_source_class(
    response_redaction: &Option<Value>,
    readback_reconciliation: &Option<Value>,
    adapter_runtime_integrated: bool,
) -> String {
    let response_source = response_redaction
        .as_ref()
        .and_then(|value| json_string_field(value, "evidence_source").value);
    let readback_source = readback_reconciliation
        .as_ref()
        .and_then(|value| json_string_field(value, "evidence_source").value);
    let source_class = if adapter_runtime_integrated {
        "adapter_integrated_runtime"
    } else if response_source.as_deref() == Some("manual_structured")
        && readback_source.as_deref() == Some("manual_structured")
    {
        "foundation_only_manual_structured"
    } else {
        "mixed_or_unknown_source"
    };
    source_class.to_string()
}

fn v20_order_lifecycle_foundation_boundary_status(
    source_diagnostics: &[String],
    foundation_only: Option<bool>,
    adapter_runtime_integrated: Option<bool>,
    exchange_truth_claimed: Option<bool>,
) -> String {
    if source_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("adapter_runtime")
            || diagnostic.contains("exchange_truth_without_adapter_runtime")
    }) {
        "adapter_runtime_claim_mismatch"
    } else if foundation_only == Some(true)
        && adapter_runtime_integrated != Some(true)
        && exchange_truth_claimed != Some(true)
    {
        "foundation_only_no_adapter_runtime"
    } else if adapter_runtime_integrated == Some(true) {
        "adapter_integrated_runtime"
    } else if exchange_truth_claimed == Some(true) {
        "exchange_truth_claim_without_adapter_runtime"
    } else {
        "mixed_or_unknown_foundation_boundary"
    }
    .to_string()
}

fn v20_order_lifecycle_foundation_boundary_diagnostics(
    foundation_boundary_status: Option<&str>,
    evidence_source_class: Option<&str>,
) -> Vec<String> {
    match foundation_boundary_status {
        Some("foundation_only_no_adapter_runtime" | "adapter_integrated_runtime") => Vec::new(),
        Some(status) => vec![format!(
            "{status}:source_class={}",
            evidence_source_class.unwrap_or("unknown")
        )],
        None => vec!["foundation_boundary_status_missing".to_string()],
    }
}

fn v18_first_available_string_any(
    artifacts: &[&Option<Value>],
    field: &str,
) -> DashboardValue<String> {
    v17_first_available_string_any(artifacts, field)
}

fn v18_any_available_bool_any(artifacts: &[&Option<Value>], field: &str) -> DashboardValue<bool> {
    v17_any_available_bool_any(artifacts, field)
}

fn v18_max_available_u64_any(artifacts: &[&Option<Value>], field: &str) -> DashboardValue<u64> {
    v16_max_available_u64_any(artifacts, field)
}

fn v18_array_has_items(artifact: &Option<Value>, field: &str) -> bool {
    artifact
        .as_ref()
        .and_then(|value| value.get(field))
        .and_then(Value::as_array)
        .is_some_and(|items| !items.is_empty())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = digest::digest(&digest::SHA256, bytes);
    format!("sha256:{}", lowercase_hex(digest.as_ref()))
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn artifact_schema_matches(value: &Option<Value>, expected: &str) -> bool {
    value.as_ref().is_some_and(|value| {
        value
            .get("schema_version")
            .and_then(Value::as_str)
            .is_some_and(|schema| schema == expected)
    })
}

fn order_state_readonly_proof_boundary_violation(value: &Option<Value>, schema_ok: bool) -> bool {
    let Some(value) = value.as_ref() else {
        return false;
    };
    !schema_ok
        || json_bool(value, "production_order_submission_attempted")
        || json_bool(value, "production_order_mutation_attempted")
        || json_bool(value, "cancel_replace_amend_attempted")
        || json_bool(value, "listen_key_lifecycle_attempted")
        || json_bool(value, "dashboard_order_controls_enabled")
        || json_bool(value, "automatic_remediation_attempted")
        || json_bool(value, "real_orders_submitted")
        || json_bool(value, "real_funds")
        || json_bool(value, "production_trading_enabled")
        || json_bool(value, "shadow_values_are_exchange_truth")
        || json_bool(value, "portfolio_values_are_exchange_truth")
}

fn live_alpha_order_state_truth_source(
    order_state_proof: &Option<Value>,
    risk_preflight: &Option<Value>,
) -> DashboardValue<String> {
    if order_state_proof
        .as_ref()
        .is_some_and(|value| json_bool(value, "order_state_values_are_exchange_truth"))
    {
        return DashboardValue::available("exchange_order_state_readonly_proof".to_string());
    }
    if order_state_proof.as_ref().is_some_and(|value| {
        json_bool(value, "endpoint_shape_validated") || json_bool(value, "response_shape_validated")
    }) {
        return DashboardValue::available("endpoint_shape_only".to_string());
    }
    if risk_preflight
        .as_ref()
        .and_then(|value| value.get("order_state_readable"))
        .is_some()
    {
        return DashboardValue::available("live_alpha_risk_preflight".to_string());
    }
    DashboardValue::unknown()
}

fn live_alpha_first_available_u64_any(
    artifacts: &[&Option<Value>],
    field: &str,
) -> DashboardValue<u64> {
    first_available_u64_from_values(artifacts.iter().map(|artifact| {
        artifact
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, field)
            })
    }))
}

fn live_alpha_first_available_bool_any(
    artifacts: &[&Option<Value>],
    field: &str,
) -> DashboardValue<bool> {
    first_available_bool_from_values(artifacts.iter().map(|artifact| {
        artifact
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, field)
            })
    }))
}

fn live_alpha_any_available_bool_any(
    artifacts: &[&Option<Value>],
    field: &str,
) -> DashboardValue<bool> {
    any_available_bool_from_values(artifacts.iter().map(|artifact| {
        artifact
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, field)
            })
    }))
}

fn v16_first_available_string_any(
    artifacts: &[&Option<Value>],
    field: &str,
) -> DashboardValue<String> {
    first_available_string_from_values(artifacts.iter().map(|artifact| {
        artifact
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, field)
            })
    }))
}

fn v16_max_available_u64_any(artifacts: &[&Option<Value>], field: &str) -> DashboardValue<u64> {
    max_available_u64_from_values(artifacts.iter().map(|artifact| {
        artifact
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_u64_field(value, field)
            })
    }))
}

fn v16_any_available_bool_any(artifacts: &[&Option<Value>], field: &str) -> DashboardValue<bool> {
    any_available_bool_from_values(artifacts.iter().map(|artifact| {
        artifact
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, field)
            })
    }))
}

fn v17_first_available_string_any(
    artifacts: &[&Option<Value>],
    field: &str,
) -> DashboardValue<String> {
    first_available_string_from_values(artifacts.iter().map(|artifact| {
        artifact
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_string_field(value, field)
            })
    }))
}

fn v17_any_available_bool_any(artifacts: &[&Option<Value>], field: &str) -> DashboardValue<bool> {
    any_available_bool_from_values(artifacts.iter().map(|artifact| {
        artifact
            .as_ref()
            .map_or_else(DashboardValue::unknown, |value| {
                json_bool_field(value, field)
            })
    }))
}

fn live_alpha_v15_boundary_violation(
    manual_approval_lifecycle: &Option<Value>,
    request_preview: &Option<Value>,
    execution_dry_run: &Option<Value>,
    kill_switch_runtime_gate: &Option<Value>,
) -> bool {
    let artifacts = [
        manual_approval_lifecycle,
        request_preview,
        execution_dry_run,
        kill_switch_runtime_gate,
    ];
    let required_zero_fields = [
        "production_order_submissions_attempted",
        "production_orders_submitted",
        "production_order_mutations_attempted",
        "production_order_state_reads_attempted",
        "listen_key_lifecycle_attempted",
        "actual_submission_count",
        "automatic_correction_orders_submitted",
    ];
    let required_false_fields = [
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "production_order_state_reads_allowed",
        "listen_key_lifecycle_allowed",
        "request_sent",
        "order_endpoint_access_attempted",
        "execution_adapter_called",
        "real_execution_adapter_called",
        "production_adapter_instantiated",
        "production_adapter_called",
        "strategy_intent_reaches_production_adapter",
        "cancel_replace_amend_attempted",
        "matching_engine_submission",
        "dashboard_order_controls_enabled",
        "network_attempted",
        "real_orders_submitted",
        "real_funds",
        "production_trading_enabled",
        "shadow_values_are_exchange_truth",
        "portfolio_values_are_exchange_truth",
        "values_are_exchange_truth",
    ];
    artifacts
        .into_iter()
        .filter_map(Option::as_ref)
        .any(|artifact| {
            required_zero_fields
                .iter()
                .any(|field| artifact.get(field).and_then(Value::as_u64).unwrap_or(0) > 0)
                || required_false_fields
                    .iter()
                    .any(|field| json_bool(artifact, field))
        })
}

fn live_alpha_missing_gate_flags(
    order_gate: &Option<Value>,
    risk_preflight: &Option<Value>,
) -> DashboardValue<String> {
    let mut flags = Vec::new();
    for value in [order_gate, risk_preflight]
        .into_iter()
        .filter_map(Option::as_ref)
    {
        if let Some(items) = value.get("missing_cli_flags").and_then(Value::as_array) {
            flags.extend(items.iter().filter_map(Value::as_str).map(str::to_string));
        }
    }
    flags.sort();
    flags.dedup();
    if flags.is_empty() {
        DashboardValue::available("none".to_string())
    } else {
        DashboardValue::available(flags.join(","))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LiveAlphaSchemaHealth {
    gate_schema_ok: bool,
    risk_schema_ok: bool,
    order_state_schema_ok: bool,
    manual_approval_schema_ok: bool,
    request_preview_schema_ok: bool,
    execution_dry_run_schema_ok: bool,
    kill_switch_runtime_gate_schema_ok: bool,
}

fn live_alpha_diagnostic(
    order_gate: &Option<Value>,
    risk_preflight: &Option<Value>,
    order_state_proof: &Option<Value>,
    schema_health: LiveAlphaSchemaHealth,
    boundary_violation: bool,
    fallback: &str,
) -> DashboardValue<String> {
    if !schema_health.gate_schema_ok {
        return DashboardValue::available("live_alpha_order_gate_schema_invalid".to_string());
    }
    if !schema_health.risk_schema_ok {
        return DashboardValue::available("live_alpha_risk_preflight_schema_invalid".to_string());
    }
    if !schema_health.order_state_schema_ok {
        return DashboardValue::available(
            "live_alpha_order_state_readonly_proof_schema_invalid".to_string(),
        );
    }
    if !schema_health.manual_approval_schema_ok {
        return DashboardValue::available(
            "live_alpha_manual_approval_lifecycle_schema_invalid".to_string(),
        );
    }
    if !schema_health.request_preview_schema_ok {
        return DashboardValue::available(
            "live_alpha_order_request_preview_schema_invalid".to_string(),
        );
    }
    if !schema_health.execution_dry_run_schema_ok {
        return DashboardValue::available(
            "live_alpha_execution_dry_run_schema_invalid".to_string(),
        );
    }
    if !schema_health.kill_switch_runtime_gate_schema_ok {
        return DashboardValue::available(
            "live_alpha_kill_switch_runtime_gate_schema_invalid".to_string(),
        );
    }
    if boundary_violation {
        return DashboardValue::available(
            "live_alpha_dry_run_readonly_boundary_violation".to_string(),
        );
    }
    order_state_proof
        .as_ref()
        .map_or_else(DashboardValue::unknown, |value| {
            json_string_field(value, "diagnostic")
        })
        .value
        .or_else(|| {
            risk_preflight
                .as_ref()
                .map_or_else(DashboardValue::unknown, |value| {
                    json_string_field(value, "diagnostic")
                })
                .value
        })
        .or_else(|| {
            order_gate
                .as_ref()
                .and_then(|value| json_string_field(value, "diagnostic").value)
        })
        .map_or_else(
            || DashboardValue::available(fallback.to_string()),
            DashboardValue::available,
        )
}

fn dashboard_u64_gt_zero(value: &DashboardValue<u64>) -> bool {
    value.value.is_some_and(|value| value > 0)
}

fn dashboard_u64_gt(value: &DashboardValue<u64>, threshold: u64) -> bool {
    value.value.is_some_and(|value| value > threshold)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProductionShadowArtifactHealthAudit {
    boundary_violation: bool,
    diagnostic: Option<String>,
}

fn combine_production_shadow_audits(
    left: ProductionShadowArtifactHealthAudit,
    right: ProductionShadowArtifactHealthAudit,
) -> ProductionShadowArtifactHealthAudit {
    match (left.diagnostic, right.diagnostic) {
        (Some(left_diagnostic), Some(right_diagnostic)) => ProductionShadowArtifactHealthAudit {
            boundary_violation: left.boundary_violation || right.boundary_violation,
            diagnostic: Some(format!("{left_diagnostic},{right_diagnostic}")),
        },
        (Some(diagnostic), None) | (None, Some(diagnostic)) => {
            ProductionShadowArtifactHealthAudit {
                boundary_violation: left.boundary_violation || right.boundary_violation,
                diagnostic: Some(diagnostic),
            }
        }
        (None, None) => ProductionShadowArtifactHealthAudit {
            boundary_violation: left.boundary_violation || right.boundary_violation,
            diagnostic: None,
        },
    }
}

fn audit_production_kill_switch_artifact_health(
    path: &FsPath,
    required: bool,
) -> ProductionShadowArtifactHealthAudit {
    if !path.exists() {
        return ProductionShadowArtifactHealthAudit {
            boundary_violation: required,
            diagnostic: required.then(|| "kill_switch_approval_artifact_missing".to_string()),
        };
    }

    let mut diagnostics = Vec::new();
    audit_required_production_shadow_json_artifact(
        path,
        "kill_switch_approval",
        PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION,
        &[
            "production_order_submissions_attempted",
            "production_orders_submitted",
            "production_order_mutations_attempted",
            "production_order_state_reads_attempted",
            "listen_key_lifecycle_attempted",
            "actual_submission_count",
            "automatic_correction_orders_submitted",
        ],
        &[
            "new_submit_capability",
            "production_order_submission_allowed",
            "production_order_mutation_allowed",
            "production_order_state_reads_allowed",
            "listen_key_lifecycle_allowed",
            "cancel_replace_amend_attempted",
            "dashboard_order_controls_enabled",
            "real_orders_submitted",
            "production_trading_enabled",
            "network_attempted",
            "values_are_exchange_truth",
        ],
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        ProductionShadowArtifactHealthAudit {
            boundary_violation: false,
            diagnostic: None,
        }
    } else {
        ProductionShadowArtifactHealthAudit {
            boundary_violation: true,
            diagnostic: Some(format!(
                "production_kill_switch_artifact_degraded:{}",
                diagnostics.join(",")
            )),
        }
    }
}

fn audit_production_shadow_v11_artifact_health(
    account_snapshot_path: &FsPath,
    shadow_intent_path: &FsPath,
    portfolio_snapshot_path: &FsPath,
    lifecycle_path: &FsPath,
    reconciliation_path: &FsPath,
) -> ProductionShadowArtifactHealthAudit {
    let mut diagnostics = Vec::new();
    audit_required_production_shadow_json_artifact(
        account_snapshot_path,
        "account_snapshot",
        PRODUCTION_ACCOUNT_SNAPSHOT_SCHEMA_VERSION,
        &[],
        &[
            "network_attempted",
            "account_read_attempted",
            "account_mutation_attempted",
            "order_endpoint_access_attempted",
            "production_order_submission_attempted",
            "production_order_mutation_attempted",
            "dashboard_order_controls_enabled",
        ],
        &mut diagnostics,
    );
    audit_required_production_shadow_jsonl_artifact(
        shadow_intent_path,
        "shadow_execution_intent",
        PRODUCTION_SHADOW_INTENT_SCHEMA_VERSION,
        &[],
        &[
            "submission_allowed",
            "actual_submission",
            "execution_adapter_called",
            "order_endpoint_access_attempted",
            "production_order_mutation_attempted",
            "dashboard_order_controls_enabled",
        ],
        &mut diagnostics,
    );
    audit_required_production_shadow_json_artifact(
        portfolio_snapshot_path,
        "shadow_portfolio_snapshot",
        PRODUCTION_SHADOW_PORTFOLIO_SCHEMA_VERSION,
        &[
            "actual_submission_count",
            "production_orders_submitted",
            "production_order_mutations_attempted",
            "automatic_correction_orders_submitted",
        ],
        &[
            "dashboard_order_controls_enabled",
            "full_production_portfolio_parity_claimed",
        ],
        &mut diagnostics,
    );
    audit_required_production_shadow_jsonl_artifact(
        lifecycle_path,
        "order_lifecycle_state",
        PRODUCTION_SHADOW_LIFECYCLE_SCHEMA_VERSION,
        &[
            "actual_submission_count",
            "production_orders_submitted",
            "production_order_mutations_attempted",
        ],
        &[
            "actual_submission",
            "exchange_order_id_recorded",
            "venue_order_id_recorded",
            "dashboard_order_controls_enabled",
        ],
        &mut diagnostics,
    );
    audit_required_production_shadow_jsonl_artifact(
        reconciliation_path,
        "reconciliation_events",
        PRODUCTION_SHADOW_RECONCILIATION_SCHEMA_VERSION,
        &[
            "automatic_correction_orders_submitted",
            "production_orders_submitted",
            "production_order_mutations_attempted",
        ],
        &[
            "cancel_replace_amend_attempted",
            "dashboard_order_controls_enabled",
        ],
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        ProductionShadowArtifactHealthAudit {
            boundary_violation: false,
            diagnostic: None,
        }
    } else {
        ProductionShadowArtifactHealthAudit {
            boundary_violation: true,
            diagnostic: Some(format!(
                "production_shadow_artifacts_degraded:{}",
                diagnostics.join(",")
            )),
        }
    }
}

fn audit_production_shadow_v12_artifact_health(
    paths: &ProductionShadowArtifactPaths,
) -> ProductionShadowArtifactHealthAudit {
    let mut diagnostics = Vec::new();
    audit_required_production_shadow_json_artifact_one_of(
        &paths.account_snapshot_path,
        "production_account_snapshot",
        &[
            PRODUCTION_ACCOUNT_SNAPSHOT_SCHEMA_VERSION,
            PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION,
        ],
        &[],
        &[
            "account_mutation_attempted",
            "order_endpoint_access_attempted",
            "production_order_submission_attempted",
            "production_order_mutation_attempted",
            "dashboard_order_controls_enabled",
        ],
        &mut diagnostics,
    );
    audit_optional_production_shadow_json_artifact_one_of(
        &paths.public_read_probe_path,
        "production_public_read_probe",
        &[
            PRODUCTION_PUBLIC_READ_PROBE_SCHEMA_VERSION,
            PRODUCTION_PUBLIC_ONLINE_READ_PROBE_SCHEMA_VERSION,
        ],
        &[],
        &[
            "account_mutation_attempted",
            "production_order_submission_attempted",
            "production_order_mutation_attempted",
            "dashboard_order_controls_enabled",
        ],
        &mut diagnostics,
    );
    audit_required_production_shadow_json_artifact(
        &paths.portfolio_snapshot_path,
        "shadow_portfolio_runtime",
        PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION,
        &[
            "actual_submission_count",
            "production_orders_submitted",
            "production_order_mutations_attempted",
            "automatic_correction_orders_submitted",
        ],
        &[
            "dashboard_order_controls_enabled",
            "full_production_portfolio_parity_claimed",
            "real_orders_submitted",
        ],
        &mut diagnostics,
    );
    audit_production_shadow_json_nested_bool_false(
        &paths.portfolio_snapshot_path,
        "shadow_portfolio_runtime",
        "/provenance/values_are_exchange_truth",
        &mut diagnostics,
    );
    audit_required_production_shadow_jsonl_artifact(
        &paths.shadow_strategy_session_path,
        "shadow_strategy_session",
        PRODUCTION_SHADOW_STRATEGY_SESSION_EVENT_SCHEMA_VERSION,
        &[
            "production_order_submissions_attempted",
            "production_orders_submitted",
            "production_order_mutations_attempted",
            "production_order_state_reads_attempted",
            "listen_key_lifecycle_attempted",
            "actual_submission_count",
            "automatic_correction_orders_submitted",
        ],
        &[
            "dashboard_order_controls_enabled",
            "real_orders_submitted",
            "values_are_exchange_truth",
        ],
        &mut diagnostics,
    );
    audit_required_production_shadow_jsonl_artifact(
        &paths.reconciliation_path,
        "readonly_reconciliation",
        PRODUCTION_READONLY_RECONCILIATION_EVENT_SCHEMA_VERSION,
        &[
            "automatic_correction_orders_submitted",
            "production_order_submissions_attempted",
            "production_orders_submitted",
            "production_order_mutations_attempted",
            "production_order_state_reads_attempted",
            "listen_key_lifecycle_attempted",
        ],
        &[
            "cancel_replace_amend_attempted",
            "dashboard_order_controls_enabled",
            "real_orders_submitted",
            "values_are_exchange_truth",
        ],
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        ProductionShadowArtifactHealthAudit {
            boundary_violation: false,
            diagnostic: None,
        }
    } else {
        ProductionShadowArtifactHealthAudit {
            boundary_violation: true,
            diagnostic: Some(format!(
                "production_shadow_v12_artifacts_degraded:{}",
                diagnostics.join(",")
            )),
        }
    }
}

fn audit_required_production_shadow_json_artifact(
    path: &FsPath,
    name: &str,
    expected_schema: &str,
    required_zero_u64_fields: &[&str],
    required_false_bool_fields: &[&str],
    diagnostics: &mut Vec<String>,
) {
    if !path.exists() {
        diagnostics.push(format!("{name}:missing_required_artifact"));
        return;
    }
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => {
            diagnostics.push(format!("{name}:unreadable"));
            return;
        }
    };
    match serde_json::from_str::<Value>(&raw) {
        Ok(value) => audit_production_shadow_value(
            name,
            &value,
            expected_schema,
            required_zero_u64_fields,
            required_false_bool_fields,
            diagnostics,
        ),
        Err(_) => diagnostics.push(format!("{name}:invalid_json")),
    }
}

fn audit_required_production_shadow_json_artifact_one_of(
    path: &FsPath,
    name: &str,
    expected_schemas: &[&str],
    required_zero_u64_fields: &[&str],
    required_false_bool_fields: &[&str],
    diagnostics: &mut Vec<String>,
) {
    if !path.exists() {
        diagnostics.push(format!("{name}:missing_required_artifact"));
        return;
    }
    audit_production_shadow_json_artifact_one_of(
        path,
        name,
        expected_schemas,
        required_zero_u64_fields,
        required_false_bool_fields,
        diagnostics,
    );
}

fn audit_optional_production_shadow_json_artifact_one_of(
    path: &FsPath,
    name: &str,
    expected_schemas: &[&str],
    required_zero_u64_fields: &[&str],
    required_false_bool_fields: &[&str],
    diagnostics: &mut Vec<String>,
) {
    if path.exists() {
        audit_production_shadow_json_artifact_one_of(
            path,
            name,
            expected_schemas,
            required_zero_u64_fields,
            required_false_bool_fields,
            diagnostics,
        );
    }
}

fn audit_production_shadow_json_artifact_one_of(
    path: &FsPath,
    name: &str,
    expected_schemas: &[&str],
    required_zero_u64_fields: &[&str],
    required_false_bool_fields: &[&str],
    diagnostics: &mut Vec<String>,
) {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => {
            diagnostics.push(format!("{name}:unreadable"));
            return;
        }
    };
    match serde_json::from_str::<Value>(&raw) {
        Ok(value) => audit_production_shadow_value_one_of(
            name,
            &value,
            expected_schemas,
            required_zero_u64_fields,
            required_false_bool_fields,
            diagnostics,
        ),
        Err(_) => diagnostics.push(format!("{name}:invalid_json")),
    }
}

fn audit_required_production_shadow_jsonl_artifact(
    path: &FsPath,
    name: &str,
    expected_schema: &str,
    required_zero_u64_fields: &[&str],
    required_false_bool_fields: &[&str],
    diagnostics: &mut Vec<String>,
) {
    if !path.exists() {
        diagnostics.push(format!("{name}:missing_required_artifact"));
        return;
    }
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => {
            diagnostics.push(format!("{name}:unreadable"));
            return;
        }
    };
    let mut records = 0_u64;
    for (index, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        records += 1;
        match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => audit_production_shadow_value(
                name,
                &value,
                expected_schema,
                required_zero_u64_fields,
                required_false_bool_fields,
                diagnostics,
            ),
            Err(_) => diagnostics.push(format!("{name}:invalid_jsonl_line_{}", index + 1)),
        }
    }
    if records == 0 {
        diagnostics.push(format!("{name}:empty_jsonl"));
    }
}

fn audit_production_shadow_json_nested_bool_false(
    path: &FsPath,
    name: &str,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(value) = read_json_file_value(path) else {
        diagnostics.push(format!("{name}:nested_bool_unreadable"));
        return;
    };
    if value
        .pointer(pointer)
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        diagnostics.push(format!("{name}:{pointer}_true"));
    }
}

fn audit_production_shadow_value(
    name: &str,
    value: &Value,
    expected_schema: &str,
    required_zero_u64_fields: &[&str],
    required_false_bool_fields: &[&str],
    diagnostics: &mut Vec<String>,
) {
    audit_production_shadow_value_one_of(
        name,
        value,
        &[expected_schema],
        required_zero_u64_fields,
        required_false_bool_fields,
        diagnostics,
    );
}

fn audit_production_shadow_value_one_of(
    name: &str,
    value: &Value,
    expected_schemas: &[&str],
    required_zero_u64_fields: &[&str],
    required_false_bool_fields: &[&str],
    diagnostics: &mut Vec<String>,
) {
    match value.get("schema_version").and_then(Value::as_str) {
        Some(schema) if expected_schemas.contains(&schema) => {}
        Some(_) => diagnostics.push(format!("{name}:schema_version_mismatch")),
        None => diagnostics.push(format!("{name}:schema_version_missing")),
    }

    for field in required_zero_u64_fields {
        match value.get(*field).and_then(Value::as_u64) {
            Some(0) => {}
            Some(_) => diagnostics.push(format!("{name}:{field}_nonzero")),
            None => diagnostics.push(format!("{name}:{field}_missing")),
        }
    }

    for field in required_false_bool_fields {
        match value.get(*field).and_then(Value::as_bool) {
            Some(false) => {}
            Some(true) => diagnostics.push(format!("{name}:{field}_true")),
            None => diagnostics.push(format!("{name}:{field}_missing")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProductionShadowManifestAudit {
    status: DashboardValue<String>,
    artifact_count: DashboardValue<u64>,
    boundary_violation: bool,
    diagnostic: Option<String>,
}

fn audit_production_shadow_manifest(shadow_root: &FsPath) -> ProductionShadowManifestAudit {
    let manifest_path = shadow_root.join("manifest.json");
    if !manifest_path.exists() {
        return ProductionShadowManifestAudit {
            status: DashboardValue::unknown(),
            artifact_count: DashboardValue::unknown(),
            boundary_violation: false,
            diagnostic: None,
        };
    }

    let Some(manifest) = read_json_file_value(&manifest_path) else {
        return degraded_production_shadow_manifest("production_shadow_manifest_json_invalid");
    };

    let mut diagnostics = Vec::new();
    if manifest.get("schema_version").and_then(Value::as_str)
        != Some(PRODUCTION_SHADOW_MANIFEST_SCHEMA_VERSION)
    {
        diagnostics.push("schema_version_mismatch");
    }
    if manifest
        .get("generated_at")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        diagnostics.push("generated_at_missing");
    }

    let artifacts = manifest.get("artifacts").and_then(Value::as_array);
    let artifact_count = artifacts.map_or_else(DashboardValue::unknown, |items| {
        DashboardValue::available(items.len() as u64)
    });
    match (
        manifest.get("artifact_count").and_then(Value::as_u64),
        artifacts,
    ) {
        (Some(declared), Some(items)) if declared == items.len() as u64 => {}
        (Some(_), Some(_)) => diagnostics.push("artifact_count_mismatch"),
        _ => diagnostics.push("artifact_count_missing"),
    }

    if let Some(items) = artifacts {
        for artifact in items {
            audit_production_shadow_manifest_artifact(shadow_root, artifact, &mut diagnostics);
        }
    }

    if production_shadow_manifest_summary_has_boundary_violation(&manifest) {
        diagnostics.push("production_shadow_manifest_boundary_violation");
    }

    if diagnostics.is_empty() {
        ProductionShadowManifestAudit {
            status: DashboardValue::available("production_shadow_manifest_ok".to_string()),
            artifact_count,
            boundary_violation: false,
            diagnostic: None,
        }
    } else {
        ProductionShadowManifestAudit {
            status: DashboardValue::available("production_shadow_manifest_degraded".to_string()),
            artifact_count,
            boundary_violation: true,
            diagnostic: Some(format!(
                "production_shadow_manifest_degraded:{}",
                diagnostics.join(",")
            )),
        }
    }
}

fn audit_production_shadow_manifest_artifact(
    shadow_root: &FsPath,
    artifact: &Value,
    diagnostics: &mut Vec<&str>,
) {
    if json_bool(artifact, "raw_secret_recorded")
        || json_bool(artifact, "raw_payload_recorded")
        || json_bool(artifact, "signed_query_recorded")
        || json_bool(artifact, "signed_url_recorded")
    {
        diagnostics.push("forbidden_payload_or_secret_recorded");
    }

    let Some(relative_path) = artifact.get("path").and_then(Value::as_str) else {
        diagnostics.push("artifact_path_missing");
        return;
    };
    let path = FsPath::new(relative_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        diagnostics.push("artifact_path_not_relative");
        return;
    }

    let artifact_path = shadow_root.join(path);
    let present = artifact_path.exists();
    if artifact
        .get("present")
        .and_then(Value::as_bool)
        .is_some_and(|declared| declared != present)
    {
        diagnostics.push("artifact_present_mismatch");
    }
    if json_bool(artifact, "required") && !present {
        diagnostics.push("required_artifact_missing");
        return;
    }
    if !present {
        return;
    }

    let Ok(bytes) = fs::read(&artifact_path) else {
        diagnostics.push("artifact_unreadable");
        return;
    };

    match artifact.get("checksum").and_then(Value::as_str) {
        Some(expected) if expected == checksum_bytes(&bytes) => {}
        Some(_) => diagnostics.push("artifact_checksum_mismatch"),
        None => diagnostics.push("artifact_checksum_missing"),
    }

    match artifact.get("byte_len").and_then(Value::as_u64) {
        Some(expected) if expected == bytes.len() as u64 => {}
        Some(_) => diagnostics.push("artifact_byte_len_mismatch"),
        None => diagnostics.push("artifact_byte_len_missing"),
    }

    let Some(format) = artifact.get("format").and_then(Value::as_str) else {
        diagnostics.push("artifact_format_missing");
        return;
    };
    let Some(actual_record_count) = production_shadow_artifact_record_count(format, &bytes) else {
        diagnostics.push("artifact_format_unknown");
        return;
    };
    match artifact.get("record_count").and_then(Value::as_u64) {
        Some(expected) if expected == actual_record_count => {}
        Some(_) => diagnostics.push("artifact_record_count_mismatch"),
        None => diagnostics.push("artifact_record_count_missing"),
    }
}

fn production_shadow_artifact_record_count(format: &str, bytes: &[u8]) -> Option<u64> {
    match format {
        "json" => serde_json::from_slice::<Value>(bytes).ok().map(|_| 1),
        "jsonl" => Some(
            String::from_utf8_lossy(bytes)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count() as u64,
        ),
        _ => None,
    }
}

fn production_shadow_manifest_summary_has_boundary_violation(manifest: &Value) -> bool {
    let summary = manifest.get("summary").unwrap_or(&Value::Null);
    json_bool(summary, "raw_secret_recorded")
        || json_bool(summary, "raw_payload_recorded")
        || json_bool(summary, "dashboard_order_controls_enabled")
        || summary
            .get("production_orders_submitted")
            .and_then(Value::as_u64)
            .is_some_and(|value| value > 0)
        || summary
            .get("production_order_mutations_attempted")
            .and_then(Value::as_u64)
            .is_some_and(|value| value > 0)
        || summary
            .get("actual_submission_count")
            .and_then(Value::as_u64)
            .is_some_and(|value| value > 0)
}

fn degraded_production_shadow_manifest(
    diagnostic: impl Into<String>,
) -> ProductionShadowManifestAudit {
    ProductionShadowManifestAudit {
        status: DashboardValue::available("production_shadow_manifest_degraded".to_string()),
        artifact_count: DashboardValue::unknown(),
        boundary_violation: true,
        diagnostic: Some(diagnostic.into()),
    }
}

fn checksum_bytes(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn read_json_file_value(path: &FsPath) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn read_latest_jsonl_file_value(path: &FsPath) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    raw.lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(|line| serde_json::from_str(line).ok())
}

fn first_dashboard_string_field<'a>(
    values: impl IntoIterator<Item = &'a Option<Value>>,
    field: &str,
) -> DashboardValue<String> {
    values
        .into_iter()
        .filter_map(Option::as_ref)
        .find_map(|value| value.get(field).and_then(Value::as_str))
        .map(str::to_string)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn latest_signal_label(value: &Value) -> DashboardValue<String> {
    let signal = value.get("signal").and_then(Value::as_str);
    let symbol = value.get("symbol").and_then(Value::as_str);
    let generated_at = value.get("generated_at").and_then(Value::as_str);
    match (signal, symbol, generated_at) {
        (Some(signal), Some(symbol), Some(generated_at)) => {
            DashboardValue::available(format!("{signal} {symbol} @ {generated_at}"))
        }
        (Some(signal), Some(symbol), None) => {
            DashboardValue::available(format!("{signal} {symbol}"))
        }
        (Some(signal), None, _) => DashboardValue::available(signal.to_string()),
        _ => DashboardValue::unknown(),
    }
}

fn latest_order_intent_label(value: &Value) -> DashboardValue<String> {
    let side = value.get("side").and_then(Value::as_str);
    let order_type = value.get("order_type").and_then(Value::as_str);
    let symbol = value.get("symbol").and_then(Value::as_str);
    let submission_status = value.get("submission_status").and_then(Value::as_str);
    let submission_allowed = value.get("submission_allowed").and_then(Value::as_bool);
    match (
        side,
        order_type,
        symbol,
        submission_status,
        submission_allowed,
    ) {
        (Some(side), Some(order_type), Some(symbol), Some(status), Some(allowed)) => {
            DashboardValue::available(format!(
                "{side} {order_type} {symbol}; status={status}; allowed={allowed}"
            ))
        }
        (Some(side), Some(order_type), Some(symbol), Some(status), None) => {
            DashboardValue::available(format!("{side} {order_type} {symbol}; status={status}"))
        }
        (Some(side), Some(order_type), Some(symbol), _, _) => {
            DashboardValue::available(format!("{side} {order_type} {symbol}"))
        }
        _ => DashboardValue::unknown(),
    }
}

fn latest_risk_decision_label(value: &Value) -> DashboardValue<String> {
    let decision = value.get("decision").and_then(Value::as_str);
    let mode = value.get("mode").and_then(Value::as_str);
    let actual_submission = value.get("actual_submission").and_then(Value::as_bool);
    match (decision, mode, actual_submission) {
        (Some(decision), Some(mode), Some(actual_submission)) => DashboardValue::available(
            format!("{decision}; mode={mode}; actual_submission={actual_submission}"),
        ),
        (Some(decision), Some(mode), None) => {
            DashboardValue::available(format!("{decision}; mode={mode}"))
        }
        (Some(decision), None, _) => DashboardValue::available(decision.to_string()),
        _ => DashboardValue::unknown(),
    }
}

fn json_string_array_field(value: &Value, field: &str) -> DashboardValue<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|joined| !joined.is_empty())
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn dashboard_path_if_exists(path: &FsPath) -> DashboardValue<String> {
    if path.exists() {
        DashboardValue::available(path.display().to_string())
    } else {
        DashboardValue::unknown()
    }
}

fn jsonl_record_count(path: &FsPath) -> DashboardValue<u64> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return DashboardValue::unknown(),
    };
    DashboardValue::available(raw.lines().filter(|line| !line.trim().is_empty()).count() as u64)
}

fn jsonl_record_count_matching(path: &FsPath, field: &str, expected: &str) -> DashboardValue<u64> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return DashboardValue::unknown(),
    };
    let mut count = 0_u64;
    for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if serde_json::from_str::<Value>(line)
            .ok()
            .and_then(|value| value.get(field).and_then(Value::as_str).map(str::to_string))
            .as_deref()
            == Some(expected)
        {
            count += 1;
        }
    }
    DashboardValue::available(count)
}

fn workflow_artifacts_from_explicit_root(
    workflow_root: &FsPath,
    gaps: &mut Vec<DashboardGap>,
) -> Vec<WorkflowArtifactStatus> {
    let mut manifest_paths = Vec::new();
    collect_workflow_manifest_paths(workflow_root, &mut manifest_paths, gaps);
    workflow_statuses_from_manifest_paths(manifest_paths, gaps)
}

fn workflow_artifacts_from_paths(
    registry_path: &FsPath,
    workflow_root: Option<&FsPath>,
    gaps: &mut Vec<DashboardGap>,
) -> Vec<WorkflowArtifactStatus> {
    let mut manifest_paths = Vec::new();
    if let Some(dir) = workflow_root {
        collect_workflow_manifest_paths(dir, &mut manifest_paths, gaps);
    }
    for dir in workflow_artifact_candidate_dirs(registry_path) {
        collect_workflow_manifest_paths(&dir, &mut manifest_paths, gaps);
    }
    workflow_statuses_from_manifest_paths(manifest_paths, gaps)
}

fn workflow_statuses_from_manifest_paths(
    mut manifest_paths: Vec<PathBuf>,
    gaps: &mut Vec<DashboardGap>,
) -> Vec<WorkflowArtifactStatus> {
    manifest_paths.sort();
    manifest_paths.dedup();

    let mut statuses = Vec::with_capacity(manifest_paths.len());
    for manifest_path in manifest_paths {
        statuses.push(read_workflow_manifest_status(&manifest_path, gaps));
    }
    statuses.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    statuses
}

fn workflow_artifact_candidate_dirs(registry_path: &FsPath) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(parent) = registry_path.parent() {
        dirs.push(parent.join("workflows"));
        if let Some(root) = parent.parent() {
            dirs.push(root.join("workflows"));
        }
    }
    dirs.push(PathBuf::from("runs/workflows"));
    dirs.sort();
    dirs.dedup();
    dirs
}

fn collect_workflow_manifest_paths(
    dir: &FsPath,
    manifest_paths: &mut Vec<PathBuf>,
    gaps: &mut Vec<DashboardGap>,
) {
    if !dir.exists() {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            gaps.push(DashboardGap::new(
                "workflow_artifacts",
                DashboardAvailability::Unknown,
                "V05-006",
                format!("读取 workflow 目录 '{}' 失败：{error}", dir.display()),
            ));
            return;
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            let manifest = path.join("manifest.json");
            if manifest.exists() {
                manifest_paths.push(manifest);
            }
        } else if path.file_name().and_then(|name| name.to_str()) == Some("manifest.json") {
            manifest_paths.push(path);
        }
    }
}

fn read_workflow_manifest_status(
    manifest_path: &FsPath,
    gaps: &mut Vec<DashboardGap>,
) -> WorkflowArtifactStatus {
    let raw = match fs::read_to_string(manifest_path) {
        Ok(raw) => raw,
        Err(error) => {
            gaps.push(workflow_manifest_gap(
                manifest_path,
                format!("读取 manifest 失败：{error}"),
            ));
            return WorkflowArtifactStatus::unknown(
                manifest_path.display().to_string(),
                "读取 workflow manifest 失败",
            );
        }
    };

    match serde_json::from_str::<WorkflowManifest>(&raw) {
        Ok(manifest) => workflow_status_from_manifest(manifest_path, manifest, gaps),
        Err(error) => {
            gaps.push(workflow_manifest_gap(
                manifest_path,
                format!("manifest 无效：{error}"),
            ));
            WorkflowArtifactStatus::unknown(
                manifest_path.display().to_string(),
                "workflow manifest JSON 无效",
            )
        }
    }
}

fn workflow_status_from_manifest(
    manifest_path: &FsPath,
    manifest: WorkflowManifest,
    gaps: &mut Vec<DashboardGap>,
) -> WorkflowArtifactStatus {
    let child_audit = audit_workflow_manifest_artifacts(manifest_path, &manifest, gaps);
    let probe_artifacts = read_workflow_probe_artifacts(manifest_path, &manifest);
    let summary = manifest.summary;
    let testnet_connection = probe_artifacts
        .testnet_connection
        .unwrap_or(summary.testnet_connection);
    let network_attempted = probe_artifacts
        .network_attempted
        .unwrap_or(summary.network_attempted);
    let testnet_public_network_connection =
        summary.testnet_public_network_connection || testnet_connection;
    let external_network_attempted = summary.external_network_attempted || network_attempted;
    let product_health = if summary.production_venue_connection
        || summary.external_venue_connection
        || summary.real_funds
        || summary.production_trading
        || summary.real_orders_submitted
        || !summary.sandbox_only
    {
        HealthStatus::Error
    } else {
        HealthStatus::Healthy
    };
    let health = strongest_health(product_health, child_audit.health);

    WorkflowArtifactStatus {
        run_id: manifest.run_id,
        workflow: manifest.workflow,
        workflow_id: DashboardValue::available(manifest.workflow_id),
        schema_version: manifest.schema_version,
        runtime_status: manifest.runtime_status,
        health,
        manifest_path: manifest_path.display().to_string(),
        artifact_count: u64::try_from(manifest.artifact_count).unwrap_or(u64::MAX),
        market_fixture_id: non_empty_dashboard_value(summary.market_fixture_id),
        order_lifecycle_id: non_empty_dashboard_value(summary.order_lifecycle_id),
        risk_smoke_id: non_empty_dashboard_value(summary.risk_smoke_id),
        sandbox_only: summary.sandbox_only,
        fixture_replay: summary.fixture_replay,
        mock_execution: summary.mock_execution,
        external_venue_connection: summary.external_venue_connection,
        production_venue_connection: summary.production_venue_connection,
        testnet_public_network_connection,
        external_network_attempted,
        real_funds: summary.real_funds,
        production_trading: summary.production_trading,
        real_orders_submitted: summary.real_orders_submitted,
        testnet_connection,
        network_permission_requested: probe_artifacts
            .network_permission_requested
            .unwrap_or(summary.network_permission_requested),
        network_attempted,
        credential_policy: non_empty_dashboard_value(summary.credential_policy),
        connectivity_mode: non_empty_dashboard_value(summary.connectivity_mode),
        order_submission_mode: non_empty_dashboard_value(summary.order_submission_mode),
        reconciliation_mode: non_empty_dashboard_value(summary.reconciliation_mode),
        probe_status: probe_artifacts.probe_status,
        probe_latency_ms: probe_artifacts.probe_latency_ms,
        probe_endpoint_class: probe_artifacts.probe_endpoint_class,
        probe_error_code: probe_artifacts.probe_error_code,
        values_recorded: probe_artifacts.values_recorded,
        secrets_redacted: probe_artifacts.secrets_redacted,
        authenticated_probe_status: probe_artifacts.authenticated_probe_status,
        authenticated_endpoint_kind: probe_artifacts.authenticated_endpoint_kind,
        authenticated_request_method: probe_artifacts.authenticated_request_method,
        authenticated_response_shape: probe_artifacts.authenticated_response_shape,
        authenticated_response_shape_validated: probe_artifacts
            .authenticated_response_shape_validated,
        authenticated_api_key_present: probe_artifacts.authenticated_api_key_present,
        authenticated_api_secret_present: probe_artifacts.authenticated_api_secret_present,
        authenticated_secrets_redacted: probe_artifacts.authenticated_secrets_redacted,
        authenticated_account_mutation: probe_artifacts.authenticated_account_mutation,
        authenticated_real_orders_submitted: probe_artifacts.authenticated_real_orders_submitted,
        authenticated_production_venue_connection: probe_artifacts
            .authenticated_production_venue_connection,
        order_proof_risk_preflight_status: probe_artifacts.order_proof_risk_preflight_status,
        order_proof_order_test_status: probe_artifacts.order_proof_order_test_status,
        order_proof_submit_ack_status: probe_artifacts.order_proof_submit_ack_status,
        order_proof_cancel_ack_status: probe_artifacts.order_proof_cancel_ack_status,
        order_proof_terminal_status: probe_artifacts.order_proof_terminal_status,
        order_proof_reconciliation_status: probe_artifacts.order_proof_reconciliation_status,
        order_proof_manual_submit_cancel_observed: probe_artifacts
            .order_proof_manual_submit_cancel_observed,
        order_proof_testnet_orders_submitted: probe_artifacts.order_proof_testnet_orders_submitted,
        order_proof_testnet_orders_canceled: probe_artifacts.order_proof_testnet_orders_canceled,
        order_proof_production_orders_submitted: probe_artifacts
            .order_proof_production_orders_submitted,
        order_proof_production_orders_canceled: probe_artifacts
            .order_proof_production_orders_canceled,
        order_proof_dashboard_order_controls: probe_artifacts.order_proof_dashboard_order_controls,
        websocket_probe_status: probe_artifacts.websocket_probe_status,
        websocket_error_code: probe_artifacts.websocket_error_code,
        websocket_attempted: probe_artifacts.websocket_attempted,
        websocket_subscription_attempted: probe_artifacts.websocket_subscription_attempted,
        websocket_message_count: probe_artifacts.websocket_message_count,
        diagnostic: DashboardValue::available(child_audit.diagnostic),
    }
}

struct WorkflowProbeArtifactStatus {
    network_permission_requested: Option<bool>,
    network_attempted: Option<bool>,
    testnet_connection: Option<bool>,
    probe_status: DashboardValue<String>,
    probe_latency_ms: DashboardValue<u64>,
    probe_endpoint_class: DashboardValue<String>,
    probe_error_code: DashboardValue<String>,
    values_recorded: DashboardValue<bool>,
    secrets_redacted: DashboardValue<bool>,
    authenticated_probe_status: DashboardValue<String>,
    authenticated_endpoint_kind: DashboardValue<String>,
    authenticated_request_method: DashboardValue<String>,
    authenticated_response_shape: DashboardValue<String>,
    authenticated_response_shape_validated: DashboardValue<bool>,
    authenticated_api_key_present: DashboardValue<bool>,
    authenticated_api_secret_present: DashboardValue<bool>,
    authenticated_secrets_redacted: DashboardValue<bool>,
    authenticated_account_mutation: DashboardValue<bool>,
    authenticated_real_orders_submitted: DashboardValue<bool>,
    authenticated_production_venue_connection: DashboardValue<bool>,
    order_proof_risk_preflight_status: DashboardValue<String>,
    order_proof_order_test_status: DashboardValue<String>,
    order_proof_submit_ack_status: DashboardValue<String>,
    order_proof_cancel_ack_status: DashboardValue<String>,
    order_proof_terminal_status: DashboardValue<String>,
    order_proof_reconciliation_status: DashboardValue<String>,
    order_proof_manual_submit_cancel_observed: DashboardValue<bool>,
    order_proof_testnet_orders_submitted: DashboardValue<u64>,
    order_proof_testnet_orders_canceled: DashboardValue<u64>,
    order_proof_production_orders_submitted: DashboardValue<u64>,
    order_proof_production_orders_canceled: DashboardValue<u64>,
    order_proof_dashboard_order_controls: DashboardValue<bool>,
    websocket_probe_status: DashboardValue<String>,
    websocket_error_code: DashboardValue<String>,
    websocket_attempted: bool,
    websocket_subscription_attempted: bool,
    websocket_message_count: DashboardValue<u64>,
}

impl WorkflowProbeArtifactStatus {
    fn unknown() -> Self {
        Self {
            network_permission_requested: None,
            network_attempted: None,
            testnet_connection: None,
            probe_status: DashboardValue::unknown(),
            probe_latency_ms: DashboardValue::unknown(),
            probe_endpoint_class: DashboardValue::unknown(),
            probe_error_code: DashboardValue::unknown(),
            values_recorded: DashboardValue::unknown(),
            secrets_redacted: DashboardValue::unknown(),
            authenticated_probe_status: DashboardValue::unknown(),
            authenticated_endpoint_kind: DashboardValue::unknown(),
            authenticated_request_method: DashboardValue::unknown(),
            authenticated_response_shape: DashboardValue::unknown(),
            authenticated_response_shape_validated: DashboardValue::unknown(),
            authenticated_api_key_present: DashboardValue::unknown(),
            authenticated_api_secret_present: DashboardValue::unknown(),
            authenticated_secrets_redacted: DashboardValue::unknown(),
            authenticated_account_mutation: DashboardValue::unknown(),
            authenticated_real_orders_submitted: DashboardValue::unknown(),
            authenticated_production_venue_connection: DashboardValue::unknown(),
            order_proof_risk_preflight_status: DashboardValue::unknown(),
            order_proof_order_test_status: DashboardValue::unknown(),
            order_proof_submit_ack_status: DashboardValue::unknown(),
            order_proof_cancel_ack_status: DashboardValue::unknown(),
            order_proof_terminal_status: DashboardValue::unknown(),
            order_proof_reconciliation_status: DashboardValue::unknown(),
            order_proof_manual_submit_cancel_observed: DashboardValue::unknown(),
            order_proof_testnet_orders_submitted: DashboardValue::unknown(),
            order_proof_testnet_orders_canceled: DashboardValue::unknown(),
            order_proof_production_orders_submitted: DashboardValue::unknown(),
            order_proof_production_orders_canceled: DashboardValue::unknown(),
            order_proof_dashboard_order_controls: DashboardValue::unknown(),
            websocket_probe_status: DashboardValue::unknown(),
            websocket_error_code: DashboardValue::unknown(),
            websocket_attempted: false,
            websocket_subscription_attempted: false,
            websocket_message_count: DashboardValue::unknown(),
        }
    }
}

fn read_workflow_probe_artifacts(
    manifest_path: &FsPath,
    manifest: &WorkflowManifest,
) -> WorkflowProbeArtifactStatus {
    let Some(manifest_dir) = manifest_path.parent() else {
        return WorkflowProbeArtifactStatus::unknown();
    };
    let mut status = WorkflowProbeArtifactStatus::unknown();

    for artifact in &manifest.artifacts {
        let artifact_path = manifest_dir.join(&artifact.path);
        let Ok(raw) = fs::read_to_string(&artifact_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let schema_version = value
            .get("schema_version")
            .and_then(Value::as_str)
            .unwrap_or(artifact.schema_version.as_str());
        apply_order_proof_artifact_status(
            &mut status,
            artifact.path.as_str(),
            schema_version,
            &value,
        );
        match artifact.path.as_str() {
            TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH => {
                status.network_permission_requested = value
                    .get("network_permission_requested")
                    .and_then(Value::as_bool);
                status.network_attempted = value.get("network_attempted").and_then(Value::as_bool);
                status.testnet_connection =
                    value.get("testnet_connection").and_then(Value::as_bool);
                status.probe_status = json_string_field(&value, "status");
                status.probe_latency_ms = json_u64_field(&value, "latency_ms");
                status.probe_endpoint_class = json_string_field(&value, "endpoint_class");
                status.probe_error_code = json_string_field(&value, "error_code");
            }
            TESTNET_HTTP_CONNECTIVITY_PROBE_ARTIFACT_PATH => {
                status.network_permission_requested = value
                    .get("network_permission_requested")
                    .and_then(Value::as_bool);
                status.network_attempted = value.get("network_attempted").and_then(Value::as_bool);
                status.testnet_connection =
                    value.get("testnet_connection").and_then(Value::as_bool);
                status.probe_status = json_string_field(&value, "status");
                status.probe_latency_ms = json_u64_field(&value, "latency_ms");
                status.probe_endpoint_class = json_string_field(&value, "endpoint_kind");
                status.probe_error_code = json_string_field(&value, "error_code");
            }
            "testnet/credential_policy.json" => {
                status.values_recorded = json_bool_field(&value, "values_recorded");
                status.secrets_redacted = json_bool_field(&value, "secrets_redacted");
                status.authenticated_secrets_redacted = json_bool_field(&value, "secrets_redacted");
            }
            TESTNET_AUTHENTICATED_READONLY_PROBE_ARTIFACT_PATH => {
                status.network_permission_requested = merge_optional_bool_or(
                    status.network_permission_requested,
                    value
                        .get("network_permission_requested")
                        .and_then(Value::as_bool),
                );
                status.network_attempted = merge_optional_bool_or(
                    status.network_attempted,
                    value.get("network_attempted").and_then(Value::as_bool),
                );
                status.testnet_connection = merge_optional_bool_or(
                    status.testnet_connection,
                    value.get("testnet_connection").and_then(Value::as_bool),
                );
                status.authenticated_probe_status = json_string_field(&value, "status");
                status.authenticated_endpoint_kind = json_string_field(&value, "endpoint_kind");
                status.authenticated_request_method = json_string_field(&value, "request_method");
                status.authenticated_response_shape = json_string_field(&value, "response_shape");
                status.authenticated_response_shape_validated =
                    json_bool_field(&value, "response_shape_validated");
                status.authenticated_api_key_present = json_bool_field(&value, "api_key_present");
                status.authenticated_api_secret_present =
                    json_bool_field(&value, "api_secret_present");
                status.authenticated_account_mutation = json_bool_field(&value, "account_mutation");
                status.authenticated_real_orders_submitted =
                    json_bool_field(&value, "real_orders_submitted");
                status.authenticated_production_venue_connection =
                    json_bool_field(&value, "production_venue_connection");
            }
            TESTNET_WEBSOCKET_PROBE_ARTIFACT_PATH => {
                status.websocket_probe_status = json_string_field(&value, "status");
                status.websocket_error_code = json_string_field(&value, "error_code");
                status.websocket_attempted = json_bool(&value, "websocket_attempted");
                status.websocket_subscription_attempted =
                    json_bool(&value, "subscription_attempted");
                status.websocket_message_count = json_u64_field(&value, "message_count");
            }
            _ => {}
        }
    }

    status
}

fn apply_order_proof_artifact_status(
    status: &mut WorkflowProbeArtifactStatus,
    path: &str,
    schema_version: &str,
    value: &Value,
) {
    match schema_version {
        "ntpro.v100_order_preflight_report.v1" => {
            status.order_proof_risk_preflight_status = json_string_field(value, "status");
            status.order_proof_dashboard_order_controls =
                json_bool_field(value, "dashboard_order_controls");
        }
        "ntpro.v100_order_test_preflight_report.v1" => {
            status.order_proof_order_test_status = json_string_field(value, "status");
            status.order_proof_dashboard_order_controls =
                json_bool_field(value, "dashboard_order_controls");
        }
        "ntpro.v100_execution_artifact_contract.v1" => {
            status.order_proof_order_test_status =
                nested_json_string_field(value, "order_test_artifact", "status");
            status.order_proof_submit_ack_status =
                nested_json_string_field(value, "submit_ack_artifact", "status");
            status.order_proof_cancel_ack_status =
                nested_json_string_field(value, "cancel_ack_artifact", "status");
            status.order_proof_terminal_status =
                nested_json_string_field(value, "lifecycle_artifact", "status");
            status.order_proof_reconciliation_status =
                nested_json_string_field(value, "reconciliation_artifact", "status");
            status.order_proof_manual_submit_cancel_observed =
                json_bool_field(value, "manual_submit_cancel_proof_observed");
            status.order_proof_testnet_orders_submitted =
                nested_json_u64_field(value, "counters", "testnet_orders_submitted");
            status.order_proof_testnet_orders_canceled =
                nested_json_u64_field(value, "counters", "testnet_orders_canceled");
            status.order_proof_production_orders_submitted =
                nested_json_u64_field(value, "counters", "production_orders_submitted");
            status.order_proof_production_orders_canceled =
                nested_json_u64_field(value, "counters", "production_orders_canceled");
            status.order_proof_dashboard_order_controls =
                json_bool_field(value, "dashboard_order_controls");
        }
        "ntpro.v100_reconciliation_fixture_report.v1" | "ntpro.v100_reconciliation_artifact.v1" => {
            status.order_proof_reconciliation_status = json_string_field(value, "status");
            status.order_proof_testnet_orders_submitted = first_available_u64(
                nested_json_u64_field(value, "counters", "testnet_orders_submitted"),
                json_u64_field(value, "testnet_orders_submitted"),
            );
            status.order_proof_testnet_orders_canceled = first_available_u64(
                nested_json_u64_field(value, "counters", "testnet_orders_canceled"),
                json_u64_field(value, "testnet_orders_canceled"),
            );
            status.order_proof_production_orders_submitted = first_available_u64(
                nested_json_u64_field(value, "counters", "production_orders_submitted"),
                json_u64_field(value, "production_orders_submitted"),
            );
            status.order_proof_production_orders_canceled = first_available_u64(
                nested_json_u64_field(value, "counters", "production_orders_canceled"),
                json_u64_field(value, "production_orders_canceled"),
            );
            status.order_proof_manual_submit_cancel_observed =
                json_bool_field(value, "manual_submit_cancel_proof_observed");
            status.order_proof_dashboard_order_controls =
                json_bool_field(value, "dashboard_order_controls");
        }
        "ntpro.v100_submit_ack_artifact.v1" => {
            status.order_proof_submit_ack_status = json_string_field(value, "status");
        }
        "ntpro.v100_cancel_ack_artifact.v1" => {
            status.order_proof_cancel_ack_status = json_string_field(value, "status");
        }
        "ntpro.v100_order_lifecycle_artifact.v1" => {
            status.order_proof_terminal_status = json_string_field(value, "status");
        }
        _ => apply_order_proof_path_fallback(status, path, value),
    }
}

fn apply_order_proof_path_fallback(
    status: &mut WorkflowProbeArtifactStatus,
    path: &str,
    value: &Value,
) {
    if path.ends_with("risk_preflight.json") {
        status.order_proof_risk_preflight_status = json_string_field(value, "status");
    } else if path.ends_with("order_test.json") || path.ends_with("order-test-preflight.json") {
        status.order_proof_order_test_status = json_string_field(value, "status");
    } else if path.ends_with("submit_ack.json") {
        status.order_proof_submit_ack_status = json_string_field(value, "status");
    } else if path.ends_with("cancel_ack.json") {
        status.order_proof_cancel_ack_status = json_string_field(value, "status");
    } else if path.ends_with("lifecycle.json") {
        status.order_proof_terminal_status = json_string_field(value, "status");
    } else if path.ends_with("reconciliation.json") {
        status.order_proof_reconciliation_status = json_string_field(value, "status");
    }
}

struct WorkflowArtifactAudit {
    health: HealthStatus,
    diagnostic: String,
}

fn audit_workflow_manifest_artifacts(
    manifest_path: &FsPath,
    manifest: &WorkflowManifest,
    gaps: &mut Vec<DashboardGap>,
) -> WorkflowArtifactAudit {
    let Some(manifest_dir) = manifest_path.parent() else {
        return WorkflowArtifactAudit {
            health: HealthStatus::Degraded,
            diagnostic: "workflow manifest parent directory unavailable".to_string(),
        };
    };
    if manifest.artifacts.is_empty() {
        return WorkflowArtifactAudit {
            health: HealthStatus::Healthy,
            diagnostic: "workflow manifest loaded; child artifact audit skipped".to_string(),
        };
    }

    let mut health = HealthStatus::Healthy;
    let mut checked = 0_u64;
    for artifact in &manifest.artifacts {
        checked += 1;
        if let Err(message) = audit_workflow_child_artifact(manifest_dir, artifact) {
            health = strongest_health(health, HealthStatus::Degraded);
            gaps.push(workflow_child_artifact_gap(
                manifest_path,
                artifact,
                message,
            ));
        }
    }

    WorkflowArtifactAudit {
        health,
        diagnostic: if health == HealthStatus::Healthy {
            format!("workflow manifest loaded; child_artifacts={checked} ok")
        } else {
            format!("workflow manifest loaded; child_artifacts={checked} degraded")
        },
    }
}

fn audit_workflow_child_artifact(
    manifest_dir: &FsPath,
    artifact: &WorkflowManifestArtifact,
) -> Result<(), String> {
    if artifact.path.trim().is_empty() {
        return Err("artifact path is empty".to_string());
    }
    let artifact_path = manifest_dir.join(&artifact.path);
    if !artifact_path.is_file() {
        return Err(format!("artifact missing: {}", artifact_path.display()));
    }

    let raw = fs::read_to_string(&artifact_path)
        .map_err(|error| format!("读取 artifact 失败：{}: {error}", artifact_path.display()))?;
    if artifact.path.ends_with(".jsonl") {
        return audit_workflow_jsonl_artifact(&raw, &artifact.schema_version, &artifact_path);
    }
    audit_workflow_json_artifact(&raw, &artifact.schema_version, &artifact_path)
}

fn audit_workflow_json_artifact(
    raw: &str,
    expected_schema_version: &str,
    artifact_path: &FsPath,
) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|error| format!("artifact JSON 无效：{}: {error}", artifact_path.display()))?;
    let actual = value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("missing");
    if actual != expected_schema_version {
        return Err(format!(
            "artifact schema_version mismatch: {} expected={} actual={actual}",
            artifact_path.display(),
            expected_schema_version
        ));
    }
    Ok(())
}

fn audit_workflow_jsonl_artifact(
    raw: &str,
    expected_schema_version: &str,
    artifact_path: &FsPath,
) -> Result<(), String> {
    let mut records = 0_u64;
    for (index, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        records += 1;
        let value: serde_json::Value = serde_json::from_str(line).map_err(|error| {
            format!(
                "artifact JSONL 无效：{} line {}: {error}",
                artifact_path.display(),
                index + 1
            )
        })?;
        let actual = value
            .get("schema_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("missing");
        if actual != expected_schema_version {
            return Err(format!(
                "artifact schema_version mismatch: {} line {} expected={} actual={actual}",
                artifact_path.display(),
                index + 1,
                expected_schema_version
            ));
        }
    }
    if records == 0 {
        return Err(format!(
            "artifact JSONL is empty: {}",
            artifact_path.display()
        ));
    }
    Ok(())
}

fn workflow_child_artifact_gap(
    manifest_path: &FsPath,
    artifact: &WorkflowManifestArtifact,
    notes: String,
) -> DashboardGap {
    DashboardGap::new(
        format!(
            "workflow_artifacts.{}.artifacts.{}",
            manifest_path.display(),
            artifact.path
        ),
        DashboardAvailability::Unknown,
        "V061-005",
        notes,
    )
}

fn workflow_manifest_gap(manifest_path: &FsPath, notes: String) -> DashboardGap {
    DashboardGap::new(
        format!("workflow_artifacts.{}", manifest_path.display()),
        DashboardAvailability::Unknown,
        "V05-006",
        notes,
    )
}

fn aggregate_risk_status(statuses: &[NodeStatus]) -> RiskStatus {
    if statuses.is_empty() {
        return RiskStatus::unknown();
    }

    let mut risk = RiskStatus {
        availability: DashboardAvailability::Available,
        trading_state: RiskTradingState::Unknown,
        health: HealthStatus::Unknown,
        command_count: DashboardValue::unknown(),
        event_count: DashboardValue::unknown(),
        rejections_total: DashboardValue::unknown(),
        last_rejection: DashboardValue::unknown(),
        last_error: DashboardValue::unknown(),
    };

    let mut commands = 0;
    let mut events = 0;
    let mut rejections = 0;
    let mut has_commands = false;
    let mut has_events = false;
    let mut has_rejections = false;

    for status in statuses {
        risk.trading_state = strongest_trading_state(risk.trading_state, status.risk.trading_state);
        risk.health = strongest_health(risk.health, status.risk.health);
        if let Some(value) = status.risk.command_count.value {
            commands += value;
            has_commands = true;
        }
        if let Some(value) = status.risk.event_count.value {
            events += value;
            has_events = true;
        }
        if let Some(value) = status.risk.rejections_total.value {
            rejections += value;
            has_rejections = true;
        }
        if risk.last_rejection.value.is_none()
            && let Some(reason) = status.risk.last_rejection.clone()
        {
            risk.last_rejection = DashboardValue::available(RejectionSummary {
                reason: DashboardValue::available(reason),
                last_rejected_at: dashboard_value_from_snapshot(&status.generated_at),
            });
        }
        if risk.last_error.value.is_none()
            && let Some(error) = status.risk.last_error.clone()
        {
            risk.last_error = DashboardValue::available(error);
        }
    }

    if has_commands {
        risk.command_count = DashboardValue::available(commands);
    }
    if has_events {
        risk.event_count = DashboardValue::available(events);
    }
    if has_rejections {
        risk.rejections_total = DashboardValue::available(rejections);
    }
    risk
}

#[derive(Debug, Deserialize)]
struct DashboardMockOrderLifecycleEvent {
    event_type: String,
    client_order_id: String,
    order_status: String,
    reason: Option<String>,
}

#[derive(Debug, Default)]
struct DashboardMockOrderLifecycleSummary {
    event_count: u64,
    submitted_count: u64,
    accepted_count: u64,
    filled_count: u64,
    canceled_count: u64,
    rejected_count: u64,
    event_types: Vec<String>,
    rejected_client_order_id: Option<String>,
    rejected_order_status: Option<String>,
    rejected_reason: Option<String>,
}

fn sandbox_business_status_from_v04_evidence() -> SandboxBusinessStatus {
    build_sandbox_business_status_from_v04_evidence().unwrap_or_else(|error| {
        SandboxBusinessStatus::unknown(format!("V04 Binance sandbox evidence unavailable: {error}"))
    })
}

fn build_sandbox_business_status_from_v04_evidence() -> anyhow::Result<SandboxBusinessStatus> {
    let ema = v04_ema_smoke_from_csv(V04_BINANCE_SPOT_BARS_CSV)
        .context("failed to build V04 EMA sandbox smoke summary")?;
    let rsi = v04_rsi_smoke_from_csv(V04_BINANCE_SPOT_BARS_CSV)
        .context("failed to build V04 RSI sandbox smoke summary")?;
    let order = summarize_v04_mock_order_lifecycle(V04_BINANCE_MOCK_ORDER_LIFECYCLE_JSONL)
        .context("failed to build V04 mock order lifecycle summary")?;

    ensure!(
        ema.instrument_id == rsi.instrument_id,
        "EMA and RSI smokes use different instruments"
    );
    ensure!(
        ema.bar_type == rsi.bar_type,
        "EMA and RSI smokes use different bar types"
    );
    ensure!(
        ema.fixture_id == rsi.fixture_id && ema.fixture_checksum == rsi.fixture_checksum,
        "EMA and RSI smokes use different fixtures"
    );
    ensure!(
        ema.mock_lifecycle_id == V04_BINANCE_EMA_MOCK_LIFECYCLE_ID
            && rsi.mock_lifecycle_id == V04_BINANCE_EMA_MOCK_LIFECYCLE_ID,
        "strategy smokes do not point at the V04 mock lifecycle"
    );
    ensure!(
        ema.risk_smoke_id == V04_BINANCE_EMA_RISK_SMOKE_ID
            && rsi.risk_smoke_id == V04_BINANCE_EMA_RISK_SMOKE_ID,
        "strategy smokes do not point at the V04 risk rejection smoke"
    );
    ensure!(
        !ema.real_exchange_connection
            && !rsi.real_exchange_connection
            && !ema.real_orders_submitted
            && !rsi.real_orders_submitted,
        "V04 dashboard sandbox evidence must not report real venue or order activity"
    );

    let rejected_client_order_id = order
        .rejected_client_order_id
        .clone()
        .context("mock lifecycle does not contain a rejected order")?;
    let rejected_order_status = order
        .rejected_order_status
        .clone()
        .context("mock lifecycle rejected order has no status")?;
    let rejected_reason = order
        .rejected_reason
        .clone()
        .context("mock lifecycle rejected order has no reason")?;
    ensure!(
        rejected_client_order_id == V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID,
        "mock lifecycle rejected order does not match V04 risk smoke"
    );
    ensure!(
        rejected_reason == V04_BINANCE_RISK_REJECTION_FIXTURE_REASON,
        "mock lifecycle rejected reason does not match V04 risk fixture reason"
    );

    let mock_orders_requested = ema.mock_orders_requested as u64 + rsi.mock_orders_requested as u64;
    Ok(SandboxBusinessStatus {
        availability: DashboardAvailability::Available,
        exchange: SandboxExchangePanel {
            venue: DashboardValue::available("BINANCE".to_string()),
            instrument_id: DashboardValue::available(ema.instrument_id.clone()),
            bar_type: DashboardValue::available(ema.bar_type.clone()),
            fixture_id: DashboardValue::available(ema.fixture_id.clone()),
            fixture_checksum: DashboardValue::available(ema.fixture_checksum.clone()),
            bars_processed: DashboardValue::available(ema.bars_processed as u64),
            connection_mode: DashboardValue::available("fixture_replay".to_string()),
            external_venue_connection: ema.real_exchange_connection || rsi.real_exchange_connection,
        },
        strategies: vec![
            SandboxStrategyPanel {
                strategy_id: "ema".to_string(),
                strategy_name: DashboardValue::available(ema.strategy_name.clone()),
                smoke_id: DashboardValue::available(ema.smoke_id.clone()),
                runtime_status: DashboardValue::available("ema_smoke_ready".to_string()),
                signal_mode: DashboardValue::available(ema.signal_mode.clone()),
                bars_processed: DashboardValue::available(ema.bars_processed as u64),
                signals_emitted: DashboardValue::available(ema.signals_emitted as u64),
                mock_orders_requested: DashboardValue::available(ema.mock_orders_requested as u64),
                final_signal: DashboardValue::available(ema.final_signal.clone()),
                indicator_value: DashboardValue::available(format!(
                    "fast={} slow={}",
                    ema.final_fast_ema, ema.final_slow_ema
                )),
                checksum: DashboardValue::available(ema.checksum.clone()),
                real_orders_submitted: ema.real_orders_submitted,
            },
            SandboxStrategyPanel {
                strategy_id: "rsi".to_string(),
                strategy_name: DashboardValue::available(rsi.strategy_name.clone()),
                smoke_id: DashboardValue::available(rsi.smoke_id.clone()),
                runtime_status: DashboardValue::available("rsi_smoke_ready".to_string()),
                signal_mode: DashboardValue::available(format!(
                    "oversold={} overbought={}",
                    rsi.oversold_threshold, rsi.overbought_threshold
                )),
                bars_processed: DashboardValue::available(rsi.bars_processed as u64),
                signals_emitted: DashboardValue::available(rsi.signals_emitted as u64),
                mock_orders_requested: DashboardValue::available(rsi.mock_orders_requested as u64),
                final_signal: DashboardValue::available(rsi.final_signal.clone()),
                indicator_value: DashboardValue::available(format!("rsi={}", rsi.final_rsi)),
                checksum: DashboardValue::available(rsi.checksum.clone()),
                real_orders_submitted: rsi.real_orders_submitted,
            },
        ],
        order: SandboxOrderPanel {
            lifecycle_id: DashboardValue::available(V04_BINANCE_EMA_MOCK_LIFECYCLE_ID.to_string()),
            source_path: DashboardValue::available(
                V04_BINANCE_MOCK_ORDER_LIFECYCLE_PATH.to_string(),
            ),
            event_count: DashboardValue::available(order.event_count),
            submitted_count: DashboardValue::available(order.submitted_count),
            accepted_count: DashboardValue::available(order.accepted_count),
            filled_count: DashboardValue::available(order.filled_count),
            canceled_count: DashboardValue::available(order.canceled_count),
            rejected_count: DashboardValue::available(order.rejected_count),
            event_types: order.event_types,
            mock_orders_requested: DashboardValue::available(mock_orders_requested),
            real_orders_submitted: false,
            evidence_source: DashboardValue::available("V04-008".to_string()),
        },
        risk: SandboxRiskPanel {
            smoke_id: DashboardValue::available(V04_BINANCE_EMA_RISK_SMOKE_ID.to_string()),
            lifecycle_id: DashboardValue::available(V04_BINANCE_EMA_MOCK_LIFECYCLE_ID.to_string()),
            client_order_id: DashboardValue::available(rejected_client_order_id),
            fixture_reason: DashboardValue::available(rejected_reason),
            risk_reason: DashboardValue::available(V04_BINANCE_RISK_REJECTION_REASON.to_string()),
            order_status: DashboardValue::available(rejected_order_status),
            forwarded_to_execution: false,
            rejection_count: DashboardValue::available(order.rejected_count),
            real_orders_submitted: false,
            health: HealthStatus::Healthy,
        },
        diagnostic: DashboardValue::available(
            "V04 Binance sandbox fixture, strategy, order, and risk evidence loaded".to_string(),
        ),
    })
}

fn summarize_v04_mock_order_lifecycle(
    jsonl: &str,
) -> anyhow::Result<DashboardMockOrderLifecycleSummary> {
    let mut summary = DashboardMockOrderLifecycleSummary::default();
    let mut event_types = BTreeMap::<String, ()>::new();

    for (index, line) in jsonl.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let event: DashboardMockOrderLifecycleEvent = serde_json::from_str(trimmed)
            .with_context(|| format!("invalid V04 mock lifecycle JSONL at line {}", index + 1))?;
        summary.event_count += 1;
        event_types.insert(event.event_type.clone(), ());
        match event.event_type.as_str() {
            "order.submitted" => summary.submitted_count += 1,
            "order.accepted" => summary.accepted_count += 1,
            "order.filled" => summary.filled_count += 1,
            "order.canceled" => summary.canceled_count += 1,
            "order.rejected" => {
                summary.rejected_count += 1;
                summary.rejected_client_order_id = Some(event.client_order_id);
                summary.rejected_order_status = Some(event.order_status);
                summary.rejected_reason = event.reason;
            }
            event_type => anyhow::bail!("unsupported V04 mock lifecycle event type {event_type}"),
        }
    }

    ensure!(
        summary.event_count > 0,
        "V04 mock lifecycle JSONL has no events"
    );
    summary.event_types = event_types.into_keys().collect();
    Ok(summary)
}

fn control_statuses_from_nodes(nodes: &[DashboardNodeSummary]) -> Vec<ControlStatus> {
    let mut controls = Vec::with_capacity(nodes.len() * 6);
    for node in nodes {
        controls.push(ControlStatus {
            action: format!("start:{}", node.node_id),
            availability: DashboardAvailability::Available,
            enabled: node.lifecycle_state == LifecycleStatus::Stopped,
            reason: if node.lifecycle_state == LifecycleStatus::Running {
                DashboardValue::available("节点已经在运行".to_string())
            } else if node.lifecycle_state == LifecycleStatus::Stopped {
                DashboardValue::available("可以通过监督器控制启动该节点".to_string())
            } else {
                DashboardValue::available("节点不是已停止状态".to_string())
            },
        });
        controls.push(ControlStatus {
            action: format!("stop:{}", node.node_id),
            availability: DashboardAvailability::Available,
            enabled: matches!(
                node.lifecycle_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            ),
            reason: if matches!(
                node.lifecycle_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            ) {
                DashboardValue::available("可以通过监督器控制停止该节点".to_string())
            } else {
                DashboardValue::available("节点不是运行中或已暂停状态".to_string())
            },
        });
        controls.push(ControlStatus {
            action: format!("pause:{}", node.node_id),
            availability: DashboardAvailability::Available,
            enabled: node.lifecycle_state == LifecycleStatus::Running,
            reason: if node.lifecycle_state == LifecycleStatus::Running {
                DashboardValue::available("可以通过监督器控制暂停该节点".to_string())
            } else {
                DashboardValue::available("节点不是运行中状态".to_string())
            },
        });
        controls.push(ControlStatus {
            action: format!("resume:{}", node.node_id),
            availability: DashboardAvailability::Available,
            enabled: node.lifecycle_state == LifecycleStatus::Paused,
            reason: if node.lifecycle_state == LifecycleStatus::Paused {
                DashboardValue::available("可以通过监督器控制恢复该节点".to_string())
            } else {
                DashboardValue::available("节点不是已暂停状态".to_string())
            },
        });
        for (action, reason) in [
            (
                "reconnect_data",
                "本地沙盒仅记录数据源重连为不支持，不会连接真实交易所或真实 adapter",
            ),
            (
                "reconnect_execution",
                "本地沙盒仅记录执行网关重连为不支持，不会连接真实交易所或真实 adapter",
            ),
        ] {
            let control_available = matches!(
                node.lifecycle_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            );
            controls.push(ControlStatus {
                action: format!("{action}:{}", node.node_id),
                availability: DashboardAvailability::Available,
                enabled: control_available,
                reason: if control_available {
                    DashboardValue::available(reason.to_string())
                } else {
                    DashboardValue::available("重连控制要求节点处于运行中或已暂停状态".to_string())
                },
            });
        }
    }
    controls
}

fn alert_summary_from_gaps(gaps: &[DashboardGap]) -> AlertSummary {
    let mut summary = AlertSummary::default();
    for (index, gap) in gaps.iter().enumerate() {
        let severity = match gap.reason {
            DashboardAvailability::Stale => "warning",
            DashboardAvailability::NotSupported | DashboardAvailability::NotConfigured => "info",
            DashboardAvailability::Available
            | DashboardAvailability::Redacted
            | DashboardAvailability::Unknown => "warning",
        };
        *summary
            .counts_by_severity
            .entry(severity.to_string())
            .or_insert(0) += 1;
        summary.active.push(DashboardAlert {
            alert_id: format!("gap-{index}"),
            severity: severity.to_string(),
            source: gap.field_path.clone(),
            message: gap
                .notes
                .value
                .clone()
                .unwrap_or_else(|| "检测到 Dashboard 待补能力".to_string()),
            first_seen_at: DashboardValue::unknown(),
            last_seen_at: DashboardValue::unknown(),
        });
    }
    summary.active_count = summary.active.len() as u64;
    summary
}

fn dashboard_value_from_snapshot<T: Clone>(value: &SnapshotValue<T>) -> DashboardValue<T> {
    match value.availability {
        SnapshotAvailability::Available => value
            .value
            .clone()
            .map_or_else(DashboardValue::unknown, DashboardValue::available),
        SnapshotAvailability::NotConfigured => DashboardValue::not_configured(),
        SnapshotAvailability::NotSupported => DashboardValue::not_supported(),
        SnapshotAvailability::Stale => DashboardValue::stale(),
        SnapshotAvailability::Unknown => DashboardValue::unknown(),
    }
}

fn optional_dashboard_value(value: Option<String>) -> DashboardValue<String> {
    value.map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn non_empty_dashboard_value(value: String) -> DashboardValue<String> {
    if value.trim().is_empty() {
        DashboardValue::unknown()
    } else {
        DashboardValue::available(value)
    }
}

fn json_string_field(value: &Value, field: &str) -> DashboardValue<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn json_string_field_with_fallback(
    value: &Value,
    primary_field: &str,
    fallback_field: &str,
) -> DashboardValue<String> {
    let primary = json_string_field(value, primary_field);
    if primary.availability == DashboardAvailability::Available {
        primary
    } else {
        json_string_field(value, fallback_field)
    }
}

fn json_u64_field(value: &Value, field: &str) -> DashboardValue<u64> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn nested_json_string_field(value: &Value, object: &str, field: &str) -> DashboardValue<String> {
    value
        .get(object)
        .and_then(|nested| nested.get(field))
        .and_then(Value::as_str)
        .map(str::to_string)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn nested_json_u64_field(value: &Value, object: &str, field: &str) -> DashboardValue<u64> {
    value
        .get(object)
        .and_then(|nested| nested.get(field))
        .and_then(Value::as_u64)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn nested_json_bool_field(value: &Value, object: &str, field: &str) -> DashboardValue<bool> {
    value
        .get(object)
        .and_then(|nested| nested.get(field))
        .and_then(Value::as_bool)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn nested_json_bool_field_with_fallback(
    value: &Value,
    object: &str,
    field: &str,
    fallback: &str,
) -> DashboardValue<bool> {
    first_available_bool_from_values([
        nested_json_bool_field(value, object, field),
        nested_json_bool_field(value, object, fallback),
    ])
}

fn json_bool_field(value: &Value, field: &str) -> DashboardValue<bool> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn json_bool_field_with_fallback(
    value: &Value,
    field: &str,
    fallback: &str,
) -> DashboardValue<bool> {
    first_available_bool_from_values([
        json_bool_field(value, field),
        json_bool_field(value, fallback),
    ])
}

fn json_bool_as_u64_field(value: &Value, field: &str) -> DashboardValue<u64> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .map(u64::from)
        .map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn first_available_u64(
    primary: DashboardValue<u64>,
    fallback: DashboardValue<u64>,
) -> DashboardValue<u64> {
    if primary.availability == DashboardAvailability::Available {
        primary
    } else {
        fallback
    }
}

fn first_available_u64_from_values(
    values: impl IntoIterator<Item = DashboardValue<u64>>,
) -> DashboardValue<u64> {
    values
        .into_iter()
        .find(|value| value.availability == DashboardAvailability::Available)
        .unwrap_or_else(DashboardValue::unknown)
}

fn max_available_u64_from_values(
    values: impl IntoIterator<Item = DashboardValue<u64>>,
) -> DashboardValue<u64> {
    let mut max_value = None;
    for value in values {
        if value.availability != DashboardAvailability::Available {
            continue;
        }
        if let Some(value) = value.value {
            max_value = Some(max_value.map_or(value, |current: u64| current.max(value)));
        }
    }
    max_value.map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn first_available_bool_from_values(
    values: impl IntoIterator<Item = DashboardValue<bool>>,
) -> DashboardValue<bool> {
    values
        .into_iter()
        .find(|value| value.availability == DashboardAvailability::Available)
        .unwrap_or_else(DashboardValue::unknown)
}

fn first_available_string_from_values(
    values: impl IntoIterator<Item = DashboardValue<String>>,
) -> DashboardValue<String> {
    values
        .into_iter()
        .find(|value| value.availability == DashboardAvailability::Available)
        .unwrap_or_else(DashboardValue::unknown)
}

fn dashboard_string_available(value: &DashboardValue<String>) -> bool {
    value.availability == DashboardAvailability::Available && value.value.is_some()
}

fn dashboard_bool_is_false(value: &DashboardValue<bool>) -> bool {
    value.availability == DashboardAvailability::Available && value.value == Some(false)
}

fn any_available_bool_from_values(
    values: impl IntoIterator<Item = DashboardValue<bool>>,
) -> DashboardValue<bool> {
    let mut saw_available = false;
    for value in values {
        if value.availability != DashboardAvailability::Available {
            continue;
        }
        saw_available = true;
        if value.value == Some(true) {
            return DashboardValue::available(true);
        }
    }
    if saw_available {
        DashboardValue::available(false)
    } else {
        DashboardValue::unknown()
    }
}

fn json_bool(value: &Value, field: &str) -> bool {
    value.get(field).and_then(Value::as_bool).unwrap_or(false)
}

fn merge_optional_bool_or(current: Option<bool>, next: Option<bool>) -> Option<bool> {
    match (current, next) {
        (Some(left), Some(right)) => Some(left || right),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn redacted_optional_dashboard_error(value: Option<&str>) -> DashboardValue<String> {
    if value.is_some() {
        DashboardValue::available("present".to_string())
    } else {
        DashboardValue::unknown()
    }
}

fn availability_from_registry_state(state: RegistryArtifactState) -> DashboardAvailability {
    match state {
        RegistryArtifactState::Available => DashboardAvailability::Available,
        RegistryArtifactState::Missing
        | RegistryArtifactState::Invalid
        | RegistryArtifactState::Unknown => DashboardAvailability::Unknown,
        RegistryArtifactState::Stale => DashboardAvailability::Stale,
    }
}

fn health_from_process_state(state: SupervisorProcessState) -> HealthStatus {
    match state {
        SupervisorProcessState::Running | SupervisorProcessState::Stopped => HealthStatus::Healthy,
        SupervisorProcessState::Stale => HealthStatus::Stale,
        SupervisorProcessState::NotStarted | SupervisorProcessState::Unknown => {
            HealthStatus::Unknown
        }
    }
}

fn health_from_connection(connection: ConnectionStatus) -> HealthStatus {
    match connection {
        ConnectionStatus::Connected | ConnectionStatus::NotConfigured => HealthStatus::Healthy,
        ConnectionStatus::Connecting | ConnectionStatus::Disconnecting => HealthStatus::Degraded,
        ConnectionStatus::Disconnected | ConnectionStatus::Stale => HealthStatus::Stale,
        ConnectionStatus::NotSupported | ConnectionStatus::Unknown => HealthStatus::Unknown,
    }
}

fn strongest_health(current: HealthStatus, next: HealthStatus) -> HealthStatus {
    match (current, next) {
        (HealthStatus::Error, _) | (_, HealthStatus::Error) => HealthStatus::Error,
        (HealthStatus::Stale, _) | (_, HealthStatus::Stale) => HealthStatus::Stale,
        (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
        (HealthStatus::Healthy, _) | (_, HealthStatus::Healthy) => HealthStatus::Healthy,
        _ => HealthStatus::Unknown,
    }
}

fn strongest_trading_state(current: RiskTradingState, next: RiskTradingState) -> RiskTradingState {
    match (current, next) {
        (RiskTradingState::Halted, _) | (_, RiskTradingState::Halted) => RiskTradingState::Halted,
        (RiskTradingState::Reducing, _) | (_, RiskTradingState::Reducing) => {
            RiskTradingState::Reducing
        }
        (RiskTradingState::Active, _) | (_, RiskTradingState::Active) => RiskTradingState::Active,
        _ => RiskTradingState::Unknown,
    }
}

fn json_label<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn derive_node_health(status: &NodeStatus) -> HealthStatus {
    if status.last_error.is_some() {
        return HealthStatus::Error;
    }

    match status.lifecycle_state {
        LifecycleStatus::Running | LifecycleStatus::Stopped => HealthStatus::Healthy,
        LifecycleStatus::Error => HealthStatus::Error,
        LifecycleStatus::Unknown => HealthStatus::Unknown,
        LifecycleStatus::Starting
        | LifecycleStatus::Pausing
        | LifecycleStatus::Paused
        | LifecycleStatus::Resuming
        | LifecycleStatus::Stopping => HealthStatus::Degraded,
    }
}

fn derive_overview_health(overview: &DashboardOverview) -> HealthStatus {
    if overview.error_nodes > 0 || overview.latest_error.is_some() {
        HealthStatus::Error
    } else if overview.node_count == 0 || overview.unknown_nodes > 0 {
        HealthStatus::Unknown
    } else if overview.running_nodes > 0 {
        HealthStatus::Healthy
    } else {
        HealthStatus::Degraded
    }
}

#[cfg(test)]
#[path = "dashboard/tests.rs"]
mod tests;
