//! Backtest、Sandbox 与 Live 三环境 Run 的只读产品合同。

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    io::{Error as IoError, ErrorKind, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering as AtomicOrdering},
    thread,
    time::{Duration, Instant},
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
use nautilus_live::status::{LifecycleStatus, SnapshotAvailability, SnapshotValue};
use nautilus_model::types::{Money, Quantity};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    dashboard::ApiStatusResult,
    strategy_session::{StrategyMarketEventKind, StrategySessionState},
    supervisor::{
        NodeMetrics, StartNodeRequest, StopNodeRequest, SupervisorNodeRecord,
        SupervisorRegistryStore, SupervisorRunOwnership, SupervisorRunTerminalAnchor,
    },
};

use super::*;

const RUN_LIST_SCHEMA_VERSION: &str = "ntpro.product_api.run_list.response.v1";
const RUN_DETAIL_SCHEMA_VERSION: &str = "ntpro.product_api.run_detail.response.v1";
const RUN_METRICS_SCHEMA_VERSION: &str = "ntpro.product_api.run_metrics.response.v1";
const RUN_REPORT_SCHEMA_VERSION: &str = "ntpro.product_api.run_report.response.v1";
const RUN_CREATE_SCHEMA_VERSION: &str = "ntpro.product_api.run_create.response.v1";
const BACKTEST_RUN_MANIFEST_SCHEMA_VERSION: &str = "ntpro.product_api.backtest_run_manifest.v1";
const BACKTEST_RESULT_SCHEMA_VERSION: &str = "ntpro.backtest_result.v1";
const BACKTEST_DETAILS_SCHEMA_VERSION: &str = "ntpro.backtest_details.v1";
const BACKTEST_ANALYSIS_SCHEMA_VERSION: &str = "ntpro.backtest_analysis.v1";
const RUN_ANALYSIS_SCHEMA_VERSION: &str = "ntpro.product_api.run_analysis.response.v1";
const RUN_COMPARISON_SCHEMA_VERSION: &str = "ntpro.product_api.run_comparison.response.v2";
const RUN_REPRODUCTION_SCHEMA_VERSION: &str = "ntpro.product_api.run_reproduction.response.v1";
const RUN_REPRODUCTION_PROOF_SCHEMA_VERSION: &str =
    "ntpro.product_api.run_reproduction_proof.response.v1";
