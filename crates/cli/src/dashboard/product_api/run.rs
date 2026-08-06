//! Backtest、Sandbox 与 Live 三环境 Run 的只读产品合同。

use std::{cmp::Ordering, collections::BTreeSet};

use axum::{
    Json,
    extract::{Path as AxumPath, RawQuery, State, rejection::PathRejection},
};
use serde::{Deserialize, Serialize};

use super::*;

const RUN_LIST_SCHEMA_VERSION: &str = "ntpro.product_api.run_list.response.v1";
const RUN_DETAIL_SCHEMA_VERSION: &str = "ntpro.product_api.run_detail.response.v1";
const RUN_CURSOR_PREFIX: &str = "run-v1-";

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

#[derive(Clone, Debug, Deserialize)]
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

pub(super) fn load_product_runs(
    state: &DashboardServerState,
    now_unix_ms: u64,
) -> Result<Vec<ProductRun>, ProductError> {
    let source = load_product_source(state, now_unix_ms)?;
    let strategy_version = strategy_version::load_product_strategy_version(&source, now_unix_ms)?;
    let projection: ProductRunConfigProjection = toml::from_str(&source.raw_config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "run_manifest"))?;
    if projection.product_runs.is_empty() || projection.product_runs.len() > MAX_PAGE_LIMIT {
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
    projection
        .product_runs
        .into_iter()
        .map(|config| {
            if !run_ids.insert(config.run_id.clone()) {
                return Err(product_error(ProductErrorKind::SourceInvalid, "run_id"));
            }
            validate_and_project_run(
                config,
                &source,
                &strategy_version,
                &expected_version_id,
                now_unix_ms,
            )
        })
        .collect()
}

fn validate_and_project_run(
    config: ProductRunConfig,
    source: &ValidatedProductSource,
    strategy_version: &strategy_version::ProductStrategyVersion,
    expected_version_id: &str,
    now_unix_ms: u64,
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
            source_refs: vec![
                MVP_IDENTITY_CONTRACT_PATH.to_string(),
                MVP_STATUS_CONTRACT_PATH.to_string(),
                format!(
                    "node-config:{}#product_runs:{}",
                    source.config_name, config.run_id
                ),
            ],
        },
        capabilities,
    })
}

fn validate_run_references(
    config: &ProductRunConfig,
    source: &ValidatedProductSource,
    strategy_version: &strategy_version::ProductStrategyVersion,
) -> Result<(), ProductError> {
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
    if config.config_ref != format!("node-config:{}#product_runs", source.config_name)
        || config.risk_ref != format!("node-config:{}#risk", source.config_name)
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "run_config_reference",
        ));
    }
    let expected_backtest_data_ref = format!(
        "dataset://fixtures/{}",
        source.strategy.strategy_id.replace('_', "-")
    );
    let valid = match config.environment {
        RunEnvironment::Backtest => {
            config.run_id == source.identity.identities.backtest_run_id
                && config.result_ref.as_deref()
                    == Some(source.identity.identities.backtest_result_ref.as_str())
                && config.data_ref == expected_backtest_data_ref
                && config.adapter_ref == "adapter://backtest/simulated"
                && config.account_ref
                    == format!(
                        "account://simulated/{}",
                        source.identity.identities.backtest_run_id
                    )
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
