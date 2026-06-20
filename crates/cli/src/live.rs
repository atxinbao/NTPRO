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

use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use nautilus_binance::common::{consts::BINANCE_API_KEY_HEADER, credential::SigningCredential};
use nautilus_common::enums::Environment;
use nautilus_core::string::urlencoding;
use nautilus_live::{
    node::{LiveNode, NodeState},
    status::{
        ConnectionStatus, ExecutionStatus, LifecycleStatus, NodeStatus, ProcessMode, SnapshotValue,
    },
};
use nautilus_model::{
    identifiers::{AccountId, TraderId, Venue},
    types::Money,
};
use nautilus_sandbox::{SandboxExecutionClientConfig, SandboxExecutionClientFactory};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, timeout};

use crate::{
    artifacts::{atomic_write_json, atomic_write_text},
    endpoint_classifier::{EndpointAuthKind, EndpointClassifier},
    opt::{
        LiveCommand, LiveOpt, LiveProductionAccountSnapshotContractOpt,
        LiveProductionPublicReadProbeOpt, LiveProductionShadowPortfolioRuntimeOpt,
        LiveProductionShadowStrategySessionOpt, LiveRunOpt,
        LiveTestnetExecutionArtifactContractOpt, LiveTestnetOrderGateOpt,
        LiveTestnetOrderPreflightOpt, LiveTestnetOrderRequestPreviewOpt,
        LiveTestnetOrderTestPreflightOpt, LiveTestnetReconciliationFixtureOpt, LiveValidateOpt,
        ProductionPublicReadEndpoint, TestnetReconciliationScenario,
    },
    process::process_is_alive,
    strategy_session::{
        STRATEGY_ORDER_PREFLIGHT_SCHEMA_VERSION, StrategyOrderPreflightInput, StrategyRiskControls,
        StrategyRuntimeCounters, StrategySession, ema_cross_demo_fixture_bars,
    },
    supervisor::{NodeMetricArtifacts, NodeMetricCounts, NodeMetrics, write_node_metrics_artifact},
};

const LIVE_INIT_SMOKE_MODE: &str = "live-init-smoke";
const STRATEGY_SESSION_SHADOW_MODE: &str = "shadow";
const BUILTIN_STRATEGY_PACKAGE: &str = "builtin";
const EMA_CROSS_DEMO_STRATEGY: &str = "ema_cross_demo";
const FIXTURE_STREAM_DATA_MODE: &str = "fixture_stream";
const SANDBOX_ENVIRONMENT: &str = "sandbox";
const SANDBOX_SIMULATED_EXECUTION: &str = "sandbox-simulated-execution";
const DISABLED_ORDER_SUBMISSION: &str = "disabled";
const BINANCE_TESTNET_HTTP_BASE_URL: &str = "https://testnet.binance.vision";
const BINANCE_PRODUCTION_HTTP_BASE_URL: &str = "https://api.binance.com";
const TESTNET_ORDER_DISABLED_MODE: &str = "disabled";
const TESTNET_ORDER_OWNER_MANUAL_GATE: &str = "owner-approved-manual";
const TESTNET_ORDER_LIMIT_TYPE: &str = "LIMIT";
const TESTNET_ORDER_GTC_TIF: &str = "GTC";
const TESTNET_ORDER_ENV_ALLOW: &str = "NTPRO_ALLOW_BINANCE_TESTNET_ORDER";
const TESTNET_ORDER_ENV_OWNER_APPROVED: &str = "NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER";
const TESTNET_ORDER_ENV_TINY_NOTIONAL: &str = "NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL";
const TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT: &str = "NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT";
const TESTNET_ORDER_ENDPOINT_TEST: &str = "/api/v3/order/test";
const TESTNET_ORDER_ENDPOINT_ORDER: &str = "/api/v3/order";
const TESTNET_ORDER_METHOD_POST: &str = "POST";
const TESTNET_ORDER_METHOD_DELETE: &str = "DELETE";
const TESTNET_ORDER_PREVIEW_SCHEMA_VERSION: &str = "ntpro.v100_signed_order_request_preview.v1";
const TESTNET_EXECUTION_ARTIFACT_SCHEMA_VERSION: &str = "ntpro.v100_execution_artifact_contract.v1";
const TESTNET_RECONCILIATION_FIXTURE_SCHEMA_VERSION: &str =
    "ntpro.v100_reconciliation_fixture_report.v1";
const PRODUCTION_PUBLIC_READ_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v110_production_public_read_probe.v1";
const PRODUCTION_PUBLIC_ONLINE_READ_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v120_production_public_online_read_probe.v1";
const PRODUCTION_PUBLIC_READ_ENV_ALLOW: &str = "NTPRO_ALLOW_PRODUCTION_PUBLIC_READ";
const PRODUCTION_PUBLIC_READ_ENV_READ_ONLY: &str = "NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY";
const PRODUCTION_PUBLIC_READ_ENV_NO_ORDER_MUTATION: &str =
    "NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION";
const PRODUCTION_PUBLIC_READ_ENV_MANUAL_ONLINE: &str = "NTPRO_V12_MANUAL_ONLINE";
const PRODUCTION_PUBLIC_READ_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PRODUCTION_ACCOUNT_SNAPSHOT_SCHEMA_VERSION: &str =
    "ntpro.v110_authenticated_account_snapshot_contract.v1";
const PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION: &str =
    "ntpro.v120_authenticated_account_snapshot_online_read.v1";
const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW: &str = "NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ";
const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED: &str =
    "NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY";
const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION: &str =
    "NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION";
const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE: &str =
    "NTPRO_CONFIRM_NO_SECRET_PERSISTENCE";
const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE: &str = "NTPRO_V12_MANUAL_ONLINE";
const PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT: &str = "/api/v3/account";
const PRODUCTION_ACCOUNT_SNAPSHOT_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION: &str =
    "ntpro.v120_shadow_portfolio_runtime.v1";
const PRODUCTION_SHADOW_PORTFOLIO_COMPAT_SCHEMA_VERSION: &str =
    "ntpro.v110_shadow_portfolio_snapshot.v1";
const PRODUCTION_SHADOW_STRATEGY_SESSION_EVENT_SCHEMA_VERSION: &str =
    "ntpro.v120_shadow_strategy_session_event.v1";
const START_STOP_SHUTDOWN: &str = "start-stop";
const DEFAULT_NTPRO_NODE_HEARTBEAT_INTERVAL_MS: u64 = 1_000;
const DEFAULT_NTPRO_NODE_SHUTDOWN_TIMEOUT_MS: u64 = 5_000;
const SHUTDOWN_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NtproNodeRunControls {
    pub max_runtime: Option<Duration>,
    pub heartbeat_interval: Duration,
    pub parent_pid: Option<u32>,
    pub shutdown_timeout: Duration,
}

impl NtproNodeRunControls {
    /// # Errors
    ///
    /// Returns an error when any non-optional duration is zero.
    pub fn from_millis(
        max_runtime_ms: Option<u64>,
        heartbeat_interval_ms: u64,
        parent_pid: Option<u32>,
        shutdown_timeout_ms: u64,
    ) -> anyhow::Result<Self> {
        let max_runtime = match max_runtime_ms {
            Some(0) => anyhow::bail!("max_runtime_ms must be greater than zero when set"),
            Some(millis) => Some(Duration::from_millis(millis)),
            None => None,
        };
        Ok(Self {
            max_runtime,
            heartbeat_interval: non_zero_duration("heartbeat_interval_ms", heartbeat_interval_ms)?,
            parent_pid,
            shutdown_timeout: non_zero_duration("shutdown_timeout_ms", shutdown_timeout_ms)?,
        })
    }
}