const BACKTEST_REPRODUCTION_PROOF_SCHEMA_VERSION: &str = "ntpro.backtest_reproduction_proof.v1";
const DEMO_RUN_CREATE_SCHEMA_VERSION: &str = "ntpro.product_api.demo_run_create.response.v1";
const DEMO_RUN_ACTION_SCHEMA_VERSION: &str = "ntpro.product_api.demo_run_action.response.v1";
const DEMO_RUN_MANIFEST_SCHEMA_VERSION: &str = "ntpro.product_api.demo_run_manifest.v1";
const DEMO_RUN_SNAPSHOT_SCHEMA_VERSION: &str = "ntpro.product_api.demo_run_snapshot.response.v2";
const DEMO_RUN_RESULT_SCHEMA_VERSION: &str = "ntpro.product_api.demo_run_result.v2";
const DEMO_RUN_TERMINAL_STATE_SCHEMA_VERSION: &str = "ntpro.product_api.demo_run_terminal_state.v2";
const RUN_CURSOR_PREFIX: &str = "run-v1-";
static RUN_MANIFEST_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct ProductRun {
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    environment: RunEnvironment,
    data_ref: String,
    config_ref: String,
    adapter_ref: String,
    account_ref: String,
    venue_ref: String,
    lifecycle: RunLifecycle,
    result: ProductRunResult,
    risk: ProductRunRisk,
    error: Option<ProductRunError>,
    created_at_unix_ms: u64,
    started_at_unix_ms: Option<u64>,
    completed_at_unix_ms: Option<u64>,
    updated_at_unix_ms: u64,
    source: ProductSource,
    capabilities: ProductRunCapabilities,
    runtime: Option<ProductRunRuntime>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct ProductRunRuntime {
    supervisor_node_id: String,
    strategy_instance_id: String,
    process_state: SupervisorProcessState,
    lifecycle_state: LifecycleStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RunEnvironment {
    Backtest,
    Sandbox,
    Live,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RunLifecycle {
    Created,
    Queued,
    Running,
    Stopping,
    Completed,
    Failed,
    Cancelled,
    Stopped,
    Paused,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RunResultStatus {
    Pending,
    Available,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RunRiskStatus {
    Pending,
    Active,
    Passed,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ProductRunResult {
    status: RunResultStatus,
    result_ref: Option<String>,
    report_ref: Option<String>,
    analysis_ref: Option<String>,
    reproduction_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ProductRunRisk {
    status: RunRiskStatus,
    risk_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ProductRunError {
    code: String,
    summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct ProductRunCapabilities {
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct ProductRunConfigProjection {
    product_runs: Vec<ProductRunConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProductRunConfig {
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    environment: RunEnvironment,
    data_ref: String,
    config_ref: String,
    adapter_ref: String,
    account_ref: String,
    venue_ref: String,
    lifecycle: RunLifecycle,
    result_status: RunResultStatus,
    result_ref: Option<String>,
    backtest_config_sha256: Option<String>,
    backtest_data_sha256: Option<String>,
    backtest_result_sha256: Option<String>,
    backtest_details_sha256: Option<String>,
    backtest_analysis_sha256: Option<String>,
    strategy_version_snapshot_sha256: Option<String>,
    reproduction_source_run_id: Option<String>,
    reproduction_input_sha256: Option<String>,
    reproduction_output_sha256: Option<String>,
    reproduction_proof_sha256: Option<String>,
    backtest_trade_size: Option<String>,
    backtest_quotes: Option<usize>,
    backtest_fast_period: Option<usize>,
    backtest_slow_period: Option<usize>,
    risk_status: RunRiskStatus,
    risk_ref: String,
    error_code: Option<String>,
    error_summary: Option<String>,
    created_at_unix_ms: u64,
    started_at_unix_ms: Option<u64>,
    completed_at_unix_ms: Option<u64>,
    updated_at_unix_ms: u64,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
    #[serde(default)]
    demo_supervisor_node_id: Option<String>,
    #[serde(default)]
    demo_strategy_instance_id: Option<String>,
    #[serde(default)]
    demo_identity_contract_id: Option<String>,
    #[serde(default)]
    demo_supervisor_record_baseline_unix_ms: Option<u64>,
    #[serde(skip)]
    demo_process_state: Option<SupervisorProcessState>,
    #[serde(skip)]
    demo_lifecycle_state: Option<LifecycleStatus>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::dashboard) struct CreateBacktestRunRequest {
    strategy_id: String,
    strategy_version_id: String,
    environment: RunEnvironment,
    data_ref: String,
    venue_ref: String,
    starting_balance: String,
    quotes: usize,
    trade_size: String,
    fast_period: usize,
    slow_period: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::dashboard) struct ReproduceBacktestRunRequest {
    source_run_id: String,
    deterministic_replay: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::dashboard) struct CreateDemoRunRequest {
    strategy_id: String,
    strategy_version_id: String,
    environment: RunEnvironment,
    supervisor_node_id: String,
    account_ref: String,
    venue_ref: String,
    user_confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::dashboard) struct DemoRunActionRequest {
    run_id: String,
    action: DemoRunAction,
    user_confirmed: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DemoRunAction {
    Start,
    Stop,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct BacktestRunCreationBoundaries {
    backtest_run_creation_allowed: bool,
    sandbox_run_creation_allowed: bool,
    live_run_creation_allowed: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

impl BacktestRunCreationBoundaries {
    const fn enforced() -> Self {
        Self {
            backtest_run_creation_allowed: true,
            sandbox_run_creation_allowed: false,
            live_run_creation_allowed: false,
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
struct DemoRunBoundaries {
    demo_run_creation_allowed: bool,
    demo_start_allowed: bool,
    demo_stop_allowed: bool,
    live_run_creation_allowed: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

impl DemoRunBoundaries {
    const fn enforced() -> Self {
        Self {
            demo_run_creation_allowed: true,
            demo_start_allowed: true,
            demo_stop_allowed: true,
            live_run_creation_allowed: false,
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
pub(in crate::dashboard) struct RunCreateResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: ProductRun,
    boundaries: BacktestRunCreationBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct DemoRunCreateResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: ProductRun,
    boundaries: DemoRunBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DemoRunActionResult {
    run_id: String,
    action: DemoRunAction,
    previous_lifecycle: RunLifecycle,
    current_run: ProductRun,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct DemoRunActionResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: DemoRunActionResult,
    boundaries: DemoRunBoundaries,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DynamicBacktestRunManifest {
    schema_version: String,
    request_sha256: String,
    config: ProductRunConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DynamicDemoRunManifest {
    schema_version: String,
    request_sha256: String,
    strategy_version_snapshot_sha256: String,
    config: ProductRunConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DynamicDemoRunTerminalState {
    schema_version: String,
    source_manifest_sha256: String,
    run_id: String,
    lifecycle: RunLifecycle,
    runtime: ProductRunRuntime,
    started_at_unix_ms: Option<u64>,
    completed_at_unix_ms: u64,
    updated_at_unix_ms: u64,
    demo_result_sha256: String,
    error_code: Option<String>,
    error_summary: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DemoSnapshotStatus {
    NotStarted,
    Running,
    Frozen,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DemoTechnicalHealthStatus {
    Healthy,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSnapshotRuntime {
    supervisor_node_id: String,
    strategy_instance_id: String,
    process_state: SupervisorProcessState,
    lifecycle_state: LifecycleStatus,
    data_connection: String,
    execution_connection: String,
    uptime_ms: Option<u64>,
    generated_at_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoMarketSnapshot {
    connection: String,
    state: String,
    source: String,
    event_count: u64,
    last_event_at_unix_ms: Option<u64>,
    updated_at_unix_ms: u64,
    latest_event: Option<DemoMarketEvent>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoMarketEvent {
    event_type: StrategyMarketEventKind,
    source: String,
    seq: u64,
    symbol: String,
    price: f64,
    event_at_unix_ms: u64,
    recorded_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSessionSnapshot {
    state: StrategySessionState,
    reason: String,
    event_count: u64,
    market_event_count: u64,
    signal_count: u64,
    intent_count: u64,
    risk_decision_count: u64,
    rejection_count: u64,
    actual_submission_count: u64,
    updated_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSignalSnapshot {
    symbol: String,
    signal: String,
    confidence: f64,
    market_event_seq: u64,
    generated_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoOrderIntentSnapshot {
    intent_id: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: f64,
    source_signal: String,
    confidence: f64,
    market_event_seq: u64,
    created_at_unix_ms: u64,
    submission_allowed: bool,
    submission_status: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoRiskDecisionSnapshot {
    decision_id: String,
    intent_id: String,
    symbol: String,
    decision: String,
    reasons: Vec<String>,
    mode: String,
    order_submission: String,
    kill_switch_enabled: bool,
    kill_switch_active: bool,
    account_state: String,
    market_state: String,
    actual_submission: bool,
    evaluated_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSimulationParameters {
    trade_size: String,
    fast_period: usize,
    slow_period: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSimulationArtifactBoundaries {
    simulation_only: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSimulationSummarySnapshot {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    instrument_id: String,
    engine: String,
    execution_mode: String,
    data_sha256: String,
    parameters: DemoSimulationParameters,
    fill_count: usize,
    position_count: usize,
    equity_point_count: usize,
    boundaries: DemoSimulationArtifactBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSimulatedFillSnapshot {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    simulation_only: bool,
    trade_id: String,
    client_order_id: String,
    venue_order_id: String,
    position_id: Option<String>,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    currency: String,
    liquidity_side: String,
    commission: Option<String>,
    ts_event: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSimulatedPositionSnapshot {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    simulation_only: bool,
    position_id: String,
    account_id: String,
    side: String,
    entry_side: String,
    peak_quantity: String,
    buy_quantity: String,
    sell_quantity: String,
    avg_price_open: String,
    avg_price_close: Option<String>,
    realized_return: String,
    realized_pnl: Option<String>,
    trade_count: usize,
    ts_opened: String,
    ts_closed: Option<String>,
    duration_ns: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoEquityPointSnapshot {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    simulation_only: bool,
    account_id: String,
    currency: String,
    total: String,
    free: String,
    locked: String,
    ts_event: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSimulationSnapshot {
    summary: DemoSimulationSummarySnapshot,
    fills: Vec<DemoSimulatedFillSnapshot>,
    positions: Vec<DemoSimulatedPositionSnapshot>,
    equity_curve: Vec<DemoEquityPointSnapshot>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoTechnicalHealth {
    status: DemoTechnicalHealthStatus,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSnapshotProvenance {
    source_refs: Vec<String>,
    manifest_sha256: Option<String>,
    result_ref: Option<String>,
    result_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoRunSnapshotData {
    schema_version: String,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    observed_at_unix_ms: u64,
    lifecycle: RunLifecycle,
    snapshot_status: DemoSnapshotStatus,
    runtime: DemoSnapshotRuntime,
    market: Option<DemoMarketSnapshot>,
    session: Option<DemoSessionSnapshot>,
    latest_signal: Option<DemoSignalSnapshot>,
    latest_order_intent: Option<DemoOrderIntentSnapshot>,
    latest_risk_decision: Option<DemoRiskDecisionSnapshot>,
    simulation: Option<DemoSimulationSnapshot>,
    technical_health: DemoTechnicalHealth,
    provenance: DemoSnapshotProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DemoSnapshotBoundaries {
    read_only: bool,
    sandbox_only: bool,
    live_run_creation_allowed: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

impl DemoSnapshotBoundaries {
    const fn enforced() -> Self {
        Self {
            read_only: true,
            sandbox_only: true,
            live_run_creation_allowed: false,
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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(in crate::dashboard) struct DemoRunSnapshotResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: DemoRunSnapshotData,
    boundaries: DemoSnapshotBoundaries,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategySessionStatus {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    state: StrategySessionState,
    reason: String,
    updated_at_unix_ms: u64,
    artifacts: StoredStrategyArtifactPaths,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategyArtifactPaths {
    session_status: String,
    events: String,
    market_status: String,
    market_events: String,
    signal: String,
    order_intent: String,
    risk_decision: String,
    summary: String,
    simulation_summary: String,
    simulated_fills: String,
    simulated_positions: String,
    equity_curve: String,
    manifest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategyManifest {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    state: StrategySessionState,
    created_at_unix_ms: u64,
    updated_at_unix_ms: u64,
    artifacts: Vec<StoredStrategyManifestArtifact>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategyManifestArtifact {
    name: String,
    path: String,
    format: String,
    present: bool,
    record_count: Option<u64>,
    byte_len: Option<u64>,
    checksum: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategySessionEvent {
    schema_version: String,
    event_type: String,
    session_id: String,
    strategy_id: String,
    previous_state: Option<StrategySessionState>,
    state: StrategySessionState,
    reason: String,
    occurred_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategyMarketStatus {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    connection: String,
    state: String,
    source: String,
    event_count: u64,
    last_event_at_unix_ms: Option<u64>,
    updated_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategyMarketEvent {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    event_type: StrategyMarketEventKind,
    source: String,
    seq: u64,
    symbol: String,
    price: f64,
    event_at_unix_ms: u64,
    recorded_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategySignal {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    symbol: String,
    signal: String,
    confidence: f64,
    market_event_seq: u64,
    generated_at: String,
    generated_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategyOrderIntent {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    intent_id: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: f64,
    source_signal: String,
    confidence: f64,
    market_event_seq: u64,
    signal_generated_at: String,
    created_at: String,
    created_at_unix_ms: u64,
    submission_allowed: bool,
    submission_status: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategyRiskDecision {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    decision_id: String,
    intent_id: String,
    symbol: String,
    decision: String,
    reasons: Vec<String>,
    mode: String,
    order_submission: String,
    kill_switch_enabled: bool,
    kill_switch_active: bool,
    account_state: String,
    market_state: String,
    actual_submission: bool,
    evaluated_at: String,
    evaluated_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStrategySummary {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    state: StrategySessionState,
    event_count: u64,
    market_event_count: u64,
    signal_count: u64,
    intent_count: u64,
    risk_decision_count: u64,
    rejection_count: u64,
    actual_submission_count: u64,
    updated_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DemoTerminalIdentity {
    run_id: String,
    supervisor_node_id: String,
    strategy_instance_id: String,
    created_at_unix_ms: u64,
    started_at_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct RunListResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: Vec<ProductRun>,
    page: ProductPage,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct RunDetailResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: ProductRun,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BacktestResultArtifact {
    schema_version: String,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    strategy_version_content_hash: String,
    data_ref: String,
    data_sha256: String,
    config_ref: String,
    config_sha256: String,
    result_ref: String,
    instrument_id: String,
    strategy: String,
    parameters: BacktestParameters,
    backtest_start: String,
    backtest_end: String,
    metrics: BacktestMetrics,
    boundaries: BacktestResultBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestParameters {
    trade_size: String,
    fast_period: usize,
    slow_period: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestMetrics {
    quotes: usize,
    iterations: usize,
    total_events: usize,
    total_orders: usize,
    total_positions: usize,
    pnl_stats: BTreeMap<String, BTreeMap<String, String>>,
    return_stats: BTreeMap<String, String>,
    general_stats: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestResultBoundaries {
    read_only: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BacktestResultExpectation {
    config_sha256: String,
    data_sha256: String,
    result_sha256: String,
    trade_size: String,
    quotes: usize,
    fast_period: usize,
    slow_period: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestDetailsArtifact {
    schema_version: String,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    strategy_version_content_hash: String,
    data_ref: String,
    data_sha256: String,
    config_ref: String,
    config_sha256: String,
    details_ref: String,
    instrument_id: String,
    equity_basis: String,
    trades: Vec<BacktestTrade>,
    positions: Vec<BacktestPosition>,
    equity_curve: Vec<BacktestEquityPoint>,
    boundaries: BacktestResultBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestTrade {
    trade_id: String,
    client_order_id: String,
    venue_order_id: String,
    position_id: Option<String>,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    currency: String,
    liquidity_side: String,
    commission: Option<String>,
    ts_event: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestPosition {
    position_id: String,
    account_id: String,
    side: String,
    entry_side: String,
    peak_quantity: String,
    buy_quantity: String,
    sell_quantity: String,
    avg_price_open: String,
    avg_price_close: Option<String>,
    realized_return: String,
    realized_pnl: Option<String>,
    trade_count: usize,
    ts_opened: String,
    ts_closed: Option<String>,
    duration_ns: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestEquityPoint {
    account_id: String,
    currency: String,
    total: String,
    free: String,
    locked: String,
    ts_event: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestAnalysisArtifact {
    schema_version: String,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    strategy_version_content_hash: String,
    analysis_ref: String,
    instrument_id: String,
    risk: BacktestRiskSummary,
    drawdown_curve: Vec<BacktestDrawdownPoint>,
    timeline: Vec<BacktestTimelineEvent>,
    provenance: BacktestAnalysisProvenance,
    boundaries: BacktestResultBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestRiskSummary {
    currency: String,
    starting_equity: String,
    ending_equity: String,
    peak_equity: String,
    max_drawdown_amount: String,
    max_drawdown_rate: String,
    max_drawdown_started_at: String,
    max_drawdown_trough_at: String,
    current_drawdown_amount: String,
    current_drawdown_rate: String,
    open_positions: usize,
    closed_positions: usize,
    profitable_positions: usize,
    losing_positions: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestDrawdownPoint {
    ts_event: String,
    equity: String,
    peak_equity: String,
    drawdown_amount: String,
    drawdown_rate: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestTimelineEvent {
    event_id: String,
    event_type: String,
    ts_event: String,
    entity_ref: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestAnalysisProvenance {
    generator: String,
    engine_mode: String,
    data_ref: String,
    data_sha256: String,
    config_ref: String,
    config_sha256: String,
    summary_ref: String,
    summary_sha256: String,
    details_ref: String,
    details_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct RunMetricsResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: BacktestResultArtifact,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct RunReportResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: BacktestDetailsArtifact,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct RunAnalysisResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: BacktestAnalysisArtifact,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct RunComparisonItem {
    run_id: String,
    environment: RunEnvironment,
    strategy_id: String,
    strategy_version_id: String,
    data_ref: String,
    data_sha256: String,
    config_sha256: String,
    instrument_id: String,
    parameters: BacktestParameters,
    metrics: RunComparisonMetrics,
    risk: RunComparisonRisk,
    provenance: RunComparisonProvenance,
    reproduction_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct RunComparisonMetrics {
    market_event_count: usize,
    fill_count: usize,
    position_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct RunComparisonRisk {
    currency: String,
    starting_equity: String,
    ending_equity: String,
    max_drawdown_rate: String,
    open_positions: usize,
    closed_positions: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct RunComparisonProvenance {
    engine: String,
    data_ref: String,
    data_sha256: String,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct RunComparisonCompatibility {
    same_strategy: bool,
    same_strategy_version: bool,
    same_data: bool,
    same_instrument: bool,
    same_currency: bool,
    same_environment: bool,
    behaviorally_comparable: bool,
    directly_comparable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct RunComparison {
    baseline_run_id: String,
    run_ids: Vec<String>,
    items: Vec<RunComparisonItem>,
    compatibility: RunComparisonCompatibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct RunComparisonResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: RunComparison,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestReproductionProof {
    schema_version: String,
    source_run_id: String,
    reproduced_run_id: String,
    proof_ref: String,
    source_input_sha256: String,
    reproduced_input_sha256: String,
    source_output_sha256: String,
    reproduced_output_sha256: String,
    input_equivalent: bool,
    output_equivalent: bool,
    user_initiated: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct BacktestReproduction {
    source_run_id: String,
    reproduced_run: ProductRun,
    proof: BacktestReproductionProof,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct RunReproductionResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: BacktestReproduction,
    boundaries: BacktestRunCreationBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct RunReproductionProofResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: BacktestReproductionProof,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug)]
struct ReproductionExpectation {
    source_run_id: String,
    source_input_sha256: String,
    source_output_sha256: String,
    strategy_version: strategy_version::ProductStrategyVersion,
}

struct VerifiedBacktestBundle {
    run: ProductRun,
    config: ProductRunConfig,
    strategy_version: strategy_version::ProductStrategyVersion,
    summary: BacktestResultArtifact,
    details: BacktestDetailsArtifact,
    analysis: BacktestAnalysisArtifact,
}

#[derive(Serialize)]
struct BacktestReproductionInput<'a> {
    strategy_id: &'a str,
    strategy_version_id: &'a str,
    strategy_version_content_hash: &'a str,
    data_ref: &'a str,
    data_sha256: &'a str,
    venue_ref: &'a str,
    starting_balance: &'a str,
    quotes: usize,
    trade_size: &'a str,
    fast_period: usize,
    slow_period: usize,
    instrument_id: &'a str,
}

#[derive(Serialize)]
struct BacktestReproductionOutput<'a> {
    strategy_version_content_hash: &'a str,
    data_ref: &'a str,
    data_sha256: &'a str,
    instrument_id: &'a str,
    strategy: &'a str,
    parameters: &'a BacktestParameters,
    backtest_start: &'a str,
    backtest_end: &'a str,
    metrics: &'a BacktestMetrics,
    equity_basis: &'a str,
    trades: &'a [BacktestTrade],
    positions: &'a [BacktestPosition],
    equity_curve: &'a [BacktestEquityPoint],
    risk: &'a BacktestRiskSummary,
    drawdown_curve: &'a [BacktestDrawdownPoint],
    timeline: Vec<BacktestTimelineEvent>,
    generator: &'a str,
    engine_mode: &'a str,
    boundaries: &'a BacktestResultBoundaries,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBacktestConfig {
    run: StoredBacktestRunConfig,
    data: StoredBacktestDataConfig,
    strategy: StoredBacktestStrategyConfig,
    venue: StoredBacktestVenueConfig,
    product: StoredBacktestProductConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBacktestRunConfig {
    id: String,
    mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBacktestDataConfig {
    source: String,
    instrument_id: String,
    quotes: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBacktestStrategyConfig {
    name: String,
    trade_size: String,
    fast_period: usize,
    slow_period: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBacktestVenueConfig {
    name: String,
    starting_balance: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBacktestProductConfig {
    strategy_id: String,
    strategy_version_id: String,
    strategy_version_content_hash: String,
    data_ref: String,
    config_ref: String,
    result_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RunListQuery {
    limit: usize,
    cursor: Option<String>,
    sort: RunSort,
    order: SortOrder,
    strategy_id: Option<String>,
    strategy_version_id: Option<String>,
    environment: Option<RunEnvironment>,
    lifecycle: Option<RunLifecycle>,
}

pub(in crate::dashboard) async fn run_create_api(
    State(state): State<DashboardServerState>,
    payload: Result<Json<CreateBacktestRunRequest>, JsonRejection>,
) -> ApiStatusResult<RunCreateResponse> {
    let request_id = product_request_id();
    let Json(request) = payload.map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "request_body"),
            &request_id,
        )
    })?;
    let permit = state
        .backtest_creation_gate
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            product_error_response(
                &product_error(ProductErrorKind::Conflict, "backtest_creation_in_progress"),
                &request_id,
            )
        })?;
    let worker_state = state.clone();
    let worker_request_id = request_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        create_backtest_run(&worker_state, request, &worker_request_id, None)
    })
    .await
    .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "backtest_worker"))
    .and_then(|result| result);

    result
        .map(|(data, _)| {
            (
                StatusCode::CREATED,
                Json(RunCreateResponse {
                    schema_version: RUN_CREATE_SCHEMA_VERSION.to_string(),
                    contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
                    request_id: request_id.clone(),
                    data,
                    boundaries: BacktestRunCreationBoundaries::enforced(),
                }),
            )
        })
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn demo_run_create_api(
    State(state): State<DashboardServerState>,
    payload: Result<Json<CreateDemoRunRequest>, JsonRejection>,
) -> ApiStatusResult<DemoRunCreateResponse> {
    let request_id = product_request_id();
    let Json(request) = payload.map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "request_body"),
            &request_id,
        )
    })?;
    let permit = state
        .backtest_creation_gate
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            product_error_response(
                &product_error(ProductErrorKind::DemoConflict, "run_creation_in_progress"),
                &request_id,
            )
        })?;
    let worker_state = state.clone();
    let worker_request_id = request_id.clone();
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        create_demo_run(&worker_state, request, &worker_request_id)
    })
    .await
    .map_err(|_| product_error(ProductErrorKind::DemoExecutionFailed, "demo_worker"))
    .and_then(|result| result)
    .map(|data| {
        (
            StatusCode::CREATED,
            Json(DemoRunCreateResponse {
                schema_version: DEMO_RUN_CREATE_SCHEMA_VERSION.to_string(),
                contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
                request_id: request_id.clone(),
                data,
                boundaries: DemoRunBoundaries::enforced(),
            }),
        )
    })
    .map_err(|error| demo_product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn demo_run_action_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<DemoRunActionRequest>, JsonRejection>,
) -> ApiStatusResult<DemoRunActionResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(run_id)| run_id).map_err(|_| {
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
    tokio::task::spawn_blocking(move || run_demo_action(&worker_state, &run_id, &request))
        .await
        .map_err(|_| product_error(ProductErrorKind::DemoExecutionFailed, "demo_action_worker"))
        .and_then(|result| result)
        .map(|data| {
            (
                StatusCode::OK,
                Json(DemoRunActionResponse {
                    schema_version: DEMO_RUN_ACTION_SCHEMA_VERSION.to_string(),
                    contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
                    request_id: request_id.clone(),
                    data,
                    boundaries: DemoRunBoundaries::enforced(),
                }),
            )
        })
        .map_err(|error| demo_product_error_response(&error, &request_id))
}

fn create_demo_run(
    state: &DashboardServerState,
    request: CreateDemoRunRequest,
    request_id: &str,
) -> Result<ProductRun, ProductError> {
    let _guard = state
        .lifecycle_action_lock
        .lock()
        .map_err(|_| product_error(ProductErrorKind::DemoConflict, "demo_action_lock"))?;
    let now = unix_time_ms();
    finalize_demo_run_ownerships(state, now)?;
    let source = load_product_source(state, now)?;
    let version = strategy_version::load_product_strategy_version(&source, now)?;
    validate_demo_creation_request(&request, &source, &version)?;
    let existing_runs = load_product_runs_unlocked(state, now)?;
    if existing_runs.iter().any(|run| {
        run.environment == RunEnvironment::Sandbox && !is_terminal_demo_lifecycle(run.lifecycle)
    }) {
        return Err(product_error(
            ProductErrorKind::DemoConflict,
            "active_demo_run",
        ));
    }
    let store = SupervisorRegistryStore::new(&state.registry_path);
    let registry = store
        .load()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "registry"))?;
    let record = registry
        .nodes
        .get(&request.supervisor_node_id)
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "supervisor_node_id"))?;
    if !matches!(
        record.process.state,
        SupervisorProcessState::NotStarted | SupervisorProcessState::Stopped
    ) || record.last_known_status.lifecycle_state != LifecycleStatus::Stopped
    {
        return Err(product_error(
            ProductErrorKind::Conflict,
            "supervisor_node_state",
        ));
    }

    let run_id = request_id.replacen("product-", "demo-", 1);
    validate_identifier("run_id", &run_id)?;
    let request_raw = serde_json::to_vec_pretty(&request)
        .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "demo_request"))?;
    let request_sha256 = sha256_ref(&request_raw);
    let version_raw = strategy_version::serialize_strategy_version_snapshot(&version)?;
    let version_sha256 = sha256_ref(&version_raw);
    let config = ProductRunConfig {
        run_id: run_id.clone(),
        strategy_id: request.strategy_id,
        strategy_version_id: request.strategy_version_id,
        environment: RunEnvironment::Sandbox,
        data_ref: format!(
            "market://sandbox/{}",
            version.data_symbols().first().ok_or_else(|| {
                product_error(ProductErrorKind::SourceInvalid, "strategy_data_symbol")
            })?
        ),
        config_ref: format!("artifact://demo-runs/{run_id}/request.json"),
        adapter_ref: "adapter://sandbox/fixture-stream".to_string(),
        account_ref: request.account_ref,
        venue_ref: request.venue_ref,
        lifecycle: RunLifecycle::Created,
        result_status: RunResultStatus::Pending,
        result_ref: None,
        backtest_config_sha256: None,
        backtest_data_sha256: None,
        backtest_result_sha256: None,
        backtest_details_sha256: None,
        backtest_analysis_sha256: None,
        strategy_version_snapshot_sha256: Some(version_sha256.clone()),
        reproduction_source_run_id: None,
        reproduction_input_sha256: None,
        reproduction_output_sha256: None,
        reproduction_proof_sha256: None,
        backtest_trade_size: None,
        backtest_quotes: None,
        backtest_fast_period: None,
        backtest_slow_period: None,
        risk_status: RunRiskStatus::Pending,
        risk_ref: format!("artifact://demo-runs/{run_id}/run-manifest.json#risk"),
        error_code: None,
        error_summary: None,
        created_at_unix_ms: now,
        started_at_unix_ms: None,
        completed_at_unix_ms: None,
        updated_at_unix_ms: now,
        external_venue_connection: false,
        order_submission_allowed: false,
        order_mutation_allowed: false,
        automatic_retry_allowed: false,
        automatic_remediation_allowed: false,
        real_orders_submitted: false,
        trading_controls_enabled: false,
        demo_supervisor_node_id: Some(request.supervisor_node_id),
        demo_strategy_instance_id: Some(source.identity.identities.strategy_instance_id.clone()),
        demo_identity_contract_id: Some(source.identity.contract_id.clone()),
        demo_supervisor_record_baseline_unix_ms: Some(
            snapshot_timestamp(&record.updated_at).ok_or_else(|| {
                product_error(
                    ProductErrorKind::SourceInvalid,
                    "supervisor_record_baseline",
                )
            })?,
        ),
        demo_process_state: Some(record.process.state),
        demo_lifecycle_state: Some(record.last_known_status.lifecycle_state),
    };
    let directory = create_demo_run_directory(state, &run_id)?;
    write_new_run_file(&directory, "request.json", &request_raw)?;
    write_new_run_file(&directory, "strategy-version.json", &version_raw)?;
    let manifest = DynamicDemoRunManifest {
        schema_version: DEMO_RUN_MANIFEST_SCHEMA_VERSION.to_string(),
        request_sha256,
        strategy_version_snapshot_sha256: version_sha256,
        config: config.clone(),
    };
    let manifest_raw = serde_json::to_vec_pretty(&manifest)
        .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "demo_manifest"))?;
    let projected = validate_and_project_run(
        config,
        &source,
        &version,
        version.strategy_version_id(),
        now,
        Some(format!("artifact://demo-runs/{run_id}/run-manifest.json")),
    )?;
    write_new_run_file(&directory, "run-manifest.json", &manifest_raw)?;
    let manifest_sha256 = sha256_ref(&manifest_raw);
    if store
        .claim_run_ownership(
            &projected
                .runtime
                .as_ref()
                .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "run_runtime"))?
                .supervisor_node_id,
            SupervisorRunOwnership {
                run_id: run_id.clone(),
                manifest_sha256,
                claimed_at_unix_ms: now,
                terminal: None,
            },
        )
        .is_err()
    {
        drop(directory);
        fs::remove_dir_all(canonical_demo_artifact_root(state, false)?.join(&run_id)).map_err(
            |_| product_error(ProductErrorKind::SourceUnavailable, "demo_claim_cleanup"),
        )?;
        return Err(product_error(
            ProductErrorKind::DemoConflict,
            "demo_run_ownership",
        ));
    }
    Ok(projected)
}

const fn is_terminal_demo_lifecycle(lifecycle: RunLifecycle) -> bool {
    matches!(lifecycle, RunLifecycle::Stopped | RunLifecycle::Failed)
}

fn validate_demo_creation_request(
    request: &CreateDemoRunRequest,
    source: &ValidatedProductSource,
    version: &strategy_version::ProductStrategyVersion,
) -> Result<(), ProductError> {
    if request.environment != RunEnvironment::Sandbox || !request.user_confirmed {
        return Err(product_error(
            ProductErrorKind::BadRequest,
            "demo_confirmation",
        ));
    }
    if request.strategy_id != source.strategy.strategy_id
        || request.strategy_id != version.strategy_id()
        || request.strategy_version_id != version.strategy_version_id()
        || request.supervisor_node_id != source.identity.identities.node_id
        || request.account_ref
            != format!(
                "account://sandbox/{}",
                source.identity.identities.account_id
            )
        || request.venue_ref != format!("venue://sandbox/{}", source.identity.identities.venue_id)
    {
        return Err(product_error(ProductErrorKind::BadRequest, "demo_identity"));
    }
    Ok(())
}

fn run_demo_action(
    state: &DashboardServerState,
    path_run_id: &str,
    request: &DemoRunActionRequest,
) -> Result<DemoRunActionResult, ProductError> {
    validate_requested_run_id("run_id", path_run_id)?;
    if request.run_id != path_run_id || !request.user_confirmed {
        return Err(product_error(
            ProductErrorKind::BadRequest,
            "demo_action_identity",
        ));
    }
    let _guard = state
        .lifecycle_action_lock
        .lock()
        .map_err(|_| product_error(ProductErrorKind::Conflict, "demo_action_lock"))?;
    let previous = load_product_runs_unlocked(state, unix_time_ms())?
        .into_iter()
        .find(|run| run.run_id == path_run_id && run.environment == RunEnvironment::Sandbox)
        .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
    let runtime = previous
        .runtime
        .as_ref()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "run_runtime"))?;
    let store = SupervisorRegistryStore::new(&state.registry_path);
    let manifest_sha256 = demo_run_manifest_sha256(state, path_run_id)?;
    let current = match request.action {
        DemoRunAction::Start if previous.lifecycle == RunLifecycle::Created => {
            let started = store
                .start_node_process_for_run(
                    &StartNodeRequest {
                        node_id: runtime.supervisor_node_id.clone(),
                        ntpro_node_bin: state.ntpro_node_bin.clone(),
                        startup_timeout: Duration::from_millis(
                            super::super::DASHBOARD_ACTION_TIMEOUT_MS,
                        ),
                        node_max_runtime: Duration::from_millis(3_600_000),
                        node_heartbeat_interval: Duration::from_millis(1_000),
                        node_parent_pid: Some(std::process::id()),
                        node_shutdown_timeout: Duration::from_millis(5_000),
                    },
                    path_run_id,
                    &manifest_sha256,
                )
                .map_err(|_| product_error(ProductErrorKind::DemoExecutionFailed, "demo_start"))?;
            let initial_validation = wait_for_demo_metrics_artifact(
                &started.metrics_path,
                Duration::from_millis(super::super::DASHBOARD_ACTION_TIMEOUT_MS),
            )
            .map_err(|()| {
                product_error(ProductErrorKind::DemoExecutionFailed, "demo_start_metrics")
            })
            .and_then(|()| {
                store
                    .node_metrics(&runtime.supervisor_node_id)
                    .map(|_| ())
                    .map_err(|_| {
                        product_error(ProductErrorKind::DemoExecutionFailed, "demo_start_metrics")
                    })
            });
            if let Err(error) = initial_validation {
                cleanup_failed_demo_start(state, &store, &previous, &manifest_sha256, &error)?;
                return Err(error);
            }
            if let Err(error) = refresh_product_status_contract(state, &runtime.supervisor_node_id)
            {
                cleanup_failed_demo_start(state, &store, &previous, &manifest_sha256, &error)?;
                return Err(error);
            }
            match load_demo_run_by_id(state, path_run_id) {
                Ok(current) if current.lifecycle == RunLifecycle::Running => {
                    let snapshot_validation = wait_for_demo_snapshot(
                        &store,
                        &current,
                        Duration::from_millis(super::super::DASHBOARD_ACTION_TIMEOUT_MS),
                    );
                    if let Err(error) = snapshot_validation {
                        cleanup_failed_demo_start(
                            state,
                            &store,
                            &previous,
                            &manifest_sha256,
                            &error,
                        )?;
                        return Err(error);
                    }
                    current
                }
                Ok(_) => {
                    let error = product_error(
                        ProductErrorKind::DemoExecutionFailed,
                        "demo_start_lifecycle",
                    );
                    cleanup_failed_demo_start(state, &store, &previous, &manifest_sha256, &error)?;
                    return Err(error);
                }
                Err(error) => {
                    cleanup_failed_demo_start(state, &store, &previous, &manifest_sha256, &error)?;
                    return Err(error);
                }
            }
        }
        DemoRunAction::Stop
            if matches!(
                previous.lifecycle,
                RunLifecycle::Running | RunLifecycle::Paused
            ) =>
        {
            let stopped = store
                .stop_node_process_for_run(
                    &StopNodeRequest {
                        node_id: runtime.supervisor_node_id.clone(),
                        stop_timeout: Duration::from_millis(
                            super::super::DASHBOARD_ACTION_TIMEOUT_MS,
                        ),
                    },
                    path_run_id,
                    &manifest_sha256,
                )
                .map_err(|_| product_error(ProductErrorKind::DemoExecutionFailed, "demo_stop"))?;
            publish_demo_terminal_state(
                state,
                &store,
                &demo_terminal_identity_from_run(&previous)?,
                &stopped,
                &manifest_sha256,
                RunLifecycle::Stopped,
                None,
            )?;
            refresh_product_status_contract(state, &runtime.supervisor_node_id)?;
            load_demo_run_by_id(state, path_run_id)?
        }
        _ => {
            return Err(product_error(
                ProductErrorKind::DemoConflict,
                "demo_lifecycle",
            ));
        }
    };
    Ok(DemoRunActionResult {
        run_id: path_run_id.to_string(),
        action: request.action,
        previous_lifecycle: previous.lifecycle,
        current_run: current,
    })
}

fn load_demo_snapshot_by_id(
    state: &DashboardServerState,
    run_id: &str,
    now_unix_ms: u64,
) -> Result<DemoRunSnapshotData, ProductError> {
    let run = load_product_runs(state, now_unix_ms)?
        .into_iter()
        .find(|run| run.run_id == run_id && run.environment == RunEnvironment::Sandbox)
        .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "demo_snapshot"))?;
    let observed_at_unix_ms = now_unix_ms.max(unix_time_ms());
    let runtime = run
        .runtime
        .as_ref()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_snapshot_runtime"))?;
    if run.lifecycle == RunLifecycle::Created {
        return Ok(DemoRunSnapshotData {
            schema_version: DEMO_RUN_RESULT_SCHEMA_VERSION.to_string(),
            run_id: run.run_id,
            strategy_id: run.strategy_id,
            strategy_version_id: run.strategy_version_id,
            observed_at_unix_ms,
            lifecycle: run.lifecycle,
            snapshot_status: DemoSnapshotStatus::NotStarted,
            runtime: DemoSnapshotRuntime {
                supervisor_node_id: runtime.supervisor_node_id.clone(),
                strategy_instance_id: runtime.strategy_instance_id.clone(),
                process_state: runtime.process_state,
                lifecycle_state: runtime.lifecycle_state,
                data_connection: "not_configured".to_string(),
                execution_connection: "not_configured".to_string(),
                uptime_ms: None,
                generated_at_unix_ms: None,
            },
            market: None,
            session: None,
            latest_signal: None,
            latest_order_intent: None,
            latest_risk_decision: None,
            simulation: None,
            technical_health: DemoTechnicalHealth {
                status: DemoTechnicalHealthStatus::Blocked,
                diagnostics: vec!["demo_not_started".to_string()],
            },
            provenance: DemoSnapshotProvenance {
                source_refs: vec![run.config_ref],
                manifest_sha256: Some(demo_run_manifest_sha256(state, run_id)?),
                result_ref: None,
                result_sha256: None,
            },
        });
    }
    if is_terminal_demo_lifecycle(run.lifecycle) {
        return load_frozen_demo_result(state, &run);
    }
    let store = SupervisorRegistryStore::new(&state.registry_path);
    let record = store
        .refresh_process_state(&runtime.supervisor_node_id)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "demo_snapshot_runtime"))?;
    let metrics = store
        .node_metrics(&runtime.supervisor_node_id)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "demo_snapshot_metrics"))?;
    build_demo_snapshot_from_record(
        &run,
        &record,
        &metrics,
        observed_at_unix_ms,
        DemoSnapshotStatus::Running,
    )
}

fn load_frozen_demo_result(
    state: &DashboardServerState,
    run: &ProductRun,
) -> Result<DemoRunSnapshotData, ProductError> {
    let run_root = canonical_demo_artifact_root(state, false)?.join(&run.run_id);
    let terminal_raw = read_backtest_result_bytes(&run_root.join("terminal-state.json"))?;
    let terminal: DynamicDemoRunTerminalState = serde_json::from_slice(&terminal_raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_terminal_state"))?;
    let result_raw = read_backtest_result_bytes(&run_root.join("demo-result.json"))?;
    if !is_sha256_ref(&terminal.demo_result_sha256)
        || sha256_ref(&result_raw) != terminal.demo_result_sha256
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_result_sha256",
        ));
    }
    let mut result: DemoRunSnapshotData = serde_json::from_slice(&result_raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_result"))?;
    let expected_ref = format!("artifact://demo-runs/{}/demo-result.json", run.run_id);
    if result.schema_version != DEMO_RUN_RESULT_SCHEMA_VERSION
        || result.run_id != run.run_id
        || result.strategy_id != run.strategy_id
        || result.strategy_version_id != run.strategy_version_id
        || result.lifecycle != run.lifecycle
        || result.snapshot_status != DemoSnapshotStatus::Frozen
        || result.provenance.result_ref.as_deref() != Some(expected_ref.as_str())
        || result.provenance.result_sha256.is_some()
        || result.runtime.supervisor_node_id
            != run
                .runtime
                .as_ref()
                .map(|runtime| runtime.supervisor_node_id.as_str())
                .unwrap_or_default()
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_result",
        ));
    }
    result.provenance.result_sha256 = Some(terminal.demo_result_sha256);
    validate_demo_snapshot_boundaries(&result)?;
    Ok(result)
}

fn build_demo_snapshot_from_record(
    run: &ProductRun,
    record: &SupervisorNodeRecord,
    metrics: &NodeMetrics,
    observed_at_unix_ms: u64,
    snapshot_status: DemoSnapshotStatus,
) -> Result<DemoRunSnapshotData, ProductError> {
    let runtime = run
        .runtime
        .as_ref()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_snapshot_runtime"))?;
    if record.node_id != runtime.supervisor_node_id
        || metrics.node_id != runtime.supervisor_node_id
        || metrics.external_venue_connection
        || metrics.real_orders_submitted
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "demo_snapshot_runtime",
        ));
    }
    let strategy = load_strategy_snapshot(record, &record.node_id, &run.strategy_id)?;
    let observed_at_unix_ms = observed_at_unix_ms.max(unix_time_ms());
    let generated_at_unix_ms =
        snapshot_unix_ms(&metrics.generated_at, "demo_snapshot_metrics_generated_at")?;
    let result = DemoRunSnapshotData {
        schema_version: DEMO_RUN_RESULT_SCHEMA_VERSION.to_string(),
        run_id: run.run_id.clone(),
        strategy_id: run.strategy_id.clone(),
        strategy_version_id: run.strategy_version_id.clone(),
        observed_at_unix_ms,
        lifecycle: run.lifecycle,
        snapshot_status,
        runtime: DemoSnapshotRuntime {
            supervisor_node_id: runtime.supervisor_node_id.clone(),
            strategy_instance_id: runtime.strategy_instance_id.clone(),
            process_state: record.process.state,
            lifecycle_state: record.last_known_status.lifecycle_state,
            data_connection: connection_count_label(
                metrics.connection_counts.data_connected,
                metrics.connection_counts.data_disconnected,
                metrics.connection_counts.data_not_configured,
            )?,
            execution_connection: connection_count_label(
                metrics.connection_counts.execution_connected,
                metrics.connection_counts.execution_disconnected,
                metrics.connection_counts.execution_not_configured,
            )?,
            uptime_ms: metrics.uptime_ms.value,
            generated_at_unix_ms,
        },
        market: Some(strategy.market),
        session: Some(strategy.session),
        latest_signal: strategy.latest_signal,
        latest_order_intent: strategy.latest_order_intent,
        latest_risk_decision: strategy.latest_risk_decision,
        simulation: Some(strategy.simulation),
        technical_health: DemoTechnicalHealth {
            status: DemoTechnicalHealthStatus::Healthy,
            diagnostics: Vec::new(),
        },
        provenance: DemoSnapshotProvenance {
            source_refs: vec![
                format!("artifact://demo-runs/{}/run-manifest.json", run.run_id),
                format!(
                    "artifact://demo-runs/{}/strategy-session/manifest.json",
                    run.run_id
                ),
            ],
            manifest_sha256: Some(strategy.manifest_sha256),
            result_ref: None,
            result_sha256: None,
        },
    };
    validate_demo_snapshot_boundaries(&result)?;
    Ok(result)
}

struct LoadedStrategySnapshot {
    market: DemoMarketSnapshot,
    session: DemoSessionSnapshot,
    latest_signal: Option<DemoSignalSnapshot>,
    latest_order_intent: Option<DemoOrderIntentSnapshot>,
    latest_risk_decision: Option<DemoRiskDecisionSnapshot>,
    simulation: DemoSimulationSnapshot,
    manifest_sha256: String,
}

fn load_strategy_snapshot(
    record: &SupervisorNodeRecord,
    run_id: &str,
    strategy_id: &str,
) -> Result<LoadedStrategySnapshot, ProductError> {
    let artifact_root = canonical_path(&record.artifact_root, "demo_strategy_root")?;
    let strategy_candidate = artifact_root.join("strategy");
    let strategy_root = canonical_path(&strategy_candidate, "demo_strategy_root")?;
    if strategy_root != strategy_candidate {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_strategy_root_containment",
        ));
    }
    let manifest_raw = read_backtest_result_bytes(&strategy_root.join("manifest.json"))?;
    let manifest: StoredStrategyManifest = strict_json(&manifest_raw, "demo_strategy_manifest")?;
    if manifest.schema_version != "ntpro.v091_strategy_session_manifest.v1"
        || manifest.session_id != run_id
        || manifest.strategy_id != strategy_id
        || manifest.created_at_unix_ms == 0
        || manifest.updated_at_unix_ms < manifest.created_at_unix_ms
        || manifest.artifacts.len() != 12
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_strategy_manifest",
        ));
    }
    let expected = [
        ("session_status", "session_status.json", "json"),
        ("events", "events.jsonl", "jsonl"),
        ("market_status", "market_status.json", "json"),
        ("market_events", "market_events.jsonl", "jsonl"),
        ("signal", "signal.jsonl", "jsonl"),
        ("order_intent", "order_intent.jsonl", "jsonl"),
        ("risk_decision", "risk_decision.jsonl", "jsonl"),
        ("summary", "summary.json", "json"),
        ("simulation_summary", "simulation_summary.json", "json"),
        ("simulated_fills", "simulated_fills.jsonl", "jsonl"),
        ("simulated_positions", "simulated_positions.jsonl", "jsonl"),
        ("equity_curve", "equity_curve.jsonl", "jsonl"),
    ];
    let mut artifacts = BTreeMap::new();
    for (name, file_name, format) in expected {
        let artifact = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.name == name)
            .ok_or_else(|| {
                product_error(ProductErrorKind::SourceInvalid, "demo_strategy_manifest")
            })?;
        let expected_path = strategy_root.join(file_name);
        if !strategy_artifact_path_matches(&artifact.path, &expected_path)
            || artifact.format != format
            || !artifact.present
            || artifact.record_count.is_none()
            || artifact.byte_len.is_none()
            || artifact
                .checksum
                .as_deref()
                .is_none_or(|value| !value.starts_with("blake3:") && !value.starts_with("sha256:"))
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_strategy_manifest",
            ));
        }
        let raw = read_backtest_result_bytes(&expected_path)?;
        if artifact.byte_len != Some(u64::try_from(raw.len()).unwrap_or(u64::MAX))
            || artifact.checksum.as_deref().is_none_or(|expected| {
                strategy_checksum(&raw, expected).as_deref() != Some(expected)
            })
            || (format == "jsonl" && artifact.record_count != Some(jsonl_record_count(&raw)))
            || (format == "json" && artifact.record_count != Some(1))
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_strategy_artifact",
            ));
        }
        if artifacts.insert(name, raw).is_some() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_strategy_manifest",
            ));
        }
    }
    let required_artifact = |name: &str| {
        artifacts
            .get(name)
            .map(Vec::as_slice)
            .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_strategy_artifact"))
    };
    let status: StoredStrategySessionStatus =
        strict_json(required_artifact("session_status")?, "demo_session_status")?;
    let market: StoredStrategyMarketStatus =
        strict_json(required_artifact("market_status")?, "demo_market_status")?;
    let summary: StoredStrategySummary =
        strict_json(required_artifact("summary")?, "demo_session_summary")?;
    validate_strategy_identity(&status.session_id, &status.strategy_id, run_id, strategy_id)?;
    validate_strategy_identity(&market.session_id, &market.strategy_id, run_id, strategy_id)?;
    validate_strategy_identity(
        &summary.session_id,
        &summary.strategy_id,
        run_id,
        strategy_id,
    )?;
    if status.schema_version != "ntpro.v09_strategy_session_status.v1"
        || market.schema_version != "ntpro.v09_market_stream_status.v1"
        || summary.schema_version != "ntpro.v09_strategy_session_summary.v1"
        || status.state != manifest.state
        || summary.state != manifest.state
        || status.reason.trim().is_empty()
        || status.updated_at_unix_ms != manifest.updated_at_unix_ms
        || market.updated_at_unix_ms < status.updated_at_unix_ms
        || summary.updated_at_unix_ms < market.updated_at_unix_ms
        || summary.event_count != artifact_count(&manifest, "events")?
        || summary.market_event_count != artifact_count(&manifest, "market_events")?
        || summary.signal_count != artifact_count(&manifest, "signal")?
        || summary.intent_count != artifact_count(&manifest, "order_intent")?
        || summary.risk_decision_count != artifact_count(&manifest, "risk_decision")?
        || market.event_count != summary.market_event_count
        || summary.actual_submission_count != 0
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_strategy_projection",
        ));
    }
    validate_strategy_artifact_paths(&status.artifacts, &strategy_root)?;
    let events = parse_jsonl::<StoredStrategySessionEvent>(
        required_artifact("events")?,
        "demo_session_event",
    )?;
    let market_events = parse_jsonl::<StoredStrategyMarketEvent>(
        required_artifact("market_events")?,
        "demo_market_event",
    )?;
    let signals = parse_jsonl::<StoredStrategySignal>(required_artifact("signal")?, "demo_signal")?;
    let intents = parse_jsonl::<StoredStrategyOrderIntent>(
        required_artifact("order_intent")?,
        "demo_order_intent",
    )?;
    let risk_decisions = parse_jsonl::<StoredStrategyRiskDecision>(
        required_artifact("risk_decision")?,
        "demo_risk_decision",
    )?;
    let simulation_summary = strict_json::<DemoSimulationSummarySnapshot>(
        required_artifact("simulation_summary")?,
        "demo_simulation_summary",
    )?;
    let simulated_fills = parse_jsonl::<DemoSimulatedFillSnapshot>(
        required_artifact("simulated_fills")?,
        "demo_simulated_fill",
    )?;
    let simulated_positions = parse_jsonl::<DemoSimulatedPositionSnapshot>(
        required_artifact("simulated_positions")?,
        "demo_simulated_position",
    )?;
    let equity_curve = parse_jsonl::<DemoEquityPointSnapshot>(
        required_artifact("equity_curve")?,
        "demo_equity_point",
    )?;
    validate_strategy_records(
        run_id,
        strategy_id,
        manifest.state,
        &market,
        &summary,
        &events,
        &market_events,
        &signals,
        &intents,
        &risk_decisions,
    )?;
    validate_demo_simulation_records(
        run_id,
        strategy_id,
        &simulation_summary,
        &simulated_fills,
        &simulated_positions,
        &equity_curve,
    )?;
    let simulation = DemoSimulationSnapshot {
        summary: simulation_summary,
        fills: simulated_fills,
        positions: simulated_positions,
        equity_curve,
    };
    let latest_market = market_events.into_iter().last();
    let latest_signal = signals.into_iter().last();
    let latest_intent = intents.into_iter().last();
    let latest_risk = risk_decisions.into_iter().last();
    Ok(LoadedStrategySnapshot {
        market: DemoMarketSnapshot {
            connection: market.connection,
            state: market.state,
            source: market.source,
            event_count: market.event_count,
            last_event_at_unix_ms: market.last_event_at_unix_ms,
            updated_at_unix_ms: market.updated_at_unix_ms,
            latest_event: latest_market.map(|value| DemoMarketEvent {
                event_type: value.event_type,
                source: value.source,
                seq: value.seq,
                symbol: value.symbol,
                price: value.price,
                event_at_unix_ms: value.event_at_unix_ms,
                recorded_at_unix_ms: value.recorded_at_unix_ms,
            }),
        },
        session: DemoSessionSnapshot {
            state: status.state,
            reason: status.reason,
            event_count: summary.event_count,
            market_event_count: summary.market_event_count,
            signal_count: summary.signal_count,
            intent_count: summary.intent_count,
            risk_decision_count: summary.risk_decision_count,
            rejection_count: summary.rejection_count,
            actual_submission_count: summary.actual_submission_count,
            updated_at_unix_ms: summary.updated_at_unix_ms,
        },
        latest_signal: latest_signal.map(|value| DemoSignalSnapshot {
            symbol: value.symbol,
            signal: value.signal,
            confidence: value.confidence,
            market_event_seq: value.market_event_seq,
            generated_at_unix_ms: value.generated_at_unix_ms,
        }),
        latest_order_intent: latest_intent.map(|value| DemoOrderIntentSnapshot {
            intent_id: value.intent_id,
            symbol: value.symbol,
            side: value.side,
            order_type: value.order_type,
            quantity: value.quantity,
            source_signal: value.source_signal,
            confidence: value.confidence,
            market_event_seq: value.market_event_seq,
            created_at_unix_ms: value.created_at_unix_ms,
            submission_allowed: value.submission_allowed,
            submission_status: value.submission_status,
        }),
        latest_risk_decision: latest_risk.map(|value| DemoRiskDecisionSnapshot {
            decision_id: value.decision_id,
            intent_id: value.intent_id,
            symbol: value.symbol,
            decision: value.decision,
            reasons: value.reasons,
            mode: value.mode,
            order_submission: value.order_submission,
            kill_switch_enabled: value.kill_switch_enabled,
            kill_switch_active: value.kill_switch_active,
            account_state: value.account_state,
            market_state: value.market_state,
            actual_submission: value.actual_submission,
            evaluated_at_unix_ms: value.evaluated_at_unix_ms,
        }),
        simulation,
        manifest_sha256: sha256_ref(&manifest_raw),
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_strategy_records(
    run_id: &str,
    strategy_id: &str,
    manifest_state: StrategySessionState,
    market: &StoredStrategyMarketStatus,
    summary: &StoredStrategySummary,
    events: &[StoredStrategySessionEvent],
    market_events: &[StoredStrategyMarketEvent],
    signals: &[StoredStrategySignal],
    intents: &[StoredStrategyOrderIntent],
    risk_decisions: &[StoredStrategyRiskDecision],
) -> Result<(), ProductError> {
    for (session_id, observed_strategy_id) in events
        .iter()
        .map(|value| (&value.session_id, &value.strategy_id))
        .chain(
            market_events
                .iter()
                .map(|value| (&value.session_id, &value.strategy_id)),
        )
        .chain(
            signals
                .iter()
                .map(|value| (&value.session_id, &value.strategy_id)),
        )
        .chain(
            intents
                .iter()
                .map(|value| (&value.session_id, &value.strategy_id)),
        )
        .chain(
            risk_decisions
                .iter()
                .map(|value| (&value.session_id, &value.strategy_id)),
        )
    {
        validate_strategy_identity(session_id, observed_strategy_id, run_id, strategy_id)?;
    }
    if market.connection.trim().is_empty()
        || market.state.trim().is_empty()
        || market.source.trim().is_empty()
        || market.last_event_at_unix_ms == Some(0)
        || (market.event_count == 0) != market_events.is_empty()
        || market.last_event_at_unix_ms != market_events.last().map(|value| value.event_at_unix_ms)
        || events.iter().any(|value| {
            value.schema_version != "ntpro.v09_strategy_session_event.v1"
                || value.event_type.trim().is_empty()
                || value.reason.trim().is_empty()
                || value.previous_state == Some(value.state)
                || value.occurred_at_unix_ms == 0
        })
        || market_events.iter().any(|value| {
            value.schema_version != "ntpro.v09_market_stream_event.v1"
                || value.seq == 0
                || value.source.trim().is_empty()
                || value.symbol.trim().is_empty()
                || !value.price.is_finite()
                || value.event_at_unix_ms == 0
                || value.recorded_at_unix_ms == 0
                || value.recorded_at_unix_ms < value.event_at_unix_ms
        })
        || signals.iter().any(|value| {
            value.schema_version != "ntpro.v09_strategy_signal.v1"
                || value.symbol.trim().is_empty()
                || value.signal.trim().is_empty()
                || !value.confidence.is_finite()
                || !(0.0..=1.0).contains(&value.confidence)
                || value.market_event_seq == 0
                || value.generated_at.trim().is_empty()
                || value.generated_at_unix_ms == 0
        })
        || intents.iter().any(|value| {
            value.schema_version != "ntpro.v09_order_intent.v1"
                || validate_identifier("demo_order_intent_id", &value.intent_id).is_err()
                || value.symbol.trim().is_empty()
                || value.side.trim().is_empty()
                || value.order_type.trim().is_empty()
                || value.source_signal.trim().is_empty()
                || value.submission_status.trim().is_empty()
                || !value.quantity.is_finite()
                || value.quantity <= 0.0
                || !value.confidence.is_finite()
                || !(0.0..=1.0).contains(&value.confidence)
                || value.market_event_seq == 0
                || value.signal_generated_at.trim().is_empty()
                || value.created_at.trim().is_empty()
                || value.created_at_unix_ms == 0
        })
        || risk_decisions.iter().any(|value| {
            value.schema_version != "ntpro.v09_risk_decision.v1"
                || validate_identifier("demo_risk_decision_id", &value.decision_id).is_err()
                || validate_identifier("demo_risk_intent_id", &value.intent_id).is_err()
                || value.symbol.trim().is_empty()
                || value.decision.trim().is_empty()
                || value.reasons.is_empty()
                || value.reasons.iter().any(|reason| reason.trim().is_empty())
                || value.mode.trim().is_empty()
                || value.order_submission.trim().is_empty()
                || value.account_state.trim().is_empty()
                || value.market_state.trim().is_empty()
                || value.evaluated_at.trim().is_empty()
                || value.evaluated_at_unix_ms == 0
        })
        || events
            .last()
            .is_none_or(|value| value.state != manifest_state)
        || (summary.event_count == 0) != events.is_empty()
        || (summary.market_event_count == 0) != market_events.is_empty()
        || (summary.signal_count == 0) != signals.is_empty()
        || (summary.intent_count == 0) != intents.is_empty()
        || (summary.risk_decision_count == 0) != risk_decisions.is_empty()
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_strategy_projection",
        ));
    }
    if intents.iter().any(|intent| intent.submission_allowed)
        || risk_decisions.iter().any(|risk| risk.actual_submission)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "demo_strategy_submission",
        ));
    }
    Ok(())
}

fn validate_demo_simulation_records(
    run_id: &str,
    strategy_id: &str,
    summary: &DemoSimulationSummarySnapshot,
    fills: &[DemoSimulatedFillSnapshot],
    positions: &[DemoSimulatedPositionSnapshot],
    equity_curve: &[DemoEquityPointSnapshot],
) -> Result<(), ProductError> {
    validate_strategy_identity(
        &summary.session_id,
        &summary.strategy_id,
        run_id,
        strategy_id,
    )?;
    let boundaries = &summary.boundaries;
    if summary.schema_version != "ntpro.demo_simulation_summary.v1"
        || summary.instrument_id != "BTCUSDT.BINANCE"
        || summary.engine != "nautilus_backtest::engine::BacktestEngine"
        || summary.execution_mode != "simulated"
        || !is_sha256_ref(&summary.data_sha256)
        || summary.parameters.trade_size.parse::<Quantity>().is_err()
        || summary.parameters.fast_period != 3
        || summary.parameters.slow_period != 5
        || summary.fill_count != fills.len()
        || summary.position_count != positions.len()
        || summary.equity_point_count != equity_curve.len()
        || fills.is_empty()
        || positions.is_empty()
        || equity_curve.is_empty()
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_simulation_summary",
        ));
    }
    if !boundaries.simulation_only
        || boundaries.external_venue_connection
        || boundaries.order_submission_allowed
        || boundaries.order_mutation_allowed
        || boundaries.automatic_retry_allowed
        || boundaries.automatic_remediation_allowed
        || boundaries.real_orders_submitted
        || boundaries.trading_controls_enabled
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "demo_simulation_boundaries",
        ));
    }

    for fill in fills {
        validate_strategy_identity(&fill.session_id, &fill.strategy_id, run_id, strategy_id)?;
        if !fill.simulation_only {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "demo_simulated_fill_boundary",
            ));
        }
        if fill.schema_version != "ntpro.demo_simulated_fill.v1"
            || fill.trade_id.trim().is_empty()
            || fill.client_order_id.trim().is_empty()
            || fill.venue_order_id.trim().is_empty()
            || fill.side.trim().is_empty()
            || fill.order_type.trim().is_empty()
            || fill.quantity.parse::<Quantity>().is_err()
            || fill.price.parse::<Decimal>().is_err()
            || fill.currency.trim().is_empty()
            || fill.liquidity_side.trim().is_empty()
            || fill
                .commission
                .as_deref()
                .is_some_and(|value| value.parse::<Money>().is_err())
            || !positive_timestamp_text(&fill.ts_event)
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_simulated_fill",
            ));
        }
    }
    for position in positions {
        validate_strategy_identity(
            &position.session_id,
            &position.strategy_id,
            run_id,
            strategy_id,
        )?;
        if !position.simulation_only {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "demo_simulated_position_boundary",
            ));
        }
        if position.schema_version != "ntpro.demo_simulated_position.v1"
            || position.position_id.trim().is_empty()
            || position.account_id.trim().is_empty()
            || position.side.trim().is_empty()
            || position.entry_side.trim().is_empty()
            || position.peak_quantity.parse::<Quantity>().is_err()
            || position.buy_quantity.parse::<Quantity>().is_err()
            || position.sell_quantity.parse::<Quantity>().is_err()
            || position.avg_price_open.parse::<Decimal>().is_err()
            || position
                .avg_price_close
                .as_deref()
                .is_some_and(|value| value.parse::<Decimal>().is_err())
            || position.realized_return.parse::<Decimal>().is_err()
            || position
                .realized_pnl
                .as_deref()
                .is_some_and(|value| value.parse::<Money>().is_err())
            || position.trade_count == 0
            || !positive_timestamp_text(&position.ts_opened)
            || position
                .ts_closed
                .as_deref()
                .is_some_and(|value| !positive_timestamp_text(value))
            || position.duration_ns.parse::<u64>().is_err()
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_simulated_position",
            ));
        }
    }
    for point in equity_curve {
        validate_strategy_identity(&point.session_id, &point.strategy_id, run_id, strategy_id)?;
        let total = point.total.parse::<Money>();
        let free = point.free.parse::<Money>();
        let locked = point.locked.parse::<Money>();
        if !point.simulation_only {
            return Err(product_error(
                ProductErrorKind::BoundaryViolation,
                "demo_equity_point_boundary",
            ));
        }
        if point.schema_version != "ntpro.demo_equity_point.v1"
            || point.account_id.trim().is_empty()
            || point.currency.trim().is_empty()
            || total.as_ref().is_err()
            || free.as_ref().is_err()
            || locked.as_ref().is_err()
            || total
                .as_ref()
                .is_ok_and(|value| value.currency.to_string() != point.currency)
            || free
                .as_ref()
                .is_ok_and(|value| value.currency.to_string() != point.currency)
            || locked
                .as_ref()
                .is_ok_and(|value| value.currency.to_string() != point.currency)
            || !positive_timestamp_text(&point.ts_event)
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_equity_point",
            ));
        }
    }
    if fills
        .windows(2)
        .any(|pair| pair[0].ts_event > pair[1].ts_event)
        || positions
            .windows(2)
            .any(|pair| pair[0].ts_opened > pair[1].ts_opened)
        || equity_curve
            .windows(2)
            .any(|pair| pair[0].ts_event > pair[1].ts_event)
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_simulation_ordering",
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn validate_demo_simulation_value_for_test(
    run_id: &str,
    strategy_id: &str,
    value: serde_json::Value,
) -> Result<(), ProductError> {
    let snapshot: DemoSimulationSnapshot = serde_json::from_value(value)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_simulation_fixture"))?;
    validate_demo_simulation_records(
        run_id,
        strategy_id,
        &snapshot.summary,
        &snapshot.fills,
        &snapshot.positions,
        &snapshot.equity_curve,
    )
}

fn positive_timestamp_text(value: &str) -> bool {
    value.parse::<u64>().is_ok_and(|timestamp| timestamp > 0)
}

fn strict_json<T: serde::de::DeserializeOwned>(
    raw: &[u8],
    field: &'static str,
) -> Result<T, ProductError> {
    serde_json::from_slice(raw).map_err(|_| product_error(ProductErrorKind::SourceInvalid, field))
}

fn parse_jsonl<T: serde::de::DeserializeOwned>(
    raw: &[u8],
    field: &'static str,
) -> Result<Vec<T>, ProductError> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, field))?;
    let mut records = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        records.push(
            serde_json::from_str(line)
                .map_err(|_| product_error(ProductErrorKind::SourceInvalid, field))?,
        );
    }
    Ok(records)
}

