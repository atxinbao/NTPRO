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

//! Live 独立准入的只读产品合同。

use aws_lc_rs::digest::{SHA256, digest};
use axum::{
    Json,
    extract::{
        Path as AxumPath, RawQuery, State,
        rejection::{JsonRejection, PathRejection},
    },
};
use serde::{Deserialize, Serialize};

use super::{
    DashboardServerState, PRODUCT_API_CONTRACT_VERSION, ProductError, ProductErrorKind,
    ValidatedProductSource, load_product_source, product_error, product_error_response,
    product_request_id, reject_detail_query, strategy_version, unix_time_ms, validate_identifier,
};
use crate::dashboard::ApiResult;
use crate::live::{
    PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW, PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE,
    PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION,
    PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE,
    PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED, ProductLiveAccountReadObservation,
    ProductLiveAssetBalance, execute_product_live_account_read,
};

const LIVE_ADMISSION_CONFIG_SCHEMA_VERSION: &str = "ntpro.live_admission.config.v1";
const LIVE_ADMISSION_RESPONSE_SCHEMA_VERSION: &str = "ntpro.product_api.live_admission.response.v1";
const LIVE_ACCOUNT_REFRESH_RESPONSE_SCHEMA_VERSION: &str =
    "ntpro.product_api.live_account_refresh.response.v2";
const BINANCE_PRODUCTION_HTTP_BASE_URL: &str = "https://api.binance.com";
const BINANCE_PRODUCTION_WEBSOCKET_BASE_URL: &str = "wss://stream.binance.com:9443/ws";
const BINANCE_MARKET_DATA_ADAPTER_REF: &str = "adapter://binance/spot/production-market-data";
const BINANCE_EXECUTION_ADAPTER_REF: &str = "adapter://binance/spot/production-execution";
const BINANCE_LIVE_ACCOUNT_REF: &str = "account://live/binance/primary";
const BINANCE_LIVE_API_KEY_ENV: &str = "NTPRO_BINANCE_LIVE_API_KEY";
const BINANCE_LIVE_API_SECRET_ENV: &str = "NTPRO_BINANCE_LIVE_API_SECRET";

#[derive(Debug, Deserialize)]
pub(in crate::dashboard) struct LiveAdmissionPath {
    strategy_id: String,
    version_id: String,
}