impl Default for NtproNodeRunControls {
    fn default() -> Self {
        Self {
            max_runtime: None,
            heartbeat_interval: Duration::from_millis(DEFAULT_NTPRO_NODE_HEARTBEAT_INTERVAL_MS),
            parent_pid: None,
            shutdown_timeout: Duration::from_millis(DEFAULT_NTPRO_NODE_SHUTDOWN_TIMEOUT_MS),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalLiveConfig {
    run: LiveRunConfig,
    system: LiveSystemConfig,
    adapter: LiveAdapterConfig,
    execution: LiveExecutionConfig,
    shutdown: LiveShutdownConfig,
    output: Option<LiveOutputConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveRunConfig {
    id: String,
    mode: String,
    environment: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveSystemConfig {
    trader_id: String,
    node_name: Option<String>,
    instance_id: Option<String>,
    load_state: Option<bool>,
    save_state: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveAdapterConfig {
    name: String,
    kind: String,
    account_id: String,
    venue: String,
    starting_balances: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveExecutionConfig {
    order_submission: String,
    reconciliation: bool,
    external_venue_connection: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveShutdownConfig {
    mode: String,
    post_stop_delay_secs: u64,
    connection_timeout_secs: u64,
    disconnection_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveOutputConfig {
    dir: Option<PathBuf>,
    write_summary: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct StrategyNodeConfig {
    node: StrategyNodeSection,
    strategy: StrategyNodeStrategySection,
    market: StrategyNodeMarketSection,
    execution: StrategyNodeExecutionSection,
    testnet_order: Option<StrategyNodeTestnetOrderSection>,
    risk: StrategyNodeRiskSection,
    shutdown: Option<LiveShutdownConfig>,
    output: Option<LiveOutputConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StrategyNodeSection {
    node_id: String,
    mode: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StrategyNodeStrategySection {
    strategy_id: String,
    strategy_package: Option<String>,
    strategy_runtime: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StrategyNodeMarketSection {
    venue: Option<String>,
    symbols: Vec<String>,
    data_mode: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StrategyNodeExecutionSection {
    venue: Option<String>,
    order_submission: String,
    external_venue_connection: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StrategyNodeTestnetOrderSection {
    enabled: bool,
    mode: String,
    manual_gate: String,
    http_base_url: String,
    symbol: String,
    instrument_id: String,
    side: String,
    order_type: String,
    time_in_force: String,
    price: String,
    quantity: String,
    notional: String,
    cancel_after_submit_ms: u64,
    owner_approval_required: bool,
    manual_env_gate_required: bool,
    production_endpoint_allowed: bool,
    dashboard_order_controls: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StrategyNodeRiskSection {
    kill_switch_enabled: bool,
    kill_switch_active: bool,
}

#[derive(Debug, Serialize)]
struct TestnetOrderPreflightReport {
    schema_version: String,
    status: String,
    passed: bool,
    reasons: Vec<String>,
    symbol: String,
    account_id: String,
    notional: String,
    max_order_notional: String,
    open_order_count: u64,
    max_open_orders: u64,
    observed_clock_skew_ms: u64,
    max_clock_skew_ms: u64,
    market_age_ms: Option<u64>,
    max_market_age_ms: u64,
    order_submission_remains_disabled: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    production_endpoint_allowed: bool,
    dashboard_order_controls: bool,
}

struct EnvOnlyTestnetOrderCredentials {
    api_key_env: String,
    api_secret_env: String,
    api_key_value: Option<String>,
    api_secret_value: Option<String>,
    sensitive_values: Vec<String>,
}

impl EnvOnlyTestnetOrderCredentials {
    fn from_values(
        api_key_env: String,
        api_key_value: Option<String>,
        api_secret_env: String,
        api_secret_value: Option<String>,
    ) -> Self {
        let sensitive_values = [api_key_value.as_ref(), api_secret_value.as_ref()]
            .into_iter()
            .flatten()
            .filter(|value| !value.is_empty())
            .cloned()
            .collect();

        Self {
            api_key_env,
            api_secret_env,
            api_key_value,
            api_secret_value,
            sensitive_values,
        }
    }

    fn signing_credential(&self) -> anyhow::Result<SigningCredential> {
        let api_key = self
            .api_key_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("signed order request preview requires API key env value")?;
        let api_secret = self
            .api_secret_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("signed order request preview requires API secret env value")?;

        Ok(SigningCredential::new(
            api_key.to_string(),
            api_secret.to_string(),
        ))
    }

    fn ensure_no_secret_values_absent(&self, label: &str, body: &str) -> anyhow::Result<()> {
        for secret_value in &self.sensitive_values {
            if body.contains(secret_value) {
                anyhow::bail!(
                    "testnet signed order redaction guard blocked secret value leak in {label}"
                );
            }
        }
        Ok(())
    }
}

struct EnvOnlyProductionReadCredentials {
    api_key_env: String,
    api_secret_env: String,
    api_key_value: Option<String>,
    api_secret_value: Option<String>,
    sensitive_values: Vec<String>,
}

impl EnvOnlyProductionReadCredentials {
    fn from_values(
        api_key_env: String,
        api_key_value: Option<String>,
        api_secret_env: String,
        api_secret_value: Option<String>,
    ) -> Self {
        let sensitive_values = [api_key_value.as_ref(), api_secret_value.as_ref()]
            .into_iter()
            .flatten()
            .filter(|value| !value.is_empty())
            .cloned()
            .collect();

        Self {
            api_key_env,
            api_secret_env,
            api_key_value,
            api_secret_value,
            sensitive_values,
        }
    }

    fn api_key_present(&self) -> bool {
        self.api_key_value
            .as_deref()
            .is_some_and(|value| !value.is_empty())
    }

    fn api_secret_present(&self) -> bool {
        self.api_secret_value
            .as_deref()
            .is_some_and(|value| !value.is_empty())
    }

    fn signing_credential(&self) -> anyhow::Result<SigningCredential> {
        let api_key = self
            .api_key_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("production account snapshot requires API key env value")?;
        let api_secret = self
            .api_secret_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("production account snapshot requires API secret env value")?;

        Ok(SigningCredential::new(
            api_key.to_string(),
            api_secret.to_string(),
        ))
    }

    fn ensure_no_secret_values_absent(&self, label: &str, body: &str) -> anyhow::Result<()> {
        for secret_value in &self.sensitive_values {
            if body.contains(secret_value) {
                anyhow::bail!(
                    "production account snapshot redaction guard blocked secret value leak in {label}"
                );
            }
        }
        Ok(())
    }
}

struct ProductionAccountSnapshotSignedRequest {
    method: String,
    endpoint_path: String,
    endpoint_url_redacted: String,
    query_without_signature: String,
    signature: String,
    signed_query: String,
    api_key_header_name: String,
    api_key_header_value: String,
}

impl Debug for ProductionAccountSnapshotSignedRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductionAccountSnapshotSignedRequest")
            .field("method", &self.method)
            .field("endpoint_path", &self.endpoint_path)
            .field("endpoint_url_redacted", &self.endpoint_url_redacted)
            .field("query_without_signature", &self.query_without_signature)
            .field("signature", &"<redacted>")
            .field("signed_query", &"<redacted>")
            .field("api_key_header_name", &self.api_key_header_name)
            .field("api_key_header_value", &"<redacted>")
            .finish()
    }
}

impl ProductionAccountSnapshotSignedRequest {
    fn signed_url_for_execution(&self) -> String {
        format!("{}?{}", self.endpoint_url_redacted, self.signed_query)
    }

    fn ensure_redacted(
        &self,
        credentials: &EnvOnlyProductionReadCredentials,
    ) -> anyhow::Result<()> {
        let body = format!("{self:?}");
        credentials.ensure_no_secret_values_absent("production-account-snapshot-request", &body)?;
        for (label, sensitive_value) in [
            ("signature", self.signature.as_str()),
            ("signed query", self.signed_query.as_str()),
            ("API key header value", self.api_key_header_value.as_str()),
        ] {
            if !sensitive_value.is_empty() && body.contains(sensitive_value) {
                anyhow::bail!("production account snapshot request leaked {label}");
            }
        }
        Ok(())
    }
}

struct TestnetSignedOrderRequest {
    method: String,
    endpoint_path: String,
    endpoint_url_redacted: String,
    query_without_signature: String,
    signature: String,
    signed_query: String,
    api_key_header_name: String,
    api_key_header_value: String,
    action: String,
}

impl Debug for TestnetSignedOrderRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TestnetSignedOrderRequest")
            .field("method", &self.method)
            .field("endpoint_path", &self.endpoint_path)
            .field("endpoint_url_redacted", &self.endpoint_url_redacted)
            .field("query_without_signature", &self.query_without_signature)
            .field("signature", &"<redacted>")
            .field("signed_query", &"<redacted>")
            .field("api_key_header_name", &self.api_key_header_name)
            .field("api_key_header_value", &"<redacted>")
            .field("action", &self.action)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestnetSignedOrderRequestPreview {
    schema_version: String,
    endpoint_class: String,
    endpoint_url_redacted: String,
    request_method: String,
    request_target: String,
    query_shape: String,
    order_action: String,
    api_key_env: String,
    api_secret_env: String,
    api_key_header_name: String,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    order_submission: String,
    order_submission_remains_disabled: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    production_endpoint_allowed: bool,
    dashboard_order_controls: bool,
    secrets_redacted: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestnetOrderTestPreflightReport {
    schema_version: String,
    status: String,
    endpoint_class: String,
    request_method: String,
    request_target: String,
    query_shape: String,
    api_key_header_name: String,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    signature_preflight: String,
    binance_order_test_acceptance: String,
    matching_engine_submission: bool,
    order_submission: String,
    order_submission_remains_disabled: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    production_endpoint_allowed: bool,
    dashboard_order_controls: bool,
    secrets_redacted: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestnetExecutionArtifactContractReport {
    schema_version: String,
    status: String,
    artifact_family: String,
    request_artifact: TestnetExecutionArtifactContractEntry,
    order_test_artifact: TestnetExecutionArtifactContractEntry,
    submit_ack_artifact: TestnetExecutionArtifactContractEntry,
    cancel_ack_artifact: TestnetExecutionArtifactContractEntry,
    lifecycle_artifact: TestnetExecutionArtifactContractEntry,
    reconciliation_artifact: TestnetExecutionArtifactContractEntry,
    counters: TestnetExecutionArtifactCounters,
    manual_submit_cancel_proof_observed: bool,
    matching_engine_submission: bool,
    order_submission_remains_disabled: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    production_endpoint_allowed: bool,
    dashboard_order_controls: bool,
    secrets_redacted: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestnetExecutionArtifactContractEntry {
    name: String,
    schema: String,
    status: String,
    source: String,
    redaction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestnetExecutionArtifactCounters {
    testnet_orders_submitted: u64,
    testnet_orders_canceled: u64,
    production_orders_submitted: u64,
    production_orders_canceled: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestnetReconciliationFixtureReport {
    schema_version: String,
    status: String,
    symbol: String,
    scenario: String,
    scenario_count: usize,
    scenarios: Vec<TestnetReconciliationFixtureEntry>,
    counters: TestnetExecutionArtifactCounters,
    risk_halted: bool,
    new_orders_blocked: bool,
    manual_submit_cancel_proof_observed: bool,
    matching_engine_submission: bool,
    order_submission_remains_disabled: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    production_endpoint_allowed: bool,
    dashboard_order_controls: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestnetReconciliationFixtureEntry {
    name: String,
    local_state: String,
    exchange_state: String,
    risk_halted: bool,
    new_orders_blocked: bool,
    action_required: String,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionPublicReadProbeReport {
    schema_version: String,
    status: String,
    endpoint: String,
    endpoint_class: String,
    http_base_url: String,
    method: String,
    path: String,
    request_url_redacted: String,
    requires_api_key: bool,
    requires_signature: bool,
    read_allowed: bool,
    contract_ready: bool,
    online_read_allowed: bool,
    mutation_allowed: bool,
    manual_gate_required: bool,
    missing_cli_flags: Vec<String>,
    missing_env_vars: Vec<String>,
    manual_online_requested: bool,
    online_execution_supported: bool,
    network_attempted: bool,
    production_public_online_read_attempted: bool,
    response_status_code: Option<u16>,
    response_shape: String,
    response_shape_validated: bool,
    latency_ms: Option<u64>,
    error_code: String,
    credentials_used: bool,
    account_mutation_attempted: bool,
    production_order_submission_attempted: bool,
    production_order_mutation_attempted: bool,
    dashboard_order_controls_enabled: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProductionPublicReadProbeHttpResult {
    status: String,
    latency_ms: Option<u64>,
    http_status: Option<u16>,
    response_shape: String,
    response_shape_validated: bool,
    error_code: String,
    network_attempted: bool,
    diagnostic: String,
}

impl ProductionPublicReadProbeHttpResult {
    fn success(endpoint: ProductionPublicReadEndpoint, latency_ms: u64, http_status: u16) -> Self {
        Self {
            status: "online_read_probe_ok".to_string(),
            latency_ms: Some(latency_ms),
            http_status: Some(http_status),
            response_shape: production_public_read_response_shape(endpoint).to_string(),
            response_shape_validated: true,
            error_code: "none".to_string(),
            network_attempted: true,
            diagnostic: format!(
                "V120 production public read-only probe succeeded with GET {} and HTTP {http_status}; no credentials, account reads, order endpoints, or Dashboard controls were used.",
                production_public_read_endpoint_parts(endpoint).1
            ),
        }
    }

    fn failure(
        endpoint: ProductionPublicReadEndpoint,
        latency_ms: Option<u64>,
        http_status: Option<u16>,
        error_code: &str,
    ) -> Self {
        let status_detail = http_status
            .map(|status| format!(" HTTP {status}"))
            .unwrap_or_default();
        Self {
            status: "online_read_probe_failed".to_string(),
            latency_ms,
            http_status,
            response_shape: production_public_read_response_shape(endpoint).to_string(),
            response_shape_validated: false,
            error_code: error_code.to_string(),
            network_attempted: true,
            diagnostic: format!(
                "V120 production public read-only probe attempted GET {} and failed with {error_code}.{status_detail} No credentials, account reads, order endpoints, or Dashboard controls were used.",
                production_public_read_endpoint_parts(endpoint).1
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BinanceServerTimeResponse {
    #[serde(rename = "serverTime")]
    server_time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionAccountSnapshotContractReport {
    schema_version: String,
    status: String,
    endpoint_class: String,
    http_base_url: String,
    method: String,
    path: String,
    request_url_redacted: String,
    query_shape: String,
    requires_api_key: bool,
    requires_signature: bool,
    read_allowed: bool,
    contract_ready: bool,
    online_read_allowed: bool,
    mutation_allowed: bool,
    owner_gate_required: bool,
    manual_gate_required: bool,
    missing_cli_flags: Vec<String>,
    missing_env_vars: Vec<String>,
    manual_online_requested: bool,
    online_execution_supported: bool,
    network_attempted: bool,
    response_status_code: Option<u16>,
    response_shape: String,
    response_shape_validated: bool,
    response_shape_summary: ProductionAccountSnapshotShapeSummary,
    latency_ms: Option<u64>,
    error_code: String,
    env_credentials_only: bool,
    api_key_env: String,
    api_secret_env: String,
    api_key_present: bool,
    api_secret_present: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    account_read_attempted: bool,
    account_mutation_attempted: bool,
    order_endpoint_access_attempted: bool,
    production_order_submission_attempted: bool,
    production_order_mutation_attempted: bool,
    dashboard_order_controls_enabled: bool,
    secrets_redacted: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProductionAccountSnapshotHttpResult {
    status: String,
    latency_ms: Option<u64>,
    http_status: Option<u16>,
    response_shape: String,
    response_shape_validated: bool,
    response_shape_summary: ProductionAccountSnapshotShapeSummary,
    error_code: String,
    network_attempted: bool,
    diagnostic: String,
}

impl ProductionAccountSnapshotHttpResult {
    #[cfg(test)]
    fn success(latency_ms: u64, http_status: u16) -> Self {
        Self::success_with_shape(
            latency_ms,
            http_status,
            ProductionAccountSnapshotShapeSummary::accepted_fixture(),
        )
    }

    fn success_with_shape(
        latency_ms: u64,
        http_status: u16,
        response_shape_summary: ProductionAccountSnapshotShapeSummary,
    ) -> Self {
        Self {
            status: "online_account_snapshot_ok".to_string(),
            latency_ms: Some(latency_ms),
            http_status: Some(http_status),
            response_shape: production_account_snapshot_response_shape().to_string(),
            response_shape_validated: response_shape_summary.shape_validated,
            response_shape_summary,
            error_code: "none".to_string(),
            network_attempted: true,
            diagnostic: format!(
                "V120 authenticated production account snapshot read succeeded with GET {PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT} and HTTP {http_status}; raw account response, balances, uid, headers, signature, signed query, and signed URL were not recorded."
            ),
        }
    }

    fn failure(latency_ms: Option<u64>, http_status: Option<u16>, error_code: &str) -> Self {
        Self::failure_with_shape(
            latency_ms,
            http_status,
            error_code,
            ProductionAccountSnapshotShapeSummary::not_attempted(),
        )
    }

    fn failure_with_shape(
        latency_ms: Option<u64>,
        http_status: Option<u16>,
        error_code: &str,
        response_shape_summary: ProductionAccountSnapshotShapeSummary,
    ) -> Self {
        let status_detail = http_status
            .map(|status| format!(" HTTP {status}"))
            .unwrap_or_default();
        Self {
            status: "online_account_snapshot_failed".to_string(),
            latency_ms,
            http_status,
            response_shape: production_account_snapshot_response_shape().to_string(),
            response_shape_validated: response_shape_summary.shape_validated,
            response_shape_summary,
            error_code: error_code.to_string(),
            network_attempted: true,
            diagnostic: format!(
                "V120 authenticated production account snapshot read attempted GET {PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT} and failed with {error_code}.{status_detail} Raw account response, balances, uid, headers, signature, signed query, and signed URL were not recorded."
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionAccountSnapshotShapeSummary {
    status: String,
    account_type_present: bool,
    account_type_is_string: bool,
    balances_present: bool,
    balances_is_array: bool,
    balance_entry_count: Option<usize>,
    balance_entry_shape_validated: bool,
    permissions_present: bool,
    permissions_is_array: bool,
    permission_entry_count: Option<usize>,
    permission_entry_shape_validated: bool,
    can_trade_present: bool,
    can_trade_is_bool: bool,
    can_withdraw_present: bool,
    can_withdraw_is_bool: bool,
    can_deposit_present: bool,
    can_deposit_is_bool: bool,
    raw_account_response_recorded: bool,
    raw_balances_recorded: bool,
    raw_permissions_recorded: bool,
    shape_validated: bool,
    rejection_reason: String,
}

impl ProductionAccountSnapshotShapeSummary {
    fn not_attempted() -> Self {
        Self {
            status: "not_attempted".to_string(),
            account_type_present: false,
            account_type_is_string: false,
            balances_present: false,
            balances_is_array: false,
            balance_entry_count: None,
            balance_entry_shape_validated: false,
            permissions_present: false,
            permissions_is_array: false,
            permission_entry_count: None,
            permission_entry_shape_validated: false,
            can_trade_present: false,
            can_trade_is_bool: false,
            can_withdraw_present: false,
            can_withdraw_is_bool: false,
            can_deposit_present: false,
            can_deposit_is_bool: false,
            raw_account_response_recorded: false,
            raw_balances_recorded: false,
            raw_permissions_recorded: false,
            shape_validated: false,
            rejection_reason: "not_attempted".to_string(),
        }
    }

    #[cfg(test)]
    fn accepted_fixture() -> Self {
        Self {
            status: "accepted".to_string(),
            account_type_present: true,
            account_type_is_string: true,
            balances_present: true,
            balances_is_array: true,
            balance_entry_count: Some(1),
            balance_entry_shape_validated: true,
            permissions_present: true,
            permissions_is_array: true,
            permission_entry_count: Some(1),
            permission_entry_shape_validated: true,
            can_trade_present: true,
            can_trade_is_bool: true,
            can_withdraw_present: true,
            can_withdraw_is_bool: true,
            can_deposit_present: true,
            can_deposit_is_bool: true,
            raw_account_response_recorded: false,
            raw_balances_recorded: false,
            raw_permissions_recorded: false,
            shape_validated: true,
            rejection_reason: "none".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionShadowPortfolioRuntimeReport {
    schema_version: String,
    status: String,
    run_id: String,
    snapshot_id: String,
    snapshot_mode: String,
    created_at: String,
    source_account_snapshot_ref: ShadowSourceRef,
    source_shadow_intent_refs: Vec<ShadowIntentRef>,
    balances: ShadowBalancesSummary,
    positions: Vec<ShadowPositionSummary>,
    exposure: ShadowExposureSummary,
    pnl: ShadowPnlSummary,
    risk_summary: ShadowRiskSummary,
    provenance: ShadowPortfolioProvenance,
    shadow_intents_created: u64,
    actual_submission_count: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    full_production_portfolio_parity_claimed: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowSourceRef {
    path: String,
    schema_version: Option<String>,
    status: String,
    response_shape_validated: bool,
    raw_payload_recorded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowIntentRef {
    intent_id: String,
    symbol: Option<String>,
    venue: Option<String>,
    side: Option<String>,
    quantity: Option<String>,
    notional: Option<String>,
    submission_status: String,
    actual_submission: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowBalancesSummary {
    status: String,
    source: String,
    confidence: String,
    observed_balance_entry_count: Option<u64>,
    asset_values_recorded: bool,
    free_values_recorded: bool,
    locked_values_recorded: bool,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowPositionSummary {
    instrument_id: String,
    quantity: Option<String>,
    average_price: Option<String>,
    source: String,
    status: String,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowExposureSummary {
    asset: Option<String>,
    gross: Option<String>,
    net: Option<String>,
    notional: Option<String>,
    quote_currency: Option<String>,
    status: String,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowPnlSummary {
    realized: Option<String>,
    unrealized: Option<String>,
    quote_currency: Option<String>,
    status: String,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowRiskSummary {
    status: String,
    new_orders_blocked: bool,
    risk_halted: bool,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowPortfolioProvenance {
    account_snapshot_source: String,
    shadow_intent_source: String,
    balances_source: String,
    positions_source: String,
    exposure_source: String,
    pnl_source: String,
    values_are_exchange_truth: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ShadowIntentInputs {
    refs: Vec<ShadowIntentRef>,
    record_count: u64,
    notional_sum: Option<f64>,
    quote_currency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionShadowStrategySessionEvent {
    schema_version: String,
    run_id: String,
    session_id: String,
    strategy_id: String,
    event_type: String,
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    heartbeat_seq: Option<u64>,
    occurred_at: String,
    shadow_portfolio_runtime_ref: ShadowStrategyPortfolioRuntimeRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy_session_status_ref: Option<ShadowStrategySessionStatusRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_gap: Option<ShadowStrategyArtifactGap>,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    values_are_exchange_truth: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowStrategyPortfolioRuntimeRef {
    path: String,
    schema_version: String,
    status: String,
    snapshot_id: Option<String>,
    exposure_status: String,
    pnl_status: String,
    risk_status: String,
    shadow_intents_created: u64,
    network_attempted: bool,
    values_are_exchange_truth: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowStrategySessionStatusRef {
    path: String,
    schema_version: Option<String>,
    session_id: Option<String>,
    strategy_id: Option<String>,
    state: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShadowStrategyArtifactGap {
    path: Option<String>,
    status: String,
    required: bool,
    reason: String,
}

struct ShadowStrategyEventInput<'a> {
    opt: &'a LiveProductionShadowStrategySessionOpt,
    session_id: &'a str,
    event_type: &'a str,
    state: &'a str,
    heartbeat_seq: Option<u64>,
    portfolio_ref: &'a ShadowStrategyPortfolioRuntimeRef,
    session_status_ref: Option<ShadowStrategySessionStatusRef>,
    artifact_gap: Option<ShadowStrategyArtifactGap>,
    diagnostic: &'a str,
}

impl TestnetSignedOrderRequest {
    fn redacted_preview(
        &self,
        credentials: &EnvOnlyTestnetOrderCredentials,
    ) -> TestnetSignedOrderRequestPreview {
        TestnetSignedOrderRequestPreview {
            schema_version: TESTNET_ORDER_PREVIEW_SCHEMA_VERSION.to_string(),
            endpoint_class: "binance-testnet-signed-order-request-preview".to_string(),
            endpoint_url_redacted: self.endpoint_url_redacted.clone(),
            request_method: self.method.clone(),
            request_target: self.endpoint_path.clone(),
            query_shape: format!(
                "{}&signature=<redacted>",
                self.query_without_signature
            ),
            order_action: self.action.clone(),
            api_key_env: credentials.api_key_env.clone(),
            api_secret_env: credentials.api_secret_env.clone(),
            api_key_header_name: self.api_key_header_name.clone(),
            api_key_header_value_recorded: false,
            signature_recorded: false,
            signed_query_recorded: false,
            signed_url_recorded: false,
            request_body_recorded: false,
            order_submission: "request_preview_only".to_string(),
            order_submission_remains_disabled: true,
            network_attempted: false,
            real_orders_submitted: false,
            production_endpoint_allowed: false,
            dashboard_order_controls: false,
            secrets_redacted: true,
            diagnostic: "V100 signed Binance testnet order request layer built request metadata only; API key header value, signature, signed query, signed URL, and request body stay memory-only and redacted.".to_string(),
        }
    }

    fn ensure_preview_redacted(
        &self,
        credentials: &EnvOnlyTestnetOrderCredentials,
    ) -> anyhow::Result<()> {
        let preview = self.redacted_preview(credentials);
        let body = serde_json::to_string(&preview)?;
        credentials.ensure_no_secret_values_absent("signed-order-request-preview", &body)?;
        for (label, sensitive_value) in [
            ("signature", self.signature.as_str()),
            ("signed query", self.signed_query.as_str()),
            ("API key header value", self.api_key_header_value.as_str()),
        ] {
            if !sensitive_value.is_empty() && body.contains(sensitive_value) {
                anyhow::bail!("signed order request preview leaked {label}");
            }
        }
        Ok(())
    }
}

pub(crate) async fn run_live_command(opt: LiveOpt) -> anyhow::Result<()> {
    match opt.command {
        LiveCommand::Validate(validate) => run_live_validate(&validate),
        LiveCommand::Run(run) => run_live_run(&run).await,
        LiveCommand::TestnetOrderGate(gate) => run_live_testnet_order_gate(&gate),
        LiveCommand::TestnetOrderPreflight(preflight) => {
            run_live_testnet_order_preflight(&preflight)
        }
        LiveCommand::TestnetOrderRequestPreview(preview) => {
            run_live_testnet_order_request_preview(&preview)
        }
        LiveCommand::TestnetOrderTestPreflight(preflight) => {
            run_live_testnet_order_test_preflight(&preflight)
        }
        LiveCommand::TestnetExecutionArtifactContract(contract) => {
            run_live_testnet_execution_artifact_contract(&contract)
        }
        LiveCommand::TestnetReconciliationFixture(fixture) => {
            run_live_testnet_reconciliation_fixture(&fixture)
        }
        LiveCommand::ProductionPublicReadProbe(probe) => {
            run_live_production_public_read_probe(&probe)
        }
        LiveCommand::ProductionAccountSnapshotContract(contract) => {
            run_live_production_account_snapshot_contract(&contract)
        }
        LiveCommand::ProductionShadowPortfolioRuntime(runtime) => {
            run_live_production_shadow_portfolio_runtime(&runtime)
        }
        LiveCommand::ProductionShadowStrategySession(session) => {
            run_live_production_shadow_strategy_session(&session)
        }
    }
}

fn run_live_validate(opt: &LiveValidateOpt) -> anyhow::Result<()> {
    let config = load_minimal_live_config(&opt.config)?;

    println!(
        "live.validate status=ok mode={} run_id={} config={} environment={} node_name={} adapter={} external_venue_connection=false real_orders_submitted=false",
        config.run.mode,
        config.run.id,
        opt.config.display(),
        config.run.environment,
        node_name(&config),
        config.adapter.kind,
    );

    Ok(())
}

async fn run_live_run(opt: &LiveRunOpt) -> anyhow::Result<()> {
    run_live_run_with_command(
        opt,
        "live.run",
        ProcessMode::TestHarness,
        None,
        NtproNodeRunControls::default(),
    )
    .await
}

fn run_live_testnet_order_gate(opt: &LiveTestnetOrderGateOpt) -> anyhow::Result<()> {
    run_live_testnet_order_gate_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_testnet_order_preflight(opt: &LiveTestnetOrderPreflightOpt) -> anyhow::Result<()> {
    run_live_testnet_order_preflight_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_testnet_order_request_preview(
    opt: &LiveTestnetOrderRequestPreviewOpt,
) -> anyhow::Result<()> {
    run_live_testnet_order_request_preview_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_testnet_order_test_preflight(
    opt: &LiveTestnetOrderTestPreflightOpt,
) -> anyhow::Result<()> {
    run_live_testnet_order_test_preflight_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_testnet_execution_artifact_contract(
    opt: &LiveTestnetExecutionArtifactContractOpt,
) -> anyhow::Result<()> {
    run_live_testnet_execution_artifact_contract_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_testnet_reconciliation_fixture(
    opt: &LiveTestnetReconciliationFixtureOpt,
) -> anyhow::Result<()> {
    let config = load_strategy_node_config(&opt.config)?;
    let Some(testnet_order) = &config.testnet_order else {
        anyhow::bail!("testnet_order section is required for v0.10 reconciliation fixture");
    };

    let report = build_testnet_reconciliation_fixture_report(testnet_order, opt.scenario);
    if let Some(output) = &opt.output {
        atomic_write_json(output, &report)?;
    }

    println!(
        "live.testnet_reconciliation_fixture status={} config={} scenario={} scenario_count={} risk_halted=true new_orders_blocked=true order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false production_endpoint_allowed=false dashboard_order_controls=false",
        report.status,
        opt.config.display(),
        report.scenario,
        report.scenario_count,
    );
    Ok(())
}

fn run_live_production_public_read_probe(
    opt: &LiveProductionPublicReadProbeOpt,
) -> anyhow::Result<()> {
    run_live_production_public_read_probe_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_production_account_snapshot_contract(
    opt: &LiveProductionAccountSnapshotContractOpt,
) -> anyhow::Result<()> {
    run_live_production_account_snapshot_contract_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_production_shadow_portfolio_runtime(
    opt: &LiveProductionShadowPortfolioRuntimeOpt,
) -> anyhow::Result<()> {
    let report = build_production_shadow_portfolio_runtime_report(
        &opt.run_id,
        opt.snapshot_id.as_deref(),
        &opt.account_snapshot,
        &opt.shadow_intent,
    )?;
    atomic_write_json(&opt.output, &report).with_context(|| {
        format!(
            "failed to write shadow portfolio runtime '{}'",
            opt.output.display()
        )
    })?;

    if let Some(compat_output) = &opt.compat_snapshot_output {
        let compat_snapshot = build_production_shadow_portfolio_compat_snapshot(&report);
        atomic_write_json(compat_output, &compat_snapshot).with_context(|| {
            format!(
                "failed to write v0.11-compatible shadow portfolio snapshot '{}'",
                compat_output.display()
            )
        })?;
    }

    println!(
        "live.production_shadow_portfolio_runtime status={} run_id={} snapshot_id={} output={} compat_snapshot_output={} shadow_intents_created={} exposure_status={} pnl_status={} production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false full_production_portfolio_parity_claimed=false",
        report.status,
        report.run_id,
        report.snapshot_id,
        opt.output.display(),
        opt.compat_snapshot_output.as_ref().map_or_else(
            || "not_requested".to_string(),
            |path| path.display().to_string()
        ),
        report.shadow_intents_created,
        report.exposure.status,
        report.pnl.status,
    );
    Ok(())
}

fn run_live_production_shadow_strategy_session(
    opt: &LiveProductionShadowStrategySessionOpt,
) -> anyhow::Result<()> {
    let events = build_production_shadow_strategy_session_events(opt)?;
    write_production_shadow_strategy_session_events(&opt.output, &events)?;

    let final_state = events
        .last()
        .map_or("unknown", |event| event.state.as_str());
    let gap_count = events
        .iter()
        .filter(|event| event.artifact_gap.is_some())
        .count();
    let heartbeat_count = events
        .iter()
        .filter(|event| event.event_type == "shadow_strategy_session_heartbeat")
        .count();
    println!(
        "live.production_shadow_strategy_session status=ok run_id={} output={} events={} heartbeats={} artifact_gaps={} final_state={} production_order_submissions_attempted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false values_are_exchange_truth=false",
        opt.run_id,
        opt.output.display(),
        events.len(),
        heartbeat_count,
        gap_count,
        final_state,
    );
    Ok(())
}

fn run_live_production_public_read_probe_with_env<F>(
    opt: &LiveProductionPublicReadProbeOpt,
    mut read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    run_live_production_public_read_probe_with_env_and_http(
        opt,
        &mut read_env,
        execute_production_public_read_probe,
    )
}

fn run_live_production_public_read_probe_with_env_and_http<F, H>(
    opt: &LiveProductionPublicReadProbeOpt,
    read_env: &mut F,
    mut http_probe: H,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
    H: FnMut(ProductionPublicReadEndpoint, &str) -> ProductionPublicReadProbeHttpResult,
{
    let missing_cli_flags = missing_production_public_read_cli_flags(opt);
    let missing_env_vars = missing_production_public_read_env_gates(read_env, opt.manual_online);
    let should_attempt_online =
        should_attempt_production_public_read_probe(opt, &missing_cli_flags, &missing_env_vars);
    let (_, path) = production_public_read_endpoint_parts(opt.endpoint);
    let request_url = format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{path}");
    let http_result = should_attempt_online.then(|| http_probe(opt.endpoint, &request_url));
    let report = build_production_public_read_probe_report(
        opt.endpoint,
        opt.manual_online,
        &missing_cli_flags,
        &missing_env_vars,
        http_result.as_ref(),
    );

    if let Some(output) = &opt.output {
        atomic_write_json(output, &report)?;
    }

    println!(
        "live.production_public_read_probe status={} endpoint={} endpoint_class={} method={} path={} manual_online_requested={} contract_ready={} online_read_allowed={} online_execution_supported={} read_allowed={} mutation_allowed=false credentials_used=false network_attempted={} response_shape={} response_shape_validated={} error_code={} production_order_submission_attempted=false production_order_mutation_attempted=false dashboard_order_controls_enabled=false",
        report.status,
        report.endpoint,
        report.endpoint_class,
        report.method,
        report.path,
        report.manual_online_requested,
        report.contract_ready,
        report.online_read_allowed,
        report.online_execution_supported,
        report.read_allowed,
        report.network_attempted,
        report.response_shape,
        report.response_shape_validated,
        report.error_code,
    );
    Ok(())
}

fn build_production_public_read_probe_report(
    endpoint: ProductionPublicReadEndpoint,
    manual_online_requested: bool,
    missing_cli_flags: &[&'static str],
    missing_env_vars: &[&'static str],
    http_result: Option<&ProductionPublicReadProbeHttpResult>,
) -> ProductionPublicReadProbeReport {
    let (endpoint_name, path) = production_public_read_endpoint_parts(endpoint);
    let classified_endpoint = EndpointClassifier::classify(
        "GET",
        &format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{path}"),
        EndpointAuthKind::None,
    );
    let gates_missing = !missing_cli_flags.is_empty() || !missing_env_vars.is_empty();
    let status = if let Some(result) = http_result {
        result.status.as_str()
    } else if gates_missing && manual_online_requested {
        "blocked_missing_manual_online_gate"
    } else if gates_missing {
        "blocked_missing_gate"
    } else if manual_online_requested {
        "blocked_online_execution_not_attempted"
    } else {
        "ready_offline_contract"
    };
    let diagnostic = if let Some(result) = http_result {
        result.diagnostic.as_str()
    } else if gates_missing && manual_online_requested {
        "manual online production public read probe is closed because explicit v0.12 owner gates are missing"
    } else if gates_missing {
        "production public read probe is closed because explicit CLI/env gates are missing"
    } else if manual_online_requested {
        "manual online production read gates are present, but no HTTP probe result was produced"
    } else {
        "offline production public read-only probe contract is ready; no network was opened"
    };

    let contract_ready =
        !gates_missing && !manual_online_requested && classified_endpoint.read_allowed;
    let online_read_allowed =
        !gates_missing && manual_online_requested && classified_endpoint.read_allowed;
    let response_shape = http_result.map_or_else(
        || production_public_read_response_shape(endpoint).to_string(),
        |result| result.response_shape.clone(),
    );

    ProductionPublicReadProbeReport {
        schema_version: if manual_online_requested {
            PRODUCTION_PUBLIC_ONLINE_READ_PROBE_SCHEMA_VERSION
        } else {
            PRODUCTION_PUBLIC_READ_PROBE_SCHEMA_VERSION
        }
        .to_string(),
        status: status.to_string(),
        endpoint: endpoint_name.to_string(),
        endpoint_class: classified_endpoint.endpoint_class.as_str().to_string(),
        http_base_url: BINANCE_PRODUCTION_HTTP_BASE_URL.to_string(),
        method: classified_endpoint.method,
        path: classified_endpoint.path,
        request_url_redacted: classified_endpoint.input_url_redacted,
        requires_api_key: classified_endpoint.requires_api_key,
        requires_signature: classified_endpoint.requires_signature,
        read_allowed: contract_ready,
        contract_ready,
        online_read_allowed,
        mutation_allowed: classified_endpoint.mutation_allowed,
        manual_gate_required: true,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        missing_env_vars: missing_env_vars
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        manual_online_requested,
        online_execution_supported: manual_online_requested,
        network_attempted: http_result.is_some_and(|result| result.network_attempted),
        production_public_online_read_attempted: http_result
            .is_some_and(|result| result.network_attempted),
        response_status_code: http_result.and_then(|result| result.http_status),
        response_shape,
        response_shape_validated: http_result.is_some_and(|result| result.response_shape_validated),
        latency_ms: http_result.and_then(|result| result.latency_ms),
        error_code: http_result.map_or_else(
            || "not_attempted".to_string(),
            |result| result.error_code.clone(),
        ),
        credentials_used: false,
        account_mutation_attempted: false,
        production_order_submission_attempted: false,
        production_order_mutation_attempted: false,
        dashboard_order_controls_enabled: false,
        diagnostic: diagnostic.to_string(),
    }
}

fn production_public_read_endpoint_parts(
    endpoint: ProductionPublicReadEndpoint,
) -> (&'static str, &'static str) {
    match endpoint {
        ProductionPublicReadEndpoint::ServerTime => ("server_time", "/api/v3/time"),
        ProductionPublicReadEndpoint::ExchangeInfo => ("exchange_info", "/api/v3/exchangeInfo"),
    }
}

fn production_public_read_response_shape(endpoint: ProductionPublicReadEndpoint) -> &'static str {
    match endpoint {
        ProductionPublicReadEndpoint::ServerTime => "binance_server_time_v1",
        ProductionPublicReadEndpoint::ExchangeInfo => "binance_exchange_info_v1",
    }
}

fn should_attempt_production_public_read_probe(
    opt: &LiveProductionPublicReadProbeOpt,
    missing_cli_flags: &[&'static str],
    missing_env_vars: &[&'static str],
) -> bool {
    opt.manual_online && missing_cli_flags.is_empty() && missing_env_vars.is_empty()
}

fn execute_production_public_read_probe(
    endpoint: ProductionPublicReadEndpoint,
    request_url: &str,
) -> ProductionPublicReadProbeHttpResult {
    std::thread::spawn({
        let request_url = request_url.to_string();
        move || execute_production_public_read_probe_on_thread(endpoint, &request_url)
    })
    .join()
    .unwrap_or_else(|_| {
        ProductionPublicReadProbeHttpResult::failure(
            endpoint,
            None,
            None,
            "http_probe_thread_panicked",
        )
    })
}

fn execute_production_public_read_probe_on_thread(
    endpoint: ProductionPublicReadEndpoint,
    request_url: &str,
) -> ProductionPublicReadProbeHttpResult {
    let started = Instant::now();
    let client = match reqwest::blocking::Client::builder()
        .timeout(PRODUCTION_PUBLIC_READ_PROBE_TIMEOUT)
        .user_agent("NTPRO-v120-production-public-readonly-probe")
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return ProductionPublicReadProbeHttpResult::failure(
                endpoint,
                None,
                None,
                "http_client_build_failed",
            );
        }
    };

    match client.get(request_url).send() {
        Ok(response) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            let status = response.status().as_u16();
            if response.status().is_success() {
                match response.json::<serde_json::Value>() {
                    Ok(body)
                        if validates_production_public_read_response_shape(endpoint, &body) =>
                    {
                        ProductionPublicReadProbeHttpResult::success(endpoint, latency_ms, status)
                    }
                    Ok(_) | Err(_) => ProductionPublicReadProbeHttpResult::failure(
                        endpoint,
                        Some(latency_ms),
                        Some(status),
                        "response_shape_invalid",
                    ),
                }
            } else {
                ProductionPublicReadProbeHttpResult::failure(
                    endpoint,
                    Some(latency_ms),
                    Some(status),
                    "http_status_not_success",
                )
            }
        }
        Err(error) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            ProductionPublicReadProbeHttpResult::failure(
                endpoint,
                Some(latency_ms),
                error.status().map(|status| status.as_u16()),
                classify_production_public_read_error(&error),
            )
        }
    }
}

fn validates_production_public_read_response_shape(
    endpoint: ProductionPublicReadEndpoint,
    body: &serde_json::Value,
) -> bool {
    match endpoint {
        ProductionPublicReadEndpoint::ServerTime => {
            serde_json::from_value::<BinanceServerTimeResponse>(body.clone())
                .is_ok_and(|response| response.server_time > 0)
        }
        ProductionPublicReadEndpoint::ExchangeInfo => body.as_object().is_some_and(|object| {
            object
                .get("symbols")
                .is_some_and(serde_json::Value::is_array)
        }),
    }
}

fn classify_production_public_read_error(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect_error"
    } else if error.is_decode() {
        "decode_error"
    } else if error.is_request() {
        "request_error"
    } else if error.is_body() {
        "body_error"
    } else {
        "unknown_http_error"
    }
}

fn current_unix_timestamp_ms() -> anyhow::Result<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_millis();
    u64::try_from(millis).context("current UNIX timestamp milliseconds exceeds u64")
}

fn run_live_production_account_snapshot_contract_with_env<F>(
    opt: &LiveProductionAccountSnapshotContractOpt,
    mut read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    run_live_production_account_snapshot_contract_with_env_and_http(
        opt,
        &mut read_env,
        execute_production_account_snapshot_read,
    )
}

fn run_live_production_account_snapshot_contract_with_env_and_http<F, H>(
    opt: &LiveProductionAccountSnapshotContractOpt,
    read_env: &mut F,
    mut http_probe: H,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
    H: FnMut(&EnvOnlyProductionReadCredentials, u64) -> ProductionAccountSnapshotHttpResult,
{
    let missing_cli_flags = missing_production_account_snapshot_cli_flags(opt);
    let missing_env_vars =
        missing_production_account_snapshot_env_gates(read_env, opt.manual_online);
    let credentials = EnvOnlyProductionReadCredentials::from_values(
        opt.api_key_env.clone(),
        read_env(&opt.api_key_env),
        opt.api_secret_env.clone(),
        read_env(&opt.api_secret_env),
    );
    let should_attempt_online = should_attempt_production_account_snapshot_read(
        opt,
        &credentials,
        &missing_cli_flags,
        &missing_env_vars,
    );
    let http_result = should_attempt_online.then(|| http_probe(&credentials, opt.recv_window_ms));
    let report = build_production_account_snapshot_contract_report(
        opt,
        &credentials,
        &missing_cli_flags,
        &missing_env_vars,
        http_result.as_ref(),
    );

    if let Some(output) = &opt.output {
        write_production_account_snapshot_report(output, &report, &credentials)?;
    }

    println!(
        "live.production_account_snapshot_contract status={} endpoint_class={} method={} path={} manual_online_requested={} contract_ready={} online_read_allowed={} online_execution_supported={} read_allowed={} mutation_allowed=false env_credentials_only=true credentials_used={} network_attempted={} account_read_attempted={} account_mutation_attempted=false order_endpoint_access_attempted=false production_order_submission_attempted=false production_order_mutation_attempted=false dashboard_order_controls_enabled=false secrets_redacted=true response_shape={} response_shape_validated={} error_code={}",
        report.status,
        report.endpoint_class,
        report.method,
        report.path,
        report.manual_online_requested,
        report.contract_ready,
        report.online_read_allowed,
        report.online_execution_supported,
        report.read_allowed,
        report.api_key_present && report.api_secret_present,
        report.network_attempted,
        report.account_read_attempted,
        report.response_shape,
        report.response_shape_validated,
        report.error_code,
    );
    Ok(())
}

fn build_production_account_snapshot_contract_report(
    opt: &LiveProductionAccountSnapshotContractOpt,
    credentials: &EnvOnlyProductionReadCredentials,
    missing_cli_flags: &[&'static str],
    missing_env_vars: &[&'static str],
    http_result: Option<&ProductionAccountSnapshotHttpResult>,
) -> ProductionAccountSnapshotContractReport {
    let classified_endpoint = EndpointClassifier::classify(
        "GET",
        &format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT}"),
        EndpointAuthKind::Signed,
    );
    let gates_missing = !missing_cli_flags.is_empty() || !missing_env_vars.is_empty();
    let credentials_missing = !credentials.api_key_present() || !credentials.api_secret_present();
    let status = if let Some(result) = http_result {
        result.status.as_str()
    } else if gates_missing && opt.manual_online {
        "blocked_missing_manual_online_gate"
    } else if gates_missing {
        "blocked_missing_gate"
    } else if credentials_missing {
        "blocked_missing_credentials"
    } else {
        "ready_offline_contract"
    };
    let diagnostic = if let Some(result) = http_result {
        result.diagnostic.as_str()
    } else if gates_missing && opt.manual_online {
        "manual online authenticated production account snapshot is closed because explicit v0.12 owner gates are missing"
    } else if gates_missing {
        "authenticated production account snapshot is closed because explicit CLI/env gates are missing"
    } else if credentials_missing {
        "authenticated production account snapshot contract requires env-only API key and secret presence"
    } else {
        "offline authenticated production account snapshot contract is ready; no network was opened"
    };

    let contract_ready = !gates_missing
        && !credentials_missing
        && !opt.manual_online
        && classified_endpoint.read_allowed;
    let online_read_allowed = !gates_missing
        && !credentials_missing
        && opt.manual_online
        && classified_endpoint.read_allowed;

    ProductionAccountSnapshotContractReport {
        schema_version: if opt.manual_online {
            PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION
        } else {
            PRODUCTION_ACCOUNT_SNAPSHOT_SCHEMA_VERSION
        }
        .to_string(),
        status: status.to_string(),
        endpoint_class: classified_endpoint.endpoint_class.as_str().to_string(),
        http_base_url: BINANCE_PRODUCTION_HTTP_BASE_URL.to_string(),
        method: classified_endpoint.method,
        path: classified_endpoint.path,
        request_url_redacted: format!(
            "{BINANCE_PRODUCTION_HTTP_BASE_URL}{PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT}?timestamp=<redacted>&recvWindow={}&signature=<redacted>",
            opt.recv_window_ms,
        ),
        query_shape: format!(
            "timestamp=<redacted>&recvWindow={}&signature=<redacted>",
            opt.recv_window_ms,
        ),
        requires_api_key: classified_endpoint.requires_api_key,
        requires_signature: classified_endpoint.requires_signature,
        read_allowed: contract_ready,
        contract_ready,
        online_read_allowed,
        mutation_allowed: classified_endpoint.mutation_allowed,
        owner_gate_required: classified_endpoint.owner_gate_required,
        manual_gate_required: true,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        missing_env_vars: missing_env_vars
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        manual_online_requested: opt.manual_online,
        online_execution_supported: opt.manual_online,
        network_attempted: http_result.is_some_and(|result| result.network_attempted),
        response_status_code: http_result.and_then(|result| result.http_status),
        response_shape: http_result.map_or_else(
            || production_account_snapshot_response_shape().to_string(),
            |result| result.response_shape.clone(),
        ),
        response_shape_validated: http_result.is_some_and(|result| result.response_shape_validated),
        response_shape_summary: http_result.map_or_else(
            ProductionAccountSnapshotShapeSummary::not_attempted,
            |result| result.response_shape_summary.clone(),
        ),
        latency_ms: http_result.and_then(|result| result.latency_ms),
        error_code: http_result.map_or_else(
            || "not_attempted".to_string(),
            |result| result.error_code.clone(),
        ),
        env_credentials_only: true,
        api_key_env: credentials.api_key_env.clone(),
        api_secret_env: credentials.api_secret_env.clone(),
        api_key_present: credentials.api_key_present(),
        api_secret_present: credentials.api_secret_present(),
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        account_read_attempted: http_result.is_some_and(|result| result.network_attempted),
        account_mutation_attempted: false,
        order_endpoint_access_attempted: false,
        production_order_submission_attempted: false,
        production_order_mutation_attempted: false,
        dashboard_order_controls_enabled: false,
        secrets_redacted: true,
        diagnostic: diagnostic.to_string(),
    }
}

fn should_attempt_production_account_snapshot_read(
    opt: &LiveProductionAccountSnapshotContractOpt,
    credentials: &EnvOnlyProductionReadCredentials,
    missing_cli_flags: &[&'static str],
    missing_env_vars: &[&'static str],
) -> bool {
    opt.manual_online
        && missing_cli_flags.is_empty()
        && missing_env_vars.is_empty()
        && credentials.api_key_present()
        && credentials.api_secret_present()
}

fn build_production_account_snapshot_signed_request(
    credentials: &EnvOnlyProductionReadCredentials,
    timestamp_ms: u64,
    recv_window_ms: u64,
) -> anyhow::Result<ProductionAccountSnapshotSignedRequest> {
    if recv_window_ms == 0 {
        anyhow::bail!("production account snapshot recvWindow must be positive");
    }

    let classified_endpoint = EndpointClassifier::classify(
        "GET",
        &format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT}"),
        EndpointAuthKind::Signed,
    );
    if !classified_endpoint.read_allowed || classified_endpoint.mutation_allowed {
        anyhow::bail!(
            "production account snapshot allowlist rejected endpoint {}",
            classified_endpoint.path
        );
    }

    let signing_credential = credentials.signing_credential()?;
    let query_without_signature = format!("timestamp={timestamp_ms}&recvWindow={recv_window_ms}");
    let signature =
        urlencoding::encode(&signing_credential.sign(&query_without_signature)).into_owned();
    let signed_query = format!("{query_without_signature}&signature={signature}");
    let request = ProductionAccountSnapshotSignedRequest {
        method: "GET".to_string(),
        endpoint_path: PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT.to_string(),
        endpoint_url_redacted: format!(
            "{BINANCE_PRODUCTION_HTTP_BASE_URL}{PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT}"
        ),
        query_without_signature,
        signature,
        signed_query,
        api_key_header_name: BINANCE_API_KEY_HEADER.to_string(),
        api_key_header_value: signing_credential.api_key().to_string(),
    };
    request.ensure_redacted(credentials)?;
    Ok(request)
}

fn execute_production_account_snapshot_read(
    credentials: &EnvOnlyProductionReadCredentials,
    recv_window_ms: u64,
) -> ProductionAccountSnapshotHttpResult {
    match build_production_account_snapshot_signed_request(
        credentials,
        current_unix_timestamp_ms().unwrap_or(0),
        recv_window_ms,
    ) {
        Ok(request) => {
            let signed_url = request.signed_url_for_execution();
            let api_key_header_name = request.api_key_header_name;
            let api_key_header_value = request.api_key_header_value;
            std::thread::spawn(move || {
                execute_production_account_snapshot_read_on_thread(
                    &signed_url,
                    &api_key_header_name,
                    &api_key_header_value,
                )
            })
            .join()
            .unwrap_or_else(|_| {
                ProductionAccountSnapshotHttpResult::failure(
                    None,
                    None,
                    "http_probe_thread_panicked",
                )
            })
        }
        Err(_) => ProductionAccountSnapshotHttpResult::failure(
            None,
            None,
            "signed_request_builder_failed",
        ),
    }
}

fn execute_production_account_snapshot_read_on_thread(
    signed_url: &str,
    api_key_header_name: &str,
    api_key_header_value: &str,
) -> ProductionAccountSnapshotHttpResult {
    let started = Instant::now();
    let client = match reqwest::blocking::Client::builder()
        .timeout(PRODUCTION_ACCOUNT_SNAPSHOT_PROBE_TIMEOUT)
        .user_agent("NTPRO-v120-production-account-snapshot-readonly-probe")
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return ProductionAccountSnapshotHttpResult::failure(
                None,
                None,
                "http_client_build_failed",
            );
        }
    };

    match client
        .get(signed_url)
        .header(api_key_header_name, api_key_header_value)
        .send()
    {
        Ok(response) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            let status = response.status().as_u16();
            if response.status().is_success() {
                match response.json::<serde_json::Value>() {
                    Ok(body) => {
                        let shape_summary = summarize_production_account_snapshot_shape(&body);
                        if shape_summary.shape_validated {
                            ProductionAccountSnapshotHttpResult::success_with_shape(
                                latency_ms,
                                status,
                                shape_summary,
                            )
                        } else {
                            ProductionAccountSnapshotHttpResult::failure_with_shape(
                                Some(latency_ms),
                                Some(status),
                                "response_shape_invalid",
                                shape_summary,
                            )
                        }
                    }
                    Err(_) => ProductionAccountSnapshotHttpResult::failure(
                        Some(latency_ms),
                        Some(status),
                        "response_shape_invalid",
                    ),
                }
            } else {
                ProductionAccountSnapshotHttpResult::failure(
                    Some(latency_ms),
                    Some(status),
                    "http_status_not_success",
                )
            }
        }
        Err(error) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            ProductionAccountSnapshotHttpResult::failure(
                Some(latency_ms),
                error.status().map(|status| status.as_u16()),
                classify_production_public_read_error(&error),
            )
        }
    }
}

fn production_account_snapshot_response_shape() -> &'static str {
    "binance_account_snapshot_v1"
}

fn summarize_production_account_snapshot_shape(
    body: &serde_json::Value,
) -> ProductionAccountSnapshotShapeSummary {
    let Some(object) = body.as_object() else {
        return ProductionAccountSnapshotShapeSummary {
            status: "rejected".to_string(),
            rejection_reason: "root_not_object".to_string(),
            ..ProductionAccountSnapshotShapeSummary::not_attempted()
        };
    };

    let account_type_present = object.contains_key("accountType");
    let account_type_is_string = object
        .get("accountType")
        .is_some_and(serde_json::Value::is_string);
    let balances_present = object.contains_key("balances");
    let balances_array = object.get("balances").and_then(serde_json::Value::as_array);
    let balances_is_array = balances_array.is_some();
    let balance_entry_count = balances_array.map(Vec::len);
    let balance_entry_shape_validated = balances_array.is_some_and(|balances| {
        balances.iter().all(|entry| {
            entry.as_object().is_some_and(|entry| {
                entry.get("asset").is_some_and(serde_json::Value::is_string)
                    && entry.get("free").is_some_and(serde_json::Value::is_string)
                    && entry
                        .get("locked")
                        .is_some_and(serde_json::Value::is_string)
            })
        })
    });
    let permissions_present = object.contains_key("permissions");
    let permissions_array = object
        .get("permissions")
        .and_then(serde_json::Value::as_array);
    let permissions_is_array = permissions_array.is_some();
    let permission_entry_count = permissions_array.map(Vec::len);
    let permission_entry_shape_validated = permissions_array
        .is_some_and(|permissions| permissions.iter().all(serde_json::Value::is_string));
    let can_trade_present = object.contains_key("canTrade");
    let can_trade_is_bool = object
        .get("canTrade")
        .is_some_and(serde_json::Value::is_boolean);
    let can_withdraw_present = object.contains_key("canWithdraw");
    let can_withdraw_is_bool = object
        .get("canWithdraw")
        .is_some_and(serde_json::Value::is_boolean);
    let can_deposit_present = object.contains_key("canDeposit");
    let can_deposit_is_bool = object
        .get("canDeposit")
        .is_some_and(serde_json::Value::is_boolean);
    let shape_validated = account_type_is_string
        && balances_is_array
        && balance_entry_shape_validated
        && permissions_is_array
        && permission_entry_shape_validated
        && can_trade_is_bool
        && can_withdraw_is_bool
        && can_deposit_is_bool;

    ProductionAccountSnapshotShapeSummary {
        status: if shape_validated {
            "accepted"
        } else {
            "rejected"
        }
        .to_string(),
        account_type_present,
        account_type_is_string,
        balances_present,
        balances_is_array,
        balance_entry_count,
        balance_entry_shape_validated,
        permissions_present,
        permissions_is_array,
        permission_entry_count,
        permission_entry_shape_validated,
        can_trade_present,
        can_trade_is_bool,
        can_withdraw_present,
        can_withdraw_is_bool,
        can_deposit_present,
        can_deposit_is_bool,
        raw_account_response_recorded: false,
        raw_balances_recorded: false,
        raw_permissions_recorded: false,
        shape_validated,
        rejection_reason: if shape_validated {
            "none"
        } else {
            "missing_or_invalid_required_fields"
        }
        .to_string(),
    }
}

fn build_production_shadow_portfolio_runtime_report(
    run_id: &str,
    snapshot_id: Option<&str>,
    account_snapshot_path: &Path,
    shadow_intent_path: &Path,
) -> anyhow::Result<ProductionShadowPortfolioRuntimeReport> {
    validate_non_empty("run_id", run_id)?;
    let account_snapshot = read_json_artifact(account_snapshot_path, "account snapshot")?;
    ensure_account_snapshot_artifact_is_redacted(&account_snapshot)?;
    let intent_inputs = read_shadow_intent_inputs(shadow_intent_path)?;
    ensure_shadow_intents_are_readonly(&intent_inputs.refs)?;

    let account_status =
        json_string_value(&account_snapshot, "status").unwrap_or_else(|| "unknown".to_string());
    let account_schema = json_string_value(&account_snapshot, "schema_version");
    let account_shape_validated =
        json_bool_value(&account_snapshot, "response_shape_validated").unwrap_or(false);
    let shape_summary = account_snapshot.get("response_shape_summary");
    let balance_entry_count = shape_summary
        .and_then(|shape| json_u64_value(shape, "balance_entry_count"))
        .or_else(|| json_u64_value(&account_snapshot, "balance_entry_count"));
    let network_attempted =
        json_bool_value(&account_snapshot, "network_attempted").unwrap_or(false);
    let account_read_attempted =
        json_bool_value(&account_snapshot, "account_read_attempted").unwrap_or(false);

    let balances = if account_shape_validated {
        ShadowBalancesSummary {
            status: "observed_shape_only".to_string(),
            source: "redacted_production_account_snapshot_shape".to_string(),
            confidence: "observed_shape_only".to_string(),
            observed_balance_entry_count: balance_entry_count,
            asset_values_recorded: false,
            free_values_recorded: false,
            locked_values_recorded: false,
            reason: "production account response shape was validated, but asset names and balance values remain redacted".to_string(),
        }
    } else {
        ShadowBalancesSummary {
            status: "unavailable".to_string(),
            source: "redacted_production_account_snapshot_shape".to_string(),
            confidence: "unavailable".to_string(),
            observed_balance_entry_count: balance_entry_count,
            asset_values_recorded: false,
            free_values_recorded: false,
            locked_values_recorded: false,
            reason: format!(
                "production account snapshot shape is not validated; account_status={account_status}"
            ),
        }
    };

    let positions = build_shadow_positions(&intent_inputs.refs);
    let exposure = build_shadow_exposure(&intent_inputs);
    let pnl = ShadowPnlSummary {
        realized: None,
        unrealized: None,
        quote_currency: intent_inputs.quote_currency.clone(),
        status: "unavailable".to_string(),
        reason: "production fills, cost basis, and mark prices are not available in v0.12 read-only shadow runtime".to_string(),
    };
    let risk_summary = ShadowRiskSummary {
        status: "risk_halted".to_string(),
        new_orders_blocked: true,
        risk_halted: true,
        reason: "production shadow portfolio runtime is read-only evidence; it cannot unlock production orders".to_string(),
    };
    let snapshot_id =
        snapshot_id.map_or_else(|| format!("{run_id}-shadow-portfolio"), ToString::to_string);
    let status = if account_shape_validated {
        "ready_redacted_shadow_portfolio"
    } else {
        "degraded_account_snapshot_unavailable"
    };

    Ok(ProductionShadowPortfolioRuntimeReport {
        schema_version: PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION.to_string(),
        status: status.to_string(),
        run_id: run_id.to_string(),
        snapshot_id,
        snapshot_mode: "production_readonly_shadow".to_string(),
        created_at: now_millis(),
        source_account_snapshot_ref: ShadowSourceRef {
            path: account_snapshot_path.display().to_string(),
            schema_version: account_schema,
            status: account_status,
            response_shape_validated: account_shape_validated,
            raw_payload_recorded: false,
        },
        source_shadow_intent_refs: intent_inputs.refs.clone(),
        balances,
        positions,
        exposure,
        pnl,
        risk_summary,
        provenance: ShadowPortfolioProvenance {
            account_snapshot_source: "redacted_account_snapshot_summary".to_string(),
            shadow_intent_source: "local_shadow_execution_intent_jsonl".to_string(),
            balances_source: "redacted_shape_summary_only".to_string(),
            positions_source: "unavailable_without_production_fills".to_string(),
            exposure_source: "derived_from_local_shadow_intent_notional_only".to_string(),
            pnl_source: "unavailable_without_fills_cost_basis_and_mark_prices".to_string(),
            values_are_exchange_truth: false,
        },
        shadow_intents_created: intent_inputs.record_count,
        actual_submission_count: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        full_production_portfolio_parity_claimed: false,
        network_attempted,
        real_orders_submitted: false,
        diagnostic: format!(
            "V120 shadow portfolio runtime built from redacted account summary and local shadow intents only; account_read_attempted={account_read_attempted}; production orders and mutations remain zero."
        ),
    })
}

fn build_shadow_positions(intents: &[ShadowIntentRef]) -> Vec<ShadowPositionSummary> {
    if intents.is_empty() {
        return vec![ShadowPositionSummary {
            instrument_id: "unavailable".to_string(),
            quantity: None,
            average_price: None,
            source: "unavailable".to_string(),
            status: "unavailable".to_string(),
            reason: "no shadow execution intents were provided".to_string(),
        }];
    }

    intents
        .iter()
        .map(|intent| ShadowPositionSummary {
            instrument_id: intent
                .symbol
                .clone()
                .unwrap_or_else(|| "unavailable".to_string()),
            quantity: None,
            average_price: None,
            source: "local_shadow_execution_intent".to_string(),
            status: "unavailable".to_string(),
            reason: "shadow intents are not production fills, so exchange position quantity and average price are unavailable".to_string(),
        })
        .collect()
}

fn build_shadow_exposure(intent_inputs: &ShadowIntentInputs) -> ShadowExposureSummary {
    let Some(notional_sum) = intent_inputs.notional_sum else {
        return ShadowExposureSummary {
            asset: None,
            gross: None,
            net: None,
            notional: None,
            quote_currency: intent_inputs.quote_currency.clone(),
            status: "unavailable".to_string(),
            reason: "no parseable shadow intent notional was available".to_string(),
        };
    };
    let notional = format_decimal(notional_sum);
    ShadowExposureSummary {
        asset: None,
        gross: Some(notional.clone()),
        net: Some(notional.clone()),
        notional: Some(notional),
        quote_currency: intent_inputs.quote_currency.clone(),
        status: "derived_from_shadow_intents".to_string(),
        reason: "derived from local shadow intent notional only; this is not exchange-confirmed portfolio exposure".to_string(),
    }
}

fn read_json_artifact(path: &Path, label: &str) -> anyhow::Result<serde_json::Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {label} artifact '{}'", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {label} artifact '{}'", path.display()))
}

fn ensure_account_snapshot_artifact_is_redacted(value: &serde_json::Value) -> anyhow::Result<()> {
    if value.get("balances").is_some()
        || value.get("permissions").is_some()
        || value.get("raw_account_response").is_some()
        || value.get("raw_balances").is_some()
    {
        anyhow::bail!(
            "shadow portfolio runtime requires a redacted account summary, not raw account response fields"
        );
    }

    let shape_summary = value.get("response_shape_summary");
    for field in [
        "raw_account_response_recorded",
        "raw_balances_recorded",
        "raw_permissions_recorded",
    ] {
        if json_bool_value(value, field).unwrap_or(false)
            || shape_summary
                .and_then(|summary| json_bool_value(summary, field))
                .unwrap_or(false)
        {
            anyhow::bail!("shadow portfolio runtime rejected account artifact with {field}=true");
        }
    }

    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
    ] {
        if json_bool_value(value, field).unwrap_or(false) {
            anyhow::bail!("shadow portfolio runtime rejected account artifact with {field}=true");
        }
    }

    Ok(())
}

fn read_shadow_intent_inputs(path: &Path) -> anyhow::Result<ShadowIntentInputs> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read shadow intent '{}'", path.display()))?;
    let mut refs = Vec::new();
    let mut notional_sum = 0.0;
    let mut parsed_notional_count = 0_u64;
    let mut quote_currency = None;

    for (index, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line).with_context(|| {
            format!(
                "failed to parse shadow intent JSONL line {} in '{}'",
                index + 1,
                path.display()
            )
        })?;
        ensure_shadow_intent_value_is_readonly(&value)?;
        let symbol = json_string_value(&value, "symbol");
        if quote_currency.is_none() {
            quote_currency = symbol.as_deref().and_then(quote_currency_from_symbol);
        }
        let notional = json_string_value(&value, "notional");
        if let Some(parsed) = notional
            .as_deref()
            .and_then(|notional| parse_non_negative_decimal(notional).ok())
        {
            notional_sum += parsed;
            parsed_notional_count += 1;
        }
        refs.push(ShadowIntentRef {
            intent_id: json_string_value(&value, "intent_id")
                .unwrap_or_else(|| format!("line-{}", index + 1)),
            symbol,
            venue: json_string_value(&value, "venue"),
            side: json_string_value(&value, "side"),
            quantity: json_string_value(&value, "quantity"),
            notional,
            submission_status: json_string_value(&value, "submission_status")
                .unwrap_or_else(|| "unknown".to_string()),
            actual_submission: json_bool_value(&value, "actual_submission").unwrap_or(false),
        });
    }

    Ok(ShadowIntentInputs {
        record_count: u64::try_from(refs.len()).unwrap_or(u64::MAX),
        refs,
        notional_sum: (parsed_notional_count > 0).then_some(notional_sum),
        quote_currency,
    })
}

fn ensure_shadow_intents_are_readonly(intents: &[ShadowIntentRef]) -> anyhow::Result<()> {
    if intents.iter().any(|intent| intent.actual_submission) {
        anyhow::bail!(
            "shadow portfolio runtime rejected shadow intents with actual_submission=true"
        );
    }
    Ok(())
}

fn ensure_shadow_intent_value_is_readonly(value: &serde_json::Value) -> anyhow::Result<()> {
    for field in [
        "actual_submission",
        "execution_adapter_called",
        "order_endpoint_access_attempted",
        "production_order_mutation_attempted",
        "dashboard_order_controls_enabled",
    ] {
        if json_bool_value(value, field).unwrap_or(false) {
            anyhow::bail!("shadow portfolio runtime rejected shadow intent with {field}=true");
        }
    }
    Ok(())
}

fn build_production_shadow_portfolio_compat_snapshot(
    report: &ProductionShadowPortfolioRuntimeReport,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": PRODUCTION_SHADOW_PORTFOLIO_COMPAT_SCHEMA_VERSION,
        "run_id": &report.run_id,
        "snapshot_id": &report.snapshot_id,
        "snapshot_mode": &report.snapshot_mode,
        "source_account_snapshot_ref": &report.source_account_snapshot_ref.path,
        "source_shadow_intent_refs": report
            .source_shadow_intent_refs
            .iter()
            .map(|intent| intent.intent_id.clone())
            .collect::<Vec<_>>(),
        "balances": [
            {
                "asset": "redacted",
                "free": null,
                "locked": null,
                "source": &report.balances.source,
                "confidence": &report.balances.confidence,
                "status": &report.balances.status,
                "observed_balance_entry_count": report.balances.observed_balance_entry_count,
                "asset_values_recorded": false,
                "free_values_recorded": false,
                "locked_values_recorded": false,
                "reason": &report.balances.reason
            }
        ],
        "positions": &report.positions,
        "exposure": &report.exposure,
        "pnl": &report.pnl,
        "risk_summary": &report.risk_summary,
        "created_at": &report.created_at,
        "actual_submission_count": report.actual_submission_count,
        "production_orders_submitted": report.production_orders_submitted,
        "production_order_mutations_attempted": report.production_order_mutations_attempted,
        "automatic_correction_orders_submitted": report.automatic_correction_orders_submitted,
        "dashboard_order_controls_enabled": report.dashboard_order_controls_enabled,
        "full_production_portfolio_parity_claimed": report.full_production_portfolio_parity_claimed
    })
}

fn build_production_shadow_strategy_session_events(
    opt: &LiveProductionShadowStrategySessionOpt,
) -> anyhow::Result<Vec<ProductionShadowStrategySessionEvent>> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("strategy_id", &opt.strategy_id)?;
    if opt.heartbeat_count == 0 {
        anyhow::bail!("heartbeat_count must be greater than zero");
    }

    let session_id = opt
        .session_id
        .as_deref()
        .unwrap_or(opt.run_id.as_str())
        .to_string();
    validate_non_empty("session_id", &session_id)?;

    let portfolio_runtime =
        read_json_artifact(&opt.shadow_portfolio_runtime, "shadow portfolio runtime")?;
    ensure_shadow_portfolio_runtime_is_readonly(&portfolio_runtime)?;
    let portfolio_ref = build_shadow_strategy_portfolio_runtime_ref(
        &opt.shadow_portfolio_runtime,
        &portfolio_runtime,
    );
    let (session_status_ref, artifact_gap) =
        read_shadow_strategy_session_status_ref(opt.strategy_session_status.as_deref());
    let base_state = if artifact_gap.is_some() {
        "degraded_artifact_gap"
    } else {
        "running"
    };
    let mut events = Vec::new();
    events.push(build_shadow_strategy_session_event(ShadowStrategyEventInput {
        opt,
        session_id: &session_id,
        event_type: "shadow_strategy_session_started",
        state: base_state,
        heartbeat_seq: None,
        portfolio_ref: &portfolio_ref,
        session_status_ref: session_status_ref.clone(),
        artifact_gap: artifact_gap.clone(),
        diagnostic: "local persistent shadow strategy session started from read-only shadow artifacts",
    }));

    if let Some(gap) = artifact_gap.clone() {
        events.push(build_shadow_strategy_session_event(ShadowStrategyEventInput {
            opt,
            session_id: &session_id,
            event_type: "shadow_strategy_session_artifact_gap",
            state: "degraded_artifact_gap",
            heartbeat_seq: None,
            portfolio_ref: &portfolio_ref,
            session_status_ref: session_status_ref.clone(),
            artifact_gap: Some(gap),
            diagnostic: "optional strategy session status was unavailable; session remains local read-only evidence",
        }));
    }

    for heartbeat_seq in 1..=opt.heartbeat_count {
        events.push(build_shadow_strategy_session_event(ShadowStrategyEventInput {
            opt,
            session_id: &session_id,
            event_type: "shadow_strategy_session_heartbeat",
            state: base_state,
            heartbeat_seq: Some(heartbeat_seq),
            portfolio_ref: &portfolio_ref,
            session_status_ref: session_status_ref.clone(),
            artifact_gap: artifact_gap.clone(),
            diagnostic: "local persistent shadow strategy session heartbeat; no production mutation attempted",
        }));
    }

    let stop_file_requested = opt.stop_file.as_ref().is_some_and(|path| path.exists());
    if opt.stop_after_heartbeats || stop_file_requested {
        let diagnostic = if stop_file_requested {
            "local owner stop-file observed; session stopped without production mutation"
        } else {
            "local stop-after-heartbeats requested; session stopped without production mutation"
        };
        events.push(build_shadow_strategy_session_event(
            ShadowStrategyEventInput {
                opt,
                session_id: &session_id,
                event_type: "shadow_strategy_session_stopped",
                state: "stopped",
                heartbeat_seq: None,
                portfolio_ref: &portfolio_ref,
                session_status_ref,
                artifact_gap,
                diagnostic,
            },
        ));
    }

    Ok(events)
}

fn build_shadow_strategy_session_event(
    input: ShadowStrategyEventInput<'_>,
) -> ProductionShadowStrategySessionEvent {
    ProductionShadowStrategySessionEvent {
        schema_version: PRODUCTION_SHADOW_STRATEGY_SESSION_EVENT_SCHEMA_VERSION.to_string(),
        run_id: input.opt.run_id.clone(),
        session_id: input.session_id.to_string(),
        strategy_id: input.opt.strategy_id.clone(),
        event_type: input.event_type.to_string(),
        state: input.state.to_string(),
        heartbeat_seq: input.heartbeat_seq,
        occurred_at: now_millis(),
        shadow_portfolio_runtime_ref: input.portfolio_ref.clone(),
        strategy_session_status_ref: input.session_status_ref,
        artifact_gap: input.artifact_gap,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: false,
        values_are_exchange_truth: false,
        diagnostic: input.diagnostic.to_string(),
    }
}

fn write_production_shadow_strategy_session_events(
    path: &Path,
    events: &[ProductionShadowStrategySessionEvent],
) -> anyhow::Result<()> {
    if events.is_empty() {
        anyhow::bail!("shadow strategy session must write at least one event");
    }
    let mut body = String::new();
    for event in events {
        body.push_str(&serde_json::to_string(event)?);
        body.push('\n');
    }
    atomic_write_text(path, &body).with_context(|| {
        format!(
            "failed to write shadow strategy session events '{}'",
            path.display()
        )
    })
}

fn ensure_shadow_portfolio_runtime_is_readonly(value: &serde_json::Value) -> anyhow::Result<()> {
    if json_string_value(value, "schema_version").as_deref()
        != Some(PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION)
    {
        anyhow::bail!("shadow strategy session requires v0.12 shadow portfolio runtime input");
    }

    for field in [
        "actual_submission_count",
        "production_orders_submitted",
        "production_order_mutations_attempted",
        "automatic_correction_orders_submitted",
    ] {
        if json_u64_value(value, field).unwrap_or(0) != 0 {
            anyhow::bail!("shadow strategy session rejected portfolio runtime with {field} > 0");
        }
    }

    for field in [
        "dashboard_order_controls_enabled",
        "full_production_portfolio_parity_claimed",
        "real_orders_submitted",
    ] {
        if json_bool_value(value, field).unwrap_or(false) {
            anyhow::bail!("shadow strategy session rejected portfolio runtime with {field}=true");
        }
    }

    if value
        .pointer("/provenance/values_are_exchange_truth")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        anyhow::bail!("shadow strategy session rejected portfolio runtime claiming exchange truth");
    }

    Ok(())
}

fn build_shadow_strategy_portfolio_runtime_ref(
    path: &Path,
    value: &serde_json::Value,
) -> ShadowStrategyPortfolioRuntimeRef {
    ShadowStrategyPortfolioRuntimeRef {
        path: path.display().to_string(),
        schema_version: json_string_value(value, "schema_version")
            .unwrap_or_else(|| "unknown".to_string()),
        status: json_string_value(value, "status").unwrap_or_else(|| "unknown".to_string()),
        snapshot_id: json_string_value(value, "snapshot_id"),
        exposure_status: value
            .get("exposure")
            .and_then(|exposure| json_string_value(exposure, "status"))
            .unwrap_or_else(|| "unknown".to_string()),
        pnl_status: value
            .get("pnl")
            .and_then(|pnl| json_string_value(pnl, "status"))
            .unwrap_or_else(|| "unknown".to_string()),
        risk_status: value
            .get("risk_summary")
            .and_then(|risk| json_string_value(risk, "status"))
            .unwrap_or_else(|| "unknown".to_string()),
        shadow_intents_created: json_u64_value(value, "shadow_intents_created").unwrap_or(0),
        network_attempted: json_bool_value(value, "network_attempted").unwrap_or(false),
        values_are_exchange_truth: value
            .pointer("/provenance/values_are_exchange_truth")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    }
}

fn read_shadow_strategy_session_status_ref(
    path: Option<&Path>,
) -> (
    Option<ShadowStrategySessionStatusRef>,
    Option<ShadowStrategyArtifactGap>,
) {
    let Some(path) = path else {
        return (
            None,
            Some(ShadowStrategyArtifactGap {
                path: None,
                status: "not_provided".to_string(),
                required: false,
                reason: "strategy session status artifact was not provided; using shadow portfolio runtime only".to_string(),
            }),
        );
    };

    match read_json_artifact(path, "strategy session status") {
        Ok(value) => (
            Some(ShadowStrategySessionStatusRef {
                path: path.display().to_string(),
                schema_version: json_string_value(&value, "schema_version"),
                session_id: json_string_value(&value, "session_id"),
                strategy_id: json_string_value(&value, "strategy_id"),
                state: json_string_value(&value, "state"),
                reason: json_string_value(&value, "reason"),
            }),
            None,
        ),
        Err(error) => (
            None,
            Some(ShadowStrategyArtifactGap {
                path: Some(path.display().to_string()),
                status: "missing_or_unreadable".to_string(),
                required: false,
                reason: format!("strategy session status artifact unavailable: {error}"),
            }),
        ),
    }
}

fn json_string_value(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn json_bool_value(value: &serde_json::Value, field: &str) -> Option<bool> {
    value.get(field).and_then(serde_json::Value::as_bool)
}

fn json_u64_value(value: &serde_json::Value, field: &str) -> Option<u64> {
    value.get(field).and_then(serde_json::Value::as_u64)
}

fn parse_non_negative_decimal(value: &str) -> anyhow::Result<f64> {
    let value = value.trim();
    if value.is_empty() {
        anyhow::bail!("decimal value must not be empty");
    }
    if value.starts_with('-') {
        anyhow::bail!("decimal value must not be negative");
    }
    value
        .parse::<f64>()
        .ok()
        .filter(|parsed| parsed.is_finite())
        .context("decimal value must parse as finite f64")
}

fn format_decimal(value: f64) -> String {
    let formatted = format!("{value:.8}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn quote_currency_from_symbol(symbol: &str) -> Option<String> {
    let base = symbol.split_once('.').map_or(symbol, |(base, _)| base);
    ["USDT", "USDC", "USD", "BTC", "ETH"]
        .into_iter()
        .find(|quote| base.ends_with(quote))
        .map(ToString::to_string)
}

fn run_live_testnet_order_gate_with_env<F>(
    opt: &LiveTestnetOrderGateOpt,
    mut read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    let config = load_strategy_node_config(&opt.config)?;
    let Some(testnet_order) = &config.testnet_order else {
        anyhow::bail!("testnet_order section is required for v0.10 order gate");
    };

    let missing_cli_flags = missing_testnet_order_cli_flags(opt);
    let missing_env_vars = missing_testnet_order_env_gates(&mut read_env);
    if !missing_cli_flags.is_empty() || !missing_env_vars.is_empty() {
        anyhow::bail!(
            "testnet order gate blocked: missing_cli_flags={} missing_env_vars={} config_enabled={} order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false",
            join_gate_labels(&missing_cli_flags),
            join_gate_labels(&missing_env_vars),
            testnet_order.enabled,
        );
    }

    println!(
        "live.testnet_order_gate status=ready config={} symbol={} instrument_id={} manual_gate_ready=true order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false production_endpoint_allowed=false dashboard_order_controls=false",
        opt.config.display(),
        testnet_order.symbol,
        testnet_order.instrument_id,
    );
    Ok(())
}

fn run_live_testnet_order_request_preview_with_env<F>(
    opt: &LiveTestnetOrderRequestPreviewOpt,
    mut read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    let config = load_strategy_node_config(&opt.config)?;
    let Some(testnet_order) = &config.testnet_order else {
        anyhow::bail!("testnet_order section is required for v0.10 signed order request preview");
    };

    let missing_cli_flags = missing_testnet_order_request_preview_cli_flags(opt);
    let missing_env_vars = missing_testnet_order_env_gates(&mut read_env);
    if !missing_cli_flags.is_empty() || !missing_env_vars.is_empty() {
        anyhow::bail!(
            "testnet signed order request preview blocked: missing_cli_flags={} missing_env_vars={} request_built=false order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false",
            join_gate_labels(&missing_cli_flags),
            join_gate_labels(&missing_env_vars),
        );
    }

    let credentials = EnvOnlyTestnetOrderCredentials::from_values(
        opt.api_key_env.clone(),
        read_env(&opt.api_key_env),
        opt.api_secret_env.clone(),
        read_env(&opt.api_secret_env),
    );
    let request = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        &opt.method,
        &opt.endpoint_path,
        opt.timestamp_ms,
        opt.recv_window_ms,
        opt.orig_client_order_id.as_deref(),
    )?;
    let preview = request.redacted_preview(&credentials);
    if let Some(output) = &opt.output {
        write_secret_redacted_json(output, &preview, &credentials)?;
    }

    println!(
        "live.testnet_order_request_preview status=ready config={} method={} endpoint={} order_action={} order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false production_endpoint_allowed=false dashboard_order_controls=false secrets_redacted=true",
        opt.config.display(),
        preview.request_method,
        preview.request_target,
        preview.order_action,
    );
    Ok(())
}

fn run_live_testnet_order_test_preflight_with_env<F>(
    opt: &LiveTestnetOrderTestPreflightOpt,
    mut read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    let config = load_strategy_node_config(&opt.config)?;
    let Some(testnet_order) = &config.testnet_order else {
        anyhow::bail!("testnet_order section is required for v0.10 order-test preflight");
    };

    let missing_cli_flags = missing_testnet_order_test_preflight_cli_flags(opt);
    let missing_env_vars = missing_testnet_order_env_gates(&mut read_env);
    if !missing_cli_flags.is_empty() || !missing_env_vars.is_empty() {
        anyhow::bail!(
            "testnet order-test preflight blocked: missing_cli_flags={} missing_env_vars={} request_built=false matching_engine_submission=false order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false",
            join_gate_labels(&missing_cli_flags),
            join_gate_labels(&missing_env_vars),
        );
    }

    let credentials = EnvOnlyTestnetOrderCredentials::from_values(
        opt.api_key_env.clone(),
        read_env(&opt.api_key_env),
        opt.api_secret_env.clone(),
        read_env(&opt.api_secret_env),
    );
    let request = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_POST,
        TESTNET_ORDER_ENDPOINT_TEST,
        opt.timestamp_ms,
        opt.recv_window_ms,
        None,
    )?;
    let report = build_order_test_preflight_report(&request, &credentials);
    if let Some(output) = &opt.output {
        write_secret_redacted_json(output, &report, &credentials)?;
    }

    println!(
        "live.testnet_order_test_preflight status=ready config={} method={} endpoint={} binance_order_test_acceptance=not_attempted_offline_manual_only matching_engine_submission=false order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false production_endpoint_allowed=false dashboard_order_controls=false secrets_redacted=true",
        opt.config.display(),
        report.request_method,
        report.request_target,
    );
    Ok(())
}

fn run_live_testnet_execution_artifact_contract_with_env<F>(
    opt: &LiveTestnetExecutionArtifactContractOpt,
    mut read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    let config = load_strategy_node_config(&opt.config)?;
    let Some(testnet_order) = &config.testnet_order else {
        anyhow::bail!("testnet_order section is required for v0.10 execution artifact contract");
    };

    let missing_cli_flags = missing_testnet_execution_artifact_contract_cli_flags(opt);
    let missing_env_vars = missing_testnet_order_env_gates(&mut read_env);
    if !missing_cli_flags.is_empty() || !missing_env_vars.is_empty() {
        anyhow::bail!(
            "testnet execution artifact contract blocked: missing_cli_flags={} missing_env_vars={} artifact_built=false matching_engine_submission=false order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false",
            join_gate_labels(&missing_cli_flags),
            join_gate_labels(&missing_env_vars),
        );
    }

    let credentials = EnvOnlyTestnetOrderCredentials::from_values(
        opt.api_key_env.clone(),
        read_env(&opt.api_key_env),
        opt.api_secret_env.clone(),
        read_env(&opt.api_secret_env),
    );
    let order_test_request = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_POST,
        TESTNET_ORDER_ENDPOINT_TEST,
        opt.timestamp_ms,
        opt.recv_window_ms,
        None,
    )?;
    let submit_request = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_POST,
        TESTNET_ORDER_ENDPOINT_ORDER,
        opt.timestamp_ms,
        opt.recv_window_ms,
        None,
    )?;
    let cancel_request = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_DELETE,
        TESTNET_ORDER_ENDPOINT_ORDER,
        opt.timestamp_ms,
        opt.recv_window_ms,
        Some(&opt.orig_client_order_id),
    )?;
    let report = build_execution_artifact_contract_report(
        &order_test_request,
        &submit_request,
        &cancel_request,
        &credentials,
    );
    if let Some(output) = &opt.output {
        write_secret_redacted_json(output, &report, &credentials)?;
    }

    println!(
        "live.testnet_execution_artifact_contract status=ready config={} schema={} testnet_orders_submitted=0 production_orders_submitted=0 manual_submit_cancel_proof_observed=false matching_engine_submission=false order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false production_endpoint_allowed=false dashboard_order_controls=false secrets_redacted=true",
        opt.config.display(),
        report.schema_version,
    );
    Ok(())
}

fn run_live_testnet_order_preflight_with_env<F>(
    opt: &LiveTestnetOrderPreflightOpt,
    mut read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    let config = load_strategy_node_config(&opt.config)?;
    let Some(testnet_order) = &config.testnet_order else {
        anyhow::bail!("testnet_order section is required for v0.10 order preflight");
    };

    let missing_cli_flags = missing_testnet_order_preflight_cli_flags(opt);
    let missing_env_vars = missing_testnet_order_env_gates(&mut read_env);
    if !missing_cli_flags.is_empty() || !missing_env_vars.is_empty() {
        anyhow::bail!(
            "testnet order preflight blocked: missing_cli_flags={} missing_env_vars={} preflight_evaluated=false order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false",
            join_gate_labels(&missing_cli_flags),
            join_gate_labels(&missing_env_vars),
        );
    }

    let input = load_strategy_order_preflight_input(&opt.input)?;
    let report = evaluate_testnet_order_preflight(&config, testnet_order, &input);
    if let Some(output) = &opt.output {
        atomic_write_json(output, &report)?;
    }

    if !report.passed {
        anyhow::bail!(
            "testnet order preflight failed: reasons={} order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false",
            join_owned_gate_labels(&report.reasons),
        );
    }

    println!(
        "live.testnet_order_preflight status=pass config={} input={} symbol={} notional={} open_order_count={} observed_clock_skew_ms={} order_submission_remains_disabled=true network_attempted=false real_orders_submitted=false",
        opt.config.display(),
        opt.input.display(),
        report.symbol,
        report.notional,
        report.open_order_count,
        report.observed_clock_skew_ms,
    );
    Ok(())
}

pub(crate) async fn run_ntpro_node(
    config: PathBuf,
    run_id: Option<String>,
    output: Option<PathBuf>,
    stop_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    run_ntpro_node_with_controls(
        config,
        run_id,
        output,
        stop_file,
        NtproNodeRunControls::default(),
    )
    .await
}

pub(crate) async fn run_ntpro_node_with_controls(
    config: PathBuf,
    run_id: Option<String>,
    output: Option<PathBuf>,
    stop_file: Option<PathBuf>,
    controls: NtproNodeRunControls,
) -> anyhow::Result<()> {
    if is_strategy_session_node_config(&config)? {
        return run_strategy_session_node_with_command(
            &LiveRunOpt {
                config,
                run_id,
                output,
            },
            "ntpro-node.run",
            ProcessMode::SpawnedProcess,
            stop_file.as_deref(),
            controls,
        )
        .await;
    }

    run_live_run_with_command(
        &LiveRunOpt {
            config,
            run_id,
            output,
        },
        "ntpro-node.run",
        ProcessMode::SpawnedProcess,
        stop_file.as_deref(),
        controls,
    )
    .await
}

async fn run_live_run_with_command(
    opt: &LiveRunOpt,
    command_name: &str,
    process_mode: ProcessMode,
    stop_file: Option<&Path>,
    shutdown_controls: NtproNodeRunControls,
) -> anyhow::Result<()> {
    let config = load_minimal_live_config(&opt.config)?;
    let run_id = opt.run_id.as_deref().unwrap_or(config.run.id.as_str());
    validate_non_empty("run_id", run_id)?;

    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let summary_path = output_dir.join("summary.txt");
    let legacy_events_path = output_dir.join("events.log");
    let events_path = output_dir.join("logs").join("events.log");
    let status_path = output_dir.join("status.json");
    let metrics_path = output_dir.join("metrics.json");
    let stdout_log_path = output_dir.join("logs").join("stdout.log");
    let stderr_log_path = output_dir.join("logs").join("stderr.log");
    if let Some(parent) = events_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log dir '{}'", parent.display()))?;
    }

    let context = LiveRunContext {
        config: &config,
        config_path: &opt.config,
        run_id,
        output_dir: &output_dir,
        process_mode,
        status_path: &status_path,
        metrics_path: &metrics_path,
        stdout_log_path: &stdout_log_path,
        stderr_log_path: &stderr_log_path,
        events_log_path: &events_path,
        stop_file,
        shutdown_controls,
    };
    let smoke = run_live_init_smoke(&context).await?;
    let status = build_node_status(&context, &smoke);
    write_metrics(
        &metrics_path,
        &status,
        &context,
        NodeMetricCounts {
            uptime_ms: Some(smoke.uptime_ms),
            starts_total: 1,
            stops_total: 1,
            state_transitions_total: 2,
        },
    )?;

    let summary = format!(
        "command={command_name}\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\nenvironment={}\nnode_name={}\nprocess_mode={}\nadapter={}\naccount_id={}\nvenue={}\npre_start_state={}\nrunning_state={}\nfinal_state={}\naccount_cached={}\nstatus_artifact={}\nmetrics_artifact={}\nevents_log={}\nexternal_venue_connection=false\nreal_orders_submitted=false\nruntime_status=completed\nshutdown_reason={}\n",
        config.run.mode,
        opt.config.display(),
        config.run.environment,
        node_name(&config),
        process_mode_label(process_mode),
        config.adapter.kind,
        config.adapter.account_id,
        config.adapter.venue,
        smoke.pre_start_state,
        smoke.running_state,
        smoke.final_state,
        smoke.account_cached,
        status_path.display(),
        metrics_path.display(),
        events_path.display(),
        smoke.shutdown_reason.label(),
    );
    atomic_write_text(&summary_path, &summary)
        .with_context(|| format!("failed to write summary '{}'", summary_path.display()))?;

    let status_json = serde_json::to_string_pretty(&status)?;
    atomic_write_text(&status_path, &format!("{status_json}\n"))
        .with_context(|| format!("failed to write status '{}'", status_path.display()))?;

    let event_log = format!(
        "phase=validate_config status=ok\n\
         phase=build_node status=ok node_name={}\n\
         phase=register_adapter status=ok adapter={} venue={}\n\
         phase=start status=ok state={} account_cached={}\n\
         phase=shutdown_trigger status=ok reason={}\n\
         phase=stop status=ok state={} external_venue_connection=false real_orders_submitted=false\n",
        node_name(&config),
        config.adapter.kind,
        config.adapter.venue,
        smoke.running_state,
        smoke.account_cached,
        smoke.shutdown_reason.label(),
        smoke.final_state,
    );
    atomic_write_text(&events_path, &event_log)
        .with_context(|| format!("failed to write events '{}'", events_path.display()))?;
    atomic_write_text(&legacy_events_path, &event_log).with_context(|| {
        format!(
            "failed to write legacy events '{}'",
            legacy_events_path.display()
        )
    })?;

    println!(
        "{command_name} status=ok mode={} run_id={} config={} output={} summary={} events={} status_artifact={} metrics_artifact={} node_name={} adapter={} final_state={} external_venue_connection=false real_orders_submitted=false runtime_status=completed",
        config.run.mode,
        run_id,
        opt.config.display(),
        output_dir.display(),
        summary_path.display(),
        events_path.display(),
        status_path.display(),
        metrics_path.display(),
        node_name(&config),
        config.adapter.kind,
        smoke.final_state,
    );

    Ok(())
}

async fn run_strategy_session_node_with_command(
    opt: &LiveRunOpt,
    command_name: &str,
    process_mode: ProcessMode,
    stop_file: Option<&Path>,
    shutdown_controls: NtproNodeRunControls,
) -> anyhow::Result<()> {
    let config = load_strategy_node_config(&opt.config)?;
    let run_id = opt
        .run_id
        .as_deref()
        .unwrap_or(config.node.node_id.as_str());
    validate_non_empty("run_id", run_id)?;

    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let summary_path = output_dir.join("summary.txt");
    let legacy_events_path = output_dir.join("events.log");
    let events_path = output_dir.join("logs").join("events.log");
    let status_path = output_dir.join("status.json");
    let metrics_path = output_dir.join("metrics.json");
    let stdout_log_path = output_dir.join("logs").join("stdout.log");
    let stderr_log_path = output_dir.join("logs").join("stderr.log");
    if let Some(parent) = events_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log dir '{}'", parent.display()))?;
    }

    let started_at = now_millis();
    let started_instant = Instant::now();
    let symbol = config
        .market
        .symbols
        .first()
        .context("market.symbols must not be empty")?;
    let mut session = StrategySession::new(run_id, &config.strategy.strategy_id, &output_dir)?;
    session.set_risk_controls(StrategyRiskControls {
        kill_switch_enabled: config.risk.kill_switch_enabled,
        kill_switch_active: config.risk.kill_switch_active,
    });
    let bars = ema_cross_demo_fixture_bars(symbol);
    let runtime = session.run_ema_cross_demo(&bars)?;
    let counters = runtime.counters;

    let shutdown_reason = wait_for_strategy_shutdown_trigger(
        stop_file,
        shutdown_controls,
        &status_path,
        &metrics_path,
        &stdout_log_path,
        &stderr_log_path,
        &events_path,
        &opt.config,
        &output_dir,
        run_id,
        process_mode,
        &started_at,
        started_instant,
        counters,
    )
    .await?;
    session.stop_after_shutdown(shutdown_reason.label())?;

    let stopped_at = now_millis();
    let uptime_ms = millis_to_u64(started_instant.elapsed().as_millis());
    let status = build_strategy_node_status(
        &StrategyNodeStatusContext {
            config_path: &opt.config,
            output_dir: &output_dir,
            run_id,
            process_mode,
            started_at: &started_at,
            stopped_at: Some(&stopped_at),
            counters,
        },
        NodeState::Stopped,
    );
    atomic_write_json(&status_path, &status)
        .with_context(|| format!("failed to write status '{}'", status_path.display()))?;
    write_strategy_node_metrics(
        &metrics_path,
        &status,
        &StrategyNodeMetricPaths {
            status_path: &status_path,
            stdout_log_path: &stdout_log_path,
            stderr_log_path: &stderr_log_path,
            events_log_path: &events_path,
        },
        NodeMetricCounts {
            uptime_ms: Some(uptime_ms),
            starts_total: 1,
            stops_total: 1,
            state_transitions_total: 2,
        },
    )?;

    let strategy_summary_path = runtime.summary_artifact.clone();
    let summary = format!(
        "command={command_name}\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\nenvironment=sandbox\nprocess_mode={}\nstrategy_id={}\nstrategy_runtime={}\nmarket_source={}\nmarket_symbol={symbol}\nprocessed_events={}\nmarket_event_count={}\nsignal_count={}\norder_intent_count={}\nrisk_decision_count={}\nrejection_count={}\nactual_submission_count={}\norder_submission_allowed=false\nstatus_artifact={}\nmetrics_artifact={}\nevents_log={}\nsession_status_artifact={}\nsignal_artifact={}\norder_intent_artifact={}\nrisk_decision_artifact={}\nstrategy_summary_artifact={}\nfinal_state=Stopped\nexternal_venue_connection=false\nreal_orders_submitted=false\nruntime_status=completed\nshutdown_reason={}\n",
        config.node.mode,
        opt.config.display(),
        process_mode_label(process_mode),
        config.strategy.strategy_id,
        config
            .strategy
            .strategy_runtime
            .as_deref()
            .unwrap_or(EMA_CROSS_DEMO_STRATEGY),
        config
            .market
            .data_mode
            .as_deref()
            .unwrap_or(FIXTURE_STREAM_DATA_MODE),
        runtime.processed_events,
        counters.market_event_count,
        runtime.signal_count,
        runtime.order_intent_count,
        runtime.risk_decision_count,
        counters.rejection_count,
        counters.actual_submission_count,
        status_path.display(),
        metrics_path.display(),
        events_path.display(),
        session.status().artifacts.session_status,
        runtime.signal_artifact,
        runtime.order_intent_artifact,
        runtime.risk_decision_artifact,
        strategy_summary_path,
        shutdown_reason.label(),
    );
    atomic_write_text(&summary_path, &summary)
        .with_context(|| format!("failed to write summary '{}'", summary_path.display()))?;

    let event_log = format!(
        "phase=validate_config status=ok mode={} strategy_id={}\n\
         phase=strategy_session_start status=ok session_id={run_id} strategy_id={}\n\
         phase=fixture_market_stream status=ok symbol={symbol} processed_events={}\n\
         phase=strategy_loop status=ok signal_count={} order_intent_count={} risk_decision_count={} rejection_count={} actual_submission_count={}\n\
         phase=shutdown_trigger status=ok reason={}\n\
         phase=strategy_session_stop status=ok state=stopped external_venue_connection=false real_orders_submitted=false\n",
        config.node.mode,
        config.strategy.strategy_id,
        config.strategy.strategy_id,
        runtime.processed_events,
        runtime.signal_count,
        runtime.order_intent_count,
        runtime.risk_decision_count,
        counters.rejection_count,
        counters.actual_submission_count,
        shutdown_reason.label(),
    );
    atomic_write_text(&events_path, &event_log)
        .with_context(|| format!("failed to write events '{}'", events_path.display()))?;
    atomic_write_text(&legacy_events_path, &event_log).with_context(|| {
        format!(
            "failed to write legacy events '{}'",
            legacy_events_path.display()
        )
    })?;

    println!(
        "{command_name} status=ok mode={} run_id={} config={} output={} summary={} events={} status_artifact={} metrics_artifact={} strategy_id={} final_state=Stopped external_venue_connection=false real_orders_submitted=false runtime_status=completed",
        config.node.mode,
        run_id,
        opt.config.display(),
        output_dir.display(),
        summary_path.display(),
        events_path.display(),
        status_path.display(),
        metrics_path.display(),
        config.strategy.strategy_id,
    );

    Ok(())
}

pub(crate) fn validate_minimal_live_config_file(path: &Path) -> anyhow::Result<()> {
    load_minimal_live_config(path)?;
    Ok(())
}

pub(crate) fn validate_strategy_node_config_file(path: &Path) -> anyhow::Result<()> {
    load_strategy_node_config(path)?;
    Ok(())
}

fn load_minimal_live_config(path: &Path) -> anyhow::Result<MinimalLiveConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read live config '{}'", path.display()))?;
    let config: MinimalLiveConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse live config '{}'", path.display()))?;
    validate_minimal_live_config(&config)?;
    Ok(config)
}

fn validate_minimal_live_config(config: &MinimalLiveConfig) -> anyhow::Result<()> {
    validate_non_empty("run.id", &config.run.id)?;
    validate_exact("run.mode", &config.run.mode, LIVE_INIT_SMOKE_MODE)?;
    validate_exact(
        "run.environment",
        &config.run.environment,
        SANDBOX_ENVIRONMENT,
    )?;
    validate_non_empty("system.trader_id", &config.system.trader_id)?;
    if config
        .system
        .node_name
        .as_ref()
        .or(config.system.instance_id.as_ref())
        .is_none_or(|value| value.trim().is_empty())
    {
        anyhow::bail!("system.node_name or system.instance_id must be set");
    }
    validate_non_empty("adapter.name", &config.adapter.name)?;
    validate_exact(
        "adapter.kind",
        &config.adapter.kind,
        SANDBOX_SIMULATED_EXECUTION,
    )?;
    validate_non_empty("adapter.account_id", &config.adapter.account_id)?;
    validate_non_empty("adapter.venue", &config.adapter.venue)?;
    if config.adapter.starting_balances.is_empty() {
        anyhow::bail!("adapter.starting_balances must not be empty");
    }
    for balance in &config.adapter.starting_balances {
        validate_non_empty("adapter.starting_balances", balance)?;
    }
    validate_exact(
        "execution.order_submission",
        &config.execution.order_submission,
        DISABLED_ORDER_SUBMISSION,
    )?;
    if config.execution.reconciliation {
        anyhow::bail!("execution.reconciliation must be false for live-init-smoke");
    }
    if config.execution.external_venue_connection {
        anyhow::bail!("execution.external_venue_connection must be false for live-init-smoke");
    }
    validate_exact("shutdown.mode", &config.shutdown.mode, START_STOP_SHUTDOWN)?;
    if config.shutdown.connection_timeout_secs == 0 {
        anyhow::bail!("shutdown.connection_timeout_secs must be greater than zero");
    }
    if config.shutdown.disconnection_timeout_secs == 0 {
        anyhow::bail!("shutdown.disconnection_timeout_secs must be greater than zero");
    }
    if let Some(output) = &config.output {
        if let Some(dir) = &output.dir {
            validate_non_empty("output.dir", dir.to_string_lossy().as_ref())?;
        }
        if matches!(output.write_summary, Some(false)) {
            anyhow::bail!("output.write_summary must be true for live-init-smoke");
        }
    }
    Ok(())
}

fn is_strategy_session_node_config(path: &Path) -> anyhow::Result<bool> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read ntpro-node config '{}'", path.display()))?;
    let value: toml::Value = toml::from_str(&raw)
        .with_context(|| format!("failed to parse ntpro-node config '{}'", path.display()))?;
    Ok(value.get("node").is_some())
}

fn load_strategy_node_config(path: &Path) -> anyhow::Result<StrategyNodeConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read strategy node config '{}'", path.display()))?;
    let config: StrategyNodeConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse strategy node config '{}'", path.display()))?;
    validate_strategy_node_config(&config)?;
    Ok(config)
}

fn load_strategy_order_preflight_input(path: &Path) -> anyhow::Result<StrategyOrderPreflightInput> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read order preflight input '{}'", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse order preflight input '{}'", path.display()))
}

fn validate_strategy_node_config(config: &StrategyNodeConfig) -> anyhow::Result<()> {
    validate_non_empty("node.node_id", &config.node.node_id)?;
    validate_exact("node.mode", &config.node.mode, STRATEGY_SESSION_SHADOW_MODE)?;
    validate_non_empty("strategy.strategy_id", &config.strategy.strategy_id)?;
    if let Some(package) = &config.strategy.strategy_package {
        validate_exact(
            "strategy.strategy_package",
            package,
            BUILTIN_STRATEGY_PACKAGE,
        )?;
    }
    if let Some(runtime) = &config.strategy.strategy_runtime {
        validate_exact(
            "strategy.strategy_runtime",
            runtime,
            EMA_CROSS_DEMO_STRATEGY,
        )?;
    }
    if config.market.symbols.is_empty() {
        anyhow::bail!("market.symbols must not be empty");
    }
    if config.market.symbols.len() != 1 {
        anyhow::bail!(
            "market.symbols must contain exactly one symbol for v0.9.1 strategy sessions, got {}",
            config.market.symbols.len()
        );
    }
    for symbol in &config.market.symbols {
        validate_non_empty("market.symbols", symbol)?;
    }
    if let Some(venue) = &config.market.venue {
        validate_non_empty("market.venue", venue)?;
    }
    if let Some(data_mode) = &config.market.data_mode {
        validate_exact("market.data_mode", data_mode, FIXTURE_STREAM_DATA_MODE)?;
    }
    if let Some(venue) = &config.execution.venue {
        validate_non_empty("execution.venue", venue)?;
    }
    validate_exact(
        "execution.order_submission",
        &config.execution.order_submission,
        DISABLED_ORDER_SUBMISSION,
    )?;
    if config.execution.external_venue_connection.unwrap_or(false) {
        anyhow::bail!("execution.external_venue_connection must be false for strategy session");
    }
    if let Some(testnet_order) = &config.testnet_order {
        validate_strategy_node_testnet_order_config(
            testnet_order,
            config
                .market
                .symbols
                .first()
                .map(String::as_str)
                .unwrap_or_default(),
        )?;
    }
    if !config.risk.kill_switch_enabled {
        anyhow::bail!("risk.kill_switch_enabled must be true for v0.9.1 shadow strategy sessions");
    }
    if let Some(shutdown) = &config.shutdown {
        validate_exact("shutdown.mode", &shutdown.mode, START_STOP_SHUTDOWN)?;
        if shutdown.connection_timeout_secs == 0 {
            anyhow::bail!("shutdown.connection_timeout_secs must be greater than zero");
        }
        if shutdown.disconnection_timeout_secs == 0 {
            anyhow::bail!("shutdown.disconnection_timeout_secs must be greater than zero");
        }
    }
    if let Some(output) = &config.output {
        if let Some(dir) = &output.dir {
            validate_non_empty("output.dir", dir.to_string_lossy().as_ref())?;
        }
        if matches!(output.write_summary, Some(false)) {
            anyhow::bail!("output.write_summary must be true for strategy session");
        }
    }
    Ok(())
}

fn validate_strategy_node_testnet_order_config(
    config: &StrategyNodeTestnetOrderSection,
    market_symbol: &str,
) -> anyhow::Result<()> {
    if config.enabled {
        anyhow::bail!("testnet_order.enabled must be false until explicit v0.10 manual gates run");
    }
    validate_exact(
        "testnet_order.mode",
        &config.mode,
        TESTNET_ORDER_DISABLED_MODE,
    )?;
    validate_exact(
        "testnet_order.manual_gate",
        &config.manual_gate,
        TESTNET_ORDER_OWNER_MANUAL_GATE,
    )?;
    validate_exact(
        "testnet_order.http_base_url",
        &config.http_base_url,
        BINANCE_TESTNET_HTTP_BASE_URL,
    )?;
    validate_non_empty("testnet_order.symbol", &config.symbol)?;
    validate_non_empty("testnet_order.instrument_id", &config.instrument_id)?;
    validate_exact(
        "testnet_order.instrument_id",
        &config.instrument_id,
        market_symbol,
    )?;
    validate_exact(
        "testnet_order.symbol",
        &config.symbol,
        market_symbol_base(market_symbol),
    )?;
    validate_non_empty("testnet_order.side", &config.side)?;
    if config.side != "BUY" && config.side != "SELL" {
        anyhow::bail!(
            "testnet_order.side must be 'BUY' or 'SELL', got '{}'",
            config.side
        );
    }
    validate_exact(
        "testnet_order.order_type",
        &config.order_type,
        TESTNET_ORDER_LIMIT_TYPE,
    )?;
    validate_exact(
        "testnet_order.time_in_force",
        &config.time_in_force,
        TESTNET_ORDER_GTC_TIF,
    )?;
    validate_positive_decimal_string("testnet_order.price", &config.price)?;
    validate_positive_decimal_string("testnet_order.quantity", &config.quantity)?;
    validate_positive_decimal_string("testnet_order.notional", &config.notional)?;
    if config.cancel_after_submit_ms == 0 {
        anyhow::bail!("testnet_order.cancel_after_submit_ms must be greater than zero");
    }
    if !config.owner_approval_required {
        anyhow::bail!("testnet_order.owner_approval_required must be true");
    }
    if !config.manual_env_gate_required {
        anyhow::bail!("testnet_order.manual_env_gate_required must be true");
    }
    if config.production_endpoint_allowed {
        anyhow::bail!("testnet_order.production_endpoint_allowed must be false");
    }
    if config.dashboard_order_controls {
        anyhow::bail!("testnet_order.dashboard_order_controls must be false");
    }
    Ok(())
}

fn market_symbol_base(symbol: &str) -> &str {
    symbol.split_once('.').map_or(symbol, |(base, _)| base)
}

fn validate_positive_decimal_string(field: &str, value: &str) -> anyhow::Result<()> {
    let value = value.trim();
    if value.is_empty() {
        anyhow::bail!("{field} must not be empty");
    }
    let mut saw_digit = false;
    let mut saw_non_zero_digit = false;
    let mut saw_dot = false;
    for character in value.chars() {
        match character {
            '0'..='9' => {
                saw_digit = true;
                saw_non_zero_digit |= character != '0';
            }
            '.' if !saw_dot => saw_dot = true,
            _ => anyhow::bail!("{field} must be a positive decimal string, got '{value}'"),
        }
    }
    if !saw_digit || !saw_non_zero_digit {
        anyhow::bail!("{field} must be greater than zero");
    }
    Ok(())
}

fn evaluate_testnet_order_preflight(
    config: &StrategyNodeConfig,
    testnet_order: &StrategyNodeTestnetOrderSection,
    input: &StrategyOrderPreflightInput,
) -> TestnetOrderPreflightReport {
    let mut reasons = Vec::new();
    if input.schema_version != STRATEGY_ORDER_PREFLIGHT_SCHEMA_VERSION {
        reasons.push("schema_version_mismatch".to_string());
    }
    if input.session.state != "running" {
        reasons.push("session_not_running".to_string());
    }
    if input.market.symbol != testnet_order.instrument_id {
        reasons.push("market_symbol_mismatch".to_string());
    }
    let market_age_ms = if input.market.now_unix_ms >= input.market.last_event_at_unix_ms {
        Some(input.market.now_unix_ms - input.market.last_event_at_unix_ms)
    } else {
        reasons.push("market_event_in_future".to_string());
        None
    };
    if market_age_ms.is_some_and(|age| age > input.market.max_age_ms) {
        reasons.push("market_stale".to_string());
    }
    if !input.account.readable {
        reasons.push("account_unreadable".to_string());
    }
    if input.account.readable && input.account.account_id.trim().is_empty() {
        reasons.push("account_id_missing".to_string());
    }
    if config.risk.kill_switch_active || input.risk.kill_switch_active {
        reasons.push("kill_switch_active".to_string());
    }
    if !input
        .risk
        .allowed_symbols
        .iter()
        .any(|symbol| symbol == &testnet_order.instrument_id)
    {
        reasons.push("symbol_not_allowlisted".to_string());
    }
    match decimal_string_to_f64(
        "limits.max_order_notional",
        &input.limits.max_order_notional,
    ) {
        Ok(max_order_notional) => {
            match decimal_string_to_f64("testnet_order.notional", &testnet_order.notional) {
                Ok(order_notional) if order_notional > max_order_notional => {
                    reasons.push("notional_limit_exceeded".to_string());
                }
                Ok(_) => {}
                Err(reason) => reasons.push(reason),
            }
        }
        Err(reason) => reasons.push(reason),
    }
    if input.limits.open_order_count >= input.limits.max_open_orders {
        reasons.push("open_order_limit_exceeded".to_string());
    }
    if input.limits.observed_clock_skew_ms > input.limits.max_clock_skew_ms {
        reasons.push("clock_skew_limit_exceeded".to_string());
    }
    if input.endpoint.http_base_url != BINANCE_TESTNET_HTTP_BASE_URL
        || input.endpoint.http_base_url != testnet_order.http_base_url
    {
        reasons.push("endpoint_not_testnet".to_string());
    }
    if input.endpoint.production_endpoint_allowed || testnet_order.production_endpoint_allowed {
        reasons.push("production_endpoint_allowed".to_string());
    }
    if testnet_order.dashboard_order_controls {
        reasons.push("dashboard_order_controls_enabled".to_string());
    }

    let passed = reasons.is_empty();
    TestnetOrderPreflightReport {
        schema_version: "ntpro.v100_order_preflight_report.v1".to_string(),
        status: if passed { "pass" } else { "fail" }.to_string(),
        passed,
        reasons,
        symbol: testnet_order.instrument_id.clone(),
        account_id: input.account.account_id.clone(),
        notional: testnet_order.notional.clone(),
        max_order_notional: input.limits.max_order_notional.clone(),
        open_order_count: input.limits.open_order_count,
        max_open_orders: input.limits.max_open_orders,
        observed_clock_skew_ms: input.limits.observed_clock_skew_ms,
        max_clock_skew_ms: input.limits.max_clock_skew_ms,
        market_age_ms,
        max_market_age_ms: input.market.max_age_ms,
        order_submission_remains_disabled: true,
        network_attempted: false,
        real_orders_submitted: false,
        production_endpoint_allowed: false,
        dashboard_order_controls: false,
    }
}

fn build_testnet_signed_order_request(
    testnet_order: &StrategyNodeTestnetOrderSection,
    credentials: &EnvOnlyTestnetOrderCredentials,
    method: &str,
    endpoint_path: &str,
    timestamp_ms: u64,
    recv_window_ms: u64,
    orig_client_order_id: Option<&str>,
) -> anyhow::Result<TestnetSignedOrderRequest> {
    validate_exact(
        "testnet_order.http_base_url",
        &testnet_order.http_base_url,
        BINANCE_TESTNET_HTTP_BASE_URL,
    )?;
    if recv_window_ms == 0 {
        anyhow::bail!("signed order request preview recvWindow must be positive");
    }
    if timestamp_ms == 0 {
        anyhow::bail!("signed order request preview timestamp_ms must be positive");
    }

    let method = normalize_testnet_order_method(method)?;
    let endpoint_path = normalize_testnet_order_endpoint_path(endpoint_path)?;
    let action = ensure_testnet_signed_order_request_allowed(&method, &endpoint_path)?;
    let signing_credential = credentials.signing_credential()?;
    let query_without_signature = build_testnet_signed_order_query(
        testnet_order,
        &method,
        &endpoint_path,
        timestamp_ms,
        recv_window_ms,
        orig_client_order_id,
    )?;
    let signature =
        urlencoding::encode(&signing_credential.sign(&query_without_signature)).into_owned();
    let signed_query = format!("{query_without_signature}&signature={signature}");
    let request = TestnetSignedOrderRequest {
        method,
        endpoint_path: endpoint_path.clone(),
        endpoint_url_redacted: format!(
            "{}{}",
            testnet_order.http_base_url.trim_end_matches('/'),
            endpoint_path,
        ),
        query_without_signature,
        signature,
        signed_query,
        api_key_header_name: BINANCE_API_KEY_HEADER.to_string(),
        api_key_header_value: signing_credential.api_key().to_string(),
        action: action.to_string(),
    };
    request.ensure_preview_redacted(credentials)?;
    Ok(request)
}

fn build_order_test_preflight_report(
    request: &TestnetSignedOrderRequest,
    credentials: &EnvOnlyTestnetOrderCredentials,
) -> TestnetOrderTestPreflightReport {
    let preview = request.redacted_preview(credentials);
    TestnetOrderTestPreflightReport {
        schema_version: "ntpro.v100_order_test_preflight_report.v1".to_string(),
        status: "ready".to_string(),
        endpoint_class: "binance-testnet-order-test-preflight".to_string(),
        request_method: preview.request_method,
        request_target: preview.request_target,
        query_shape: preview.query_shape,
        api_key_header_name: preview.api_key_header_name,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        request_body_recorded: false,
        signature_preflight: "created_in_memory_not_recorded".to_string(),
        binance_order_test_acceptance: "not_attempted_offline_manual_only".to_string(),
        matching_engine_submission: false,
        order_submission: "order_test_preflight_only".to_string(),
        order_submission_remains_disabled: true,
        network_attempted: false,
        real_orders_submitted: false,
        production_endpoint_allowed: false,
        dashboard_order_controls: false,
        secrets_redacted: true,
        diagnostic: "V100 /api/v3/order/test preflight prepared redacted request metadata only; Binance acceptance is not attempted in offline CI and matching engine submission remains false.".to_string(),
    }
}

fn build_execution_artifact_contract_report(
    order_test_request: &TestnetSignedOrderRequest,
    submit_request: &TestnetSignedOrderRequest,
    cancel_request: &TestnetSignedOrderRequest,
    credentials: &EnvOnlyTestnetOrderCredentials,
) -> TestnetExecutionArtifactContractReport {
    let order_test_preview = order_test_request.redacted_preview(credentials);
    let submit_preview = submit_request.redacted_preview(credentials);
    let cancel_preview = cancel_request.redacted_preview(credentials);

    TestnetExecutionArtifactContractReport {
        schema_version: TESTNET_EXECUTION_ARTIFACT_SCHEMA_VERSION.to_string(),
        status: "ready".to_string(),
        artifact_family: "binance-testnet-order-lifecycle-proof".to_string(),
        request_artifact: TestnetExecutionArtifactContractEntry {
            name: "request.json".to_string(),
            schema: "ntpro.v100_execution_request_artifact.v1".to_string(),
            status: "schema_defined_redacted_preview_only".to_string(),
            source: format!(
                "{} {} plus {} {}",
                submit_preview.request_method,
                submit_preview.request_target,
                cancel_preview.request_method,
                cancel_preview.request_target,
            ),
            redaction: "signature, signed query, signed URL, API key value, and body are not recorded".to_string(),
        },
        order_test_artifact: TestnetExecutionArtifactContractEntry {
            name: "order_test.json".to_string(),
            schema: "ntpro.v100_order_test_preflight_report.v1".to_string(),
            status: "schema_defined_offline_acceptance_not_attempted".to_string(),
            source: format!(
                "{} {}",
                order_test_preview.request_method, order_test_preview.request_target
            ),
            redaction: "signature and API key material are not recorded".to_string(),
        },
        submit_ack_artifact: TestnetExecutionArtifactContractEntry {
            name: "submit_ack.json".to_string(),
            schema: "ntpro.v100_submit_ack_artifact.v1".to_string(),
            status: "manual_online_artifact_required_not_observed_offline".to_string(),
            source: format!(
                "{} {}",
                submit_preview.request_method, submit_preview.request_target
            ),
            redaction: "exchange order id, client order id, and timestamps only; no secrets or signatures".to_string(),
        },
        cancel_ack_artifact: TestnetExecutionArtifactContractEntry {
            name: "cancel_ack.json".to_string(),
            schema: "ntpro.v100_cancel_ack_artifact.v1".to_string(),
            status: "manual_online_artifact_required_not_observed_offline".to_string(),
            source: format!(
                "{} {}",
                cancel_preview.request_method, cancel_preview.request_target
            ),
            redaction: "exchange order id, client order id, and terminal status only; no secrets or signatures".to_string(),
        },
        lifecycle_artifact: TestnetExecutionArtifactContractEntry {
            name: "lifecycle.json".to_string(),
            schema: "ntpro.v100_order_lifecycle_artifact.v1".to_string(),
            status: "manual_online_artifact_required_not_observed_offline".to_string(),
            source: "request -> order_test -> submit_ack -> cancel_ack -> terminal_state".to_string(),
            redaction: "contains state transitions and counters only".to_string(),
        },
        reconciliation_artifact: TestnetExecutionArtifactContractEntry {
            name: "reconciliation.json".to_string(),
            schema: "ntpro.v100_reconciliation_artifact.v1".to_string(),
            status: "schema_defined_manual_or_fixture_input_required".to_string(),
            source: "local lifecycle plus exchange open-order/account readback".to_string(),
            redaction: "contains reconciliation status and risk_halt decision only".to_string(),
        },
        counters: TestnetExecutionArtifactCounters {
            testnet_orders_submitted: 0,
            testnet_orders_canceled: 0,
            production_orders_submitted: 0,
            production_orders_canceled: 0,
        },
        manual_submit_cancel_proof_observed: false,
        matching_engine_submission: false,
        order_submission_remains_disabled: true,
        network_attempted: false,
        real_orders_submitted: false,
        production_endpoint_allowed: false,
        dashboard_order_controls: false,
        secrets_redacted: true,
        diagnostic: "V100 execution artifact contract defines redacted artifact schemas and counters only; real Binance testnet submit/cancel proof remains manual-gated and is not observed offline.".to_string(),
    }
}

fn build_testnet_reconciliation_fixture_report(
    testnet_order: &StrategyNodeTestnetOrderSection,
    scenario: TestnetReconciliationScenario,
) -> TestnetReconciliationFixtureReport {
    let scenarios = reconciliation_fixture_entries(scenario);
    TestnetReconciliationFixtureReport {
        schema_version: TESTNET_RECONCILIATION_FIXTURE_SCHEMA_VERSION.to_string(),
        status: "risk_halted".to_string(),
        symbol: testnet_order.symbol.clone(),
        scenario: reconciliation_scenario_label(scenario).to_string(),
        scenario_count: scenarios.len(),
        scenarios,
        counters: TestnetExecutionArtifactCounters {
            testnet_orders_submitted: 0,
            testnet_orders_canceled: 0,
            production_orders_submitted: 0,
            production_orders_canceled: 0,
        },
        risk_halted: true,
        new_orders_blocked: true,
        manual_submit_cancel_proof_observed: false,
        matching_engine_submission: false,
        order_submission_remains_disabled: true,
        network_attempted: false,
        real_orders_submitted: false,
        production_endpoint_allowed: false,
        dashboard_order_controls: false,
        diagnostic: "Offline V100 reconciliation fixtures classify inconsistent order state as risk_halted and block new order submission; no Binance network calls or real orders are attempted.".to_string(),
    }
}

fn reconciliation_fixture_entries(
    scenario: TestnetReconciliationScenario,
) -> Vec<TestnetReconciliationFixtureEntry> {
    match scenario {
        TestnetReconciliationScenario::All => vec![
            reconciliation_fixture_entry(TestnetReconciliationScenario::SubmitWithoutLocalAck),
            reconciliation_fixture_entry(TestnetReconciliationScenario::CancelTimeout),
            reconciliation_fixture_entry(TestnetReconciliationScenario::LocalOpenExchangeFilled),
            reconciliation_fixture_entry(TestnetReconciliationScenario::RestartUnfinishedOrder),
        ],
        scenario => vec![reconciliation_fixture_entry(scenario)],
    }
}

fn reconciliation_fixture_entry(
    scenario: TestnetReconciliationScenario,
) -> TestnetReconciliationFixtureEntry {
    match scenario {
        TestnetReconciliationScenario::All => {
            unreachable!("all scenario expands before fixture entry construction")
        }
        TestnetReconciliationScenario::SubmitWithoutLocalAck => TestnetReconciliationFixtureEntry {
            name: "submit_without_local_ack".to_string(),
            local_state: "submit_request_recorded_ack_missing".to_string(),
            exchange_state: "unknown_until_readback".to_string(),
            risk_halted: true,
            new_orders_blocked: true,
            action_required: "read_exchange_order_status_then_cancel_or_mark_terminal_before_new_orders".to_string(),
            diagnostic: "A submit request without local ack is ambiguous; the offline contract risk-halts until exchange readback resolves the order.".to_string(),
        },
        TestnetReconciliationScenario::CancelTimeout => TestnetReconciliationFixtureEntry {
            name: "cancel_timeout".to_string(),
            local_state: "cancel_request_recorded_terminal_state_missing".to_string(),
            exchange_state: "open_or_canceled_unknown".to_string(),
            risk_halted: true,
            new_orders_blocked: true,
            action_required: "read_exchange_order_status_until_terminal_or_keep_risk_halt".to_string(),
            diagnostic: "A cancel timeout is not safe to treat as canceled; new orders stay blocked until terminal exchange state is known.".to_string(),
        },
        TestnetReconciliationScenario::LocalOpenExchangeFilled => {
            TestnetReconciliationFixtureEntry {
                name: "local_open_exchange_filled".to_string(),
                local_state: "open".to_string(),
                exchange_state: "filled".to_string(),
                risk_halted: true,
                new_orders_blocked: true,
                action_required: "import_exchange_fill_then_reconcile_position_before_new_orders".to_string(),
                diagnostic: "Exchange-filled while local-open creates position/account ambiguity; risk halt is mandatory before any new order.".to_string(),
            }
        }
        TestnetReconciliationScenario::RestartUnfinishedOrder => TestnetReconciliationFixtureEntry {
            name: "restart_unfinished_order".to_string(),
            local_state: "unfinished_testnet_order_loaded_on_restart".to_string(),
            exchange_state: "requires_readback".to_string(),
            risk_halted: true,
            new_orders_blocked: true,
            action_required: "reconcile_unfinished_order_on_startup_before_enabling_order_path".to_string(),
            diagnostic: "Restart with unfinished order state must remain risk-halted until readback and terminal handling complete.".to_string(),
        },
    }
}

fn reconciliation_scenario_label(scenario: TestnetReconciliationScenario) -> &'static str {
    match scenario {
        TestnetReconciliationScenario::All => "all",
        TestnetReconciliationScenario::SubmitWithoutLocalAck => "submit_without_local_ack",
        TestnetReconciliationScenario::CancelTimeout => "cancel_timeout",
        TestnetReconciliationScenario::LocalOpenExchangeFilled => "local_open_exchange_filled",
        TestnetReconciliationScenario::RestartUnfinishedOrder => "restart_unfinished_order",
    }
}

fn normalize_testnet_order_method(method: &str) -> anyhow::Result<String> {
    let method = method.trim().to_ascii_uppercase();
    if method.is_empty() {
        anyhow::bail!("signed order request preview method must not be empty");
    }
    Ok(method)
}

fn normalize_testnet_order_endpoint_path(endpoint_path: &str) -> anyhow::Result<String> {
    let endpoint_path = endpoint_path.trim();
    if endpoint_path.is_empty() {
        anyhow::bail!("signed order request preview endpoint must not be empty");
    }
    if endpoint_path.contains('?') {
        anyhow::bail!("signed order request preview endpoint must not include query parameters");
    }
    if !endpoint_path.starts_with('/') {
        anyhow::bail!("signed order request preview endpoint must start with '/'");
    }
    Ok(endpoint_path.to_string())
}

fn ensure_testnet_signed_order_request_allowed(
    method: &str,
    endpoint_path: &str,
) -> anyhow::Result<&'static str> {
    match (method, endpoint_path) {
        (TESTNET_ORDER_METHOD_POST, TESTNET_ORDER_ENDPOINT_TEST) => Ok("order_test"),
        (TESTNET_ORDER_METHOD_POST, TESTNET_ORDER_ENDPOINT_ORDER) => Ok("submit"),
        (TESTNET_ORDER_METHOD_DELETE, TESTNET_ORDER_ENDPOINT_ORDER) => Ok("cancel"),
        _ => anyhow::bail!(
            "signed order request allowlist only includes POST /api/v3/order/test, POST /api/v3/order, and DELETE /api/v3/order; got {method} {endpoint_path}"
        ),
    }
}

fn build_testnet_signed_order_query(
    testnet_order: &StrategyNodeTestnetOrderSection,
    method: &str,
    endpoint_path: &str,
    timestamp_ms: u64,
    recv_window_ms: u64,
    orig_client_order_id: Option<&str>,
) -> anyhow::Result<String> {
    if method == TESTNET_ORDER_METHOD_DELETE && endpoint_path == TESTNET_ORDER_ENDPOINT_ORDER {
        let orig_client_order_id = orig_client_order_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("DELETE /api/v3/order preview requires --orig-client-order-id")?;
        let recv_window = recv_window_ms.to_string();
        let timestamp = timestamp_ms.to_string();
        return Ok(join_query_pairs([
            ("symbol", testnet_order.symbol.as_str()),
            ("origClientOrderId", orig_client_order_id),
            ("recvWindow", recv_window.as_str()),
            ("timestamp", timestamp.as_str()),
        ]));
    }

    let recv_window = recv_window_ms.to_string();
    let timestamp = timestamp_ms.to_string();
    Ok(join_query_pairs([
        ("symbol", testnet_order.symbol.as_str()),
        ("side", testnet_order.side.as_str()),
        ("type", testnet_order.order_type.as_str()),
        ("timeInForce", testnet_order.time_in_force.as_str()),
        ("quantity", testnet_order.quantity.as_str()),
        ("price", testnet_order.price.as_str()),
        ("newOrderRespType", "ACK"),
        ("recvWindow", recv_window.as_str()),
        ("timestamp", timestamp.as_str()),
    ]))
}

fn join_query_pairs<const N: usize>(pairs: [(&str, &str); N]) -> String {
    pairs
        .into_iter()
        .map(|(name, value)| {
            format!(
                "{}={}",
                urlencoding::encode(name),
                urlencoding::encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn write_secret_redacted_json<T>(
    path: &Path,
    value: &T,
    credentials: &EnvOnlyTestnetOrderCredentials,
) -> anyhow::Result<()>
where
    T: Serialize,
{
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    credentials.ensure_no_secret_values_absent(&path.display().to_string(), &body)?;
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_account_snapshot_report(
    path: &Path,
    value: &ProductionAccountSnapshotContractReport,
    credentials: &EnvOnlyProductionReadCredentials,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    credentials.ensure_no_secret_values_absent(&path.display().to_string(), &body)?;
    atomic_write_text(path, &body)?;
    Ok(())
}

fn decimal_string_to_f64(field: &str, value: &str) -> Result<f64, String> {
    validate_positive_decimal_string(field, value).map_err(|error| error.to_string())?;
    value
        .parse::<f64>()
        .ok()
        .filter(|parsed| parsed.is_finite())
        .ok_or_else(|| format!("{field} must parse as a finite decimal"))
}

fn missing_testnet_order_cli_flags(opt: &LiveTestnetOrderGateOpt) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_testnet_order {
        missing.push("--allow-testnet-order");
    }
    if !opt.confirm_owner_approved_testnet_order {
        missing.push("--confirm-owner-approved-testnet-order");
    }
    if !opt.confirm_tiny_notional {
        missing.push("--confirm-tiny-notional");
    }
    if !opt.confirm_cancel_after_submit {
        missing.push("--confirm-cancel-after-submit");
    }
    missing
}

fn missing_testnet_order_preflight_cli_flags(
    opt: &LiveTestnetOrderPreflightOpt,
) -> Vec<&'static str> {
    missing_testnet_order_manual_cli_flags(
        opt.allow_testnet_order,
        opt.confirm_owner_approved_testnet_order,
        opt.confirm_tiny_notional,
        opt.confirm_cancel_after_submit,
    )
}

fn missing_testnet_order_request_preview_cli_flags(
    opt: &LiveTestnetOrderRequestPreviewOpt,
) -> Vec<&'static str> {
    missing_testnet_order_manual_cli_flags(
        opt.allow_testnet_order,
        opt.confirm_owner_approved_testnet_order,
        opt.confirm_tiny_notional,
        opt.confirm_cancel_after_submit,
    )
}

fn missing_testnet_order_test_preflight_cli_flags(
    opt: &LiveTestnetOrderTestPreflightOpt,
) -> Vec<&'static str> {
    missing_testnet_order_manual_cli_flags(
        opt.allow_testnet_order,
        opt.confirm_owner_approved_testnet_order,
        opt.confirm_tiny_notional,
        opt.confirm_cancel_after_submit,
    )
}

fn missing_testnet_execution_artifact_contract_cli_flags(
    opt: &LiveTestnetExecutionArtifactContractOpt,
) -> Vec<&'static str> {
    missing_testnet_order_manual_cli_flags(
        opt.allow_testnet_order,
        opt.confirm_owner_approved_testnet_order,
        opt.confirm_tiny_notional,
        opt.confirm_cancel_after_submit,
    )
}

fn missing_testnet_order_manual_cli_flags(
    allow_testnet_order: bool,
    confirm_owner_approved_testnet_order: bool,
    confirm_tiny_notional: bool,
    confirm_cancel_after_submit: bool,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !allow_testnet_order {
        missing.push("--allow-testnet-order");
    }
    if !confirm_owner_approved_testnet_order {
        missing.push("--confirm-owner-approved-testnet-order");
    }
    if !confirm_tiny_notional {
        missing.push("--confirm-tiny-notional");
    }
    if !confirm_cancel_after_submit {
        missing.push("--confirm-cancel-after-submit");
    }
    missing
}

fn missing_testnet_order_env_gates<F>(read_env: &mut F) -> Vec<&'static str>
where
    F: FnMut(&str) -> Option<String>,
{
    [
        TESTNET_ORDER_ENV_ALLOW,
        TESTNET_ORDER_ENV_OWNER_APPROVED,
        TESTNET_ORDER_ENV_TINY_NOTIONAL,
        TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT,
    ]
    .into_iter()
    .filter(|name| read_env(name).as_deref() != Some("1"))
    .collect()
}

fn missing_production_public_read_cli_flags(
    opt: &LiveProductionPublicReadProbeOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_public_read {
        missing.push("--allow-production-public-read");
    }
    if !opt.confirm_read_only {
        missing.push("--confirm-read-only");
    }
    if !opt.confirm_no_order_mutation {
        missing.push("--confirm-no-order-mutation");
    }
    missing
}

fn missing_production_public_read_env_gates<F>(
    read_env: &mut F,
    manual_online_requested: bool,
) -> Vec<&'static str>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut missing: Vec<&'static str> = [
        PRODUCTION_PUBLIC_READ_ENV_ALLOW,
        PRODUCTION_PUBLIC_READ_ENV_READ_ONLY,
        PRODUCTION_PUBLIC_READ_ENV_NO_ORDER_MUTATION,
    ]
    .into_iter()
    .filter(|name| read_env(name).as_deref() != Some("1"))
    .collect();
    if manual_online_requested
        && read_env(PRODUCTION_PUBLIC_READ_ENV_MANUAL_ONLINE).as_deref() != Some("1")
    {
        missing.push(PRODUCTION_PUBLIC_READ_ENV_MANUAL_ONLINE);
    }
    missing
}

fn missing_production_account_snapshot_cli_flags(
    opt: &LiveProductionAccountSnapshotContractOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_authenticated_read {
        missing.push("--allow-production-authenticated-read");
    }
    if !opt.confirm_owner_approved_read_only {
        missing.push("--confirm-owner-approved-read-only");
    }
    if !opt.confirm_no_order_mutation {
        missing.push("--confirm-no-order-mutation");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_account_snapshot_env_gates<F>(
    read_env: &mut F,
    manual_online_requested: bool,
) -> Vec<&'static str>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut missing: Vec<&'static str> = [
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW,
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED,
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION,
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE,
    ]
    .into_iter()
    .filter(|name| read_env(name).as_deref() != Some("1"))
    .collect();
    if manual_online_requested
        && read_env(PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE).as_deref() != Some("1")
    {
        missing.push(PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE);
    }
    missing
}

fn join_gate_labels(labels: &[&str]) -> String {
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(",")
    }
}

fn join_owned_gate_labels(labels: &[String]) -> String {
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(",")
    }
}

#[derive(Debug)]
struct LiveSmokeResult {
    pre_start_state: String,
    running_state: String,
    final_state: String,
    final_node_state: NodeState,
    account_cached: bool,
    started_at: String,
    stopped_at: String,
    uptime_ms: u64,
    shutdown_reason: ShutdownReason,
}

#[derive(Clone, Copy)]
struct LiveRunContext<'a> {
    config: &'a MinimalLiveConfig,
    config_path: &'a Path,
    run_id: &'a str,
    output_dir: &'a Path,
    process_mode: ProcessMode,
    status_path: &'a Path,
    metrics_path: &'a Path,
    stdout_log_path: &'a Path,
    stderr_log_path: &'a Path,
    events_log_path: &'a Path,
    stop_file: Option<&'a Path>,
    shutdown_controls: NtproNodeRunControls,
}

struct StrategyNodeStatusContext<'a> {
    config_path: &'a Path,
    output_dir: &'a Path,
    run_id: &'a str,
    process_mode: ProcessMode,
    started_at: &'a str,
    stopped_at: Option<&'a str>,
    counters: StrategyRuntimeCounters,
}

struct StrategyNodeMetricPaths<'a> {
    status_path: &'a Path,
    stdout_log_path: &'a Path,
    stderr_log_path: &'a Path,
    events_log_path: &'a Path,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShutdownReason {
    StartStop,
    StopFile,
    Signal,
    MaxRuntime,
    ParentExited,
}

impl ShutdownReason {
    const fn label(self) -> &'static str {
        match self {
            Self::StartStop => "start-stop",
            Self::StopFile => "stop-file",
            Self::Signal => "signal",
            Self::MaxRuntime => "max-runtime",
            Self::ParentExited => "parent-exited",
        }
    }
}

async fn run_live_init_smoke(context: &LiveRunContext<'_>) -> anyhow::Result<LiveSmokeResult> {
    let config = context.config;
    let trader_id = TraderId::from(config.system.trader_id.as_str());
    let account_id = AccountId::from(config.adapter.account_id.as_str());
    let venue = Venue::from(config.adapter.venue.as_str());
    let sandbox_config = SandboxExecutionClientConfig {
        trader_id,
        account_id,
        venue,
        starting_balances: config
            .adapter
            .starting_balances
            .iter()
            .map(|balance| Money::from(balance.as_str()))
            .collect(),
        ..Default::default()
    };

    let mut node = LiveNode::builder(trader_id, Environment::Sandbox)?
        .with_name(node_name(config))
        .with_reconciliation(false)
        .with_load_state(config.system.load_state.unwrap_or(false))
        .with_save_state(config.system.save_state.unwrap_or(false))
        .with_timeout_connection(config.shutdown.connection_timeout_secs)
        .with_timeout_disconnection_secs(config.shutdown.disconnection_timeout_secs)
        .with_delay_post_stop_secs(config.shutdown.post_stop_delay_secs)
        .add_simulated_exec_client(
            Some(config.adapter.name.clone()),
            Box::new(SandboxExecutionClientFactory::new()),
            Box::new(sandbox_config),
        )?
        .build()?;
    let handle = node.handle();

    if node.environment() != Environment::Sandbox {
        anyhow::bail!("live-init-smoke must run in sandbox environment");
    }
    if handle.state() != NodeState::Idle {
        anyhow::bail!("live-init-smoke expected Idle before start");
    }
    let pre_start_state = format!("{:?}", handle.state());

    node.start().await?;
    let started_at = now_millis();
    let started_instant = Instant::now();
    let running_state = format!("{:?}", handle.state());
    let account_cached = node
        .kernel()
        .cache
        .borrow()
        .account_owned(&account_id)
        .is_some();
    if handle.state() != NodeState::Running {
        anyhow::bail!("live-init-smoke expected Running after start");
    }
    if !account_cached {
        anyhow::bail!("live-init-smoke expected sandbox account to be cached");
    }

    let shutdown_reason = wait_for_shutdown_trigger(context, &started_at, started_instant).await?;

    timeout(context.shutdown_controls.shutdown_timeout, node.stop())
        .await
        .with_context(|| {
            format!(
                "ntpro-node shutdown timed out after {} ms",
                millis_to_u64(context.shutdown_controls.shutdown_timeout.as_millis())
            )
        })??;
    let stopped_at = now_millis();
    let uptime_ms = millis_to_u64(started_instant.elapsed().as_millis());
    let final_state = format!("{:?}", handle.state());
    if handle.state() != NodeState::Stopped {
        anyhow::bail!("live-init-smoke expected Stopped after stop");
    }
    let final_node_state = handle.state();

    Ok(LiveSmokeResult {
        pre_start_state,
        running_state,
        final_state,
        final_node_state,
        account_cached,
        started_at,
        stopped_at,
        uptime_ms,
        shutdown_reason,
    })
}

fn build_node_status(context: &LiveRunContext<'_>, smoke: &LiveSmokeResult) -> NodeStatus {
    build_node_status_for_state(
        context,
        smoke.final_node_state,
        LifecycleStatus::Running,
        ConnectionStatus::Disconnected,
        false,
        Some(&smoke.started_at),
        Some(&smoke.stopped_at),
    )
}

fn build_strategy_node_status(
    context: &StrategyNodeStatusContext<'_>,
    state: NodeState,
) -> NodeStatus {
    let mut status = NodeStatus::from_node_state(context.run_id, state);
    let generated_at = now_millis();
    status.process_mode = context.process_mode;
    status.config_path = SnapshotValue::available(context.config_path.display().to_string());
    status.artifact_root = SnapshotValue::available(context.output_dir.display().to_string());
    status.previous_lifecycle_state = LifecycleStatus::Running;
    status.data_connection = match state {
        NodeState::Running => ConnectionStatus::Connected,
        NodeState::Stopped => ConnectionStatus::Disconnected,
        _ => ConnectionStatus::NotConfigured,
    };
    status.execution_connection = ConnectionStatus::NotConfigured;
    status.execution = ExecutionStatus {
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
    status.risk.trading_state = nautilus_live::status::RiskTradingState::Halted;
    status.risk.health = nautilus_live::status::HealthStatus::Healthy;
    status.risk.command_count = SnapshotValue::available(context.counters.signal_count);
    status.risk.event_count = SnapshotValue::available(context.counters.risk_decision_count);
    status.risk.rejections_total = SnapshotValue::available(context.counters.rejection_count);
    if context.counters.rejection_count > 0 {
        status.risk.last_rejection = Some("order_submission_disabled".to_string());
    }
    status.generated_at = SnapshotValue::available(generated_at.clone());
    status.started_at = SnapshotValue::available(context.started_at.to_string());
    status.stopped_at = context
        .stopped_at
        .map_or_else(SnapshotValue::unknown, |value| {
            SnapshotValue::available(value.to_string())
        });
    status.last_transition_at = SnapshotValue::available(generated_at);
    status.external_venue_connection = false;
    status.real_orders_submitted = false;
    status
}

fn build_node_status_for_state(
    context: &LiveRunContext<'_>,
    state: NodeState,
    previous_lifecycle_state: LifecycleStatus,
    execution_connection: ConnectionStatus,
    execution_started: bool,
    started_at: Option<&str>,
    stopped_at: Option<&str>,
) -> NodeStatus {
    let config = context.config;
    let mut status = NodeStatus::from_node_state(context.run_id, state);
    let generated_at = now_millis();
    status.process_mode = context.process_mode;
    status.config_path = SnapshotValue::available(context.config_path.display().to_string());
    status.artifact_root = SnapshotValue::available(context.output_dir.display().to_string());
    status.previous_lifecycle_state = previous_lifecycle_state;
    status.data_connection = ConnectionStatus::NotConfigured;
    status.execution_connection = execution_connection;
    status.execution = ExecutionStatus {
        gateway_id: SnapshotValue::available(config.adapter.name.clone()),
        connection: execution_connection,
        started: SnapshotValue::available(execution_started),
        account_ref: SnapshotValue::available("configured".to_string()),
        orders_open: SnapshotValue::unknown(),
        orders_inflight: SnapshotValue::unknown(),
        orders_closed: SnapshotValue::unknown(),
        last_report_at: SnapshotValue::unknown(),
        last_reconciliation_at: SnapshotValue::unknown(),
        last_error: None,
    };
    status.generated_at = SnapshotValue::available(generated_at.clone());
    status.started_at = started_at.map_or_else(SnapshotValue::unknown, |value| {
        SnapshotValue::available(value.to_string())
    });
    status.stopped_at = stopped_at.map_or_else(SnapshotValue::unknown, |value| {
        SnapshotValue::available(value.to_string())
    });
    status.last_transition_at = SnapshotValue::available(generated_at);
    status
}

fn write_status(path: &Path, status: &NodeStatus) -> anyhow::Result<()> {
    let status_json = serde_json::to_string_pretty(status)?;
    atomic_write_text(path, &format!("{status_json}\n"))
        .with_context(|| format!("failed to write status '{}'", path.display()))?;
    Ok(())
}

fn write_metrics(
    path: &Path,
    status: &NodeStatus,
    context: &LiveRunContext<'_>,
    counts: NodeMetricCounts,
) -> anyhow::Result<()> {
    let artifacts = NodeMetricArtifacts {
        status_path: context.status_path.to_path_buf(),
        stdout_log_path: context.stdout_log_path.to_path_buf(),
        stderr_log_path: context.stderr_log_path.to_path_buf(),
        events_log_path: context.events_log_path.to_path_buf(),
    };
    let metrics = NodeMetrics::from_status(status, &artifacts, counts);
    write_node_metrics_artifact(path, &metrics)
}

fn write_strategy_node_metrics(
    path: &Path,
    status: &NodeStatus,
    paths: &StrategyNodeMetricPaths<'_>,
    counts: NodeMetricCounts,
) -> anyhow::Result<()> {
    let artifacts = NodeMetricArtifacts {
        status_path: paths.status_path.to_path_buf(),
        stdout_log_path: paths.stdout_log_path.to_path_buf(),
        stderr_log_path: paths.stderr_log_path.to_path_buf(),
        events_log_path: paths.events_log_path.to_path_buf(),
    };
    let metrics = NodeMetrics::from_status(status, &artifacts, counts);
    write_node_metrics_artifact(path, &metrics)
}

async fn wait_for_shutdown_trigger(
    context: &LiveRunContext<'_>,
    started_at: &str,
    started_instant: Instant,
) -> anyhow::Result<ShutdownReason> {
    let Some(stop_file) = context.stop_file else {
        return Ok(ShutdownReason::StartStop);
    };
    let shutdown_signal = wait_for_shutdown_signal();
    tokio::pin!(shutdown_signal);
    let mut last_heartbeat: Option<Instant> = None;

    loop {
        if stop_file.exists() {
            return Ok(ShutdownReason::StopFile);
        }
        if let Some(parent_pid) = context.shutdown_controls.parent_pid
            && !process_is_alive(parent_pid)
        {
            return Ok(ShutdownReason::ParentExited);
        }
        if let Some(max_runtime) = context.shutdown_controls.max_runtime
            && started_instant.elapsed() >= max_runtime
        {
            return Ok(ShutdownReason::MaxRuntime);
        }
        if last_heartbeat
            .is_none_or(|last| last.elapsed() >= context.shutdown_controls.heartbeat_interval)
        {
            write_running_heartbeat(context, started_at, started_instant)?;
            last_heartbeat = Some(Instant::now());
        }

        tokio::select! {
            result = &mut shutdown_signal => return result,
            () = sleep(SHUTDOWN_POLL_INTERVAL) => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn wait_for_strategy_shutdown_trigger(
    stop_file: Option<&Path>,
    shutdown_controls: NtproNodeRunControls,
    status_path: &Path,
    metrics_path: &Path,
    stdout_log_path: &Path,
    stderr_log_path: &Path,
    events_log_path: &Path,
    config_path: &Path,
    output_dir: &Path,
    run_id: &str,
    process_mode: ProcessMode,
    started_at: &str,
    started_instant: Instant,
    counters: StrategyRuntimeCounters,
) -> anyhow::Result<ShutdownReason> {
    let Some(stop_file) = stop_file else {
        return Ok(ShutdownReason::StartStop);
    };
    let shutdown_signal = wait_for_shutdown_signal();
    tokio::pin!(shutdown_signal);
    let mut last_heartbeat: Option<Instant> = None;

    loop {
        if stop_file.exists() {
            return Ok(ShutdownReason::StopFile);
        }
        if let Some(parent_pid) = shutdown_controls.parent_pid
            && !process_is_alive(parent_pid)
        {
            return Ok(ShutdownReason::ParentExited);
        }
        if let Some(max_runtime) = shutdown_controls.max_runtime
            && started_instant.elapsed() >= max_runtime
        {
            return Ok(ShutdownReason::MaxRuntime);
        }
        if last_heartbeat.is_none_or(|last| last.elapsed() >= shutdown_controls.heartbeat_interval)
        {
            let status = build_strategy_node_status(
                &StrategyNodeStatusContext {
                    config_path,
                    output_dir,
                    run_id,
                    process_mode,
                    started_at,
                    stopped_at: None,
                    counters,
                },
                NodeState::Running,
            );
            atomic_write_json(status_path, &status)?;
            write_strategy_node_metrics(
                metrics_path,
                &status,
                &StrategyNodeMetricPaths {
                    status_path,
                    stdout_log_path,
                    stderr_log_path,
                    events_log_path,
                },
                NodeMetricCounts {
                    uptime_ms: Some(millis_to_u64(started_instant.elapsed().as_millis())),
                    starts_total: 1,
                    stops_total: 0,
                    state_transitions_total: 1,
                },
            )?;
            last_heartbeat = Some(Instant::now());
        }

        tokio::select! {
            result = &mut shutdown_signal => return result,
            () = sleep(SHUTDOWN_POLL_INTERVAL) => {}
        }
    }
}

fn write_running_heartbeat(
    context: &LiveRunContext<'_>,
    started_at: &str,
    started_instant: Instant,
) -> anyhow::Result<()> {
    let running_status = build_node_status_for_state(
        context,
        NodeState::Running,
        LifecycleStatus::Starting,
        ConnectionStatus::Disconnected,
        true,
        Some(started_at),
        None,
    );
    write_status(context.status_path, &running_status)?;
    write_metrics(
        context.metrics_path,
        &running_status,
        context,
        NodeMetricCounts {
            uptime_ms: Some(millis_to_u64(started_instant.elapsed().as_millis())),
            starts_total: 1,
            stops_total: 0,
            state_transitions_total: 1,
        },
    )
}

async fn wait_for_shutdown_signal() -> anyhow::Result<ShutdownReason> {
    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .context("failed to register SIGTERM handler for ntpro-node shutdown")?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result.context("failed to wait for Ctrl-C shutdown signal")?;
                Ok(ShutdownReason::Signal)
            }
            _ = sigterm.recv() => Ok(ShutdownReason::Signal),
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .context("failed to wait for Ctrl-C shutdown signal")?;
        Ok(ShutdownReason::Signal)
    }
}

fn now_millis() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    format!("{millis}")
}

fn millis_to_u64(millis: u128) -> u64 {
    u64::try_from(millis).unwrap_or(u64::MAX)
}

const fn process_mode_label(mode: ProcessMode) -> &'static str {
    match mode {
        ProcessMode::SpawnedProcess => "spawned_process",
        ProcessMode::TestHarness => "test_harness",
        ProcessMode::Unknown => "unknown",
    }
}

fn node_name(config: &MinimalLiveConfig) -> &str {
    config
        .system
        .node_name
        .as_deref()
        .or(config.system.instance_id.as_deref())
        .unwrap_or("LiveInitSmoke")
}

fn validate_non_empty(field: &str, value: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} must not be empty");
    }
    Ok(())
}

fn validate_exact(field: &str, value: &str, expected: &str) -> anyhow::Result<()> {
    if value != expected {
        anyhow::bail!("{field} must be '{expected}', got '{value}'");
    }
    Ok(())
}

fn non_zero_duration(field: &str, millis: u64) -> anyhow::Result<Duration> {
    if millis == 0 {
        anyhow::bail!("{field} must be greater than zero");
    }
    Ok(Duration::from_millis(millis))
}

fn resolve_output_dir(
    run_id: &str,
    cli_output: Option<&PathBuf>,
    config_output: Option<&LiveOutputConfig>,
) -> PathBuf {
    if let Some(output) = cli_output {
        return output.clone();
    }
    if let Some(output) = config_output
        && let Some(dir) = &output.dir
    {
        return dir.clone();
    }
    PathBuf::from("runs").join(run_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy_session::{
        StrategyOrderPreflightAccount, StrategyOrderPreflightEndpoint,
        StrategyOrderPreflightLimits, StrategyOrderPreflightMarket, StrategyOrderPreflightRisk,
        StrategyOrderPreflightSession,
    };

    fn write_config(name: &str, content: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ntpro-drg-005-live-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        fs::write(&path, content).unwrap();
        path
    }

    fn minimal_config(output_dir: &Path) -> String {
        format!(
            r#"[run]
id = "live-init-smoke"
mode = "live-init-smoke"
environment = "sandbox"

[system]
trader_id = "LIVE-INIT-001"
node_name = "LiveInitSmoke"
load_state = false
save_state = false

[adapter]
name = "SANDBOX"
kind = "sandbox-simulated-execution"
account_id = "SANDBOX-001"
venue = "SANDBOX"
starting_balances = ["100000 USDT"]

[execution]
order_submission = "disabled"
reconciliation = false
external_venue_connection = false

[shutdown]
mode = "start-stop"
post_stop_delay_secs = 0
connection_timeout_secs = 5
disconnection_timeout_secs = 5

[output]
dir = "{}"
write_summary = true
"#,
            output_dir.display()
        )
    }

    fn strategy_node_config(output_dir: &Path) -> String {
        format!(
            r#"[node]
node_id = "btc-ema-shadow-001"
mode = "shadow"

[strategy]
strategy_id = "ema_cross_btcusdt_v1"
strategy_package = "builtin"
strategy_runtime = "ema_cross_demo"

[market]
venue = "BINANCE_TESTNET"
symbols = ["BTCUSDT.BINANCE"]
data_mode = "fixture_stream"

[execution]
venue = "BINANCE_TESTNET"
order_submission = "disabled"
external_venue_connection = false

[testnet_order]
enabled = false
mode = "disabled"
manual_gate = "owner-approved-manual"
http_base_url = "https://testnet.binance.vision"
symbol = "BTCUSDT"
instrument_id = "BTCUSDT.BINANCE"
side = "BUY"
order_type = "LIMIT"
time_in_force = "GTC"
price = "1.00"
quantity = "0.00001000"
notional = "0.00001000"
cancel_after_submit_ms = 3000
owner_approval_required = true
manual_env_gate_required = true
production_endpoint_allowed = false
dashboard_order_controls = false

[risk]
kill_switch_enabled = true
kill_switch_active = false

[shutdown]
mode = "start-stop"
post_stop_delay_secs = 0
connection_timeout_secs = 1
disconnection_timeout_secs = 1

[output]
dir = "{}"
write_summary = true
"#,
            output_dir.display()
        )
    }

    #[test]
    fn validates_minimal_live_config() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-drg-005-live-validate-{}",
            std::process::id()
        ));
        let path = write_config("validate", &minimal_config(&output_dir));

        validate_minimal_live_config_file(&path).unwrap();
    }

    #[test]
    fn validates_strategy_node_testnet_order_contract() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-001-strategy-contract-{}",
            std::process::id()
        ));
        let path = write_config("strategy-contract", &strategy_node_config(&output_dir));

        validate_strategy_node_config_file(&path).unwrap();
    }

    #[test]
    fn rejects_strategy_node_testnet_order_enabled_by_default() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-001-strategy-order-enabled-{}",
            std::process::id()
        ));
        let config = strategy_node_config(&output_dir).replace("enabled = false", "enabled = true");
        let path = write_config("strategy-order-enabled", &config);

        let error = validate_strategy_node_config_file(&path)
            .unwrap_err()
            .to_string();

        assert!(error.contains("testnet_order.enabled must be false"));
    }

    #[test]
    fn rejects_strategy_node_testnet_order_non_decimal_quantity() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-001-strategy-order-bad-quantity-{}",
            std::process::id()
        ));
        let config = strategy_node_config(&output_dir)
            .replace(r#"quantity = "0.00001000""#, r#"quantity = "1e-5""#);
        let path = write_config("strategy-order-bad-quantity", &config);

        let error = validate_strategy_node_config_file(&path)
            .unwrap_err()
            .to_string();

        assert!(error.contains("testnet_order.quantity must be a positive decimal string"));
    }

    fn testnet_order_gate_opt(config: PathBuf, all_cli_gates: bool) -> LiveTestnetOrderGateOpt {
        LiveTestnetOrderGateOpt {
            config,
            allow_testnet_order: all_cli_gates,
            confirm_owner_approved_testnet_order: all_cli_gates,
            confirm_tiny_notional: all_cli_gates,
            confirm_cancel_after_submit: all_cli_gates,
        }
    }

    fn testnet_order_preflight_opt(
        config: PathBuf,
        input: PathBuf,
        output: Option<PathBuf>,
        all_cli_gates: bool,
    ) -> LiveTestnetOrderPreflightOpt {
        LiveTestnetOrderPreflightOpt {
            config,
            input,
            output,
            allow_testnet_order: all_cli_gates,
            confirm_owner_approved_testnet_order: all_cli_gates,
            confirm_tiny_notional: all_cli_gates,
            confirm_cancel_after_submit: all_cli_gates,
        }
    }

    fn testnet_order_request_preview_opt(
        config: PathBuf,
        output: Option<PathBuf>,
        all_cli_gates: bool,
    ) -> LiveTestnetOrderRequestPreviewOpt {
        LiveTestnetOrderRequestPreviewOpt {
            config,
            method: TESTNET_ORDER_METHOD_POST.to_string(),
            endpoint_path: TESTNET_ORDER_ENDPOINT_TEST.to_string(),
            timestamp_ms: 1_718_400_000_000,
            recv_window_ms: 5_000,
            api_key_env: "NTPRO_V100004_API_KEY".to_string(),
            api_secret_env: "NTPRO_V100004_API_SECRET".to_string(),
            orig_client_order_id: None,
            output,
            allow_testnet_order: all_cli_gates,
            confirm_owner_approved_testnet_order: all_cli_gates,
            confirm_tiny_notional: all_cli_gates,
            confirm_cancel_after_submit: all_cli_gates,
        }
    }

    fn testnet_order_test_preflight_opt(
        config: PathBuf,
        output: Option<PathBuf>,
        all_cli_gates: bool,
    ) -> LiveTestnetOrderTestPreflightOpt {
        LiveTestnetOrderTestPreflightOpt {
            config,
            timestamp_ms: 1_718_400_000_000,
            recv_window_ms: 5_000,
            api_key_env: "NTPRO_V100005_API_KEY".to_string(),
            api_secret_env: "NTPRO_V100005_API_SECRET".to_string(),
            output,
            allow_testnet_order: all_cli_gates,
            confirm_owner_approved_testnet_order: all_cli_gates,
            confirm_tiny_notional: all_cli_gates,
            confirm_cancel_after_submit: all_cli_gates,
        }
    }

    fn testnet_execution_artifact_contract_opt(
        config: PathBuf,
        output: Option<PathBuf>,
        all_cli_gates: bool,
    ) -> LiveTestnetExecutionArtifactContractOpt {
        LiveTestnetExecutionArtifactContractOpt {
            config,
            timestamp_ms: 1_718_400_000_000,
            recv_window_ms: 5_000,
            api_key_env: "NTPRO_V100007_API_KEY".to_string(),
            api_secret_env: "NTPRO_V100007_API_SECRET".to_string(),
            orig_client_order_id: "ntpro-v100007-cancel-only".to_string(),
            output,
            allow_testnet_order: all_cli_gates,
            confirm_owner_approved_testnet_order: all_cli_gates,
            confirm_tiny_notional: all_cli_gates,
            confirm_cancel_after_submit: all_cli_gates,
        }
    }

    fn production_public_read_probe_opt(
        endpoint: ProductionPublicReadEndpoint,
        output: Option<PathBuf>,
        all_cli_gates: bool,
        manual_online: bool,
    ) -> LiveProductionPublicReadProbeOpt {
        LiveProductionPublicReadProbeOpt {
            endpoint,
            output,
            manual_online,
            allow_production_public_read: all_cli_gates,
            confirm_read_only: all_cli_gates,
            confirm_no_order_mutation: all_cli_gates,
        }
    }

    fn all_env_enabled(name: &str) -> Option<String> {
        (!name.is_empty()).then(|| "1".to_string())
    }

    fn production_account_snapshot_contract_opt(
        output: Option<PathBuf>,
        all_cli_gates: bool,
        manual_online: bool,
    ) -> LiveProductionAccountSnapshotContractOpt {
        LiveProductionAccountSnapshotContractOpt {
            output,
            manual_online,
            api_key_env: "NTPRO_V110003_API_KEY".to_string(),
            api_secret_env: "NTPRO_V110003_API_SECRET".to_string(),
            recv_window_ms: 5_000,
            allow_production_authenticated_read: all_cli_gates,
            confirm_owner_approved_read_only: all_cli_gates,
            confirm_no_order_mutation: all_cli_gates,
            confirm_no_secret_persistence: all_cli_gates,
        }
    }

    fn write_redacted_account_snapshot_report(path: &Path, response_shape_validated: bool) {
        let shape_summary = if response_shape_validated {
            serde_json::json!({
                "status": "accepted",
                "balance_entry_count": 2,
                "shape_validated": true,
                "raw_account_response_recorded": false,
                "raw_balances_recorded": false,
                "raw_permissions_recorded": false
            })
        } else {
            serde_json::json!({
                "status": "not_attempted",
                "balance_entry_count": null,
                "shape_validated": false,
                "raw_account_response_recorded": false,
                "raw_balances_recorded": false,
                "raw_permissions_recorded": false
            })
        };
        fs::write(
            path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION,
                "status": if response_shape_validated { "online_account_snapshot_ok" } else { "ready_offline_contract" },
                "response_shape_validated": response_shape_validated,
                "response_shape_summary": shape_summary,
                "network_attempted": response_shape_validated,
                "account_read_attempted": response_shape_validated,
                "api_key_value_recorded": false,
                "api_secret_value_recorded": false,
                "signature_recorded": false,
                "signed_query_recorded": false,
                "signed_url_recorded": false,
                "production_order_submission_attempted": false,
                "production_order_mutation_attempted": false,
                "dashboard_order_controls_enabled": false,
                "secrets_redacted": true
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_shadow_intent(path: &Path, actual_submission: bool) {
        fs::write(
            path,
            format!(
                r#"{{"schema_version":"ntpro.v110_shadow_execution_intent.v1","run_id":"v120-shadow","intent_id":"intent-1","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","venue":"BINANCE","side":"buy","order_type":"market","quantity":"0.001","notional":"10.00","mode":"production_shadow","submission_allowed":false,"actual_submission":{actual_submission},"submission_status":"blocked_by_v110_shadow_execution_boundary","execution_adapter_called":false,"order_endpoint_access_attempted":false,"production_order_mutation_attempted":false,"dashboard_order_controls_enabled":false}}
"#
            ),
        )
        .unwrap();
    }

    fn read_jsonl_values(path: &Path) -> Vec<serde_json::Value> {
        fs::read_to_string(path)
            .unwrap()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn synthetic_order_credentials() -> EnvOnlyTestnetOrderCredentials {
        EnvOnlyTestnetOrderCredentials::from_values(
            "NTPRO_V100004_API_KEY".to_string(),
            Some("ntpro_v100004_synthetic_api_key_value".to_string()),
            "NTPRO_V100004_API_SECRET".to_string(),
            Some("ntpro_v100004_synthetic_api_secret_value".to_string()),
        )
    }

    fn passing_preflight_input() -> StrategyOrderPreflightInput {
        StrategyOrderPreflightInput {
            schema_version: STRATEGY_ORDER_PREFLIGHT_SCHEMA_VERSION.to_string(),
            session: StrategyOrderPreflightSession {
                state: "running".to_string(),
            },
            market: StrategyOrderPreflightMarket {
                symbol: "BTCUSDT.BINANCE".to_string(),
                last_event_at_unix_ms: 1_000,
                now_unix_ms: 1_500,
                max_age_ms: 1_000,
            },
            account: StrategyOrderPreflightAccount {
                readable: true,
                account_id: "BINANCE_TESTNET-001".to_string(),
            },
            risk: StrategyOrderPreflightRisk {
                kill_switch_active: false,
                allowed_symbols: vec!["BTCUSDT.BINANCE".to_string()],
            },
            limits: StrategyOrderPreflightLimits {
                max_order_notional: "1.00".to_string(),
                max_open_orders: 1,
                open_order_count: 0,
                max_clock_skew_ms: 100,
                observed_clock_skew_ms: 25,
            },
            endpoint: StrategyOrderPreflightEndpoint {
                http_base_url: BINANCE_TESTNET_HTTP_BASE_URL.to_string(),
                production_endpoint_allowed: false,
            },
        }
    }

    fn write_preflight_input(name: &str, input: &StrategyOrderPreflightInput) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ntpro-{name}-preflight-input-{}.json",
            std::process::id()
        ));
        fs::write(path.clone(), serde_json::to_string_pretty(input).unwrap()).unwrap();
        path
    }

    #[test]
    fn testnet_order_gate_blocks_missing_cli_and_env_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-002-gate-blocked-{}",
            std::process::id()
        ));
        let path = write_config(
            "testnet-order-gate-blocked",
            &strategy_node_config(&output_dir),
        );
        let opt = testnet_order_gate_opt(path, false);

        let error = run_live_testnet_order_gate_with_env(&opt, |_| None)
            .unwrap_err()
            .to_string();

        assert!(error.contains("testnet order gate blocked"));
        assert!(error.contains("--allow-testnet-order"));
        assert!(error.contains("--confirm-owner-approved-testnet-order"));
        assert!(error.contains("NTPRO_ALLOW_BINANCE_TESTNET_ORDER"));
        assert!(error.contains("NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER"));
        assert!(error.contains("order_submission_remains_disabled=true"));
        assert!(error.contains("network_attempted=false"));
        assert!(error.contains("real_orders_submitted=false"));
    }

    #[test]
    fn testnet_order_gate_accepts_all_manual_gates_without_network() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-v100-002-gate-ready-{}", std::process::id()));
        let path = write_config(
            "testnet-order-gate-ready",
            &strategy_node_config(&output_dir),
        );
        let opt = testnet_order_gate_opt(path, true);

        run_live_testnet_order_gate_with_env(&opt, |_| Some("1".to_string())).unwrap();
    }

    #[test]
    fn testnet_order_preflight_passes_with_ready_snapshot() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-003-preflight-pass-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-preflight-pass",
            &strategy_node_config(&output_dir),
        );
        let input = write_preflight_input("v100-003-pass", &passing_preflight_input());
        let report = output_dir.join("preflight-report.json");
        let opt = testnet_order_preflight_opt(config, input, Some(report.clone()), true);

        run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string())).unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(report).unwrap()).unwrap();
        assert_eq!(report["status"], "pass");
        assert_eq!(report["order_submission_remains_disabled"], true);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["real_orders_submitted"], false);
    }

    #[test]
    fn testnet_order_preflight_blocks_missing_manual_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-003-preflight-missing-gates-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-preflight-missing-gates",
            &strategy_node_config(&output_dir),
        );
        let input = write_preflight_input("v100-003-missing-gates", &passing_preflight_input());
        let opt = testnet_order_preflight_opt(config, input, None, false);

        let error = run_live_testnet_order_preflight_with_env(&opt, |_| None)
            .unwrap_err()
            .to_string();

        assert!(error.contains("testnet order preflight blocked"));
        assert!(error.contains("--allow-testnet-order"));
        assert!(error.contains("NTPRO_ALLOW_BINANCE_TESTNET_ORDER"));
        assert!(error.contains("preflight_evaluated=false"));
        assert!(error.contains("network_attempted=false"));
        assert!(error.contains("real_orders_submitted=false"));
    }

    #[test]
    fn testnet_order_preflight_rejects_stale_market() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-003-preflight-stale-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-preflight-stale",
            &strategy_node_config(&output_dir),
        );
        let mut input = passing_preflight_input();
        input.market.now_unix_ms = 3_000;
        input.market.max_age_ms = 100;
        let input = write_preflight_input("v100-003-stale", &input);
        let opt = testnet_order_preflight_opt(config, input, None, true);

        let error = run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string()))
            .unwrap_err()
            .to_string();

        assert!(error.contains("market_stale"));
        assert!(error.contains("network_attempted=false"));
        assert!(error.contains("real_orders_submitted=false"));
    }

    #[test]
    fn testnet_order_preflight_rejects_kill_switch_active() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-003-preflight-kill-switch-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-preflight-kill-switch",
            &strategy_node_config(&output_dir),
        );
        let mut input = passing_preflight_input();
        input.risk.kill_switch_active = true;
        let input = write_preflight_input("v100-003-kill-switch", &input);
        let opt = testnet_order_preflight_opt(config, input, None, true);

        let error = run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string()))
            .unwrap_err()
            .to_string();

        assert!(error.contains("kill_switch_active"));
        assert!(error.contains("real_orders_submitted=false"));
    }

    #[test]
    fn testnet_order_preflight_rejects_production_endpoint() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-003-preflight-production-endpoint-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-preflight-production-endpoint",
            &strategy_node_config(&output_dir),
        );
        let mut input = passing_preflight_input();
        input.endpoint.http_base_url = "https://api.binance.com".to_string();
        input.endpoint.production_endpoint_allowed = true;
        let input = write_preflight_input("v100-003-production-endpoint", &input);
        let opt = testnet_order_preflight_opt(config, input, None, true);

        let error = run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string()))
            .unwrap_err()
            .to_string();

        assert!(error.contains("endpoint_not_testnet"));
        assert!(error.contains("production_endpoint_allowed"));
        assert!(error.contains("network_attempted=false"));
    }

    #[test]
    fn testnet_order_preflight_rejects_limit_violations() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-003-preflight-limit-violations-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-preflight-limit-violations",
            &strategy_node_config(&output_dir),
        );
        let mut input = passing_preflight_input();
        input.limits.max_order_notional = "0.00000001".to_string();
        input.limits.max_open_orders = 1;
        input.limits.open_order_count = 1;
        input.limits.max_clock_skew_ms = 10;
        input.limits.observed_clock_skew_ms = 25;
        let input = write_preflight_input("v100-003-limit-violations", &input);
        let opt = testnet_order_preflight_opt(config, input, None, true);

        let error = run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string()))
            .unwrap_err()
            .to_string();

        assert!(error.contains("notional_limit_exceeded"));
        assert!(error.contains("open_order_limit_exceeded"));
        assert!(error.contains("clock_skew_limit_exceeded"));
        assert!(error.contains("real_orders_submitted=false"));
    }

    #[test]
    fn testnet_signed_order_request_builder_constructs_order_test_preview() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-004-request-preview-{}",
            std::process::id()
        ));
        let path = write_config(
            "testnet-order-request-preview",
            &strategy_node_config(&output_dir),
        );
        let config = load_strategy_node_config(&path).unwrap();
        let testnet_order = config.testnet_order.as_ref().unwrap();
        let credentials = synthetic_order_credentials();

        let request = build_testnet_signed_order_request(
            testnet_order,
            &credentials,
            TESTNET_ORDER_METHOD_POST,
            TESTNET_ORDER_ENDPOINT_TEST,
            1_718_400_000_000,
            5_000,
            None,
        )
        .unwrap();

        assert_eq!(request.method, TESTNET_ORDER_METHOD_POST);
        assert_eq!(request.endpoint_path, TESTNET_ORDER_ENDPOINT_TEST);
        assert_eq!(request.action, "order_test");
        assert_eq!(request.api_key_header_name, BINANCE_API_KEY_HEADER);
        assert_eq!(
            request.api_key_header_value,
            "ntpro_v100004_synthetic_api_key_value"
        );
        assert!(request.query_without_signature.contains("symbol=BTCUSDT"));
        assert!(request.query_without_signature.contains("side=BUY"));
        assert!(request.query_without_signature.contains("type=LIMIT"));
        assert!(request.query_without_signature.contains("timeInForce=GTC"));
        assert!(
            request
                .signed_query
                .starts_with("symbol=BTCUSDT&side=BUY&type=LIMIT")
        );
        assert_eq!(request.signature.len(), 64);
        assert!(
            request
                .signature
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        );
        assert_eq!(
            request.endpoint_url_redacted,
            "https://testnet.binance.vision/api/v3/order/test"
        );
        request.ensure_preview_redacted(&credentials).unwrap();
    }

    #[test]
    fn testnet_signed_order_request_preview_redacts_all_sensitive_values() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-004-request-redaction-{}",
            std::process::id()
        ));
        let path = write_config(
            "testnet-order-request-redaction",
            &strategy_node_config(&output_dir),
        );
        let config = load_strategy_node_config(&path).unwrap();
        let testnet_order = config.testnet_order.as_ref().unwrap();
        let credentials = synthetic_order_credentials();
        let request = build_testnet_signed_order_request(
            testnet_order,
            &credentials,
            TESTNET_ORDER_METHOD_POST,
            TESTNET_ORDER_ENDPOINT_ORDER,
            1_718_400_000_000,
            5_000,
            None,
        )
        .unwrap();
        let preview_body = serde_json::to_string(&request.redacted_preview(&credentials)).unwrap();
        let debug_body = format!("{request:?}");

        for body in [&preview_body, &debug_body] {
            assert!(!body.contains("ntpro_v100004_synthetic_api_key_value"));
            assert!(!body.contains("ntpro_v100004_synthetic_api_secret_value"));
            assert!(!body.contains(&request.signature));
            assert!(!body.contains(&request.signed_query));
        }
        assert!(preview_body.contains("\"order_submission_remains_disabled\":true"));
        assert!(preview_body.contains("\"network_attempted\":false"));
        assert!(preview_body.contains("\"real_orders_submitted\":false"));
        assert!(preview_body.contains("\"signature_recorded\":false"));
        assert!(preview_body.contains("\"signed_query_recorded\":false"));
        assert!(preview_body.contains("\"signed_url_recorded\":false"));
        assert!(preview_body.contains("\"api_key_header_value_recorded\":false"));
    }

    #[test]
    fn testnet_signed_order_request_builder_rejects_non_allowlisted_endpoint() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-004-request-bad-endpoint-{}",
            std::process::id()
        ));
        let path = write_config(
            "testnet-order-request-bad-endpoint",
            &strategy_node_config(&output_dir),
        );
        let config = load_strategy_node_config(&path).unwrap();
        let testnet_order = config.testnet_order.as_ref().unwrap();
        let credentials = synthetic_order_credentials();

        let error = build_testnet_signed_order_request(
            testnet_order,
            &credentials,
            TESTNET_ORDER_METHOD_POST,
            "/api/v3/account",
            1_718_400_000_000,
            5_000,
            None,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("signed order request allowlist only includes"));
        assert!(error.contains("POST /api/v3/account"));
    }

    #[test]
    fn testnet_signed_order_request_builder_rejects_production_base_url() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-004-request-production-base-{}",
            std::process::id()
        ));
        let path = write_config(
            "testnet-order-request-production-base",
            &strategy_node_config(&output_dir),
        );
        let mut config = load_strategy_node_config(&path).unwrap();
        let testnet_order = config.testnet_order.as_mut().unwrap();
        testnet_order.http_base_url = "https://api.binance.com".to_string();
        let credentials = synthetic_order_credentials();

        let error = build_testnet_signed_order_request(
            testnet_order,
            &credentials,
            TESTNET_ORDER_METHOD_POST,
            TESTNET_ORDER_ENDPOINT_TEST,
            1_718_400_000_000,
            5_000,
            None,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("testnet_order.http_base_url"));
        assert!(error.contains(BINANCE_TESTNET_HTTP_BASE_URL));
    }

    #[test]
    fn testnet_signed_order_request_builder_fails_closed_without_secret() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-004-request-missing-secret-{}",
            std::process::id()
        ));
        let path = write_config(
            "testnet-order-request-missing-secret",
            &strategy_node_config(&output_dir),
        );
        let config = load_strategy_node_config(&path).unwrap();
        let testnet_order = config.testnet_order.as_ref().unwrap();
        let credentials = EnvOnlyTestnetOrderCredentials::from_values(
            "NTPRO_V100004_API_KEY".to_string(),
            Some("ntpro_v100004_synthetic_api_key_value".to_string()),
            "NTPRO_V100004_API_SECRET".to_string(),
            None,
        );

        let error = build_testnet_signed_order_request(
            testnet_order,
            &credentials,
            TESTNET_ORDER_METHOD_POST,
            TESTNET_ORDER_ENDPOINT_TEST,
            1_718_400_000_000,
            5_000,
            None,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("requires API secret env value"));
    }

    #[test]
    fn testnet_signed_order_request_builder_requires_cancel_client_order_id() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-004-request-cancel-missing-id-{}",
            std::process::id()
        ));
        let path = write_config(
            "testnet-order-request-cancel-missing-id",
            &strategy_node_config(&output_dir),
        );
        let config = load_strategy_node_config(&path).unwrap();
        let testnet_order = config.testnet_order.as_ref().unwrap();
        let credentials = synthetic_order_credentials();

        let error = build_testnet_signed_order_request(
            testnet_order,
            &credentials,
            TESTNET_ORDER_METHOD_DELETE,
            TESTNET_ORDER_ENDPOINT_ORDER,
            1_718_400_000_000,
            5_000,
            None,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("requires --orig-client-order-id"));
    }

    #[test]
    fn testnet_signed_order_request_builder_constructs_cancel_preview() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-004-request-cancel-{}",
            std::process::id()
        ));
        let path = write_config(
            "testnet-order-request-cancel",
            &strategy_node_config(&output_dir),
        );
        let config = load_strategy_node_config(&path).unwrap();
        let testnet_order = config.testnet_order.as_ref().unwrap();
        let credentials = synthetic_order_credentials();

        let request = build_testnet_signed_order_request(
            testnet_order,
            &credentials,
            TESTNET_ORDER_METHOD_DELETE,
            TESTNET_ORDER_ENDPOINT_ORDER,
            1_718_400_000_000,
            5_000,
            Some("ntpro-cancel-001"),
        )
        .unwrap();

        assert_eq!(request.method, TESTNET_ORDER_METHOD_DELETE);
        assert_eq!(request.endpoint_path, TESTNET_ORDER_ENDPOINT_ORDER);
        assert_eq!(request.action, "cancel");
        assert!(request.query_without_signature.contains("symbol=BTCUSDT"));
        assert!(
            request
                .query_without_signature
                .contains("origClientOrderId=ntpro-cancel-001")
        );
        assert!(
            !request
                .query_without_signature
                .contains("newOrderRespType=ACK")
        );
        request.ensure_preview_redacted(&credentials).unwrap();
    }

    #[test]
    fn testnet_signed_order_request_preview_command_writes_redacted_artifact() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-004-request-command-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-request-command",
            &strategy_node_config(&output_dir),
        );
        let output = output_dir.join("request-preview.json");
        let opt = testnet_order_request_preview_opt(config, Some(output.clone()), true);

        run_live_testnet_order_request_preview_with_env(&opt, |name| match name {
            TESTNET_ORDER_ENV_ALLOW
            | TESTNET_ORDER_ENV_OWNER_APPROVED
            | TESTNET_ORDER_ENV_TINY_NOTIONAL
            | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
            "NTPRO_V100004_API_KEY" => Some("ntpro_v100004_synthetic_api_key_value".to_string()),
            "NTPRO_V100004_API_SECRET" => {
                Some("ntpro_v100004_synthetic_api_secret_value".to_string())
            }
            _ => None,
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(body.contains(TESTNET_ORDER_PREVIEW_SCHEMA_VERSION));
        assert!(body.contains("\"order_action\": \"order_test\""));
        assert!(body.contains("\"network_attempted\": false"));
        assert!(body.contains("\"real_orders_submitted\": false"));
        assert!(!body.contains("ntpro_v100004_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v100004_synthetic_api_secret_value"));
    }

    #[test]
    fn testnet_order_test_preflight_command_writes_redacted_report() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-005-order-test-preflight-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-test-preflight",
            &strategy_node_config(&output_dir),
        );
        let output = output_dir.join("order-test-preflight.json");
        let opt = testnet_order_test_preflight_opt(config, Some(output.clone()), true);

        run_live_testnet_order_test_preflight_with_env(&opt, |name| match name {
            TESTNET_ORDER_ENV_ALLOW
            | TESTNET_ORDER_ENV_OWNER_APPROVED
            | TESTNET_ORDER_ENV_TINY_NOTIONAL
            | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
            "NTPRO_V100005_API_KEY" => Some("ntpro_v100005_synthetic_api_key_value".to_string()),
            "NTPRO_V100005_API_SECRET" => {
                Some("ntpro_v100005_synthetic_api_secret_value".to_string())
            }
            _ => None,
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(body.contains("ntpro.v100_order_test_preflight_report.v1"));
        assert!(body.contains("\"status\": \"ready\""));
        assert!(body.contains("\"request_method\": \"POST\""));
        assert!(body.contains("\"request_target\": \"/api/v3/order/test\""));
        assert!(
            body.contains(
                "\"binance_order_test_acceptance\": \"not_attempted_offline_manual_only\""
            )
        );
        assert!(body.contains("\"matching_engine_submission\": false"));
        assert!(body.contains("\"network_attempted\": false"));
        assert!(body.contains("\"real_orders_submitted\": false"));
        assert!(!body.contains("ntpro_v100005_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v100005_synthetic_api_secret_value"));
    }

    #[test]
    fn testnet_order_test_preflight_blocks_missing_manual_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-005-order-test-missing-gates-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-test-missing-gates",
            &strategy_node_config(&output_dir),
        );
        let opt = testnet_order_test_preflight_opt(config, None, false);

        let error = run_live_testnet_order_test_preflight_with_env(&opt, |_| None)
            .unwrap_err()
            .to_string();

        assert!(error.contains("testnet order-test preflight blocked"));
        assert!(error.contains("--allow-testnet-order"));
        assert!(error.contains("matching_engine_submission=false"));
        assert!(error.contains("network_attempted=false"));
        assert!(error.contains("real_orders_submitted=false"));
    }

    #[test]
    fn testnet_order_test_preflight_fails_closed_without_secret() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-005-order-test-missing-secret-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-order-test-missing-secret",
            &strategy_node_config(&output_dir),
        );
        let opt = testnet_order_test_preflight_opt(config, None, true);

        let error = run_live_testnet_order_test_preflight_with_env(&opt, |name| match name {
            TESTNET_ORDER_ENV_ALLOW
            | TESTNET_ORDER_ENV_OWNER_APPROVED
            | TESTNET_ORDER_ENV_TINY_NOTIONAL
            | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
            "NTPRO_V100005_API_KEY" => Some("ntpro_v100005_synthetic_api_key_value".to_string()),
            _ => None,
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("requires API secret env value"));
    }

    #[test]
    fn testnet_order_test_preflight_rejects_production_base_url() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-005-order-test-production-base-{}",
            std::process::id()
        ));
        let config = strategy_node_config(&output_dir).replace(
            r#"http_base_url = "https://testnet.binance.vision""#,
            r#"http_base_url = "https://api.binance.com""#,
        );
        let config = write_config("testnet-order-test-production-base", &config);
        let opt = testnet_order_test_preflight_opt(config, None, true);

        let error = run_live_testnet_order_test_preflight_with_env(&opt, |name| match name {
            TESTNET_ORDER_ENV_ALLOW
            | TESTNET_ORDER_ENV_OWNER_APPROVED
            | TESTNET_ORDER_ENV_TINY_NOTIONAL
            | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT
            | "NTPRO_V100005_API_KEY"
            | "NTPRO_V100005_API_SECRET" => Some("1".to_string()),
            _ => None,
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("testnet_order.http_base_url"));
        assert!(error.contains(BINANCE_TESTNET_HTTP_BASE_URL));
    }

    #[test]
    fn testnet_execution_artifact_contract_writes_redacted_report() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-007-artifact-contract-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-execution-artifact-contract",
            &strategy_node_config(&output_dir),
        );
        let output = output_dir.join("execution-artifact-contract.json");
        let opt = testnet_execution_artifact_contract_opt(config, Some(output.clone()), true);

        run_live_testnet_execution_artifact_contract_with_env(&opt, |name| match name {
            TESTNET_ORDER_ENV_ALLOW
            | TESTNET_ORDER_ENV_OWNER_APPROVED
            | TESTNET_ORDER_ENV_TINY_NOTIONAL
            | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
            "NTPRO_V100007_API_KEY" => Some("ntpro_v100007_synthetic_api_key_value".to_string()),
            "NTPRO_V100007_API_SECRET" => {
                Some("ntpro_v100007_synthetic_api_secret_value".to_string())
            }
            _ => None,
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(body.contains(TESTNET_EXECUTION_ARTIFACT_SCHEMA_VERSION));
        assert!(body.contains("\"artifact_family\": \"binance-testnet-order-lifecycle-proof\""));
        assert!(body.contains("\"name\": \"request.json\""));
        assert!(body.contains("\"name\": \"submit_ack.json\""));
        assert!(body.contains("\"name\": \"cancel_ack.json\""));
        assert!(body.contains("\"name\": \"lifecycle.json\""));
        assert!(body.contains("\"name\": \"reconciliation.json\""));
        assert!(body.contains("\"testnet_orders_submitted\": 0"));
        assert!(body.contains("\"testnet_orders_canceled\": 0"));
        assert!(body.contains("\"production_orders_submitted\": 0"));
        assert!(body.contains("\"production_orders_canceled\": 0"));
        assert!(body.contains("\"manual_submit_cancel_proof_observed\": false"));
        assert!(body.contains("\"network_attempted\": false"));
        assert!(body.contains("\"real_orders_submitted\": false"));
        assert!(!body.contains("ntpro_v100007_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v100007_synthetic_api_secret_value"));
    }

    #[test]
    fn testnet_execution_artifact_contract_blocks_missing_manual_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-007-artifact-missing-gates-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-execution-artifact-missing-gates",
            &strategy_node_config(&output_dir),
        );
        let opt = testnet_execution_artifact_contract_opt(config, None, false);

        let error = run_live_testnet_execution_artifact_contract_with_env(&opt, |_| None)
            .unwrap_err()
            .to_string();

        assert!(error.contains("testnet execution artifact contract blocked"));
        assert!(error.contains("--allow-testnet-order"));
        assert!(error.contains("artifact_built=false"));
        assert!(error.contains("network_attempted=false"));
        assert!(error.contains("real_orders_submitted=false"));
    }

    #[test]
    fn testnet_execution_artifact_contract_fails_closed_without_secret() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-007-artifact-missing-secret-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-execution-artifact-missing-secret",
            &strategy_node_config(&output_dir),
        );
        let opt = testnet_execution_artifact_contract_opt(config, None, true);

        let error =
            run_live_testnet_execution_artifact_contract_with_env(&opt, |name| match name {
                TESTNET_ORDER_ENV_ALLOW
                | TESTNET_ORDER_ENV_OWNER_APPROVED
                | TESTNET_ORDER_ENV_TINY_NOTIONAL
                | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
                "NTPRO_V100007_API_KEY" => {
                    Some("ntpro_v100007_synthetic_api_key_value".to_string())
                }
                _ => None,
            })
            .unwrap_err()
            .to_string();

        assert!(error.contains("requires API secret env value"));
    }

    #[test]
    fn testnet_reconciliation_fixture_writes_all_risk_halt_scenarios() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-008-reconciliation-all-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-reconciliation-all",
            &strategy_node_config(&output_dir),
        );
        let output = output_dir.join("reconciliation-fixture.json");

        run_live_testnet_reconciliation_fixture(&LiveTestnetReconciliationFixtureOpt {
            config,
            scenario: TestnetReconciliationScenario::All,
            output: Some(output.clone()),
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(body.contains(TESTNET_RECONCILIATION_FIXTURE_SCHEMA_VERSION));
        assert!(body.contains("\"status\": \"risk_halted\""));
        assert!(body.contains("\"scenario_count\": 4"));
        assert!(body.contains("\"name\": \"submit_without_local_ack\""));
        assert!(body.contains("\"name\": \"cancel_timeout\""));
        assert!(body.contains("\"name\": \"local_open_exchange_filled\""));
        assert!(body.contains("\"name\": \"restart_unfinished_order\""));
        assert!(body.contains("\"risk_halted\": true"));
        assert!(body.contains("\"new_orders_blocked\": true"));
        assert!(body.contains("\"testnet_orders_submitted\": 0"));
        assert!(body.contains("\"production_orders_submitted\": 0"));
        assert!(body.contains("\"network_attempted\": false"));
        assert!(body.contains("\"real_orders_submitted\": false"));
    }

    #[test]
    fn testnet_reconciliation_fixture_filters_single_scenario() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v100-008-reconciliation-one-{}",
            std::process::id()
        ));
        let config = write_config(
            "testnet-reconciliation-one",
            &strategy_node_config(&output_dir),
        );
        let output = output_dir.join("reconciliation-fixture.json");

        run_live_testnet_reconciliation_fixture(&LiveTestnetReconciliationFixtureOpt {
            config,
            scenario: TestnetReconciliationScenario::CancelTimeout,
            output: Some(output.clone()),
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(body.contains("\"scenario\": \"cancel_timeout\""));
        assert!(body.contains("\"scenario_count\": 1"));
        assert!(body.contains("\"name\": \"cancel_timeout\""));
        assert!(!body.contains("\"name\": \"submit_without_local_ack\""));
    }

    #[test]
    fn production_public_read_probe_blocks_missing_gates_without_network() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v110-002-public-read-blocked-{}",
            std::process::id()
        ));
        let output = output_dir.join("public-read-probe.json");
        let opt = production_public_read_probe_opt(
            ProductionPublicReadEndpoint::ServerTime,
            Some(output.clone()),
            false,
            false,
        );

        run_live_production_public_read_probe_with_env(&opt, |_| None).unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            report["schema_version"],
            PRODUCTION_PUBLIC_READ_PROBE_SCHEMA_VERSION
        );
        assert_eq!(report["status"], "blocked_missing_gate");
        assert_eq!(report["endpoint_class"], "production_public_read_only");
        assert_eq!(report["method"], "GET");
        assert_eq!(report["path"], "/api/v3/time");
        assert_eq!(report["requires_api_key"], false);
        assert_eq!(report["requires_signature"], false);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["mutation_allowed"], false);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["production_public_online_read_attempted"], false);
        assert_eq!(report["response_status_code"], serde_json::Value::Null);
        assert_eq!(report["response_shape"], "binance_server_time_v1");
        assert_eq!(report["response_shape_validated"], false);
        assert_eq!(report["latency_ms"], serde_json::Value::Null);
        assert_eq!(report["error_code"], "not_attempted");
        assert_eq!(report["credentials_used"], false);
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_public_read_probe_writes_ready_offline_contract() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v110-002-public-read-ready-{}",
            std::process::id()
        ));
        let output = output_dir.join("public-read-probe.json");
        let opt = production_public_read_probe_opt(
            ProductionPublicReadEndpoint::ExchangeInfo,
            Some(output.clone()),
            true,
            false,
        );

        run_live_production_public_read_probe_with_env(&opt, |_| Some("1".to_string())).unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(report["status"], "ready_offline_contract");
        assert_eq!(report["endpoint"], "exchange_info");
        assert_eq!(report["path"], "/api/v3/exchangeInfo");
        assert_eq!(report["read_allowed"], true);
        assert_eq!(report["contract_ready"], true);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["mutation_allowed"], false);
        assert_eq!(report["online_execution_supported"], false);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["production_public_online_read_attempted"], false);
        assert_eq!(report["response_status_code"], serde_json::Value::Null);
        assert_eq!(report["response_shape"], "binance_exchange_info_v1");
        assert_eq!(report["response_shape_validated"], false);
        assert_eq!(report["latency_ms"], serde_json::Value::Null);
        assert_eq!(report["error_code"], "not_attempted");
        assert_eq!(report["credentials_used"], false);
        assert_eq!(report["account_mutation_attempted"], false);
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_public_read_probe_blocks_manual_online_without_v12_owner_gate() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-001-public-read-online-blocked-{}",
            std::process::id()
        ));
        let output = output_dir.join("public-read-probe.json");
        let opt = production_public_read_probe_opt(
            ProductionPublicReadEndpoint::ServerTime,
            Some(output.clone()),
            true,
            true,
        );
        let mut http_called = false;
        let mut read_env = |name: &str| match name {
            PRODUCTION_PUBLIC_READ_ENV_MANUAL_ONLINE => None,
            _ => Some("1".to_string()),
        };

        run_live_production_public_read_probe_with_env_and_http(
            &opt,
            &mut read_env,
            |endpoint, _url| {
                http_called = true;
                ProductionPublicReadProbeHttpResult::success(endpoint, 1, 200)
            },
        )
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert!(!http_called);
        assert_eq!(
            report["schema_version"],
            PRODUCTION_PUBLIC_ONLINE_READ_PROBE_SCHEMA_VERSION
        );
        assert_eq!(report["status"], "blocked_missing_manual_online_gate");
        assert_eq!(report["manual_online_requested"], true);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["online_execution_supported"], true);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["production_public_online_read_attempted"], false);
        assert_eq!(report["response_status_code"], serde_json::Value::Null);
        assert_eq!(report["response_shape"], "binance_server_time_v1");
        assert_eq!(report["response_shape_validated"], false);
        assert_eq!(report["error_code"], "not_attempted");
        assert_eq!(report["production_order_mutation_attempted"], false);
    }

    #[test]
    fn production_public_read_probe_records_owner_gated_online_success_without_credentials() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-001-public-read-online-success-{}",
            std::process::id()
        ));
        let output = output_dir.join("public-read-probe.json");
        let opt = production_public_read_probe_opt(
            ProductionPublicReadEndpoint::ServerTime,
            Some(output.clone()),
            true,
            true,
        );
        let mut read_env = all_env_enabled;

        run_live_production_public_read_probe_with_env_and_http(
            &opt,
            &mut read_env,
            |endpoint, url| {
                assert_eq!(endpoint, ProductionPublicReadEndpoint::ServerTime);
                assert_eq!(url, "https://api.binance.com/api/v3/time");
                ProductionPublicReadProbeHttpResult::success(endpoint, 42, 200)
            },
        )
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            report["schema_version"],
            PRODUCTION_PUBLIC_ONLINE_READ_PROBE_SCHEMA_VERSION
        );
        assert_eq!(report["status"], "online_read_probe_ok");
        assert_eq!(report["endpoint"], "server_time");
        assert_eq!(report["method"], "GET");
        assert_eq!(report["path"], "/api/v3/time");
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], true);
        assert_eq!(report["online_execution_supported"], true);
        assert_eq!(report["network_attempted"], true);
        assert_eq!(report["production_public_online_read_attempted"], true);
        assert_eq!(report["response_status_code"], 200);
        assert_eq!(report["response_shape"], "binance_server_time_v1");
        assert_eq!(report["response_shape_validated"], true);
        assert_eq!(report["latency_ms"], 42);
        assert_eq!(report["error_code"], "none");
        assert_eq!(report["credentials_used"], false);
        assert_eq!(report["account_mutation_attempted"], false);
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_public_read_probe_records_owner_gated_online_failure_as_no_proof() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-001-public-read-online-failure-{}",
            std::process::id()
        ));
        let output = output_dir.join("public-read-probe.json");
        let opt = production_public_read_probe_opt(
            ProductionPublicReadEndpoint::ExchangeInfo,
            Some(output.clone()),
            true,
            true,
        );
        let mut read_env = all_env_enabled;

        run_live_production_public_read_probe_with_env_and_http(
            &opt,
            &mut read_env,
            |endpoint, url| {
                assert_eq!(endpoint, ProductionPublicReadEndpoint::ExchangeInfo);
                assert_eq!(url, "https://api.binance.com/api/v3/exchangeInfo");
                ProductionPublicReadProbeHttpResult::failure(
                    endpoint,
                    Some(7),
                    Some(503),
                    "http_status_not_success",
                )
            },
        )
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(report["status"], "online_read_probe_failed");
        assert_eq!(report["endpoint"], "exchange_info");
        assert_eq!(report["path"], "/api/v3/exchangeInfo");
        assert_eq!(report["online_read_allowed"], true);
        assert_eq!(report["network_attempted"], true);
        assert_eq!(report["production_public_online_read_attempted"], true);
        assert_eq!(report["response_status_code"], 503);
        assert_eq!(report["response_shape"], "binance_exchange_info_v1");
        assert_eq!(report["response_shape_validated"], false);
        assert_eq!(report["latency_ms"], 7);
        assert_eq!(report["error_code"], "http_status_not_success");
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_account_snapshot_contract_blocks_missing_gates_without_network() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v110-003-account-blocked-{}",
            std::process::id()
        ));
        let output = output_dir.join("account-snapshot-contract.json");
        let opt = production_account_snapshot_contract_opt(Some(output.clone()), false, false);

        run_live_production_account_snapshot_contract_with_env(&opt, |_| None).unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            report["schema_version"],
            PRODUCTION_ACCOUNT_SNAPSHOT_SCHEMA_VERSION
        );
        assert_eq!(report["status"], "blocked_missing_gate");
        assert_eq!(
            report["endpoint_class"],
            "production_authenticated_read_only"
        );
        assert_eq!(report["method"], "GET");
        assert_eq!(report["path"], "/api/v3/account");
        assert_eq!(report["requires_api_key"], true);
        assert_eq!(report["requires_signature"], true);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["mutation_allowed"], false);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["env_credentials_only"], true);
        assert_eq!(report["api_key_value_recorded"], false);
        assert_eq!(report["api_secret_value_recorded"], false);
        assert_eq!(report["signature_recorded"], false);
        assert_eq!(report["signed_query_recorded"], false);
        assert_eq!(report["signed_url_recorded"], false);
        assert_eq!(report["account_read_attempted"], false);
        assert_eq!(report["account_mutation_attempted"], false);
        assert_eq!(report["order_endpoint_access_attempted"], false);
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
        assert_eq!(report["secrets_redacted"], true);
    }

    #[test]
    fn production_account_snapshot_contract_blocks_missing_credentials() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v110-003-account-missing-credentials-{}",
            std::process::id()
        ));
        let output = output_dir.join("account-snapshot-contract.json");
        let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, false);

        run_live_production_account_snapshot_contract_with_env(&opt, |name| match name {
            PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE => Some("1".to_string()),
            _ => None,
        })
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(report["status"], "blocked_missing_credentials");
        assert_eq!(report["api_key_present"], false);
        assert_eq!(report["api_secret_present"], false);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
    }

    #[test]
    fn production_account_snapshot_contract_writes_ready_offline_redacted_contract() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v110-003-account-ready-{}",
            std::process::id()
        ));
        let output = output_dir.join("account-snapshot-contract.json");
        let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, false);

        run_live_production_account_snapshot_contract_with_env(&opt, |name| match name {
            PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE => Some("1".to_string()),
            "NTPRO_V110003_API_KEY" => Some("ntpro_v110003_synthetic_api_key_value".to_string()),
            "NTPRO_V110003_API_SECRET" => {
                Some("ntpro_v110003_synthetic_api_secret_value".to_string())
            }
            _ => None,
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v110003_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v110003_synthetic_api_secret_value"));
        let report: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(report["status"], "ready_offline_contract");
        assert_eq!(report["read_allowed"], true);
        assert_eq!(report["contract_ready"], true);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["api_key_present"], true);
        assert_eq!(report["api_secret_present"], true);
        assert_eq!(report["api_key_value_recorded"], false);
        assert_eq!(report["api_secret_value_recorded"], false);
        assert_eq!(report["signature_recorded"], false);
        assert_eq!(report["signed_query_recorded"], false);
        assert_eq!(report["signed_url_recorded"], false);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["account_read_attempted"], false);
        assert_eq!(report["order_endpoint_access_attempted"], false);
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
        assert_eq!(report["secrets_redacted"], true);
    }

    #[test]
    fn production_account_snapshot_contract_blocks_manual_online_without_v12_owner_gate() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-002-account-online-blocked-{}",
            std::process::id()
        ));
        let output = output_dir.join("account-snapshot-contract.json");
        let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, true);
        let mut http_called = false;

        let mut read_env = |name: &str| match name {
            PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE => None,
            PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION
            | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE => Some("1".to_string()),
            "NTPRO_V110003_API_KEY" => Some("ntpro_v120002_synthetic_api_key_value".to_string()),
            "NTPRO_V110003_API_SECRET" => {
                Some("ntpro_v120002_synthetic_api_secret_value".to_string())
            }
            _ => None,
        };

        run_live_production_account_snapshot_contract_with_env_and_http(
            &opt,
            &mut read_env,
            |_credentials, _recv_window_ms| {
                http_called = true;
                ProductionAccountSnapshotHttpResult::success(1, 200)
            },
        )
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert!(!http_called);
        assert_eq!(
            report["schema_version"],
            PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION
        );
        assert_eq!(report["status"], "blocked_missing_manual_online_gate");
        assert_eq!(report["manual_online_requested"], true);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["online_execution_supported"], true);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["account_read_attempted"], false);
        assert_eq!(report["response_shape"], "binance_account_snapshot_v1");
        assert_eq!(report["response_shape_validated"], false);
        assert_eq!(report["error_code"], "not_attempted");
        assert_eq!(report["production_order_mutation_attempted"], false);
    }

    #[test]
    fn production_account_snapshot_contract_records_owner_gated_online_success() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-002-account-online-success-{}",
            std::process::id()
        ));
        let output = output_dir.join("account-snapshot-contract.json");
        let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, true);
        let mut read_env = |name: &str| match name {
            "NTPRO_V110003_API_KEY" => Some("ntpro_v120002_synthetic_api_key_value".to_string()),
            "NTPRO_V110003_API_SECRET" => {
                Some("ntpro_v120002_synthetic_api_secret_value".to_string())
            }
            _ => all_env_enabled(name),
        };

        run_live_production_account_snapshot_contract_with_env_and_http(
            &opt,
            &mut read_env,
            |credentials, recv_window_ms| {
                assert!(credentials.api_key_present());
                assert!(credentials.api_secret_present());
                assert_eq!(recv_window_ms, 5_000);
                ProductionAccountSnapshotHttpResult::success(53, 200)
            },
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v120002_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v120002_synthetic_api_secret_value"));
        let report: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            report["schema_version"],
            PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION
        );
        assert_eq!(report["status"], "online_account_snapshot_ok");
        assert_eq!(report["method"], "GET");
        assert_eq!(report["path"], "/api/v3/account");
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], true);
        assert_eq!(report["online_execution_supported"], true);
        assert_eq!(report["network_attempted"], true);
        assert_eq!(report["account_read_attempted"], true);
        assert_eq!(report["response_status_code"], 200);
        assert_eq!(report["response_shape"], "binance_account_snapshot_v1");
        assert_eq!(report["response_shape_validated"], true);
        assert_eq!(report["latency_ms"], 53);
        assert_eq!(report["error_code"], "none");
        assert_eq!(report["signature_recorded"], false);
        assert_eq!(report["signed_query_recorded"], false);
        assert_eq!(report["signed_url_recorded"], false);
        assert_eq!(report["account_mutation_attempted"], false);
        assert_eq!(report["order_endpoint_access_attempted"], false);
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_account_snapshot_contract_records_owner_gated_online_failure_as_no_proof() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-002-account-online-failure-{}",
            std::process::id()
        ));
        let output = output_dir.join("account-snapshot-contract.json");
        let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, true);
        let mut read_env = |name: &str| match name {
            "NTPRO_V110003_API_KEY" => Some("ntpro_v120002_synthetic_api_key_value".to_string()),
            "NTPRO_V110003_API_SECRET" => {
                Some("ntpro_v120002_synthetic_api_secret_value".to_string())
            }
            _ => all_env_enabled(name),
        };

        run_live_production_account_snapshot_contract_with_env_and_http(
            &opt,
            &mut read_env,
            |_credentials, _recv_window_ms| {
                ProductionAccountSnapshotHttpResult::failure(
                    Some(9),
                    Some(401),
                    "http_status_not_success",
                )
            },
        )
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(report["status"], "online_account_snapshot_failed");
        assert_eq!(report["online_read_allowed"], true);
        assert_eq!(report["network_attempted"], true);
        assert_eq!(report["account_read_attempted"], true);
        assert_eq!(report["response_status_code"], 401);
        assert_eq!(report["response_shape"], "binance_account_snapshot_v1");
        assert_eq!(report["response_shape_validated"], false);
        assert_eq!(report["latency_ms"], 9);
        assert_eq!(report["error_code"], "http_status_not_success");
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_account_snapshot_signed_request_redacts_secret_values() {
        let credentials = EnvOnlyProductionReadCredentials::from_values(
            "NTPRO_V120002_API_KEY".to_string(),
            Some("ntpro_v120002_synthetic_api_key_value".to_string()),
            "NTPRO_V120002_API_SECRET".to_string(),
            Some("ntpro_v120002_synthetic_api_secret_value".to_string()),
        );
        let request = build_production_account_snapshot_signed_request(
            &credentials,
            1_718_400_000_000,
            5_000,
        )
        .unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.endpoint_path, "/api/v3/account");
        assert_eq!(request.api_key_header_name, BINANCE_API_KEY_HEADER);
        assert_eq!(
            request.query_without_signature,
            "timestamp=1718400000000&recvWindow=5000"
        );
        assert!(
            request
                .signed_query
                .starts_with("timestamp=1718400000000&recvWindow=5000&signature=")
        );
        assert_eq!(request.signature.len(), 64);
        assert!(
            request
                .signature
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        );
        assert_eq!(
            request.endpoint_url_redacted,
            "https://api.binance.com/api/v3/account"
        );

        let debug_body = format!("{request:?}");
        assert!(!debug_body.contains("ntpro_v120002_synthetic_api_key_value"));
        assert!(!debug_body.contains("ntpro_v120002_synthetic_api_secret_value"));
        assert!(!debug_body.contains(&request.signature));
        assert!(!debug_body.contains(&request.signed_query));
    }

    #[test]
    fn production_account_snapshot_shape_summary_accepts_expected_shape() {
        let body = serde_json::json!({
            "accountType": "SPOT",
            "canTrade": true,
            "canWithdraw": false,
            "canDeposit": true,
            "permissions": ["SPOT"],
            "balances": [
                {"asset": "BTC", "free": "0.12345678", "locked": "0.00000000"},
                {"asset": "USDT", "free": "100.00", "locked": "0.00"}
            ]
        });

        let summary = summarize_production_account_snapshot_shape(&body);

        assert!(summary.shape_validated);
        assert_eq!(summary.status, "accepted");
        assert_eq!(summary.balance_entry_count, Some(2));
        assert_eq!(summary.permission_entry_count, Some(1));
        assert!(summary.account_type_is_string);
        assert!(summary.balance_entry_shape_validated);
        assert!(summary.permission_entry_shape_validated);
        assert!(summary.can_trade_is_bool);
        assert!(summary.can_withdraw_is_bool);
        assert!(summary.can_deposit_is_bool);
        assert!(!summary.raw_account_response_recorded);
        assert!(!summary.raw_balances_recorded);
        assert!(!summary.raw_permissions_recorded);

        let summary_body = serde_json::to_string(&summary).unwrap();
        assert!(!summary_body.contains("BTC"));
        assert!(!summary_body.contains("USDT"));
        assert!(!summary_body.contains("0.12345678"));
        assert!(!summary_body.contains("SPOT"));
    }

    #[test]
    fn production_account_snapshot_shape_summary_rejects_missing_required_fields() {
        let body = serde_json::json!({
            "accountType": "SPOT",
            "canTrade": true,
            "balances": [
                {"asset": "BTC", "free": "0.12345678"}
            ]
        });

        let summary = summarize_production_account_snapshot_shape(&body);

        assert!(!summary.shape_validated);
        assert_eq!(summary.status, "rejected");
        assert_eq!(
            summary.rejection_reason,
            "missing_or_invalid_required_fields"
        );
        assert!(summary.account_type_is_string);
        assert!(summary.balances_is_array);
        assert_eq!(summary.balance_entry_count, Some(1));
        assert!(!summary.balance_entry_shape_validated);
        assert!(!summary.permissions_present);
        assert!(!summary.permissions_is_array);
        assert!(summary.can_trade_is_bool);
        assert!(!summary.can_withdraw_present);
        assert!(!summary.can_deposit_present);
    }

    #[test]
    fn production_account_snapshot_online_invalid_shape_records_redacted_summary() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-003-account-shape-invalid-{}",
            std::process::id()
        ));
        let output = output_dir.join("account-snapshot-contract.json");
        let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, true);
        let mut read_env = |name: &str| match name {
            "NTPRO_V110003_API_KEY" => Some("ntpro_v120003_synthetic_api_key_value".to_string()),
            "NTPRO_V110003_API_SECRET" => {
                Some("ntpro_v120003_synthetic_api_secret_value".to_string())
            }
            _ => all_env_enabled(name),
        };
        let invalid_summary = summarize_production_account_snapshot_shape(&serde_json::json!({
            "accountType": "SPOT",
            "canTrade": true,
            "canWithdraw": false,
            "canDeposit": true,
            "balances": [
                {"asset": "ETH", "free": "1.50000000", "locked": "0.00000000"}
            ]
        }));

        run_live_production_account_snapshot_contract_with_env_and_http(
            &opt,
            &mut read_env,
            |_credentials, _recv_window_ms| {
                ProductionAccountSnapshotHttpResult::failure_with_shape(
                    Some(11),
                    Some(200),
                    "response_shape_invalid",
                    invalid_summary.clone(),
                )
            },
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v120003_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v120003_synthetic_api_secret_value"));
        assert!(!body.contains("ETH"));
        assert!(!body.contains("1.50000000"));
        let report: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(report["status"], "online_account_snapshot_failed");
        assert_eq!(report["error_code"], "response_shape_invalid");
        assert_eq!(report["response_shape_validated"], false);
        assert_eq!(report["response_shape_summary"]["status"], "rejected");
        assert_eq!(
            report["response_shape_summary"]["permissions_present"],
            false
        );
        assert_eq!(
            report["response_shape_summary"]["raw_account_response_recorded"],
            false
        );
        assert_eq!(
            report["response_shape_summary"]["raw_balances_recorded"],
            false
        );
        assert_eq!(
            report["response_shape_summary"]["raw_permissions_recorded"],
            false
        );
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_shadow_portfolio_runtime_writes_redacted_runtime_and_compat_snapshot() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-004-shadow-portfolio-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let runtime_output = output_dir.join("shadow_portfolio_runtime.json");
        let compat_output = output_dir.join("shadow_portfolio_snapshot.json");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);

        run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
            run_id: "v120-shadow".to_string(),
            snapshot_id: Some("portfolio-1".to_string()),
            account_snapshot,
            shadow_intent,
            output: runtime_output.clone(),
            compat_snapshot_output: Some(compat_output.clone()),
        })
        .unwrap();

        let runtime_body = fs::read_to_string(runtime_output).unwrap();
        assert!(!runtime_body.contains("\"asset\": \"BTC\""));
        assert!(!runtime_body.contains("\"free\":"));
        assert!(!runtime_body.contains("\"locked\":"));
        assert!(!runtime_body.contains("api_secret"));
        let runtime: serde_json::Value = serde_json::from_str(&runtime_body).unwrap();
        assert_eq!(
            runtime["schema_version"],
            PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION
        );
        assert_eq!(runtime["status"], "ready_redacted_shadow_portfolio");
        assert_eq!(runtime["balances"]["status"], "observed_shape_only");
        assert_eq!(runtime["balances"]["observed_balance_entry_count"], 2);
        assert_eq!(runtime["balances"]["asset_values_recorded"], false);
        assert_eq!(
            runtime["source_shadow_intent_refs"][0]["intent_id"],
            "intent-1"
        );
        assert_eq!(runtime["exposure"]["status"], "derived_from_shadow_intents");
        assert_eq!(runtime["exposure"]["notional"], "10");
        assert_eq!(runtime["pnl"]["status"], "unavailable");
        assert_eq!(runtime["risk_summary"]["new_orders_blocked"], true);
        assert_eq!(runtime["production_orders_submitted"], 0);
        assert_eq!(runtime["production_order_mutations_attempted"], 0);
        assert_eq!(runtime["dashboard_order_controls_enabled"], false);
        assert_eq!(runtime["full_production_portfolio_parity_claimed"], false);

        let compat: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(compat_output).unwrap()).unwrap();
        assert_eq!(
            compat["schema_version"],
            PRODUCTION_SHADOW_PORTFOLIO_COMPAT_SCHEMA_VERSION
        );
        assert_eq!(compat["snapshot_mode"], "production_readonly_shadow");
        assert_eq!(compat["balances"][0]["asset"], "redacted");
        assert_eq!(compat["exposure"]["status"], "derived_from_shadow_intents");
        assert_eq!(compat["pnl"]["status"], "unavailable");
        assert_eq!(compat["production_orders_submitted"], 0);
        assert_eq!(compat["dashboard_order_controls_enabled"], false);
        assert_eq!(compat["full_production_portfolio_parity_claimed"], false);
    }

    #[test]
    fn production_shadow_portfolio_runtime_rejects_raw_account_balances() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-004-shadow-portfolio-raw-account-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("raw-account.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        fs::write(
            &account_snapshot,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION,
                "balances": [{"asset": "BTC", "free": "1", "locked": "0"}],
                "response_shape_validated": true
            }))
            .unwrap(),
        )
        .unwrap();
        write_shadow_intent(&shadow_intent, false);

        let error = build_production_shadow_portfolio_runtime_report(
            "v120-shadow",
            None,
            &account_snapshot,
            &shadow_intent,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("requires a redacted account summary"));
    }

    #[test]
    fn production_shadow_portfolio_runtime_rejects_actual_submission_intents() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-004-shadow-portfolio-submission-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, true);

        let error = build_production_shadow_portfolio_runtime_report(
            "v120-shadow",
            None,
            &account_snapshot,
            &shadow_intent,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("actual_submission=true"));
    }

    #[test]
    fn production_shadow_strategy_session_writes_heartbeat_gap_and_stop_events() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-005-shadow-strategy-session-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let session_events = output_dir.join("shadow_strategy_session.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);
        run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
            run_id: "v120-shadow".to_string(),
            snapshot_id: Some("portfolio-1".to_string()),
            account_snapshot,
            shadow_intent,
            output: portfolio_runtime.clone(),
            compat_snapshot_output: None,
        })
        .unwrap();

        run_live_production_shadow_strategy_session(&LiveProductionShadowStrategySessionOpt {
            run_id: "v120-shadow".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            shadow_portfolio_runtime: portfolio_runtime,
            strategy_session_status: None,
            output: session_events.clone(),
            heartbeat_count: 2,
            stop_after_heartbeats: true,
            stop_file: None,
        })
        .unwrap();

        let events = read_jsonl_values(&session_events);
        assert_eq!(events.len(), 5);
        assert_eq!(
            events[0]["schema_version"],
            PRODUCTION_SHADOW_STRATEGY_SESSION_EVENT_SCHEMA_VERSION
        );
        assert_eq!(events[0]["event_type"], "shadow_strategy_session_started");
        assert_eq!(events[0]["state"], "degraded_artifact_gap");
        assert_eq!(events[0]["artifact_gap"]["status"], "not_provided");
        assert_eq!(
            events[1]["event_type"],
            "shadow_strategy_session_artifact_gap"
        );
        assert_eq!(events[2]["event_type"], "shadow_strategy_session_heartbeat");
        assert_eq!(events[2]["heartbeat_seq"], 1);
        assert_eq!(events[3]["event_type"], "shadow_strategy_session_heartbeat");
        assert_eq!(events[3]["heartbeat_seq"], 2);
        assert_eq!(events[4]["event_type"], "shadow_strategy_session_stopped");
        assert_eq!(events[4]["state"], "stopped");
        for event in &events {
            assert_eq!(event["production_order_submissions_attempted"], 0);
            assert_eq!(event["production_orders_submitted"], 0);
            assert_eq!(event["production_order_mutations_attempted"], 0);
            assert_eq!(event["production_order_state_reads_attempted"], 0);
            assert_eq!(event["listen_key_lifecycle_attempted"], 0);
            assert_eq!(event["dashboard_order_controls_enabled"], false);
            assert_eq!(event["values_are_exchange_truth"], false);
        }
    }

    #[test]
    fn production_shadow_strategy_session_consumes_existing_session_status() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-005-shadow-strategy-session-status-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let status_path = output_dir.join("strategy_session_status.json");
        let session_events = output_dir.join("shadow_strategy_session.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);
        run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
            run_id: "v120-shadow".to_string(),
            snapshot_id: Some("portfolio-1".to_string()),
            account_snapshot,
            shadow_intent,
            output: portfolio_runtime.clone(),
            compat_snapshot_output: None,
        })
        .unwrap();
        fs::write(
            &status_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "ntpro.v09_strategy_session_status.v1",
                "session_id": "session-1",
                "strategy_id": "ema_cross_btcusdt_v1",
                "state": "running",
                "reason": "fixture strategy running"
            }))
            .unwrap(),
        )
        .unwrap();

        run_live_production_shadow_strategy_session(&LiveProductionShadowStrategySessionOpt {
            run_id: "v120-shadow".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            shadow_portfolio_runtime: portfolio_runtime,
            strategy_session_status: Some(status_path.clone()),
            output: session_events.clone(),
            heartbeat_count: 1,
            stop_after_heartbeats: false,
            stop_file: None,
        })
        .unwrap();

        let events = read_jsonl_values(&session_events);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["state"], "running");
        assert!(events[0].get("artifact_gap").is_none());
        assert_eq!(
            events[0]["strategy_session_status_ref"]["path"],
            status_path.display().to_string()
        );
        assert_eq!(events[0]["strategy_session_status_ref"]["state"], "running");
        assert_eq!(events[1]["event_type"], "shadow_strategy_session_heartbeat");
    }

    #[test]
    fn production_shadow_strategy_session_rejects_mutating_portfolio_runtime() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-005-shadow-strategy-session-mutating-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let session_events = output_dir.join("shadow_strategy_session.jsonl");
        fs::write(
            &portfolio_runtime,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION,
                "status": "ready_redacted_shadow_portfolio",
                "production_orders_submitted": 1,
                "production_order_mutations_attempted": 0,
                "automatic_correction_orders_submitted": 0,
                "actual_submission_count": 0,
                "dashboard_order_controls_enabled": false,
                "full_production_portfolio_parity_claimed": false,
                "real_orders_submitted": false,
                "provenance": {
                    "values_are_exchange_truth": false
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let error = build_production_shadow_strategy_session_events(
            &LiveProductionShadowStrategySessionOpt {
                run_id: "v120-shadow".to_string(),
                session_id: Some("session-1".to_string()),
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                shadow_portfolio_runtime: portfolio_runtime,
                strategy_session_status: None,
                output: session_events,
                heartbeat_count: 1,
                stop_after_heartbeats: false,
                stop_file: None,
            },
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("production_orders_submitted > 0"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_live_init_smoke_writes_summary_and_events() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-drg-005-live-run-{}", std::process::id()));
        let path = write_config("run", &minimal_config(&output_dir));

        run_live_run(&LiveRunOpt {
            config: path,
            run_id: None,
            output: None,
        })
        .await
        .unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("command=live.run"));
        assert!(summary.contains("runtime_status=completed"));
        assert!(summary.contains("final_state=Stopped"));
        assert!(summary.contains("status_artifact="));
        assert!(summary.contains("metrics_artifact="));
        assert!(summary.contains("events_log="));
        assert!(summary.contains("external_venue_connection=false"));
        assert!(summary.contains("real_orders_submitted=false"));

        let events = fs::read_to_string(output_dir.join("logs").join("events.log")).unwrap();
        assert!(events.contains("phase=start status=ok"));
        assert!(events.contains("phase=stop status=ok"));
        let legacy_events = fs::read_to_string(output_dir.join("events.log")).unwrap();
        assert_eq!(legacy_events, events);

        let status: NodeStatus =
            serde_json::from_str(&fs::read_to_string(output_dir.join("status.json")).unwrap())
                .unwrap();
        assert_eq!(status.node_id, "live-init-smoke");
        assert_eq!(status.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(status.process_mode, ProcessMode::TestHarness);
        assert_eq!(status.execution_connection, ConnectionStatus::Disconnected);
        assert_eq!(
            status.generated_at.availability,
            nautilus_live::status::SnapshotAvailability::Available
        );
        assert_eq!(
            status.started_at.availability,
            nautilus_live::status::SnapshotAvailability::Available
        );
        assert_eq!(
            status.stopped_at.availability,
            nautilus_live::status::SnapshotAvailability::Available
        );
        assert!(!status.external_venue_connection);
        assert!(!status.real_orders_submitted);

        let metrics: NodeMetrics =
            serde_json::from_str(&fs::read_to_string(output_dir.join("metrics.json")).unwrap())
                .unwrap();
        assert_eq!(metrics.node_id, "live-init-smoke");
        assert_eq!(metrics.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(metrics.starts_total, 1);
        assert_eq!(metrics.stops_total, 1);
        assert_eq!(metrics.state_transitions_total, 2);
        assert_eq!(metrics.connection_counts.execution_disconnected, 1);
        assert!(!metrics.external_venue_connection);
        assert!(!metrics.real_orders_submitted);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_ntpro_node_writes_spawned_process_status() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-v02-004-node-run-{}", std::process::id()));
        let path = write_config("ntpro-node", &minimal_config(&output_dir));

        run_ntpro_node(path, Some("sandbox-a".to_string()), None, None)
            .await
            .unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("command=ntpro-node.run"));
        assert!(summary.contains("process_mode=spawned_process"));
        assert!(summary.contains("final_state=Stopped"));
        assert!(summary.contains("metrics_artifact="));
        assert!(summary.contains("shutdown_reason=start-stop"));

        let status: NodeStatus =
            serde_json::from_str(&fs::read_to_string(output_dir.join("status.json")).unwrap())
                .unwrap();
        assert_eq!(status.node_id, "sandbox-a");
        assert_eq!(status.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(status.process_mode, ProcessMode::SpawnedProcess);
        assert_eq!(
            status.config_path.availability,
            nautilus_live::status::SnapshotAvailability::Available
        );
        assert!(!status.external_venue_connection);
        assert!(!status.real_orders_submitted);

        let metrics: NodeMetrics =
            serde_json::from_str(&fs::read_to_string(output_dir.join("metrics.json")).unwrap())
                .unwrap();
        assert_eq!(metrics.node_id, "sandbox-a");
        assert_eq!(metrics.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(metrics.process_mode, ProcessMode::SpawnedProcess);
        assert_eq!(metrics.starts_total, 1);
        assert_eq!(metrics.stops_total, 1);
        assert!(
            metrics
                .status_artifact_path
                .value
                .as_deref()
                .unwrap()
                .ends_with("status.json")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_ntpro_node_hosts_strategy_session_artifacts() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-v090-009-node-run-{}", std::process::id()));
        let path = write_config("ntpro-node-strategy", &strategy_node_config(&output_dir));

        run_ntpro_node(path, None, None, None).await.unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("command=ntpro-node.run"));
        assert!(summary.contains("mode=shadow"));
        assert!(summary.contains("strategy_id=ema_cross_btcusdt_v1"));
        assert!(summary.contains("order_submission_allowed=false"));
        assert!(summary.contains("session_status_artifact="));
        assert!(summary.contains("signal_artifact="));
        assert!(summary.contains("order_intent_artifact="));
        assert!(summary.contains("risk_decision_artifact="));
        assert!(summary.contains("final_state=Stopped"));
        assert!(summary.contains("external_venue_connection=false"));
        assert!(summary.contains("real_orders_submitted=false"));

        let status: NodeStatus =
            serde_json::from_str(&fs::read_to_string(output_dir.join("status.json")).unwrap())
                .unwrap();
        assert_eq!(status.node_id, "btc-ema-shadow-001");
        assert_eq!(status.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(status.process_mode, ProcessMode::SpawnedProcess);
        assert_eq!(status.data_connection, ConnectionStatus::Disconnected);
        assert_eq!(status.execution_connection, ConnectionStatus::NotConfigured);
        assert!(!status.external_venue_connection);
        assert!(!status.real_orders_submitted);
        assert_eq!(status.risk.command_count.value, Some(2));
        assert_eq!(status.risk.event_count.value, Some(2));
        assert_eq!(status.risk.rejections_total.value, Some(2));

        let session_status: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(output_dir.join("strategy").join("session_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(session_status["state"], "stopped");
        assert_eq!(session_status["session_id"], "btc-ema-shadow-001");
        assert_eq!(session_status["strategy_id"], "ema_cross_btcusdt_v1");
        let market_status: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(output_dir.join("strategy").join("market_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(market_status["state"], "stopped");
        assert_eq!(market_status["connection"], "stopped");

        let signals = fs::read_to_string(output_dir.join("strategy").join("signal.jsonl")).unwrap();
        assert!(!signals.trim().is_empty());
        let intents =
            fs::read_to_string(output_dir.join("strategy").join("order_intent.jsonl")).unwrap();
        assert!(!intents.trim().is_empty());
        for line in intents.lines() {
            let intent: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(intent["submission_allowed"], false);
        }
        let decisions =
            fs::read_to_string(output_dir.join("strategy").join("risk_decision.jsonl")).unwrap();
        assert!(!decisions.trim().is_empty());
        for line in decisions.lines() {
            let decision: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(decision["decision"], "rejected");
            assert_eq!(decision["actual_submission"], false);
        }

        let events = fs::read_to_string(output_dir.join("logs").join("events.log")).unwrap();
        assert!(events.contains("phase=strategy_session_start status=ok"));
        assert!(events.contains("phase=strategy_session_stop status=ok"));

        let metrics: NodeMetrics =
            serde_json::from_str(&fs::read_to_string(output_dir.join("metrics.json")).unwrap())
                .unwrap();
        assert_eq!(metrics.node_id, "btc-ema-shadow-001");
        assert_eq!(metrics.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(metrics.process_mode, ProcessMode::SpawnedProcess);
        assert_eq!(metrics.strategy_signal_count.value, Some(2));
        assert_eq!(metrics.strategy_rejection_count.value, Some(2));
        assert!(!metrics.external_venue_connection);
        assert!(!metrics.real_orders_submitted);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_ntpro_node_keeps_strategy_session_running_until_shutdown() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-v091-003-node-run-{}", std::process::id()));
        let stop_file = output_dir.join("stop.request");
        let path = write_config(
            "ntpro-node-strategy-persistent",
            &strategy_node_config(&output_dir),
        );
        let session_status_path = output_dir.join("strategy").join("session_status.json");
        let node_status_path = output_dir.join("status.json");
        let node_metrics_path = output_dir.join("metrics.json");
        let stop_file_writer = stop_file.clone();
        let watcher = tokio::spawn(async move {
            for _ in 0..40 {
                if session_status_path.exists() {
                    let status: serde_json::Value =
                        serde_json::from_str(&fs::read_to_string(&session_status_path)?)?;
                    if status["state"] == "running"
                        && node_status_path.exists()
                        && node_metrics_path.exists()
                    {
                        let node_status: NodeStatus =
                            serde_json::from_str(&fs::read_to_string(&node_status_path)?)?;
                        let node_metrics: NodeMetrics =
                            serde_json::from_str(&fs::read_to_string(&node_metrics_path)?)?;
                        if node_status.lifecycle_state == LifecycleStatus::Running
                            && node_status.risk.command_count.value == Some(2)
                            && node_status.risk.event_count.value == Some(2)
                            && node_status.risk.rejections_total.value == Some(2)
                            && node_metrics.strategy_signal_count.value == Some(2)
                            && node_metrics.strategy_rejection_count.value == Some(2)
                        {
                            fs::write(&stop_file_writer, "stop\n")?;
                            return Ok::<_, anyhow::Error>(());
                        }
                    }
                }
                sleep(Duration::from_millis(50)).await;
            }
            anyhow::bail!(
                "strategy session heartbeat counters did not remain non-zero before shutdown"
            )
        });

        run_ntpro_node_with_controls(
            path,
            None,
            None,
            Some(stop_file),
            NtproNodeRunControls::from_millis(Some(3_000), 50, None, 3_000).unwrap(),
        )
        .await
        .unwrap();
        watcher.await.unwrap().unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("shutdown_reason=stop-file"));

        let session_status: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(output_dir.join("strategy").join("session_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(session_status["state"], "stopped");
        assert_eq!(session_status["reason"], "shutdown complete: stop-file");

        let events = fs::read_to_string(output_dir.join("strategy").join("events.jsonl")).unwrap();
        assert!(events.contains(r#""state":"running""#));
        assert!(events.contains("shutdown requested: stop-file"));
        assert!(events.contains("shutdown complete: stop-file"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_ntpro_node_stops_when_stop_file_is_written() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-p0-007-stop-file-{}", std::process::id()));
        let stop_file = output_dir.join("stop.request");
        let path = write_config("ntpro-node-stop-file", &minimal_config(&output_dir));
        let stop_file_writer = stop_file.clone();
        let writer = tokio::spawn(async move {
            sleep(Duration::from_millis(150)).await;
            fs::write(stop_file_writer, "stop\n").unwrap();
        });

        run_ntpro_node_with_controls(
            path,
            Some("sandbox-stop-file".to_string()),
            None,
            Some(stop_file),
            NtproNodeRunControls::from_millis(Some(2_000), 50, None, 3_000).unwrap(),
        )
        .await
        .unwrap();
        writer.await.unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("shutdown_reason=stop-file"));
        let events = fs::read_to_string(output_dir.join("logs").join("events.log")).unwrap();
        assert!(events.contains("phase=shutdown_trigger status=ok reason=stop-file"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_ntpro_node_stops_when_max_runtime_expires() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-p0-007-max-runtime-{}", std::process::id()));
        let stop_file = output_dir.join("missing-stop.request");
        let path = write_config("ntpro-node-max-runtime", &minimal_config(&output_dir));

        run_ntpro_node_with_controls(
            path,
            Some("sandbox-max-runtime".to_string()),
            None,
            Some(stop_file),
            NtproNodeRunControls::from_millis(Some(150), 50, None, 3_000).unwrap(),
        )
        .await
        .unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("shutdown_reason=max-runtime"));
        let events = fs::read_to_string(output_dir.join("logs").join("events.log")).unwrap();
        assert!(events.contains("phase=shutdown_trigger status=ok reason=max-runtime"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_ntpro_node_stops_when_parent_process_is_dead() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-p0-007-parent-dead-{}", std::process::id()));
        let stop_file = output_dir.join("missing-stop.request");
        let path = write_config("ntpro-node-parent-dead", &minimal_config(&output_dir));

        run_ntpro_node_with_controls(
            path,
            Some("sandbox-parent-dead".to_string()),
            None,
            Some(stop_file),
            NtproNodeRunControls::from_millis(Some(2_000), 50, Some(u32::MAX), 3_000).unwrap(),
        )
        .await
        .unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("shutdown_reason=parent-exited"));
        let events = fs::read_to_string(output_dir.join("logs").join("events.log")).unwrap();
        assert!(events.contains("phase=shutdown_trigger status=ok reason=parent-exited"));
    }

    #[test]
    fn rejects_external_venue_connection() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-drg-005-live-reject-{}", std::process::id()));
        let config = minimal_config(&output_dir).replace(
            "external_venue_connection = false",
            "external_venue_connection = true",
        );
        let path = write_config("reject", &config);

        let error = validate_minimal_live_config_file(&path)
            .unwrap_err()
            .to_string();

        assert!(error.contains("execution.external_venue_connection must be false"));
    }
}