fn snapshot_unix_ms(
    value: &SnapshotValue<String>,
    field: &'static str,
) -> Result<Option<u64>, ProductError> {
    match (value.availability, value.value.as_deref()) {
        (SnapshotAvailability::Available, Some(raw)) => raw
            .parse::<u64>()
            .ok()
            .filter(|parsed| *parsed > 0)
            .map(Some)
            .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, field)),
        (SnapshotAvailability::Available, None) | (_, Some(_)) => {
            Err(product_error(ProductErrorKind::SourceInvalid, field))
        }
        (_, None) => Ok(None),
    }
}

#[cfg(test)]
mod strategy_record_validation_tests {
    use super::*;

    #[test]
    fn rejects_invalid_or_forbidden_early_jsonl_records() {
        let events = parse_jsonl::<StoredStrategySessionEvent>(
            br#"
{"schema_version":"ntpro.v09_strategy_session_event.v1","event_type":"started","session_id":"demo-run","strategy_id":"ema-cross","previous_state":null,"state":"running","reason":"started","occurred_at_unix_ms":1}
"#,
            "demo_session_event",
        )
        .expect("session event should parse");
        let signals = parse_jsonl::<StoredStrategySignal>(
            br#"
{"schema_version":"ntpro.v09_strategy_signal.v1","session_id":"demo-run","strategy_id":"ema-cross","symbol":"BTCUSDT.BINANCE","signal":"buy","confidence":0.8,"market_event_seq":1,"generated_at":"invalid-early","generated_at_unix_ms":0}
{"schema_version":"ntpro.v09_strategy_signal.v1","session_id":"demo-run","strategy_id":"ema-cross","symbol":"BTCUSDT.BINANCE","signal":"sell","confidence":0.7,"market_event_seq":2,"generated_at":"valid-latest","generated_at_unix_ms":2}
"#,
            "demo_signal",
        )
        .expect("signals should parse");
        let invalid_summary = StoredStrategySummary {
            schema_version: "ntpro.v09_strategy_session_summary.v1".to_string(),
            session_id: "demo-run".to_string(),
            strategy_id: "ema-cross".to_string(),
            state: StrategySessionState::Running,
            event_count: 1,
            market_event_count: 0,
            signal_count: 2,
            intent_count: 0,
            risk_decision_count: 0,
            rejection_count: 0,
            actual_submission_count: 0,
            updated_at_unix_ms: 2,
        };
        let market = StoredStrategyMarketStatus {
            schema_version: "ntpro.v09_market_stream_status.v1".to_string(),
            session_id: "demo-run".to_string(),
            strategy_id: "ema-cross".to_string(),
            connection: "connected".to_string(),
            state: "streaming".to_string(),
            source: "fixture".to_string(),
            event_count: 0,
            last_event_at_unix_ms: None,
            updated_at_unix_ms: 2,
        };
        let error = validate_strategy_records(
            "demo-run",
            "ema-cross",
            StrategySessionState::Running,
            &market,
            &invalid_summary,
            &events,
            &[],
            &signals,
            &[],
            &[],
        )
        .expect_err("an invalid earlier signal must not be hidden by a valid latest signal");
        assert_eq!(error.kind, ProductErrorKind::SourceInvalid);

        let intents = parse_jsonl::<StoredStrategyOrderIntent>(
            br#"
{"schema_version":"ntpro.v09_order_intent.v1","session_id":"demo-run","strategy_id":"ema-cross","intent_id":"intent-early","symbol":"BTCUSDT.BINANCE","side":"buy","order_type":"market","quantity":1.0,"source_signal":"buy","confidence":0.8,"market_event_seq":1,"signal_generated_at":"early","created_at":"early","created_at_unix_ms":1,"submission_allowed":true,"submission_status":"forbidden"}
{"schema_version":"ntpro.v09_order_intent.v1","session_id":"demo-run","strategy_id":"ema-cross","intent_id":"intent-latest","symbol":"BTCUSDT.BINANCE","side":"sell","order_type":"market","quantity":1.0,"source_signal":"sell","confidence":0.7,"market_event_seq":2,"signal_generated_at":"latest","created_at":"latest","created_at_unix_ms":2,"submission_allowed":false,"submission_status":"blocked"}
"#,
            "demo_order_intent",
        )
        .expect("order intents should parse");
        let forbidden_summary = StoredStrategySummary {
            signal_count: 0,
            intent_count: 2,
            ..invalid_summary
        };
        let error = validate_strategy_records(
            "demo-run",
            "ema-cross",
            StrategySessionState::Running,
            &market,
            &forbidden_summary,
            &events,
            &[],
            &[],
            &intents,
            &[],
        )
        .expect_err("a forbidden earlier intent must not be hidden by a safe latest intent");
        assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);

        let invalid_contract_intents = parse_jsonl::<StoredStrategyOrderIntent>(
            br#"
{"schema_version":"ntpro.v09_order_intent.v1","session_id":"demo-run","strategy_id":"ema-cross","intent_id":"bad/intent","symbol":"BTCUSDT.BINANCE","side":"buy","order_type":"market","quantity":1.0,"source_signal":"buy","confidence":0.8,"market_event_seq":1,"signal_generated_at":"early","created_at":"early","created_at_unix_ms":1,"submission_allowed":false,"submission_status":""}
{"schema_version":"ntpro.v09_order_intent.v1","session_id":"demo-run","strategy_id":"ema-cross","intent_id":"intent-latest","symbol":"BTCUSDT.BINANCE","side":"sell","order_type":"market","quantity":1.0,"source_signal":"sell","confidence":0.7,"market_event_seq":2,"signal_generated_at":"latest","created_at":"latest","created_at_unix_ms":2,"submission_allowed":false,"submission_status":"blocked"}
"#,
            "demo_order_intent",
        )
        .expect("contract-negative order intents should parse");
        let invalid_contract_summary = StoredStrategySummary {
            schema_version: "ntpro.v09_strategy_session_summary.v1".to_string(),
            session_id: "demo-run".to_string(),
            strategy_id: "ema-cross".to_string(),
            state: StrategySessionState::Running,
            event_count: 1,
            market_event_count: 0,
            signal_count: 0,
            intent_count: 2,
            risk_decision_count: 0,
            rejection_count: 0,
            actual_submission_count: 0,
            updated_at_unix_ms: 2,
        };
        let error = validate_strategy_records(
            "demo-run",
            "ema-cross",
            StrategySessionState::Running,
            &market,
            &invalid_contract_summary,
            &events,
            &[],
            &[],
            &invalid_contract_intents,
            &[],
        )
        .expect_err("an invalid early intent contract must fail closed");
        assert_eq!(error.kind, ProductErrorKind::SourceInvalid);

        let invalid_risk_decisions = parse_jsonl::<StoredStrategyRiskDecision>(
            br#"
{"schema_version":"ntpro.v09_risk_decision.v1","session_id":"demo-run","strategy_id":"ema-cross","decision_id":"bad/decision","intent_id":"bad/intent","symbol":"BTCUSDT.BINANCE","decision":"rejected","reasons":[""],"mode":"sandbox","order_submission":"disabled","kill_switch_enabled":true,"kill_switch_active":false,"account_state":"sandbox","market_state":"fresh","actual_submission":false,"evaluated_at":"early","evaluated_at_unix_ms":1}
{"schema_version":"ntpro.v09_risk_decision.v1","session_id":"demo-run","strategy_id":"ema-cross","decision_id":"decision-latest","intent_id":"intent-latest","symbol":"BTCUSDT.BINANCE","decision":"rejected","reasons":["blocked"],"mode":"sandbox","order_submission":"disabled","kill_switch_enabled":true,"kill_switch_active":false,"account_state":"sandbox","market_state":"fresh","actual_submission":false,"evaluated_at":"latest","evaluated_at_unix_ms":2}
"#,
            "demo_risk_decision",
        )
        .expect("contract-negative risk decisions should parse");
        let error = validate_strategy_records(
            "demo-run",
            "ema-cross",
            StrategySessionState::Running,
            &market,
            &StoredStrategySummary {
                intent_count: 0,
                risk_decision_count: 2,
                ..invalid_contract_summary
            },
            &events,
            &[],
            &[],
            &[],
            &invalid_risk_decisions,
        )
        .expect_err("invalid early risk IDs and reasons must fail closed");
        assert_eq!(error.kind, ProductErrorKind::SourceInvalid);

        let invalid_market = StoredStrategyMarketStatus {
            event_count: 1,
            last_event_at_unix_ms: Some(0),
            ..market
        };
        let error = validate_strategy_records(
            "demo-run",
            "ema-cross",
            StrategySessionState::Running,
            &invalid_market,
            &StoredStrategySummary {
                market_event_count: 0,
                intent_count: 0,
                ..forbidden_summary
            },
            &events,
            &[],
            &[],
            &[],
            &[],
        )
        .expect_err("zero market timestamps must fail the public schema boundary");
        assert_eq!(error.kind, ProductErrorKind::SourceInvalid);

        let error = snapshot_unix_ms(
            &SnapshotValue::available("0".to_string()),
            "demo_snapshot_metrics_generated_at",
        )
        .expect_err("zero generated time must fail the public schema boundary");
        assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    }
}

fn validate_strategy_identity(
    session_id: &str,
    observed_strategy_id: &str,
    run_id: &str,
    strategy_id: &str,
) -> Result<(), ProductError> {
    if session_id != run_id || observed_strategy_id != strategy_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_strategy_identity",
        ));
    }
    Ok(())
}

fn validate_strategy_artifact_paths(
    paths: &StoredStrategyArtifactPaths,
    root: &Path,
) -> Result<(), ProductError> {
    for (actual, name) in [
        (&paths.session_status, "session_status.json"),
        (&paths.events, "events.jsonl"),
        (&paths.market_status, "market_status.json"),
        (&paths.market_events, "market_events.jsonl"),
        (&paths.signal, "signal.jsonl"),
        (&paths.order_intent, "order_intent.jsonl"),
        (&paths.risk_decision, "risk_decision.jsonl"),
        (&paths.summary, "summary.json"),
        (&paths.simulation_summary, "simulation_summary.json"),
        (&paths.simulated_fills, "simulated_fills.jsonl"),
        (&paths.simulated_positions, "simulated_positions.jsonl"),
        (&paths.equity_curve, "equity_curve.jsonl"),
        (&paths.manifest, "manifest.json"),
    ] {
        if !strategy_artifact_path_matches(actual, &root.join(name)) {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_strategy_artifact_paths",
            ));
        }
    }
    Ok(())
}

fn strategy_artifact_path_matches(actual: &str, expected: &Path) -> bool {
    Path::new(actual).is_absolute()
        && fs::canonicalize(actual).is_ok_and(|canonical_actual| canonical_actual == expected)
}

fn artifact_count(manifest: &StoredStrategyManifest, name: &str) -> Result<u64, ProductError> {
    manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.name == name)
        .and_then(|artifact| artifact.record_count)
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_strategy_manifest"))
}

fn strategy_checksum(raw: &[u8], expected: &str) -> Option<String> {
    if expected.starts_with("blake3:") {
        Some(format!("blake3:{}", blake3::hash(raw).to_hex()))
    } else if expected.starts_with("sha256:") {
        Some(sha256_ref(raw))
    } else {
        None
    }
}

fn jsonl_record_count(raw: &[u8]) -> u64 {
    std::str::from_utf8(raw).map_or(u64::MAX, |text| {
        u64::try_from(text.lines().filter(|line| !line.trim().is_empty()).count())
            .unwrap_or(u64::MAX)
    })
}

fn connection_count_label(
    connected: u64,
    disconnected: u64,
    not_configured: u64,
) -> Result<String, ProductError> {
    match (connected, disconnected, not_configured) {
        (1, 0, 0) => Ok("connected".to_string()),
        (0, 1, 0) => Ok("disconnected".to_string()),
        (0, 0, 1) => Ok("not_configured".to_string()),
        _ => Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_connection_counts",
        )),
    }
}

fn validate_demo_snapshot_boundaries(result: &DemoRunSnapshotData) -> Result<(), ProductError> {
    if result
        .session
        .as_ref()
        .is_some_and(|session| session.actual_submission_count != 0)
        || result
            .latest_order_intent
            .as_ref()
            .is_some_and(|intent| intent.submission_allowed)
        || result
            .latest_risk_decision
            .as_ref()
            .is_some_and(|decision| decision.actual_submission)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "demo_snapshot_boundaries",
        ));
    }
    Ok(())
}

fn load_demo_run_by_id(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<ProductRun, ProductError> {
    load_product_runs_unlocked(state, unix_time_ms())?
        .into_iter()
        .find(|run| run.run_id == run_id && run.environment == RunEnvironment::Sandbox)
        .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))
}

fn demo_terminal_identity_from_run(run: &ProductRun) -> Result<DemoTerminalIdentity, ProductError> {
    let runtime = run
        .runtime
        .as_ref()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "run_runtime"))?;
    Ok(DemoTerminalIdentity {
        run_id: run.run_id.clone(),
        supervisor_node_id: runtime.supervisor_node_id.clone(),
        strategy_instance_id: runtime.strategy_instance_id.clone(),
        created_at_unix_ms: run.created_at_unix_ms,
        started_at_unix_ms: run.started_at_unix_ms,
    })
}

fn demo_terminal_identity_from_config(
    config: &ProductRunConfig,
) -> Result<DemoTerminalIdentity, ProductError> {
    Ok(DemoTerminalIdentity {
        run_id: config.run_id.clone(),
        supervisor_node_id: config.demo_supervisor_node_id.clone().ok_or_else(|| {
            product_error(ProductErrorKind::SourceInvalid, "demo_supervisor_node_id")
        })?,
        strategy_instance_id: config.demo_strategy_instance_id.clone().ok_or_else(|| {
            product_error(ProductErrorKind::SourceInvalid, "demo_strategy_instance_id")
        })?,
        created_at_unix_ms: config.created_at_unix_ms,
        started_at_unix_ms: config.started_at_unix_ms,
    })
}