#[derive(Debug, Deserialize)]
struct LiveAdmissionConfigDocument {
    live_admission: LiveAdmissionConfig,
    risk: LiveRiskConfig,
    live_sizing: LiveSizingConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveRiskConfig {
    kill_switch_enabled: bool,
    kill_switch_active: bool,
    live_execution_policy_enabled: bool,
    max_live_order_notional: String,
    owner_authority_ref: String,
    risk_authority_ref: String,
    operator_authority_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LiveExecutionRiskPolicy {
    pub(super) max_order_notional: String,
    pub(super) owner_authority_ref: String,
    pub(super) risk_authority_ref: String,
    pub(super) operator_authority_ref: String,
    pub(super) source_ref: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LiveSizingConfig {
    instrument_id: String,
    base_asset: String,
    quote_asset: String,
    price_tick: String,
    quantity_step: String,
    min_quantity: String,
    max_quantity: String,
    min_notional: String,
    max_account_budget_fraction: String,
    evidence_max_age_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LiveSizingPreflight {
    pub(super) instrument_id: String,
    pub(super) base_asset: String,
    pub(super) quote_asset: String,
    pub(super) base_free: String,
    pub(super) quote_free: String,
    pub(super) price_tick: String,
    pub(super) quantity_step: String,
    pub(super) min_quantity: String,
    pub(super) max_quantity: String,
    pub(super) min_notional: String,
    pub(super) max_account_budget_fraction: String,
    pub(super) evidence_expires_at_unix_ms: u64,
    pub(super) source_ref: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveAdmissionConfig {
    schema_version: String,
    strategy_version_id: String,
    venue_id: String,
    product_type: String,
    production_http_base_url: String,
    production_websocket_base_url: String,
    market_data_adapter_ref: String,
    execution_adapter_ref: String,
    account_ref: String,
    credential_provider: String,
    api_key_env: String,
    api_secret_env: String,
    account_read_recv_window_ms: u64,
    owner_approval: bool,
    production_network_allowed: bool,
    authenticated_account_read_allowed: bool,
    live_run_creation_allowed: bool,
    order_submission_allowed: bool,
    cancel_order_allowed: bool,
    replace_order_allowed: bool,
    fill_reconciliation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    automatic_recovery_allowed: bool,
    manual_stop_required: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LiveAdmissionStatus {
    Blocked,
    ReadOnlyReady,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CredentialPresence {
    Missing,
    Present,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LiveGateState {
    Blocked,
    Ready,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LiveAccountRefreshAction {
    Refresh,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::dashboard) struct LiveAccountRefreshRequest {
    action: LiveAccountRefreshAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LiveAccountConnectionStatus {
    Blocked,
    Connected,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveVenueAdmission {
    venue_id: String,
    product_type: String,
    environment: String,
    production_http_base_url: String,
    production_websocket_base_url: String,
    market_data_adapter_ref: String,
    execution_adapter_ref: String,
    connection_state: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveAccountAdmission {
    account_ref: String,
    binding_status: String,
    authenticated_read_state: LiveGateState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveCredentialAdmission {
    provider: String,
    api_key_ref: String,
    api_secret_ref: String,
    api_key_presence: CredentialPresence,
    api_secret_presence: CredentialPresence,
    secret_values_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveRuntimeGateState {
    production_authenticated_read: bool,
    owner_approved_read_only: bool,
    no_order_mutation: bool,
    no_secret_persistence: bool,
    manual_online: bool,
}

impl LiveRuntimeGateState {
    fn from_reader<F>(mut runtime_gate_enabled: F) -> Self
    where
        F: FnMut(&str) -> bool,
    {
        Self {
            production_authenticated_read: runtime_gate_enabled(
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW,
            ),
            owner_approved_read_only: runtime_gate_enabled(
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED,
            ),
            no_order_mutation: runtime_gate_enabled(
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION,
            ),
            no_secret_persistence: runtime_gate_enabled(
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE,
            ),
            manual_online: runtime_gate_enabled(PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE),
        }
    }

    const fn all_open(&self) -> bool {
        self.production_authenticated_read
            && self.owner_approved_read_only
            && self.no_order_mutation
            && self.no_secret_persistence
            && self.manual_online
    }

    fn missing_refs(&self) -> Vec<String> {
        [
            (
                self.production_authenticated_read,
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW,
            ),
            (
                self.owner_approved_read_only,
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED,
            ),
            (
                self.no_order_mutation,
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION,
            ),
            (
                self.no_secret_persistence,
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE,
            ),
            (
                self.manual_online,
                PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE,
            ),
        ]
        .into_iter()
        .filter(|(open, _)| !open)
        .map(|(_, name)| format!("env://{name}"))
        .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveOrderLifecycleAdmission {
    submit: LiveGateState,
    cancel: LiveGateState,
    replace: LiveGateState,
    fill_reconciliation: LiveGateState,
    manual_stop_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveAdmissionBoundaries {
    read_only: bool,
    independent_live_admission_required: bool,
    owner_approval_granted: bool,
    inherited_from_backtest: bool,
    inherited_from_demo: bool,
    external_venue_connection: bool,
    production_venue_connection: bool,
    production_network_allowed: bool,
    external_network_attempted: bool,
    authenticated_account_read_allowed: bool,
    live_run_creation_allowed: bool,
    order_submission_allowed: bool,
    cancel_order_allowed: bool,
    replace_order_allowed: bool,
    order_mutation_allowed: bool,
    fill_reconciliation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    automatic_recovery_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

impl LiveAdmissionBoundaries {
    const fn effective(owner_approval_granted: bool, account_read_allowed: bool) -> Self {
        Self {
            read_only: true,
            independent_live_admission_required: true,
            owner_approval_granted,
            inherited_from_backtest: false,
            inherited_from_demo: false,
            external_venue_connection: false,
            production_venue_connection: false,
            production_network_allowed: account_read_allowed,
            external_network_attempted: false,
            authenticated_account_read_allowed: account_read_allowed,
            live_run_creation_allowed: false,
            order_submission_allowed: false,
            cancel_order_allowed: false,
            replace_order_allowed: false,
            order_mutation_allowed: false,
            fill_reconciliation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            automatic_recovery_allowed: false,
            real_orders_submitted: false,
            trading_controls_enabled: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveAccountShapeSummary {
    account_type_present: bool,
    balance_entry_count: Option<usize>,
    permission_entry_count: Option<usize>,
    can_trade_present: bool,
    can_withdraw_present: bool,
    can_deposit_present: bool,
    raw_account_response_exposed: bool,
    raw_balances_exposed: bool,
    raw_permissions_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveAccountResult {
    account_type: String,
    can_trade: bool,
    can_withdraw: bool,
    can_deposit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LiveRunCreationAdmission {
    pub(super) account_ref: String,
    pub(super) venue_id: String,
    pub(super) ready: bool,
    pub(super) risk_ready: bool,
    pub(super) source_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LiveRunPreflightAdmission {
    pub(super) connected: bool,
    pub(super) can_trade: bool,
    pub(super) evaluated_at_unix_ms: u64,
    pub(super) source_refs: Vec<String>,
    pub(super) sizing: LiveSizingPreflight,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveAssetBalance {
    asset: String,
    free: String,
    locked: String,
    total: String,
}

impl From<ProductLiveAssetBalance> for LiveAssetBalance {
    fn from(value: ProductLiveAssetBalance) -> Self {
        Self {
            asset: value.asset,
            free: value.free,
            locked: value.locked,
            total: value.total,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LiveValuationStatus {
    NotEvaluated,
    UnavailableWithoutPriceConversion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveFundsSummary {
    source_balance_entry_count: Option<usize>,
    non_zero_asset_count: usize,
    zero_balance_entry_count: Option<usize>,
    native_asset_units: bool,
    valuation_status: LiveValuationStatus,
    valuation_currency: Option<String>,
    portfolio_value: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveAccountRefreshData {
    strategy_id: String,
    strategy_version_id: String,
    environment: String,
    venue_id: String,
    account_ref: String,
    connection_status: LiveAccountConnectionStatus,
    evaluated_at_unix_ms: u64,
    endpoint_method: String,
    endpoint_url_redacted: String,
    runtime_gates: LiveRuntimeGateState,
    missing_runtime_gate_refs: Vec<String>,
    api_key_presence: CredentialPresence,
    api_secret_presence: CredentialPresence,
    network_attempted: bool,
    account_read_attempted: bool,
    response_status_code: Option<u16>,
    latency_ms: Option<u64>,
    response_shape: String,
    response_shape_validated: bool,
    shape_summary: LiveAccountShapeSummary,
    account_result: Option<LiveAccountResult>,
    funds_summary: LiveFundsSummary,
    asset_balances: Vec<LiveAssetBalance>,
    error_code: String,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveAccountRefreshBoundaries {
    read_only: bool,
    independent_live_admission_required: bool,
    owner_approval_granted: bool,
    production_network_allowed: bool,
    authenticated_account_read_allowed: bool,
    external_network_attempted: bool,
    account_mutation_allowed: bool,
    order_endpoint_access_allowed: bool,
    order_submission_allowed: bool,
    cancel_order_allowed: bool,
    replace_order_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    automatic_recovery_allowed: bool,
    secret_values_exposed: bool,
    raw_account_response_exposed: bool,
    normalized_account_results_exposed: bool,
    account_results_persisted: bool,
    trading_controls_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct LiveAccountRefreshResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: LiveAccountRefreshData,
    boundaries: LiveAccountRefreshBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct LiveAdmissionData {
    strategy_id: String,
    strategy_version_id: String,
    environment: String,
    admission_status: LiveAdmissionStatus,
    evaluated_at_unix_ms: u64,
    venue: LiveVenueAdmission,
    account: LiveAccountAdmission,
    credentials: LiveCredentialAdmission,
    order_lifecycle: LiveOrderLifecycleAdmission,
    blockers: Vec<String>,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct LiveAdmissionResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: LiveAdmissionData,
    boundaries: LiveAdmissionBoundaries,
}

pub(in crate::dashboard) async fn live_admission_api(
    State(state): State<DashboardServerState>,
    path: Result<AxumPath<LiveAdmissionPath>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<LiveAdmissionResponse> {
    let request_id = product_request_id();
    let path = path.map(|AxumPath(path)| path).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "live_admission_path"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_identifier("strategy_id", &path.strategy_id)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "strategy_id"))?;
        strategy_version::validate_requested_version_id("version_id", &path.version_id)?;
        let now = unix_time_ms();
        let source = load_product_source(&state, now)?;
        project_live_admission(
            &source,
            &path.strategy_id,
            &path.version_id,
            now,
            request_id.clone(),
            credential_is_present,
            runtime_gate_is_enabled,
        )
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn live_account_refresh_api(
    State(state): State<DashboardServerState>,
    path: Result<AxumPath<LiveAdmissionPath>, PathRejection>,
    payload: Result<Json<LiveAccountRefreshRequest>, JsonRejection>,
) -> ApiResult<LiveAccountRefreshResponse> {
    let request_id = product_request_id();
    let path = path.map(|AxumPath(path)| path).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "live_account_refresh_path"),
            &request_id,
        )
    })?;
    let Json(request) = payload.map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "request_body"),
            &request_id,
        )
    })?;
    if request.action != LiveAccountRefreshAction::Refresh {
        return Err(product_error_response(
            &product_error(ProductErrorKind::BadRequest, "action"),
            &request_id,
        ));
    }

    let worker_request_id = request_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        validate_identifier("strategy_id", &path.strategy_id)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "strategy_id"))?;
        strategy_version::validate_requested_version_id("version_id", &path.version_id)?;
        let now = unix_time_ms();
        let source = load_product_source(&state, now)?;
        project_live_account_refresh(
            &source,
            &path.strategy_id,
            &path.version_id,
            now,
            worker_request_id,
            credential_is_present,
            runtime_gate_is_enabled,
            execute_product_live_account_read,
        )
    })
    .await
    .map_err(|_| product_error(ProductErrorKind::ExecutionFailed, "live_account_worker"))
    .and_then(|result| result);

    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn project_live_account_refresh<F, G, H>(
    source: &ValidatedProductSource,
    strategy_id: &str,
    version_id: &str,
    evaluated_at_unix_ms: u64,
    request_id: String,
    credential_present: F,
    runtime_gate_enabled: G,
    account_reader: H,
) -> Result<LiveAccountRefreshResponse, ProductError>
where
    F: Fn(&str) -> bool,
    G: Fn(&str) -> bool,
    H: FnOnce(&str, &str, u64) -> ProductLiveAccountReadObservation,
{
    if source.strategy.strategy_id != strategy_id {
        return Err(product_error(ProductErrorKind::NotFound, "strategy_id"));
    }
    let strategy_version =
        strategy_version::load_product_strategy_version(source, evaluated_at_unix_ms)?;
    if strategy_version.strategy_version_id() != version_id {
        return Err(product_error(
            ProductErrorKind::VersionNotFound,
            "version_id",
        ));
    }
    let document: LiveAdmissionConfigDocument = toml::from_str(&source.raw_config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_admission_config"))?;
    validate_live_admission_config(&document.live_admission, version_id)?;

    let config = document.live_admission;
    let api_key_present = credential_present(&config.api_key_env);
    let api_secret_present = credential_present(&config.api_secret_env);
    let runtime_gates = LiveRuntimeGateState::from_reader(runtime_gate_enabled);
    let source_capability_enabled =
        config.production_network_allowed && config.authenticated_account_read_allowed;
    let account_read_allowed = source_capability_enabled
        && runtime_gates.all_open()
        && api_key_present
        && api_secret_present;
    let missing_runtime_gate_refs = runtime_gates.missing_refs();
    let observation = if account_read_allowed {
        account_reader(
            &config.api_key_env,
            &config.api_secret_env,
            config.account_read_recv_window_ms,
        )
    } else if !api_key_present || !api_secret_present {
        ProductLiveAccountReadObservation::blocked("credentials_missing")
    } else {
        ProductLiveAccountReadObservation::blocked("runtime_gates_missing")
    };
    let connected_observation_valid = observation.status == "connected"
        && observation.account_snapshot.is_some()
        && observation
            .response_status_code
            .is_some_and(|status| (200..=299).contains(&status))
        && observation.latency_ms.is_some()
        && observation.response_shape_validated
        && observation.account_type_present
        && observation.balance_entry_count.is_some()
        && observation.permission_entry_count.is_some()
        && observation.can_trade_present
        && observation.can_withdraw_present
        && observation.can_deposit_present;
    let connection_status = match observation.status.as_str() {
        "connected" if connected_observation_valid => LiveAccountConnectionStatus::Connected,
        "connected" => LiveAccountConnectionStatus::Failed,
        "failed" => LiveAccountConnectionStatus::Failed,
        _ => LiveAccountConnectionStatus::Blocked,
    };
    let error_code = if observation.status == "connected" && !connected_observation_valid {
        if observation.account_snapshot.is_none() {
            "account_result_missing".to_string()
        } else {
            "account_result_invalid".to_string()
        }
    } else {
        observation.error_code.clone()
    };
    let effective_account_read_allowed =
        account_read_allowed && connection_status != LiveAccountConnectionStatus::Blocked;
    let account_snapshot = if connection_status == LiveAccountConnectionStatus::Connected {
        observation.account_snapshot.clone()
    } else {
        None
    };
    let account_result = account_snapshot.as_ref().map(|snapshot| LiveAccountResult {
        account_type: snapshot.account_type.clone(),
        can_trade: snapshot.can_trade,
        can_withdraw: snapshot.can_withdraw,
        can_deposit: snapshot.can_deposit,
    });
    let funds_summary = LiveFundsSummary {
        source_balance_entry_count: account_snapshot
            .as_ref()
            .map(|snapshot| snapshot.source_balance_entry_count),
        non_zero_asset_count: account_snapshot
            .as_ref()
            .map_or(0, |snapshot| snapshot.assets.len()),
        zero_balance_entry_count: account_snapshot
            .as_ref()
            .map(|snapshot| snapshot.zero_balance_entry_count),
        native_asset_units: true,
        valuation_status: if account_snapshot.is_some() {
            LiveValuationStatus::UnavailableWithoutPriceConversion
        } else {
            LiveValuationStatus::NotEvaluated
        },
        valuation_currency: None,
        portfolio_value: None,
    };
    let mut asset_balances: Vec<LiveAssetBalance> = account_snapshot
        .map(|snapshot| {
            snapshot
                .assets
                .into_iter()
                .map(LiveAssetBalance::from)
                .collect()
        })
        .unwrap_or_default();
    asset_balances.sort_by(|left, right| left.asset.cmp(&right.asset));

    Ok(LiveAccountRefreshResponse {
        schema_version: LIVE_ACCOUNT_REFRESH_RESPONSE_SCHEMA_VERSION.to_string(),
        contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
        request_id,
        data: LiveAccountRefreshData {
            strategy_id: strategy_id.to_string(),
            strategy_version_id: version_id.to_string(),
            environment: "live".to_string(),
            venue_id: config.venue_id,
            account_ref: config.account_ref,
            connection_status,
            evaluated_at_unix_ms,
            endpoint_method: "GET".to_string(),
            endpoint_url_redacted: format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}/api/v3/account"),
            runtime_gates: runtime_gates.clone(),
            missing_runtime_gate_refs,
            api_key_presence: credential_presence(api_key_present),
            api_secret_presence: credential_presence(api_secret_present),
            network_attempted: observation.network_attempted,
            account_read_attempted: observation.account_read_attempted,
            response_status_code: observation.response_status_code,
            latency_ms: observation.latency_ms,
            response_shape: observation.response_shape,
            response_shape_validated: observation.response_shape_validated,
            shape_summary: LiveAccountShapeSummary {
                account_type_present: observation.account_type_present,
                balance_entry_count: observation.balance_entry_count,
                permission_entry_count: observation.permission_entry_count,
                can_trade_present: observation.can_trade_present,
                can_withdraw_present: observation.can_withdraw_present,
                can_deposit_present: observation.can_deposit_present,
                raw_account_response_exposed: false,
                raw_balances_exposed: false,
                raw_permissions_exposed: false,
            },
            account_result,
            funds_summary,
            asset_balances,
            error_code,
            source_refs: vec![
                format!("node-config:{}#live_admission", source.config_name),
                strategy_version.content_hash().to_string(),
            ],
        },
        boundaries: LiveAccountRefreshBoundaries {
            read_only: true,
            independent_live_admission_required: true,
            owner_approval_granted: runtime_gates.owner_approved_read_only,
            production_network_allowed: effective_account_read_allowed,
            authenticated_account_read_allowed: effective_account_read_allowed,
            external_network_attempted: observation.network_attempted,
            account_mutation_allowed: false,
            order_endpoint_access_allowed: false,
            order_submission_allowed: false,
            cancel_order_allowed: false,
            replace_order_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            automatic_recovery_allowed: false,
            secret_values_exposed: false,
            raw_account_response_exposed: false,
            normalized_account_results_exposed: connection_status
                == LiveAccountConnectionStatus::Connected,
            account_results_persisted: false,
            trading_controls_enabled: false,
        },
    })
}

pub(super) fn project_live_admission<F, G>(
    source: &ValidatedProductSource,
    strategy_id: &str,
    version_id: &str,
    evaluated_at_unix_ms: u64,
    request_id: String,
    credential_present: F,
    runtime_gate_enabled: G,
) -> Result<LiveAdmissionResponse, ProductError>
where
    F: Fn(&str) -> bool,
    G: Fn(&str) -> bool,
{
    if source.strategy.strategy_id != strategy_id {
        return Err(product_error(ProductErrorKind::NotFound, "strategy_id"));
    }
    let strategy_version =
        strategy_version::load_product_strategy_version(source, evaluated_at_unix_ms)?;
    if strategy_version.strategy_version_id() != version_id {
        return Err(product_error(
            ProductErrorKind::VersionNotFound,
            "version_id",
        ));
    }
    let document: LiveAdmissionConfigDocument = toml::from_str(&source.raw_config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_admission_config"))?;
    validate_live_admission_config(&document.live_admission, version_id)?;

    let config = document.live_admission;
    let api_key_present = credential_present(&config.api_key_env);
    let api_secret_present = credential_present(&config.api_secret_env);
    let runtime_gates = LiveRuntimeGateState::from_reader(runtime_gate_enabled);
    let account_read_allowed = config.production_network_allowed
        && config.authenticated_account_read_allowed
        && runtime_gates.all_open();
    let credentials_present = api_key_present && api_secret_present;
    let mut blockers = vec![
        "live_run_creation_not_authorized".to_string(),
        "follow_up_order_mutation_not_authorized".to_string(),
        "automatic_recovery_not_authorized".to_string(),
    ];
    if !runtime_gates.owner_approved_read_only {
        blockers.insert(0, "independent_owner_approval_missing".to_string());
    }
    if !config.production_network_allowed || !runtime_gates.all_open() {
        blockers.push("production_network_not_authorized".to_string());
    }
    if !config.authenticated_account_read_allowed || !runtime_gates.all_open() {
        blockers.push("authenticated_account_read_not_authorized".to_string());
    }
    if !api_key_present {
        blockers.push("api_key_missing".to_string());
    }
    if !api_secret_present {
        blockers.push("api_secret_missing".to_string());
    }

    Ok(LiveAdmissionResponse {
        schema_version: LIVE_ADMISSION_RESPONSE_SCHEMA_VERSION.to_string(),
        contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
        request_id,
        data: LiveAdmissionData {
            strategy_id: strategy_id.to_string(),
            strategy_version_id: version_id.to_string(),
            environment: "live".to_string(),
            admission_status: if account_read_allowed && credentials_present {
                LiveAdmissionStatus::ReadOnlyReady
            } else {
                LiveAdmissionStatus::Blocked
            },
            evaluated_at_unix_ms,
            venue: LiveVenueAdmission {
                venue_id: config.venue_id,
                product_type: config.product_type,
                environment: "production".to_string(),
                production_http_base_url: config.production_http_base_url,
                production_websocket_base_url: config.production_websocket_base_url,
                market_data_adapter_ref: config.market_data_adapter_ref,
                execution_adapter_ref: config.execution_adapter_ref,
                connection_state: "not_attempted".to_string(),
            },
            account: LiveAccountAdmission {
                account_ref: config.account_ref,
                binding_status: if account_read_allowed {
                    "authorized_read_only".to_string()
                } else {
                    "configured_not_authorized".to_string()
                },
                authenticated_read_state: if account_read_allowed && credentials_present {
                    LiveGateState::Ready
                } else {
                    LiveGateState::Blocked
                },
            },
            credentials: LiveCredentialAdmission {
                provider: config.credential_provider,
                api_key_ref: format!("env://{}", config.api_key_env),
                api_secret_ref: format!("env://{}", config.api_secret_env),
                api_key_presence: credential_presence(api_key_present),
                api_secret_presence: credential_presence(api_secret_present),
                secret_values_exposed: false,
            },
            order_lifecycle: LiveOrderLifecycleAdmission {
                submit: LiveGateState::Blocked,
                cancel: LiveGateState::Blocked,
                replace: LiveGateState::Blocked,
                fill_reconciliation: LiveGateState::Blocked,
                manual_stop_required: config.manual_stop_required,
            },
            blockers,
            source_refs: vec![
                format!("node-config:{}#live_admission", source.config_name),
                strategy_version.content_hash().to_string(),
            ],
        },
        boundaries: LiveAdmissionBoundaries::effective(
            runtime_gates.owner_approved_read_only,
            account_read_allowed,
        ),
    })
}

pub(super) fn evaluate_live_run_creation_admission(
    source: &ValidatedProductSource,
    strategy_id: &str,
    version_id: &str,
    evaluated_at_unix_ms: u64,
) -> Result<LiveRunCreationAdmission, ProductError> {
    let response = project_live_admission(
        source,
        strategy_id,
        version_id,
        evaluated_at_unix_ms,
        product_request_id(),
        credential_is_present,
        runtime_gate_is_enabled,
    )?;
    let document: LiveAdmissionConfigDocument = toml::from_str(&source.raw_config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_risk_config"))?;
    let risk_ready = document.risk.kill_switch_enabled && !document.risk.kill_switch_active;
    let credentials_present = response.data.credentials.api_key_presence
        == CredentialPresence::Present
        && response.data.credentials.api_secret_presence == CredentialPresence::Present;
    let mut source_refs = response.data.source_refs;
    source_refs.push(format!("node-config:{}#risk", source.config_name));
    source_refs.push(live_risk_config_ref(&document.risk)?);
    source_refs.sort();
    source_refs.dedup();
    Ok(LiveRunCreationAdmission {
        account_ref: response.data.account.account_ref,
        venue_id: response.data.venue.venue_id,
        ready: response.data.admission_status == LiveAdmissionStatus::ReadOnlyReady
            && response.boundaries.owner_approval_granted
            && response.boundaries.authenticated_account_read_allowed
            && credentials_present
            && risk_ready,
        risk_ready,
        source_refs,
    })
}

pub(super) fn evaluate_live_run_preflight_admission(
    source: &ValidatedProductSource,
    strategy_id: &str,
    version_id: &str,
    evaluated_at_unix_ms: u64,
) -> Result<LiveRunPreflightAdmission, ProductError> {
    let response = project_live_account_refresh(
        source,
        strategy_id,
        version_id,
        evaluated_at_unix_ms,
        product_request_id(),
        credential_is_present,
        runtime_gate_is_enabled,
        execute_product_live_account_read,
    )?;
    let document: LiveAdmissionConfigDocument = toml::from_str(&source.raw_config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_risk_config"))?;
    if !document.risk.kill_switch_enabled || document.risk.kill_switch_active {
        return Err(product_error(
            ProductErrorKind::LiveExecutionFailed,
            "live_risk_gate",
        ));
    }
    let connected = response.data.connection_status == LiveAccountConnectionStatus::Connected
        && response.data.response_shape_validated
        && response.data.missing_runtime_gate_refs.is_empty();
    let can_trade = response
        .data
        .account_result
        .as_ref()
        .is_some_and(|account| account.can_trade);
    let sizing = project_live_sizing_preflight(
        &document.live_sizing,
        &response.data.asset_balances,
        response.data.evaluated_at_unix_ms,
    )?;
    let mut source_refs = response.data.source_refs;
    source_refs.push(format!("node-config:{}#risk", source.config_name));
    source_refs.push(live_risk_config_ref(&document.risk)?);
    source_refs.sort();
    source_refs.dedup();
    Ok(LiveRunPreflightAdmission {
        connected,
        can_trade,
        evaluated_at_unix_ms: response.data.evaluated_at_unix_ms,
        source_refs,
        sizing,
    })
}

fn project_live_sizing_preflight(
    config: &LiveSizingConfig,
    balances: &[LiveAssetBalance],
    evaluated_at_unix_ms: u64,
) -> Result<LiveSizingPreflight, ProductError> {
    let decimals = [
        &config.price_tick,
        &config.quantity_step,
        &config.min_quantity,
        &config.max_quantity,
        &config.min_notional,
        &config.max_account_budget_fraction,
    ]
    .map(|value| rust_decimal::Decimal::from_str_exact(value).ok());
    let [
        Some(price_tick),
        Some(quantity_step),
        Some(min_quantity),
        Some(max_quantity),
        Some(min_notional),
        Some(max_account_budget_fraction),
    ] = decimals
    else {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_sizing_config",
        ));
    };
    if config.instrument_id.trim().is_empty()
        || config.base_asset.trim().is_empty()
        || config.quote_asset.trim().is_empty()
        || config.base_asset == config.quote_asset
        || price_tick <= rust_decimal::Decimal::ZERO
        || quantity_step <= rust_decimal::Decimal::ZERO
        || min_quantity <= rust_decimal::Decimal::ZERO
        || max_quantity < min_quantity
        || min_notional <= rust_decimal::Decimal::ZERO
        || max_account_budget_fraction <= rust_decimal::Decimal::ZERO
        || max_account_budget_fraction > rust_decimal::Decimal::ONE
        || config.evidence_max_age_ms == 0
        || config.evidence_max_age_ms > 15 * 60 * 1_000
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_sizing_config",
        ));
    }
    let balance = |asset: &str| {
        balances
            .iter()
            .find(|value| value.asset == asset)
            .map_or("0", |value| value.free.as_str())
            .to_string()
    };
    let base_free = balance(&config.base_asset);
    let quote_free = balance(&config.quote_asset);
    if rust_decimal::Decimal::from_str_exact(&base_free).is_err()
        || rust_decimal::Decimal::from_str_exact(&quote_free).is_err()
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_sizing_balance",
        ));
    }
    let raw = serde_json::to_vec(config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_sizing_config"))?;
    let hash = digest(&SHA256, &raw);
    let mut source_ref = String::from("sizing-config-sha256:");
    for byte in hash.as_ref() {
        source_ref.push_str(&format!("{byte:02x}"));
    }
    Ok(LiveSizingPreflight {
        instrument_id: config.instrument_id.clone(),
        base_asset: config.base_asset.clone(),
        quote_asset: config.quote_asset.clone(),
        base_free,
        quote_free,
        price_tick: config.price_tick.clone(),
        quantity_step: config.quantity_step.clone(),
        min_quantity: config.min_quantity.clone(),
        max_quantity: config.max_quantity.clone(),
        min_notional: config.min_notional.clone(),
        max_account_budget_fraction: config.max_account_budget_fraction.clone(),
        evidence_expires_at_unix_ms: evaluated_at_unix_ms
            .saturating_add(config.evidence_max_age_ms),
        source_ref,
    })
}

fn live_risk_config_ref(risk: &LiveRiskConfig) -> Result<String, ProductError> {
    let raw = serde_json::to_vec(risk)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_risk_config"))?;
    let hash = digest(&SHA256, &raw);
    let mut value = String::from("risk-config-sha256:");
    for byte in hash.as_ref() {
        value.push_str(&format!("{byte:02x}"));
    }
    Ok(value)
}

pub(super) fn evaluate_live_execution_risk_policy(
    source: &ValidatedProductSource,
) -> Result<LiveExecutionRiskPolicy, ProductError> {
    let document: LiveAdmissionConfigDocument = toml::from_str(&source.raw_config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "live_risk_config"))?;
    let risk = document.risk;
    let max_order_notional = rust_decimal::Decimal::from_str_exact(&risk.max_live_order_notional)
        .map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "live_execution_risk_policy",
        )
    })?;
    let authorities = [
        risk.owner_authority_ref.as_str(),
        risk.risk_authority_ref.as_str(),
        risk.operator_authority_ref.as_str(),
    ];
    let authorities_distinct = authorities.iter().all(|value| {
        !value.trim().is_empty() && !value.contains(['\n', '\r']) && value.len() <= 160
    }) && authorities
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        == 3;
    if !risk.kill_switch_enabled
        || risk.kill_switch_active
        || !risk.live_execution_policy_enabled
        || max_order_notional <= rust_decimal::Decimal::ZERO
        || !authorities_distinct
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_execution_risk_policy",
        ));
    }
    Ok(LiveExecutionRiskPolicy {
        max_order_notional: risk.max_live_order_notional.clone(),
        owner_authority_ref: risk.owner_authority_ref.clone(),
        risk_authority_ref: risk.risk_authority_ref.clone(),
        operator_authority_ref: risk.operator_authority_ref.clone(),
        source_ref: live_risk_config_ref(&risk)?,
    })
}

fn validate_live_admission_config(
    config: &LiveAdmissionConfig,
    version_id: &str,
) -> Result<(), ProductError> {
    let identity_matches = config.schema_version == LIVE_ADMISSION_CONFIG_SCHEMA_VERSION
        && config.strategy_version_id == version_id;
    let venue_matches = config.venue_id == "BINANCE"
        && config.product_type == "spot"
        && config.production_http_base_url == BINANCE_PRODUCTION_HTTP_BASE_URL
        && config.production_websocket_base_url == BINANCE_PRODUCTION_WEBSOCKET_BASE_URL
        && config.market_data_adapter_ref == BINANCE_MARKET_DATA_ADAPTER_REF
        && config.execution_adapter_ref == BINANCE_EXECUTION_ADAPTER_REF
        && config.account_ref == BINANCE_LIVE_ACCOUNT_REF;
    let credentials_match = config.credential_provider == "environment"
        && config.api_key_env == BINANCE_LIVE_API_KEY_ENV
        && config.api_secret_env == BINANCE_LIVE_API_SECRET_ENV
        && config.account_read_recv_window_ms == 5_000;
    if !identity_matches {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_admission_identity",
        ));
    }
    if !venue_matches {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_admission_venue",
        ));
    }
    if !credentials_match {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "live_admission_credentials",
        ));
    }
    if config.owner_approval
        || !config.production_network_allowed
        || !config.authenticated_account_read_allowed
        || config.live_run_creation_allowed
        || config.order_submission_allowed
        || config.cancel_order_allowed
        || config.replace_order_allowed
        || config.fill_reconciliation_allowed
        || config.automatic_retry_allowed
        || config.automatic_remediation_allowed
        || config.automatic_recovery_allowed
        || !config.manual_stop_required
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "live_admission_boundaries",
        ));
    }
    Ok(())
}

fn credential_is_present(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn runtime_gate_is_enabled(name: &str) -> bool {
    std::env::var(name).ok().as_deref() == Some("1")
}

const fn credential_presence(present: bool) -> CredentialPresence {
    if present {
        CredentialPresence::Present
    } else {
        CredentialPresence::Missing
    }
}

#[cfg(test)]
mod tests {
    use super::{LiveRiskConfig, live_risk_config_ref};

    #[test]
    fn live_risk_source_ref_binds_kill_switch_content() {
        let ready = live_risk_config_ref(&LiveRiskConfig {
            kill_switch_enabled: true,
            kill_switch_active: false,
            live_execution_policy_enabled: true,
            max_live_order_notional: "10.00".to_string(),
            owner_authority_ref: "role://institution-owner".to_string(),
            risk_authority_ref: "policy://risk/v1".to_string(),
            operator_authority_ref: "role://operations-operator".to_string(),
        })
        .unwrap();
        let active = live_risk_config_ref(&LiveRiskConfig {
            kill_switch_enabled: true,
            kill_switch_active: true,
            live_execution_policy_enabled: true,
            max_live_order_notional: "10.00".to_string(),
            owner_authority_ref: "role://institution-owner".to_string(),
            risk_authority_ref: "policy://risk/v1".to_string(),
            operator_authority_ref: "role://operations-operator".to_string(),
        })
        .unwrap();
        assert_eq!(
            ready,
            "risk-config-sha256:bd212bfc481c2216bdbfd12700fefe664a166634905ee4fa86909a3c6c5c1454"
        );
        assert_ne!(ready, active);
    }

    #[test]
    fn live_risk_config_rejects_unbound_unknown_fields() {
        let raw = "
kill_switch_enabled = true
kill_switch_active = false
max_notional = 1000
";
        assert!(toml::from_str::<LiveRiskConfig>(raw).is_err());
    }
}
