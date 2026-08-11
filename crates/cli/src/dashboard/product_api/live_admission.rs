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

use axum::{
    Json,
    extract::{Path as AxumPath, RawQuery, State, rejection::PathRejection},
};
use serde::{Deserialize, Serialize};

use super::{
    DashboardServerState, PRODUCT_API_CONTRACT_VERSION, ProductError, ProductErrorKind,
    ValidatedProductSource, load_product_source, product_error, product_error_response,
    product_request_id, reject_detail_query, strategy_version, unix_time_ms, validate_identifier,
};
use crate::dashboard::ApiResult;

const LIVE_ADMISSION_CONFIG_SCHEMA_VERSION: &str = "ntpro.live_admission.config.v1";
const LIVE_ADMISSION_RESPONSE_SCHEMA_VERSION: &str = "ntpro.product_api.live_admission.response.v1";
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
    const fn blocked() -> Self {
        Self {
            read_only: true,
            independent_live_admission_required: true,
            owner_approval_granted: false,
            inherited_from_backtest: false,
            inherited_from_demo: false,
            external_venue_connection: false,
            production_venue_connection: false,
            production_network_allowed: false,
            external_network_attempted: false,
            authenticated_account_read_allowed: false,
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
        )
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(super) fn project_live_admission<F>(
    source: &ValidatedProductSource,
    strategy_id: &str,
    version_id: &str,
    evaluated_at_unix_ms: u64,
    request_id: String,
    credential_present: F,
) -> Result<LiveAdmissionResponse, ProductError>
where
    F: Fn(&str) -> bool,
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
    let mut blockers = vec![
        "independent_owner_approval_missing".to_string(),
        "production_network_not_authorized".to_string(),
        "authenticated_account_read_not_authorized".to_string(),
        "live_run_creation_not_authorized".to_string(),
        "order_lifecycle_not_authorized".to_string(),
        "automatic_recovery_not_authorized".to_string(),
    ];
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
            admission_status: LiveAdmissionStatus::Blocked,
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
                binding_status: "configured_not_authorized".to_string(),
                authenticated_read_state: LiveGateState::Blocked,
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
        boundaries: LiveAdmissionBoundaries::blocked(),
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
        && config.api_secret_env == BINANCE_LIVE_API_SECRET_ENV;
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
        || config.production_network_allowed
        || config.authenticated_account_read_allowed
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

const fn credential_presence(present: bool) -> CredentialPresence {
    if present {
        CredentialPresence::Present
    } else {
        CredentialPresence::Missing
    }
}