fn cleanup_failed_demo_start(
    state: &DashboardServerState,
    store: &SupervisorRegistryStore,
    run: &ProductRun,
    manifest_sha256: &str,
    failure: &ProductError,
) -> Result<(), ProductError> {
    let runtime = run
        .runtime
        .as_ref()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "run_runtime"))?;
    let stopped = store
        .stop_node_process_for_run(
            &StopNodeRequest {
                node_id: runtime.supervisor_node_id.clone(),
                stop_timeout: Duration::from_millis(super::super::DASHBOARD_ACTION_TIMEOUT_MS),
            },
            &run.run_id,
            manifest_sha256,
        )
        .map_err(|_| product_error(ProductErrorKind::DemoExecutionFailed, "demo_start_cleanup"))?;
    if matches!(
        failure.kind,
        ProductErrorKind::SourceUnavailable
            | ProductErrorKind::SourceInvalid
            | ProductErrorKind::BoundaryViolation
    ) {
        remove_unanchored_demo_terminal_files(state, store, run)?;
        return refresh_product_status_contract(state, &runtime.supervisor_node_id);
    }
    let publication = publish_demo_terminal_state(
        state,
        store,
        &demo_terminal_identity_from_run(run)?,
        &stopped,
        manifest_sha256,
        RunLifecycle::Failed,
        Some((
            "demo_start_validation_failed",
            "Demo 启动后运行时校验失败，节点已停止",
        )),
    );
    match publication {
        Ok(()) => {}
        Err(error)
            if matches!(
                error.kind,
                ProductErrorKind::SourceUnavailable
                    | ProductErrorKind::SourceInvalid
                    | ProductErrorKind::BoundaryViolation
            ) =>
        {
            remove_unanchored_demo_terminal_files(state, store, run)?;
        }
        Err(_) => {
            return Err(product_error(
                ProductErrorKind::DemoExecutionFailed,
                "demo_start_cleanup_state",
            ));
        }
    }
    refresh_product_status_contract(state, &runtime.supervisor_node_id)
}

fn remove_unanchored_demo_terminal_files(
    state: &DashboardServerState,
    store: &SupervisorRegistryStore,
    run: &ProductRun,
) -> Result<(), ProductError> {
    let runtime = run
        .runtime
        .as_ref()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "run_runtime"))?;
    let registry = store
        .load()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "registry"))?;
    let ownership = registry
        .nodes
        .get(&runtime.supervisor_node_id)
        .and_then(|record| record.run_ownership.get(&run.run_id))
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_run_ownership"))?;
    if ownership.terminal.is_some() {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_terminal_anchor",
        ));
    }
    let run_root = canonical_demo_artifact_root(state, false)?.join(&run.run_id);
    let directory = open_absolute_directory_nofollow(&run_root)?;
    for name in ["terminal-state.json", "demo-result.json"] {
        match directory.remove_file(name) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err(product_error(
                    ProductErrorKind::DemoExecutionFailed,
                    "demo_start_cleanup_state",
                ));
            }
        }
    }
    Ok(())
}

fn product_run_for_demo_terminal(
    config: &ProductRunConfig,
    record: &SupervisorNodeRecord,
    lifecycle: RunLifecycle,
    completed_at_unix_ms: u64,
    error_code: Option<&str>,
    error_summary: Option<&str>,
) -> Result<ProductRun, ProductError> {
    let supervisor_node_id = config
        .demo_supervisor_node_id
        .clone()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_supervisor_node_id"))?;
    let strategy_instance_id = config.demo_strategy_instance_id.clone().ok_or_else(|| {
        product_error(ProductErrorKind::SourceInvalid, "demo_strategy_instance_id")
    })?;
    if supervisor_node_id != record.node_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_terminal_identity",
        ));
    }
    Ok(ProductRun {
        run_id: config.run_id.clone(),
        strategy_id: config.strategy_id.clone(),
        strategy_version_id: config.strategy_version_id.clone(),
        environment: RunEnvironment::Sandbox,
        data_ref: config.data_ref.clone(),
        config_ref: config.config_ref.clone(),
        adapter_ref: config.adapter_ref.clone(),
        account_ref: config.account_ref.clone(),
        venue_ref: config.venue_ref.clone(),
        lifecycle,
        result: ProductRunResult {
            status: if lifecycle == RunLifecycle::Failed {
                RunResultStatus::Unavailable
            } else {
                RunResultStatus::Pending
            },
            result_ref: None,
            report_ref: None,
            analysis_ref: None,
            reproduction_ref: None,
        },
        risk: ProductRunRisk {
            status: RunRiskStatus::Blocked,
            risk_ref: config.risk_ref.clone(),
        },
        error: match (error_code, error_summary) {
            (Some(code), Some(summary)) => Some(ProductRunError {
                code: code.to_string(),
                summary: summary.to_string(),
            }),
            (None, None) => None,
            _ => return Err(product_error(ProductErrorKind::SourceInvalid, "run_error")),
        },
        created_at_unix_ms: config.created_at_unix_ms,
        started_at_unix_ms: config.started_at_unix_ms.or_else(|| {
            snapshot_timestamp(&record.last_known_status.started_at)
                .map(|value| value.max(config.created_at_unix_ms))
        }),
        completed_at_unix_ms: Some(completed_at_unix_ms),
        updated_at_unix_ms: completed_at_unix_ms,
        source: ProductSource {
            source_type: "demo_runtime".to_string(),
            freshness_status: "frozen".to_string(),
            source_refs: vec![format!(
                "artifact://demo-runs/{}/run-manifest.json",
                config.run_id
            )],
        },
        capabilities: ProductRunCapabilities {
            external_venue_connection: false,
            order_submission_allowed: false,
            order_mutation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            real_orders_submitted: false,
            trading_controls_enabled: false,
        },
        runtime: Some(ProductRunRuntime {
            supervisor_node_id,
            strategy_instance_id,
            process_state: record.process.state,
            lifecycle_state: record.last_known_status.lifecycle_state,
        }),
    })
}

fn failed_demo_snapshot(run: &ProductRun, observed_at_unix_ms: u64) -> DemoRunSnapshotData {
    let runtime = run
        .runtime
        .as_ref()
        .expect("terminal Demo runtime is required");
    DemoRunSnapshotData {
        schema_version: DEMO_RUN_RESULT_SCHEMA_VERSION.to_string(),
        run_id: run.run_id.clone(),
        strategy_id: run.strategy_id.clone(),
        strategy_version_id: run.strategy_version_id.clone(),
        observed_at_unix_ms,
        lifecycle: RunLifecycle::Failed,
        snapshot_status: DemoSnapshotStatus::Frozen,
        runtime: DemoSnapshotRuntime {
            supervisor_node_id: runtime.supervisor_node_id.clone(),
            strategy_instance_id: runtime.strategy_instance_id.clone(),
            process_state: runtime.process_state,
            lifecycle_state: runtime.lifecycle_state,
            data_connection: "unknown".to_string(),
            execution_connection: "unknown".to_string(),
            uptime_ms: None,
            generated_at_unix_ms: None,
        },
        market: None,
        session: None,
        latest_signal: None,
        latest_order_intent: None,
        latest_risk_decision: None,
        simulation: None,
        technical_health: DemoTechnicalHealth {
            status: DemoTechnicalHealthStatus::Blocked,
            diagnostics: vec!["demo_runtime_validation_failed".to_string()],
        },
        provenance: DemoSnapshotProvenance {
            source_refs: vec![format!(
                "artifact://demo-runs/{}/run-manifest.json",
                run.run_id
            )],
            manifest_sha256: None,
            result_ref: None,
            result_sha256: None,
        },
    }
}

fn publish_demo_terminal_state(
    state: &DashboardServerState,
    store: &SupervisorRegistryStore,
    run: &DemoTerminalIdentity,
    record: &SupervisorNodeRecord,
    manifest_sha256: &str,
    lifecycle: RunLifecycle,
    failure: Option<(&str, &str)>,
) -> Result<(), ProductError> {
    let clean_stop = record.process.state == SupervisorProcessState::Stopped
        && record.last_known_status.lifecycle_state == LifecycleStatus::Stopped;
    let failed_exit =
        lifecycle == RunLifecycle::Failed && record.process.state == SupervisorProcessState::Stale;
    if !is_terminal_demo_lifecycle(lifecycle) || (!clean_stop && !failed_exit) {
        return Err(product_error(
            ProductErrorKind::DemoExecutionFailed,
            "demo_terminal_process",
        ));
    }
    if record.node_id != run.supervisor_node_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_terminal_identity",
        ));
    }
    let run_root = canonical_demo_artifact_root(state, false)?.join(&run.run_id);
    let manifest_raw = read_backtest_result_bytes(&run_root.join("run-manifest.json"))?;
    if sha256_ref(&manifest_raw) != manifest_sha256 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_terminal_manifest",
        ));
    }
    let directory = open_absolute_directory_nofollow(&run_root)?;
    let completed_at_unix_ms = snapshot_timestamp(&record.last_known_status.stopped_at)
        .or_else(|| snapshot_timestamp(&record.updated_at))
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_terminal_time"))?
        .max(run.created_at_unix_ms);
    let (error_code, error_summary) = failure.map_or((None, None), |(code, summary)| {
        (Some(code.to_string()), Some(summary.to_string()))
    });
    let manifest: DynamicDemoRunManifest = serde_json::from_slice(&manifest_raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_terminal_manifest"))?;
    let terminal_run = product_run_for_demo_terminal(
        &manifest.config,
        record,
        lifecycle,
        completed_at_unix_ms,
        error_code.as_deref(),
        error_summary.as_deref(),
    )?;
    let mut result = match store
        .node_metrics(&run.supervisor_node_id)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "demo_snapshot_metrics"))
        .and_then(|metrics| {
            build_demo_snapshot_from_record(
                &terminal_run,
                record,
                &metrics,
                completed_at_unix_ms,
                DemoSnapshotStatus::Frozen,
            )
        }) {
        Ok(result) => result,
        Err(error)
            if lifecycle == RunLifecycle::Failed
                && failure.is_some_and(|(code, _)| code == "demo_runtime_unavailable")
                && error.kind == ProductErrorKind::SourceUnavailable =>
        {
            failed_demo_snapshot(&terminal_run, completed_at_unix_ms)
        }
        Err(error) => return Err(error),
    };
    result.provenance.result_ref = Some(format!(
        "artifact://demo-runs/{}/demo-result.json",
        run.run_id
    ));
    result.provenance.result_sha256 = None;
    let result_raw = serde_json::to_vec_pretty(&result)
        .map_err(|_| product_error(ProductErrorKind::DemoExecutionFailed, "demo_result"))?;
    match publish_new_run_file(&directory, "demo-result.json", &result_raw) {
        Ok(()) => {}
        Err(error) if error.kind == ProductErrorKind::Conflict => {
            let existing = read_backtest_result_bytes(&run_root.join("demo-result.json"))?;
            if existing != result_raw {
                return Err(product_error(
                    ProductErrorKind::SourceInvalid,
                    "demo_result",
                ));
            }
        }
        Err(error) => return Err(error),
    }
    let demo_result_sha256 = sha256_ref(&result_raw);
    let terminal = DynamicDemoRunTerminalState {
        schema_version: DEMO_RUN_TERMINAL_STATE_SCHEMA_VERSION.to_string(),
        source_manifest_sha256: sha256_ref(&manifest_raw),
        run_id: run.run_id.clone(),
        lifecycle,
        runtime: ProductRunRuntime {
            supervisor_node_id: run.supervisor_node_id.clone(),
            strategy_instance_id: run.strategy_instance_id.clone(),
            process_state: record.process.state,
            lifecycle_state: record.last_known_status.lifecycle_state,
        },
        started_at_unix_ms: run.started_at_unix_ms.or_else(|| {
            snapshot_timestamp(&record.last_known_status.started_at)
                .map(|started| started.max(run.created_at_unix_ms))
        }),
        completed_at_unix_ms,
        updated_at_unix_ms: completed_at_unix_ms,
        demo_result_sha256,
        error_code,
        error_summary,
    };
    let raw = serde_json::to_vec_pretty(&terminal)
        .map_err(|_| product_error(ProductErrorKind::DemoExecutionFailed, "demo_terminal_state"))?;
    let published_by_this_call = match publish_new_run_file(&directory, "terminal-state.json", &raw)
    {
        Ok(()) => true,
        Err(error) if error.kind == ProductErrorKind::Conflict => {
            let existing = read_backtest_result_bytes(&run_root.join("terminal-state.json"))?;
            if existing != raw {
                return Err(product_error(
                    ProductErrorKind::SourceInvalid,
                    "demo_terminal_anchor",
                ));
            }
            false
        }
        Err(error) => return Err(error),
    };
    if store
        .anchor_run_terminal(
            &run.supervisor_node_id,
            &run.run_id,
            manifest_sha256,
            SupervisorRunTerminalAnchor {
                lifecycle: match lifecycle {
                    RunLifecycle::Stopped => "stopped",
                    RunLifecycle::Failed => "failed",
                    _ => unreachable!("terminal lifecycle validated above"),
                }
                .to_string(),
                terminal_state_sha256: sha256_ref(&raw),
                completed_at_unix_ms,
            },
        )
        .is_err()
    {
        if published_by_this_call {
            directory.remove_file("terminal-state.json").map_err(|_| {
                product_error(
                    ProductErrorKind::DemoExecutionFailed,
                    "demo_terminal_anchor_cleanup",
                )
            })?;
        }
        return Err(product_error(
            ProductErrorKind::DemoExecutionFailed,
            "demo_terminal_anchor",
        ));
    }
    Ok(())
}

fn demo_run_manifest_sha256(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<String, ProductError> {
    let run_root = canonical_demo_artifact_root(state, false)?.join(run_id);
    read_backtest_result_bytes(&run_root.join("run-manifest.json")).map(|raw| sha256_ref(&raw))
}

pub(crate) fn shutdown_active_demo_run(
    registry_path: &Path,
    stop_timeout: Duration,
) -> anyhow::Result<()> {
    let state = DashboardServerState {
        registry_path: registry_path.to_path_buf(),
        workflow_root: None,
        ntpro_node_bin: PathBuf::new(),
        lifecycle_action_lock: std::sync::Arc::new(std::sync::Mutex::new(())),
        backtest_creation_gate: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
    };
    let _guard = state
        .lifecycle_action_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Demo lifecycle lock is unavailable during MVP shutdown"))?;
    let store = SupervisorRegistryStore::new(registry_path);
    let registry = store.load()?;
    let Some((node_id, ownership)) = registry.nodes.iter().find_map(|(node_id, record)| {
        record
            .run_ownership
            .values()
            .find(|ownership| ownership.terminal.is_none())
            .map(|ownership| (node_id.clone(), ownership.clone()))
    }) else {
        return Ok(());
    };
    let record = store.refresh_process_state(&node_id)?;
    let stopped = match record.process.state {
        SupervisorProcessState::NotStarted => return Ok(()),
        SupervisorProcessState::Stopped => {
            finalize_demo_run_ownerships(&state, unix_time_ms()).map_err(|error| {
                anyhow::anyhow!(
                    "failed to anchor stopped Demo during MVP shutdown: {:?}:{}",
                    error.kind,
                    error.field
                )
            })?;
            return Ok(());
        }
        SupervisorProcessState::Stale => {
            finalize_demo_run_ownerships(&state, unix_time_ms()).map_err(|error| {
                anyhow::anyhow!(
                    "failed to anchor failed Demo during MVP shutdown: {:?}:{}",
                    error.kind,
                    error.field
                )
            })?;
            return Ok(());
        }
        SupervisorProcessState::Unknown => {
            anyhow::bail!("Demo-owned node '{node_id}' process state is unknown")
        }
        SupervisorProcessState::Running => store.stop_node_process_for_run(
            &StopNodeRequest {
                node_id: node_id.clone(),
                stop_timeout,
            },
            &ownership.run_id,
            &ownership.manifest_sha256,
        )?,
    };
    let run_root = canonical_demo_artifact_root(&state, false)
        .map_err(|error| {
            anyhow::anyhow!(
                "failed to locate active Demo artifacts during MVP shutdown: {:?}:{}",
                error.kind,
                error.field
            )
        })?
        .join(&ownership.run_id);
    let manifest_raw =
        read_backtest_result_bytes(&run_root.join("run-manifest.json")).map_err(|error| {
            anyhow::anyhow!(
                "failed to load active Demo manifest during MVP shutdown: {:?}:{}",
                error.kind,
                error.field
            )
        })?;
    let manifest: DynamicDemoRunManifest = serde_json::from_slice(&manifest_raw)
        .map_err(|error| anyhow::anyhow!("active Demo manifest is invalid: {error}"))?;
    if manifest.schema_version != DEMO_RUN_MANIFEST_SCHEMA_VERSION
        || manifest.config.run_id != ownership.run_id
        || manifest.config.environment != RunEnvironment::Sandbox
        || manifest.config.demo_supervisor_node_id.as_deref() != Some(node_id.as_str())
        || sha256_ref(&manifest_raw) != ownership.manifest_sha256
    {
        anyhow::bail!("active Demo ownership does not match its immutable manifest");
    }
    validate_run_config_capabilities(&manifest.config).map_err(|error| {
        anyhow::anyhow!(
            "active Demo manifest violates product boundaries: {:?}:{}",
            error.kind,
            error.field
        )
    })?;
    let terminal_identity =
        demo_terminal_identity_from_config(&manifest.config).map_err(|error| {
            anyhow::anyhow!(
                "invalid active Demo identity during MVP shutdown: {:?}:{}",
                error.kind,
                error.field
            )
        })?;
    publish_demo_terminal_state(
        &state,
        &store,
        &terminal_identity,
        &stopped,
        &ownership.manifest_sha256,
        RunLifecycle::Stopped,
        None,
    )
    .map_err(|error| {
        anyhow::anyhow!(
            "failed to anchor stopped Demo during MVP shutdown: {:?}:{}",
            error.kind,
            error.field
        )
    })
}

fn wait_for_demo_metrics_artifact(path: &Path, timeout: Duration) -> Result<(), ()> {
    let deadline = Instant::now() + timeout;
    loop {
        if path.is_file() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_demo_snapshot(
    store: &SupervisorRegistryStore,
    run: &ProductRun,
    timeout: Duration,
) -> Result<(), ProductError> {
    let node_id = run
        .runtime
        .as_ref()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_snapshot_runtime"))?
        .supervisor_node_id
        .clone();
    let deadline = Instant::now() + timeout;
    loop {
        let snapshot = store
            .refresh_process_state(&node_id)
            .map_err(|_| {
                product_error(ProductErrorKind::DemoExecutionFailed, "demo_start_snapshot")
            })
            .and_then(|record| {
                store
                    .node_metrics(&node_id)
                    .map_err(|_| {
                        product_error(ProductErrorKind::DemoExecutionFailed, "demo_start_snapshot")
                    })
                    .and_then(|metrics| {
                        build_demo_snapshot_from_record(
                            run,
                            &record,
                            &metrics,
                            unix_time_ms(),
                            DemoSnapshotStatus::Running,
                        )
                        .map(|_| ())
                    })
            });
        match snapshot {
            Ok(()) => return Ok(()),
            Err(_) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Err(error) => return Err(error),
        }
    }
}

fn create_backtest_run(
    state: &DashboardServerState,
    request: CreateBacktestRunRequest,
    request_id: &str,
    reproduction: Option<ReproductionExpectation>,
) -> Result<(ProductRun, Option<BacktestReproductionProof>), ProductError> {
    let created_at = unix_time_ms();
    let source = load_product_source(state, created_at)?;
    let current_strategy_version =
        strategy_version::load_product_strategy_version(&source, created_at)?;
    let strategy_version = reproduction
        .as_ref()
        .map_or(current_strategy_version, |expectation| {
            expectation.strategy_version.clone()
        });
    validate_backtest_creation_request(&request, &source, &strategy_version)?;
    if load_run_configs(state, &source)?.len() >= MAX_PAGE_LIMIT {
        return Err(product_error(ProductErrorKind::Conflict, "run_capacity"));
    }

    let run_id = request_id.replacen("product-", "backtest-", 1);
    validate_identifier("run_id", &run_id)?;
    let config_ref = format!("artifact://backtests/{run_id}/request.toml");
    let result_ref = format!("artifact://backtests/{run_id}/summary.json");
    let risk_ref = format!("artifact://backtests/{run_id}/run-manifest.json#risk");
    let instrument_id = strategy_version
        .data_symbols()
        .first()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "strategy_data_symbol"))?;
    let engine_venue = instrument_id
        .rsplit_once('.')
        .map(|(_, venue)| venue)
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "strategy_data_symbol"))?;
    let config_raw = build_backtest_config(
        &run_id,
        instrument_id,
        engine_venue,
        &request,
        strategy_version.content_hash(),
        &config_ref,
        &result_ref,
    )?;
    let request_sha256 = sha256_ref(&config_raw);
    let strategy_version_raw =
        strategy_version::serialize_strategy_version_snapshot(&strategy_version)?;
    let strategy_version_snapshot_sha256 = sha256_ref(&strategy_version_raw);
    let run_directory = create_dynamic_run_directory(state, &run_id)?;
    write_new_run_file(&run_directory, "request.toml", &config_raw)?;
    write_new_run_file(
        &run_directory,
        "strategy-version.json",
        &strategy_version_raw,
    )?;

    let started_at = unix_time_ms();
    let execution = crate::backtest::execute_product_backtest(&config_raw);
    let completed_at = unix_time_ms().max(started_at);
    let config = match execution {
        Ok(artifacts) => {
            let artifact: BacktestResultArtifact = serde_json::from_slice(&artifacts.summary)
                .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "backtest_result"))?;
            let details: BacktestDetailsArtifact = serde_json::from_slice(&artifacts.details)
                .map_err(|_| {
                    product_error(ProductErrorKind::ExecutionFailed, "backtest_details")
                })?;
            let analysis: BacktestAnalysisArtifact = serde_json::from_slice(&artifacts.analysis)
                .map_err(|_| {
                    product_error(ProductErrorKind::ExecutionFailed, "backtest_analysis")
                })?;
            let result_sha256 = sha256_ref(&artifacts.summary);
            let details_sha256 = sha256_ref(&artifacts.details);
            if details.run_id != run_id
                || details.strategy_id != request.strategy_id
                || details.strategy_version_id != request.strategy_version_id
                || details.data_sha256 != artifact.data_sha256
                || details.config_sha256 != request_sha256
                || details.details_ref != format!("artifact://backtests/{run_id}/details.json")
                || analysis.run_id != run_id
                || analysis.strategy_id != request.strategy_id
                || analysis.strategy_version_id != request.strategy_version_id
                || analysis.analysis_ref != format!("artifact://backtests/{run_id}/analysis.json")
                || analysis.provenance.summary_sha256 != result_sha256
                || analysis.provenance.details_sha256 != details_sha256
            {
                return Err(product_error(
                    ProductErrorKind::ExecutionFailed,
                    "backtest_analysis",
                ));
            }
            let analysis_sha256 = sha256_ref(&artifacts.analysis);
            let reproduction_proof = reproduction
                .as_ref()
                .map(|expectation| {
                    let reproduced_input_sha256 = backtest_reproduction_input_sha256(
                        &request,
                        strategy_version.content_hash(),
                        &artifact.data_sha256,
                        &artifact.instrument_id,
                    )?;
                    let reproduced_output_sha256 = backtest_reproduction_output_sha256(
                        &run_id, &artifact, &details, &analysis,
                    )?;
                    if reproduced_input_sha256 != expectation.source_input_sha256
                        || reproduced_output_sha256 != expectation.source_output_sha256
                    {
                        return Err(product_error(
                            ProductErrorKind::ExecutionFailed,
                            "backtest_reproduction_mismatch",
                        ));
                    }
                    Ok(BacktestReproductionProof {
                        schema_version: BACKTEST_REPRODUCTION_PROOF_SCHEMA_VERSION.to_string(),
                        source_run_id: expectation.source_run_id.clone(),
                        reproduced_run_id: run_id.clone(),
                        proof_ref: format!("artifact://backtests/{run_id}/reproduction.json"),
                        source_input_sha256: expectation.source_input_sha256.clone(),
                        reproduced_input_sha256,
                        source_output_sha256: expectation.source_output_sha256.clone(),
                        reproduced_output_sha256,
                        input_equivalent: true,
                        output_equivalent: true,
                        user_initiated: true,
                        automatic_retry_allowed: false,
                        automatic_remediation_allowed: false,
                    })
                })
                .transpose()?;
            let reproduction_raw = reproduction_proof
                .as_ref()
                .map(|proof| {
                    serde_json::to_string_pretty(proof)
                        .map(|value| format!("{value}\n").into_bytes())
                        .map_err(|_| {
                            product_error(
                                ProductErrorKind::ExecutionFailed,
                                "backtest_reproduction_proof",
                            )
                        })
                })
                .transpose()?;
            let config = ProductRunConfig {
                run_id: run_id.clone(),
                strategy_id: request.strategy_id,
                strategy_version_id: request.strategy_version_id,
                environment: RunEnvironment::Backtest,
                data_ref: request.data_ref,
                config_ref,
                adapter_ref: "adapter://backtest/simulated".to_string(),
                account_ref: format!("account://simulated/{run_id}"),
                venue_ref: request.venue_ref,
                lifecycle: RunLifecycle::Completed,
                result_status: RunResultStatus::Available,
                result_ref: Some(result_ref),
                backtest_config_sha256: Some(request_sha256.clone()),
                backtest_data_sha256: Some(artifact.data_sha256.clone()),
                backtest_result_sha256: Some(result_sha256),
                backtest_details_sha256: Some(details_sha256),
                backtest_analysis_sha256: Some(analysis_sha256),
                strategy_version_snapshot_sha256: Some(strategy_version_snapshot_sha256),
                reproduction_source_run_id: reproduction_proof
                    .as_ref()
                    .map(|proof| proof.source_run_id.clone()),
                reproduction_input_sha256: reproduction_proof
                    .as_ref()
                    .map(|proof| proof.reproduced_input_sha256.clone()),
                reproduction_output_sha256: reproduction_proof
                    .as_ref()
                    .map(|proof| proof.reproduced_output_sha256.clone()),
                reproduction_proof_sha256: reproduction_raw.as_ref().map(|raw| sha256_ref(raw)),
                backtest_trade_size: Some(artifact.parameters.trade_size.clone()),
                backtest_quotes: Some(artifact.metrics.quotes),
                backtest_fast_period: Some(artifact.parameters.fast_period),
                backtest_slow_period: Some(artifact.parameters.slow_period),
                risk_status: RunRiskStatus::Passed,
                risk_ref,
                error_code: None,
                error_summary: None,
                created_at_unix_ms: created_at,
                started_at_unix_ms: Some(started_at),
                completed_at_unix_ms: Some(completed_at),
                updated_at_unix_ms: completed_at,
                external_venue_connection: false,
                order_submission_allowed: false,
                order_mutation_allowed: false,
                automatic_retry_allowed: false,
                automatic_remediation_allowed: false,
                real_orders_submitted: false,
                trading_controls_enabled: false,
                demo_supervisor_node_id: None,
                demo_strategy_instance_id: None,
                demo_identity_contract_id: None,
                demo_supervisor_record_baseline_unix_ms: None,
                demo_process_state: None,
                demo_lifecycle_state: None,
            };
            let expected_version_id = strategy_version.strategy_version_id();
            validate_created_backtest_artifacts(
                &config,
                &source,
                &strategy_version,
                expected_version_id,
                completed_at,
                &CreatedBacktestArtifacts {
                    summary: &artifact,
                    details: &details,
                    analysis: &analysis,
                },
            )?;
            write_new_run_file(&run_directory, "summary.json", &artifacts.summary)?;
            write_new_run_file(&run_directory, "details.json", &artifacts.details)?;
            write_new_run_file(&run_directory, "analysis.json", &artifacts.analysis)?;
            if let Some(raw) = reproduction_raw.as_deref() {
                write_new_run_file(&run_directory, "reproduction.json", raw)?;
            }
            config
        }
        Err(error) => {
            let summary = sanitize_execution_error(&error.to_string());
            let config = ProductRunConfig {
                run_id: run_id.clone(),
                strategy_id: request.strategy_id,
                strategy_version_id: request.strategy_version_id,
                environment: RunEnvironment::Backtest,
                data_ref: request.data_ref,
                config_ref,
                adapter_ref: "adapter://backtest/simulated".to_string(),
                account_ref: format!("account://simulated/{run_id}"),
                venue_ref: request.venue_ref,
                lifecycle: RunLifecycle::Failed,
                result_status: RunResultStatus::Unavailable,
                result_ref: None,
                backtest_config_sha256: None,
                backtest_data_sha256: None,
                backtest_result_sha256: None,
                backtest_details_sha256: None,
                backtest_analysis_sha256: None,
                strategy_version_snapshot_sha256: Some(strategy_version_snapshot_sha256),
                reproduction_source_run_id: None,
                reproduction_input_sha256: None,
                reproduction_output_sha256: None,
                reproduction_proof_sha256: None,
                backtest_trade_size: None,
                backtest_quotes: None,
                backtest_fast_period: None,
                backtest_slow_period: None,
                risk_status: RunRiskStatus::Blocked,
                risk_ref,
                error_code: Some("backtest_execution_failed".to_string()),
                error_summary: Some(summary),
                created_at_unix_ms: created_at,
                started_at_unix_ms: Some(started_at),
                completed_at_unix_ms: Some(completed_at),
                updated_at_unix_ms: completed_at,
                external_venue_connection: false,
                order_submission_allowed: false,
                order_mutation_allowed: false,
                automatic_retry_allowed: false,
                automatic_remediation_allowed: false,
                real_orders_submitted: false,
                trading_controls_enabled: false,
                demo_supervisor_node_id: None,
                demo_strategy_instance_id: None,
                demo_identity_contract_id: None,
                demo_supervisor_record_baseline_unix_ms: None,
                demo_process_state: None,
                demo_lifecycle_state: None,
            };
            write_dynamic_manifest(&run_directory, &request_sha256, &config)?;
            return Err(product_error(
                ProductErrorKind::ExecutionFailed,
                "backtest_engine",
            ));
        }
    };
    write_dynamic_manifest(&run_directory, &request_sha256, &config)?;
    let expected_version_id = strategy_version.strategy_version_id();
    let run = validate_and_project_run(
        config,
        &source,
        &strategy_version,
        expected_version_id,
        completed_at,
        Some(format!("artifact://backtests/{run_id}/run-manifest.json")),
    )?;
    let proof = if let Some(expected) = reproduction {
        Some(load_backtest_reproduction_proof(
            state,
            &run,
            &expected.source_run_id,
        )?)
    } else {
        None
    };
    Ok((run, proof))
}

