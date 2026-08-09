//! Backtest、Sandbox 与 Live 三环境 Run 的只读产品合同。

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    io::{Error as IoError, ErrorKind, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering as AtomicOrdering},
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
use nautilus_model::types::{Money, Quantity};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::dashboard::ApiStatusResult;

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
pub(in crate::dashboard) struct RunCreateResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: ProductRun,
    boundaries: BacktestRunCreationBoundaries,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DynamicBacktestRunManifest {
    schema_version: String,
    request_sha256: String,
    config: ProductRunConfig,
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
        create_backtest_run(&worker_state, request, &worker_request_id)
    })
    .await
    .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "backtest_worker"))
    .and_then(|result| result);

    result
        .map(|data| {
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

fn create_backtest_run(
    state: &DashboardServerState,
    request: CreateBacktestRunRequest,
    request_id: &str,
) -> Result<ProductRun, ProductError> {
    let created_at = unix_time_ms();
    let source = load_product_source(state, created_at)?;
    let strategy_version = strategy_version::load_product_strategy_version(&source, created_at)?;
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
    let run_directory = create_dynamic_run_directory(state, &run_id)?;
    write_new_run_file(&run_directory, "request.toml", &config_raw)?;

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
            };
            let expected_version_id = strategy_version_resource_id(
                &source.strategy.strategy_id,
                &source.identity.identities.strategy_version,
            );
            validate_created_backtest_artifacts(
                &config,
                &source,
                &strategy_version,
                &expected_version_id,
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
            };
            write_dynamic_manifest(&run_directory, &request_sha256, &config)?;
            return Err(product_error(
                ProductErrorKind::ExecutionFailed,
                "backtest_engine",
            ));
        }
    };
    write_dynamic_manifest(&run_directory, &request_sha256, &config)?;
    let expected_version_id = strategy_version_resource_id(
        &source.strategy.strategy_id,
        &source.identity.identities.strategy_version,
    );
    validate_and_project_run(
        config,
        &source,
        &strategy_version,
        &expected_version_id,
        completed_at,
        Some(format!("artifact://backtests/{run_id}/run-manifest.json")),
    )
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
    let expected_version_id = strategy_version_resource_id(
        &source.strategy.strategy_id,
        &source.identity.identities.strategy_version,
    );
    if request.strategy_id != source.strategy.strategy_id
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
        let strategy_version =
            strategy_version::load_product_strategy_version(&source, unix_time_ms())?;
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
        let strategy_version =
            strategy_version::load_product_strategy_version(&source, unix_time_ms())?;
        let run = load_product_runs(&state, unix_time_ms())?
            .into_iter()
            .find(|run| run.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
        let config = load_run_configs(&state, &source)?
            .into_iter()
            .map(|(config, _)| config)
            .find(|config| config.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
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
        let strategy_version =
            strategy_version::load_product_strategy_version(&source, unix_time_ms())?;
        let run = load_product_runs(&state, unix_time_ms())?
            .into_iter()
            .find(|run| run.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
        let config = load_run_configs(&state, &source)?
            .into_iter()
            .map(|(config, _)| config)
            .find(|config| config.run_id == run_id)
            .ok_or_else(|| product_error(ProductErrorKind::RunNotFound, "run_id"))?;
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
    let source = load_product_source(state, now_unix_ms)?;
    let strategy_version = strategy_version::load_product_strategy_version(&source, now_unix_ms)?;
    let configs = load_run_configs(state, &source)?;
    if configs.is_empty() || configs.len() > MAX_PAGE_LIMIT {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_manifest",
        ));
    }
    let expected_version_id = strategy_version_resource_id(
        &source.strategy.strategy_id,
        &source.identity.identities.strategy_version,
    );
    let mut run_ids = BTreeSet::new();
    configs
        .into_iter()
        .map(|(config, source_ref)| {
            if !run_ids.insert(config.run_id.clone()) {
                return Err(product_error(ProductErrorKind::SourceInvalid, "run_id"));
            }
            validate_and_project_run(
                config,
                &source,
                &strategy_version,
                &expected_version_id,
                now_unix_ms,
                source_ref,
            )
        })
        .collect()
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
    Ok(configs)
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
    })
}

fn validate_run_references(
    config: &ProductRunConfig,
    source: &ValidatedProductSource,
    strategy_version: &strategy_version::ProductStrategyVersion,
) -> Result<(), ProductError> {
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
    let is_static_config = config.config_ref
        == format!("node-config:{}#product_runs", source.config_name)
        && config.risk_ref == format!("node-config:{}#risk", source.config_name);
    if !is_dynamic_backtest && !is_static_config {
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
            config.run_id == source.identity.identities.strategy_instance_id
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
            config.data_ref == "market://live/disabled"
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
        RunLifecycle::Running | RunLifecycle::Stopping => {
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
        RunLifecycle::Running | RunLifecycle::Stopping => RunRiskStatus::Active,
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