struct CreatedBacktestArtifacts<'a> {
    summary: &'a BacktestResultArtifact,
    details: &'a BacktestDetailsArtifact,
    analysis: &'a BacktestAnalysisArtifact,
}

fn validate_created_backtest_artifacts(
    config: &ProductRunConfig,
    source: &ValidatedProductSource,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected_version_id: &str,
    completed_at: u64,
    artifacts: &CreatedBacktestArtifacts<'_>,
) -> Result<(), ProductError> {
    let run = validate_and_project_run(
        config.clone(),
        source,
        strategy_version,
        expected_version_id,
        completed_at,
        Some(format!(
            "artifact://backtests/{}/run-manifest.json",
            config.run_id
        )),
    )?;
    let expected = backtest_result_expectation(config)?
        .ok_or_else(|| product_error(ProductErrorKind::ExecutionFailed, "backtest_result"))?;
    validate_backtest_result_artifact(artifacts.summary, &run, strategy_version, &expected)?;
    validate_backtest_details_artifact(
        artifacts.details,
        &run,
        strategy_version,
        &expected,
        artifacts.summary,
    )?;
    validate_backtest_analysis_artifact(
        artifacts.analysis,
        &run,
        strategy_version,
        &expected,
        artifacts.summary,
        artifacts.details,
        config
            .backtest_details_sha256
            .as_deref()
            .ok_or_else(|| product_error(ProductErrorKind::ExecutionFailed, "backtest_analysis"))?,
    )
}

fn build_backtest_config(
    run_id: &str,
    instrument_id: &str,
    engine_venue: &str,
    request: &CreateBacktestRunRequest,
    content_hash: &str,
    config_ref: &str,
    result_ref: &str,
) -> Result<Vec<u8>, ProductError> {
    let quoted = |value: &str| {
        serde_json::to_string(value)
            .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "backtest_config"))
    };
    let raw = format!(
        "[run]\nid = {}\nmode = \"engine-smoke\"\n\n[data]\nsource = \"synthetic-quotes\"\ninstrument_id = {}\nquotes = {}\n\n[strategy]\nname = \"ema-cross\"\ntrade_size = {}\nfast_period = {}\nslow_period = {}\n\n[venue]\nname = {}\nstarting_balance = {}\n\n[product]\nstrategy_id = {}\nstrategy_version_id = {}\nstrategy_version_content_hash = {}\ndata_ref = {}\nconfig_ref = {}\nresult_ref = {}\n",
        quoted(run_id)?,
        quoted(instrument_id)?,
        request.quotes,
        quoted(&request.trade_size)?,
        request.fast_period,
        request.slow_period,
        quoted(engine_venue)?,
        quoted(&request.starting_balance)?,
        quoted(&request.strategy_id)?,
        quoted(&request.strategy_version_id)?,
        quoted(content_hash)?,
        quoted(&request.data_ref)?,
        quoted(config_ref)?,
        quoted(result_ref)?,
    );
    Ok(raw.into_bytes())
}

fn validate_backtest_creation_request(
    request: &CreateBacktestRunRequest,
    source: &ValidatedProductSource,
    strategy_version: &strategy_version::ProductStrategyVersion,
) -> Result<(), ProductError> {
    if request.environment != RunEnvironment::Backtest {
        return Err(product_error(ProductErrorKind::BadRequest, "environment"));
    }
    let expected_version_id = strategy_version.strategy_version_id();
    if request.strategy_id != source.strategy.strategy_id
        || request.strategy_id != strategy_version.strategy_id()
        || request.strategy_version_id != expected_version_id
    {
        return Err(product_error(
            ProductErrorKind::BadRequest,
            "strategy_version_id",
        ));
    }
    let expected_data_ref = format!(
        "dataset://fixtures/{}",
        source.strategy.strategy_id.replace('_', "-")
    );
    if request.data_ref != expected_data_ref {
        return Err(product_error(ProductErrorKind::BadRequest, "data_ref"));
    }
    if !strategy_version
        .data_venues()
        .iter()
        .any(|venue| request.venue_ref == format!("venue://simulated/{venue}"))
    {
        return Err(product_error(ProductErrorKind::BadRequest, "venue_ref"));
    }
    if !(30..=10_000).contains(&request.quotes) {
        return Err(product_error(ProductErrorKind::BadRequest, "quotes"));
    }
    if request.fast_period == 0
        || request.fast_period >= request.slow_period
        || request.slow_period > 500
        || request.quotes <= request.slow_period
        || strategy_version.parameter_const_u64("fast_period") != Some(request.fast_period as u64)
        || strategy_version.parameter_const_u64("slow_period") != Some(request.slow_period as u64)
    {
        return Err(product_error(
            ProductErrorKind::BadRequest,
            "strategy_parameters",
        ));
    }
    let trade_size = request
        .trade_size
        .parse::<Quantity>()
        .map_err(|_| product_error(ProductErrorKind::BadRequest, "trade_size"))?;
    if trade_size.raw == 0 || trade_size.precision != 6 {
        return Err(product_error(ProductErrorKind::BadRequest, "trade_size"));
    }
    let starting_balance = request
        .starting_balance
        .parse::<Money>()
        .map_err(|_| product_error(ProductErrorKind::BadRequest, "starting_balance"))?;
    if starting_balance.raw <= 0 || starting_balance.currency.to_string() != "USDT" {
        return Err(product_error(
            ProductErrorKind::BadRequest,
            "starting_balance",
        ));
    }
    Ok(())
}

fn create_dynamic_run_directory(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<cap_std::fs::Dir, ProductError> {
    let artifact_root = canonical_backtest_artifact_root(state)?;
    let root = open_absolute_directory_nofollow(&artifact_root)?;
    root.create_dir(run_id).map_err(|error| {
        if error.kind() == ErrorKind::AlreadyExists {
            product_error(ProductErrorKind::Conflict, "run_id")
        } else {
            product_error(ProductErrorKind::SourceUnavailable, "result_root")
        }
    })?;
    root.open_dir_nofollow(run_id)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "result_root_containment"))
}

fn write_new_run_file(
    directory: &cap_std::fs::Dir,
    name: &str,
    raw: &[u8],
) -> Result<(), ProductError> {
    use cap_std::fs::OpenOptions;

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    let mut file = directory.open_with(name, &options).map_err(|error| {
        if error.kind() == ErrorKind::AlreadyExists {
            product_error(ProductErrorKind::Conflict, name)
        } else {
            product_error(ProductErrorKind::SourceUnavailable, name)
        }
    })?;
    file.write_all(raw)
        .and_then(|()| file.sync_all())
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, name))
}

fn write_dynamic_manifest(
    directory: &cap_std::fs::Dir,
    request_sha256: &str,
    config: &ProductRunConfig,
) -> Result<(), ProductError> {
    let manifest = DynamicBacktestRunManifest {
        schema_version: BACKTEST_RUN_MANIFEST_SCHEMA_VERSION.to_string(),
        request_sha256: request_sha256.to_string(),
        config: config.clone(),
    };
    let raw = format!(
        "{}\n",
        serde_json::to_string_pretty(&manifest)
            .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "run_manifest"))?
    );
    publish_new_run_file(directory, "run-manifest.json", raw.as_bytes())
}

pub(super) fn publish_new_run_file(
    directory: &cap_std::fs::Dir,
    name: &str,
    raw: &[u8],
) -> Result<(), ProductError> {
    let sequence = RUN_MANIFEST_TEMP_SEQUENCE.fetch_add(1, AtomicOrdering::Relaxed);
    let temp_name = format!(".{name}.tmp.{}.{}", std::process::id(), sequence);
    let result = (|| {
        write_new_run_file(directory, &temp_name, raw)?;
        directory
            .hard_link(&temp_name, directory, name)
            .map_err(|error| {
                if error.kind() == ErrorKind::AlreadyExists {
                    product_error(ProductErrorKind::Conflict, name)
                } else {
                    product_error(ProductErrorKind::SourceUnavailable, name)
                }
            })?;
        if let Ok(parent) = directory.open(".") {
            let _ = parent.sync_all();
        }
        Ok(())
    })();
    let _ = directory.remove_file(&temp_name);
    result
}

fn sanitize_execution_error(value: &str) -> String {
    let summary: String = value
        .chars()
        .filter(|character| !character.is_control())
        .take(300)
        .collect();
    if summary.trim().is_empty() {
        "回测引擎执行失败".to_string()
    } else {
        summary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunSort {
    RunId,
    CreatedAt,
    UpdatedAt,
}

pub(in crate::dashboard) async fn run_list_api(
    State(state): State<DashboardServerState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<RunListResponse> {
    let request_id = product_request_id();
    let result = parse_run_list_query(raw_query.as_deref()).and_then(|query| {
        project_run_list(
            load_product_runs(&state, unix_time_ms())?,
            &query,
            request_id.clone(),
        )
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn run_detail_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<RunDetailResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(run_id)| run_id).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "run_id"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_requested_run_id("run_id", &run_id)?;
        let run = load_product_runs(&state, unix_time_ms())?
            .into_iter()
            .find(|run| run.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
        Ok(RunDetailResponse {
            schema_version: RUN_DETAIL_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: run,
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn demo_run_snapshot_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<DemoRunSnapshotResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(run_id)| run_id).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "run_id"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_requested_run_id("run_id", &run_id)?;
        let data = load_demo_snapshot_by_id(&state, &run_id, unix_time_ms())?;
        Ok(DemoRunSnapshotResponse {
            schema_version: DEMO_RUN_SNAPSHOT_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data,
            boundaries: DemoSnapshotBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn run_metrics_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<RunMetricsResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(run_id)| run_id).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "run_id"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_requested_run_id("run_id", &run_id)?;
        let source = load_product_source(&state, unix_time_ms())?;
        let run = load_product_runs(&state, unix_time_ms())?
            .into_iter()
            .find(|run| run.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
        if run.environment != RunEnvironment::Backtest
            || run.lifecycle != RunLifecycle::Completed
            || run.result.status != RunResultStatus::Available
        {
            return Err(product_error(ProductErrorKind::RunNotFound, "run_metrics"));
        }
        let (_, strategy_version) = load_run_config_and_version(&state, &source, &run_id)?;
        let expected = load_backtest_result_expectation(&state, &source, &run.run_id)?;
        let artifact = load_backtest_result_artifact(&state, &run, &strategy_version, &expected)?;
        Ok(RunMetricsResponse {
            schema_version: RUN_METRICS_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: artifact,
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn run_report_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<RunReportResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(run_id)| run_id).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "run_id"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_requested_run_id("run_id", &run_id)?;
        let source = load_product_source(&state, unix_time_ms())?;
        let run = load_product_runs(&state, unix_time_ms())?
            .into_iter()
            .find(|run| run.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
        let (config, strategy_version) = load_run_config_and_version(&state, &source, &run_id)?;
        let details_sha256 = config
            .backtest_details_sha256
            .as_deref()
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_report"))?;
        let expected = load_backtest_result_expectation(&state, &source, &run.run_id)?;
        let summary = load_backtest_result_artifact(&state, &run, &strategy_version, &expected)?;
        let artifact = load_backtest_details_artifact(
            &state,
            &run,
            &strategy_version,
            &expected,
            &summary,
            details_sha256,
        )?;
        Ok(RunReportResponse {
            schema_version: RUN_REPORT_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: artifact,
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn run_analysis_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<RunAnalysisResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(run_id)| run_id).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "run_id"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_requested_run_id("run_id", &run_id)?;
        let source = load_product_source(&state, unix_time_ms())?;
        let run = load_product_runs(&state, unix_time_ms())?
            .into_iter()
            .find(|run| run.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
        let (config, strategy_version) = load_run_config_and_version(&state, &source, &run_id)?;
        let details_sha256 = config
            .backtest_details_sha256
            .as_deref()
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_analysis"))?;
        let analysis_sha256 = config
            .backtest_analysis_sha256
            .as_deref()
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_analysis"))?;
        let expected = load_backtest_result_expectation(&state, &source, &run.run_id)?;
        let summary = load_backtest_result_artifact(&state, &run, &strategy_version, &expected)?;
        let details = load_backtest_details_artifact(
            &state,
            &run,
            &strategy_version,
            &expected,
            &summary,
            details_sha256,
        )?;
        let artifact = load_backtest_analysis_artifact(
            &state,
            &run,
            &strategy_version,
            &expected,
            &summary,
            &details,
            details_sha256,
            analysis_sha256,
        )?;
        Ok(RunAnalysisResponse {
            schema_version: RUN_ANALYSIS_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: artifact,
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

fn demo_comparison_risk(
    simulation: &DemoSimulationSnapshot,
) -> Result<RunComparisonRisk, ProductError> {
    let first = simulation
        .equity_curve
        .first()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_equity_curve"))?;
    let last = simulation
        .equity_curve
        .last()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_equity_curve"))?;
    let starting = first
        .total
        .parse::<Money>()
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_equity_curve"))?;
    let ending = last
        .total
        .parse::<Money>()
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_equity_curve"))?;
    let mut peak = starting.as_decimal();
    let mut max_drawdown_rate = Decimal::ZERO;
    for point in &simulation.equity_curve {
        let equity = point
            .total
            .parse::<Money>()
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_equity_curve"))?;
        if equity.currency != starting.currency {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_equity_currency",
            ));
        }
        peak = peak.max(equity.as_decimal());
        if peak > Decimal::ZERO {
            max_drawdown_rate = max_drawdown_rate.max((peak - equity.as_decimal()) / peak);
        }
    }
    let open_positions = simulation
        .positions
        .iter()
        .filter(|position| position.ts_closed.is_none())
        .count();
    Ok(RunComparisonRisk {
        currency: starting.currency.to_string(),
        starting_equity: starting.to_string(),
        ending_equity: ending.to_string(),
        max_drawdown_rate: canonical_analysis_decimal(max_drawdown_rate),
        open_positions,
        closed_positions: simulation.positions.len().saturating_sub(open_positions),
    })
}

pub(in crate::dashboard) async fn run_comparison_api(
    State(state): State<DashboardServerState>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<RunComparisonResponse> {
    let request_id = product_request_id();
    let result = parse_run_comparison_query(raw_query.as_deref()).and_then(|run_ids| {
        let source = load_product_source(&state, unix_time_ms())?;
        let runs = load_product_runs(&state, unix_time_ms())?;
        let mut items = Vec::with_capacity(run_ids.len());
        for run_id in &run_ids {
            let run = runs
                .iter()
                .find(|run| run.run_id == *run_id)
                .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_ids"))?;
            match run.environment {
                RunEnvironment::Backtest => {
                    let bundle = load_verified_backtest_bundle(&state, &source, run_id)?;
                    items.push(RunComparisonItem {
                        run_id: bundle.run.run_id,
                        environment: RunEnvironment::Backtest,
                        strategy_id: bundle.run.strategy_id,
                        strategy_version_id: bundle.run.strategy_version_id,
                        data_ref: bundle.summary.data_ref.clone(),
                        data_sha256: bundle.summary.data_sha256.clone(),
                        config_sha256: bundle.summary.config_sha256.clone(),
                        instrument_id: bundle.summary.instrument_id.clone(),
                        parameters: bundle.summary.parameters.clone(),
                        metrics: RunComparisonMetrics {
                            market_event_count: bundle.summary.metrics.quotes,
                            fill_count: bundle.details.trades.len(),
                            position_count: bundle.details.positions.len(),
                        },
                        risk: RunComparisonRisk {
                            currency: bundle.analysis.risk.currency.clone(),
                            starting_equity: bundle.analysis.risk.starting_equity.clone(),
                            ending_equity: bundle.analysis.risk.ending_equity.clone(),
                            max_drawdown_rate: bundle.analysis.risk.max_drawdown_rate.clone(),
                            open_positions: bundle.analysis.risk.open_positions,
                            closed_positions: bundle.analysis.risk.closed_positions,
                        },
                        provenance: RunComparisonProvenance {
                            engine: bundle.analysis.provenance.generator.clone(),
                            data_ref: bundle.analysis.provenance.data_ref.clone(),
                            data_sha256: bundle.analysis.provenance.data_sha256.clone(),
                            source_refs: vec![
                                bundle.analysis.provenance.summary_ref.clone(),
                                bundle.analysis.provenance.details_ref.clone(),
                                bundle.analysis.analysis_ref.clone(),
                            ],
                        },
                        reproduction_ref: bundle.run.result.reproduction_ref,
                    });
                }
                RunEnvironment::Sandbox if run.lifecycle == RunLifecycle::Stopped => {
                    let snapshot = load_demo_snapshot_by_id(&state, run_id, unix_time_ms())?;
                    if snapshot.snapshot_status != DemoSnapshotStatus::Frozen {
                        return Err(product_error(
                            ProductErrorKind::SourceInvalid,
                            "demo_comparison_snapshot",
                        ));
                    }
                    let simulation = snapshot.simulation.ok_or_else(|| {
                        product_error(
                            ProductErrorKind::SourceInvalid,
                            "demo_comparison_simulation",
                        )
                    })?;
                    let risk = demo_comparison_risk(&simulation)?;
                    items.push(RunComparisonItem {
                        run_id: run.run_id.clone(),
                        environment: RunEnvironment::Sandbox,
                        strategy_id: run.strategy_id.clone(),
                        strategy_version_id: run.strategy_version_id.clone(),
                        data_ref: run.data_ref.clone(),
                        data_sha256: simulation.summary.data_sha256.clone(),
                        config_sha256: demo_run_manifest_sha256(&state, run_id)?,
                        instrument_id: simulation.summary.instrument_id.clone(),
                        parameters: BacktestParameters {
                            trade_size: simulation.summary.parameters.trade_size.clone(),
                            fast_period: simulation.summary.parameters.fast_period,
                            slow_period: simulation.summary.parameters.slow_period,
                        },
                        metrics: RunComparisonMetrics {
                            market_event_count: snapshot.session.as_ref().map_or(0, |session| {
                                usize::try_from(session.market_event_count).unwrap_or(usize::MAX)
                            }),
                            fill_count: simulation.fills.len(),
                            position_count: simulation.positions.len(),
                        },
                        risk,
                        provenance: RunComparisonProvenance {
                            engine: simulation.summary.engine.clone(),
                            data_ref: run.data_ref.clone(),
                            data_sha256: simulation.summary.data_sha256.clone(),
                            source_refs: snapshot.provenance.source_refs,
                        },
                        reproduction_ref: None,
                    });
                }
                _ => {
                    return Err(product_error(
                        ProductErrorKind::RunNotFound,
                        "run_comparison",
                    ));
                }
            }
        }
        let first = items
            .first()
            .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "run_ids"))?;
        let same_strategy_version = items
            .iter()
            .all(|item| item.strategy_version_id == first.strategy_version_id);
        let same_data = items
            .iter()
            .all(|item| item.data_sha256 == first.data_sha256);
        let same_instrument = items
            .iter()
            .all(|item| item.instrument_id == first.instrument_id);
        let same_currency = items
            .iter()
            .all(|item| item.risk.currency == first.risk.currency);
        let same_strategy = items
            .iter()
            .all(|item| item.strategy_id == first.strategy_id);
        let same_environment = items
            .iter()
            .all(|item| item.environment == first.environment);
        let behaviorally_comparable =
            same_strategy && same_strategy_version && same_instrument && same_currency;
        Ok(RunComparisonResponse {
            schema_version: RUN_COMPARISON_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: RunComparison {
                baseline_run_id: run_ids[0].clone(),
                run_ids,
                items,
                compatibility: RunComparisonCompatibility {
                    same_strategy,
                    same_strategy_version,
                    same_data,
                    same_instrument,
                    same_currency,
                    same_environment,
                    behaviorally_comparable,
                    directly_comparable: behaviorally_comparable && same_data,
                },
            },
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn run_reproduce_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    payload: Result<Json<ReproduceBacktestRunRequest>, JsonRejection>,
) -> ApiStatusResult<RunReproductionResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(run_id)| run_id).map_err(|_| {
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
    validate_requested_run_id("run_id", &run_id)
        .and_then(|()| validate_requested_run_id("source_run_id", &request.source_run_id))
        .and_then(|()| {
            if request.source_run_id != run_id || !request.deterministic_replay {
                Err(product_error(
                    ProductErrorKind::BadRequest,
                    "deterministic_replay",
                ))
            } else {
                Ok(())
            }
        })
        .map_err(|error| product_error_response(&error, &request_id))?;
    let permit = state
        .backtest_creation_gate
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            product_error_response(
                &product_error(ProductErrorKind::Conflict, "backtest_creation_in_progress"),
                &request_id,
            )
        })?;
    let worker_state = state.clone();
    let worker_request_id = request_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        reproduce_backtest_run(&worker_state, &run_id, &worker_request_id)
    })
    .await
    .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "backtest_worker"))
    .and_then(|result| result);

    result
        .map(|data| {
            (
                StatusCode::CREATED,
                Json(RunReproductionResponse {
                    schema_version: RUN_REPRODUCTION_SCHEMA_VERSION.to_string(),
                    contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
                    request_id: request_id.clone(),
                    data,
                    boundaries: BacktestRunCreationBoundaries::enforced(),
                }),
            )
        })
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn run_reproduction_proof_api(
    State(state): State<DashboardServerState>,
    run_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<RunReproductionProofResponse> {
    let request_id = product_request_id();
    let run_id = run_path.map(|AxumPath(run_id)| run_id).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "run_id"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_requested_run_id("run_id", &run_id)?;
        let source = load_product_source(&state, unix_time_ms())?;
        let config = load_run_configs(&state, &source)?
            .into_iter()
            .map(|(config, _)| config)
            .find(|config| config.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
        let source_run_id = config
            .reproduction_source_run_id
            .as_deref()
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_reproduction"))?;
        let run = load_product_runs(&state, unix_time_ms())?
            .into_iter()
            .find(|run| run.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
        let proof = load_backtest_reproduction_proof(&state, &run, source_run_id)?;
        Ok(RunReproductionProofResponse {
            schema_version: RUN_REPRODUCTION_PROOF_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: proof,
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

fn parse_run_comparison_query(raw_query: Option<&str>) -> Result<Vec<String>, ProductError> {
    let values = parse_query_values(raw_query)?;
    if values.len() != 1 || !values.contains_key("run_ids") {
        return Err(product_error(ProductErrorKind::BadRequest, "run_ids"));
    }
    let run_ids = values["run_ids"]
        .split(',')
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !(2..=4).contains(&run_ids.len())
        || run_ids.iter().collect::<BTreeSet<_>>().len() != run_ids.len()
    {
        return Err(product_error(ProductErrorKind::BadRequest, "run_ids"));
    }
    for run_id in &run_ids {
        validate_requested_run_id("run_ids", run_id)?;
    }
    Ok(run_ids)
}

fn reproduce_backtest_run(
    state: &DashboardServerState,
    source_run_id: &str,
    request_id: &str,
) -> Result<BacktestReproduction, ProductError> {
    let source = load_product_source(state, unix_time_ms())?;
    let source_bundle = load_verified_backtest_bundle(state, &source, source_run_id)?;
    let request = load_stored_backtest_request(state, &source_bundle)?;
    validate_backtest_creation_request(&request, &source, &source_bundle.strategy_version)?;
    let source_input_sha256 = backtest_reproduction_input_sha256(
        &request,
        source_bundle.strategy_version.content_hash(),
        &source_bundle.summary.data_sha256,
        &source_bundle.summary.instrument_id,
    )?;
    let source_output_sha256 = backtest_reproduction_output_sha256(
        source_run_id,
        &source_bundle.summary,
        &source_bundle.details,
        &source_bundle.analysis,
    )?;
    let (reproduced_run, proof) = create_backtest_run(
        state,
        request,
        request_id,
        Some(ReproductionExpectation {
            source_run_id: source_run_id.to_string(),
            source_input_sha256,
            source_output_sha256,
            strategy_version: source_bundle.strategy_version,
        }),
    )?;
    let proof = proof.ok_or_else(|| {
        product_error(
            ProductErrorKind::ExecutionFailed,
            "backtest_reproduction_proof",
        )
    })?;
    Ok(BacktestReproduction {
        source_run_id: source_run_id.to_string(),
        reproduced_run,
        proof,
    })
}

fn load_verified_backtest_bundle(
    state: &DashboardServerState,
    source: &ValidatedProductSource,
    run_id: &str,
) -> Result<VerifiedBacktestBundle, ProductError> {
    let run = load_product_runs(state, unix_time_ms())?
        .into_iter()
        .find(|run| run.run_id == run_id)
        .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
    if run.environment != RunEnvironment::Backtest
        || run.lifecycle != RunLifecycle::Completed
        || run.result.status != RunResultStatus::Available
    {
        return Err(product_error(
            ProductErrorKind::RunNotFound,
            "run_comparison",
        ));
    }
    let (config, strategy_version) = load_run_config_and_version(state, source, run_id)?;
    let details_sha256 = config
        .backtest_details_sha256
        .as_deref()
        .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_report"))?;
    let analysis_sha256 = config
        .backtest_analysis_sha256
        .as_deref()
        .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_analysis"))?;
    let expected = load_backtest_result_expectation(state, source, run_id)?;
    let summary = load_backtest_result_artifact(state, &run, &strategy_version, &expected)?;
    let details = load_backtest_details_artifact(
        state,
        &run,
        &strategy_version,
        &expected,
        &summary,
        details_sha256,
    )?;
    let analysis = load_backtest_analysis_artifact(
        state,
        &run,
        &strategy_version,
        &expected,
        &summary,
        &details,
        details_sha256,
        analysis_sha256,
    )?;
    Ok(VerifiedBacktestBundle {
        run,
        config,
        strategy_version,
        summary,
        details,
        analysis,
    })
}

fn load_run_config_and_version(
    state: &DashboardServerState,
    source: &ValidatedProductSource,
    run_id: &str,
) -> Result<(ProductRunConfig, strategy_version::ProductStrategyVersion), ProductError> {
    let (config, source_ref) = load_run_configs(state, source)?
        .into_iter()
        .find(|(config, _)| config.run_id == run_id)
        .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
    let current = strategy_version::load_product_strategy_version(source, unix_time_ms())?;
    let version = load_run_strategy_version(
        state,
        source,
        &current,
        &config,
        source_ref.as_deref(),
        unix_time_ms(),
    )?;
    Ok((config, version))
}

fn load_stored_backtest_request(
    state: &DashboardServerState,
    bundle: &VerifiedBacktestBundle,
) -> Result<CreateBacktestRunRequest, ProductError> {
    let expected_ref = format!("artifact://backtests/{}/request.toml", bundle.run.run_id);
    if bundle.config.config_ref != expected_ref {
        return Err(product_error(
            ProductErrorKind::RunNotFound,
            "run_reproduction",
        ));
    }
    let artifact_root = canonical_backtest_artifact_root(state)?;
    let run_root = canonical_path(
        &artifact_root.join(&bundle.run.run_id),
        "reproduction_source_root",
    )?;
    if run_root != artifact_root.join(&bundle.run.run_id) || !run_root.starts_with(&artifact_root) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "reproduction_source_containment",
        ));
    }
    let raw = read_backtest_result_bytes(&run_root.join("request.toml"))?;
    if bundle.config.backtest_config_sha256.as_deref() != Some(sha256_ref(&raw).as_str()) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "reproduction_source_config_sha256",
        ));
    }
    let stored: StoredBacktestConfig = std::str::from_utf8(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "reproduction_source"))
        .and_then(|raw| {
            toml::from_str(raw)
                .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "reproduction_source"))
        })?;
    let expected_venue = bundle
        .summary
        .instrument_id
        .rsplit_once('.')
        .map(|(_, venue)| venue)
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "reproduction_source"))?;
    if stored.run.id != bundle.run.run_id
        || stored.run.mode != "engine-smoke"
        || stored.data.source != "synthetic-quotes"
        || stored.data.instrument_id != bundle.summary.instrument_id
        || stored.data.quotes != bundle.summary.metrics.quotes
        || stored.strategy.name != bundle.summary.strategy
        || stored.strategy.trade_size != bundle.summary.parameters.trade_size
        || stored.strategy.fast_period != bundle.summary.parameters.fast_period
        || stored.strategy.slow_period != bundle.summary.parameters.slow_period
        || stored.venue.name != expected_venue
        || stored.product.strategy_id != bundle.run.strategy_id
        || stored.product.strategy_version_id != bundle.run.strategy_version_id
        || stored.product.strategy_version_content_hash
            != bundle.summary.strategy_version_content_hash
        || stored.product.data_ref != bundle.run.data_ref
        || stored.product.config_ref != bundle.run.config_ref
        || stored.product.result_ref != bundle.run.result.result_ref.as_deref().unwrap_or_default()
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "reproduction_source",
        ));
    }
    Ok(CreateBacktestRunRequest {
        strategy_id: stored.product.strategy_id,
        strategy_version_id: stored.product.strategy_version_id,
        environment: RunEnvironment::Backtest,
        data_ref: stored.product.data_ref,
        venue_ref: bundle.run.venue_ref.clone(),
        starting_balance: stored.venue.starting_balance,
        quotes: stored.data.quotes,
        trade_size: stored.strategy.trade_size,
        fast_period: stored.strategy.fast_period,
        slow_period: stored.strategy.slow_period,
    })
}

fn backtest_reproduction_input_sha256(
    request: &CreateBacktestRunRequest,
    strategy_version_content_hash: &str,
    data_sha256: &str,
    instrument_id: &str,
) -> Result<String, ProductError> {
    let material = BacktestReproductionInput {
        strategy_id: &request.strategy_id,
        strategy_version_id: &request.strategy_version_id,
        strategy_version_content_hash,
        data_ref: &request.data_ref,
        data_sha256,
        venue_ref: &request.venue_ref,
        starting_balance: &request.starting_balance,
        quotes: request.quotes,
        trade_size: &request.trade_size,
        fast_period: request.fast_period,
        slow_period: request.slow_period,
        instrument_id,
    };
    serde_json::to_vec(&material)
        .map(|raw| sha256_ref(&raw))
        .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "reproduction_input"))
}

fn backtest_reproduction_output_sha256(
    run_id: &str,
    summary: &BacktestResultArtifact,
    details: &BacktestDetailsArtifact,
    analysis: &BacktestAnalysisArtifact,
) -> Result<String, ProductError> {
    let timeline = analysis
        .timeline
        .iter()
        .cloned()
        .map(|mut event| {
            if event.entity_ref == format!("run://{run_id}") {
                event.entity_ref = "run://<run_id>".to_string();
            }
            event
        })
        .collect();
    let material = BacktestReproductionOutput {
        strategy_version_content_hash: &summary.strategy_version_content_hash,
        data_ref: &summary.data_ref,
        data_sha256: &summary.data_sha256,
        instrument_id: &summary.instrument_id,
        strategy: &summary.strategy,
        parameters: &summary.parameters,
        backtest_start: &summary.backtest_start,
        backtest_end: &summary.backtest_end,
        metrics: &summary.metrics,
        equity_basis: &details.equity_basis,
        trades: &details.trades,
        positions: &details.positions,
        equity_curve: &details.equity_curve,
        risk: &analysis.risk,
        drawdown_curve: &analysis.drawdown_curve,
        timeline,
        generator: &analysis.provenance.generator,
        engine_mode: &analysis.provenance.engine_mode,
        boundaries: &analysis.boundaries,
    };
    serde_json::to_vec(&material)
        .map(|raw| sha256_ref(&raw))
        .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "reproduction_output"))
}

fn load_backtest_reproduction_proof(
    state: &DashboardServerState,
    run: &ProductRun,
    source_run_id: &str,
) -> Result<BacktestReproductionProof, ProductError> {
    if run.run_id == source_run_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "reproduction_source_run_id",
        ));
    }
    let source = load_product_source(state, unix_time_ms())?;
    let target_bundle = load_verified_backtest_bundle(state, &source, &run.run_id)?;
    let source_bundle = load_verified_backtest_bundle(state, &source, source_run_id)?;
    let proof_sha256 = target_bundle
        .config
        .reproduction_proof_sha256
        .as_deref()
        .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_reproduction"))?;
    let artifact_root = canonical_backtest_artifact_root(state)?;
    let run_root = canonical_path(&artifact_root.join(&run.run_id), "reproduction_result_root")?;
    if run_root != artifact_root.join(&run.run_id) || !run_root.starts_with(&artifact_root) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "reproduction_result_containment",
        ));
    }
    let raw = read_backtest_result_bytes(&run_root.join("reproduction.json"))?;
    if sha256_ref(&raw) != proof_sha256 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "reproduction_proof_sha256",
        ));
    }
    let proof: BacktestReproductionProof = serde_json::from_slice(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "reproduction_proof"))?;
    let source_request = load_stored_backtest_request(state, &source_bundle)?;
    let target_request = load_stored_backtest_request(state, &target_bundle)?;
    let source_input_sha256 = backtest_reproduction_input_sha256(
        &source_request,
        source_bundle.strategy_version.content_hash(),
        &source_bundle.summary.data_sha256,
        &source_bundle.summary.instrument_id,
    )?;
    let reproduced_input_sha256 = backtest_reproduction_input_sha256(
        &target_request,
        target_bundle.strategy_version.content_hash(),
        &target_bundle.summary.data_sha256,
        &target_bundle.summary.instrument_id,
    )?;
    let source_output_sha256 = backtest_reproduction_output_sha256(
        source_run_id,
        &source_bundle.summary,
        &source_bundle.details,
        &source_bundle.analysis,
    )?;
    let reproduced_output_sha256 = backtest_reproduction_output_sha256(
        &run.run_id,
        &target_bundle.summary,
        &target_bundle.details,
        &target_bundle.analysis,
    )?;
    let expected_ref = format!("artifact://backtests/{}/reproduction.json", run.run_id);
    if proof.schema_version != BACKTEST_REPRODUCTION_PROOF_SCHEMA_VERSION
        || proof.source_run_id != source_run_id
        || proof.reproduced_run_id != run.run_id
        || proof.proof_ref != expected_ref
        || proof.source_input_sha256 != source_input_sha256
        || proof.reproduced_input_sha256 != reproduced_input_sha256
        || proof.source_output_sha256 != source_output_sha256
        || proof.reproduced_output_sha256 != reproduced_output_sha256
        || source_input_sha256 != reproduced_input_sha256
        || source_output_sha256 != reproduced_output_sha256
        || !proof.input_equivalent
        || !proof.output_equivalent
        || !proof.user_initiated
        || proof.automatic_retry_allowed
        || proof.automatic_remediation_allowed
        || target_bundle.config.reproduction_source_run_id.as_deref() != Some(source_run_id)
        || target_bundle.config.reproduction_input_sha256.as_deref()
            != Some(reproduced_input_sha256.as_str())
        || target_bundle.config.reproduction_output_sha256.as_deref()
            != Some(reproduced_output_sha256.as_str())
        || run.result.reproduction_ref.as_deref() != Some(expected_ref.as_str())
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "reproduction_proof",
        ));
    }
    Ok(proof)
}

#[allow(clippy::too_many_arguments)]
fn load_backtest_analysis_artifact(
    state: &DashboardServerState,
    run: &ProductRun,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected: &BacktestResultExpectation,
    summary: &BacktestResultArtifact,
    details: &BacktestDetailsArtifact,
    details_sha256: &str,
    expected_sha256: &str,
) -> Result<BacktestAnalysisArtifact, ProductError> {
    if !is_sha256_ref(expected_sha256) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "analysis_sha256",
        ));
    }
    let artifact_root = canonical_backtest_artifact_root(state)?;
    let run_root = canonical_path(&artifact_root.join(&run.run_id), "result_run_root")?;
    if run_root != artifact_root.join(&run.run_id) || !run_root.starts_with(&artifact_root) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "result_containment",
        ));
    }
    let raw = read_backtest_result_bytes(&run_root.join("analysis.json"))?;
    if sha256_ref(&raw) != expected_sha256 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "analysis_sha256",
        ));
    }
    let artifact: BacktestAnalysisArtifact = serde_json::from_slice(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "analysis_artifact"))?;
    validate_backtest_analysis_artifact(
        &artifact,
        run,
        strategy_version,
        expected,
        summary,
        details,
        details_sha256,
    )?;
    Ok(artifact)
}

fn validate_backtest_analysis_artifact(
    artifact: &BacktestAnalysisArtifact,
    run: &ProductRun,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected: &BacktestResultExpectation,
    summary: &BacktestResultArtifact,
    details: &BacktestDetailsArtifact,
    details_sha256: &str,
) -> Result<(), ProductError> {
    let expected_artifact = expected_backtest_analysis_artifact(
        run,
        strategy_version,
        expected,
        summary,
        details,
        details_sha256,
    )?;
    if artifact != &expected_artifact {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "analysis_artifact",
        ));
    }
    Ok(())
}

fn expected_backtest_analysis_artifact(
    run: &ProductRun,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected: &BacktestResultExpectation,
    summary: &BacktestResultArtifact,
    details: &BacktestDetailsArtifact,
    details_sha256: &str,
) -> Result<BacktestAnalysisArtifact, ProductError> {
    let first = details
        .equity_curve
        .first()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "analysis_artifact"))?;
    let starting = first
        .total
        .parse::<Money>()
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "analysis_artifact"))?;
    let mut peak = starting;
    let mut peak_at = first.ts_event.clone();
    let mut max_amount = Decimal::ZERO;
    let mut max_rate = Decimal::ZERO;
    let mut max_started_at = peak_at.clone();
    let mut max_trough_at = peak_at.clone();
    let mut drawdown_curve = Vec::with_capacity(details.equity_curve.len());
    for point in &details.equity_curve {
        let equity = point
            .total
            .parse::<Money>()
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "analysis_artifact"))?;
        if point.currency != first.currency || equity.currency != starting.currency {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "analysis_artifact",
            ));
        }
        if equity > peak {
            peak = equity;
            peak_at.clone_from(&point.ts_event);
        }
        let amount = peak.as_decimal() - equity.as_decimal();
        let rate = if peak.as_decimal() == Decimal::ZERO {
            Decimal::ZERO
        } else {
            amount / peak.as_decimal()
        };
        if rate > max_rate {
            max_amount = amount;
            max_rate = rate;
            max_started_at.clone_from(&peak_at);
            max_trough_at.clone_from(&point.ts_event);
        }
        drawdown_curve.push(BacktestDrawdownPoint {
            ts_event: point.ts_event.clone(),
            equity: point.total.clone(),
            peak_equity: peak.to_string(),
            drawdown_amount: Money::from_decimal(amount, starting.currency)
                .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "analysis_artifact"))?
                .to_string(),
            drawdown_rate: canonical_analysis_decimal(rate),
        });
    }
    let ending = details
        .equity_curve
        .last()
        .and_then(|point| point.total.parse::<Money>().ok())
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "analysis_artifact"))?;
    let current_amount = peak.as_decimal() - ending.as_decimal();
    let current_rate = if peak.as_decimal() == Decimal::ZERO {
        Decimal::ZERO
    } else {
        current_amount / peak.as_decimal()
    };
    let open_positions = details
        .positions
        .iter()
        .filter(|position| position.ts_closed.is_none())
        .count();
    let profitable_positions = details
        .positions
        .iter()
        .filter(|position| {
            position.ts_closed.is_some()
                && parsed_realized_pnl(position).is_some_and(|value| value.raw > 0)
        })
        .count();
    let losing_positions = details
        .positions
        .iter()
        .filter(|position| {
            position.ts_closed.is_some()
                && parsed_realized_pnl(position).is_some_and(|value| value.raw < 0)
        })
        .count();

    Ok(BacktestAnalysisArtifact {
        schema_version: BACKTEST_ANALYSIS_SCHEMA_VERSION.to_string(),
        run_id: run.run_id.clone(),
        strategy_id: run.strategy_id.clone(),
        strategy_version_id: run.strategy_version_id.clone(),
        strategy_version_content_hash: strategy_version.content_hash().to_string(),
        analysis_ref: format!("artifact://backtests/{}/analysis.json", run.run_id),
        instrument_id: summary.instrument_id.clone(),
        risk: BacktestRiskSummary {
            currency: first.currency.clone(),
            starting_equity: starting.to_string(),
            ending_equity: ending.to_string(),
            peak_equity: peak.to_string(),
            max_drawdown_amount: Money::from_decimal(max_amount, starting.currency)
                .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "analysis_artifact"))?
                .to_string(),
            max_drawdown_rate: canonical_analysis_decimal(max_rate),
            max_drawdown_started_at: max_started_at,
            max_drawdown_trough_at: max_trough_at,
            current_drawdown_amount: Money::from_decimal(current_amount, starting.currency)
                .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "analysis_artifact"))?
                .to_string(),
            current_drawdown_rate: canonical_analysis_decimal(current_rate),
            open_positions,
            closed_positions: details.positions.len().saturating_sub(open_positions),
            profitable_positions,
            losing_positions,
        },
        drawdown_curve,
        timeline: expected_backtest_timeline(&run.run_id, summary, details),
        provenance: BacktestAnalysisProvenance {
            generator: "nautilus_backtest::engine::BacktestEngine".to_string(),
            engine_mode: "engine-smoke".to_string(),
            data_ref: run.data_ref.clone(),
            data_sha256: expected.data_sha256.clone(),
            config_ref: run.config_ref.clone(),
            config_sha256: expected.config_sha256.clone(),
            summary_ref: summary.result_ref.clone(),
            summary_sha256: expected.result_sha256.clone(),
            details_ref: details.details_ref.clone(),
            details_sha256: details_sha256.to_string(),
        },
        boundaries: BacktestResultBoundaries {
            read_only: true,
            external_venue_connection: false,
            order_submission_allowed: false,
            order_mutation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            real_orders_submitted: false,
            trading_controls_enabled: false,
        },
    })
}

fn parsed_realized_pnl(position: &BacktestPosition) -> Option<Money> {
    position.realized_pnl.as_deref()?.parse::<Money>().ok()
}

fn expected_backtest_timeline(
    run_id: &str,
    summary: &BacktestResultArtifact,
    details: &BacktestDetailsArtifact,
) -> Vec<BacktestTimelineEvent> {
    let mut events = vec![
        (
            summary.backtest_start.clone(),
            0_u8,
            "run_started".to_string(),
            format!("run://{run_id}"),
        ),
        (
            summary.backtest_end.clone(),
            5_u8,
            "run_completed".to_string(),
            format!("run://{run_id}"),
        ),
    ];
    events.extend(details.equity_curve.iter().map(|point| {
        (
            point.ts_event.clone(),
            1_u8,
            "equity_updated".to_string(),
            format!("account://{}", point.account_id),
        )
    }));
    events.extend(details.trades.iter().map(|trade| {
        (
            trade.ts_event.clone(),
            2_u8,
            "trade_filled".to_string(),
            format!("trade://{}", trade.trade_id),
        )
    }));
    events.extend(details.positions.iter().flat_map(|position| {
        let opened = (
            position.ts_opened.clone(),
            3_u8,
            "position_opened".to_string(),
            format!("position://{}", position.position_id),
        );
        let closed = position.ts_closed.as_ref().map(|timestamp| {
            (
                timestamp.clone(),
                4_u8,
                "position_closed".to_string(),
                format!("position://{}", position.position_id),
            )
        });
        std::iter::once(opened).chain(closed)
    }));
    events.sort_by(|left, right| {
        numeric_timestamp(&left.0)
            .cmp(&numeric_timestamp(&right.0))
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.3.cmp(&right.3))
    });
    events
        .into_iter()
        .enumerate()
        .map(
            |(index, (ts_event, _, event_type, entity_ref))| BacktestTimelineEvent {
                event_id: format!("event-{index:06}"),
                event_type,
                ts_event,
                entity_ref,
            },
        )
        .collect()
}

fn canonical_analysis_decimal(value: Decimal) -> String {
    format!("{value:.12}")
}

fn numeric_timestamp(value: &str) -> Option<u64> {
    value.parse::<u64>().ok()
}

fn load_backtest_details_artifact(
    state: &DashboardServerState,
    run: &ProductRun,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected: &BacktestResultExpectation,
    summary: &BacktestResultArtifact,
    expected_sha256: &str,
) -> Result<BacktestDetailsArtifact, ProductError> {
    if !is_sha256_ref(expected_sha256) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "details_sha256",
        ));
    }
    let artifact_root = canonical_backtest_artifact_root(state)?;
    let run_root = canonical_path(&artifact_root.join(&run.run_id), "result_run_root")?;
    if run_root != artifact_root.join(&run.run_id) || !run_root.starts_with(&artifact_root) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "result_containment",
        ));
    }
    let raw = read_backtest_result_bytes(&run_root.join("details.json"))?;
    if sha256_ref(&raw) != expected_sha256 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "details_sha256",
        ));
    }
    let artifact: BacktestDetailsArtifact = serde_json::from_slice(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "details_artifact"))?;
    validate_backtest_details_artifact(&artifact, run, strategy_version, expected, summary)?;
    Ok(artifact)
}

fn validate_backtest_details_artifact(
    artifact: &BacktestDetailsArtifact,
    run: &ProductRun,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected: &BacktestResultExpectation,
    summary: &BacktestResultArtifact,
) -> Result<(), ProductError> {
    let expected_ref = format!("artifact://backtests/{}/details.json", run.run_id);
    let start = summary.backtest_start.parse::<u64>();
    let end = summary.backtest_end.parse::<u64>();
    let position_ids = artifact
        .positions
        .iter()
        .map(|position| position.position_id.as_str())
        .collect::<BTreeSet<_>>();
    let trade_ids = artifact
        .trades
        .iter()
        .map(|trade| trade.trade_id.as_str())
        .collect::<BTreeSet<_>>();
    let position_trade_counts =
        artifact
            .trades
            .iter()
            .fold(BTreeMap::<&str, usize>::new(), |mut counts, trade| {
                if let Some(position_id) = trade.position_id.as_deref() {
                    *counts.entry(position_id).or_default() += 1;
                }
                counts
            });
    let equity_account_id = artifact
        .equity_curve
        .first()
        .map(|point| point.account_id.as_str());
    let equity_currency = artifact
        .equity_curve
        .first()
        .map(|point| point.currency.as_str());
    let trades_valid = artifact.trades.iter().all(|trade| {
        let timestamp = trade.ts_event.parse::<u64>().ok();
        let commission_valid = trade.commission.as_deref().is_none_or(|value| {
            value
                .parse::<Money>()
                .is_ok_and(|money| money.currency.code.as_str() == trade.currency)
        });
        !trade.trade_id.is_empty()
            && !trade.client_order_id.is_empty()
            && !trade.venue_order_id.is_empty()
            && matches!(trade.side.as_str(), "BUY" | "SELL")
            && !trade.order_type.is_empty()
            && !trade.currency.is_empty()
            && equity_currency.is_some_and(|currency| trade.currency == currency)
            && matches!(trade.liquidity_side.as_str(), "MAKER" | "TAKER")
            && trade.quantity.parse::<Quantity>().is_ok()
            && trade.price.parse::<nautilus_model::types::Price>().is_ok()
            && commission_valid
            && trade
                .position_id
                .as_deref()
                .is_none_or(|value| position_ids.contains(value))
            && timestamp.is_some_and(|value| {
                start.as_ref().is_ok_and(|start| value >= *start)
                    && end.as_ref().is_ok_and(|end| value <= *end)
            })
    });
    let positions_valid = artifact.positions.iter().all(|position| {
        let opened = position.ts_opened.parse::<u64>();
        let closed = position
            .ts_closed
            .as_deref()
            .map(str::parse::<u64>)
            .transpose();
        !position.position_id.is_empty()
            && !position.account_id.is_empty()
            && equity_account_id.is_some_and(|account_id| position.account_id == account_id)
            && matches!(position.side.as_str(), "FLAT" | "LONG" | "SHORT")
            && matches!(position.entry_side.as_str(), "BUY" | "SELL")
            && position.peak_quantity.parse::<Quantity>().is_ok()
            && position.buy_quantity.parse::<Quantity>().is_ok()
            && position.sell_quantity.parse::<Quantity>().is_ok()
            && position
                .avg_price_open
                .parse::<f64>()
                .is_ok_and(f64::is_finite)
            && position
                .avg_price_close
                .as_deref()
                .is_none_or(|value| value.parse::<f64>().is_ok_and(f64::is_finite))
            && position
                .realized_return
                .parse::<f64>()
                .is_ok_and(f64::is_finite)
            && position.realized_pnl.as_deref().is_none_or(|value| {
                value.parse::<Money>().is_ok_and(|money| {
                    equity_currency.is_some_and(|currency| money.currency.code.as_str() == currency)
                })
            })
            && position.trade_count > 0
            && position_trade_counts.get(position.position_id.as_str())
                == Some(&position.trade_count)
            && opened.as_ref().is_ok_and(|opened| {
                start.as_ref().is_ok_and(|start| opened >= start)
                    && end.as_ref().is_ok_and(|end| opened <= end)
            })
            && closed.as_ref().is_ok_and(|value| {
                value.is_none_or(|closed| {
                    opened.as_ref().is_ok_and(|opened| closed >= *opened)
                        && end.as_ref().is_ok_and(|end| closed <= *end)
                })
            })
            && position.duration_ns.parse::<u64>().is_ok_and(|duration| {
                closed.as_ref().is_ok_and(|closed| {
                    closed.is_none_or(|closed| {
                        opened
                            .as_ref()
                            .is_ok_and(|opened| duration == closed.saturating_sub(*opened))
                    })
                })
            })
    });
    let equity_valid = !artifact.equity_curve.is_empty()
        && artifact.equity_curve.iter().all(|point| {
            let Ok(total) = point.total.parse::<Money>() else {
                return false;
            };
            let Ok(free) = point.free.parse::<Money>() else {
                return false;
            };
            let Ok(locked) = point.locked.parse::<Money>() else {
                return false;
            };
            let timestamp = point.ts_event.parse::<u64>();
            !point.account_id.is_empty()
                && !point.currency.is_empty()
                && total.currency.code.as_str() == point.currency
                && free.currency.code.as_str() == point.currency
                && locked.currency.code.as_str() == point.currency
                && free.raw.checked_add(locked.raw) == Some(total.raw)
                && timestamp.as_ref().is_ok_and(|timestamp| {
                    start.as_ref().is_ok_and(|start| timestamp >= start)
                        && end.as_ref().is_ok_and(|end| timestamp <= end)
                })
        });
    let sorted = artifact.trades.windows(2).all(|pair| {
        numeric_string_key(&pair[0].ts_event, &pair[0].trade_id)
            <= numeric_string_key(&pair[1].ts_event, &pair[1].trade_id)
    }) && artifact.positions.windows(2).all(|pair| {
        numeric_string_key(&pair[0].ts_opened, &pair[0].position_id)
            <= numeric_string_key(&pair[1].ts_opened, &pair[1].position_id)
    }) && artifact.equity_curve.windows(2).all(|pair| {
        numeric_string_key(&pair[0].ts_event, (&pair[0].currency, &pair[0].total))
            <= numeric_string_key(&pair[1].ts_event, (&pair[1].currency, &pair[1].total))
    });
    if artifact.schema_version != BACKTEST_DETAILS_SCHEMA_VERSION
        || artifact.run_id != run.run_id
        || artifact.strategy_id != run.strategy_id
        || artifact.strategy_version_id != run.strategy_version_id
        || artifact.strategy_version_content_hash != strategy_version.content_hash()
        || artifact.data_ref != run.data_ref
        || artifact.data_sha256 != expected.data_sha256
        || artifact.config_ref != run.config_ref
        || artifact.config_sha256 != expected.config_sha256
        || artifact.details_ref != expected_ref
        || run.result.report_ref.as_deref() != Some(artifact.details_ref.as_str())
        || artifact.instrument_id != summary.instrument_id
        || artifact.equity_basis != "account_balance_total"
        || trade_ids.len() != artifact.trades.len()
        || position_ids.len() != artifact.positions.len()
        || artifact.equity_curve.first().is_none_or(|first| {
            artifact.equity_curve.iter().any(|point| {
                point.account_id != first.account_id || point.currency != first.currency
            })
        })
        || artifact
            .equity_curve
            .iter()
            .map(|point| {
                (
                    point.account_id.as_str(),
                    point.currency.as_str(),
                    point.ts_event.as_str(),
                    point.total.as_str(),
                    point.free.as_str(),
                    point.locked.as_str(),
                )
            })
            .collect::<BTreeSet<_>>()
            .len()
            != artifact.equity_curve.len()
        || (summary.metrics.total_orders > 0 && artifact.trades.is_empty())
        || artifact.trades.len() > summary.metrics.total_events
        || artifact.positions.len() != summary.metrics.total_positions
        || !artifact.boundaries.read_only
        || artifact.boundaries.external_venue_connection
        || artifact.boundaries.order_submission_allowed
        || artifact.boundaries.order_mutation_allowed
        || artifact.boundaries.automatic_retry_allowed
        || artifact.boundaries.automatic_remediation_allowed
        || artifact.boundaries.real_orders_submitted
        || artifact.boundaries.trading_controls_enabled
        || start.is_err()
        || end.is_err()
        || !trades_valid
        || !positions_valid
        || !equity_valid
        || !sorted
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "details_artifact",
        ));
    }
    Ok(())
}

fn numeric_string_key<T>(timestamp: &str, secondary: T) -> (Option<u64>, T) {
    (timestamp.parse::<u64>().ok(), secondary)
}

fn load_backtest_result_artifact(
    state: &DashboardServerState,
    run: &ProductRun,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected: &BacktestResultExpectation,
) -> Result<BacktestResultArtifact, ProductError> {
    if run.environment != RunEnvironment::Backtest
        || run.lifecycle != RunLifecycle::Completed
        || run.result.status != RunResultStatus::Available
    {
        return Err(product_error(ProductErrorKind::RunNotFound, "run_metrics"));
    }
    let result_ref = run
        .result
        .result_ref
        .as_deref()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "result_ref"))?;
    let expected_ref = format!("artifact://backtests/{}/summary.json", run.run_id);
    if result_ref != expected_ref {
        return Err(product_error(ProductErrorKind::SourceInvalid, "result_ref"));
    }

    let artifact_root = canonical_backtest_artifact_root(state)?;
    let run_root = canonical_path(&artifact_root.join(&run.run_id), "result_run_root")?;
    let expected_run_root = artifact_root.join(&run.run_id);
    if run_root != expected_run_root || !run_root.starts_with(&artifact_root) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "result_containment",
        ));
    }
    let result_path = run_root.join("summary.json");
    let artifact = read_verified_backtest_result(&result_path, &expected.result_sha256)?;
    validate_backtest_result_artifact(&artifact, run, strategy_version, expected)?;
    Ok(artifact)
}

fn load_backtest_result_expectation(
    state: &DashboardServerState,
    source: &ValidatedProductSource,
    run_id: &str,
) -> Result<BacktestResultExpectation, ProductError> {
    let config = load_run_configs(state, source)?
        .into_iter()
        .map(|(config, _)| config)
        .find(|config| config.run_id == run_id)
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "run_expectation"))?;
    backtest_result_expectation(&config)?
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "run_expectation"))
}

fn backtest_result_expectation(
    config: &ProductRunConfig,
) -> Result<Option<BacktestResultExpectation>, ProductError> {
    let fields_present = [
        config.backtest_config_sha256.is_some(),
        config.backtest_data_sha256.is_some(),
        config.backtest_result_sha256.is_some(),
        config.backtest_trade_size.is_some(),
        config.backtest_quotes.is_some(),
        config.backtest_fast_period.is_some(),
        config.backtest_slow_period.is_some(),
    ];
    if fields_present.iter().all(|present| !present) {
        return Ok(None);
    }
    if !fields_present.iter().all(|present| *present) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_expectation",
        ));
    }
    let expected = BacktestResultExpectation {
        config_sha256: config.backtest_config_sha256.clone().unwrap_or_default(),
        data_sha256: config.backtest_data_sha256.clone().unwrap_or_default(),
        result_sha256: config.backtest_result_sha256.clone().unwrap_or_default(),
        trade_size: config.backtest_trade_size.clone().unwrap_or_default(),
        quotes: config.backtest_quotes.unwrap_or_default(),
        fast_period: config.backtest_fast_period.unwrap_or_default(),
        slow_period: config.backtest_slow_period.unwrap_or_default(),
    };
    if !is_sha256_ref(&expected.config_sha256)
        || !is_sha256_ref(&expected.data_sha256)
        || !is_sha256_ref(&expected.result_sha256)
        || expected.trade_size.trim().is_empty()
        || expected.quotes == 0
        || expected.fast_period == 0
        || expected.fast_period >= expected.slow_period
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_expectation",
        ));
    }
    Ok(Some(expected))
}

fn read_verified_backtest_result(
    path: &Path,
    expected_sha256: &str,
) -> Result<BacktestResultArtifact, ProductError> {
    let raw = read_backtest_result_bytes(path)?;
    if sha256_ref(&raw) != expected_sha256 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "result_sha256",
        ));
    }
    serde_json::from_slice(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "result_artifact"))
}

fn read_backtest_result_bytes(path: &Path) -> Result<Vec<u8>, ProductError> {
    use cap_std::fs::OpenOptions;

    let parent = path
        .parent()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "result_artifact_path"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "result_artifact_path"))?;
    let directory = open_absolute_directory_nofollow(parent)?;
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    let file = directory.open_with(file_name, &options).map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            result_io_error(error, "result_artifact")
        } else {
            product_error(ProductErrorKind::SourceInvalid, "result_artifact_type")
        }
    })?;
    let metadata = file
        .metadata()
        .map_err(|error| result_io_error(error, "result_artifact"))?;
    if !metadata.is_file() {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "result_artifact_type",
        ));
    }
    let mut raw = Vec::new();
    file.into_std()
        .read_to_end(&mut raw)
        .map_err(|error| result_io_error(error, "result_artifact"))?;
    Ok(raw)
}

pub(super) fn open_absolute_directory_nofollow(
    path: &Path,
) -> Result<cap_std::fs::Dir, ProductError> {
    use cap_std::fs::Dir;

    if !path.is_absolute() {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "result_root_containment",
        ));
    }
    let (root, components) = absolute_root_and_components(path)?;
    let mut directory = Dir::open_ambient_dir(root, cap_std::ambient_authority())
        .map_err(|error| result_io_error(error, "result_root"))?;
    for name in components {
        directory = open_directory_component_nofollow(&directory, name.as_os_str())?;
    }
    Ok(directory)
}

pub(super) fn open_directory_component_nofollow(
    parent: &cap_std::fs::Dir,
    name: &OsStr,
) -> Result<cap_std::fs::Dir, ProductError> {
    parent
        .open_dir_nofollow(name)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "result_root_containment"))
}

fn absolute_root_and_components(path: &Path) -> Result<(PathBuf, Vec<PathBuf>), ProductError> {
    let invalid = || product_error(ProductErrorKind::SourceInvalid, "result_root_containment");
    let mut source = path.components();
    let mut root = PathBuf::new();
    match source.next() {
        Some(Component::Prefix(prefix)) => {
            root.push(prefix.as_os_str());
            if !matches!(source.next(), Some(Component::RootDir)) {
                return Err(invalid());
            }
            root.push(Path::new(std::path::MAIN_SEPARATOR_STR));
        }
        Some(Component::RootDir) => root.push(Path::new(std::path::MAIN_SEPARATOR_STR)),
        _ => return Err(invalid()),
    }
    let mut components = Vec::new();
    for component in source {
        match component {
            Component::Normal(name) => components.push(PathBuf::from(name)),
            _ => return Err(invalid()),
        }
    }
    Ok((root, components))
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

fn is_sha256_ref(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn result_io_error(_error: IoError, field: &str) -> ProductError {
    product_error(ProductErrorKind::SourceUnavailable, field)
}

fn validate_backtest_result_artifact(
    artifact: &BacktestResultArtifact,
    run: &ProductRun,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected: &BacktestResultExpectation,
) -> Result<(), ProductError> {
    let backtest_start = artifact.backtest_start.parse::<u64>();
    let backtest_end = artifact.backtest_end.parse::<u64>();
    let expected_fast_period = strategy_version.parameter_const_u64("fast_period");
    let expected_slow_period = strategy_version.parameter_const_u64("slow_period");
    if artifact.schema_version != BACKTEST_RESULT_SCHEMA_VERSION
        || artifact.run_id != run.run_id
        || artifact.strategy_id != run.strategy_id
        || artifact.strategy_version_id != run.strategy_version_id
        || artifact.strategy_version_content_hash != strategy_version.content_hash()
        || artifact.data_ref != run.data_ref
        || artifact.data_sha256 != expected.data_sha256
        || artifact.config_ref != run.config_ref
        || artifact.config_sha256 != expected.config_sha256
        || run.result.result_ref.as_deref() != Some(artifact.result_ref.as_str())
        || artifact.instrument_id.trim().is_empty()
        || !strategy_version
            .data_symbols()
            .contains(&artifact.instrument_id)
        || artifact.strategy.trim().is_empty()
        || artifact.parameters.trade_size != expected.trade_size
        || artifact.parameters.fast_period != expected.fast_period
        || artifact.parameters.slow_period != expected.slow_period
        || expected_fast_period != Some(artifact.parameters.fast_period as u64)
        || expected_slow_period != Some(artifact.parameters.slow_period as u64)
        || backtest_start.is_err()
        || backtest_end.is_err()
        || backtest_start.ok() > backtest_end.ok()
        || artifact.metrics.quotes != expected.quotes
        || artifact.metrics.iterations == 0
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "result_artifact",
        ));
    }
    let boundaries = &artifact.boundaries;
    if !boundaries.read_only
        || boundaries.external_venue_connection
        || boundaries.order_submission_allowed
        || boundaries.order_mutation_allowed
        || boundaries.automatic_retry_allowed
        || boundaries.automatic_remediation_allowed
        || boundaries.real_orders_submitted
        || boundaries.trading_controls_enabled
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "result_boundaries",
        ));
    }
    Ok(())
}

pub(super) fn load_product_runs(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<Vec<ProductRun>, ProductError> {
    let _guard = state
        .lifecycle_action_lock
        .lock()
        .map_err(|_| product_error(ProductErrorKind::Conflict, "demo_action_lock"))?;
    // A lifecycle action can hold the lock longer than the allowed clock skew. Validate any
    // contracts published by that action against a timestamp observed after acquiring the lock.
    load_product_runs_unlocked(state, now_unix_ms.max(unix_time_ms()))
}

fn load_product_runs_unlocked(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<Vec<ProductRun>, ProductError> {
    finalize_demo_run_ownerships(state, now_unix_ms)?;
    let source = load_product_source(state, now_unix_ms)?;
    let strategy_version = strategy_version::load_product_strategy_version(&source, now_unix_ms)?;
    let configs = load_run_configs(state, &source)?;
    if configs.is_empty() || configs.len() > MAX_PAGE_LIMIT {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_manifest",
        ));
    }
    let mut run_ids = BTreeSet::new();
    configs
        .into_iter()
        .map(|(config, source_ref)| {
            if !run_ids.insert(config.run_id.clone()) {
                return Err(product_error(ProductErrorKind::SourceInvalid, "run_id"));
            }
            let run_strategy_version = load_run_strategy_version(
                state,
                &source,
                &strategy_version,
                &config,
                source_ref.as_deref(),
                now_unix_ms,
            )?;
            validate_and_project_run(
                config,
                &source,
                &run_strategy_version,
                run_strategy_version.strategy_version_id(),
                now_unix_ms,
                source_ref,
            )
        })
        .collect()
}

fn finalize_demo_run_ownerships(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<(), ProductError> {
    let artifact_root = canonical_demo_artifact_root(state, false)?;
    if !artifact_root.exists() {
        return Ok(());
    }
    let store = SupervisorRegistryStore::new(&state.registry_path);
    let node_ids = store
        .load()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "registry"))?
        .nodes
        .into_keys()
        .collect::<Vec<_>>();
    for node_id in node_ids {
        let record = store
            .refresh_process_state(&node_id)
            .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "registry"))?;
        let Some(ownership) = record
            .run_ownership
            .values()
            .find(|ownership| ownership.terminal.is_none())
        else {
            continue;
        };
        let expected_run_root = artifact_root.join(&ownership.run_id);
        let run_root = canonical_path(&expected_run_root, "demo_root_containment")?;
        if run_root != expected_run_root || !run_root.starts_with(&artifact_root) {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_root_containment",
            ));
        }
        let manifest_path = run_root.join("run-manifest.json");
        let manifest_raw = read_backtest_result_bytes(&manifest_path)?;
        let manifest: DynamicDemoRunManifest = serde_json::from_slice(&manifest_raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_manifest"))?;
        if manifest.schema_version != DEMO_RUN_MANIFEST_SCHEMA_VERSION
            || manifest.config.run_id != ownership.run_id
            || manifest.config.environment != RunEnvironment::Sandbox
            || manifest.config.demo_supervisor_node_id.as_deref() != Some(node_id.as_str())
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_run_ownership",
            ));
        }
        validate_run_config_capabilities(&manifest.config)?;
        if sha256_ref(&manifest_raw) != ownership.manifest_sha256 {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_run_ownership",
            ));
        }
        let terminal_path = run_root.join("terminal-state.json");
        if fs::symlink_metadata(&terminal_path).is_ok() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_terminal_anchor",
            ));
        }
        let mut config = manifest.config;
        project_demo_lifecycle(&mut config, &record, now_unix_ms)?;
        if !is_terminal_demo_lifecycle(config.lifecycle) {
            continue;
        }
        let failure = (config.lifecycle == RunLifecycle::Failed).then_some((
            "demo_runtime_unavailable",
            "Demo runtime 进程已退出且状态不可验证",
        ));
        publish_demo_terminal_state(
            state,
            &store,
            &demo_terminal_identity_from_config(&config)?,
            &record,
            &ownership.manifest_sha256,
            config.lifecycle,
            failure,
        )?;
    }
    Ok(())
}

fn load_run_strategy_version(
    state: &DashboardServerState,
    source: &ValidatedProductSource,
    current: &strategy_version::ProductStrategyVersion,
    config: &ProductRunConfig,
    dynamic_source_ref: Option<&str>,
    now_unix_ms: u64,
) -> Result<strategy_version::ProductStrategyVersion, ProductError> {
    if dynamic_source_ref.is_none() {
        if config.strategy_version_id != current.strategy_version_id() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "run_ownership",
            ));
        }
        if config.strategy_version_snapshot_sha256.is_some() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "run_strategy_version_snapshot",
            ));
        }
        return Ok(current.clone());
    }
    let Some(expected_sha256) = config.strategy_version_snapshot_sha256.as_deref() else {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_strategy_version_snapshot",
        ));
    };
    if !is_sha256_ref(expected_sha256) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_strategy_version_snapshot_sha256",
        ));
    }
    let (artifact_root, source_prefix) =
        if dynamic_source_ref.is_some_and(|value| value.starts_with("artifact://demo-runs/")) {
            (canonical_demo_artifact_root(state, false)?, "demo-runs")
        } else {
            (canonical_backtest_artifact_root(state)?, "backtests")
        };
    let run_root = canonical_path(
        &artifact_root.join(&config.run_id),
        "run_strategy_version_snapshot_root",
    )?;
    if run_root != artifact_root.join(&config.run_id) || !run_root.starts_with(&artifact_root) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_strategy_version_snapshot_containment",
        ));
    }
    let raw = read_backtest_result_bytes(&run_root.join("strategy-version.json"))?;
    if sha256_ref(&raw) != expected_sha256 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_strategy_version_snapshot_sha256",
        ));
    }
    let version = strategy_version::deserialize_strategy_version_snapshot(
        &raw,
        format!(
            "artifact://{source_prefix}/{}/strategy-version.json",
            config.run_id,
        ),
        now_unix_ms,
    )?;
    if version.strategy_id() != source.strategy.strategy_id
        || version.strategy_id() != config.strategy_id
        || version.strategy_version_id() != config.strategy_version_id
        || (version.strategy_version_id() == current.strategy_version_id()
            && version.content_hash() != current.content_hash())
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_strategy_version_snapshot",
        ));
    }
    Ok(version)
}

fn load_run_configs(
    state: &DashboardServerState,
    source: &ValidatedProductSource,
) -> Result<Vec<(ProductRunConfig, Option<String>)>, ProductError> {
    let projection: ProductRunConfigProjection = toml::from_str(&source.raw_config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "run_manifest"))?;
    let mut configs: Vec<_> = projection
        .product_runs
        .into_iter()
        .map(|config| (config, None))
        .collect();
    configs.extend(load_dynamic_run_configs(state)?);
    configs.extend(load_dynamic_demo_run_configs(
        state,
        source,
        unix_time_ms(),
    )?);
    Ok(configs)
}

fn load_dynamic_demo_run_configs(
    state: &DashboardServerState,
    source: &ValidatedProductSource,
    now_unix_ms: u64,
) -> Result<Vec<(ProductRunConfig, Option<String>)>, ProductError> {
    let artifact_root = canonical_demo_artifact_root(state, false)?;
    if !artifact_root.exists() {
        return Ok(Vec::new());
    }
    let store = SupervisorRegistryStore::new(&state.registry_path);
    let record = store
        .refresh_process_state(&source.identity.identities.node_id)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "registry"))?;
    let mut entries = fs::read_dir(&artifact_root)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "demo_root"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "demo_root"))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    let mut configs = Vec::new();
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "demo_root"))?;
        if file_type.is_symlink() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_root_containment",
            ));
        }
        if !file_type.is_dir() {
            continue;
        }
        let run_id = entry
            .file_name()
            .into_string()
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "run_id"))?;
        validate_identifier("run_id", &run_id)?;
        let manifest_path = entry.path().join("run-manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let request_path = entry.path().join("request.json");
        let version_path = entry.path().join("strategy-version.json");
        let manifest_raw = read_backtest_result_bytes(&manifest_path)?;
        let request_raw = read_backtest_result_bytes(&request_path)?;
        let version_raw = read_backtest_result_bytes(&version_path)?;
        let manifest: DynamicDemoRunManifest = serde_json::from_slice(&manifest_raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_manifest"))?;
        if manifest.schema_version != DEMO_RUN_MANIFEST_SCHEMA_VERSION
            || manifest.config.run_id != run_id
            || manifest.config.environment != RunEnvironment::Sandbox
            || manifest.config.config_ref != format!("artifact://demo-runs/{run_id}/request.json")
            || manifest.config.risk_ref
                != format!("artifact://demo-runs/{run_id}/run-manifest.json#risk")
            || !is_sha256_ref(&manifest.request_sha256)
            || sha256_ref(&request_raw) != manifest.request_sha256
            || !is_sha256_ref(&manifest.strategy_version_snapshot_sha256)
            || sha256_ref(&version_raw) != manifest.strategy_version_snapshot_sha256
            || manifest.config.strategy_version_snapshot_sha256.as_deref()
                != Some(manifest.strategy_version_snapshot_sha256.as_str())
            || manifest.config.demo_supervisor_node_id.as_deref()
                != Some(source.identity.identities.node_id.as_str())
            || manifest.config.demo_strategy_instance_id.as_deref()
                != Some(source.identity.identities.strategy_instance_id.as_str())
            || manifest.config.demo_identity_contract_id.as_deref()
                != Some(source.identity.contract_id.as_str())
            || manifest
                .config
                .demo_supervisor_record_baseline_unix_ms
                .is_none()
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_manifest",
            ));
        }
        validate_run_config_capabilities(&manifest.config)?;
        let request: CreateDemoRunRequest = serde_json::from_slice(&request_raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_request"))?;
        let version = strategy_version::deserialize_strategy_version_snapshot(
            &version_raw,
            format!("artifact://demo-runs/{run_id}/strategy-version.json"),
            now_unix_ms,
        )?;
        validate_demo_creation_request(&request, source, &version)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_manifest"))?;
        if request.supervisor_node_id != record.node_id {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "supervisor_node_id",
            ));
        }
        let manifest_sha256 = sha256_ref(&manifest_raw);
        let ownership = record
            .run_ownership
            .get(&run_id)
            .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_run_ownership"))?;
        if ownership.manifest_sha256 != manifest_sha256 {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_run_ownership",
            ));
        }
        let mut config = manifest.config;
        let terminal_path = entry.path().join("terminal-state.json");
        if let Some(anchor) = ownership.terminal.as_ref() {
            let terminal =
                load_demo_terminal_state(&terminal_path, &manifest_raw, &config, anchor)?;
            apply_demo_terminal_state(&mut config, terminal)?;
        } else {
            if fs::symlink_metadata(&terminal_path).is_ok() {
                return Err(product_error(
                    ProductErrorKind::SourceInvalid,
                    "demo_terminal_anchor",
                ));
            }
            project_demo_lifecycle(&mut config, &record, now_unix_ms)?;
            if is_terminal_demo_lifecycle(config.lifecycle) {
                let failure = (config.lifecycle == RunLifecycle::Failed).then_some((
                    "demo_runtime_unavailable",
                    "Demo runtime 进程已退出且状态不可验证",
                ));
                publish_demo_terminal_state(
                    state,
                    &store,
                    &demo_terminal_identity_from_config(&config)?,
                    &record,
                    &manifest_sha256,
                    config.lifecycle,
                    failure,
                )?;
                let anchored_record = store
                    .load()
                    .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "registry"))?;
                let anchor = anchored_record
                    .nodes
                    .get(&record.node_id)
                    .and_then(|node| node.run_ownership.get(&run_id))
                    .and_then(|ownership| ownership.terminal.as_ref())
                    .ok_or_else(|| {
                        product_error(ProductErrorKind::SourceInvalid, "demo_terminal_anchor")
                    })?;
                let terminal =
                    load_demo_terminal_state(&terminal_path, &manifest_raw, &config, anchor)?;
                apply_demo_terminal_state(&mut config, terminal)?;
            }
        }
        configs.push((
            config,
            Some(format!("artifact://demo-runs/{run_id}/run-manifest.json")),
        ));
    }
    Ok(configs)
}

fn load_demo_terminal_state(
    path: &Path,
    manifest_raw: &[u8],
    config: &ProductRunConfig,
    anchor: &SupervisorRunTerminalAnchor,
) -> Result<DynamicDemoRunTerminalState, ProductError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(product_error(
                ProductErrorKind::SourceUnavailable,
                "demo_terminal_state",
            ));
        }
        Err(_) => {
            return Err(product_error(
                ProductErrorKind::SourceUnavailable,
                "demo_terminal_state",
            ));
        }
    }
    let raw = read_backtest_result_bytes(path)?;
    if sha256_ref(&raw) != anchor.terminal_state_sha256 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_terminal_anchor",
        ));
    }
    let terminal: DynamicDemoRunTerminalState = serde_json::from_slice(&raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_terminal_state"))?;
    let result_path = path
        .parent()
        .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "demo_result"))?
        .join("demo-result.json");
    let result_raw = read_backtest_result_bytes(&result_path)?;
    let error_pair_valid = matches!(
        (&terminal.error_code, &terminal.error_summary),
        (None, None) | (Some(_), Some(_))
    );
    if terminal.schema_version != DEMO_RUN_TERMINAL_STATE_SCHEMA_VERSION
        || terminal.source_manifest_sha256 != sha256_ref(manifest_raw)
        || !is_sha256_ref(&terminal.demo_result_sha256)
        || terminal.demo_result_sha256 != sha256_ref(&result_raw)
        || terminal.run_id != config.run_id
        || !is_terminal_demo_lifecycle(terminal.lifecycle)
        || terminal.runtime.supervisor_node_id
            != config
                .demo_supervisor_node_id
                .as_deref()
                .unwrap_or_default()
        || terminal.runtime.strategy_instance_id
            != config
                .demo_strategy_instance_id
                .as_deref()
                .unwrap_or_default()
        || terminal.completed_at_unix_ms < config.created_at_unix_ms
        || terminal.updated_at_unix_ms < terminal.completed_at_unix_ms
        || terminal
            .started_at_unix_ms
            .is_some_and(|started| started > terminal.completed_at_unix_ms)
        || !error_pair_valid
        || (terminal.lifecycle == RunLifecycle::Failed) != terminal.error_code.is_some()
        || anchor.lifecycle
            != match terminal.lifecycle {
                RunLifecycle::Stopped => "stopped",
                RunLifecycle::Failed => "failed",
                _ => "invalid",
            }
        || anchor.completed_at_unix_ms != terminal.completed_at_unix_ms
        || (terminal.lifecycle == RunLifecycle::Stopped
            && (terminal.runtime.process_state != SupervisorProcessState::Stopped
                || terminal.runtime.lifecycle_state != LifecycleStatus::Stopped))
        || (terminal.lifecycle == RunLifecycle::Failed
            && !matches!(
                terminal.runtime.process_state,
                SupervisorProcessState::Stopped | SupervisorProcessState::Stale
            ))
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_terminal_state",
        ));
    }
    Ok(terminal)
}

fn apply_demo_terminal_state(
    config: &mut ProductRunConfig,
    terminal: DynamicDemoRunTerminalState,
) -> Result<(), ProductError> {
    if let Some(code) = terminal.error_code.as_deref() {
        validate_identifier("run_error_code", code)?;
    }
    if let Some(summary) = terminal.error_summary.as_deref() {
        validate_text("run_error_summary", summary, 500)?;
    }
    config.lifecycle = terminal.lifecycle;
    config.demo_process_state = Some(terminal.runtime.process_state);
    config.demo_lifecycle_state = Some(terminal.runtime.lifecycle_state);
    config.started_at_unix_ms = terminal.started_at_unix_ms;
    config.completed_at_unix_ms = Some(terminal.completed_at_unix_ms);
    config.updated_at_unix_ms = terminal.updated_at_unix_ms;
    config.risk_status = RunRiskStatus::Blocked;
    config.error_code = terminal.error_code;
    config.error_summary = terminal.error_summary;
    if config.lifecycle == RunLifecycle::Failed {
        config.result_status = RunResultStatus::Unavailable;
    }
    Ok(())
}

fn project_demo_lifecycle(
    config: &mut ProductRunConfig,
    record: &SupervisorNodeRecord,
    now_unix_ms: u64,
) -> Result<(), ProductError> {
    config.demo_process_state = Some(record.process.state);
    config.demo_lifecycle_state = Some(record.last_known_status.lifecycle_state);
    config.updated_at_unix_ms = now_unix_ms.max(config.created_at_unix_ms);
    let observed_started = snapshot_timestamp(&record.last_known_status.started_at);
    let record_updated_at = snapshot_timestamp(&record.updated_at).ok_or_else(|| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "supervisor_record_updated_at",
        )
    })?;
    let record_baseline = config
        .demo_supervisor_record_baseline_unix_ms
        .ok_or_else(|| {
            product_error(
                ProductErrorKind::SourceInvalid,
                "supervisor_record_baseline",
            )
        })?;
    let started = observed_started.map(|value| value.max(config.created_at_unix_ms));
    let observed_stopped = snapshot_timestamp(&record.last_known_status.stopped_at);
    let stopped =
        observed_stopped.map(|value| value.max(started.unwrap_or(config.created_at_unix_ms)));
    if record_updated_at < record_baseline {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_runtime_updated_at",
        ));
    }
    match (
        record.process.state,
        record.last_known_status.lifecycle_state,
    ) {
        (SupervisorProcessState::NotStarted, LifecycleStatus::Stopped) => {
            config.lifecycle = RunLifecycle::Created;
            config.started_at_unix_ms = None;
            config.completed_at_unix_ms = None;
            config.risk_status = RunRiskStatus::Pending;
        }
        (
            SupervisorProcessState::Running,
            LifecycleStatus::Starting | LifecycleStatus::Resuming,
        ) => {
            config.lifecycle = RunLifecycle::Queued;
            config.started_at_unix_ms = None;
            config.completed_at_unix_ms = None;
            config.risk_status = RunRiskStatus::Pending;
        }
        (SupervisorProcessState::Running, LifecycleStatus::Running) => {
            config.lifecycle = RunLifecycle::Running;
            config.started_at_unix_ms = Some(started.ok_or_else(|| {
                product_error(ProductErrorKind::SourceInvalid, "demo_runtime_started_at")
            })?);
            config.completed_at_unix_ms = None;
            config.risk_status = RunRiskStatus::Active;
        }
        (SupervisorProcessState::Running, LifecycleStatus::Paused | LifecycleStatus::Pausing) => {
            config.lifecycle = RunLifecycle::Paused;
            config.started_at_unix_ms = Some(started.ok_or_else(|| {
                product_error(ProductErrorKind::SourceInvalid, "demo_runtime_started_at")
            })?);
            config.completed_at_unix_ms = None;
            config.risk_status = RunRiskStatus::Active;
        }
        (SupervisorProcessState::Running, LifecycleStatus::Stopping) => {
            config.lifecycle = RunLifecycle::Stopping;
            config.started_at_unix_ms = Some(started.ok_or_else(|| {
                product_error(ProductErrorKind::SourceInvalid, "demo_runtime_started_at")
            })?);
            config.completed_at_unix_ms = None;
            config.risk_status = RunRiskStatus::Active;
        }
        (SupervisorProcessState::Stopped, LifecycleStatus::Stopped) => {
            if observed_started.is_none()
                && observed_stopped.is_none_or(|value| value <= config.created_at_unix_ms)
            {
                config.lifecycle = RunLifecycle::Created;
                config.started_at_unix_ms = None;
                config.completed_at_unix_ms = None;
                config.risk_status = RunRiskStatus::Pending;
                return Ok(());
            }
            let completed = stopped.ok_or_else(|| {
                product_error(ProductErrorKind::SourceInvalid, "demo_runtime_stopped_at")
            })?;
            config.lifecycle = RunLifecycle::Stopped;
            config.started_at_unix_ms = Some(started.unwrap_or(completed));
            config.completed_at_unix_ms = Some(completed);
            config.risk_status = RunRiskStatus::Blocked;
        }
        (SupervisorProcessState::Stale | SupervisorProcessState::Unknown, _)
        | (_, LifecycleStatus::Error | LifecycleStatus::Unknown) => {
            if record_updated_at <= record_baseline {
                return Err(product_error(
                    ProductErrorKind::SourceInvalid,
                    "demo_runtime_updated_at",
                ));
            }
            let completed = stopped.unwrap_or(record_updated_at);
            config.lifecycle = RunLifecycle::Failed;
            config.started_at_unix_ms = Some(started.unwrap_or(completed));
            config.completed_at_unix_ms = Some(completed);
            config.risk_status = RunRiskStatus::Blocked;
            config.result_status = RunResultStatus::Unavailable;
            config.error_code = Some("demo_runtime_unavailable".to_string());
            config.error_summary = Some("Demo runtime 状态不可验证".to_string());
        }
        _ => {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "demo_runtime_state",
            ));
        }
    }
    Ok(())
}

fn snapshot_timestamp(value: &nautilus_live::status::SnapshotValue<String>) -> Option<u64> {
    value
        .value
        .as_deref()
        .and_then(|raw| raw.parse::<u64>().ok())
}

fn create_demo_run_directory(
    state: &DashboardServerState,
    run_id: &str,
) -> Result<cap_std::fs::Dir, ProductError> {
    let artifact_root = canonical_demo_artifact_root(state, true)?;
    let root = open_absolute_directory_nofollow(&artifact_root)?;
    root.create_dir(run_id).map_err(|error| {
        if error.kind() == ErrorKind::AlreadyExists {
            product_error(ProductErrorKind::Conflict, "run_id")
        } else {
            product_error(ProductErrorKind::SourceUnavailable, "demo_root")
        }
    })?;
    root.open_dir_nofollow(run_id)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "demo_root_containment"))
}

fn canonical_demo_artifact_root(
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
    let candidate = workspace.join("artifacts/demo-runs");
    if create && !candidate.exists() {
        let root = open_absolute_directory_nofollow(&artifacts)?;
        match root.create_dir("demo-runs") {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(_) => {
                return Err(product_error(
                    ProductErrorKind::SourceUnavailable,
                    "demo_root",
                ));
            }
        }
    }
    if !candidate.exists() {
        return Ok(candidate);
    }
    let root = canonical_path(&candidate, "demo_root")?;
    if root != canonical_workspace.join("artifacts/demo-runs") {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "demo_root_containment",
        ));
    }
    Ok(root)
}

fn load_dynamic_run_configs(
    state: &DashboardServerState,
) -> Result<Vec<(ProductRunConfig, Option<String>)>, ProductError> {
    let artifact_root = canonical_backtest_artifact_root(state)?;
    let mut entries = fs::read_dir(&artifact_root)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "result_root"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "result_root"))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    let mut configs = Vec::new();
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "result_root"))?;
        if file_type.is_symlink() {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "result_root_containment",
            ));
        }
        if !file_type.is_dir() {
            continue;
        }
        let run_id = entry
            .file_name()
            .into_string()
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "run_id"))?;
        validate_identifier("run_id", &run_id)?;
        let manifest_path = entry.path().join("run-manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let raw = read_backtest_result_bytes(&manifest_path)?;
        let manifest: DynamicBacktestRunManifest = serde_json::from_slice(&raw)
            .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "run_manifest"))?;
        let request_path = entry.path().join("request.toml");
        let request_raw = read_backtest_result_bytes(&request_path)?;
        if manifest.schema_version != BACKTEST_RUN_MANIFEST_SCHEMA_VERSION
            || manifest.config.run_id != run_id
            || !is_sha256_ref(&manifest.request_sha256)
            || sha256_ref(&request_raw) != manifest.request_sha256
            || manifest.config.config_ref != format!("artifact://backtests/{run_id}/request.toml")
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "run_manifest",
            ));
        }
        configs.push((
            manifest.config,
            Some(format!("artifact://backtests/{run_id}/run-manifest.json")),
        ));
    }
    Ok(configs)
}

fn canonical_backtest_artifact_root(state: &DashboardServerState) -> Result<PathBuf, ProductError> {
    let workspace = mvp_workspace_root(&state.registry_path)?;
    let canonical_workspace = canonical_path(&workspace, "workspace")?;
    let artifact_root = canonical_path(&workspace.join("artifacts/backtests"), "result_root")?;
    if artifact_root != canonical_workspace.join("artifacts/backtests") {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "result_root_containment",
        ));
    }
    Ok(artifact_root)
}

fn validate_and_project_run(
    config: ProductRunConfig,
    source: &ValidatedProductSource,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected_version_id: &str,
    now_unix_ms: u64,
    dynamic_source_ref: Option<String>,
) -> Result<ProductRun, ProductError> {
    validate_identifier("run_id", &config.run_id)?;
    validate_identifier("run_strategy_id", &config.strategy_id)?;
    strategy_version::validate_version_resource_id(
        "run_strategy_version_id",
        &config.strategy_version_id,
    )?;
    if config.strategy_id != source.strategy.strategy_id
        || config.strategy_version_id != expected_version_id
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_ownership",
        ));
    }
    validate_run_references(&config, source, strategy_version)?;
    validate_run_lifecycle(&config, strategy_version.created_at_unix_ms(), now_unix_ms)?;
    let capabilities = ProductRunCapabilities {
        external_venue_connection: config.external_venue_connection,
        order_submission_allowed: config.order_submission_allowed,
        order_mutation_allowed: config.order_mutation_allowed,
        automatic_retry_allowed: config.automatic_retry_allowed,
        automatic_remediation_allowed: config.automatic_remediation_allowed,
        real_orders_submitted: config.real_orders_submitted,
        trading_controls_enabled: config.trading_controls_enabled,
    };
    validate_run_capabilities(&capabilities)?;
    let error = match (&config.error_code, &config.error_summary) {
        (None, None) => None,
        (Some(code), Some(summary)) => {
            validate_identifier("run_error_code", code)?;
            validate_text("run_error_summary", summary, 500)?;
            Some(ProductRunError {
                code: code.clone(),
                summary: summary.clone(),
            })
        }
        _ => return Err(product_error(ProductErrorKind::SourceInvalid, "run_error")),
    };
    if (config.lifecycle == RunLifecycle::Failed) != error.is_some() {
        return Err(product_error(ProductErrorKind::SourceInvalid, "run_error"));
    }
    let runtime = match (
        config.demo_supervisor_node_id.as_deref(),
        config.demo_strategy_instance_id.as_deref(),
        config.demo_process_state,
        config.demo_lifecycle_state,
    ) {
        (Some(node_id), Some(instance_id), Some(process_state), Some(lifecycle_state)) => {
            Some(ProductRunRuntime {
                supervisor_node_id: node_id.to_string(),
                strategy_instance_id: instance_id.to_string(),
                process_state,
                lifecycle_state,
            })
        }
        (None, None, None, None) => None,
        _ => {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "run_runtime",
            ));
        }
    };
    Ok(ProductRun {
        run_id: config.run_id.clone(),
        strategy_id: config.strategy_id,
        strategy_version_id: config.strategy_version_id,
        environment: config.environment,
        data_ref: config.data_ref,
        config_ref: config.config_ref,
        adapter_ref: config.adapter_ref,
        account_ref: config.account_ref,
        venue_ref: config.venue_ref,
        lifecycle: config.lifecycle,
        result: ProductRunResult {
            status: config.result_status,
            result_ref: config.result_ref,
            report_ref: config
                .backtest_details_sha256
                .as_ref()
                .map(|_| format!("artifact://backtests/{}/details.json", config.run_id)),
            analysis_ref: config
                .backtest_analysis_sha256
                .as_ref()
                .map(|_| format!("artifact://backtests/{}/analysis.json", config.run_id)),
            reproduction_ref: config
                .reproduction_proof_sha256
                .as_ref()
                .map(|_| format!("artifact://backtests/{}/reproduction.json", config.run_id)),
        },
        risk: ProductRunRisk {
            status: config.risk_status,
            risk_ref: config.risk_ref,
        },
        error,
        created_at_unix_ms: config.created_at_unix_ms,
        started_at_unix_ms: config.started_at_unix_ms,
        completed_at_unix_ms: config.completed_at_unix_ms,
        updated_at_unix_ms: config.updated_at_unix_ms,
        source: ProductSource {
            source_type: "run_manifest".to_string(),
            freshness_status: "fresh".to_string(),
            source_refs: dynamic_source_ref.map_or_else(
                || {
                    vec![
                        MVP_IDENTITY_CONTRACT_PATH.to_string(),
                        MVP_STATUS_CONTRACT_PATH.to_string(),
                        format!(
                            "node-config:{}#product_runs:{}",
                            source.config_name, config.run_id
                        ),
                    ]
                },
                |value| {
                    vec![
                        MVP_IDENTITY_CONTRACT_PATH.to_string(),
                        MVP_STATUS_CONTRACT_PATH.to_string(),
                        value,
                    ]
                },
            ),
        },
        capabilities,
        runtime,
    })
}

fn validate_run_references(
    config: &ProductRunConfig,
    source: &ValidatedProductSource,
    strategy_version: &strategy_version::ProductStrategyVersion,
) -> Result<(), ProductError> {
    let reproduction_values = [
        config.reproduction_source_run_id.as_deref(),
        config.reproduction_input_sha256.as_deref(),
        config.reproduction_output_sha256.as_deref(),
        config.reproduction_proof_sha256.as_deref(),
    ];
    let reproduction_count = reproduction_values
        .iter()
        .filter(|value| value.is_some())
        .count();
    if reproduction_count != 0 && reproduction_count != reproduction_values.len() {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_reproduction_expectation",
        ));
    }
    if reproduction_count == reproduction_values.len() {
        let source_run_id = config
            .reproduction_source_run_id
            .as_deref()
            .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "run_reproduction"))?;
        validate_identifier("reproduction_source_run_id", source_run_id)?;
        if source_run_id == config.run_id
            || config.environment != RunEnvironment::Backtest
            || config.lifecycle != RunLifecycle::Completed
            || config.result_status != RunResultStatus::Available
            || !is_sha256_ref(
                config
                    .reproduction_input_sha256
                    .as_deref()
                    .unwrap_or_default(),
            )
            || !is_sha256_ref(
                config
                    .reproduction_output_sha256
                    .as_deref()
                    .unwrap_or_default(),
            )
            || !is_sha256_ref(
                config
                    .reproduction_proof_sha256
                    .as_deref()
                    .unwrap_or_default(),
            )
        {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "run_reproduction_expectation",
            ));
        }
    }
    let backtest_expectation = backtest_result_expectation(config)?;
    let expectation_required = config.environment == RunEnvironment::Backtest
        && config.lifecycle == RunLifecycle::Completed;
    if expectation_required != backtest_expectation.is_some() {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_expectation_environment",
        ));
    }
    if config
        .backtest_details_sha256
        .as_deref()
        .is_some_and(|value| {
            !is_sha256_ref(value)
                || config.environment != RunEnvironment::Backtest
                || config.lifecycle != RunLifecycle::Completed
                || config.result_status != RunResultStatus::Available
        })
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_details_expectation",
        ));
    }
    if config
        .backtest_analysis_sha256
        .as_deref()
        .is_some_and(|value| {
            !is_sha256_ref(value)
                || config.backtest_details_sha256.is_none()
                || config.environment != RunEnvironment::Backtest
                || config.lifecycle != RunLifecycle::Completed
                || config.result_status != RunResultStatus::Available
        })
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_analysis_expectation",
        ));
    }
    for (field, value) in [
        ("run_data_ref", config.data_ref.as_str()),
        ("run_config_ref", config.config_ref.as_str()),
        ("run_adapter_ref", config.adapter_ref.as_str()),
        ("run_account_ref", config.account_ref.as_str()),
        ("run_venue_ref", config.venue_ref.as_str()),
        ("run_risk_ref", config.risk_ref.as_str()),
    ] {
        validate_reference(field, value)?;
    }
    if let Some(result_ref) = config.result_ref.as_deref() {
        validate_reference("run_result_ref", result_ref)?;
        if !result_ref.starts_with("artifact://") {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "run_result_ref",
            ));
        }
    }
    let dynamic_config_ref = format!("artifact://backtests/{}/request.toml", config.run_id);
    let dynamic_risk_ref = format!(
        "artifact://backtests/{}/run-manifest.json#risk",
        config.run_id
    );
    let is_dynamic_backtest = config.environment == RunEnvironment::Backtest
        && config.config_ref == dynamic_config_ref
        && config.risk_ref == dynamic_risk_ref;
    let dynamic_demo_config_ref = format!("artifact://demo-runs/{}/request.json", config.run_id);
    let dynamic_demo_risk_ref = format!(
        "artifact://demo-runs/{}/run-manifest.json#risk",
        config.run_id
    );
    let is_dynamic_demo = config.environment == RunEnvironment::Sandbox
        && config.config_ref == dynamic_demo_config_ref
        && config.risk_ref == dynamic_demo_risk_ref;
    let is_static_config = config.config_ref
        == format!("node-config:{}#product_runs", source.config_name)
        && config.risk_ref == format!("node-config:{}#risk", source.config_name);
    if !is_dynamic_backtest && !is_dynamic_demo && !is_static_config {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_config_reference",
        ));
    }
    let expected_backtest_data_ref = format!(
        "dataset://fixtures/{}",
        source.strategy.strategy_id.replace('_', "-")
    );
    let expected_dynamic_result_ref =
        format!("artifact://backtests/{}/summary.json", config.run_id);
    let valid = match config.environment {
        RunEnvironment::Backtest => {
            let result_reference_valid = if is_dynamic_backtest {
                match config.lifecycle {
                    RunLifecycle::Completed => {
                        config.result_ref.as_deref() == Some(expected_dynamic_result_ref.as_str())
                    }
                    RunLifecycle::Failed => config.result_ref.is_none(),
                    _ => false,
                }
            } else {
                config.run_id == source.identity.identities.backtest_run_id
                    && config.result_ref.as_deref()
                        == Some(source.identity.identities.backtest_result_ref.as_str())
            };
            result_reference_valid
                && config.data_ref == expected_backtest_data_ref
                && config.adapter_ref == "adapter://backtest/simulated"
                && config.account_ref == format!("account://simulated/{}", config.run_id)
                && strategy_version
                    .data_venues()
                    .iter()
                    .any(|venue| config.venue_ref == format!("venue://simulated/{venue}"))
        }
        RunEnvironment::Sandbox => {
            let identity_valid = if is_dynamic_demo {
                config.demo_supervisor_node_id.as_deref()
                    == Some(source.identity.identities.node_id.as_str())
                    && config.demo_strategy_instance_id.as_deref()
                        == Some(source.identity.identities.strategy_instance_id.as_str())
                    && config.demo_identity_contract_id.as_deref()
                        == Some(source.identity.contract_id.as_str())
                    && config.demo_supervisor_record_baseline_unix_ms.is_some()
                    && config.strategy_version_snapshot_sha256.is_some()
                    && config.demo_process_state.is_some()
                    && config.demo_lifecycle_state.is_some()
            } else {
                config.run_id == source.identity.identities.strategy_instance_id
                    && config.demo_supervisor_node_id.is_none()
                    && config.demo_strategy_instance_id.is_none()
                    && config.demo_identity_contract_id.is_none()
                    && config.demo_supervisor_record_baseline_unix_ms.is_none()
                    && config.demo_process_state.is_none()
                    && config.demo_lifecycle_state.is_none()
            };
            identity_valid
                && config.account_ref
                    == format!(
                        "account://sandbox/{}",
                        source.identity.identities.account_id
                    )
                && config.venue_ref
                    == format!("venue://sandbox/{}", source.identity.identities.venue_id)
                && strategy_version
                    .data_symbols()
                    .iter()
                    .any(|symbol| config.data_ref == format!("market://sandbox/{symbol}"))
                && config.adapter_ref == "adapter://sandbox/fixture-stream"
        }
        RunEnvironment::Live => {
            config.demo_supervisor_node_id.is_none()
                && config.demo_strategy_instance_id.is_none()
                && config.demo_identity_contract_id.is_none()
                && config.demo_supervisor_record_baseline_unix_ms.is_none()
                && config.demo_process_state.is_none()
                && config.demo_lifecycle_state.is_none()
                && config.data_ref == "market://live/disabled"
                && config.adapter_ref == "adapter://live/disabled"
                && config.account_ref == "account://live/unconfigured"
                && config.venue_ref == "venue://live/unconfigured/disabled"
        }
    };
    if !valid {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "run_environment_references",
        ));
    }
    Ok(())
}

fn validate_reference(field: &str, value: &str) -> Result<(), ProductError> {
    validate_text(field, value, 512)?;
    let lowered = value.to_ascii_lowercase();
    if value.bytes().any(|byte| byte.is_ascii_whitespace())
        || !value.contains(':')
        || ["secret", "credential", "api_key", "access_token"]
            .iter()
            .any(|marker| lowered.contains(marker))
    {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    Ok(())
}

fn validate_run_lifecycle(
    config: &ProductRunConfig,
    strategy_version_created_at_unix_ms: u64,
    now: u64,
) -> Result<(), ProductError> {
    if config.created_at_unix_ms == 0
        || config.created_at_unix_ms < strategy_version_created_at_unix_ms
        || config.updated_at_unix_ms < config.created_at_unix_ms
        || config.updated_at_unix_ms > now.saturating_add(MAX_CLOCK_SKEW_MS)
        || config.started_at_unix_ms.is_some_and(|value| {
            value < config.created_at_unix_ms || value > config.updated_at_unix_ms
        })
        || config.completed_at_unix_ms.is_some_and(|value| {
            value
                < config
                    .started_at_unix_ms
                    .unwrap_or(config.created_at_unix_ms)
                || value > config.updated_at_unix_ms
        })
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_timestamps",
        ));
    }
    let valid = match config.lifecycle {
        RunLifecycle::Created | RunLifecycle::Queued => {
            config.started_at_unix_ms.is_none()
                && config.completed_at_unix_ms.is_none()
                && config.result_status == RunResultStatus::Pending
                && config.result_ref.is_none()
        }
        RunLifecycle::Running | RunLifecycle::Stopping | RunLifecycle::Paused => {
            config.started_at_unix_ms.is_some()
                && config.completed_at_unix_ms.is_none()
                && config.result_status == RunResultStatus::Pending
                && config.result_ref.is_none()
        }
        RunLifecycle::Completed => {
            config.started_at_unix_ms.is_some()
                && config.completed_at_unix_ms.is_some()
                && config.result_status == RunResultStatus::Available
                && config.result_ref.is_some()
        }
        RunLifecycle::Failed => {
            config.started_at_unix_ms.is_some()
                && config.completed_at_unix_ms.is_some()
                && config.result_status == RunResultStatus::Unavailable
                && config.result_ref.is_none()
        }
        RunLifecycle::Cancelled | RunLifecycle::Stopped => {
            config.started_at_unix_ms.is_some()
                && config.completed_at_unix_ms.is_some()
                && config.result_status != RunResultStatus::Available
                && config.result_ref.is_none()
        }
    };
    if !valid {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_lifecycle",
        ));
    }
    if config.environment == RunEnvironment::Live
        && (config.lifecycle != RunLifecycle::Created
            || config.result_status != RunResultStatus::Pending
            || config.risk_status != RunRiskStatus::Blocked)
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_run_state",
        ));
    }
    let expected_risk_status = match config.lifecycle {
        RunLifecycle::Created | RunLifecycle::Queued => RunRiskStatus::Pending,
        RunLifecycle::Running | RunLifecycle::Stopping | RunLifecycle::Paused => {
            RunRiskStatus::Active
        }
        RunLifecycle::Completed => RunRiskStatus::Passed,
        RunLifecycle::Failed | RunLifecycle::Cancelled | RunLifecycle::Stopped => {
            RunRiskStatus::Blocked
        }
    };
    if config.environment != RunEnvironment::Live && config.risk_status != expected_risk_status {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_risk_status",
        ));
    }
    Ok(())
}

fn validate_run_capabilities(value: &ProductRunCapabilities) -> Result<(), ProductError> {
    if value.external_venue_connection
        || value.order_submission_allowed
        || value.order_mutation_allowed
        || value.automatic_retry_allowed
        || value.automatic_remediation_allowed
        || value.real_orders_submitted
        || value.trading_controls_enabled
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "run_capabilities",
        ));
    }
    Ok(())
}

fn validate_run_config_capabilities(config: &ProductRunConfig) -> Result<(), ProductError> {
    validate_run_capabilities(&ProductRunCapabilities {
        external_venue_connection: config.external_venue_connection,
        order_submission_allowed: config.order_submission_allowed,
        order_mutation_allowed: config.order_mutation_allowed,
        automatic_retry_allowed: config.automatic_retry_allowed,
        automatic_remediation_allowed: config.automatic_remediation_allowed,
        real_orders_submitted: config.real_orders_submitted,
        trading_controls_enabled: config.trading_controls_enabled,
    })
}

pub(super) fn project_run_list(
    mut runs: Vec<ProductRun>,
    query: &RunListQuery,
    request_id: String,
) -> Result<RunListResponse, ProductError> {
    runs.retain(|run| {
        query
            .strategy_id
            .as_deref()
            .is_none_or(|value| run.strategy_id == value)
            && query
                .strategy_version_id
                .as_deref()
                .is_none_or(|value| run.strategy_version_id == value)
            && query
                .environment
                .is_none_or(|value| run.environment == value)
            && query.lifecycle.is_none_or(|value| run.lifecycle == value)
    });
    runs.sort_by(|left, right| run_comparison(left, right, query));
    let start = if let Some(cursor) = query.cursor.as_deref() {
        let cursor_id = decode_run_cursor(cursor)?;
        runs.iter()
            .position(|run| run.run_id == cursor_id)
            .map(|position| position + 1)
            .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "cursor"))?
    } else {
        0
    };
    let end = start.saturating_add(query.limit).min(runs.len());
    let data = runs[start..end].to_vec();
    let has_more = end < runs.len();
    let next_cursor = has_more
        .then(|| data.last().map(|run| encode_run_cursor(&run.run_id)))
        .flatten();
    Ok(RunListResponse {
        schema_version: RUN_LIST_SCHEMA_VERSION.to_string(),
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

fn run_comparison(left: &ProductRun, right: &ProductRun, query: &RunListQuery) -> Ordering {
    let value = match query.sort {
        RunSort::RunId => left.run_id.cmp(&right.run_id),
        RunSort::CreatedAt => left.created_at_unix_ms.cmp(&right.created_at_unix_ms),
        RunSort::UpdatedAt => left.updated_at_unix_ms.cmp(&right.updated_at_unix_ms),
    }
    .then_with(|| left.run_id.cmp(&right.run_id));
    match query.order {
        SortOrder::Asc => value,
        SortOrder::Desc => value.reverse(),
    }
}

pub(super) fn parse_run_list_query(raw: Option<&str>) -> Result<RunListQuery, ProductError> {
    let values = parse_query_values(raw)?;
    for key in values.keys() {
        if !matches!(
            key.as_str(),
            "limit"
                | "cursor"
                | "sort"
                | "order"
                | "strategy_id"
                | "strategy_version_id"
                | "environment"
                | "lifecycle"
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
        decode_run_cursor(cursor)?;
    }
    let sort = match values.get("sort").map(String::as_str) {
        None | Some("run_id") => RunSort::RunId,
        Some("created_at") => RunSort::CreatedAt,
        Some("updated_at") => RunSort::UpdatedAt,
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "sort")),
    };
    let order = match values.get("order").map(String::as_str) {
        None | Some("asc") => SortOrder::Asc,
        Some("desc") => SortOrder::Desc,
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "order")),
    };
    let strategy_id = optional_identifier(&values, "strategy_id")?;
    let strategy_version_id = values
        .get("strategy_version_id")
        .cloned()
        .map(|value| {
            strategy_version::validate_requested_version_id("strategy_version_id", &value)
                .map(|()| value)
        })
        .transpose()?;
    let environment = match values.get("environment").map(String::as_str) {
        None => None,
        Some("backtest") => Some(RunEnvironment::Backtest),
        Some("sandbox") => Some(RunEnvironment::Sandbox),
        Some("live") => Some(RunEnvironment::Live),
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "environment")),
    };
    let lifecycle = match values.get("lifecycle").map(String::as_str) {
        None => None,
        Some("created") => Some(RunLifecycle::Created),
        Some("queued") => Some(RunLifecycle::Queued),
        Some("running") => Some(RunLifecycle::Running),
        Some("stopping") => Some(RunLifecycle::Stopping),
        Some("completed") => Some(RunLifecycle::Completed),
        Some("failed") => Some(RunLifecycle::Failed),
        Some("cancelled") => Some(RunLifecycle::Cancelled),
        Some("stopped") => Some(RunLifecycle::Stopped),
        Some("paused") => Some(RunLifecycle::Paused),
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "lifecycle")),
    };
    Ok(RunListQuery {
        limit,
        cursor,
        sort,
        order,
        strategy_id,
        strategy_version_id,
        environment,
        lifecycle,
    })
}

fn optional_identifier(
    values: &BTreeMap<String, String>,
    field: &str,
) -> Result<Option<String>, ProductError> {
    values
        .get(field)
        .cloned()
        .map(|value| {
            validate_identifier(field, &value)
                .map_err(|_| product_error(ProductErrorKind::BadRequest, field))
                .map(|()| value)
        })
        .transpose()
}

pub(super) fn encode_run_cursor(run_id: &str) -> String {
    let mut encoded = String::with_capacity(RUN_CURSOR_PREFIX.len() + run_id.len() * 2);
    encoded.push_str(RUN_CURSOR_PREFIX);
    for byte in run_id.bytes() {
        use std::fmt::Write as _;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn decode_run_cursor(cursor: &str) -> Result<String, ProductError> {
    let encoded = cursor
        .strip_prefix(RUN_CURSOR_PREFIX)
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
    let run_id = String::from_utf8(bytes)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, "cursor"))?;
    validate_requested_run_id("cursor", &run_id)?;
    Ok(run_id)
}

fn validate_requested_run_id(field: &str, value: &str) -> Result<(), ProductError> {
    validate_identifier(field, value)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, field))
}
