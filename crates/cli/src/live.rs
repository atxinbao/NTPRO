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
    fs::{self, OpenOptions},
    io::Write,
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
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use tokio::time::{sleep, timeout};

use crate::{
    artifacts::{atomic_write_json, atomic_write_text},
    endpoint_classifier::{EndpointAuthKind, EndpointClassifier, EndpointDecision},
    opt::{
        LiveCommand, LiveOpt, LiveProductionAccountSnapshotContractOpt,
        LiveProductionKillSwitchApprovalArtifactOpt, LiveProductionLiveAlphaDryRunOrderGateOpt,
        LiveProductionLiveAlphaExecutionDryRunOpt, LiveProductionLiveAlphaKillSwitchRuntimeGateOpt,
        LiveProductionLiveAlphaManualApprovalLifecycleOpt,
        LiveProductionLiveAlphaOrderRequestPreviewOpt, LiveProductionLiveAlphaRiskPreflightOpt,
        LiveProductionMutationAuditTrailOpt, LiveProductionMutationFailureSemanticsOpt,
        LiveProductionMutationGuardedSendOpt, LiveProductionMutationOrderStateReadbackOpt,
        LiveProductionMutationRequestBuilderOpt, LiveProductionMutationResponseRedactionOpt,
        LiveProductionMutationRuntimeGateOpt, LiveProductionMutationSigningApprovalOpt,
        LiveProductionOrderStateReadOnlyProofOpt, LiveProductionPublicReadProbeOpt,
        LiveProductionReadonlyReconciliationOpt, LiveProductionShadowPortfolioRuntimeOpt,
        LiveProductionShadowPreflightSessionOpt, LiveProductionShadowStrategySessionOpt,
        LiveRunOpt, LiveTestnetExecutionArtifactContractOpt, LiveTestnetOrderGateOpt,
        LiveTestnetOrderPreflightOpt, LiveTestnetOrderRequestPreviewOpt,
        LiveTestnetOrderTestPreflightOpt, LiveTestnetReconciliationFixtureOpt, LiveValidateOpt,
        ProductionMutationFailureMode, ProductionOrderStateReadEndpoint,
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
const PRODUCTION_ORDER_STATE_READONLY_SCHEMA_VERSION: &str =
    "ntpro.v140_production_order_state_readonly_proof.v1";
const PRODUCTION_ORDER_STATE_ENV_ALLOW: &str = "NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ";
const PRODUCTION_ORDER_STATE_ENV_OWNER_APPROVED: &str =
    "NTPRO_OWNER_APPROVED_PRODUCTION_ORDER_STATE_READ_ONLY";
const PRODUCTION_ORDER_STATE_ENV_NO_ORDER_MUTATION: &str =
    "NTPRO_CONFIRM_PRODUCTION_ORDER_STATE_NO_ORDER_MUTATION";
const PRODUCTION_ORDER_STATE_ENV_NO_SECRET_PERSISTENCE: &str =
    "NTPRO_CONFIRM_NO_SECRET_PERSISTENCE";
const PRODUCTION_ORDER_STATE_ENV_NO_LISTEN_KEY: &str = "NTPRO_CONFIRM_NO_LISTEN_KEY_LIFECYCLE";
const PRODUCTION_ORDER_STATE_ENV_DASHBOARD_DISABLED: &str =
    "NTPRO_CONFIRM_DASHBOARD_ORDER_CONTROLS_DISABLED";
const PRODUCTION_ORDER_STATE_ENV_MANUAL_ONLINE: &str = "NTPRO_V14_MANUAL_ONLINE";
const PRODUCTION_ORDER_STATE_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION: &str =
    "ntpro.v140_live_alpha_dry_run_order_gate.v1";
const PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION: &str =
    "ntpro.v150_live_alpha_order_request_preview.v1";
const PRODUCTION_MUTATION_PREVIEW_SYNTHETIC_API_KEY: &str = "ntpro_v151003_synthetic_api_key_value";
const PRODUCTION_MUTATION_PREVIEW_SYNTHETIC_API_SECRET: &str =
    "ntpro_v151003_synthetic_api_secret_value";
const PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW: &str =
    "NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL";
const PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED: &str =
    "NTPRO_OWNER_APPROVED_MUTATION_SIGNING_DRY_RUN";
const PRODUCTION_LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_SCHEMA_VERSION: &str =
    "ntpro.v150_live_alpha_manual_approval_lifecycle.v1";
const PRODUCTION_LIVE_ALPHA_EXECUTION_DRY_RUN_SCHEMA_VERSION: &str =
    "ntpro.v150_live_alpha_execution_dry_run.v1";
const PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION: &str =
    "ntpro.v150_live_alpha_kill_switch_runtime_gate.v1";
const PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_INPUT_SCHEMA_VERSION: &str =
    "ntpro.v140_live_alpha_risk_preflight_input.v1";
const PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_REPORT_SCHEMA_VERSION: &str =
    "ntpro.v140_live_alpha_risk_preflight.v1";
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
const PRODUCTION_MUTATION_HTTP_SEND_ENV_ALLOW: &str = "NTPRO_ALLOW_PRODUCTION_MUTATION_HTTP_SEND";
const PRODUCTION_MUTATION_HTTP_SEND_ENV_OWNER_APPROVED: &str =
    "NTPRO_OWNER_APPROVED_PRODUCTION_MUTATION_HTTP_SEND";
const PRODUCTION_MUTATION_HTTP_SEND_ENV_SINGLE_SHOT: &str =
    "NTPRO_CONFIRM_PRODUCTION_MUTATION_SINGLE_SHOT";
const PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION: &str =
    "ntpro.v120_shadow_portfolio_runtime.v1";
const PRODUCTION_SHADOW_PORTFOLIO_COMPAT_SCHEMA_VERSION: &str =
    "ntpro.v110_shadow_portfolio_snapshot.v1";
const PRODUCTION_SHADOW_STRATEGY_SESSION_EVENT_SCHEMA_VERSION: &str =
    "ntpro.v120_shadow_strategy_session_event.v1";
const PRODUCTION_SHADOW_PREFLIGHT_SESSION_EVENT_SCHEMA_VERSION: &str =
    "ntpro.v130_shadow_preflight_session_event.v1";
const PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION: &str =
    "ntpro.v130_kill_switch_approval_artifact.v1";
const PRODUCTION_READONLY_RECONCILIATION_EVENT_SCHEMA_VERSION: &str =
    "ntpro.v120_readonly_reconciliation_event.v1";
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

struct EnvOnlyProductionMutationPreviewCredentials {
    api_key_env: String,
    api_secret_env: String,
    credential_material: String,
    api_key_value: Option<String>,
    api_secret_value: Option<String>,
    production_signing_material_gate_required: bool,
    production_signing_material_gate_open: bool,
    production_signing_material_env_read: bool,
    production_signing_material_missing_gate_env_vars: Vec<String>,
    sensitive_values: Vec<String>,
}

struct ProductionMutationPreviewCredentialInput {
    api_key_env: String,
    api_key_value: Option<String>,
    api_secret_env: String,
    api_secret_value: Option<String>,
    credential_material: &'static str,
    production_signing_material_gate_required: bool,
    production_signing_material_gate_open: bool,
    production_signing_material_env_read: bool,
    production_signing_material_missing_gate_env_vars: Vec<String>,
}

impl EnvOnlyProductionMutationPreviewCredentials {
    fn from_order_request_preview_opt<F>(
        opt: &LiveProductionLiveAlphaOrderRequestPreviewOpt,
        mut read_env: F,
    ) -> anyhow::Result<Self>
    where
        F: FnMut(&str) -> Option<String>,
    {
        match opt.credential_material.trim() {
            "synthetic" => Ok(Self::from_values(
                ProductionMutationPreviewCredentialInput {
                    api_key_env: opt.api_key_env.clone(),
                    api_key_value: Some(PRODUCTION_MUTATION_PREVIEW_SYNTHETIC_API_KEY.to_string()),
                    api_secret_env: opt.api_secret_env.clone(),
                    api_secret_value: Some(
                        PRODUCTION_MUTATION_PREVIEW_SYNTHETIC_API_SECRET.to_string(),
                    ),
                    credential_material: "synthetic",
                    production_signing_material_gate_required: false,
                    production_signing_material_gate_open: false,
                    production_signing_material_env_read: false,
                    production_signing_material_missing_gate_env_vars: Vec::new(),
                },
            )),
            "production_live_alpha" => {
                let mut missing_gate_env_vars = Vec::new();
                if read_env(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW).as_deref() != Some("1")
                {
                    missing_gate_env_vars
                        .push(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW.to_string());
                }
                if read_env(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED).as_deref()
                    != Some("1")
                {
                    missing_gate_env_vars
                        .push(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED.to_string());
                }

                let production_signing_material_gate_open = missing_gate_env_vars.is_empty();
                let (api_key_value, api_secret_value, production_signing_material_env_read) =
                    if production_signing_material_gate_open {
                        (
                            read_env(&opt.api_key_env),
                            read_env(&opt.api_secret_env),
                            true,
                        )
                    } else {
                        (None, None, false)
                    };

                Ok(Self::from_values(
                    ProductionMutationPreviewCredentialInput {
                        api_key_env: opt.api_key_env.clone(),
                        api_key_value,
                        api_secret_env: opt.api_secret_env.clone(),
                        api_secret_value,
                        credential_material: "production_live_alpha",
                        production_signing_material_gate_required: true,
                        production_signing_material_gate_open,
                        production_signing_material_env_read,
                        production_signing_material_missing_gate_env_vars: missing_gate_env_vars,
                    },
                ))
            }
            other => anyhow::bail!(
                "production live-alpha request preview credential_material must be synthetic or production_live_alpha, got {other}"
            ),
        }
    }

    fn from_request_builder_opt<F>(
        opt: &LiveProductionMutationRequestBuilderOpt,
        mut read_env: F,
    ) -> Self
    where
        F: FnMut(&str) -> Option<String>,
    {
        let mut missing_gate_env_vars = Vec::new();
        if read_env(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW).as_deref() != Some("1") {
            missing_gate_env_vars.push(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW.to_string());
        }
        if read_env(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED).as_deref() != Some("1")
        {
            missing_gate_env_vars
                .push(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED.to_string());
        }

        let production_signing_material_gate_open = missing_gate_env_vars.is_empty();
        let (api_key_value, api_secret_value, production_signing_material_env_read) =
            if production_signing_material_gate_open {
                (
                    read_env(&opt.api_key_env),
                    read_env(&opt.api_secret_env),
                    true,
                )
            } else {
                (None, None, false)
            };

        Self::from_values(ProductionMutationPreviewCredentialInput {
            api_key_env: opt.api_key_env.clone(),
            api_key_value,
            api_secret_env: opt.api_secret_env.clone(),
            api_secret_value,
            credential_material: "production_live_alpha",
            production_signing_material_gate_required: true,
            production_signing_material_gate_open,
            production_signing_material_env_read,
            production_signing_material_missing_gate_env_vars: missing_gate_env_vars,
        })
    }

    fn from_guarded_send_opt<F>(opt: &LiveProductionMutationGuardedSendOpt, mut read_env: F) -> Self
    where
        F: FnMut(&str) -> Option<String>,
    {
        let mut missing_gate_env_vars = Vec::new();
        for env_name in [
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW,
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED,
            PRODUCTION_MUTATION_HTTP_SEND_ENV_ALLOW,
            PRODUCTION_MUTATION_HTTP_SEND_ENV_OWNER_APPROVED,
            PRODUCTION_MUTATION_HTTP_SEND_ENV_SINGLE_SHOT,
        ] {
            if read_env(env_name).as_deref() != Some("1") {
                missing_gate_env_vars.push(env_name.to_string());
            }
        }

        let production_signing_material_gate_open = missing_gate_env_vars.is_empty();
        let (api_key_value, api_secret_value, production_signing_material_env_read) =
            if production_signing_material_gate_open {
                (
                    read_env(&opt.api_key_env),
                    read_env(&opt.api_secret_env),
                    true,
                )
            } else {
                (None, None, false)
            };

        Self::from_values(ProductionMutationPreviewCredentialInput {
            api_key_env: opt.api_key_env.clone(),
            api_key_value,
            api_secret_env: opt.api_secret_env.clone(),
            api_secret_value,
            credential_material: "production_live_alpha",
            production_signing_material_gate_required: true,
            production_signing_material_gate_open,
            production_signing_material_env_read,
            production_signing_material_missing_gate_env_vars: missing_gate_env_vars,
        })
    }

    fn from_values(input: ProductionMutationPreviewCredentialInput) -> Self {
        let sensitive_values = [
            input.api_key_value.as_ref(),
            input.api_secret_value.as_ref(),
        ]
        .into_iter()
        .flatten()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect();

        Self {
            api_key_env: input.api_key_env,
            api_secret_env: input.api_secret_env,
            credential_material: input.credential_material.to_string(),
            api_key_value: input.api_key_value,
            api_secret_value: input.api_secret_value,
            production_signing_material_gate_required: input
                .production_signing_material_gate_required,
            production_signing_material_gate_open: input.production_signing_material_gate_open,
            production_signing_material_env_read: input.production_signing_material_env_read,
            production_signing_material_missing_gate_env_vars: input
                .production_signing_material_missing_gate_env_vars,
            sensitive_values,
        }
    }

    fn signing_credential(&self) -> anyhow::Result<SigningCredential> {
        let api_key = self
            .api_key_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("production live-alpha request preview requires API key env value")?;
        let api_secret = self
            .api_secret_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("production live-alpha request preview requires API secret env value")?;

        Ok(SigningCredential::new(
            api_key.to_string(),
            api_secret.to_string(),
        ))
    }

    fn ensure_no_secret_values_absent(&self, label: &str, body: &str) -> anyhow::Result<()> {
        for secret_value in &self.sensitive_values {
            if body.contains(secret_value) {
                anyhow::bail!(
                    "production live-alpha request preview redaction guard blocked secret value leak in {label}"
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
    endpoint_read_allowed: bool,
    offline_contract_ready: bool,
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
    endpoint_read_allowed: bool,
    offline_contract_ready: bool,
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
struct ProductionOrderStateReadOnlyProofReport {
    schema_version: String,
    status: String,
    endpoint: String,
    endpoint_class: String,
    http_base_url: String,
    method: String,
    path: String,
    request_url_redacted: String,
    query_shape: String,
    symbol: String,
    order_id_provided: bool,
    orig_client_order_id_provided: bool,
    requires_api_key: bool,
    requires_signature: bool,
    endpoint_read_allowed: bool,
    offline_contract_ready: bool,
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
    response_shape_summary: ProductionOrderStateShapeSummary,
    endpoint_shape_validated: bool,
    order_entries_observed: usize,
    non_empty_order_state_observed: bool,
    order_lifecycle_readiness: bool,
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
    order_state_read_attempted: bool,
    production_order_state_reads_attempted: u64,
    production_order_submission_attempted: bool,
    production_order_mutation_attempted: bool,
    cancel_replace_amend_attempted: bool,
    listen_key_lifecycle_attempted: bool,
    dashboard_order_controls_enabled: bool,
    automatic_remediation_attempted: bool,
    real_orders_submitted: bool,
    production_trading_enabled: bool,
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
    values_are_exchange_truth: bool,
    secrets_redacted: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionLiveAlphaDryRunOrderGateArtifact {
    schema_version: String,
    run_id: String,
    session_id: String,
    strategy_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    notional: String,
    owner_gate_required: bool,
    manual_gate_required: bool,
    missing_cli_flags: Vec<String>,
    dry_run_order_intent_recorded: bool,
    dry_run_order_gate_ready: bool,
    order_submission_mode: String,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_called: bool,
    matching_engine_submission: bool,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    external_venue_connection: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
    values_are_exchange_truth: bool,
    no_production_order_submission_confirmed: bool,
    no_production_order_mutation_confirmed: bool,
    no_execution_adapter_call_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_real_funds_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionLiveAlphaManualApprovalLifecycleArtifact {
    schema_version: String,
    run_id: String,
    strategy_id: String,
    symbol: String,
    notional: String,
    artifact_type: String,
    status: String,
    created_at: String,
    approval_state: String,
    manual_approval_recorded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    now_unix_ms: u64,
    expires_at_unix_ms: u64,
    approval_expired: bool,
    approval_revoked: bool,
    approval_used: bool,
    dry_run_request_preview_only: bool,
    one_time_approval: bool,
    approval_lifecycle_valid: bool,
    lifecycle_issues: Vec<String>,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_called: bool,
    production_adapter_called: bool,
    matching_engine_submission: bool,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    external_venue_connection: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    dry_run_request_preview_only_confirmed: bool,
    one_time_approval_confirmed: bool,
    no_production_mutation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionLiveAlphaOrderRequestPreviewArtifact {
    schema_version: String,
    run_id: String,
    session_id: String,
    strategy_id: String,
    source_order_gate_path: String,
    source_manual_approval_lifecycle_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    endpoint_class: String,
    endpoint_decision: String,
    endpoint_reason: String,
    endpoint_url_redacted: String,
    request_method: String,
    request_target: String,
    query_shape_without_signature: String,
    signature_preflight: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    time_in_force: String,
    notional: String,
    recv_window_ms: u64,
    timestamp_recorded: bool,
    timestamp_shape: String,
    credential_material: String,
    production_signing_material_gate_required: bool,
    production_signing_material_gate_open: bool,
    production_signing_material_env_read: bool,
    production_signing_material_missing_gate_env_vars: Vec<String>,
    api_key_env: String,
    api_secret_env: String,
    api_key_header_name: String,
    api_key_header_value_recorded: bool,
    api_secret_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    raw_request_body_recorded: bool,
    owner_gate_required: bool,
    manual_gate_required: bool,
    manual_approval_lifecycle_status: String,
    manual_approval_lifecycle_state: String,
    manual_approval_lifecycle_valid: bool,
    manual_approval_lifecycle_issues: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_approval_id: Option<String>,
    manual_approval_expires_at_unix_ms: Option<u64>,
    manual_approval_now_unix_ms: Option<u64>,
    manual_approval_one_time: bool,
    manual_approval_used: bool,
    manual_approval_consumed: bool,
    manual_approval_consume_status: String,
    manual_approval_consume_transition: String,
    manual_approval_consume_artifact_path: String,
    missing_cli_flags: Vec<String>,
    missing_env_vars: Vec<String>,
    order_gate_ready: bool,
    request_preview_allowed: bool,
    request_preview_built: bool,
    request_sent: bool,
    order_submission_mode: String,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_called: bool,
    production_adapter_called: bool,
    matching_engine_submission: bool,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    external_venue_connection: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    signed_request_memory_only: bool,
    secrets_redacted: bool,
    no_production_order_submission_confirmed: bool,
    no_production_order_mutation_confirmed: bool,
    no_execution_adapter_call_confirmed: bool,
    no_network_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_real_funds_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionLiveAlphaKillSwitchRuntimeGateArtifact {
    schema_version: String,
    run_id: String,
    source_kill_switch_approval_path: String,
    source_risk_preflight_path: String,
    source_request_preview_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    runtime_gate_decision: String,
    runtime_gate_open: bool,
    runtime_gate_reasons: Vec<String>,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    kill_switch_enabled: bool,
    kill_switch_active: bool,
    approval_state: String,
    manual_approval_required: bool,
    manual_approval_recorded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    risk_preflight_decision: String,
    risk_preflight_kill_switch_active: bool,
    request_preview_status: String,
    request_preview_built: bool,
    request_sent: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_called: bool,
    production_adapter_called: bool,
    matching_engine_submission: bool,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    external_venue_connection: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    no_production_order_submission_confirmed: bool,
    no_production_order_mutation_confirmed: bool,
    no_network_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_real_funds_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationSigningApprovalArtifact {
    schema_version: String,
    run_id: String,
    source_request_preview_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    credential_material: String,
    approval_state: String,
    manual_approval_recorded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    now_unix_ms: u64,
    expires_at_unix_ms: u64,
    approval_expired: bool,
    approval_revoked: bool,
    owner_approval_required: bool,
    owner_approved_signing_material: bool,
    signing_approval_ready: bool,
    signing_material_scope: String,
    production_signing_material_gate_required: bool,
    production_signing_material_gate_open: bool,
    production_signing_material_env_read: bool,
    production_signing_material_missing_gate_env_vars: Vec<String>,
    api_key_env_name_recorded: bool,
    api_secret_env_name_recorded: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    raw_request_body_recorded: bool,
    request_preview_status: String,
    request_preview_built: bool,
    request_sent: bool,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    time_in_force: String,
    notional: String,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    retry_attempted: bool,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_called: bool,
    production_adapter_called: bool,
    network_attempted: bool,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    env_only_signing_material_confirmed: bool,
    memory_only_signing_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    no_network_confirmed: bool,
    no_production_order_submission_confirmed: bool,
    no_production_order_mutation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationRequestBuilderArtifact {
    schema_version: String,
    run_id: String,
    source_runtime_gate_path: String,
    source_signing_approval_path: String,
    source_request_preview_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    request_builder_ready: bool,
    request_object_built: bool,
    runtime_gate_status: String,
    runtime_gate_open: bool,
    send_consideration_allowed: bool,
    signing_approval_status: String,
    signing_approval_ready: bool,
    explicit_send_gate_open: bool,
    credential_material: String,
    production_signing_material_gate_required: bool,
    production_signing_material_gate_open: bool,
    production_signing_material_env_read: bool,
    production_signing_material_missing_gate_env_vars: Vec<String>,
    api_key_env: String,
    api_secret_env: String,
    api_key_env_name_recorded: bool,
    api_secret_env_name_recorded: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_name: String,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    raw_request_body_recorded: bool,
    request_method: String,
    request_target: String,
    endpoint_url_redacted: String,
    query_shape_without_signature: String,
    signed_query_shape: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    time_in_force: String,
    notional: String,
    recv_window_ms: u64,
    timestamp_recorded: bool,
    timestamp_shape: String,
    max_order_notional: String,
    single_order_candidate: bool,
    tiny_notional_gate_ready: bool,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    missing_env_vars: Vec<String>,
    request_sent: bool,
    network_attempted: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_called: bool,
    production_adapter_called: bool,
    production_adapter_instantiated: bool,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    env_only_signing_material_confirmed: bool,
    memory_only_signing_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    no_network_confirmed: bool,
    no_production_order_submission_confirmed: bool,
    no_production_order_mutation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    no_retry_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationGuardedSendArtifact {
    schema_version: String,
    run_id: String,
    source_request_builder_path: String,
    source_kill_switch_runtime_gate_path: String,
    source_request_preview_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    manual_online_requested: bool,
    guarded_send_ready: bool,
    send_path_evaluated: bool,
    kill_switch_enforcement_ready: bool,
    kill_switch_checked_before_send: bool,
    kill_switch_checked_after_send: bool,
    pre_send_kill_switch_snapshot_source: String,
    pre_send_kill_switch_snapshot_hash: String,
    pre_send_kill_switch_checked_at: String,
    pre_send_kill_switch_runtime_gate_open: bool,
    pre_send_kill_switch_active: bool,
    post_send_kill_switch_snapshot_source: String,
    post_send_kill_switch_snapshot_hash: String,
    post_send_kill_switch_checked_at: String,
    post_send_kill_switch_runtime_gate_open: bool,
    post_send_kill_switch_active: bool,
    post_send_kill_switch_clean: bool,
    kill_switch_blocked_send: bool,
    post_send_progression_blocked: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    single_shot_send_allowed: bool,
    request_builder_status: String,
    request_object_built: bool,
    request_method: String,
    request_target: String,
    endpoint_url_redacted: String,
    credential_material: String,
    production_signing_material_gate_required: bool,
    production_signing_material_gate_open: bool,
    production_signing_material_env_read: bool,
    production_signing_material_missing_gate_env_vars: Vec<String>,
    api_key_env: String,
    api_secret_env: String,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    raw_request_body_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_redacted: bool,
    http_status_code: Option<u16>,
    latency_ms: Option<u64>,
    error_code: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    time_in_force: String,
    notional: String,
    max_order_notional: String,
    recv_window_ms: u64,
    timestamp_recorded: bool,
    timestamp_shape: String,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    missing_env_vars: Vec<String>,
    request_sent: bool,
    network_attempted: bool,
    production_order_request_attempted: bool,
    http_send_attempted: bool,
    exchange_ack_observed: bool,
    exchange_order_id_observed: bool,
    exchange_order_status_observed: bool,
    confirmed_production_order_submission: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    platform_production_trading_enabled: bool,
    production_trading_enabled: bool,
    single_shot_confirmed: bool,
    no_retry_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    response_redaction_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationResponseRedactionArtifact {
    schema_version: String,
    run_id: String,
    source_guarded_send_path: String,
    source_response_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    response_redaction_ready: bool,
    source_guarded_send_status: String,
    source_request_sent: bool,
    source_network_attempted: bool,
    response_shape_validated: bool,
    response_type: String,
    symbol: String,
    side: String,
    order_type: String,
    time_in_force: String,
    order_id: String,
    client_order_id: String,
    exchange_status: String,
    transact_time_shape: String,
    working_time_shape: String,
    allowed_response_fields: Vec<String>,
    forbidden_response_markers: Vec<String>,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    raw_request_body_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    unrestricted_payload_recorded: bool,
    account_balances_recorded: bool,
    fills_recorded: bool,
    response_redacted: bool,
    request_sent: bool,
    network_attempted: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    no_raw_response_persistence_confirmed: bool,
    no_headers_persistence_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    order_metadata_only_confirmed: bool,
    no_account_balances_confirmed: bool,
    no_unrestricted_payload_confirmed: bool,
    no_retry_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationOrderStateReadbackArtifact {
    schema_version: String,
    run_id: String,
    source_response_redaction_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    readback_contract_ready: bool,
    source_response_redaction_status: String,
    known_order_identifier_source: String,
    known_order_id: String,
    known_client_order_id: String,
    symbol: String,
    endpoint: String,
    method: String,
    path: String,
    request_url_redacted: String,
    query_shape: String,
    manual_online_requested: bool,
    order_state_read_allowed: bool,
    order_state_read_attempted: bool,
    response_shape: String,
    response_shape_validated: bool,
    endpoint_shape_validated: bool,
    order_entries_observed: usize,
    non_empty_order_state_observed: bool,
    order_lifecycle_readiness: bool,
    strategy_success_inferred: bool,
    strategy_success_proof: String,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    missing_env_vars: Vec<String>,
    api_key_env: String,
    api_secret_env: String,
    api_key_present: bool,
    api_secret_present: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    response_redacted: bool,
    network_attempted: bool,
    latency_ms: Option<u64>,
    response_status_code: Option<u16>,
    error_code: String,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    known_order_identifier_only_confirmed: bool,
    read_only_get_order_confirmed: bool,
    no_production_order_mutation_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    no_retry_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationAuditTrailArtifact {
    schema_version: String,
    run_id: String,
    source_request_builder_path: String,
    source_guarded_send_path: String,
    source_response_redaction_path: String,
    source_order_state_readback_path: String,
    source_runtime_gate_path: String,
    source_signing_approval_path: String,
    source_request_preview_path: String,
    source_kill_switch_runtime_gate_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    audit_trail_ready: bool,
    preview_hash: String,
    signing_approval_status: String,
    approval_state: String,
    manual_approval_recorded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    runtime_gate_status: String,
    runtime_gate_open: bool,
    send_consideration_allowed: bool,
    guarded_send_status: String,
    request_sent: bool,
    network_attempted: bool,
    response_redaction_status: String,
    response_redaction_ready: bool,
    order_state_readback_status: String,
    readback_contract_ready: bool,
    order_state_read_attempted: bool,
    kill_switch_checked_before_send: bool,
    kill_switch_checked_after_send: bool,
    pre_send_kill_switch_runtime_gate_open: bool,
    pre_send_kill_switch_active: bool,
    post_send_kill_switch_runtime_gate_open: bool,
    post_send_kill_switch_active: bool,
    kill_switch_blocked_send: bool,
    symbol: String,
    side: String,
    order_type: String,
    time_in_force: String,
    quantity: String,
    price: String,
    notional: String,
    order_id: String,
    client_order_id: String,
    exchange_status: String,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    failure_state: String,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    raw_request_body_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    unrestricted_payload_recorded: bool,
    account_balances_recorded: bool,
    response_redacted: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    redacted_artifacts_only_confirmed: bool,
    no_secret_or_raw_payload_persistence_confirmed: bool,
    no_retry_or_followup_mutation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationFailureSemanticsArtifact {
    schema_version: String,
    run_id: String,
    source_audit_trail_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    failure_semantics_ready: bool,
    failure_mode: String,
    failure_category: String,
    failure_state: String,
    source_audit_trail_status: String,
    source_audit_trail_ready: bool,
    source_failure_state: String,
    terminal_action: String,
    evidence_written: bool,
    stop_after_evidence: bool,
    strategy_continuation_allowed: bool,
    request_sent: bool,
    network_attempted: bool,
    order_state_read_attempted: bool,
    kill_switch_checked_before_send: bool,
    kill_switch_checked_after_send: bool,
    kill_switch_blocked_send: bool,
    retry_allowed: bool,
    retry_attempted: bool,
    retry_attempts: u64,
    max_retry_attempts: u64,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    correction_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_remediation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    evidence_only_failure_handling_confirmed: bool,
    no_retry_confirmed: bool,
    no_automatic_cancel_replace_amend_confirmed: bool,
    no_correction_or_flatten_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_strategy_continuation_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProductionMutationGuardedSendHttpResult {
    request_sent: bool,
    network_attempted: bool,
    http_send_attempted: bool,
    exchange_ack_observed: bool,
    exchange_order_id_observed: bool,
    exchange_order_status_observed: bool,
    latency_ms: Option<u64>,
    status_code: Option<u16>,
    error_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProductionMutationGuardedSendCounters {
    request_sent: bool,
    network_attempted: bool,
    production_order_request_attempted: bool,
    http_send_attempted: bool,
    exchange_ack_observed: bool,
    exchange_order_id_observed: bool,
    exchange_order_status_observed: bool,
    confirmed_production_order_submission: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    real_orders_submitted: bool,
    platform_production_trading_enabled: bool,
    production_trading_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProductionMutationKillSwitchSnapshot {
    runtime_gate_open: bool,
    kill_switch_active: bool,
    checked: bool,
    source_path: String,
    source_hash: String,
    checked_at: String,
}

impl ProductionMutationGuardedSendHttpResult {
    fn success(latency_ms: u64, status_code: u16) -> Self {
        Self {
            request_sent: true,
            network_attempted: true,
            http_send_attempted: true,
            exchange_ack_observed: (200..300).contains(&status_code),
            exchange_order_id_observed: false,
            exchange_order_status_observed: false,
            latency_ms: Some(latency_ms),
            status_code: Some(status_code),
            error_code: "none".to_string(),
        }
    }

    fn failure(latency_ms: Option<u64>, status_code: Option<u16>, error_code: &str) -> Self {
        Self {
            request_sent: true,
            network_attempted: true,
            http_send_attempted: true,
            exchange_ack_observed: false,
            exchange_order_id_observed: false,
            exchange_order_status_observed: false,
            latency_ms,
            status_code,
            error_code: error_code.to_string(),
        }
    }

    fn pre_http_failure(error_code: &str) -> Self {
        Self {
            request_sent: true,
            network_attempted: false,
            http_send_attempted: false,
            exchange_ack_observed: false,
            exchange_order_id_observed: false,
            exchange_order_status_observed: false,
            latency_ms: None,
            status_code: None,
            error_code: error_code.to_string(),
        }
    }
}

fn production_mutation_guarded_send_counters(
    http_result: Option<&ProductionMutationGuardedSendHttpResult>,
) -> ProductionMutationGuardedSendCounters {
    let request_sent = http_result.is_some_and(|result| result.request_sent);
    let network_attempted = http_result.is_some_and(|result| result.network_attempted);
    let production_order_request_attempted = http_result.is_some();
    let http_send_attempted = http_result.is_some_and(|result| result.http_send_attempted);
    let exchange_ack_observed = http_result.is_some_and(|result| result.exchange_ack_observed);
    let exchange_order_id_observed =
        http_result.is_some_and(|result| result.exchange_order_id_observed);
    let exchange_order_status_observed =
        http_result.is_some_and(|result| result.exchange_order_status_observed);
    let confirmed_production_order_submission = exchange_ack_observed;
    let production_order_submissions_attempted = u64::from(production_order_request_attempted);
    let production_orders_submitted = u64::from(confirmed_production_order_submission);
    let production_order_mutations_attempted = u64::from(request_sent);
    let real_orders_submitted = confirmed_production_order_submission;

    ProductionMutationGuardedSendCounters {
        request_sent,
        network_attempted,
        production_order_request_attempted,
        http_send_attempted,
        exchange_ack_observed,
        exchange_order_id_observed,
        exchange_order_status_observed,
        confirmed_production_order_submission,
        production_order_submissions_attempted,
        production_orders_submitted,
        production_order_mutations_attempted,
        real_orders_submitted,
        platform_production_trading_enabled: false,
        production_trading_enabled: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationRuntimeGateArtifact {
    schema_version: String,
    run_id: String,
    source_order_gate_path: String,
    source_risk_preflight_path: String,
    source_request_preview_path: String,
    source_kill_switch_runtime_gate_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_signing_approval_path: Option<String>,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion_from_v15: bool,
    default_fail_closed: bool,
    runtime_gate_decision: String,
    runtime_gate_open: bool,
    send_consideration_allowed: bool,
    runtime_gate_reasons: Vec<String>,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    owner_approval_required: bool,
    owner_approval_consumed: bool,
    manual_approval_consumed: bool,
    manual_approval_consume_status: String,
    kill_switch_checked_before_send: bool,
    kill_switch_runtime_gate_open: bool,
    kill_switch_active: bool,
    risk_preflight_decision: String,
    request_preview_built: bool,
    request_sent: bool,
    signing_approval_required: bool,
    signing_approval_ready: bool,
    signing_approval_status: String,
    explicit_send_gate_required: bool,
    explicit_send_gate_open: bool,
    single_order_candidate: bool,
    tiny_notional_gate_ready: bool,
    max_order_notional: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    time_in_force: String,
    notional: String,
    production_order_submission_allowed_policy: String,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_called: bool,
    production_adapter_called: bool,
    production_adapter_instantiated: bool,
    matching_engine_submission: bool,
    actual_submission_count: u64,
    automatic_remediation_attempted: bool,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    external_venue_connection: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    request_redacted: bool,
    response_redacted: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    raw_exchange_response_recorded: bool,
    no_network_before_send_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    no_retry_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionLiveAlphaExecutionDryRunArtifact {
    schema_version: String,
    run_id: String,
    source_order_gate_path: String,
    source_risk_preflight_path: String,
    source_request_preview_path: String,
    source_kill_switch_runtime_gate_path: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    execution_decision: String,
    execution_boundary_contract_version: String,
    execution_boundary_flow: String,
    execution_boundary_contract_ready: bool,
    isolation_route: String,
    strategy_intent_boundary: String,
    risk_decision_boundary: String,
    execution_command_boundary: String,
    execution_command_created: bool,
    execution_command_route: String,
    execution_command_destination: String,
    dry_run_adapter_boundary: String,
    dry_run_adapter_route_allowed: bool,
    production_adapter_boundary: String,
    production_adapter_route_allowed: bool,
    production_adapter_instantiation_allowed: bool,
    dry_run_execution_adapter: String,
    dry_run_execution_adapter_called: bool,
    dry_run_execution_adapter_wrote_artifact: bool,
    dry_run_adapter_artifact_only: bool,
    real_execution_adapter_called: bool,
    production_adapter_instantiated: bool,
    production_adapter_called: bool,
    strategy_intent_recorded: bool,
    strategy_intent_reaches_risk_preflight: bool,
    strategy_intent_reaches_dry_run_adapter: bool,
    strategy_intent_reaches_production_adapter: bool,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    order_gate_ready: bool,
    risk_preflight_decision: String,
    request_preview_built: bool,
    request_sent: bool,
    kill_switch_runtime_gate_status: String,
    kill_switch_runtime_gate_open: bool,
    session_id: String,
    strategy_id: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    time_in_force: String,
    notional: String,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    network_attempted: bool,
    matching_engine_submission: bool,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    external_venue_connection: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
    values_are_exchange_truth: bool,
    no_production_order_submission_confirmed: bool,
    no_production_order_mutation_confirmed: bool,
    no_production_adapter_confirmed: bool,
    no_network_confirmed: bool,
    no_listen_key_lifecycle_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_real_funds_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProductionLiveAlphaRiskPreflightInput {
    schema_version: String,
    session: ProductionLiveAlphaRiskPreflightSession,
    market: ProductionLiveAlphaRiskPreflightMarket,
    account: ProductionLiveAlphaRiskPreflightAccount,
    order_state: ProductionLiveAlphaRiskPreflightOrderState,
    risk: ProductionLiveAlphaRiskPreflightRisk,
    order: ProductionLiveAlphaRiskPreflightOrder,
    limits: ProductionLiveAlphaRiskPreflightLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProductionLiveAlphaRiskPreflightSession {
    state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProductionLiveAlphaRiskPreflightMarket {
    symbol: String,
    last_event_at_unix_ms: u64,
    now_unix_ms: u64,
    max_age_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProductionLiveAlphaRiskPreflightAccount {
    readable: bool,
    account_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProductionLiveAlphaRiskPreflightOrderState {
    readable: bool,
    open_order_count: u64,
    #[serde(default)]
    last_read_at_unix_ms: Option<u64>,
    #[serde(default)]
    now_unix_ms: Option<u64>,
    #[serde(default)]
    max_age_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProductionLiveAlphaRiskPreflightRisk {
    kill_switch_active: bool,
    allowed_symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProductionLiveAlphaRiskPreflightOrder {
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    notional: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProductionLiveAlphaRiskPreflightLimits {
    max_order_notional: String,
    current_position_notional: String,
    max_position_notional: String,
    max_open_orders: u64,
    max_clock_skew_ms: u64,
    observed_clock_skew_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionLiveAlphaRiskPreflightReport {
    schema_version: String,
    status: String,
    run_id: String,
    evaluated_at: String,
    risk_decision: String,
    execution_decision: String,
    reasons: Vec<String>,
    missing_cli_flags: Vec<String>,
    order_gate_status: String,
    order_gate_ready: bool,
    order_gate_path: String,
    session_state: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: String,
    notional: String,
    max_order_notional: String,
    current_position_notional: String,
    projected_position_notional: String,
    max_position_notional: String,
    market_age_ms: Option<u64>,
    max_market_age_ms: u64,
    account_readable: bool,
    order_state_readable: bool,
    order_state_age_ms: Option<u64>,
    max_order_state_age_ms: Option<u64>,
    open_order_count: u64,
    max_open_orders: u64,
    observed_clock_skew_ms: u64,
    max_clock_skew_ms: u64,
    kill_switch_active: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    order_endpoint_access_attempted: bool,
    execution_adapter_called: bool,
    matching_engine_submission: bool,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    external_venue_connection: bool,
    network_attempted: bool,
    real_orders_submitted: bool,
    real_funds: bool,
    production_trading_enabled: bool,
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
    values_are_exchange_truth: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProductionOrderStateHttpResult {
    status: String,
    latency_ms: Option<u64>,
    http_status: Option<u16>,
    response_shape: String,
    response_shape_validated: bool,
    response_shape_summary: ProductionOrderStateShapeSummary,
    error_code: String,
    network_attempted: bool,
    diagnostic: String,
}

impl ProductionOrderStateHttpResult {
    #[cfg(test)]
    fn success(
        endpoint: ProductionOrderStateReadEndpoint,
        latency_ms: u64,
        http_status: u16,
    ) -> Self {
        Self::success_with_shape(
            endpoint,
            latency_ms,
            http_status,
            ProductionOrderStateShapeSummary::accepted_fixture(endpoint),
        )
    }

    fn success_with_shape(
        endpoint: ProductionOrderStateReadEndpoint,
        latency_ms: u64,
        http_status: u16,
        response_shape_summary: ProductionOrderStateShapeSummary,
    ) -> Self {
        Self {
            status: "online_order_state_read_ok".to_string(),
            latency_ms: Some(latency_ms),
            http_status: Some(http_status),
            response_shape: production_order_state_response_shape(endpoint).to_string(),
            response_shape_validated: response_shape_summary.shape_validated,
            response_shape_summary,
            error_code: "none".to_string(),
            network_attempted: true,
            diagnostic: format!(
                "V140 production order-state read-only proof succeeded with GET {} and HTTP {http_status}; raw order response, signatures, signed query, and signed URL were not recorded.",
                production_order_state_endpoint_path(endpoint),
            ),
        }
    }

    fn failure(
        endpoint: ProductionOrderStateReadEndpoint,
        latency_ms: Option<u64>,
        http_status: Option<u16>,
        error_code: &str,
    ) -> Self {
        Self::failure_with_shape(
            endpoint,
            latency_ms,
            http_status,
            error_code,
            ProductionOrderStateShapeSummary::not_attempted(endpoint),
        )
    }

    fn failure_with_shape(
        endpoint: ProductionOrderStateReadEndpoint,
        latency_ms: Option<u64>,
        http_status: Option<u16>,
        error_code: &str,
        response_shape_summary: ProductionOrderStateShapeSummary,
    ) -> Self {
        let status_detail = http_status
            .map(|status| format!(" HTTP {status}"))
            .unwrap_or_default();
        Self {
            status: "online_order_state_read_failed".to_string(),
            latency_ms,
            http_status,
            response_shape: production_order_state_response_shape(endpoint).to_string(),
            response_shape_validated: response_shape_summary.shape_validated,
            response_shape_summary,
            error_code: error_code.to_string(),
            network_attempted: true,
            diagnostic: format!(
                "V140 production order-state read-only proof attempted GET {} and failed with {error_code}.{status_detail} Raw order response, signatures, signed query, and signed URL were not recorded.",
                production_order_state_endpoint_path(endpoint),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionOrderStateShapeSummary {
    status: String,
    endpoint: String,
    root_is_array: bool,
    root_is_object: bool,
    order_entry_count: Option<usize>,
    symbol_present: bool,
    symbol_is_string: bool,
    order_id_present: bool,
    status_present: bool,
    status_is_string: bool,
    raw_order_response_recorded: bool,
    raw_order_list_recorded: bool,
    shape_validated: bool,
    endpoint_shape_validated: bool,
    order_entries_observed: usize,
    non_empty_order_state_observed: bool,
    order_lifecycle_readiness: bool,
    rejection_reason: String,
}

impl ProductionOrderStateShapeSummary {
    fn not_attempted(endpoint: ProductionOrderStateReadEndpoint) -> Self {
        Self {
            status: "not_attempted".to_string(),
            endpoint: production_order_state_endpoint_name(endpoint).to_string(),
            root_is_array: false,
            root_is_object: false,
            order_entry_count: None,
            symbol_present: false,
            symbol_is_string: false,
            order_id_present: false,
            status_present: false,
            status_is_string: false,
            raw_order_response_recorded: false,
            raw_order_list_recorded: false,
            shape_validated: false,
            endpoint_shape_validated: false,
            order_entries_observed: 0,
            non_empty_order_state_observed: false,
            order_lifecycle_readiness: false,
            rejection_reason: "not_attempted".to_string(),
        }
    }

    #[cfg(test)]
    fn accepted_fixture(endpoint: ProductionOrderStateReadEndpoint) -> Self {
        match endpoint {
            ProductionOrderStateReadEndpoint::OpenOrders => Self {
                status: "accepted".to_string(),
                endpoint: "open_orders".to_string(),
                root_is_array: true,
                root_is_object: false,
                order_entry_count: Some(0),
                symbol_present: true,
                symbol_is_string: true,
                order_id_present: true,
                status_present: true,
                status_is_string: true,
                raw_order_response_recorded: false,
                raw_order_list_recorded: false,
                shape_validated: true,
                endpoint_shape_validated: true,
                order_entries_observed: 0,
                non_empty_order_state_observed: false,
                order_lifecycle_readiness: false,
                rejection_reason: "none".to_string(),
            },
            ProductionOrderStateReadEndpoint::Order => Self {
                status: "accepted".to_string(),
                endpoint: "order".to_string(),
                root_is_array: false,
                root_is_object: true,
                order_entry_count: Some(1),
                symbol_present: true,
                symbol_is_string: true,
                order_id_present: true,
                status_present: true,
                status_is_string: true,
                raw_order_response_recorded: false,
                raw_order_list_recorded: false,
                shape_validated: true,
                endpoint_shape_validated: true,
                order_entries_observed: 1,
                non_empty_order_state_observed: true,
                order_lifecycle_readiness: true,
                rejection_reason: "none".to_string(),
            },
        }
    }
}

struct ProductionOrderStateSignedRequest {
    method: String,
    endpoint_path: String,
    endpoint_url_redacted: String,
    query_without_signature: String,
    signature: String,
    signed_query: String,
    api_key_header_name: String,
    api_key_header_value: String,
}

impl Debug for ProductionOrderStateSignedRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductionOrderStateSignedRequest")
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

impl ProductionOrderStateSignedRequest {
    fn signed_url_for_execution(&self) -> String {
        format!("{}?{}", self.endpoint_url_redacted, self.signed_query)
    }

    fn ensure_redacted(
        &self,
        credentials: &EnvOnlyProductionReadCredentials,
    ) -> anyhow::Result<()> {
        let body = format!("{self:?}");
        credentials.ensure_no_secret_values_absent("production-order-state-request", &body)?;
        for (label, sensitive_value) in [
            ("signature", self.signature.as_str()),
            ("signed query", self.signed_query.as_str()),
            ("API key header value", self.api_key_header_value.as_str()),
        ] {
            if !sensitive_value.is_empty() && body.contains(sensitive_value) {
                anyhow::bail!("production order-state request leaked {label}");
            }
        }
        Ok(())
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
    notional_preflight: ShadowNotionalPreflight,
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
struct ShadowNotionalPreflight {
    status: String,
    aggregation: String,
    decimal_string_sum: Option<String>,
    parsed_notional_count: u64,
    f64_aggregation_used: bool,
    live_alpha_money_math_ready: bool,
    risk_or_execution_grade: bool,
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
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
    values_are_exchange_truth: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ShadowIntentInputs {
    refs: Vec<ShadowIntentRef>,
    record_count: u64,
    notional_sum: Option<Decimal>,
    parsed_notional_count: u64,
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
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
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
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionShadowPreflightSessionEvent {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    source_artifact_age_ms: Option<u64>,
    stale_after_ms: u64,
    stale_data_detected: bool,
    stop_file_observed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shutdown_reason: Option<String>,
    session_network_attempted: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
    values_are_exchange_truth: bool,
    diagnostic: String,
}

struct ShadowPreflightEventInput<'a> {
    opt: &'a LiveProductionShadowPreflightSessionOpt,
    session_id: &'a str,
    event_type: &'a str,
    state: &'a str,
    heartbeat_seq: Option<u64>,
    portfolio_ref: &'a ShadowStrategyPortfolioRuntimeRef,
    session_status_ref: Option<ShadowStrategySessionStatusRef>,
    artifact_gap: Option<ShadowStrategyArtifactGap>,
    source_artifact_age_ms: Option<u64>,
    stale_data_detected: bool,
    stop_file_observed: bool,
    shutdown_reason: Option<&'a str>,
    diagnostic: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionKillSwitchApprovalArtifact {
    schema_version: String,
    run_id: String,
    session_id: String,
    strategy_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    kill_switch_enabled: bool,
    kill_switch_active: bool,
    kill_switch_dry_run: bool,
    kill_switch_state_source: String,
    manual_approval_required: bool,
    manual_approval_recorded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    approval_state: String,
    approval_artifact_only: bool,
    owner_approval_required_before_any_mutation: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    actual_submission_count: u64,
    automatic_correction_orders_submitted: u64,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    production_trading_enabled: bool,
    network_attempted: bool,
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
    values_are_exchange_truth: bool,
    dry_run_confirmed: bool,
    no_production_mutation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionReadonlyReconciliationEvent {
    schema_version: String,
    run_id: String,
    event_id: String,
    event_type: String,
    classification: String,
    severity: String,
    observed_at: String,
    source_ref: ReadonlyReconciliationSourceRef,
    account_snapshot_ref: ReadonlyReconciliationArtifactRef,
    shadow_portfolio_ref: ReadonlyReconciliationArtifactRef,
    shadow_strategy_session_ref: ReadonlyReconciliationArtifactRef,
    shadow_intent_ref: ReadonlyReconciliationArtifactRef,
    recommended_action: String,
    risk_halted: bool,
    new_orders_blocked: bool,
    manual_review_required: bool,
    automatic_correction_orders_submitted: u64,
    production_order_submissions_attempted: u64,
    production_orders_submitted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    cancel_replace_amend_attempted: bool,
    dashboard_order_controls_enabled: bool,
    real_orders_submitted: bool,
    order_state_values_are_exchange_truth: bool,
    shadow_values_are_exchange_truth: bool,
    portfolio_values_are_exchange_truth: bool,
    values_are_exchange_truth: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ReadonlyReconciliationSourceRef {
    engine: String,
    mode: String,
    network_attempted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ReadonlyReconciliationArtifactRef {
    path: Option<String>,
    status: String,
    schema_version: Option<String>,
    record_count: Option<u64>,
    classification: Option<String>,
    diagnostic: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadonlyReconciliationClassification {
    Ok,
    MissingAccountSnapshot,
    PortfolioUnavailable,
    ShadowIntentWithoutPortfolio,
    ProductionMutationForbidden,
    ManualReviewRequired,
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

struct ProductionLiveAlphaSignedOrderRequestPreview {
    method: String,
    endpoint_path: String,
    endpoint_url_redacted: String,
    query_without_signature: String,
    signature: String,
    signed_query: String,
    api_key_header_name: String,
    api_key_header_value: String,
}

impl Debug for ProductionLiveAlphaSignedOrderRequestPreview {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductionLiveAlphaSignedOrderRequestPreview")
            .field("method", &self.method)
            .field("endpoint_path", &self.endpoint_path)
            .field("endpoint_url_redacted", &self.endpoint_url_redacted)
            .field("query_without_signature", &"<memory-only-redacted>")
            .field("signature", &"<memory-only-redacted>")
            .field("signed_query", &"<memory-only-redacted>")
            .field("api_key_header_name", &self.api_key_header_name)
            .field("api_key_header_value", &"<redacted>")
            .finish()
    }
}

impl ProductionLiveAlphaSignedOrderRequestPreview {
    fn ensure_memory_only_redacted(
        &self,
        credentials: &EnvOnlyProductionMutationPreviewCredentials,
    ) -> anyhow::Result<()> {
        let body = format!("{self:?}");
        credentials
            .ensure_no_secret_values_absent("production-live-alpha-request-preview", &body)?;
        for (label, sensitive_value) in [
            (
                "query without signature",
                self.query_without_signature.as_str(),
            ),
            ("signature", self.signature.as_str()),
            ("signed query", self.signed_query.as_str()),
            ("API key header value", self.api_key_header_value.as_str()),
        ] {
            if !sensitive_value.is_empty() && body.contains(sensitive_value) {
                anyhow::bail!("production live-alpha request preview leaked {label}");
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
        LiveCommand::ProductionOrderStateReadOnlyProof(proof) => {
            run_live_production_order_state_readonly_proof(&proof)
        }
        LiveCommand::ProductionLiveAlphaDryRunOrderGate(gate) => {
            run_live_production_live_alpha_dry_run_order_gate(&gate)
        }
        LiveCommand::ProductionLiveAlphaOrderRequestPreview(preview) => {
            run_live_production_live_alpha_order_request_preview(&preview)
        }
        LiveCommand::ProductionLiveAlphaManualApprovalLifecycle(approval) => {
            run_live_production_live_alpha_manual_approval_lifecycle(&approval)
        }
        LiveCommand::ProductionLiveAlphaExecutionDryRun(dry_run) => {
            run_live_production_live_alpha_execution_dry_run(&dry_run)
        }
        LiveCommand::ProductionLiveAlphaKillSwitchRuntimeGate(gate) => {
            run_live_production_live_alpha_kill_switch_runtime_gate(&gate)
        }
        LiveCommand::ProductionMutationRuntimeGate(gate) => {
            run_live_production_mutation_runtime_gate(&gate)
        }
        LiveCommand::ProductionMutationSigningApproval(approval) => {
            run_live_production_mutation_signing_approval(&approval)
        }
        LiveCommand::ProductionMutationRequestBuilder(builder) => {
            run_live_production_mutation_request_builder(&builder)
        }
        LiveCommand::ProductionMutationGuardedSend(send) => {
            run_live_production_mutation_guarded_send(&send)
        }
        LiveCommand::ProductionMutationResponseRedaction(redaction) => {
            run_live_production_mutation_response_redaction(&redaction)
        }
        LiveCommand::ProductionMutationOrderStateReadback(readback) => {
            run_live_production_mutation_order_state_readback(&readback)
        }
        LiveCommand::ProductionMutationAuditTrail(audit) => {
            run_live_production_mutation_audit_trail(&audit)
        }
        LiveCommand::ProductionMutationFailureSemantics(failure) => {
            run_live_production_mutation_failure_semantics(&failure)
        }
        LiveCommand::ProductionLiveAlphaRiskPreflight(preflight) => {
            run_live_production_live_alpha_risk_preflight(&preflight)
        }
        LiveCommand::ProductionShadowPortfolioRuntime(runtime) => {
            run_live_production_shadow_portfolio_runtime(&runtime)
        }
        LiveCommand::ProductionShadowStrategySession(session) => {
            run_live_production_shadow_strategy_session(&session)
        }
        LiveCommand::ProductionShadowPreflightSession(session) => {
            run_live_production_shadow_preflight_session(&session).await
        }
        LiveCommand::ProductionKillSwitchApprovalArtifact(artifact) => {
            run_live_production_kill_switch_approval_artifact(&artifact)
        }
        LiveCommand::ProductionReadonlyReconciliation(reconciliation) => {
            run_live_production_readonly_reconciliation(&reconciliation)
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

fn run_live_production_order_state_readonly_proof(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
) -> anyhow::Result<()> {
    run_live_production_order_state_readonly_proof_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_production_live_alpha_dry_run_order_gate(
    opt: &LiveProductionLiveAlphaDryRunOrderGateOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_live_alpha_dry_run_order_gate_artifact(opt)?;
    atomic_write_json(&opt.output, &artifact)?;

    println!(
        "live.production_live_alpha_dry_run_order_gate status={} run_id={} output={} dry_run_order_gate_ready={} production_order_submissions_attempted=0 production_orders_submitted=0 production_order_mutations_attempted=0 execution_adapter_called=false order_endpoint_access_attempted=false network_attempted=false dashboard_order_controls_enabled=false real_orders_submitted=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.dry_run_order_gate_ready,
    );
    Ok(())
}

fn run_live_production_live_alpha_order_request_preview(
    opt: &LiveProductionLiveAlphaOrderRequestPreviewOpt,
) -> anyhow::Result<()> {
    run_live_production_live_alpha_order_request_preview_with_env(opt, |name| {
        std::env::var(name).ok()
    })
}

fn run_live_production_live_alpha_order_request_preview_with_env<F>(
    opt: &LiveProductionLiveAlphaOrderRequestPreviewOpt,
    read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    let credentials =
        EnvOnlyProductionMutationPreviewCredentials::from_order_request_preview_opt(opt, read_env)?;
    let artifact = build_production_live_alpha_order_request_preview_artifact(opt, &credentials)?;
    write_production_live_alpha_order_request_preview_report(&opt.output, &artifact, &credentials)?;
    if artifact.request_preview_built {
        consume_production_live_alpha_manual_approval_lifecycle(
            &opt.manual_approval_lifecycle,
            &opt.output,
            &artifact,
        )?;
    }

    println!(
        "live.production_live_alpha_order_request_preview status={} run_id={} output={} request_preview_built={} request_sent=false production_orders_submitted=0 production_order_mutations_attempted=0 execution_adapter_called=false network_attempted=false dashboard_order_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.request_preview_built,
    );
    Ok(())
}

fn run_live_production_live_alpha_manual_approval_lifecycle(
    opt: &LiveProductionLiveAlphaManualApprovalLifecycleOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_live_alpha_manual_approval_lifecycle_artifact(opt)?;
    atomic_write_json(&opt.output, &artifact)?;

    println!(
        "live.production_live_alpha_manual_approval_lifecycle status={} run_id={} output={} approval_state={} approval_lifecycle_valid={} one_time_approval=true dry_run_request_preview_only=true production_orders_submitted=0 production_order_mutations_attempted=0 network_attempted=false dashboard_order_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.approval_state,
        artifact.approval_lifecycle_valid,
    );
    Ok(())
}

fn run_live_production_live_alpha_execution_dry_run(
    opt: &LiveProductionLiveAlphaExecutionDryRunOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_live_alpha_execution_dry_run_artifact(opt)?;
    atomic_write_json(&opt.output, &artifact)?;

    println!(
        "live.production_live_alpha_execution_dry_run status={} run_id={} output={} execution_command_route={} dry_run_execution_adapter_called={} production_adapter_called=false production_adapter_instantiated=false production_adapter_route_allowed=false production_orders_submitted=0 production_order_mutations_attempted=0 network_attempted=false dashboard_order_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.execution_command_route,
        artifact.dry_run_execution_adapter_called,
    );
    Ok(())
}

fn run_live_production_live_alpha_kill_switch_runtime_gate(
    opt: &LiveProductionLiveAlphaKillSwitchRuntimeGateOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_live_alpha_kill_switch_runtime_gate_artifact(opt)?;
    atomic_write_json(&opt.output, &artifact)?;

    println!(
        "live.production_live_alpha_kill_switch_runtime_gate status={} run_id={} output={} runtime_gate_open={} kill_switch_active={} manual_approval_recorded={} request_preview_built={} production_orders_submitted=0 production_order_mutations_attempted=0 network_attempted=false dashboard_order_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.runtime_gate_open,
        artifact.kill_switch_active,
        artifact.manual_approval_recorded,
        artifact.request_preview_built,
    );
    Ok(())
}

fn run_live_production_mutation_runtime_gate(
    opt: &LiveProductionMutationRuntimeGateOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_runtime_gate_artifact(opt)?;
    atomic_write_json(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_runtime_gate status={} run_id={} output={} runtime_gate_open={} send_consideration_allowed={} request_sent=false production_orders_submitted=0 production_order_mutations_attempted=0 network_attempted=false dashboard_order_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.runtime_gate_open,
        artifact.send_consideration_allowed,
    );
    Ok(())
}

fn run_live_production_mutation_signing_approval(
    opt: &LiveProductionMutationSigningApprovalOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_signing_approval_artifact(opt)?;
    atomic_write_json(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_signing_approval status={} run_id={} output={} signing_approval_ready={} request_sent=false production_orders_submitted=0 production_order_mutations_attempted=0 network_attempted=false dashboard_order_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.signing_approval_ready,
    );
    Ok(())
}

fn run_live_production_mutation_request_builder(
    opt: &LiveProductionMutationRequestBuilderOpt,
) -> anyhow::Result<()> {
    run_live_production_mutation_request_builder_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_production_mutation_request_builder_with_env<F>(
    opt: &LiveProductionMutationRequestBuilderOpt,
    read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    let credentials =
        EnvOnlyProductionMutationPreviewCredentials::from_request_builder_opt(opt, read_env);
    let artifact = build_production_mutation_request_builder_artifact(opt, &credentials)?;
    write_production_mutation_request_builder_artifact(&opt.output, &artifact, &credentials)?;
    println!(
        "live.production_mutation_request_builder status={} run_id={} output={} request_builder_ready={} request_object_built={} request_sent=false production_orders_submitted=0 production_order_mutations_attempted=0 network_attempted=false dashboard_order_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.request_builder_ready,
        artifact.request_object_built,
    );
    Ok(())
}

fn run_live_production_mutation_guarded_send(
    opt: &LiveProductionMutationGuardedSendOpt,
) -> anyhow::Result<()> {
    run_live_production_mutation_guarded_send_with_env(opt, |name| std::env::var(name).ok())
}

fn run_live_production_mutation_guarded_send_with_env<F>(
    opt: &LiveProductionMutationGuardedSendOpt,
    read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    let credentials =
        EnvOnlyProductionMutationPreviewCredentials::from_guarded_send_opt(opt, read_env);
    let artifact = build_production_mutation_guarded_send_artifact(opt, &credentials)?;
    write_production_mutation_guarded_send_artifact(&opt.output, &artifact, &credentials)?;
    println!(
        "live.production_mutation_guarded_send status={} run_id={} output={} manual_online_requested={} kill_switch_checked_before_send={} kill_switch_checked_after_send={} post_send_kill_switch_clean={} kill_switch_blocked_send={} post_send_progression_blocked={} manual_review_required={} new_orders_blocked={} request_sent={} production_order_request_attempted={} http_send_attempted={} exchange_ack_observed={} confirmed_production_order_submission={} production_order_submissions_attempted={} production_orders_submitted={} production_order_mutations_attempted={} network_attempted={} dashboard_order_controls_enabled=false platform_production_trading_enabled=false production_trading_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.manual_online_requested,
        artifact.kill_switch_checked_before_send,
        artifact.kill_switch_checked_after_send,
        artifact.post_send_kill_switch_clean,
        artifact.kill_switch_blocked_send,
        artifact.post_send_progression_blocked,
        artifact.manual_review_required,
        artifact.new_orders_blocked,
        artifact.request_sent,
        artifact.production_order_request_attempted,
        artifact.http_send_attempted,
        artifact.exchange_ack_observed,
        artifact.confirmed_production_order_submission,
        artifact.production_order_submissions_attempted,
        artifact.production_orders_submitted,
        artifact.production_order_mutations_attempted,
        artifact.network_attempted,
    );
    Ok(())
}

fn run_live_production_mutation_response_redaction(
    opt: &LiveProductionMutationResponseRedactionOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_response_redaction_artifact(opt)?;
    write_production_mutation_response_redaction_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_response_redaction status={} run_id={} output={} response_redaction_ready={} raw_exchange_response_recorded=false response_headers_recorded=false unrestricted_payload_recorded=false account_balances_recorded=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.response_redaction_ready,
    );
    Ok(())
}

fn run_live_production_mutation_order_state_readback(
    opt: &LiveProductionMutationOrderStateReadbackOpt,
) -> anyhow::Result<()> {
    run_live_production_mutation_order_state_readback_with_env_and_http(
        opt,
        &mut |name| std::env::var(name).ok(),
        execute_production_order_state_read,
    )
}

fn run_live_production_mutation_order_state_readback_with_env_and_http<F, H>(
    opt: &LiveProductionMutationOrderStateReadbackOpt,
    read_env: &mut F,
    http_probe: H,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
    H: FnMut(
        &LiveProductionOrderStateReadOnlyProofOpt,
        &EnvOnlyProductionReadCredentials,
        u64,
    ) -> ProductionOrderStateHttpResult,
{
    let artifact =
        build_production_mutation_order_state_readback_artifact(opt, read_env, http_probe)?;
    write_production_mutation_order_state_readback_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_order_state_readback status={} run_id={} output={} readback_contract_ready={} manual_online_requested={} order_state_read_attempted={} production_order_state_reads_attempted={} production_order_mutations_attempted=0 strategy_success_inferred=false raw_exchange_response_recorded=false response_headers_recorded=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.readback_contract_ready,
        artifact.manual_online_requested,
        artifact.order_state_read_attempted,
        artifact.production_order_state_reads_attempted,
    );
    Ok(())
}

fn run_live_production_mutation_audit_trail(
    opt: &LiveProductionMutationAuditTrailOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_audit_trail_artifact(opt)?;
    write_production_mutation_audit_trail_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_audit_trail status={} run_id={} output={} audit_trail_ready={} request_sent={} network_attempted={} kill_switch_checked_before_send={} kill_switch_checked_after_send={} retry_attempted=false cancel_attempted=false replace_attempted=false amend_attempted=false flatten_attempted=false dashboard_order_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.audit_trail_ready,
        artifact.request_sent,
        artifact.network_attempted,
        artifact.kill_switch_checked_before_send,
        artifact.kill_switch_checked_after_send,
    );
    Ok(())
}

fn run_live_production_mutation_failure_semantics(
    opt: &LiveProductionMutationFailureSemanticsOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_failure_semantics_artifact(opt)?;
    write_production_mutation_failure_semantics_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_failure_semantics status={} run_id={} output={} failure_semantics_ready={} failure_mode={} terminal_action={} retry_attempted=false cancel_attempted=false replace_attempted=false amend_attempted=false correction_attempted=false flatten_attempted=false remediation_attempted=false strategy_continuation_allowed=false dashboard_order_controls_enabled=false listen_key_lifecycle_attempted=0",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.failure_semantics_ready,
        artifact.failure_mode,
        artifact.terminal_action,
    );
    Ok(())
}

fn run_live_production_live_alpha_risk_preflight(
    opt: &LiveProductionLiveAlphaRiskPreflightOpt,
) -> anyhow::Result<()> {
    let report = build_production_live_alpha_risk_preflight_report(opt)?;
    atomic_write_json(&opt.output, &report)?;

    println!(
        "live.production_live_alpha_risk_preflight status={} run_id={} output={} risk_decision={} reasons={} production_orders_submitted=0 production_order_mutations_attempted=0 execution_adapter_called=false order_endpoint_access_attempted=false network_attempted=false dashboard_order_controls_enabled=false",
        report.status,
        report.run_id,
        opt.output.display(),
        report.risk_decision,
        join_owned_gate_labels(&report.reasons),
    );
    Ok(())
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

async fn run_live_production_shadow_preflight_session(
    opt: &LiveProductionShadowPreflightSessionOpt,
) -> anyhow::Result<()> {
    let result = run_production_shadow_preflight_session_loop(opt).await?;
    println!(
        "live.production_shadow_preflight_session status=ok run_id={} output={} events={} heartbeats={} final_state={} stop_file_observed={} stale_data_detected={} production_order_submissions_attempted=0 production_order_mutations_attempted=0 session_network_attempted=false dashboard_order_controls_enabled=false values_are_exchange_truth=false",
        opt.run_id,
        opt.output.display(),
        result.events_written,
        result.heartbeats_written,
        result.final_state,
        result.stop_file_observed,
        result.stale_data_detected,
    );
    Ok(())
}

fn run_live_production_kill_switch_approval_artifact(
    opt: &LiveProductionKillSwitchApprovalArtifactOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_kill_switch_approval_artifact(opt)?;
    atomic_write_json(&opt.output, &artifact)?;

    println!(
        "live.production_kill_switch_approval_artifact status={} run_id={} output={} kill_switch_dry_run=true kill_switch_active={} manual_approval_recorded={} production_order_submissions_attempted=0 production_order_mutations_attempted=0 network_attempted=false dashboard_order_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        opt.output.display(),
        artifact.kill_switch_active,
        artifact.manual_approval_recorded,
    );
    Ok(())
}

fn run_live_production_readonly_reconciliation(
    opt: &LiveProductionReadonlyReconciliationOpt,
) -> anyhow::Result<()> {
    let event = build_production_readonly_reconciliation_event(opt)?;
    write_production_readonly_reconciliation_events(&opt.output, std::slice::from_ref(&event))?;

    println!(
        "live.production_readonly_reconciliation status=ok run_id={} output={} classification={} severity={} recommended_action={} production_order_submissions_attempted=0 production_order_mutations_attempted=0 production_order_state_reads_attempted=0 listen_key_lifecycle_attempted=0 dashboard_order_controls_enabled=false",
        event.run_id,
        opt.output.display(),
        event.classification,
        event.severity,
        event.recommended_action,
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
        "live.production_public_read_probe status={} endpoint={} endpoint_class={} method={} path={} manual_online_requested={} endpoint_read_allowed={} offline_contract_ready={} contract_ready={} online_read_allowed={} online_execution_supported={} read_allowed={} mutation_allowed=false credentials_used=false network_attempted={} response_shape={} response_shape_validated={} error_code={} production_order_submission_attempted=false production_order_mutation_attempted=false dashboard_order_controls_enabled=false",
        report.status,
        report.endpoint,
        report.endpoint_class,
        report.method,
        report.path,
        report.manual_online_requested,
        report.endpoint_read_allowed,
        report.offline_contract_ready,
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

    let endpoint_read_allowed = classified_endpoint.read_allowed;
    let offline_contract_ready =
        !gates_missing && !manual_online_requested && endpoint_read_allowed;
    let online_read_allowed = !gates_missing && manual_online_requested && endpoint_read_allowed;
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
        endpoint_read_allowed,
        offline_contract_ready,
        read_allowed: offline_contract_ready,
        contract_ready: offline_contract_ready,
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
        "live.production_account_snapshot_contract status={} endpoint_class={} method={} path={} manual_online_requested={} endpoint_read_allowed={} offline_contract_ready={} contract_ready={} online_read_allowed={} online_execution_supported={} read_allowed={} mutation_allowed=false env_credentials_only=true credentials_used={} network_attempted={} account_read_attempted={} account_mutation_attempted=false order_endpoint_access_attempted=false production_order_submission_attempted=false production_order_mutation_attempted=false dashboard_order_controls_enabled=false secrets_redacted=true response_shape={} response_shape_validated={} error_code={}",
        report.status,
        report.endpoint_class,
        report.method,
        report.path,
        report.manual_online_requested,
        report.endpoint_read_allowed,
        report.offline_contract_ready,
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

    let endpoint_read_allowed = classified_endpoint.read_allowed;
    let offline_contract_ready =
        !gates_missing && !credentials_missing && !opt.manual_online && endpoint_read_allowed;
    let online_read_allowed =
        !gates_missing && !credentials_missing && opt.manual_online && endpoint_read_allowed;

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
        endpoint_read_allowed,
        offline_contract_ready,
        read_allowed: offline_contract_ready,
        contract_ready: offline_contract_ready,
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

fn run_live_production_order_state_readonly_proof_with_env<F>(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
    mut read_env: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
{
    run_live_production_order_state_readonly_proof_with_env_and_http(
        opt,
        &mut read_env,
        execute_production_order_state_read,
    )
}

fn run_live_production_order_state_readonly_proof_with_env_and_http<F, H>(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
    read_env: &mut F,
    mut http_probe: H,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> Option<String>,
    H: FnMut(
        &LiveProductionOrderStateReadOnlyProofOpt,
        &EnvOnlyProductionReadCredentials,
        u64,
    ) -> ProductionOrderStateHttpResult,
{
    let missing_cli_flags = missing_production_order_state_cli_flags(opt);
    let missing_env_vars = missing_production_order_state_env_gates(read_env, opt.manual_online);
    let credentials = EnvOnlyProductionReadCredentials::from_values(
        opt.api_key_env.clone(),
        read_env(&opt.api_key_env),
        opt.api_secret_env.clone(),
        read_env(&opt.api_secret_env),
    );
    let missing_order_identifier = production_order_state_missing_identifier(opt);
    let should_attempt_online = should_attempt_production_order_state_read(
        opt,
        &credentials,
        &missing_cli_flags,
        &missing_env_vars,
        missing_order_identifier,
    );
    let http_result =
        should_attempt_online.then(|| http_probe(opt, &credentials, opt.recv_window_ms));
    let report = build_production_order_state_readonly_proof_report(
        opt,
        &credentials,
        &missing_cli_flags,
        &missing_env_vars,
        missing_order_identifier,
        http_result.as_ref(),
    );

    if let Some(output) = &opt.output {
        write_production_order_state_readonly_report(output, &report, &credentials)?;
    }

    println!(
        "live.production_order_state_readonly_proof status={} endpoint={} endpoint_class={} method={} path={} symbol={} manual_online_requested={} endpoint_read_allowed={} offline_contract_ready={} contract_ready={} online_read_allowed={} online_execution_supported={} read_allowed={} mutation_allowed=false env_credentials_only=true credentials_used={} network_attempted={} order_state_read_attempted={} production_order_state_reads_attempted={} production_order_submission_attempted=false production_order_mutation_attempted=false listen_key_lifecycle_attempted=false dashboard_order_controls_enabled=false secrets_redacted=true response_shape={} response_shape_validated={} endpoint_shape_validated={} order_entries_observed={} non_empty_order_state_observed={} order_lifecycle_readiness={} error_code={}",
        report.status,
        report.endpoint,
        report.endpoint_class,
        report.method,
        report.path,
        report.symbol,
        report.manual_online_requested,
        report.endpoint_read_allowed,
        report.offline_contract_ready,
        report.contract_ready,
        report.online_read_allowed,
        report.online_execution_supported,
        report.read_allowed,
        report.api_key_present && report.api_secret_present,
        report.network_attempted,
        report.order_state_read_attempted,
        report.production_order_state_reads_attempted,
        report.response_shape,
        report.response_shape_validated,
        report.endpoint_shape_validated,
        report.order_entries_observed,
        report.non_empty_order_state_observed,
        report.order_lifecycle_readiness,
        report.error_code,
    );
    Ok(())
}

fn build_production_order_state_readonly_proof_report(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
    credentials: &EnvOnlyProductionReadCredentials,
    missing_cli_flags: &[&'static str],
    missing_env_vars: &[&'static str],
    missing_order_identifier: bool,
    http_result: Option<&ProductionOrderStateHttpResult>,
) -> ProductionOrderStateReadOnlyProofReport {
    let path = production_order_state_endpoint_path(opt.endpoint);
    let classified_endpoint = EndpointClassifier::classify(
        "GET",
        &format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{path}"),
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
    } else if missing_order_identifier {
        "blocked_missing_order_identifier"
    } else {
        "ready_offline_contract"
    };
    let diagnostic = if let Some(result) = http_result {
        result.diagnostic.as_str()
    } else if gates_missing && opt.manual_online {
        "manual online production order-state read is closed because explicit v0.14 owner gates are missing"
    } else if gates_missing {
        "production order-state read-only proof is closed because explicit CLI/env gates are missing"
    } else if credentials_missing {
        "production order-state read-only proof requires env-only API key and secret presence"
    } else if missing_order_identifier {
        "GET /api/v3/order requires orderId or origClientOrderId before any online proof"
    } else {
        "offline production order-state read-only contract is ready; no network was opened"
    };

    let endpoint_read_allowed = classified_endpoint.read_allowed;
    let offline_contract_ready = !gates_missing
        && !credentials_missing
        && !missing_order_identifier
        && !opt.manual_online
        && endpoint_read_allowed;
    let online_read_allowed = !gates_missing
        && !credentials_missing
        && !missing_order_identifier
        && opt.manual_online
        && endpoint_read_allowed;
    let response_shape = http_result.map_or_else(
        || production_order_state_response_shape(opt.endpoint).to_string(),
        |result| result.response_shape.clone(),
    );
    let order_state_read_attempted = http_result.is_some_and(|result| result.network_attempted);
    let response_shape_summary = http_result.map_or_else(
        || ProductionOrderStateShapeSummary::not_attempted(opt.endpoint),
        |result| result.response_shape_summary.clone(),
    );
    let response_shape_validated =
        http_result.is_some_and(|result| result.response_shape_validated);

    ProductionOrderStateReadOnlyProofReport {
        schema_version: PRODUCTION_ORDER_STATE_READONLY_SCHEMA_VERSION.to_string(),
        status: status.to_string(),
        endpoint: production_order_state_endpoint_name(opt.endpoint).to_string(),
        endpoint_class: classified_endpoint.endpoint_class.as_str().to_string(),
        http_base_url: BINANCE_PRODUCTION_HTTP_BASE_URL.to_string(),
        method: classified_endpoint.method,
        path: classified_endpoint.path,
        request_url_redacted: production_order_state_redacted_request_url(opt),
        query_shape: production_order_state_query_shape(opt),
        symbol: opt.symbol.clone(),
        order_id_provided: opt.order_id.is_some(),
        orig_client_order_id_provided: opt.orig_client_order_id.is_some(),
        requires_api_key: classified_endpoint.requires_api_key,
        requires_signature: classified_endpoint.requires_signature,
        endpoint_read_allowed,
        offline_contract_ready,
        read_allowed: offline_contract_ready,
        contract_ready: offline_contract_ready,
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
        network_attempted: order_state_read_attempted,
        response_status_code: http_result.and_then(|result| result.http_status),
        response_shape,
        response_shape_validated,
        endpoint_shape_validated: response_shape_summary.endpoint_shape_validated,
        order_entries_observed: response_shape_summary.order_entries_observed,
        non_empty_order_state_observed: response_shape_summary.non_empty_order_state_observed,
        order_lifecycle_readiness: response_shape_summary.order_lifecycle_readiness,
        response_shape_summary,
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
        order_state_read_attempted,
        production_order_state_reads_attempted: u64::from(order_state_read_attempted),
        production_order_submission_attempted: false,
        production_order_mutation_attempted: false,
        cancel_replace_amend_attempted: false,
        listen_key_lifecycle_attempted: false,
        dashboard_order_controls_enabled: false,
        automatic_remediation_attempted: false,
        real_orders_submitted: false,
        production_trading_enabled: false,
        order_state_values_are_exchange_truth: http_result
            .is_some_and(|result| result.response_shape_validated),
        shadow_values_are_exchange_truth: false,
        portfolio_values_are_exchange_truth: false,
        values_are_exchange_truth: http_result
            .is_some_and(|result| result.response_shape_validated),
        secrets_redacted: true,
        diagnostic: diagnostic.to_string(),
    }
}

fn should_attempt_production_order_state_read(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
    credentials: &EnvOnlyProductionReadCredentials,
    missing_cli_flags: &[&'static str],
    missing_env_vars: &[&'static str],
    missing_order_identifier: bool,
) -> bool {
    opt.manual_online
        && missing_cli_flags.is_empty()
        && missing_env_vars.is_empty()
        && credentials.api_key_present()
        && credentials.api_secret_present()
        && !missing_order_identifier
}

fn production_order_state_missing_identifier(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
) -> bool {
    opt.endpoint == ProductionOrderStateReadEndpoint::Order
        && opt.order_id.is_none()
        && opt.orig_client_order_id.is_none()
}

fn build_production_order_state_signed_request(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
    credentials: &EnvOnlyProductionReadCredentials,
    timestamp_ms: u64,
    recv_window_ms: u64,
) -> anyhow::Result<ProductionOrderStateSignedRequest> {
    if recv_window_ms == 0 {
        anyhow::bail!("production order-state recvWindow must be positive");
    }
    if production_order_state_missing_identifier(opt) {
        anyhow::bail!("GET /api/v3/order requires orderId or origClientOrderId");
    }

    let path = production_order_state_endpoint_path(opt.endpoint);
    let classified_endpoint = EndpointClassifier::classify(
        "GET",
        &format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{path}"),
        EndpointAuthKind::Signed,
    );
    if !classified_endpoint.read_allowed || classified_endpoint.mutation_allowed {
        anyhow::bail!(
            "production order-state allowlist rejected endpoint {}",
            classified_endpoint.path
        );
    }

    let signing_credential = credentials.signing_credential()?;
    let mut query_pairs = vec![
        ("symbol".to_string(), opt.symbol.clone()),
        ("timestamp".to_string(), timestamp_ms.to_string()),
        ("recvWindow".to_string(), recv_window_ms.to_string()),
    ];
    if let Some(order_id) = opt.order_id {
        query_pairs.push(("orderId".to_string(), order_id.to_string()));
    }
    if let Some(orig_client_order_id) = &opt.orig_client_order_id {
        query_pairs.push((
            "origClientOrderId".to_string(),
            orig_client_order_id.clone(),
        ));
    }
    let query_without_signature = join_query_pair_vec(&query_pairs);
    let signature =
        urlencoding::encode(&signing_credential.sign(&query_without_signature)).into_owned();
    let signed_query = format!("{query_without_signature}&signature={signature}");
    let request = ProductionOrderStateSignedRequest {
        method: "GET".to_string(),
        endpoint_path: path.to_string(),
        endpoint_url_redacted: format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{path}"),
        query_without_signature,
        signature,
        signed_query,
        api_key_header_name: BINANCE_API_KEY_HEADER.to_string(),
        api_key_header_value: signing_credential.api_key().to_string(),
    };
    request.ensure_redacted(credentials)?;
    Ok(request)
}

fn execute_production_order_state_read(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
    credentials: &EnvOnlyProductionReadCredentials,
    recv_window_ms: u64,
) -> ProductionOrderStateHttpResult {
    match build_production_order_state_signed_request(
        opt,
        credentials,
        current_unix_timestamp_ms().unwrap_or(0),
        recv_window_ms,
    ) {
        Ok(request) => {
            let signed_url = request.signed_url_for_execution();
            let api_key_header_name = request.api_key_header_name;
            let api_key_header_value = request.api_key_header_value;
            let endpoint = opt.endpoint;
            std::thread::spawn(move || {
                execute_production_order_state_read_on_thread(
                    endpoint,
                    &signed_url,
                    &api_key_header_name,
                    &api_key_header_value,
                )
            })
            .join()
            .unwrap_or_else(|_| {
                ProductionOrderStateHttpResult::failure(
                    opt.endpoint,
                    None,
                    None,
                    "http_probe_thread_panicked",
                )
            })
        }
        Err(_) => ProductionOrderStateHttpResult::failure(
            opt.endpoint,
            None,
            None,
            "signed_request_builder_failed",
        ),
    }
}

fn execute_production_order_state_read_on_thread(
    endpoint: ProductionOrderStateReadEndpoint,
    signed_url: &str,
    api_key_header_name: &str,
    api_key_header_value: &str,
) -> ProductionOrderStateHttpResult {
    let started = Instant::now();
    let client = match reqwest::blocking::Client::builder()
        .timeout(PRODUCTION_ORDER_STATE_PROBE_TIMEOUT)
        .user_agent("NTPRO-v140-production-order-state-readonly-proof")
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return ProductionOrderStateHttpResult::failure(
                endpoint,
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
                        let shape_summary = summarize_production_order_state_shape(endpoint, &body);
                        if shape_summary.shape_validated {
                            ProductionOrderStateHttpResult::success_with_shape(
                                endpoint,
                                latency_ms,
                                status,
                                shape_summary,
                            )
                        } else {
                            ProductionOrderStateHttpResult::failure_with_shape(
                                endpoint,
                                Some(latency_ms),
                                Some(status),
                                "response_shape_invalid",
                                shape_summary,
                            )
                        }
                    }
                    Err(_) => ProductionOrderStateHttpResult::failure(
                        endpoint,
                        Some(latency_ms),
                        Some(status),
                        "response_shape_invalid",
                    ),
                }
            } else {
                ProductionOrderStateHttpResult::failure(
                    endpoint,
                    Some(latency_ms),
                    Some(status),
                    "http_status_not_success",
                )
            }
        }
        Err(error) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            ProductionOrderStateHttpResult::failure(
                endpoint,
                Some(latency_ms),
                error.status().map(|status| status.as_u16()),
                classify_production_public_read_error(&error),
            )
        }
    }
}

fn production_order_state_endpoint_name(
    endpoint: ProductionOrderStateReadEndpoint,
) -> &'static str {
    match endpoint {
        ProductionOrderStateReadEndpoint::OpenOrders => "open_orders",
        ProductionOrderStateReadEndpoint::Order => "order",
    }
}

fn production_order_state_endpoint_path(
    endpoint: ProductionOrderStateReadEndpoint,
) -> &'static str {
    match endpoint {
        ProductionOrderStateReadEndpoint::OpenOrders => "/api/v3/openOrders",
        ProductionOrderStateReadEndpoint::Order => "/api/v3/order",
    }
}

fn production_order_state_response_shape(
    endpoint: ProductionOrderStateReadEndpoint,
) -> &'static str {
    match endpoint {
        ProductionOrderStateReadEndpoint::OpenOrders => "binance_open_orders_v1",
        ProductionOrderStateReadEndpoint::Order => "binance_order_state_v1",
    }
}

fn production_order_state_redacted_request_url(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
) -> String {
    format!(
        "{}{}?{}&signature=<redacted>",
        BINANCE_PRODUCTION_HTTP_BASE_URL,
        production_order_state_endpoint_path(opt.endpoint),
        production_order_state_query_shape(opt),
    )
}

fn production_order_state_query_shape(opt: &LiveProductionOrderStateReadOnlyProofOpt) -> String {
    let mut parts = vec![
        "symbol=<redacted>".to_string(),
        format!("recvWindow={}", opt.recv_window_ms),
        "timestamp=<redacted>".to_string(),
    ];
    if opt.order_id.is_some() {
        parts.push("orderId=<redacted>".to_string());
    }
    if opt.orig_client_order_id.is_some() {
        parts.push("origClientOrderId=<redacted>".to_string());
    }
    parts.join("&")
}

fn summarize_production_order_state_shape(
    endpoint: ProductionOrderStateReadEndpoint,
    body: &serde_json::Value,
) -> ProductionOrderStateShapeSummary {
    match endpoint {
        ProductionOrderStateReadEndpoint::OpenOrders => summarize_open_orders_shape(endpoint, body),
        ProductionOrderStateReadEndpoint::Order => summarize_single_order_shape(endpoint, body),
    }
}

fn summarize_open_orders_shape(
    endpoint: ProductionOrderStateReadEndpoint,
    body: &serde_json::Value,
) -> ProductionOrderStateShapeSummary {
    let Some(orders) = body.as_array() else {
        return ProductionOrderStateShapeSummary {
            status: "rejected".to_string(),
            root_is_array: false,
            root_is_object: false,
            rejection_reason: "root_not_array".to_string(),
            ..ProductionOrderStateShapeSummary::not_attempted(endpoint)
        };
    };
    let entries_valid = orders.iter().all(order_state_object_has_minimum_shape);
    let order_entries_observed = orders.len();
    let non_empty_order_state_observed = order_entries_observed > 0;
    let order_lifecycle_readiness = entries_valid && non_empty_order_state_observed;
    ProductionOrderStateShapeSummary {
        status: if entries_valid {
            "accepted"
        } else {
            "rejected"
        }
        .to_string(),
        endpoint: production_order_state_endpoint_name(endpoint).to_string(),
        root_is_array: true,
        root_is_object: false,
        order_entry_count: Some(orders.len()),
        symbol_present: orders.is_empty()
            || orders.iter().all(|entry| {
                entry
                    .get("symbol")
                    .is_some_and(serde_json::Value::is_string)
            }),
        symbol_is_string: orders.is_empty()
            || orders.iter().all(|entry| {
                entry
                    .get("symbol")
                    .is_some_and(serde_json::Value::is_string)
            }),
        order_id_present: orders.is_empty()
            || orders.iter().all(|entry| entry.get("orderId").is_some()),
        status_present: orders.is_empty()
            || orders.iter().all(|entry| entry.get("status").is_some()),
        status_is_string: orders.is_empty()
            || orders.iter().all(|entry| {
                entry
                    .get("status")
                    .is_some_and(serde_json::Value::is_string)
            }),
        raw_order_response_recorded: false,
        raw_order_list_recorded: false,
        shape_validated: entries_valid,
        endpoint_shape_validated: entries_valid,
        order_entries_observed,
        non_empty_order_state_observed,
        order_lifecycle_readiness,
        rejection_reason: if entries_valid {
            "none"
        } else {
            "missing_or_invalid_required_fields"
        }
        .to_string(),
    }
}

fn summarize_single_order_shape(
    endpoint: ProductionOrderStateReadEndpoint,
    body: &serde_json::Value,
) -> ProductionOrderStateShapeSummary {
    let Some(object) = body.as_object() else {
        return ProductionOrderStateShapeSummary {
            status: "rejected".to_string(),
            root_is_array: false,
            root_is_object: false,
            rejection_reason: "root_not_object".to_string(),
            ..ProductionOrderStateShapeSummary::not_attempted(endpoint)
        };
    };
    let symbol_is_string = object
        .get("symbol")
        .is_some_and(serde_json::Value::is_string);
    let order_id_present = object.contains_key("orderId");
    let status_is_string = object
        .get("status")
        .is_some_and(serde_json::Value::is_string);
    let shape_validated = symbol_is_string && order_id_present && status_is_string;
    let order_lifecycle_readiness = shape_validated;
    ProductionOrderStateShapeSummary {
        status: if shape_validated {
            "accepted"
        } else {
            "rejected"
        }
        .to_string(),
        endpoint: production_order_state_endpoint_name(endpoint).to_string(),
        root_is_array: false,
        root_is_object: true,
        order_entry_count: Some(1),
        symbol_present: object.contains_key("symbol"),
        symbol_is_string,
        order_id_present,
        status_present: object.contains_key("status"),
        status_is_string,
        raw_order_response_recorded: false,
        raw_order_list_recorded: false,
        shape_validated,
        endpoint_shape_validated: shape_validated,
        order_entries_observed: usize::from(shape_validated),
        non_empty_order_state_observed: shape_validated,
        order_lifecycle_readiness,
        rejection_reason: if shape_validated {
            "none"
        } else {
            "missing_or_invalid_required_fields"
        }
        .to_string(),
    }
}

fn order_state_object_has_minimum_shape(value: &serde_json::Value) -> bool {
    value.as_object().is_some_and(|object| {
        object
            .get("symbol")
            .is_some_and(serde_json::Value::is_string)
            && object.contains_key("orderId")
            && object
                .get("status")
                .is_some_and(serde_json::Value::is_string)
    })
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
    let notional_preflight = build_shadow_notional_preflight(&intent_inputs);
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
        notional_preflight,
        pnl,
        risk_summary,
        provenance: ShadowPortfolioProvenance {
            account_snapshot_source: "redacted_account_snapshot_summary".to_string(),
            shadow_intent_source: "local_shadow_execution_intent_jsonl".to_string(),
            balances_source: "redacted_shape_summary_only".to_string(),
            positions_source: "unavailable_without_production_fills".to_string(),
            exposure_source: "derived_from_local_shadow_intent_notional_only".to_string(),
            pnl_source: "unavailable_without_fills_cost_basis_and_mark_prices".to_string(),
            order_state_values_are_exchange_truth: false,
            shadow_values_are_exchange_truth: false,
            portfolio_values_are_exchange_truth: false,
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
    let Some(notional_sum) = intent_inputs.notional_sum.as_ref() else {
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

fn build_shadow_notional_preflight(intent_inputs: &ShadowIntentInputs) -> ShadowNotionalPreflight {
    let has_notional = intent_inputs.notional_sum.is_some();
    ShadowNotionalPreflight {
        status: if has_notional {
            "shadow_decimal_string_evidence_only".to_string()
        } else {
            "unavailable_shadow_notional".to_string()
        },
        aggregation: "rust_decimal_string_sum".to_string(),
        decimal_string_sum: intent_inputs.notional_sum.as_ref().map(format_decimal),
        parsed_notional_count: intent_inputs.parsed_notional_count,
        f64_aggregation_used: false,
        live_alpha_money_math_ready: false,
        risk_or_execution_grade: false,
        reason: if has_notional {
            "local shadow intent notionals are summed with Decimal/string evidence for display only; live-alpha risk/execution must revalidate with dedicated money math".to_string()
        } else {
            "no parseable shadow intent notional was available; live-alpha risk/execution must revalidate with dedicated money math".to_string()
        },
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
    let mut notional_sum = Decimal::new(0, 0);
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
        parsed_notional_count,
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
        order_state_values_are_exchange_truth: false,
        shadow_values_are_exchange_truth: false,
        portfolio_values_are_exchange_truth: false,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShadowPreflightLoopResult {
    events_written: usize,
    heartbeats_written: u64,
    final_state: String,
    stop_file_observed: bool,
    stale_data_detected: bool,
}

async fn run_production_shadow_preflight_session_loop(
    opt: &LiveProductionShadowPreflightSessionOpt,
) -> anyhow::Result<ShadowPreflightLoopResult> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("strategy_id", &opt.strategy_id)?;
    if opt.max_heartbeats == 0 {
        anyhow::bail!("max_heartbeats must be greater than zero");
    }
    let heartbeat_interval = non_zero_duration("heartbeat_interval_ms", opt.heartbeat_interval_ms)?;
    let _stale_after = non_zero_duration("stale_after_ms", opt.stale_after_ms)?;
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

    let mut writer = create_jsonl_writer(&opt.output)?;
    let mut events_written = 0_usize;
    let mut heartbeats_written = 0_u64;
    let mut terminal_written = false;
    let mut final_state = base_state.to_string();
    let mut stop_file_observed = false;
    let mut stale_data_detected = false;

    append_shadow_preflight_session_event(
        &mut writer,
        &build_shadow_preflight_session_event(ShadowPreflightEventInput {
            opt,
            session_id: &session_id,
            event_type: "shadow_preflight_session_started",
            state: base_state,
            heartbeat_seq: None,
            portfolio_ref: &portfolio_ref,
            session_status_ref: session_status_ref.clone(),
            artifact_gap: artifact_gap.clone(),
            source_artifact_age_ms: artifact_age_ms(&opt.shadow_portfolio_runtime)?,
            stale_data_detected: false,
            stop_file_observed: false,
            shutdown_reason: None,
            diagnostic: "local guarded-live-alpha preflight loop started from read-only shadow artifacts",
        }),
    )?;
    events_written += 1;

    for heartbeat_seq in 1..=opt.max_heartbeats {
        if stop_file_exists(opt.stop_file.as_deref()) {
            stop_file_observed = true;
            final_state = "stopped".to_string();
            append_shadow_preflight_session_event(
                &mut writer,
                &build_shadow_preflight_session_event(ShadowPreflightEventInput {
                    opt,
                    session_id: &session_id,
                    event_type: "shadow_preflight_session_stopped",
                    state: "stopped",
                    heartbeat_seq: None,
                    portfolio_ref: &portfolio_ref,
                    session_status_ref: session_status_ref.clone(),
                    artifact_gap: artifact_gap.clone(),
                    source_artifact_age_ms: artifact_age_ms(&opt.shadow_portfolio_runtime)?,
                    stale_data_detected: false,
                    stop_file_observed,
                    shutdown_reason: Some("owner_stop_file"),
                    diagnostic: "local owner stop-file observed; preflight loop stopped without production mutation",
                }),
            )?;
            events_written += 1;
            terminal_written = true;
            break;
        }

        let source_artifact_age_ms = artifact_age_ms(&opt.shadow_portfolio_runtime)?;
        if source_artifact_age_ms.is_some_and(|age_ms| age_ms > opt.stale_after_ms) {
            stale_data_detected = true;
            final_state = "stale_data_halted".to_string();
            append_shadow_preflight_session_event(
                &mut writer,
                &build_shadow_preflight_session_event(ShadowPreflightEventInput {
                    opt,
                    session_id: &session_id,
                    event_type: "shadow_preflight_stale_data_detected",
                    state: "stale_data_halted",
                    heartbeat_seq: None,
                    portfolio_ref: &portfolio_ref,
                    session_status_ref: session_status_ref.clone(),
                    artifact_gap: artifact_gap.clone(),
                    source_artifact_age_ms,
                    stale_data_detected,
                    stop_file_observed: false,
                    shutdown_reason: Some("stale_shadow_portfolio_runtime"),
                    diagnostic: "shadow portfolio runtime artifact exceeded stale threshold; preflight loop halted without production mutation",
                }),
            )?;
            events_written += 1;
            terminal_written = true;
            break;
        }

        append_shadow_preflight_session_event(
            &mut writer,
            &build_shadow_preflight_session_event(ShadowPreflightEventInput {
                opt,
                session_id: &session_id,
                event_type: "shadow_preflight_session_heartbeat",
                state: base_state,
                heartbeat_seq: Some(heartbeat_seq),
                portfolio_ref: &portfolio_ref,
                session_status_ref: session_status_ref.clone(),
                artifact_gap: artifact_gap.clone(),
                source_artifact_age_ms,
                stale_data_detected: false,
                stop_file_observed: false,
                shutdown_reason: None,
                diagnostic: "local guarded-live-alpha preflight heartbeat; no production mutation attempted",
            }),
        )?;
        events_written += 1;
        heartbeats_written += 1;

        if heartbeat_seq < opt.max_heartbeats {
            sleep(heartbeat_interval).await;
        }
    }

    if !terminal_written {
        final_state = "stopped".to_string();
        append_shadow_preflight_session_event(
            &mut writer,
            &build_shadow_preflight_session_event(ShadowPreflightEventInput {
                opt,
                session_id: &session_id,
                event_type: "shadow_preflight_session_stopped",
                state: "stopped",
                heartbeat_seq: None,
                portfolio_ref: &portfolio_ref,
                session_status_ref,
                artifact_gap,
                source_artifact_age_ms: artifact_age_ms(&opt.shadow_portfolio_runtime)?,
                stale_data_detected: false,
                stop_file_observed: false,
                shutdown_reason: Some("max_heartbeats_reached"),
                diagnostic: "local max heartbeat bound reached; preflight loop stopped without production mutation",
            }),
        )?;
        events_written += 1;
    }

    writer.flush().with_context(|| {
        format!(
            "failed to flush shadow preflight session '{}'",
            opt.output.display()
        )
    })?;

    Ok(ShadowPreflightLoopResult {
        events_written,
        heartbeats_written,
        final_state,
        stop_file_observed,
        stale_data_detected,
    })
}

fn create_jsonl_writer(path: &Path) -> anyhow::Result<fs::File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory '{}'", parent.display()))?;
    }
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("failed to open JSONL output '{}'", path.display()))
}

fn append_shadow_preflight_session_event(
    writer: &mut fs::File,
    event: &ProductionShadowPreflightSessionEvent,
) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *writer, event)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn build_shadow_preflight_session_event(
    input: ShadowPreflightEventInput<'_>,
) -> ProductionShadowPreflightSessionEvent {
    ProductionShadowPreflightSessionEvent {
        schema_version: PRODUCTION_SHADOW_PREFLIGHT_SESSION_EVENT_SCHEMA_VERSION.to_string(),
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
        source_artifact_age_ms: input.source_artifact_age_ms,
        stale_after_ms: input.opt.stale_after_ms,
        stale_data_detected: input.stale_data_detected,
        stop_file_observed: input.stop_file_observed,
        stop_file_path: input
            .opt
            .stop_file
            .as_ref()
            .map(|path| path.display().to_string()),
        shutdown_reason: input.shutdown_reason.map(ToString::to_string),
        session_network_attempted: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: false,
        order_state_values_are_exchange_truth: false,
        shadow_values_are_exchange_truth: false,
        portfolio_values_are_exchange_truth: false,
        values_are_exchange_truth: false,
        diagnostic: input.diagnostic.to_string(),
    }
}

fn build_production_live_alpha_dry_run_order_gate_artifact(
    opt: &LiveProductionLiveAlphaDryRunOrderGateOpt,
) -> anyhow::Result<ProductionLiveAlphaDryRunOrderGateArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("strategy_id", &opt.strategy_id)?;
    validate_non_empty("symbol", &opt.symbol)?;
    validate_positive_decimal_string("quantity", &opt.quantity)?;
    validate_positive_decimal_string("notional", &opt.notional)?;

    let session_id = opt
        .session_id
        .as_deref()
        .unwrap_or(opt.run_id.as_str())
        .to_string();
    validate_non_empty("session_id", &session_id)?;

    let side = opt.side.trim().to_ascii_uppercase();
    if !matches!(side.as_str(), "BUY" | "SELL") {
        anyhow::bail!("side must be BUY or SELL");
    }
    let order_type = opt.order_type.trim().to_ascii_uppercase();
    if order_type != "LIMIT" {
        anyhow::bail!("production live-alpha dry-run order gate only supports LIMIT order_type");
    }

    let missing_cli_flags = missing_production_live_alpha_dry_run_order_gate_cli_flags(opt);
    let dry_run_order_gate_ready = missing_cli_flags.is_empty();
    let status = if dry_run_order_gate_ready {
        "ready_dry_run_no_submission"
    } else {
        "blocked_missing_gate"
    };

    Ok(ProductionLiveAlphaDryRunOrderGateArtifact {
        schema_version: PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        session_id,
        strategy_id: opt.strategy_id.clone(),
        artifact_type: "live_alpha_dry_run_order_gate".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "production_live_alpha_dry_run".to_string(),
        symbol: opt.symbol.trim().to_string(),
        side,
        order_type,
        quantity: opt.quantity.trim().to_string(),
        notional: opt.notional.trim().to_string(),
        owner_gate_required: true,
        manual_gate_required: true,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        dry_run_order_intent_recorded: dry_run_order_gate_ready,
        dry_run_order_gate_ready,
        order_submission_mode: if dry_run_order_gate_ready {
            "dry_run_no_submission"
        } else {
            "blocked_missing_gate"
        }
        .to_string(),
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        execution_adapter_called: false,
        matching_engine_submission: false,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        external_venue_connection: false,
        network_attempted: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        order_state_values_are_exchange_truth: false,
        shadow_values_are_exchange_truth: false,
        portfolio_values_are_exchange_truth: false,
        values_are_exchange_truth: false,
        no_production_order_submission_confirmed: opt.confirm_no_production_order_submission,
        no_production_order_mutation_confirmed: opt.confirm_no_production_order_mutation,
        no_execution_adapter_call_confirmed: opt.confirm_no_execution_adapter_call,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_real_funds_confirmed: opt.confirm_no_real_funds,
        diagnostic: if dry_run_order_gate_ready {
            "local live-alpha dry-run order gate is ready; no production order was submitted or mutated"
        } else {
            "live-alpha dry-run order gate is blocked until all explicit owner dry-run confirmations are present"
        }
        .to_string(),
    })
}

fn build_production_live_alpha_manual_approval_lifecycle_artifact(
    opt: &LiveProductionLiveAlphaManualApprovalLifecycleOpt,
) -> anyhow::Result<ProductionLiveAlphaManualApprovalLifecycleArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("strategy_id", &opt.strategy_id)?;
    validate_non_empty("symbol", &opt.symbol)?;
    validate_positive_decimal_string("notional", &opt.notional)?;
    if opt.now_unix_ms == 0 {
        anyhow::bail!("manual approval lifecycle now_unix_ms must be positive");
    }
    if opt.expires_at_unix_ms == 0 {
        anyhow::bail!("manual approval lifecycle expires_at_unix_ms must be positive");
    }

    let approval_state = opt.approval_state.trim();
    if !matches!(
        approval_state,
        "pending" | "approved" | "expired" | "revoked" | "used"
    ) {
        anyhow::bail!("approval_state must be pending, approved, expired, revoked, or used");
    }
    let manual_approval_id = optional_non_empty("manual_approval_id", &opt.manual_approval_id)?;
    let approved_by = optional_non_empty("approved_by", &opt.approved_by)?;
    if approval_state != "pending" {
        if manual_approval_id.is_none() {
            anyhow::bail!("non-pending approval_state requires --manual-approval-id");
        }
        if approved_by.is_none() {
            anyhow::bail!("non-pending approval_state requires --approved-by");
        }
    }

    let approval_expired = approval_state == "expired" || opt.now_unix_ms > opt.expires_at_unix_ms;
    let approval_revoked = approval_state == "revoked";
    let approval_used = approval_state == "used";
    let manual_approval_recorded =
        manual_approval_id.is_some() && approved_by.is_some() && approval_state != "pending";

    let mut lifecycle_issues = Vec::new();
    if !opt.confirm_dry_run_request_preview_only {
        lifecycle_issues.push("missing_dry_run_request_preview_only_confirmation".to_string());
    }
    if !opt.confirm_one_time_approval {
        lifecycle_issues.push("missing_one_time_approval_confirmation".to_string());
    }
    if !opt.confirm_no_production_mutation {
        lifecycle_issues.push("missing_no_production_mutation_confirmation".to_string());
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        lifecycle_issues.push("missing_dashboard_controls_disabled_confirmation".to_string());
    }
    if approval_state != "approved" {
        lifecycle_issues.push(format!("approval_state_{approval_state}"));
    }
    if approval_expired {
        lifecycle_issues.push("approval_expired".to_string());
    }
    if approval_revoked {
        lifecycle_issues.push("approval_revoked".to_string());
    }
    if approval_used {
        lifecycle_issues.push("approval_used".to_string());
    }
    if !manual_approval_recorded {
        lifecycle_issues.push("manual_approval_not_recorded".to_string());
    }

    let approval_lifecycle_valid = lifecycle_issues.is_empty();
    let status = if approval_lifecycle_valid {
        "approval_valid_for_dry_run_request_preview"
    } else if approval_state == "pending" {
        "approval_pending"
    } else if approval_expired {
        "approval_expired"
    } else if approval_revoked {
        "approval_revoked"
    } else if approval_used {
        "approval_used"
    } else {
        "approval_invalid"
    };

    Ok(ProductionLiveAlphaManualApprovalLifecycleArtifact {
        schema_version: PRODUCTION_LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        strategy_id: opt.strategy_id.clone(),
        symbol: opt.symbol.trim().to_string(),
        notional: opt.notional.trim().to_string(),
        artifact_type: "live_alpha_manual_approval_lifecycle".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        approval_state: approval_state.to_string(),
        manual_approval_recorded,
        manual_approval_id,
        approved_by,
        now_unix_ms: opt.now_unix_ms,
        expires_at_unix_ms: opt.expires_at_unix_ms,
        approval_expired,
        approval_revoked,
        approval_used,
        dry_run_request_preview_only: true,
        one_time_approval: true,
        approval_lifecycle_valid,
        lifecycle_issues,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        execution_adapter_called: false,
        production_adapter_called: false,
        matching_engine_submission: false,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        external_venue_connection: false,
        network_attempted: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        dry_run_request_preview_only_confirmed: opt.confirm_dry_run_request_preview_only,
        one_time_approval_confirmed: opt.confirm_one_time_approval,
        no_production_mutation_confirmed: opt.confirm_no_production_mutation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        diagnostic: if approval_lifecycle_valid {
            "manual approval is valid for one dry-run request preview only; production mutation remains disabled"
        } else {
            "manual approval lifecycle is not valid for dry-run request preview progression"
        }
        .to_string(),
    })
}

fn build_production_live_alpha_order_request_preview_artifact(
    opt: &LiveProductionLiveAlphaOrderRequestPreviewOpt,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> anyhow::Result<ProductionLiveAlphaOrderRequestPreviewArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_positive_decimal_string("price", &opt.price)?;
    if opt.recv_window_ms == 0 {
        anyhow::bail!("production live-alpha request preview recvWindow must be positive");
    }
    if opt.timestamp_ms == 0 {
        anyhow::bail!("production live-alpha request preview timestamp_ms must be positive");
    }

    let endpoint_path = normalize_production_live_alpha_order_endpoint_path(&opt.endpoint_path)?;
    let time_in_force = opt.time_in_force.trim().to_ascii_uppercase();
    if time_in_force != "GTC" {
        anyhow::bail!("production live-alpha request preview only supports GTC time-in-force");
    }

    let order_gate = load_json_value(&opt.order_gate, "live-alpha dry-run order gate")?;
    let manual_approval_lifecycle = load_json_value(
        &opt.manual_approval_lifecycle,
        "live-alpha manual approval lifecycle",
    )?;
    let order_gate_schema = required_json_string(&order_gate, "schema_version")?;
    if order_gate_schema != PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION {
        let required_schema = PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION;
        anyhow::bail!(
            "production live-alpha request preview requires {required_schema} order gate, got {order_gate_schema}"
        );
    }

    let session_id = required_json_string(&order_gate, "session_id")?;
    let strategy_id = required_json_string(&order_gate, "strategy_id")?;
    let symbol = required_json_string(&order_gate, "symbol")?;
    let side = required_json_string(&order_gate, "side")?.to_ascii_uppercase();
    let order_type = required_json_string(&order_gate, "order_type")?.to_ascii_uppercase();
    let quantity = required_json_string(&order_gate, "quantity")?;
    let notional = required_json_string(&order_gate, "notional")?;
    validate_non_empty("session_id", &session_id)?;
    validate_non_empty("strategy_id", &strategy_id)?;
    validate_non_empty("symbol", &symbol)?;
    validate_positive_decimal_string("quantity", &quantity)?;
    validate_positive_decimal_string("notional", &notional)?;
    if !matches!(side.as_str(), "BUY" | "SELL") {
        anyhow::bail!("production live-alpha request preview side must be BUY or SELL");
    }
    if order_type != "LIMIT" {
        anyhow::bail!("production live-alpha request preview only supports LIMIT order gates");
    }

    let missing_cli_flags = missing_production_live_alpha_order_request_preview_cli_flags(opt);
    let missing_env_vars =
        missing_production_live_alpha_order_request_preview_env_vars(credentials);
    let manual_approval_lifecycle_issues =
        production_live_alpha_request_preview_manual_approval_issues(
            &manual_approval_lifecycle,
            opt,
            &strategy_id,
            &symbol,
            &notional,
        );
    let manual_approval_lifecycle_valid = manual_approval_lifecycle_issues.is_empty();
    let order_gate_ready =
        json_bool_value(&order_gate, "dry_run_order_gate_ready").unwrap_or(false);
    let owner_manual_scope = order_gate_ready
        && missing_cli_flags.is_empty()
        && missing_env_vars.is_empty()
        && manual_approval_lifecycle_valid;
    let endpoint_url = format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{endpoint_path}");
    let classified = EndpointClassifier::classify_with_context(
        "POST",
        &endpoint_url,
        EndpointAuthKind::Signed,
        owner_manual_scope,
    );
    let request_preview_allowed = owner_manual_scope && classified.request_preview_allowed;
    let mut request_preview_built = false;
    let mut signature_preflight = "skipped_blocked".to_string();
    let manual_approval_already_used =
        json_bool_value(&manual_approval_lifecycle, "approval_used").unwrap_or(true);

    if request_preview_allowed {
        let request = build_production_live_alpha_signed_order_request_preview(
            &ProductionLiveAlphaOrderRequestInput {
                endpoint_path: &endpoint_path,
                symbol: &symbol,
                side: &side,
                order_type: &order_type,
                quantity: &quantity,
                price: opt.price.trim(),
                time_in_force: &time_in_force,
                recv_window_ms: opt.recv_window_ms,
                timestamp_ms: opt.timestamp_ms,
            },
            credentials,
        )?;
        request.ensure_memory_only_redacted(credentials)?;
        request_preview_built = true;
        signature_preflight = "created_in_memory_not_recorded".to_string();
    }
    let manual_approval_consumed = request_preview_built;
    let manual_approval_used = manual_approval_already_used || manual_approval_consumed;
    let manual_approval_consume_status = if manual_approval_consumed {
        "approval_consumed_after_request_preview_created"
    } else if manual_approval_already_used {
        "approval_already_used"
    } else {
        "not_consumed"
    };
    let manual_approval_consume_transition = if manual_approval_consumed {
        "approved_to_request_preview_created_to_used"
    } else {
        "not_consumed"
    };

    let status = if request_preview_built {
        "ready_request_preview_only"
    } else if !manual_approval_lifecycle_valid {
        "blocked_manual_approval_lifecycle"
    } else if !classified.request_preview_allowed {
        "blocked_endpoint_or_owner_scope"
    } else {
        "blocked_missing_gate"
    };

    Ok(ProductionLiveAlphaOrderRequestPreviewArtifact {
        schema_version: PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        session_id,
        strategy_id,
        source_order_gate_path: opt.order_gate.display().to_string(),
        source_manual_approval_lifecycle_path: opt
            .manual_approval_lifecycle
            .display()
            .to_string(),
        artifact_type: "live_alpha_order_request_preview".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "production_live_alpha_request_preview_only".to_string(),
        endpoint_class: classified.endpoint_class.as_str().to_string(),
        endpoint_decision: endpoint_decision_label(classified.decision).to_string(),
        endpoint_reason: classified.reason,
        endpoint_url_redacted: classified.input_url_redacted,
        request_method: "POST".to_string(),
        request_target: endpoint_path,
        query_shape_without_signature: production_live_alpha_order_query_shape_without_signature(),
        signature_preflight,
        symbol: symbol.trim().to_string(),
        side,
        order_type,
        quantity: quantity.trim().to_string(),
        price: opt.price.trim().to_string(),
        time_in_force,
        notional: notional.trim().to_string(),
        recv_window_ms: opt.recv_window_ms,
        timestamp_recorded: false,
        timestamp_shape: "epoch_millis_present_redacted".to_string(),
        credential_material: credentials.credential_material.clone(),
        production_signing_material_gate_required: credentials
            .production_signing_material_gate_required,
        production_signing_material_gate_open: credentials.production_signing_material_gate_open,
        production_signing_material_env_read: credentials.production_signing_material_env_read,
        production_signing_material_missing_gate_env_vars: credentials
            .production_signing_material_missing_gate_env_vars
            .clone(),
        api_key_env: credentials.api_key_env.clone(),
        api_secret_env: credentials.api_secret_env.clone(),
        api_key_header_name: BINANCE_API_KEY_HEADER.to_string(),
        api_key_header_value_recorded: false,
        api_secret_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        request_body_recorded: false,
        raw_request_body_recorded: false,
        owner_gate_required: true,
        manual_gate_required: true,
        manual_approval_lifecycle_status: json_string_value(
            &manual_approval_lifecycle,
            "status",
        )
        .unwrap_or_else(|| "unknown".to_string()),
        manual_approval_lifecycle_state: json_string_value(
            &manual_approval_lifecycle,
            "approval_state",
        )
        .unwrap_or_else(|| "unknown".to_string()),
        manual_approval_lifecycle_valid,
        manual_approval_lifecycle_issues,
        manual_approval_id: json_string_value(&manual_approval_lifecycle, "manual_approval_id"),
        manual_approval_expires_at_unix_ms: json_u64_value(
            &manual_approval_lifecycle,
            "expires_at_unix_ms",
        ),
        manual_approval_now_unix_ms: json_u64_value(&manual_approval_lifecycle, "now_unix_ms"),
        manual_approval_one_time: json_bool_value(&manual_approval_lifecycle, "one_time_approval")
            .unwrap_or(false),
        manual_approval_used,
        manual_approval_consumed,
        manual_approval_consume_status: manual_approval_consume_status.to_string(),
        manual_approval_consume_transition: manual_approval_consume_transition.to_string(),
        manual_approval_consume_artifact_path: if manual_approval_consumed {
            opt.manual_approval_lifecycle.display().to_string()
        } else {
            String::new()
        },
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        missing_env_vars,
        order_gate_ready,
        request_preview_allowed,
        request_preview_built,
        request_sent: false,
        order_submission_mode: if request_preview_built {
            "dry_run_request_preview_only"
        } else {
            "blocked_no_request_preview"
        }
        .to_string(),
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        execution_adapter_called: false,
        production_adapter_called: false,
        matching_engine_submission: false,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        external_venue_connection: false,
        network_attempted: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        signed_request_memory_only: request_preview_built,
        secrets_redacted: true,
        no_production_order_submission_confirmed: opt.confirm_no_production_order_submission,
        no_production_order_mutation_confirmed: opt.confirm_no_production_order_mutation,
        no_execution_adapter_call_confirmed: opt.confirm_no_execution_adapter_call,
        no_network_confirmed: opt.confirm_no_network,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_real_funds_confirmed: opt.confirm_no_real_funds,
        diagnostic: if request_preview_built {
            "production live-alpha order request preview built as redacted metadata only; signed query, signature, signed URL, network, and adapter execution remain disabled"
        } else if !manual_approval_lifecycle_valid {
            "production live-alpha order request preview is blocked by manual approval lifecycle state or binding"
        } else {
            "production live-alpha order request preview is blocked until dry-run gate, owner confirmations, env-only credentials, and endpoint classifier scope all pass"
        }
        .to_string(),
    })
}

fn consume_production_live_alpha_manual_approval_lifecycle(
    approval_path: &Path,
    request_preview_path: &Path,
    request_preview: &ProductionLiveAlphaOrderRequestPreviewArtifact,
) -> anyhow::Result<()> {
    let mut approval = load_json_value(
        approval_path,
        "live-alpha manual approval lifecycle consume artifact",
    )?;
    if json_string_value(&approval, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_SCHEMA_VERSION)
    {
        anyhow::bail!("manual approval consume requires v0.15 manual approval lifecycle schema");
    }
    if json_string_value(&approval, "approval_state").as_deref() != Some("approved") {
        anyhow::bail!("manual approval consume requires approval_state=approved");
    }
    if json_bool_value(&approval, "approval_used").unwrap_or(true) {
        anyhow::bail!("manual approval consume requires unused one-time approval");
    }

    let mut lifecycle_issues = json_string_array(&approval, "lifecycle_issues");
    if !lifecycle_issues
        .iter()
        .any(|issue| issue == "manual_approval_used")
    {
        lifecycle_issues.push("manual_approval_used".to_string());
    }

    let Some(object) = approval.as_object_mut() else {
        anyhow::bail!("manual approval lifecycle consume artifact must be a JSON object");
    };
    object.insert("approval_state".to_string(), json!("used"));
    object.insert(
        "status".to_string(),
        json!("approval_consumed_after_request_preview_created"),
    );
    object.insert("approval_used".to_string(), json!(true));
    object.insert("approval_lifecycle_valid".to_string(), json!(false));
    object.insert("request_preview_created".to_string(), json!(true));
    object.insert("approval_consumed".to_string(), json!(true));
    object.insert(
        "approval_consume_transition".to_string(),
        json!("approved_to_request_preview_created_to_used"),
    );
    object.insert(
        "consumed_by_request_preview_run_id".to_string(),
        json!(request_preview.run_id.clone()),
    );
    object.insert(
        "consumed_request_preview_path".to_string(),
        json!(request_preview_path.display().to_string()),
    );
    object.insert("approval_consumed_at".to_string(), json!(now_millis()));
    object.insert(
        "diagnostic".to_string(),
        json!("manual approval was consumed after request preview creation and cannot be reused"),
    );
    object.insert("lifecycle_issues".to_string(), json!(lifecycle_issues));

    atomic_write_json(approval_path, &approval)?;
    Ok(())
}

fn production_live_alpha_request_preview_manual_approval_issues(
    approval: &serde_json::Value,
    opt: &LiveProductionLiveAlphaOrderRequestPreviewOpt,
    strategy_id: &str,
    symbol: &str,
    notional: &str,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(approval, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_SCHEMA_VERSION)
    {
        issues.push("manual_approval_schema_mismatch".to_string());
    }
    if json_string_value(approval, "approval_state").as_deref() != Some("approved") {
        issues.push("manual_approval_not_approved".to_string());
    }
    if json_string_value(approval, "status").as_deref()
        != Some("approval_valid_for_dry_run_request_preview")
    {
        issues.push("manual_approval_lifecycle_not_valid".to_string());
    }
    if !json_bool_value(approval, "manual_approval_recorded").unwrap_or(false) {
        issues.push("manual_approval_not_recorded".to_string());
    }
    if !json_bool_value(approval, "dry_run_request_preview_only").unwrap_or(false) {
        issues.push("manual_approval_scope_not_request_preview_only".to_string());
    }
    if !json_bool_value(approval, "one_time_approval").unwrap_or(false) {
        issues.push("manual_approval_not_one_time".to_string());
    }
    if json_bool_value(approval, "approval_expired").unwrap_or(true) {
        issues.push("manual_approval_expired".to_string());
    }
    if json_bool_value(approval, "approval_revoked").unwrap_or(false) {
        issues.push("manual_approval_revoked".to_string());
    }
    if json_bool_value(approval, "approval_used").unwrap_or(true) {
        issues.push("manual_approval_used".to_string());
    }
    match (
        json_u64_value(approval, "now_unix_ms"),
        json_u64_value(approval, "expires_at_unix_ms"),
    ) {
        (Some(now), Some(expires_at)) if now <= expires_at => {}
        (Some(_), Some(_)) => issues.push("manual_approval_expired_by_time".to_string()),
        _ => issues.push("manual_approval_expiry_missing".to_string()),
    }
    push_json_string_expected_issue(&mut issues, "run_id", approval, &opt.run_id);
    push_json_string_expected_issue(&mut issues, "strategy_id", approval, strategy_id);
    push_json_string_expected_issue(&mut issues, "symbol", approval, symbol);
    push_json_string_expected_issue(&mut issues, "notional", approval, notional);
    if artifact_has_production_mutation(Some(approval)) {
        issues.push("manual_approval_records_forbidden_production_mutation".to_string());
    }
    issues
}

fn push_json_string_expected_issue(
    issues: &mut Vec<String>,
    field: &str,
    value: &serde_json::Value,
    expected: &str,
) {
    if json_string_value(value, field).as_deref() != Some(expected) {
        issues.push(format!("manual_approval_{field}_mismatch"));
    }
}

fn build_production_live_alpha_kill_switch_runtime_gate_artifact(
    opt: &LiveProductionLiveAlphaKillSwitchRuntimeGateOpt,
) -> anyhow::Result<ProductionLiveAlphaKillSwitchRuntimeGateArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    let kill_switch_approval =
        load_json_value(&opt.kill_switch_approval, "kill-switch approval artifact")?;
    let risk_preflight = load_json_value(&opt.risk_preflight, "live-alpha risk preflight")?;
    let request_preview = load_json_value(&opt.request_preview, "live-alpha request preview")?;

    let missing_cli_flags = missing_production_live_alpha_kill_switch_runtime_gate_cli_flags(opt);
    let source_artifact_issues = production_live_alpha_kill_switch_runtime_gate_source_issues(
        &kill_switch_approval,
        &risk_preflight,
        &request_preview,
    );
    let kill_switch_enabled =
        json_bool_value(&kill_switch_approval, "kill_switch_enabled").unwrap_or(false);
    let approval_kill_switch_active =
        json_bool_value(&kill_switch_approval, "kill_switch_active").unwrap_or(true);
    let risk_preflight_kill_switch_active =
        json_bool_value(&risk_preflight, "kill_switch_active").unwrap_or(true);
    let kill_switch_active = approval_kill_switch_active || risk_preflight_kill_switch_active;
    let approval_state = json_string_value(&kill_switch_approval, "approval_state")
        .unwrap_or_else(|| "unknown".to_string());
    let manual_approval_recorded =
        json_bool_value(&kill_switch_approval, "manual_approval_recorded").unwrap_or(false);
    let manual_approval_approved = manual_approval_recorded && approval_state == "approved";
    let request_preview_status =
        json_string_value(&request_preview, "status").unwrap_or_else(|| "unknown".to_string());
    let request_preview_built =
        json_bool_value(&request_preview, "request_preview_built").unwrap_or(false);
    let request_sent = json_bool_value(&request_preview, "request_sent").unwrap_or(false);
    let request_preview_ready = request_preview_status == "ready_request_preview_only"
        && request_preview_built
        && !request_sent;

    let mut runtime_gate_reasons = Vec::new();
    if !missing_cli_flags.is_empty() {
        runtime_gate_reasons.push("missing_owner_runtime_gate_confirmation".to_string());
    }
    if kill_switch_active {
        runtime_gate_reasons.push("kill_switch_active".to_string());
    }
    if !manual_approval_approved {
        runtime_gate_reasons.push("manual_approval_missing_or_not_approved".to_string());
    }
    if !request_preview_ready {
        runtime_gate_reasons.push("request_preview_blocked".to_string());
    }
    if !source_artifact_issues.is_empty() {
        runtime_gate_reasons.push("source_artifact_issue".to_string());
    }

    let runtime_gate_open = runtime_gate_reasons.is_empty();
    let status = if runtime_gate_open {
        "ready_runtime_gate_open_for_dry_run_only"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if kill_switch_active {
        "blocked_kill_switch_active"
    } else if !manual_approval_approved {
        "blocked_missing_manual_approval"
    } else if !request_preview_ready {
        "blocked_request_preview"
    } else {
        "blocked_source_artifact"
    };
    let runtime_gate_decision = if runtime_gate_open {
        "dry_run_runtime_gate_open"
    } else {
        "blocked_no_runtime_mutation"
    };

    Ok(ProductionLiveAlphaKillSwitchRuntimeGateArtifact {
        schema_version: PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_kill_switch_approval_path: opt.kill_switch_approval.display().to_string(),
        source_risk_preflight_path: opt.risk_preflight.display().to_string(),
        source_request_preview_path: opt.request_preview.display().to_string(),
        artifact_type: "live_alpha_kill_switch_runtime_gate".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "production_live_alpha_kill_switch_runtime_gate".to_string(),
        runtime_gate_decision: runtime_gate_decision.to_string(),
        runtime_gate_open,
        runtime_gate_reasons,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        kill_switch_enabled,
        kill_switch_active,
        approval_state,
        manual_approval_required: true,
        manual_approval_recorded,
        manual_approval_id: json_string_value(&kill_switch_approval, "manual_approval_id"),
        approved_by: json_string_value(&kill_switch_approval, "approved_by"),
        risk_preflight_decision: json_string_value(&risk_preflight, "risk_decision")
            .unwrap_or_else(|| "unknown".to_string()),
        risk_preflight_kill_switch_active,
        request_preview_status,
        request_preview_built,
        request_sent,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        execution_adapter_called: false,
        production_adapter_called: false,
        matching_engine_submission: false,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        external_venue_connection: false,
        network_attempted: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        no_production_order_submission_confirmed: opt.confirm_no_production_order_submission,
        no_production_order_mutation_confirmed: opt.confirm_no_production_order_mutation,
        no_network_confirmed: opt.confirm_no_network,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_real_funds_confirmed: opt.confirm_no_real_funds,
        diagnostic: if runtime_gate_open {
            "kill-switch runtime gate is open only for local dry-run progression; production mutation remains disabled"
        } else {
            "kill-switch runtime gate blocked local dry-run progression before any production mutation path"
        }
        .to_string(),
    })
}

fn build_production_mutation_signing_approval_artifact(
    opt: &LiveProductionMutationSigningApprovalOpt,
) -> anyhow::Result<ProductionMutationSigningApprovalArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    if opt.now_unix_ms == 0 {
        anyhow::bail!("production mutation signing approval now_unix_ms must be positive");
    }
    if opt.expires_at_unix_ms == 0 {
        anyhow::bail!("production mutation signing approval expires_at_unix_ms must be positive");
    }

    let approval_state = opt.approval_state.trim();
    if !matches!(
        approval_state,
        "pending" | "approved" | "expired" | "revoked"
    ) {
        anyhow::bail!("approval_state must be pending, approved, expired, or revoked");
    }
    let manual_approval_id = optional_non_empty("manual_approval_id", &opt.manual_approval_id)?;
    let approved_by = optional_non_empty("approved_by", &opt.approved_by)?;
    if approval_state != "pending" {
        if manual_approval_id.is_none() {
            anyhow::bail!("non-pending approval_state requires --manual-approval-id");
        }
        if approved_by.is_none() {
            anyhow::bail!("non-pending approval_state requires --approved-by");
        }
    }

    let request_preview = load_json_value(
        &opt.request_preview,
        "production live-alpha request preview",
    )?;
    let missing_cli_flags = missing_production_mutation_signing_approval_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_signing_approval_source_issues(&request_preview);
    let approval_expired = approval_state == "expired" || opt.now_unix_ms > opt.expires_at_unix_ms;
    let approval_revoked = approval_state == "revoked";
    let manual_approval_recorded =
        manual_approval_id.is_some() && approved_by.is_some() && approval_state != "pending";
    let owner_approved_signing_material = approval_state == "approved"
        && manual_approval_recorded
        && !approval_expired
        && !approval_revoked;
    let signing_approval_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && owner_approved_signing_material;
    let status = if signing_approval_ready {
        "ready_signing_material_approval"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() {
        "blocked_request_preview"
    } else if approval_state == "pending" {
        "approval_pending"
    } else if approval_expired {
        "approval_expired"
    } else if approval_revoked {
        "approval_revoked"
    } else {
        "approval_invalid"
    };

    Ok(ProductionMutationSigningApprovalArtifact {
        schema_version: PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_request_preview_path: opt.request_preview.display().to_string(),
        artifact_type: "production_mutation_signing_approval".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "owner_approved_env_only_signing_material".to_string(),
        capability: "Minimum Owner-Approved Production Order Mutation Candidate".to_string(),
        credential_material: json_string_value(&request_preview, "credential_material")
            .unwrap_or_else(|| "unknown".to_string()),
        approval_state: approval_state.to_string(),
        manual_approval_recorded,
        manual_approval_id,
        approved_by,
        now_unix_ms: opt.now_unix_ms,
        expires_at_unix_ms: opt.expires_at_unix_ms,
        approval_expired,
        approval_revoked,
        owner_approval_required: true,
        owner_approved_signing_material,
        signing_approval_ready,
        signing_material_scope: "production_live_alpha_env_only_memory_only".to_string(),
        production_signing_material_gate_required: json_bool_value(
            &request_preview,
            "production_signing_material_gate_required",
        )
        .unwrap_or(false),
        production_signing_material_gate_open: json_bool_value(
            &request_preview,
            "production_signing_material_gate_open",
        )
        .unwrap_or(false),
        production_signing_material_env_read: json_bool_value(
            &request_preview,
            "production_signing_material_env_read",
        )
        .unwrap_or(false),
        production_signing_material_missing_gate_env_vars: json_string_array(
            &request_preview,
            "production_signing_material_missing_gate_env_vars",
        ),
        api_key_env_name_recorded: true,
        api_secret_env_name_recorded: true,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        request_body_recorded: false,
        raw_request_body_recorded: false,
        request_preview_status: json_string_value(&request_preview, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        request_preview_built: json_bool_value(&request_preview, "request_preview_built")
            .unwrap_or(false),
        request_sent: json_bool_value(&request_preview, "request_sent").unwrap_or(true),
        symbol: json_string_value(&request_preview, "symbol")
            .unwrap_or_else(|| "unknown".to_string()),
        side: json_string_value(&request_preview, "side").unwrap_or_else(|| "unknown".to_string()),
        order_type: json_string_value(&request_preview, "order_type")
            .unwrap_or_else(|| "unknown".to_string()),
        quantity: json_string_value(&request_preview, "quantity")
            .unwrap_or_else(|| "unknown".to_string()),
        price: json_string_value(&request_preview, "price")
            .unwrap_or_else(|| "unknown".to_string()),
        time_in_force: json_string_value(&request_preview, "time_in_force")
            .unwrap_or_else(|| "unknown".to_string()),
        notional: json_string_value(&request_preview, "notional")
            .unwrap_or_else(|| "unknown".to_string()),
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        retry_attempted: false,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        execution_adapter_called: false,
        production_adapter_called: false,
        network_attempted: false,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        env_only_signing_material_confirmed: opt.confirm_env_only_signing_material,
        memory_only_signing_confirmed: opt.confirm_memory_only_signing,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        no_network_confirmed: opt.confirm_no_network,
        no_production_order_submission_confirmed: opt.confirm_no_production_order_submission,
        no_production_order_mutation_confirmed: opt.confirm_no_production_order_mutation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        diagnostic: if signing_approval_ready {
            "owner approved env-only production_live_alpha signing material for this v0.16 candidate; secrets and signatures remain unrecorded and no request was sent"
        } else {
            "production mutation signing material approval is blocked; no secret, signature, signed query, network, or production mutation was recorded"
        }
        .to_string(),
    })
}

fn build_production_mutation_request_builder_artifact(
    opt: &LiveProductionMutationRequestBuilderOpt,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> anyhow::Result<ProductionMutationRequestBuilderArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_positive_decimal_string("max_notional", &opt.max_notional)?;
    if opt.timestamp_ms == 0 {
        anyhow::bail!("production mutation request builder timestamp_ms must be positive");
    }
    if opt.recv_window_ms == 0 {
        anyhow::bail!("production mutation request builder recvWindow must be positive");
    }

    let runtime_gate = load_json_value(&opt.runtime_gate, "production mutation runtime gate")?;
    let signing_approval = load_json_value(
        &opt.signing_approval,
        "production mutation signing approval",
    )?;
    let request_preview = load_json_value(
        &opt.request_preview,
        "production live-alpha request preview",
    )?;

    let missing_cli_flags = missing_production_mutation_request_builder_cli_flags(opt);
    let missing_env_vars = production_mutation_request_builder_missing_env_vars(credentials);
    let source_artifact_issues = production_mutation_request_builder_source_issues(
        &runtime_gate,
        &signing_approval,
        &request_preview,
        &opt.max_notional,
        credentials,
    );

    let request_method = "POST".to_string();
    let request_target = json_string_value(&request_preview, "request_target")
        .unwrap_or_else(|| TESTNET_ORDER_ENDPOINT_ORDER.to_string());
    let symbol =
        json_string_value(&request_preview, "symbol").unwrap_or_else(|| "unknown".to_string());
    let side = json_string_value(&request_preview, "side").unwrap_or_else(|| "unknown".to_string());
    let order_type = json_string_value(&request_preview, "order_type")
        .unwrap_or_else(|| "unknown".to_string())
        .to_ascii_uppercase();
    let quantity =
        json_string_value(&request_preview, "quantity").unwrap_or_else(|| "unknown".to_string());
    let price =
        json_string_value(&request_preview, "price").unwrap_or_else(|| "unknown".to_string());
    let time_in_force = json_string_value(&request_preview, "time_in_force")
        .unwrap_or_else(|| "unknown".to_string())
        .to_ascii_uppercase();
    let notional =
        json_string_value(&request_preview, "notional").unwrap_or_else(|| "unknown".to_string());
    let tiny_notional_gate_ready = match (
        parse_non_negative_decimal(&notional),
        parse_non_negative_decimal(&opt.max_notional),
    ) {
        (Ok(order_notional), Ok(max_notional)) => {
            order_notional > Decimal::ZERO && order_notional <= max_notional
        }
        _ => false,
    };
    let single_order_candidate = request_method == "POST"
        && request_target == TESTNET_ORDER_ENDPOINT_ORDER
        && order_type == "LIMIT"
        && time_in_force == "GTC"
        && tiny_notional_gate_ready;

    let request_builder_ready = missing_cli_flags.is_empty()
        && missing_env_vars.is_empty()
        && source_artifact_issues.is_empty()
        && single_order_candidate;
    let mut request_object_built = false;
    if request_builder_ready {
        let request = build_production_live_alpha_signed_order_request_preview(
            &ProductionLiveAlphaOrderRequestInput {
                endpoint_path: &request_target,
                symbol: &symbol,
                side: &side,
                order_type: &order_type,
                quantity: &quantity,
                price: &price,
                time_in_force: &time_in_force,
                recv_window_ms: opt.recv_window_ms,
                timestamp_ms: opt.timestamp_ms,
            },
            credentials,
        )?;
        request.ensure_memory_only_redacted(credentials)?;
        request_object_built = true;
    }

    let status = if request_object_built {
        "ready_request_object_built_no_send"
    } else if !missing_cli_flags.is_empty() || !missing_env_vars.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() || !single_order_candidate {
        "blocked_source_artifact"
    } else {
        "blocked_request_builder"
    };

    Ok(ProductionMutationRequestBuilderArtifact {
        schema_version: PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_runtime_gate_path: opt.runtime_gate.display().to_string(),
        source_signing_approval_path: opt.signing_approval.display().to_string(),
        source_request_preview_path: opt.request_preview.display().to_string(),
        artifact_type: "production_mutation_request_builder".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_limit_gtc_request_object_no_send".to_string(),
        capability: "Minimum Owner-Approved Production Order Mutation Candidate".to_string(),
        request_builder_ready,
        request_object_built,
        runtime_gate_status: json_string_value(&runtime_gate, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        runtime_gate_open: json_bool_value(&runtime_gate, "runtime_gate_open").unwrap_or(true),
        send_consideration_allowed: json_bool_value(&runtime_gate, "send_consideration_allowed")
            .unwrap_or(true),
        signing_approval_status: json_string_value(&signing_approval, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        signing_approval_ready: json_bool_value(&signing_approval, "signing_approval_ready")
            .unwrap_or(false),
        explicit_send_gate_open: json_bool_value(&runtime_gate, "explicit_send_gate_open")
            .unwrap_or(true),
        credential_material: "production_live_alpha".to_string(),
        production_signing_material_gate_required: credentials
            .production_signing_material_gate_required,
        production_signing_material_gate_open: credentials.production_signing_material_gate_open,
        production_signing_material_env_read: credentials.production_signing_material_env_read,
        production_signing_material_missing_gate_env_vars: credentials
            .production_signing_material_missing_gate_env_vars
            .clone(),
        api_key_env: credentials.api_key_env.clone(),
        api_secret_env: credentials.api_secret_env.clone(),
        api_key_env_name_recorded: true,
        api_secret_env_name_recorded: true,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_name: BINANCE_API_KEY_HEADER.to_string(),
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        request_body_recorded: false,
        raw_request_body_recorded: false,
        request_method,
        request_target: request_target.clone(),
        endpoint_url_redacted: format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{request_target}"),
        query_shape_without_signature: production_live_alpha_order_query_shape_without_signature(),
        signed_query_shape: format!(
            "{}&signature=<redacted>",
            production_live_alpha_order_query_shape_without_signature()
        ),
        symbol,
        side,
        order_type,
        quantity,
        price,
        time_in_force,
        notional,
        recv_window_ms: opt.recv_window_ms,
        timestamp_recorded: false,
        timestamp_shape: "epoch_millis_present_redacted".to_string(),
        max_order_notional: opt.max_notional.trim().to_string(),
        single_order_candidate,
        tiny_notional_gate_ready,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        missing_env_vars,
        request_sent: false,
        network_attempted: false,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        execution_adapter_called: false,
        production_adapter_called: false,
        production_adapter_instantiated: false,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        env_only_signing_material_confirmed: true,
        memory_only_signing_confirmed: opt.confirm_memory_only_signing,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        no_network_confirmed: opt.confirm_no_network,
        no_production_order_submission_confirmed: opt.confirm_no_production_order_submission,
        no_production_order_mutation_confirmed: opt.confirm_no_production_order_mutation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        no_retry_confirmed: opt.confirm_no_retry,
        diagnostic: if request_object_built {
            "single LIMIT GTC production order request object was built in memory and redacted; explicit send gate remains closed"
        } else {
            "production mutation request builder is blocked before any send, network, adapter call, or production mutation"
        }
        .to_string(),
    })
}

fn build_production_mutation_guarded_send_artifact(
    opt: &LiveProductionMutationGuardedSendOpt,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> anyhow::Result<ProductionMutationGuardedSendArtifact> {
    build_production_mutation_guarded_send_artifact_with_executor(
        opt,
        credentials,
        execute_production_mutation_guarded_send,
    )
}

fn build_production_mutation_guarded_send_artifact_with_executor<H>(
    opt: &LiveProductionMutationGuardedSendOpt,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
    mut http_executor: H,
) -> anyhow::Result<ProductionMutationGuardedSendArtifact>
where
    H: FnMut(
        &ProductionLiveAlphaSignedOrderRequestPreview,
    ) -> ProductionMutationGuardedSendHttpResult,
{
    validate_non_empty("run_id", &opt.run_id)?;
    validate_positive_decimal_string("max_notional", &opt.max_notional)?;
    if opt.timestamp_ms == 0 {
        anyhow::bail!("production mutation guarded send timestamp_ms must be positive");
    }
    if opt.recv_window_ms == 0 {
        anyhow::bail!("production mutation guarded send recvWindow must be positive");
    }

    let request_builder =
        load_json_value(&opt.request_builder, "production mutation request builder")?;
    let pre_send_kill_switch_runtime_gate = load_json_value(
        &opt.kill_switch_runtime_gate,
        "production live-alpha kill-switch runtime gate",
    )?;
    let request_preview = load_json_value(
        &opt.request_preview,
        "production live-alpha request preview",
    )?;
    let missing_cli_flags = missing_production_mutation_guarded_send_cli_flags(opt);
    let missing_env_vars = production_mutation_guarded_send_missing_env_vars(opt, credentials);
    let mut source_artifact_issues = production_mutation_guarded_send_source_issues(
        &request_builder,
        &pre_send_kill_switch_runtime_gate,
        &request_preview,
        &opt.max_notional,
        credentials,
    );
    let pre_send_kill_switch = production_mutation_guarded_send_kill_switch_snapshot(
        &opt.kill_switch_runtime_gate,
        &pre_send_kill_switch_runtime_gate,
    );
    let pre_send_kill_switch_clean = pre_send_kill_switch.checked
        && pre_send_kill_switch.runtime_gate_open
        && !pre_send_kill_switch.kill_switch_active;
    if !pre_send_kill_switch_clean
        && !source_artifact_issues
            .iter()
            .any(|issue| issue.starts_with("kill_switch_"))
    {
        source_artifact_issues.push("kill_switch_enforcement_not_ready".to_string());
    }

    let request_method = "POST".to_string();
    let request_target = json_string_value(&request_preview, "request_target")
        .unwrap_or_else(|| TESTNET_ORDER_ENDPOINT_ORDER.to_string());
    let symbol =
        json_string_value(&request_preview, "symbol").unwrap_or_else(|| "unknown".to_string());
    let side = json_string_value(&request_preview, "side").unwrap_or_else(|| "unknown".to_string());
    let order_type = json_string_value(&request_preview, "order_type")
        .unwrap_or_else(|| "unknown".to_string())
        .to_ascii_uppercase();
    let quantity =
        json_string_value(&request_preview, "quantity").unwrap_or_else(|| "unknown".to_string());
    let price =
        json_string_value(&request_preview, "price").unwrap_or_else(|| "unknown".to_string());
    let time_in_force = json_string_value(&request_preview, "time_in_force")
        .unwrap_or_else(|| "unknown".to_string())
        .to_ascii_uppercase();
    let notional =
        json_string_value(&request_preview, "notional").unwrap_or_else(|| "unknown".to_string());

    let guarded_send_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && pre_send_kill_switch_clean
        && (!opt.manual_online || missing_env_vars.is_empty());
    let single_shot_send_allowed = guarded_send_ready && opt.manual_online;
    let http_result = if single_shot_send_allowed {
        let request = build_production_live_alpha_signed_order_request_preview(
            &ProductionLiveAlphaOrderRequestInput {
                endpoint_path: &request_target,
                symbol: &symbol,
                side: &side,
                order_type: &order_type,
                quantity: &quantity,
                price: &price,
                time_in_force: &time_in_force,
                recv_window_ms: opt.recv_window_ms,
                timestamp_ms: opt.timestamp_ms,
            },
            credentials,
        )?;
        request.ensure_memory_only_redacted(credentials)?;
        Some(http_executor(&request))
    } else {
        None
    };
    let post_send_kill_switch_runtime_gate = load_json_value(
        &opt.kill_switch_runtime_gate,
        "production live-alpha kill-switch runtime gate post-send",
    )?;
    let post_send_kill_switch = production_mutation_guarded_send_kill_switch_snapshot(
        &opt.kill_switch_runtime_gate,
        &post_send_kill_switch_runtime_gate,
    );
    let post_send_kill_switch_clean = post_send_kill_switch.checked
        && post_send_kill_switch.runtime_gate_open
        && !post_send_kill_switch.kill_switch_active;
    let kill_switch_enforcement_ready = pre_send_kill_switch_clean && post_send_kill_switch_clean;
    if !post_send_kill_switch_clean {
        source_artifact_issues.push("post_send_kill_switch_not_clean".to_string());
    }
    source_artifact_issues.sort();
    source_artifact_issues.dedup();
    let post_send_progression_blocked = !post_send_kill_switch_clean;
    let manual_review_required = post_send_progression_blocked;
    let new_orders_blocked = post_send_progression_blocked;
    let counters = production_mutation_guarded_send_counters(http_result.as_ref());
    let status = if counters.request_sent {
        "manual_online_send_attempt_recorded"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if opt.manual_online && !missing_env_vars.is_empty() {
        "blocked_missing_manual_online_gate"
    } else if !kill_switch_enforcement_ready {
        "blocked_kill_switch_enforcement"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
    } else {
        "ready_guarded_send_path_offline_no_network"
    };

    Ok(ProductionMutationGuardedSendArtifact {
        schema_version: PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_request_builder_path: opt.request_builder.display().to_string(),
        source_kill_switch_runtime_gate_path: opt.kill_switch_runtime_gate.display().to_string(),
        source_request_preview_path: opt.request_preview.display().to_string(),
        artifact_type: "production_mutation_guarded_send".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_shot_guarded_http_send".to_string(),
        capability: "Minimum Owner-Approved Production Order Mutation Candidate".to_string(),
        manual_online_requested: opt.manual_online,
        guarded_send_ready,
        send_path_evaluated: true,
        kill_switch_enforcement_ready,
        kill_switch_checked_before_send: pre_send_kill_switch.checked,
        kill_switch_checked_after_send: post_send_kill_switch.checked,
        pre_send_kill_switch_snapshot_source: pre_send_kill_switch.source_path,
        pre_send_kill_switch_snapshot_hash: pre_send_kill_switch.source_hash,
        pre_send_kill_switch_checked_at: pre_send_kill_switch.checked_at,
        pre_send_kill_switch_runtime_gate_open: pre_send_kill_switch.runtime_gate_open,
        pre_send_kill_switch_active: pre_send_kill_switch.kill_switch_active,
        post_send_kill_switch_snapshot_source: post_send_kill_switch.source_path,
        post_send_kill_switch_snapshot_hash: post_send_kill_switch.source_hash,
        post_send_kill_switch_checked_at: post_send_kill_switch.checked_at,
        post_send_kill_switch_runtime_gate_open: post_send_kill_switch.runtime_gate_open,
        post_send_kill_switch_active: post_send_kill_switch.kill_switch_active,
        post_send_kill_switch_clean,
        kill_switch_blocked_send: !pre_send_kill_switch_clean,
        post_send_progression_blocked,
        manual_review_required,
        new_orders_blocked,
        single_shot_send_allowed,
        request_builder_status: json_string_value(&request_builder, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        request_object_built: json_bool_value(&request_builder, "request_object_built")
            .unwrap_or(false),
        request_method,
        request_target: request_target.clone(),
        endpoint_url_redacted: format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{request_target}"),
        credential_material: "production_live_alpha".to_string(),
        production_signing_material_gate_required: credentials
            .production_signing_material_gate_required,
        production_signing_material_gate_open: credentials.production_signing_material_gate_open,
        production_signing_material_env_read: credentials.production_signing_material_env_read,
        production_signing_material_missing_gate_env_vars: credentials
            .production_signing_material_missing_gate_env_vars
            .clone(),
        api_key_env: credentials.api_key_env.clone(),
        api_secret_env: credentials.api_secret_env.clone(),
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        request_body_recorded: false,
        raw_request_body_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_redacted: true,
        http_status_code: http_result.as_ref().and_then(|result| result.status_code),
        latency_ms: http_result.as_ref().and_then(|result| result.latency_ms),
        error_code: http_result.as_ref().map_or_else(
            || "not_attempted_offline".to_string(),
            |result| result.error_code.clone(),
        ),
        symbol,
        side,
        order_type,
        quantity,
        price,
        time_in_force,
        notional,
        max_order_notional: opt.max_notional.trim().to_string(),
        recv_window_ms: opt.recv_window_ms,
        timestamp_recorded: false,
        timestamp_shape: "epoch_millis_present_redacted".to_string(),
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        missing_env_vars,
        request_sent: counters.request_sent,
        network_attempted: counters.network_attempted,
        production_order_request_attempted: counters.production_order_request_attempted,
        http_send_attempted: counters.http_send_attempted,
        exchange_ack_observed: counters.exchange_ack_observed,
        exchange_order_id_observed: counters.exchange_order_id_observed,
        exchange_order_status_observed: counters.exchange_order_status_observed,
        confirmed_production_order_submission: counters.confirmed_production_order_submission,
        production_order_submission_allowed: single_shot_send_allowed,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: counters.production_order_submissions_attempted,
        production_orders_submitted: counters.production_orders_submitted,
        production_order_mutations_attempted: counters.production_order_mutations_attempted,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: counters.real_orders_submitted,
        real_funds: false,
        platform_production_trading_enabled: counters.platform_production_trading_enabled,
        production_trading_enabled: counters.production_trading_enabled,
        single_shot_confirmed: opt.confirm_single_shot,
        no_retry_confirmed: opt.confirm_no_retry,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        response_redaction_confirmed: opt.confirm_response_redacted,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        diagnostic: if counters.confirmed_production_order_submission {
            "manual owner online guarded send path observed exchange acknowledgement for one production HTTP request; raw response, signed URL, signed query, signature, and secrets were not persisted"
        } else if counters.request_sent {
            "manual owner online guarded send path attempted one production HTTP request without exchange acknowledgement; raw response, signed URL, signed query, signature, and secrets were not persisted"
        } else {
            "guarded production HTTP send path stayed offline; no request, network, retry, cancel, replace, amend, flatten, or Dashboard order control was attempted"
        }
        .to_string(),
    })
}

fn build_production_mutation_response_redaction_artifact(
    opt: &LiveProductionMutationResponseRedactionOpt,
) -> anyhow::Result<ProductionMutationResponseRedactionArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let guarded_send = load_json_value(&opt.guarded_send, "production mutation guarded send")?;
    let response = load_json_value(&opt.response, "production mutation response")?;
    let missing_cli_flags = missing_production_mutation_response_redaction_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_response_redaction_source_issues(&guarded_send);
    let forbidden_response_markers = production_mutation_response_forbidden_markers(&response);
    let response_redaction_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && forbidden_response_markers.is_empty();
    let status = if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
    } else if !forbidden_response_markers.is_empty() {
        "blocked_forbidden_response_marker"
    } else {
        "ready_response_redacted"
    };

    let source_request_sent = json_bool_value(&guarded_send, "request_sent").unwrap_or(false);
    let source_network_attempted =
        json_bool_value(&guarded_send, "network_attempted").unwrap_or(false);
    let production_order_submissions_attempted =
        json_u64_value(&guarded_send, "production_order_submissions_attempted").unwrap_or(0);
    let production_orders_submitted =
        json_u64_value(&guarded_send, "production_orders_submitted").unwrap_or(0);
    let production_order_mutations_attempted =
        json_u64_value(&guarded_send, "production_order_mutations_attempted").unwrap_or(0);
    let production_order_state_reads_attempted =
        json_u64_value(&guarded_send, "production_order_state_reads_attempted").unwrap_or(0);
    let listen_key_lifecycle_attempted =
        json_u64_value(&guarded_send, "listen_key_lifecycle_attempted").unwrap_or(0);

    Ok(ProductionMutationResponseRedactionArtifact {
        schema_version: PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_guarded_send_path: opt.guarded_send.display().to_string(),
        source_response_path: opt.response.display().to_string(),
        artifact_type: "production_mutation_response_redaction".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "production_mutation_response_redaction_contract".to_string(),
        capability: "Minimum Owner-Approved Production Order Mutation Candidate".to_string(),
        response_redaction_ready,
        source_guarded_send_status: json_string_value(&guarded_send, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        source_request_sent,
        source_network_attempted,
        response_shape_validated: response_redaction_ready,
        response_type: "binance_order_response_redacted_metadata_v1".to_string(),
        symbol: json_scalar_string_value(&response, "symbol").unwrap_or_else(|| "unknown".to_string()),
        side: json_scalar_string_value(&response, "side").unwrap_or_else(|| "unknown".to_string()),
        order_type: json_scalar_string_value(&response, "type").unwrap_or_else(|| "unknown".to_string()),
        time_in_force: json_scalar_string_value(&response, "timeInForce")
            .unwrap_or_else(|| "unknown".to_string()),
        order_id: json_scalar_string_value(&response, "orderId")
            .unwrap_or_else(|| "missing".to_string()),
        client_order_id: json_scalar_string_value(&response, "clientOrderId")
            .unwrap_or_else(|| "missing".to_string()),
        exchange_status: json_scalar_string_value(&response, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        transact_time_shape: production_mutation_response_time_shape(&response, "transactTime"),
        working_time_shape: production_mutation_response_time_shape(&response, "workingTime"),
        allowed_response_fields: production_mutation_response_allowed_fields(),
        forbidden_response_markers,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        request_body_recorded: false,
        raw_request_body_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        unrestricted_payload_recorded: false,
        account_balances_recorded: false,
        fills_recorded: false,
        response_redacted: true,
        request_sent: source_request_sent,
        network_attempted: source_network_attempted,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted,
        production_orders_submitted,
        production_order_mutations_attempted,
        production_order_state_reads_attempted,
        listen_key_lifecycle_attempted,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: json_bool_value(&guarded_send, "production_trading_enabled")
            .unwrap_or(false),
        no_raw_response_persistence_confirmed: opt.confirm_no_raw_response_persistence,
        no_headers_persistence_confirmed: opt.confirm_no_headers_persistence,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        order_metadata_only_confirmed: opt.confirm_order_metadata_only,
        no_account_balances_confirmed: opt.confirm_no_account_balances,
        no_unrestricted_payload_confirmed: opt.confirm_no_unrestricted_payload,
        no_retry_confirmed: opt.confirm_no_retry,
        diagnostic: if response_redaction_ready {
            "production mutation response was reduced to allowed order metadata only; raw body, headers, secrets, balances, fills, and unrestricted payload were not persisted"
        } else {
            "production mutation response redaction contract blocked before persisting unrestricted response material"
        }
        .to_string(),
    })
}

fn build_production_mutation_order_state_readback_artifact<F, H>(
    opt: &LiveProductionMutationOrderStateReadbackOpt,
    read_env: &mut F,
    mut http_probe: H,
) -> anyhow::Result<ProductionMutationOrderStateReadbackArtifact>
where
    F: FnMut(&str) -> Option<String>,
    H: FnMut(
        &LiveProductionOrderStateReadOnlyProofOpt,
        &EnvOnlyProductionReadCredentials,
        u64,
    ) -> ProductionOrderStateHttpResult,
{
    validate_non_empty("run_id", &opt.run_id)?;
    if opt.recv_window_ms == 0 {
        anyhow::bail!("production mutation order-state readback recvWindow must be positive");
    }

    let response_redaction = load_json_value(
        &opt.response_redaction,
        "production mutation response redaction",
    )?;
    let missing_cli_flags = missing_production_mutation_order_state_readback_cli_flags(opt);
    let missing_env_vars = production_mutation_order_state_readback_missing_env_vars(opt, read_env);
    let credentials = EnvOnlyProductionReadCredentials::from_values(
        opt.api_key_env.clone(),
        read_env(&opt.api_key_env),
        opt.api_secret_env.clone(),
        read_env(&opt.api_secret_env),
    );
    let source_artifact_issues =
        production_mutation_order_state_readback_source_issues(&response_redaction);

    let symbol = json_scalar_string_value(&response_redaction, "symbol")
        .unwrap_or_else(|| "unknown".to_string());
    let order_id_raw = json_scalar_string_value(&response_redaction, "order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id = json_scalar_string_value(&response_redaction, "client_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let order_id = order_id_raw.parse::<u64>().ok();
    let orig_client_order_id =
        (known_client_order_id != "missing").then(|| known_client_order_id.clone());
    let order_state_opt = LiveProductionOrderStateReadOnlyProofOpt {
        endpoint: ProductionOrderStateReadEndpoint::Order,
        symbol: symbol.clone(),
        order_id,
        orig_client_order_id,
        output: None,
        manual_online: opt.manual_online,
        api_key_env: opt.api_key_env.clone(),
        api_secret_env: opt.api_secret_env.clone(),
        recv_window_ms: opt.recv_window_ms,
        allow_production_order_state_read: opt.allow_production_mutation_order_state_readback,
        confirm_owner_approved_read_only: opt.confirm_owner_approved_order_state_readback,
        confirm_no_order_mutation: opt.confirm_no_production_order_mutation,
        confirm_no_secret_persistence: opt.confirm_no_secret_persistence,
        confirm_no_listen_key_lifecycle: opt.confirm_no_listen_key_lifecycle,
        confirm_dashboard_order_controls_disabled: opt.confirm_dashboard_order_controls_disabled,
    };
    let credentials_missing = !credentials.api_key_present() || !credentials.api_secret_present();
    let should_attempt_online = opt.manual_online
        && missing_cli_flags.is_empty()
        && missing_env_vars.is_empty()
        && source_artifact_issues.is_empty()
        && !credentials_missing
        && (order_state_opt.order_id.is_some() || order_state_opt.orig_client_order_id.is_some());
    let http_result = should_attempt_online.then(|| {
        http_probe(
            &order_state_opt,
            &credentials,
            order_state_opt.recv_window_ms,
        )
    });
    let response_shape_summary = http_result.as_ref().map_or_else(
        || ProductionOrderStateShapeSummary::not_attempted(ProductionOrderStateReadEndpoint::Order),
        |result| result.response_shape_summary.clone(),
    );
    let order_state_read_attempted = http_result
        .as_ref()
        .is_some_and(|result| result.network_attempted);
    let status = if let Some(result) = &http_result {
        result.status.as_str()
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
    } else if opt.manual_online && !missing_env_vars.is_empty() {
        "blocked_missing_manual_online_gate"
    } else if opt.manual_online && credentials_missing {
        "blocked_missing_credentials"
    } else {
        "ready_offline_order_state_readback_contract"
    };
    let readback_contract_ready = matches!(
        status,
        "ready_offline_order_state_readback_contract" | "online_order_state_read_ok"
    );
    let order_state_read_allowed = should_attempt_online;

    Ok(ProductionMutationOrderStateReadbackArtifact {
        schema_version: PRODUCTION_MUTATION_ORDER_STATE_READBACK_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_response_redaction_path: opt.response_redaction.display().to_string(),
        artifact_type: "production_mutation_order_state_readback".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "post_submit_order_state_readback_proof".to_string(),
        capability: "Minimum Owner-Approved Production Order Mutation Candidate".to_string(),
        readback_contract_ready,
        source_response_redaction_status: json_string_value(&response_redaction, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        known_order_identifier_source: "production_mutation_response_redaction".to_string(),
        known_order_id: order_id_raw,
        known_client_order_id,
        symbol,
        endpoint: "order".to_string(),
        method: "GET".to_string(),
        path: production_order_state_endpoint_path(ProductionOrderStateReadEndpoint::Order)
            .to_string(),
        request_url_redacted: production_order_state_redacted_request_url(&order_state_opt),
        query_shape: production_order_state_query_shape(&order_state_opt),
        manual_online_requested: opt.manual_online,
        order_state_read_allowed,
        order_state_read_attempted,
        response_shape: http_result.as_ref().map_or_else(
            || production_order_state_response_shape(ProductionOrderStateReadEndpoint::Order)
                .to_string(),
            |result| result.response_shape.clone(),
        ),
        response_shape_validated: http_result
            .as_ref()
            .is_some_and(|result| result.response_shape_validated),
        endpoint_shape_validated: response_shape_summary.endpoint_shape_validated,
        order_entries_observed: response_shape_summary.order_entries_observed,
        non_empty_order_state_observed: response_shape_summary.non_empty_order_state_observed,
        order_lifecycle_readiness: response_shape_summary.order_lifecycle_readiness,
        strategy_success_inferred: false,
        strategy_success_proof: "not_inferred_readback_is_observability_only".to_string(),
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        missing_env_vars: missing_env_vars
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        api_key_env: credentials.api_key_env.clone(),
        api_secret_env: credentials.api_secret_env.clone(),
        api_key_present: credentials.api_key_present(),
        api_secret_present: credentials.api_secret_present(),
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        response_redacted: true,
        network_attempted: order_state_read_attempted,
        latency_ms: http_result.as_ref().and_then(|result| result.latency_ms),
        response_status_code: http_result.as_ref().and_then(|result| result.http_status),
        error_code: http_result.as_ref().map_or_else(
            || "not_attempted".to_string(),
            |result| result.error_code.clone(),
        ),
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: order_state_read_allowed,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: u64::from(order_state_read_attempted),
        listen_key_lifecycle_attempted: 0,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        known_order_identifier_only_confirmed: opt.confirm_known_order_identifier_only,
        read_only_get_order_confirmed: opt.confirm_read_only_get_order,
        no_production_order_mutation_confirmed: opt.confirm_no_production_order_mutation,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        no_retry_confirmed: opt.confirm_no_retry,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        diagnostic: if order_state_read_attempted {
            "owner-gated readback attempted GET /api/v3/order using only known order identifiers; response remains redacted and strategy success is not inferred"
        } else {
            "offline readback contract prepared GET /api/v3/order from known order identifiers only; no network, mutation, retry, or strategy-success inference occurred"
        }
        .to_string(),
    })
}

fn build_production_mutation_audit_trail_artifact(
    opt: &LiveProductionMutationAuditTrailOpt,
) -> anyhow::Result<ProductionMutationAuditTrailArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let request_builder =
        load_json_value(&opt.request_builder, "production mutation request builder")?;
    let guarded_send = load_json_value(&opt.guarded_send, "production mutation guarded send")?;
    let response_redaction = load_json_value(
        &opt.response_redaction,
        "production mutation response redaction",
    )?;
    let order_state_readback = load_json_value(
        &opt.order_state_readback,
        "production mutation order-state readback",
    )?;

    let source_runtime_gate_path = json_string_value(&request_builder, "source_runtime_gate_path")
        .unwrap_or_else(|| "missing".to_string());
    let source_signing_approval_path =
        json_string_value(&request_builder, "source_signing_approval_path")
            .unwrap_or_else(|| "missing".to_string());
    let source_request_preview_path =
        json_string_value(&request_builder, "source_request_preview_path")
            .unwrap_or_else(|| "missing".to_string());
    let source_kill_switch_runtime_gate_path =
        json_string_value(&guarded_send, "source_kill_switch_runtime_gate_path")
            .unwrap_or_else(|| "missing".to_string());

    let runtime_gate = load_json_value(
        Path::new(&source_runtime_gate_path),
        "production mutation runtime gate",
    )?;
    let signing_approval = load_json_value(
        Path::new(&source_signing_approval_path),
        "production mutation signing approval",
    )?;

    let missing_cli_flags = missing_production_mutation_audit_trail_cli_flags(opt);
    let mut source_artifact_issues = production_mutation_audit_trail_source_issues(
        &request_builder,
        &guarded_send,
        &response_redaction,
        &order_state_readback,
        &runtime_gate,
        &signing_approval,
    );
    let preview_hash = file_fnv1a64_hash(&source_request_preview_path);
    if preview_hash == "unavailable" {
        source_artifact_issues.push("request_preview_hash_unavailable".to_string());
    }
    let audit_trail_ready = missing_cli_flags.is_empty() && source_artifact_issues.is_empty();
    let failure_state = production_mutation_audit_failure_state(
        &missing_cli_flags,
        &source_artifact_issues,
        &guarded_send,
        &order_state_readback,
    );

    Ok(ProductionMutationAuditTrailArtifact {
        schema_version: PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_request_builder_path: opt.request_builder.display().to_string(),
        source_guarded_send_path: opt.guarded_send.display().to_string(),
        source_response_redaction_path: opt.response_redaction.display().to_string(),
        source_order_state_readback_path: opt.order_state_readback.display().to_string(),
        source_runtime_gate_path,
        source_signing_approval_path,
        source_request_preview_path,
        source_kill_switch_runtime_gate_path,
        artifact_type: "production_mutation_audit_trail".to_string(),
        status: if audit_trail_ready {
            "ready_redacted_audit_trail"
        } else if !missing_cli_flags.is_empty() {
            "blocked_missing_gate"
        } else {
            "blocked_source_artifact"
        }
        .to_string(),
        created_at: now_millis(),
        mode: "redacted_production_mutation_audit_trail".to_string(),
        capability: "Minimum Owner-Approved Production Order Mutation Candidate".to_string(),
        audit_trail_ready,
        preview_hash,
        signing_approval_status: json_string_value(&signing_approval, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        approval_state: json_string_value(&signing_approval, "approval_state")
            .unwrap_or_else(|| "unknown".to_string()),
        manual_approval_recorded: json_bool_value(&signing_approval, "manual_approval_recorded")
            .unwrap_or(false),
        manual_approval_id: json_string_value(&signing_approval, "manual_approval_id"),
        approved_by: json_string_value(&signing_approval, "approved_by"),
        runtime_gate_status: json_string_value(&runtime_gate, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        runtime_gate_open: json_bool_value(&runtime_gate, "runtime_gate_open").unwrap_or(false),
        send_consideration_allowed: json_bool_value(&runtime_gate, "send_consideration_allowed")
            .unwrap_or(false),
        guarded_send_status: json_string_value(&guarded_send, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        request_sent: json_bool_value(&guarded_send, "request_sent").unwrap_or(false),
        network_attempted: json_bool_value(&guarded_send, "network_attempted").unwrap_or(false),
        response_redaction_status: json_string_value(&response_redaction, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        response_redaction_ready: json_bool_value(&response_redaction, "response_redaction_ready")
            .unwrap_or(false),
        order_state_readback_status: json_string_value(&order_state_readback, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        readback_contract_ready: json_bool_value(&order_state_readback, "readback_contract_ready")
            .unwrap_or(false),
        order_state_read_attempted: json_bool_value(
            &order_state_readback,
            "order_state_read_attempted",
        )
        .unwrap_or(false),
        kill_switch_checked_before_send: json_bool_value(
            &guarded_send,
            "kill_switch_checked_before_send",
        )
        .unwrap_or(false),
        kill_switch_checked_after_send: json_bool_value(
            &guarded_send,
            "kill_switch_checked_after_send",
        )
        .unwrap_or(false),
        pre_send_kill_switch_runtime_gate_open: json_bool_value(
            &guarded_send,
            "pre_send_kill_switch_runtime_gate_open",
        )
        .unwrap_or(false),
        pre_send_kill_switch_active: json_bool_value(&guarded_send, "pre_send_kill_switch_active")
            .unwrap_or(true),
        post_send_kill_switch_runtime_gate_open: json_bool_value(
            &guarded_send,
            "post_send_kill_switch_runtime_gate_open",
        )
        .unwrap_or(false),
        post_send_kill_switch_active: json_bool_value(
            &guarded_send,
            "post_send_kill_switch_active",
        )
        .unwrap_or(true),
        kill_switch_blocked_send: json_bool_value(&guarded_send, "kill_switch_blocked_send")
            .unwrap_or(true),
        symbol: json_string_value(&request_builder, "symbol").unwrap_or_else(|| {
            json_string_value(&response_redaction, "symbol").unwrap_or_else(|| "unknown".to_string())
        }),
        side: json_string_value(&request_builder, "side").unwrap_or_else(|| {
            json_string_value(&response_redaction, "side").unwrap_or_else(|| "unknown".to_string())
        }),
        order_type: json_string_value(&request_builder, "order_type")
            .unwrap_or_else(|| "unknown".to_string()),
        time_in_force: json_string_value(&request_builder, "time_in_force")
            .unwrap_or_else(|| "unknown".to_string()),
        quantity: json_string_value(&request_builder, "quantity")
            .unwrap_or_else(|| "unknown".to_string()),
        price: json_string_value(&request_builder, "price").unwrap_or_else(|| "unknown".to_string()),
        notional: json_string_value(&request_builder, "notional")
            .unwrap_or_else(|| "unknown".to_string()),
        order_id: json_string_value(&response_redaction, "order_id")
            .unwrap_or_else(|| "missing".to_string()),
        client_order_id: json_string_value(&response_redaction, "client_order_id")
            .unwrap_or_else(|| "missing".to_string()),
        exchange_status: json_string_value(&response_redaction, "exchange_status")
            .unwrap_or_else(|| "unknown".to_string()),
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        failure_state,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        request_body_recorded: false,
        raw_request_body_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        unrestricted_payload_recorded: false,
        account_balances_recorded: false,
        response_redacted: true,
        production_order_submission_allowed: json_bool_value(
            &guarded_send,
            "production_order_submission_allowed",
        )
        .unwrap_or(false),
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: json_bool_value(
            &order_state_readback,
            "production_order_state_reads_allowed",
        )
        .unwrap_or(false),
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: json_u64_value(
            &guarded_send,
            "production_order_submissions_attempted",
        )
        .unwrap_or(0),
        production_orders_submitted: json_u64_value(&guarded_send, "production_orders_submitted")
            .unwrap_or(0),
        production_order_mutations_attempted: json_u64_value(
            &guarded_send,
            "production_order_mutations_attempted",
        )
        .unwrap_or(0),
        production_order_state_reads_attempted: json_u64_value(
            &order_state_readback,
            "production_order_state_reads_attempted",
        )
        .unwrap_or(0),
        listen_key_lifecycle_attempted: 0,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: json_bool_value(&guarded_send, "real_orders_submitted")
            .unwrap_or(false),
        real_funds: false,
        production_trading_enabled: json_bool_value(&guarded_send, "production_trading_enabled")
            .unwrap_or(false),
        redacted_artifacts_only_confirmed: opt.confirm_redacted_artifacts_only,
        no_secret_or_raw_payload_persistence_confirmed: opt
            .confirm_no_secret_or_raw_payload_persistence,
        no_retry_or_followup_mutation_confirmed: opt.confirm_no_retry_or_followup_mutation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        diagnostic: if audit_trail_ready {
            "redacted audit trail links approval, runtime gate, request preview hash, guarded send, response redaction, readback, kill switch, and failure state without raw secrets or payloads"
        } else {
            "production mutation audit trail is blocked because required confirmations or source artifact evidence are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_failure_semantics_artifact(
    opt: &LiveProductionMutationFailureSemanticsOpt,
) -> anyhow::Result<ProductionMutationFailureSemanticsArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let audit_trail = load_json_value(&opt.audit_trail, "production mutation audit trail")?;
    let missing_cli_flags = missing_production_mutation_failure_semantics_cli_flags(opt);
    let source_artifact_issues = production_mutation_failure_semantics_source_issues(&audit_trail);
    let failure_semantics_ready = missing_cli_flags.is_empty() && source_artifact_issues.is_empty();
    let failure_mode = opt.failure_mode.as_str();
    let failure_category = opt.failure_mode.category();
    let failure_state = if failure_semantics_ready {
        opt.failure_mode.failure_state()
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else {
        "blocked_source_artifact"
    };

    Ok(ProductionMutationFailureSemanticsArtifact {
        schema_version: PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_audit_trail_path: opt.audit_trail.display().to_string(),
        artifact_type: "production_mutation_failure_semantics".to_string(),
        status: if failure_semantics_ready {
            "ready_failure_semantics_evidence"
        } else if !missing_cli_flags.is_empty() {
            "blocked_missing_gate"
        } else {
            "blocked_source_artifact"
        }
        .to_string(),
        created_at: now_millis(),
        mode: "evidence_only_failure_no_retry_semantics".to_string(),
        capability: "Minimum Owner-Approved Production Order Mutation Candidate".to_string(),
        failure_semantics_ready,
        failure_mode: failure_mode.to_string(),
        failure_category: failure_category.to_string(),
        failure_state: failure_state.to_string(),
        source_audit_trail_status: json_string_value(&audit_trail, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        source_audit_trail_ready: json_bool_value(&audit_trail, "audit_trail_ready")
            .unwrap_or(false),
        source_failure_state: json_string_value(&audit_trail, "failure_state")
            .unwrap_or_else(|| "unknown".to_string()),
        terminal_action: "write_evidence_and_stop".to_string(),
        evidence_written: true,
        stop_after_evidence: true,
        strategy_continuation_allowed: false,
        request_sent: json_bool_value(&audit_trail, "request_sent").unwrap_or(false),
        network_attempted: json_bool_value(&audit_trail, "network_attempted").unwrap_or(false),
        order_state_read_attempted: json_bool_value(&audit_trail, "order_state_read_attempted")
            .unwrap_or(false),
        kill_switch_checked_before_send: json_bool_value(
            &audit_trail,
            "kill_switch_checked_before_send",
        )
        .unwrap_or(false),
        kill_switch_checked_after_send: json_bool_value(
            &audit_trail,
            "kill_switch_checked_after_send",
        )
        .unwrap_or(false),
        kill_switch_blocked_send: json_bool_value(&audit_trail, "kill_switch_blocked_send")
            .unwrap_or(false),
        retry_allowed: false,
        retry_attempted: false,
        retry_attempts: 0,
        max_retry_attempts: 0,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        correction_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_remediation_allowed: false,
        dashboard_order_controls_enabled: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: json_bool_value(
            &audit_trail,
            "production_order_state_reads_allowed",
        )
        .unwrap_or(false),
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: json_u64_value(
            &audit_trail,
            "production_order_submissions_attempted",
        )
        .unwrap_or(0),
        production_orders_submitted: json_u64_value(&audit_trail, "production_orders_submitted")
            .unwrap_or(0),
        production_order_mutations_attempted: json_u64_value(
            &audit_trail,
            "production_order_mutations_attempted",
        )
        .unwrap_or(0),
        production_order_state_reads_attempted: json_u64_value(
            &audit_trail,
            "production_order_state_reads_attempted",
        )
        .unwrap_or(0),
        listen_key_lifecycle_attempted: json_u64_value(
            &audit_trail,
            "listen_key_lifecycle_attempted",
        )
        .unwrap_or(0),
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        evidence_only_failure_handling_confirmed: opt.confirm_evidence_only_failure_handling,
        no_retry_confirmed: opt.confirm_no_retry,
        no_automatic_cancel_replace_amend_confirmed: opt
            .confirm_no_automatic_cancel_replace_amend,
        no_correction_or_flatten_confirmed: opt.confirm_no_correction_or_flatten,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_strategy_continuation_confirmed: opt.confirm_no_strategy_continuation,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        diagnostic: if failure_semantics_ready {
            "failure mode writes redacted evidence and stops without retry, cancel, replace, amend, correction, flatten, Dashboard controls, listenKey lifecycle, or strategy continuation"
        } else {
            "production mutation failure semantics are blocked because required confirmations or source audit evidence are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_runtime_gate_artifact(
    opt: &LiveProductionMutationRuntimeGateOpt,
) -> anyhow::Result<ProductionMutationRuntimeGateArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_positive_decimal_string("max_notional", &opt.max_notional)?;

    let order_gate = load_json_value(&opt.order_gate, "live-alpha dry-run order gate")?;
    let risk_preflight = load_json_value(&opt.risk_preflight, "live-alpha risk preflight")?;
    let request_preview = load_json_value(&opt.request_preview, "live-alpha request preview")?;
    let kill_switch_runtime_gate = load_json_value(
        &opt.kill_switch_runtime_gate,
        "live-alpha kill-switch runtime gate",
    )?;
    let signing_approval = opt
        .signing_approval
        .as_deref()
        .map(|path| load_json_value(path, "production mutation signing approval"))
        .transpose()?;

    let missing_cli_flags = missing_production_mutation_runtime_gate_cli_flags(opt);
    let source_artifact_issues = production_mutation_runtime_gate_source_issues(
        &order_gate,
        &risk_preflight,
        &request_preview,
        &kill_switch_runtime_gate,
        signing_approval.as_ref(),
        &opt.max_notional,
    );
    let source_artifact_has_non_signing_issue = source_artifact_issues
        .iter()
        .any(|issue| !issue.starts_with("signing_approval"));
    let manual_approval_consumed =
        json_bool_value(&request_preview, "manual_approval_consumed").unwrap_or(false);
    let manual_approval_consume_status =
        json_string_value(&request_preview, "manual_approval_consume_status")
            .unwrap_or_else(|| "unknown".to_string());
    let owner_approval_consumed = manual_approval_consumed
        && manual_approval_consume_status == "approval_consumed_after_request_preview_created";
    let kill_switch_runtime_gate_open =
        json_bool_value(&kill_switch_runtime_gate, "runtime_gate_open").unwrap_or(false);
    let kill_switch_active =
        json_bool_value(&kill_switch_runtime_gate, "kill_switch_active").unwrap_or(true);
    let risk_preflight_decision = json_string_value(&risk_preflight, "risk_decision")
        .unwrap_or_else(|| "unknown".to_string());
    let request_preview_built =
        json_bool_value(&request_preview, "request_preview_built").unwrap_or(false);
    let request_sent = json_bool_value(&request_preview, "request_sent").unwrap_or(true);
    let order_type = json_string_value(&request_preview, "order_type")
        .or_else(|| json_string_value(&order_gate, "order_type"))
        .unwrap_or_else(|| "unknown".to_string())
        .to_ascii_uppercase();
    let time_in_force = json_string_value(&request_preview, "time_in_force")
        .unwrap_or_else(|| "unknown".to_string())
        .to_ascii_uppercase();
    let notional = json_string_value(&request_preview, "notional")
        .or_else(|| json_string_value(&order_gate, "notional"))
        .unwrap_or_else(|| "unknown".to_string());
    let max_order_notional = opt.max_notional.trim().to_string();
    let tiny_notional_gate_ready = match (
        parse_non_negative_decimal(&notional),
        parse_non_negative_decimal(&max_order_notional),
    ) {
        (Ok(order_notional), Ok(max_notional)) => {
            order_notional > Decimal::ZERO && order_notional <= max_notional
        }
        _ => false,
    };
    let signing_approval_ready = signing_approval
        .as_ref()
        .and_then(|artifact| json_bool_value(artifact, "signing_approval_ready"))
        .unwrap_or(false);
    let signing_approval_status = signing_approval
        .as_ref()
        .and_then(|artifact| json_string_value(artifact, "status"))
        .unwrap_or_else(|| "missing_until_v160_003_signing_approval_artifact".to_string());
    let explicit_send_gate_open = false;
    let single_order_candidate = request_preview_built
        && !request_sent
        && order_type == "LIMIT"
        && time_in_force == "GTC"
        && tiny_notional_gate_ready
        && json_u64_value(&request_preview, "production_order_submissions_attempted").unwrap_or(0)
            == 0
        && json_u64_value(&request_preview, "production_orders_submitted").unwrap_or(0) == 0
        && json_u64_value(&request_preview, "production_order_mutations_attempted").unwrap_or(0)
            == 0;
    let kill_switch_checked_before_send =
        json_string_value(&kill_switch_runtime_gate, "schema_version").as_deref()
            == Some(PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION);

    let mut runtime_gate_reasons = Vec::new();
    if !missing_cli_flags.is_empty() {
        runtime_gate_reasons.push("missing_owner_runtime_gate_confirmation".to_string());
    }
    if !owner_approval_consumed {
        runtime_gate_reasons.push("owner_approval_not_consumed".to_string());
    }
    if !kill_switch_checked_before_send {
        runtime_gate_reasons.push("kill_switch_not_checked_before_send".to_string());
    }
    if !kill_switch_runtime_gate_open {
        runtime_gate_reasons.push("kill_switch_runtime_gate_closed".to_string());
    }
    if kill_switch_active {
        runtime_gate_reasons.push("kill_switch_active".to_string());
    }
    if risk_preflight_decision != "dry_run_approved" {
        runtime_gate_reasons.push("risk_preflight_not_approved".to_string());
    }
    if !request_preview_built || request_sent {
        runtime_gate_reasons.push("request_preview_blocked_or_sent".to_string());
    }
    if !single_order_candidate {
        runtime_gate_reasons.push("single_limit_gtc_candidate_not_ready".to_string());
    }
    if !tiny_notional_gate_ready {
        runtime_gate_reasons.push("tiny_notional_gate_not_ready".to_string());
    }
    if !source_artifact_issues.is_empty() {
        runtime_gate_reasons.push("source_artifact_issue".to_string());
    }
    if !signing_approval_ready {
        runtime_gate_reasons.push("signing_approval_missing".to_string());
    }
    if !explicit_send_gate_open {
        runtime_gate_reasons.push("explicit_send_gate_closed".to_string());
    }

    let runtime_gate_open = false;
    let send_consideration_allowed = false;
    let status = if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if kill_switch_active {
        "blocked_kill_switch_active"
    } else if !request_preview_built || request_sent {
        "blocked_request_preview"
    } else if risk_preflight_decision != "dry_run_approved" {
        "blocked_risk_preflight"
    } else if !single_order_candidate || !tiny_notional_gate_ready {
        "blocked_order_scope"
    } else if !owner_approval_consumed {
        "blocked_owner_approval_consumption"
    } else if source_artifact_has_non_signing_issue {
        "blocked_source_artifact"
    } else if !signing_approval_ready {
        "blocked_signing_approval"
    } else if !explicit_send_gate_open {
        "blocked_explicit_send_gate"
    } else {
        "ready_for_guarded_send_consideration"
    };
    let runtime_gate_decision = if send_consideration_allowed {
        "ready_for_guarded_send_consideration"
    } else {
        "blocked_before_any_send_consideration"
    };

    Ok(ProductionMutationRuntimeGateArtifact {
        schema_version: PRODUCTION_MUTATION_RUNTIME_GATE_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_order_gate_path: opt.order_gate.display().to_string(),
        source_risk_preflight_path: opt.risk_preflight.display().to_string(),
        source_request_preview_path: opt.request_preview.display().to_string(),
        source_kill_switch_runtime_gate_path: opt.kill_switch_runtime_gate.display().to_string(),
        source_signing_approval_path: opt
            .signing_approval
            .as_ref()
            .map(|path| path.display().to_string()),
        artifact_type: "production_mutation_runtime_gate".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "owner_approved_single_order_production_mutation_candidate".to_string(),
        capability: "Minimum Owner-Approved Production Order Mutation Candidate".to_string(),
        capability_expansion_from_v15: true,
        default_fail_closed: true,
        runtime_gate_decision: runtime_gate_decision.to_string(),
        runtime_gate_open,
        send_consideration_allowed,
        runtime_gate_reasons,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        owner_approval_required: true,
        owner_approval_consumed,
        manual_approval_consumed,
        manual_approval_consume_status,
        kill_switch_checked_before_send,
        kill_switch_runtime_gate_open,
        kill_switch_active,
        risk_preflight_decision,
        request_preview_built,
        request_sent,
        signing_approval_required: true,
        signing_approval_ready,
        signing_approval_status,
        explicit_send_gate_required: true,
        explicit_send_gate_open,
        single_order_candidate,
        tiny_notional_gate_ready,
        max_order_notional,
        symbol: json_string_value(&request_preview, "symbol")
            .or_else(|| json_string_value(&order_gate, "symbol"))
            .unwrap_or_else(|| "unknown".to_string()),
        side: json_string_value(&request_preview, "side")
            .or_else(|| json_string_value(&order_gate, "side"))
            .unwrap_or_else(|| "unknown".to_string()),
        order_type,
        quantity: json_string_value(&request_preview, "quantity")
            .or_else(|| json_string_value(&order_gate, "quantity"))
            .unwrap_or_else(|| "unknown".to_string()),
        price: json_string_value(&request_preview, "price")
            .unwrap_or_else(|| "unknown".to_string()),
        time_in_force,
        notional,
        production_order_submission_allowed_policy: "owner_approved_single_limit_gtc_only"
            .to_string(),
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        execution_adapter_called: false,
        production_adapter_called: false,
        production_adapter_instantiated: false,
        matching_engine_submission: false,
        actual_submission_count: 0,
        automatic_remediation_attempted: false,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        external_venue_connection: false,
        network_attempted: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        request_redacted: true,
        response_redacted: true,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        raw_exchange_response_recorded: false,
        no_network_before_send_confirmed: opt.confirm_no_network_before_send,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        no_retry_confirmed: opt.confirm_no_retry,
        diagnostic: if status == "blocked_explicit_send_gate" {
            "all source artifacts and signing approval passed, but the explicit send gate remains closed; no send can be considered"
        } else if status == "blocked_signing_approval" {
            "all v0.15 source artifacts passed the v0.16 runtime gate shape, but the separate signing approval artifact is missing or not ready; no send can be considered"
        } else {
            "production mutation runtime gate is blocked before any send consideration; no network, adapter call, or production mutation was attempted"
        }
        .to_string(),
    })
}

fn build_production_live_alpha_execution_dry_run_artifact(
    opt: &LiveProductionLiveAlphaExecutionDryRunOpt,
) -> anyhow::Result<ProductionLiveAlphaExecutionDryRunArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    let order_gate = load_json_value(&opt.order_gate, "live-alpha dry-run order gate")?;
    let risk_preflight = load_json_value(&opt.risk_preflight, "live-alpha risk preflight")?;
    let request_preview = load_json_value(&opt.request_preview, "live-alpha request preview")?;
    let kill_switch_runtime_gate = load_json_value(
        &opt.kill_switch_runtime_gate,
        "live-alpha kill-switch runtime gate",
    )?;

    let missing_cli_flags = missing_production_live_alpha_execution_dry_run_cli_flags(opt);
    let source_artifact_issues = production_live_alpha_execution_dry_run_source_issues(
        &order_gate,
        &risk_preflight,
        &request_preview,
        &kill_switch_runtime_gate,
    );
    let order_gate_ready =
        json_bool_value(&order_gate, "dry_run_order_gate_ready").unwrap_or(false);
    let risk_preflight_decision = json_string_value(&risk_preflight, "risk_decision")
        .unwrap_or_else(|| "unknown".to_string());
    let request_preview_built =
        json_bool_value(&request_preview, "request_preview_built").unwrap_or(false);
    let request_sent = json_bool_value(&request_preview, "request_sent").unwrap_or(false);
    let kill_switch_runtime_gate_status = json_string_value(&kill_switch_runtime_gate, "status")
        .unwrap_or_else(|| "unknown".to_string());
    let kill_switch_runtime_gate_open =
        json_bool_value(&kill_switch_runtime_gate, "runtime_gate_open").unwrap_or(false);
    let ready = missing_cli_flags.is_empty() && source_artifact_issues.is_empty();
    let status = if ready {
        "ready_dry_run_execution_adapter_only"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else {
        "blocked_source_artifact"
    };
    let execution_decision = if ready {
        "dry_run_adapter_artifact_only"
    } else {
        "blocked_no_adapter_route"
    };

    let session_id = json_string_value(&order_gate, "session_id")
        .or_else(|| json_string_value(&request_preview, "session_id"))
        .unwrap_or_else(|| "unknown".to_string());
    let strategy_id = json_string_value(&order_gate, "strategy_id")
        .or_else(|| json_string_value(&request_preview, "strategy_id"))
        .unwrap_or_else(|| "unknown".to_string());
    let symbol = json_string_value(&order_gate, "symbol")
        .or_else(|| json_string_value(&request_preview, "symbol"))
        .unwrap_or_else(|| "unknown".to_string());
    let side = json_string_value(&order_gate, "side")
        .or_else(|| json_string_value(&request_preview, "side"))
        .unwrap_or_else(|| "unknown".to_string());
    let order_type = json_string_value(&order_gate, "order_type")
        .or_else(|| json_string_value(&request_preview, "order_type"))
        .unwrap_or_else(|| "unknown".to_string());
    let quantity = json_string_value(&order_gate, "quantity")
        .or_else(|| json_string_value(&request_preview, "quantity"))
        .unwrap_or_else(|| "unknown".to_string());
    let price =
        json_string_value(&request_preview, "price").unwrap_or_else(|| "unknown".to_string());
    let time_in_force = json_string_value(&request_preview, "time_in_force")
        .unwrap_or_else(|| "unknown".to_string());
    let notional = json_string_value(&order_gate, "notional")
        .or_else(|| json_string_value(&request_preview, "notional"))
        .unwrap_or_else(|| "unknown".to_string());

    Ok(ProductionLiveAlphaExecutionDryRunArtifact {
        schema_version: PRODUCTION_LIVE_ALPHA_EXECUTION_DRY_RUN_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_order_gate_path: opt.order_gate.display().to_string(),
        source_risk_preflight_path: opt.risk_preflight.display().to_string(),
        source_request_preview_path: opt.request_preview.display().to_string(),
        source_kill_switch_runtime_gate_path: opt.kill_switch_runtime_gate.display().to_string(),
        artifact_type: "live_alpha_execution_dry_run_isolation".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "production_live_alpha_execution_dry_run".to_string(),
        execution_decision: execution_decision.to_string(),
        execution_boundary_contract_version:
            "ntpro.v151_execution_dry_run_adapter_boundary.v1".to_string(),
        execution_boundary_flow:
            "StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter"
                .to_string(),
        execution_boundary_contract_ready: ready,
        isolation_route: "strategy_intent_to_risk_preflight_to_local_dry_run_execution_adapter"
            .to_string(),
        strategy_intent_boundary: "StrategyIntent".to_string(),
        risk_decision_boundary: "RiskDecision".to_string(),
        execution_command_boundary: "ExecutionCommand".to_string(),
        execution_command_created: ready,
        execution_command_route: if ready {
            "dry_run_adapter_only"
        } else {
            "blocked_before_execution_command"
        }
        .to_string(),
        execution_command_destination: if ready {
            "ntpro_local_artifact_dry_run_execution_adapter"
        } else {
            "none"
        }
        .to_string(),
        dry_run_adapter_boundary: "DryRunExecutionAdapter".to_string(),
        dry_run_adapter_route_allowed: ready,
        production_adapter_boundary: "ProductionExecutionAdapter".to_string(),
        production_adapter_route_allowed: false,
        production_adapter_instantiation_allowed: false,
        dry_run_execution_adapter: "ntpro_local_artifact_dry_run_execution_adapter".to_string(),
        dry_run_execution_adapter_called: ready,
        dry_run_execution_adapter_wrote_artifact: ready,
        dry_run_adapter_artifact_only: ready,
        real_execution_adapter_called: false,
        production_adapter_instantiated: false,
        production_adapter_called: false,
        strategy_intent_recorded: ready,
        strategy_intent_reaches_risk_preflight: ready,
        strategy_intent_reaches_dry_run_adapter: ready,
        strategy_intent_reaches_production_adapter: false,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        order_gate_ready,
        risk_preflight_decision,
        request_preview_built,
        request_sent,
        kill_switch_runtime_gate_status,
        kill_switch_runtime_gate_open,
        session_id,
        strategy_id,
        symbol,
        side,
        order_type,
        quantity,
        price,
        time_in_force,
        notional,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        network_attempted: false,
        matching_engine_submission: false,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        external_venue_connection: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        order_state_values_are_exchange_truth: false,
        shadow_values_are_exchange_truth: false,
        portfolio_values_are_exchange_truth: false,
        values_are_exchange_truth: false,
        no_production_order_submission_confirmed: opt.confirm_no_production_order_submission,
        no_production_order_mutation_confirmed: opt.confirm_no_production_order_mutation,
        no_production_adapter_confirmed: opt.confirm_no_production_adapter,
        no_network_confirmed: opt.confirm_no_network,
        no_listen_key_lifecycle_confirmed: opt.confirm_no_listen_key_lifecycle,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_real_funds_confirmed: opt.confirm_no_real_funds,
        diagnostic: if ready {
            "strategy intent was converted into a dry-run execution command that reached only the local dry-run execution adapter artifact; production adapters, network, and order mutation remained disabled"
        } else {
            "execution dry-run isolation is blocked before execution command creation until owner confirmations and source artifacts prove the local dry-run path"
        }
        .to_string(),
    })
}

fn production_live_alpha_execution_dry_run_source_issues(
    order_gate: &serde_json::Value,
    risk_preflight: &serde_json::Value,
    request_preview: &serde_json::Value,
    kill_switch_runtime_gate: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(order_gate, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION)
    {
        issues.push("order_gate_schema_mismatch".to_string());
    }
    if !json_bool_value(order_gate, "dry_run_order_gate_ready").unwrap_or(false) {
        issues.push("order_gate_not_ready".to_string());
    }
    if json_string_value(risk_preflight, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_REPORT_SCHEMA_VERSION)
    {
        issues.push("risk_preflight_schema_mismatch".to_string());
    }
    if json_string_value(risk_preflight, "risk_decision").as_deref() != Some("dry_run_approved") {
        issues.push("risk_preflight_not_dry_run_approved".to_string());
    }
    if json_string_value(risk_preflight, "execution_decision").as_deref()
        != Some("blocked_no_production_mutation")
    {
        issues.push("risk_preflight_execution_decision_mismatch".to_string());
    }
    if json_bool_value(risk_preflight, "execution_adapter_called").unwrap_or(false) {
        issues.push("risk_preflight_touched_execution_adapter".to_string());
    }
    if json_string_value(request_preview, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION)
    {
        issues.push("request_preview_schema_mismatch".to_string());
    }
    if !json_bool_value(request_preview, "request_preview_built").unwrap_or(false) {
        issues.push("request_preview_not_built".to_string());
    }
    if json_bool_value(request_preview, "request_sent").unwrap_or(false) {
        issues.push("request_preview_sent_request".to_string());
    }
    if json_bool_value(request_preview, "production_adapter_called").unwrap_or(false) {
        issues.push("request_preview_touched_production_adapter".to_string());
    }
    if json_bool_value(request_preview, "execution_adapter_called").unwrap_or(false) {
        issues.push("request_preview_touched_execution_adapter".to_string());
    }
    if json_bool_value(request_preview, "network_attempted").unwrap_or(false) {
        issues.push("request_preview_attempted_network".to_string());
    }
    if json_string_value(kill_switch_runtime_gate, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION)
    {
        issues.push("kill_switch_runtime_gate_schema_mismatch".to_string());
    }
    if json_string_value(kill_switch_runtime_gate, "status").as_deref()
        != Some("ready_runtime_gate_open_for_dry_run_only")
    {
        issues.push("kill_switch_runtime_gate_not_ready".to_string());
    }
    if !json_bool_value(kill_switch_runtime_gate, "runtime_gate_open").unwrap_or(false) {
        issues.push("kill_switch_runtime_gate_closed".to_string());
    }
    if json_bool_value(kill_switch_runtime_gate, "kill_switch_active").unwrap_or(true) {
        issues.push("kill_switch_runtime_gate_active".to_string());
    }
    if artifact_has_production_mutation(Some(order_gate)) {
        issues.push("order_gate_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(risk_preflight)) {
        issues.push("risk_preflight_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(request_preview)) {
        issues.push("request_preview_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(kill_switch_runtime_gate)) {
        issues.push("kill_switch_runtime_gate_records_forbidden_production_mutation".to_string());
    }

    for field in [
        "session_id",
        "strategy_id",
        "symbol",
        "side",
        "order_type",
        "quantity",
    ] {
        push_json_string_alignment_issue(
            &mut issues,
            field,
            "order_gate",
            order_gate,
            "request_preview",
            request_preview,
        );
    }
    for field in ["symbol", "side", "order_type", "quantity", "notional"] {
        push_json_string_alignment_issue(
            &mut issues,
            field,
            "order_gate",
            order_gate,
            "risk_preflight",
            risk_preflight,
        );
    }
    issues
}

fn production_mutation_runtime_gate_source_issues(
    order_gate: &serde_json::Value,
    risk_preflight: &serde_json::Value,
    request_preview: &serde_json::Value,
    kill_switch_runtime_gate: &serde_json::Value,
    signing_approval: Option<&serde_json::Value>,
    max_notional: &str,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(order_gate, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION)
    {
        issues.push("order_gate_schema_mismatch".to_string());
    }
    if !json_bool_value(order_gate, "dry_run_order_gate_ready").unwrap_or(false) {
        issues.push("order_gate_not_ready".to_string());
    }
    if json_string_value(order_gate, "order_type").as_deref() != Some("LIMIT") {
        issues.push("order_gate_not_limit".to_string());
    }
    if json_string_value(risk_preflight, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_REPORT_SCHEMA_VERSION)
    {
        issues.push("risk_preflight_schema_mismatch".to_string());
    }
    if json_string_value(risk_preflight, "risk_decision").as_deref() != Some("dry_run_approved") {
        issues.push("risk_preflight_not_dry_run_approved".to_string());
    }
    if json_bool_value(risk_preflight, "kill_switch_active").unwrap_or(true) {
        issues.push("risk_preflight_kill_switch_active".to_string());
    }
    if json_bool_value(risk_preflight, "execution_adapter_called").unwrap_or(false) {
        issues.push("risk_preflight_touched_execution_adapter".to_string());
    }
    if json_string_value(request_preview, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION)
    {
        issues.push("request_preview_schema_mismatch".to_string());
    }
    if json_string_value(request_preview, "status").as_deref() != Some("ready_request_preview_only")
    {
        issues.push("request_preview_not_ready".to_string());
    }
    if !json_bool_value(request_preview, "request_preview_built").unwrap_or(false) {
        issues.push("request_preview_not_built".to_string());
    }
    if json_bool_value(request_preview, "request_sent").unwrap_or(true) {
        issues.push("request_preview_sent_request".to_string());
    }
    if json_string_value(request_preview, "request_method").as_deref() != Some("POST") {
        issues.push("request_preview_method_not_post".to_string());
    }
    if json_string_value(request_preview, "request_target").as_deref() != Some("/api/v3/order") {
        issues.push("request_preview_target_not_order".to_string());
    }
    if json_string_value(request_preview, "order_type").as_deref() != Some("LIMIT") {
        issues.push("request_preview_not_limit".to_string());
    }
    if json_string_value(request_preview, "time_in_force").as_deref() != Some("GTC") {
        issues.push("request_preview_not_gtc".to_string());
    }
    if !json_bool_value(request_preview, "manual_approval_consumed").unwrap_or(false) {
        issues.push("manual_approval_not_consumed".to_string());
    }
    if json_string_value(request_preview, "manual_approval_consume_status").as_deref()
        != Some("approval_consumed_after_request_preview_created")
    {
        issues.push("manual_approval_consume_status_mismatch".to_string());
    }
    if json_bool_value(request_preview, "network_attempted").unwrap_or(false) {
        issues.push("request_preview_attempted_network".to_string());
    }
    if json_bool_value(request_preview, "production_adapter_called").unwrap_or(false) {
        issues.push("request_preview_touched_production_adapter".to_string());
    }
    if json_string_value(kill_switch_runtime_gate, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION)
    {
        issues.push("kill_switch_runtime_gate_schema_mismatch".to_string());
    }
    if json_string_value(kill_switch_runtime_gate, "status").as_deref()
        != Some("ready_runtime_gate_open_for_dry_run_only")
    {
        issues.push("kill_switch_runtime_gate_not_ready".to_string());
    }
    if !json_bool_value(kill_switch_runtime_gate, "runtime_gate_open").unwrap_or(false) {
        issues.push("kill_switch_runtime_gate_closed".to_string());
    }
    if json_bool_value(kill_switch_runtime_gate, "kill_switch_active").unwrap_or(true) {
        issues.push("kill_switch_runtime_gate_active".to_string());
    }
    if artifact_has_production_mutation(Some(order_gate)) {
        issues.push("order_gate_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(risk_preflight)) {
        issues.push("risk_preflight_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(request_preview)) {
        issues.push("request_preview_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(kill_switch_runtime_gate)) {
        issues.push("kill_switch_runtime_gate_records_forbidden_production_mutation".to_string());
    }
    match signing_approval {
        Some(approval) => {
            if json_string_value(approval, "schema_version").as_deref()
                != Some(PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION)
            {
                issues.push("signing_approval_schema_mismatch".to_string());
            }
            if json_string_value(approval, "status").as_deref()
                != Some("ready_signing_material_approval")
            {
                issues.push("signing_approval_not_ready".to_string());
            }
            if !json_bool_value(approval, "signing_approval_ready").unwrap_or(false) {
                issues.push("signing_approval_ready_false".to_string());
            }
            if !json_bool_value(approval, "owner_approved_signing_material").unwrap_or(false) {
                issues.push("signing_approval_owner_not_approved".to_string());
            }
            if json_string_value(approval, "credential_material").as_deref()
                != Some("production_live_alpha")
            {
                issues.push("signing_approval_material_mismatch".to_string());
            }
            if !json_bool_value(approval, "production_signing_material_gate_open").unwrap_or(false)
            {
                issues.push("signing_approval_gate_not_open".to_string());
            }
            if !json_bool_value(approval, "production_signing_material_env_read").unwrap_or(false) {
                issues.push("signing_approval_env_not_read".to_string());
            }
            for field in [
                "api_key_value_recorded",
                "api_secret_value_recorded",
                "signature_recorded",
                "signed_query_recorded",
                "signed_url_recorded",
                "request_body_recorded",
                "raw_request_body_recorded",
                "network_attempted",
                "dashboard_order_controls_enabled",
            ] {
                if json_bool_value(approval, field).unwrap_or(false) {
                    issues.push(format!("signing_approval_{field}_true"));
                }
            }
            if artifact_has_production_mutation(Some(approval)) {
                issues.push("signing_approval_records_forbidden_production_mutation".to_string());
            }
        }
        None => {
            issues.push("signing_approval_missing".to_string());
        }
    }

    for field in [
        "session_id",
        "strategy_id",
        "symbol",
        "side",
        "order_type",
        "quantity",
        "notional",
    ] {
        push_json_string_alignment_issue(
            &mut issues,
            field,
            "order_gate",
            order_gate,
            "request_preview",
            request_preview,
        );
    }
    for field in ["symbol", "side", "order_type", "quantity", "notional"] {
        push_json_string_alignment_issue(
            &mut issues,
            field,
            "order_gate",
            order_gate,
            "risk_preflight",
            risk_preflight,
        );
    }

    match (
        json_string_value(request_preview, "notional"),
        parse_non_negative_decimal(max_notional),
    ) {
        (Some(notional), Ok(max_allowed)) => match parse_non_negative_decimal(&notional) {
            Ok(order_notional)
                if order_notional > Decimal::ZERO && order_notional <= max_allowed => {}
            Ok(_) => issues.push("notional_not_tiny_or_positive".to_string()),
            Err(_) => issues.push("notional_parse_failed".to_string()),
        },
        (None, _) => issues.push("notional_missing".to_string()),
        (_, Err(_)) => issues.push("max_notional_parse_failed".to_string()),
    }
    issues
}

fn production_mutation_signing_approval_source_issues(
    request_preview: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(request_preview, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION)
    {
        issues.push("request_preview_schema_mismatch".to_string());
    }
    if json_string_value(request_preview, "status").as_deref() != Some("ready_request_preview_only")
    {
        issues.push("request_preview_not_ready".to_string());
    }
    if !json_bool_value(request_preview, "request_preview_built").unwrap_or(false) {
        issues.push("request_preview_not_built".to_string());
    }
    if json_bool_value(request_preview, "request_sent").unwrap_or(true) {
        issues.push("request_preview_sent_request".to_string());
    }
    if json_string_value(request_preview, "credential_material").as_deref()
        != Some("production_live_alpha")
    {
        issues.push("request_preview_not_production_live_alpha_material".to_string());
    }
    if !json_bool_value(request_preview, "production_signing_material_gate_required")
        .unwrap_or(false)
    {
        issues.push("production_signing_material_gate_not_required".to_string());
    }
    if !json_bool_value(request_preview, "production_signing_material_gate_open").unwrap_or(false) {
        issues.push("production_signing_material_gate_not_open".to_string());
    }
    if !json_bool_value(request_preview, "production_signing_material_env_read").unwrap_or(false) {
        issues.push("production_signing_material_env_not_read".to_string());
    }
    if !json_string_array(
        request_preview,
        "production_signing_material_missing_gate_env_vars",
    )
    .is_empty()
    {
        issues.push("production_signing_material_missing_gate_env_vars_present".to_string());
    }
    if json_string_value(request_preview, "order_type").as_deref() != Some("LIMIT") {
        issues.push("request_preview_not_limit".to_string());
    }
    if json_string_value(request_preview, "time_in_force").as_deref() != Some("GTC") {
        issues.push("request_preview_not_gtc".to_string());
    }
    for field in [
        "api_key_header_value_recorded",
        "api_secret_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "request_body_recorded",
        "raw_request_body_recorded",
        "network_attempted",
        "execution_adapter_called",
        "production_adapter_called",
        "dashboard_order_controls_enabled",
    ] {
        if json_bool_value(request_preview, field).unwrap_or(false) {
            issues.push(format!("request_preview_{field}_true"));
        }
    }
    if artifact_has_production_mutation(Some(request_preview)) {
        issues.push("request_preview_records_forbidden_production_mutation".to_string());
    }
    issues
}

fn production_mutation_request_builder_source_issues(
    runtime_gate: &serde_json::Value,
    signing_approval: &serde_json::Value,
    request_preview: &serde_json::Value,
    max_notional: &str,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(runtime_gate, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_RUNTIME_GATE_SCHEMA_VERSION)
    {
        issues.push("runtime_gate_schema_mismatch".to_string());
    }
    if json_string_value(runtime_gate, "status").as_deref() != Some("blocked_explicit_send_gate") {
        issues.push("runtime_gate_not_blocked_explicit_send_gate".to_string());
    }
    if json_bool_value(runtime_gate, "runtime_gate_open").unwrap_or(true) {
        issues.push("runtime_gate_open_unexpected".to_string());
    }
    if json_bool_value(runtime_gate, "send_consideration_allowed").unwrap_or(true) {
        issues.push("send_consideration_allowed_unexpected".to_string());
    }
    if json_bool_value(runtime_gate, "explicit_send_gate_open").unwrap_or(true) {
        issues.push("explicit_send_gate_open_unexpected".to_string());
    }
    if !json_bool_value(runtime_gate, "signing_approval_ready").unwrap_or(false) {
        issues.push("runtime_gate_signing_approval_not_ready".to_string());
    }
    if json_string_value(signing_approval, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION)
    {
        issues.push("signing_approval_schema_mismatch".to_string());
    }
    if json_string_value(signing_approval, "status").as_deref()
        != Some("ready_signing_material_approval")
    {
        issues.push("signing_approval_not_ready".to_string());
    }
    if !json_bool_value(signing_approval, "signing_approval_ready").unwrap_or(false) {
        issues.push("signing_approval_ready_false".to_string());
    }
    if json_string_value(signing_approval, "credential_material").as_deref()
        != Some("production_live_alpha")
    {
        issues.push("signing_approval_material_mismatch".to_string());
    }
    if json_string_value(request_preview, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION)
    {
        issues.push("request_preview_schema_mismatch".to_string());
    }
    if json_string_value(request_preview, "status").as_deref() != Some("ready_request_preview_only")
    {
        issues.push("request_preview_not_ready".to_string());
    }
    if json_string_value(request_preview, "credential_material").as_deref()
        != Some("production_live_alpha")
    {
        issues.push("request_preview_not_production_live_alpha_material".to_string());
    }
    if !json_bool_value(request_preview, "request_preview_built").unwrap_or(false) {
        issues.push("request_preview_not_built".to_string());
    }
    if json_bool_value(request_preview, "request_sent").unwrap_or(true) {
        issues.push("request_preview_sent_request".to_string());
    }
    if json_string_value(request_preview, "request_method").as_deref() != Some("POST") {
        issues.push("request_preview_method_not_post".to_string());
    }
    if json_string_value(request_preview, "request_target").as_deref() != Some("/api/v3/order") {
        issues.push("request_preview_target_not_order".to_string());
    }
    if json_string_value(request_preview, "order_type").as_deref() != Some("LIMIT") {
        issues.push("request_preview_not_limit".to_string());
    }
    if json_string_value(request_preview, "time_in_force").as_deref() != Some("GTC") {
        issues.push("request_preview_not_gtc".to_string());
    }
    if json_string_value(request_preview, "api_key_env").as_deref()
        != Some(credentials.api_key_env.as_str())
    {
        issues.push("request_preview_api_key_env_mismatch".to_string());
    }
    if json_string_value(request_preview, "api_secret_env").as_deref()
        != Some(credentials.api_secret_env.as_str())
    {
        issues.push("request_preview_api_secret_env_mismatch".to_string());
    }
    if !json_bool_value(request_preview, "production_signing_material_gate_open").unwrap_or(false) {
        issues.push("request_preview_signing_material_gate_not_open".to_string());
    }
    if !json_bool_value(request_preview, "production_signing_material_env_read").unwrap_or(false) {
        issues.push("request_preview_signing_material_env_not_read".to_string());
    }
    for field in [
        "api_key_header_value_recorded",
        "api_secret_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "request_body_recorded",
        "raw_request_body_recorded",
        "network_attempted",
        "execution_adapter_called",
        "production_adapter_called",
        "dashboard_order_controls_enabled",
    ] {
        if json_bool_value(request_preview, field).unwrap_or(false) {
            issues.push(format!("request_preview_{field}_true"));
        }
    }
    match (
        json_string_value(request_preview, "notional"),
        parse_non_negative_decimal(max_notional),
    ) {
        (Some(notional), Ok(max_allowed)) => match parse_non_negative_decimal(&notional) {
            Ok(order_notional)
                if order_notional > Decimal::ZERO && order_notional <= max_allowed => {}
            Ok(_) => issues.push("notional_not_tiny_or_positive".to_string()),
            Err(_) => issues.push("notional_parse_failed".to_string()),
        },
        (None, _) => issues.push("notional_missing".to_string()),
        (_, Err(_)) => issues.push("max_notional_parse_failed".to_string()),
    }
    if artifact_has_production_mutation(Some(runtime_gate)) {
        issues.push("runtime_gate_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(signing_approval)) {
        issues.push("signing_approval_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(request_preview)) {
        issues.push("request_preview_records_forbidden_production_mutation".to_string());
    }
    issues
}

fn production_mutation_guarded_send_source_issues(
    request_builder: &serde_json::Value,
    kill_switch_runtime_gate: &serde_json::Value,
    request_preview: &serde_json::Value,
    max_notional: &str,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(request_builder, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION)
    {
        issues.push("request_builder_schema_mismatch".to_string());
    }
    if json_string_value(request_builder, "status").as_deref()
        != Some("ready_request_object_built_no_send")
    {
        issues.push("request_builder_not_ready".to_string());
    }
    if !json_bool_value(request_builder, "request_object_built").unwrap_or(false) {
        issues.push("request_object_not_built".to_string());
    }
    if json_bool_value(request_builder, "request_sent").unwrap_or(false) {
        issues.push("request_builder_already_sent".to_string());
    }
    if json_bool_value(request_builder, "network_attempted").unwrap_or(false) {
        issues.push("request_builder_attempted_network".to_string());
    }
    if json_string_value(kill_switch_runtime_gate, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION)
    {
        issues.push("kill_switch_runtime_gate_schema_mismatch".to_string());
    }
    if !json_bool_value(kill_switch_runtime_gate, "runtime_gate_open").unwrap_or(false) {
        issues.push("kill_switch_runtime_gate_not_open".to_string());
    }
    if json_bool_value(kill_switch_runtime_gate, "kill_switch_active").unwrap_or(true) {
        issues.push("kill_switch_active_before_send".to_string());
    }
    if json_bool_value(kill_switch_runtime_gate, "request_sent").unwrap_or(false) {
        issues.push("kill_switch_runtime_gate_records_request_sent".to_string());
    }
    if json_string_value(request_preview, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION)
    {
        issues.push("request_preview_schema_mismatch".to_string());
    }
    if json_string_value(request_preview, "status").as_deref() != Some("ready_request_preview_only")
    {
        issues.push("request_preview_not_ready".to_string());
    }
    if json_string_value(request_preview, "credential_material").as_deref()
        != Some("production_live_alpha")
    {
        issues.push("request_preview_not_production_live_alpha_material".to_string());
    }
    if json_string_value(request_preview, "request_method").as_deref() != Some("POST") {
        issues.push("request_preview_method_not_post".to_string());
    }
    if json_string_value(request_preview, "request_target").as_deref() != Some("/api/v3/order") {
        issues.push("request_preview_target_not_order".to_string());
    }
    if json_string_value(request_preview, "order_type").as_deref() != Some("LIMIT") {
        issues.push("request_preview_not_limit".to_string());
    }
    if json_string_value(request_preview, "time_in_force").as_deref() != Some("GTC") {
        issues.push("request_preview_not_gtc".to_string());
    }
    if json_string_value(request_preview, "api_key_env").as_deref()
        != Some(credentials.api_key_env.as_str())
    {
        issues.push("request_preview_api_key_env_mismatch".to_string());
    }
    if json_string_value(request_preview, "api_secret_env").as_deref()
        != Some(credentials.api_secret_env.as_str())
    {
        issues.push("request_preview_api_secret_env_mismatch".to_string());
    }
    match (
        json_string_value(request_preview, "notional"),
        parse_non_negative_decimal(max_notional),
    ) {
        (Some(notional), Ok(max_allowed)) => match parse_non_negative_decimal(&notional) {
            Ok(order_notional)
                if order_notional > Decimal::ZERO && order_notional <= max_allowed => {}
            Ok(_) => issues.push("notional_not_tiny_or_positive".to_string()),
            Err(_) => issues.push("notional_parse_failed".to_string()),
        },
        (None, _) => issues.push("notional_missing".to_string()),
        (_, Err(_)) => issues.push("max_notional_parse_failed".to_string()),
    }
    if artifact_has_production_mutation(Some(request_builder)) {
        issues.push("request_builder_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(kill_switch_runtime_gate)) {
        issues.push("kill_switch_runtime_gate_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(request_preview)) {
        issues.push("request_preview_records_forbidden_production_mutation".to_string());
    }
    issues
}

fn production_mutation_guarded_send_kill_switch_snapshot(
    source_path: &Path,
    kill_switch_runtime_gate: &serde_json::Value,
) -> ProductionMutationKillSwitchSnapshot {
    ProductionMutationKillSwitchSnapshot {
        runtime_gate_open: json_bool_value(kill_switch_runtime_gate, "runtime_gate_open")
            .unwrap_or(false),
        kill_switch_active: json_bool_value(kill_switch_runtime_gate, "kill_switch_active")
            .unwrap_or(true),
        checked: json_string_value(kill_switch_runtime_gate, "schema_version").as_deref()
            == Some(PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION),
        source_path: source_path.display().to_string(),
        source_hash: file_fnv1a64_hash(&source_path.display().to_string()),
        checked_at: now_millis(),
    }
}

fn production_mutation_response_redaction_source_issues(
    guarded_send: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(guarded_send, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION)
    {
        issues.push("guarded_send_schema_mismatch".to_string());
    }
    if json_bool_value(guarded_send, "api_key_value_recorded").unwrap_or(false) {
        issues.push("guarded_send_records_api_key".to_string());
    }
    if json_bool_value(guarded_send, "api_secret_value_recorded").unwrap_or(false) {
        issues.push("guarded_send_records_api_secret".to_string());
    }
    if json_bool_value(guarded_send, "api_key_header_value_recorded").unwrap_or(false) {
        issues.push("guarded_send_records_api_key_header".to_string());
    }
    if json_bool_value(guarded_send, "signature_recorded").unwrap_or(false) {
        issues.push("guarded_send_records_signature".to_string());
    }
    if json_bool_value(guarded_send, "signed_query_recorded").unwrap_or(false) {
        issues.push("guarded_send_records_signed_query".to_string());
    }
    if json_bool_value(guarded_send, "signed_url_recorded").unwrap_or(false) {
        issues.push("guarded_send_records_signed_url".to_string());
    }
    if json_bool_value(guarded_send, "raw_exchange_response_recorded").unwrap_or(false) {
        issues.push("guarded_send_records_raw_exchange_response".to_string());
    }
    if json_bool_value(guarded_send, "response_body_recorded").unwrap_or(false) {
        issues.push("guarded_send_records_response_body".to_string());
    }
    if json_bool_value(guarded_send, "dashboard_order_controls_enabled").unwrap_or(false) {
        issues.push("guarded_send_enables_dashboard_order_controls".to_string());
    }
    if json_bool_value(guarded_send, "retry_attempted").unwrap_or(false) {
        issues.push("guarded_send_records_retry".to_string());
    }
    if json_bool_value(guarded_send, "cancel_attempted").unwrap_or(false)
        || json_bool_value(guarded_send, "replace_attempted").unwrap_or(false)
        || json_bool_value(guarded_send, "amend_attempted").unwrap_or(false)
        || json_bool_value(guarded_send, "flatten_attempted").unwrap_or(false)
    {
        issues.push("guarded_send_records_disallowed_followup_mutation".to_string());
    }
    issues
}

fn production_mutation_order_state_readback_source_issues(
    response_redaction: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(response_redaction, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION)
    {
        issues.push("response_redaction_schema_mismatch".to_string());
    }
    if json_string_value(response_redaction, "status").as_deref() != Some("ready_response_redacted")
    {
        issues.push("response_redaction_not_ready".to_string());
    }
    if !json_bool_value(response_redaction, "response_redacted").unwrap_or(false) {
        issues.push("response_redaction_not_confirmed".to_string());
    }
    if json_bool_value(response_redaction, "raw_exchange_response_recorded").unwrap_or(false) {
        issues.push("response_redaction_records_raw_exchange_response".to_string());
    }
    if json_bool_value(response_redaction, "response_body_recorded").unwrap_or(false) {
        issues.push("response_redaction_records_response_body".to_string());
    }
    if json_bool_value(response_redaction, "response_headers_recorded").unwrap_or(false) {
        issues.push("response_redaction_records_response_headers".to_string());
    }
    if json_bool_value(response_redaction, "signature_recorded").unwrap_or(false)
        || json_bool_value(response_redaction, "signed_query_recorded").unwrap_or(false)
        || json_bool_value(response_redaction, "signed_url_recorded").unwrap_or(false)
        || json_bool_value(response_redaction, "api_key_value_recorded").unwrap_or(false)
        || json_bool_value(response_redaction, "api_secret_value_recorded").unwrap_or(false)
    {
        issues.push("response_redaction_records_secret_material".to_string());
    }
    if json_scalar_string_value(response_redaction, "order_id")
        .as_deref()
        .is_none_or(|value| value == "missing" || value.trim().is_empty())
        && json_scalar_string_value(response_redaction, "client_order_id")
            .as_deref()
            .is_none_or(|value| value == "missing" || value.trim().is_empty())
    {
        issues.push("known_order_identifier_missing".to_string());
    }
    if json_scalar_string_value(response_redaction, "symbol")
        .as_deref()
        .is_none_or(|value| value == "unknown" || value.trim().is_empty())
    {
        issues.push("symbol_missing".to_string());
    }
    issues
}

fn production_mutation_audit_trail_source_issues(
    request_builder: &serde_json::Value,
    guarded_send: &serde_json::Value,
    response_redaction: &serde_json::Value,
    order_state_readback: &serde_json::Value,
    runtime_gate: &serde_json::Value,
    signing_approval: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(request_builder, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION)
    {
        issues.push("request_builder_schema_mismatch".to_string());
    }
    if json_string_value(guarded_send, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION)
    {
        issues.push("guarded_send_schema_mismatch".to_string());
    }
    if json_string_value(response_redaction, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION)
    {
        issues.push("response_redaction_schema_mismatch".to_string());
    }
    if json_string_value(order_state_readback, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_ORDER_STATE_READBACK_SCHEMA_VERSION)
    {
        issues.push("order_state_readback_schema_mismatch".to_string());
    }
    if json_string_value(runtime_gate, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_RUNTIME_GATE_SCHEMA_VERSION)
    {
        issues.push("runtime_gate_schema_mismatch".to_string());
    }
    if json_string_value(signing_approval, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION)
    {
        issues.push("signing_approval_schema_mismatch".to_string());
    }
    if json_string_value(request_builder, "status").as_deref()
        != Some("ready_request_object_built_no_send")
    {
        issues.push("request_builder_not_ready".to_string());
    }
    if !matches!(
        json_string_value(guarded_send, "status").as_deref(),
        Some("ready_guarded_send_path_offline_no_network" | "manual_online_send_attempt_recorded")
    ) {
        issues.push("guarded_send_not_auditable".to_string());
    }
    if json_string_value(response_redaction, "status").as_deref() != Some("ready_response_redacted")
    {
        issues.push("response_redaction_not_ready".to_string());
    }
    if !matches!(
        json_string_value(order_state_readback, "status").as_deref(),
        Some("ready_offline_order_state_readback_contract" | "online_order_state_read_ok")
    ) {
        issues.push("order_state_readback_not_ready".to_string());
    }
    if json_string_value(signing_approval, "status").as_deref()
        != Some("ready_signing_material_approval")
    {
        issues.push("signing_approval_not_ready".to_string());
    }
    if json_string_value(runtime_gate, "status").as_deref() != Some("blocked_explicit_send_gate") {
        issues.push("runtime_gate_not_request_builder_fail_closed".to_string());
    }
    if json_bool_value(runtime_gate, "runtime_gate_open").unwrap_or(true) {
        issues.push("runtime_gate_open_unexpected".to_string());
    }
    if json_bool_value(runtime_gate, "send_consideration_allowed").unwrap_or(true) {
        issues.push("send_consideration_allowed_unexpected".to_string());
    }
    if !json_bool_value(guarded_send, "kill_switch_checked_before_send").unwrap_or(false)
        || !json_bool_value(guarded_send, "kill_switch_checked_after_send").unwrap_or(false)
    {
        issues.push("kill_switch_not_checked_around_send".to_string());
    }
    if json_bool_value(guarded_send, "pre_send_kill_switch_active").unwrap_or(true)
        || json_bool_value(guarded_send, "post_send_kill_switch_active").unwrap_or(true)
        || json_bool_value(guarded_send, "kill_switch_blocked_send").unwrap_or(true)
    {
        issues.push("kill_switch_blocked_or_active".to_string());
    }
    if !json_bool_value(response_redaction, "response_redacted").unwrap_or(false)
        || !json_bool_value(order_state_readback, "response_redacted").unwrap_or(false)
    {
        issues.push("response_redaction_not_confirmed".to_string());
    }
    for (label, artifact) in [
        ("request_builder", request_builder),
        ("guarded_send", guarded_send),
        ("response_redaction", response_redaction),
        ("order_state_readback", order_state_readback),
    ] {
        for field in [
            "api_key_value_recorded",
            "api_secret_value_recorded",
            "api_key_header_value_recorded",
            "signature_recorded",
            "signed_query_recorded",
            "signed_url_recorded",
            "raw_exchange_response_recorded",
            "response_body_recorded",
            "response_headers_recorded",
            "dashboard_order_controls_enabled",
            "retry_attempted",
            "cancel_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
        ] {
            if json_bool_value(artifact, field).unwrap_or(false) {
                issues.push(format!("{label}_{field}_true"));
            }
        }
    }
    if artifact_has_production_mutation(Some(response_redaction))
        || artifact_has_production_mutation(Some(order_state_readback))
    {
        issues.push("audit_source_records_forbidden_mutation".to_string());
    }
    issues
}

fn production_mutation_audit_failure_state(
    missing_cli_flags: &[&'static str],
    source_artifact_issues: &[String],
    guarded_send: &serde_json::Value,
    order_state_readback: &serde_json::Value,
) -> String {
    if !missing_cli_flags.is_empty() {
        "blocked_missing_gate".to_string()
    } else if json_bool_value(guarded_send, "kill_switch_blocked_send").unwrap_or(false) {
        "blocked_kill_switch".to_string()
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact".to_string()
    } else if json_bool_value(guarded_send, "request_sent").unwrap_or(false)
        && json_string_value(guarded_send, "error_code").as_deref() != Some("none")
    {
        json_string_value(guarded_send, "error_code").unwrap_or_else(|| "send_error".to_string())
    } else if json_bool_value(order_state_readback, "order_state_read_attempted").unwrap_or(false)
        && json_string_value(order_state_readback, "error_code").as_deref() != Some("none")
    {
        json_string_value(order_state_readback, "error_code")
            .unwrap_or_else(|| "readback_error".to_string())
    } else {
        "none_recorded".to_string()
    }
}

impl ProductionMutationFailureMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Http4xx => "http-4xx",
            Self::Http5xx => "http-5xx",
            Self::MalformedResponse => "malformed-response",
            Self::ReadbackMismatch => "readback-mismatch",
            Self::KillSwitchTransition => "kill-switch-transition",
        }
    }

    fn category(self) -> &'static str {
        match self {
            Self::Timeout => "transport_timeout",
            Self::Http4xx => "exchange_rejected_or_client_error",
            Self::Http5xx => "exchange_or_gateway_error",
            Self::MalformedResponse => "response_shape_error",
            Self::ReadbackMismatch => "post_submit_observability_mismatch",
            Self::KillSwitchTransition => "kill_switch_transition",
        }
    }

    fn failure_state(self) -> &'static str {
        match self {
            Self::Timeout => "timeout_write_evidence_and_stop",
            Self::Http4xx => "http_4xx_write_evidence_and_stop",
            Self::Http5xx => "http_5xx_write_evidence_and_stop",
            Self::MalformedResponse => "malformed_response_write_evidence_and_stop",
            Self::ReadbackMismatch => "readback_mismatch_write_evidence_and_stop",
            Self::KillSwitchTransition => "kill_switch_transition_write_evidence_and_stop",
        }
    }
}

fn production_mutation_failure_semantics_source_issues(
    audit_trail: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(audit_trail, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION)
    {
        issues.push("audit_trail_schema_mismatch".to_string());
    }
    if json_string_value(audit_trail, "status").as_deref() != Some("ready_redacted_audit_trail") {
        issues.push("audit_trail_not_ready".to_string());
    }
    if !json_bool_value(audit_trail, "audit_trail_ready").unwrap_or(false) {
        issues.push("audit_trail_ready_false".to_string());
    }
    if !json_bool_value(audit_trail, "no_retry_or_followup_mutation_confirmed").unwrap_or(false) {
        issues.push("audit_trail_no_retry_not_confirmed".to_string());
    }
    if !json_bool_value(audit_trail, "dashboard_controls_disabled_confirmed").unwrap_or(false) {
        issues.push("audit_trail_dashboard_controls_not_confirmed".to_string());
    }
    if !json_bool_value(audit_trail, "no_listen_key_lifecycle_confirmed").unwrap_or(false) {
        issues.push("audit_trail_listen_key_not_confirmed".to_string());
    }
    for field in [
        "retry_attempted",
        "cancel_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "dashboard_order_controls_enabled",
        "production_order_mutation_allowed",
        "listen_key_lifecycle_allowed",
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "unrestricted_payload_recorded",
        "account_balances_recorded",
        "production_trading_enabled",
    ] {
        if json_bool_value(audit_trail, field).unwrap_or(false) {
            issues.push(format!("audit_trail_{field}_true"));
        }
    }
    for field in [
        "production_order_mutations_attempted",
        "listen_key_lifecycle_attempted",
    ] {
        if json_u64_value(audit_trail, field).unwrap_or(0) > 0 {
            issues.push(format!("audit_trail_{field}_nonzero"));
        }
    }
    issues
}

fn production_mutation_response_forbidden_markers(value: &serde_json::Value) -> Vec<String> {
    let mut markers = Vec::new();
    collect_production_mutation_response_forbidden_markers(value, "$", &mut markers);
    markers.sort();
    markers.dedup();
    markers
}

fn collect_production_mutation_response_forbidden_markers(
    value: &serde_json::Value,
    path: &str,
    markers: &mut Vec<String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}.{key}");
                if let Some(reason) = production_mutation_response_forbidden_key_reason(key) {
                    markers.push(format!("{child_path}:{reason}"));
                }
                collect_production_mutation_response_forbidden_markers(child, &child_path, markers);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_production_mutation_response_forbidden_markers(
                    child,
                    &format!("{path}[{index}]"),
                    markers,
                );
            }
        }
        serde_json::Value::String(text) => {
            if let Some(reason) = production_mutation_response_forbidden_text_reason(text) {
                markers.push(format!("{path}:{reason}"));
            }
        }
        _ => {}
    }
}

fn production_mutation_response_forbidden_key_reason(key: &str) -> Option<&'static str> {
    let normalized = key
        .chars()
        .filter(|ch| *ch != '_' && *ch != '-')
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if matches!(
        normalized.as_str(),
        "headers"
            | "header"
            | "apikey"
            | "apisecret"
            | "apikeyheader"
            | "signature"
            | "signedquery"
            | "signedurl"
            | "rawresponse"
            | "rawresponsebody"
            | "rawbody"
            | "responsebody"
            | "body"
            | "payload"
            | "balances"
            | "balance"
            | "account"
            | "fills"
            | "commission"
            | "commissionasset"
    ) {
        return Some("forbidden_response_key");
    }
    if normalized.contains("secret")
        || normalized.contains("signature")
        || normalized.contains("signedurl")
        || normalized.contains("signedquery")
        || normalized.contains("rawresponse")
        || normalized.contains("payload")
        || normalized.contains("balance")
    {
        return Some("forbidden_response_key");
    }
    None
}

fn production_mutation_response_forbidden_text_reason(text: &str) -> Option<&'static str> {
    let normalized = text.to_ascii_lowercase();
    if normalized.contains("signature=")
        || normalized.contains("x-mbx-apikey")
        || normalized.contains("api_secret")
        || normalized.contains("api-secret")
        || normalized.contains("signed_url")
        || normalized.contains("signed-query")
        || normalized.contains("raw response")
    {
        return Some("forbidden_response_text");
    }
    None
}

fn production_mutation_response_allowed_fields() -> Vec<String> {
    [
        "symbol",
        "side",
        "type",
        "timeInForce",
        "orderId",
        "clientOrderId",
        "status",
        "transactTime",
        "workingTime",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

fn production_mutation_response_time_shape(value: &serde_json::Value, field: &str) -> String {
    match value.get(field) {
        Some(serde_json::Value::Number(number)) if number.as_u64().is_some() => {
            "epoch_millis_present_redacted".to_string()
        }
        Some(serde_json::Value::String(text)) if !text.trim().is_empty() => {
            "epoch_millis_present_redacted".to_string()
        }
        _ => "missing".to_string(),
    }
}

fn production_live_alpha_kill_switch_runtime_gate_source_issues(
    kill_switch_approval: &serde_json::Value,
    risk_preflight: &serde_json::Value,
    request_preview: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(kill_switch_approval, "schema_version").as_deref()
        != Some(PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION)
    {
        issues.push("kill_switch_approval_schema_mismatch".to_string());
    }
    if !json_bool_value(kill_switch_approval, "kill_switch_enabled").unwrap_or(false) {
        issues.push("kill_switch_not_enabled".to_string());
    }
    if json_string_value(risk_preflight, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_REPORT_SCHEMA_VERSION)
    {
        issues.push("risk_preflight_schema_mismatch".to_string());
    }
    if json_string_value(risk_preflight, "risk_decision").as_deref() != Some("dry_run_approved") {
        issues.push("risk_preflight_not_dry_run_approved".to_string());
    }
    if json_bool_value(risk_preflight, "execution_adapter_called").unwrap_or(false) {
        issues.push("risk_preflight_touched_execution_adapter".to_string());
    }
    if json_string_value(request_preview, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION)
    {
        issues.push("request_preview_schema_mismatch".to_string());
    }
    if json_bool_value(request_preview, "network_attempted").unwrap_or(false) {
        issues.push("request_preview_attempted_network".to_string());
    }
    if json_bool_value(request_preview, "production_adapter_called").unwrap_or(false) {
        issues.push("request_preview_touched_production_adapter".to_string());
    }
    if artifact_has_production_mutation(Some(kill_switch_approval)) {
        issues.push("kill_switch_approval_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(risk_preflight)) {
        issues.push("risk_preflight_records_forbidden_production_mutation".to_string());
    }
    if artifact_has_production_mutation(Some(request_preview)) {
        issues.push("request_preview_records_forbidden_production_mutation".to_string());
    }
    issues
}

fn push_json_string_alignment_issue(
    issues: &mut Vec<String>,
    field: &str,
    left_label: &str,
    left: &serde_json::Value,
    right_label: &str,
    right: &serde_json::Value,
) {
    let left_value = json_string_value(left, field);
    let right_value = json_string_value(right, field);
    if left_value.is_some() && right_value.is_some() && left_value != right_value {
        issues.push(format!(
            "{field}_mismatch_between_{left_label}_and_{right_label}"
        ));
    }
}

fn build_production_live_alpha_risk_preflight_report(
    opt: &LiveProductionLiveAlphaRiskPreflightOpt,
) -> anyhow::Result<ProductionLiveAlphaRiskPreflightReport> {
    validate_non_empty("run_id", &opt.run_id)?;
    let order_gate = load_json_value(&opt.order_gate, "live-alpha dry-run order gate")?;
    let input: ProductionLiveAlphaRiskPreflightInput =
        load_json_file(&opt.input, "live-alpha risk preflight input")?;

    let missing_cli_flags = missing_production_live_alpha_risk_preflight_cli_flags(opt);
    let mut reasons = Vec::new();
    if !missing_cli_flags.is_empty() {
        reasons.push("missing_owner_dry_run_confirmation".to_string());
    }
    reasons.extend(evaluate_production_live_alpha_risk_reasons(
        &input,
        &order_gate,
    )?);

    let risk_decision = if reasons.is_empty() {
        "dry_run_approved"
    } else {
        "dry_run_rejected"
    };
    let status = if missing_cli_flags.is_empty() {
        if reasons.is_empty() {
            "approved"
        } else {
            "rejected"
        }
    } else {
        "blocked_missing_gate"
    };
    let order_gate_status =
        json_string_value(&order_gate, "status").unwrap_or_else(|| "unknown".to_string());
    let order_gate_ready =
        json_bool_value(&order_gate, "dry_run_order_gate_ready").unwrap_or(false);
    let market_age_ms = if input.market.now_unix_ms >= input.market.last_event_at_unix_ms {
        Some(input.market.now_unix_ms - input.market.last_event_at_unix_ms)
    } else {
        None
    };
    let order_state_age_ms = match (
        input.order_state.now_unix_ms,
        input.order_state.last_read_at_unix_ms,
    ) {
        (Some(now), Some(last_read_at)) => now.checked_sub(last_read_at),
        _ => None,
    };
    let current_position = parse_non_negative_decimal(&input.limits.current_position_notional)?;
    let order_notional = parse_non_negative_decimal(&input.order.notional)?;
    let projected_position_notional = format_decimal(&(current_position + order_notional));

    Ok(ProductionLiveAlphaRiskPreflightReport {
        schema_version: PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_REPORT_SCHEMA_VERSION.to_string(),
        status: status.to_string(),
        run_id: opt.run_id.clone(),
        evaluated_at: now_millis(),
        risk_decision: risk_decision.to_string(),
        execution_decision: "blocked_no_production_mutation".to_string(),
        reasons,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        order_gate_status,
        order_gate_ready,
        order_gate_path: opt.order_gate.display().to_string(),
        session_state: input.session.state,
        symbol: input.order.symbol,
        side: input.order.side,
        order_type: input.order.order_type,
        quantity: input.order.quantity,
        notional: input.order.notional,
        max_order_notional: input.limits.max_order_notional,
        current_position_notional: input.limits.current_position_notional,
        projected_position_notional,
        max_position_notional: input.limits.max_position_notional,
        market_age_ms,
        max_market_age_ms: input.market.max_age_ms,
        account_readable: input.account.readable,
        order_state_readable: input.order_state.readable,
        order_state_age_ms,
        max_order_state_age_ms: input.order_state.max_age_ms,
        open_order_count: input.order_state.open_order_count,
        max_open_orders: input.limits.max_open_orders,
        observed_clock_skew_ms: input.limits.observed_clock_skew_ms,
        max_clock_skew_ms: input.limits.max_clock_skew_ms,
        kill_switch_active: input.risk.kill_switch_active,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        order_endpoint_access_attempted: false,
        execution_adapter_called: false,
        matching_engine_submission: false,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        external_venue_connection: false,
        network_attempted: false,
        real_orders_submitted: false,
        real_funds: false,
        production_trading_enabled: false,
        order_state_values_are_exchange_truth: false,
        shadow_values_are_exchange_truth: false,
        portfolio_values_are_exchange_truth: false,
        values_are_exchange_truth: false,
        diagnostic: if risk_decision == "dry_run_approved" {
            "hypothetical live-alpha order passed local risk preflight; execution remains disabled"
        } else {
            "hypothetical live-alpha order rejected by local risk preflight; execution remains disabled"
        }
        .to_string(),
    })
}

fn evaluate_production_live_alpha_risk_reasons(
    input: &ProductionLiveAlphaRiskPreflightInput,
    order_gate: &serde_json::Value,
) -> anyhow::Result<Vec<String>> {
    let mut reasons = Vec::new();
    if input.schema_version != PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_INPUT_SCHEMA_VERSION {
        reasons.push("schema_version_mismatch".to_string());
    }
    if json_string_value(order_gate, "schema_version").as_deref()
        != Some(PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION)
    {
        reasons.push("order_gate_schema_mismatch".to_string());
    }
    if !json_bool_value(order_gate, "dry_run_order_gate_ready").unwrap_or(false) {
        reasons.push("order_gate_not_ready".to_string());
    }
    for field in [
        "production_orders_submitted",
        "production_order_mutations_attempted",
        "actual_submission_count",
    ] {
        if json_u64_value(order_gate, field).unwrap_or(0) != 0 {
            reasons.push(format!("order_gate_{field}_nonzero"));
        }
    }
    for field in [
        "execution_adapter_called",
        "order_endpoint_access_attempted",
        "dashboard_order_controls_enabled",
        "real_orders_submitted",
        "network_attempted",
    ] {
        if json_bool_value(order_gate, field).unwrap_or(false) {
            reasons.push(format!("order_gate_{field}_true"));
        }
    }
    if input.session.state != "running" {
        reasons.push("session_not_running".to_string());
    }
    if input.market.symbol != input.order.symbol {
        reasons.push("market_symbol_mismatch".to_string());
    }
    match input
        .market
        .now_unix_ms
        .checked_sub(input.market.last_event_at_unix_ms)
    {
        Some(age) if age > input.market.max_age_ms => reasons.push("market_stale".to_string()),
        Some(_) => {}
        None => reasons.push("market_event_in_future".to_string()),
    }
    if !input.account.readable {
        reasons.push("account_read_failed".to_string());
    }
    if input.account.readable && input.account.account_id.trim().is_empty() {
        reasons.push("account_id_missing".to_string());
    }
    if !input.order_state.readable {
        reasons.push("order_state_read_failed".to_string());
    }
    match (
        input.order_state.now_unix_ms,
        input.order_state.last_read_at_unix_ms,
        input.order_state.max_age_ms,
    ) {
        (Some(now), Some(last_read_at), Some(max_age)) => match now.checked_sub(last_read_at) {
            Some(age) if age > max_age => reasons.push("order_state_stale".to_string()),
            Some(_) => {}
            None => reasons.push("order_state_read_in_future".to_string()),
        },
        (None, None, None) => {}
        _ => reasons.push("order_state_freshness_incomplete".to_string()),
    }
    if input.risk.kill_switch_active {
        reasons.push("kill_switch_active".to_string());
    }
    if !input
        .risk
        .allowed_symbols
        .iter()
        .any(|symbol| symbol == &input.order.symbol)
    {
        reasons.push("symbol_not_allowlisted".to_string());
    }
    validate_positive_decimal_string("order.quantity", &input.order.quantity)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let order_notional = parse_non_negative_decimal(&input.order.notional)?;
    let max_order_notional = parse_non_negative_decimal(&input.limits.max_order_notional)?;
    if order_notional > max_order_notional {
        reasons.push("notional_limit_exceeded".to_string());
    }
    let current_position = parse_non_negative_decimal(&input.limits.current_position_notional)?;
    let max_position = parse_non_negative_decimal(&input.limits.max_position_notional)?;
    if current_position + order_notional > max_position {
        reasons.push("position_limit_exceeded".to_string());
    }
    if input.order_state.open_order_count >= input.limits.max_open_orders {
        reasons.push("open_order_limit_exceeded".to_string());
    }
    if input.limits.observed_clock_skew_ms > input.limits.max_clock_skew_ms {
        reasons.push("clock_skew_limit_exceeded".to_string());
    }
    Ok(reasons)
}

fn build_production_kill_switch_approval_artifact(
    opt: &LiveProductionKillSwitchApprovalArtifactOpt,
) -> anyhow::Result<ProductionKillSwitchApprovalArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("strategy_id", &opt.strategy_id)?;
    if !opt.confirm_dry_run_only {
        anyhow::bail!("--confirm-dry-run-only is required for v0.13 approval artifacts");
    }
    if !opt.confirm_no_production_mutation {
        anyhow::bail!("--confirm-no-production-mutation is required for v0.13 approval artifacts");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        anyhow::bail!(
            "--confirm-dashboard-order-controls-disabled is required for v0.13 approval artifacts"
        );
    }

    let session_id = opt
        .session_id
        .as_deref()
        .unwrap_or(opt.run_id.as_str())
        .to_string();
    validate_non_empty("session_id", &session_id)?;

    let approval_state = opt.approval_state.trim();
    if !matches!(approval_state, "pending" | "approved" | "rejected") {
        anyhow::bail!("approval_state must be pending, approved, or rejected");
    }

    let manual_approval_id = optional_non_empty("manual_approval_id", &opt.manual_approval_id)?;
    let approved_by = optional_non_empty("approved_by", &opt.approved_by)?;
    if approval_state == "approved" {
        if manual_approval_id.is_none() {
            anyhow::bail!("approval_state=approved requires --manual-approval-id");
        }
        if approved_by.is_none() {
            anyhow::bail!("approval_state=approved requires --approved-by");
        }
    }

    let manual_approval_recorded = manual_approval_id.is_some() && approved_by.is_some();
    let status = match approval_state {
        "approved" => "manual_approval_recorded",
        "rejected" => "manual_approval_rejected",
        _ => "manual_approval_pending",
    };

    Ok(ProductionKillSwitchApprovalArtifact {
        schema_version: PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        session_id,
        strategy_id: opt.strategy_id.clone(),
        artifact_type: "kill_switch_dry_run_manual_approval".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        kill_switch_enabled: true,
        kill_switch_active: opt.kill_switch_active,
        kill_switch_dry_run: true,
        kill_switch_state_source: "local_cli_dry_run".to_string(),
        manual_approval_required: true,
        manual_approval_recorded,
        manual_approval_id,
        approved_by,
        approval_state: approval_state.to_string(),
        approval_artifact_only: true,
        owner_approval_required_before_any_mutation: true,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        actual_submission_count: 0,
        automatic_correction_orders_submitted: 0,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: false,
        production_trading_enabled: false,
        network_attempted: false,
        order_state_values_are_exchange_truth: false,
        shadow_values_are_exchange_truth: false,
        portfolio_values_are_exchange_truth: false,
        values_are_exchange_truth: false,
        dry_run_confirmed: opt.confirm_dry_run_only,
        no_production_mutation_confirmed: opt.confirm_no_production_mutation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        diagnostic:
            "local guarded-live-alpha kill-switch/manual-approval artifact; no production mutation attempted"
                .to_string(),
    })
}

fn stop_file_exists(path: Option<&Path>) -> bool {
    path.is_some_and(Path::exists)
}

fn artifact_age_ms(path: &Path) -> anyhow::Result<Option<u64>> {
    let modified = fs::metadata(path)
        .with_context(|| format!("failed to inspect artifact '{}'", path.display()))?
        .modified()
        .with_context(|| format!("failed to read artifact mtime '{}'", path.display()))?;
    let age = match SystemTime::now().duration_since(modified) {
        Ok(duration) => u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
        Err(_) => 0,
    };
    Ok(Some(age))
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

    for pointer in [
        "/provenance/order_state_values_are_exchange_truth",
        "/provenance/shadow_values_are_exchange_truth",
        "/provenance/portfolio_values_are_exchange_truth",
        "/provenance/values_are_exchange_truth",
    ] {
        if value
            .pointer(pointer)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            anyhow::bail!(
                "shadow strategy session rejected portfolio runtime claiming exchange truth at {pointer}"
            );
        }
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
        order_state_values_are_exchange_truth: value
            .pointer("/provenance/order_state_values_are_exchange_truth")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        shadow_values_are_exchange_truth: value
            .pointer("/provenance/shadow_values_are_exchange_truth")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        portfolio_values_are_exchange_truth: value
            .pointer("/provenance/portfolio_values_are_exchange_truth")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_else(|| {
                value
                    .pointer("/provenance/values_are_exchange_truth")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            }),
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

fn build_production_readonly_reconciliation_event(
    opt: &LiveProductionReadonlyReconciliationOpt,
) -> anyhow::Result<ProductionReadonlyReconciliationEvent> {
    validate_non_empty("run_id", &opt.run_id)?;

    let account_snapshot =
        read_optional_json_artifact(opt.account_snapshot.as_deref(), "account snapshot");
    let shadow_portfolio = read_optional_json_artifact(
        opt.shadow_portfolio_runtime.as_deref(),
        "shadow portfolio runtime",
    );
    let shadow_strategy_session = read_optional_latest_jsonl_artifact(
        opt.shadow_strategy_session.as_deref(),
        "shadow strategy session",
    );
    let shadow_intent =
        read_optional_latest_jsonl_artifact(opt.shadow_intent.as_deref(), "shadow intent");

    let classification = classify_production_readonly_reconciliation(
        &account_snapshot,
        &shadow_portfolio,
        &shadow_strategy_session,
        &shadow_intent,
    );
    let (event_type, severity, recommended_action, risk_halted, manual_review_required) =
        reconciliation_classification_policy(classification);
    let diagnostic = reconciliation_diagnostic(
        classification,
        &account_snapshot,
        &shadow_portfolio,
        &shadow_strategy_session,
        &shadow_intent,
    );

    Ok(ProductionReadonlyReconciliationEvent {
        schema_version: PRODUCTION_READONLY_RECONCILIATION_EVENT_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        event_id: format!(
            "{}:{}",
            opt.run_id,
            reconciliation_classification_label(classification)
        ),
        event_type: event_type.to_string(),
        classification: reconciliation_classification_label(classification).to_string(),
        severity: severity.to_string(),
        observed_at: now_millis(),
        source_ref: ReadonlyReconciliationSourceRef {
            engine: "production_readonly_reconciliation".to_string(),
            mode: "local_shadow_artifact_classification".to_string(),
            network_attempted: false,
        },
        account_snapshot_ref: reconciliation_artifact_ref(
            opt.account_snapshot.as_deref(),
            &account_snapshot,
        ),
        shadow_portfolio_ref: reconciliation_artifact_ref(
            opt.shadow_portfolio_runtime.as_deref(),
            &shadow_portfolio,
        ),
        shadow_strategy_session_ref: reconciliation_artifact_ref(
            opt.shadow_strategy_session.as_deref(),
            &shadow_strategy_session,
        ),
        shadow_intent_ref: reconciliation_artifact_ref(
            opt.shadow_intent.as_deref(),
            &shadow_intent,
        ),
        recommended_action: recommended_action.to_string(),
        risk_halted,
        new_orders_blocked: true,
        manual_review_required,
        automatic_correction_orders_submitted: 0,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        cancel_replace_amend_attempted: false,
        dashboard_order_controls_enabled: false,
        real_orders_submitted: false,
        order_state_values_are_exchange_truth: false,
        shadow_values_are_exchange_truth: false,
        portfolio_values_are_exchange_truth: false,
        values_are_exchange_truth: false,
        diagnostic,
    })
}

fn write_production_readonly_reconciliation_events(
    path: &Path,
    events: &[ProductionReadonlyReconciliationEvent],
) -> anyhow::Result<()> {
    if events.is_empty() {
        anyhow::bail!("production read-only reconciliation must write at least one event");
    }
    let mut body = String::new();
    for event in events {
        body.push_str(&serde_json::to_string(event)?);
        body.push('\n');
    }
    atomic_write_text(path, &body).with_context(|| {
        format!(
            "failed to write production read-only reconciliation events '{}'",
            path.display()
        )
    })
}

fn classify_production_readonly_reconciliation(
    account_snapshot: &OptionalJsonArtifact,
    shadow_portfolio: &OptionalJsonArtifact,
    shadow_strategy_session: &OptionalJsonArtifact,
    shadow_intent: &OptionalJsonArtifact,
) -> ReadonlyReconciliationClassification {
    if [
        account_snapshot,
        shadow_portfolio,
        shadow_strategy_session,
        shadow_intent,
    ]
    .iter()
    .any(|artifact| artifact_has_production_mutation(artifact.value.as_ref()))
    {
        return ReadonlyReconciliationClassification::ProductionMutationForbidden;
    }

    if shadow_intent.value.is_some() && shadow_portfolio.value.is_none() {
        return ReadonlyReconciliationClassification::ShadowIntentWithoutPortfolio;
    }

    if account_snapshot.value.is_none() {
        return ReadonlyReconciliationClassification::MissingAccountSnapshot;
    }

    if shadow_portfolio.value.is_none() {
        return ReadonlyReconciliationClassification::PortfolioUnavailable;
    }

    if shadow_strategy_session
        .value
        .as_ref()
        .is_some_and(artifact_requires_manual_review)
        || shadow_portfolio
            .value
            .as_ref()
            .is_some_and(artifact_requires_manual_review)
    {
        return ReadonlyReconciliationClassification::ManualReviewRequired;
    }

    ReadonlyReconciliationClassification::Ok
}

fn reconciliation_classification_policy(
    classification: ReadonlyReconciliationClassification,
) -> (&'static str, &'static str, &'static str, bool, bool) {
    match classification {
        ReadonlyReconciliationClassification::Ok => (
            "observed_account_state",
            "info",
            "record_only",
            false,
            false,
        ),
        ReadonlyReconciliationClassification::MissingAccountSnapshot => {
            ("degraded_status", "degraded", "mark_degraded", true, true)
        }
        ReadonlyReconciliationClassification::PortfolioUnavailable => {
            ("degraded_status", "degraded", "mark_degraded", true, true)
        }
        ReadonlyReconciliationClassification::ShadowIntentWithoutPortfolio => (
            "shadow_mismatch",
            "halt",
            "manual_review_required",
            true,
            true,
        ),
        ReadonlyReconciliationClassification::ProductionMutationForbidden => {
            ("risk_halt", "halt", "halt_shadow_flow", true, true)
        }
        ReadonlyReconciliationClassification::ManualReviewRequired => (
            "manual_remediation_required",
            "warning",
            "manual_review_required",
            true,
            true,
        ),
    }
}

fn reconciliation_classification_label(
    classification: ReadonlyReconciliationClassification,
) -> &'static str {
    match classification {
        ReadonlyReconciliationClassification::Ok => "ok",
        ReadonlyReconciliationClassification::MissingAccountSnapshot => "missing_account_snapshot",
        ReadonlyReconciliationClassification::PortfolioUnavailable => "portfolio_unavailable",
        ReadonlyReconciliationClassification::ShadowIntentWithoutPortfolio => {
            "shadow_intent_without_portfolio"
        }
        ReadonlyReconciliationClassification::ProductionMutationForbidden => {
            "production_mutation_forbidden"
        }
        ReadonlyReconciliationClassification::ManualReviewRequired => "manual_review_required",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OptionalJsonArtifact {
    status: String,
    value: Option<serde_json::Value>,
    diagnostic: String,
}

fn read_optional_json_artifact(path: Option<&Path>, label: &str) -> OptionalJsonArtifact {
    let Some(path) = path else {
        return OptionalJsonArtifact {
            status: "not_provided".to_string(),
            value: None,
            diagnostic: format!("{label} artifact path was not provided"),
        };
    };

    match read_json_artifact(path, label) {
        Ok(value) => OptionalJsonArtifact {
            status: "available".to_string(),
            value: Some(value),
            diagnostic: format!("{label} artifact available"),
        },
        Err(error) => OptionalJsonArtifact {
            status: "missing_or_unreadable".to_string(),
            value: None,
            diagnostic: format!("{label} artifact unavailable: {error}"),
        },
    }
}

fn read_optional_latest_jsonl_artifact(path: Option<&Path>, label: &str) -> OptionalJsonArtifact {
    let Some(path) = path else {
        return OptionalJsonArtifact {
            status: "not_provided".to_string(),
            value: None,
            diagnostic: format!("{label} artifact path was not provided"),
        };
    };

    match read_latest_jsonl_artifact(path, label) {
        Ok(value) => OptionalJsonArtifact {
            status: "available".to_string(),
            value: Some(value),
            diagnostic: format!("{label} artifact available"),
        },
        Err(error) => OptionalJsonArtifact {
            status: "missing_or_unreadable".to_string(),
            value: None,
            diagnostic: format!("{label} artifact unavailable: {error}"),
        },
    }
}

fn read_latest_jsonl_artifact(path: &Path, label: &str) -> anyhow::Result<serde_json::Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {label} artifact '{}'", path.display()))?;
    let latest = raw
        .lines()
        .rfind(|line| !line.trim().is_empty())
        .with_context(|| format!("{label} artifact '{}' has no JSONL records", path.display()))?;
    serde_json::from_str(latest)
        .with_context(|| format!("failed to parse latest {label} JSONL '{}'", path.display()))
}

fn reconciliation_artifact_ref(
    path: Option<&Path>,
    artifact: &OptionalJsonArtifact,
) -> ReadonlyReconciliationArtifactRef {
    ReadonlyReconciliationArtifactRef {
        path: path.map(|path| path.display().to_string()),
        status: artifact.status.clone(),
        schema_version: artifact
            .value
            .as_ref()
            .and_then(|value| json_string_value(value, "schema_version")),
        record_count: artifact
            .value
            .as_ref()
            .and_then(|value| json_u64_value(value, "record_count")),
        classification: artifact
            .value
            .as_ref()
            .and_then(|value| json_string_value(value, "classification")),
        diagnostic: artifact.diagnostic.clone(),
    }
}

fn artifact_has_production_mutation(value: Option<&serde_json::Value>) -> bool {
    let Some(value) = value else {
        return false;
    };
    [
        "actual_submission_count",
        "production_order_submissions_attempted",
        "production_orders_submitted",
        "production_order_mutations_attempted",
        "production_order_state_reads_attempted",
        "listen_key_lifecycle_attempted",
        "automatic_correction_orders_submitted",
    ]
    .into_iter()
    .any(|field| json_u64_value(value, field).unwrap_or(0) > 0)
        || [
            "actual_submission",
            "cancel_replace_amend_attempted",
            "dashboard_order_controls_enabled",
            "real_orders_submitted",
        ]
        .into_iter()
        .any(|field| json_bool_value(value, field).unwrap_or(false))
        || value
            .pointer("/provenance/values_are_exchange_truth")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
}

fn artifact_requires_manual_review(value: &serde_json::Value) -> bool {
    json_string_value(value, "classification").as_deref() == Some("manual_review_required")
        || json_string_value(value, "severity")
            .as_deref()
            .is_some_and(|severity| matches!(severity, "warning" | "degraded" | "halt"))
        || json_string_value(value, "state")
            .as_deref()
            .is_some_and(|state| state.contains("degraded") || state.contains("halt"))
        || json_string_value(value, "status")
            .as_deref()
            .is_some_and(|status| status.contains("degraded") || status.contains("unavailable"))
        || value.get("artifact_gap").is_some()
}

fn reconciliation_diagnostic(
    classification: ReadonlyReconciliationClassification,
    account_snapshot: &OptionalJsonArtifact,
    shadow_portfolio: &OptionalJsonArtifact,
    shadow_strategy_session: &OptionalJsonArtifact,
    shadow_intent: &OptionalJsonArtifact,
) -> String {
    match classification {
        ReadonlyReconciliationClassification::Ok => {
            "read-only reconciliation classified local shadow evidence as ok; record only".to_string()
        }
        ReadonlyReconciliationClassification::MissingAccountSnapshot => {
            format!("missing account snapshot: {}", account_snapshot.diagnostic)
        }
        ReadonlyReconciliationClassification::PortfolioUnavailable => {
            format!("shadow portfolio unavailable: {}", shadow_portfolio.diagnostic)
        }
        ReadonlyReconciliationClassification::ShadowIntentWithoutPortfolio => format!(
            "shadow intent present without portfolio runtime: {}; {}",
            shadow_intent.diagnostic, shadow_portfolio.diagnostic
        ),
        ReadonlyReconciliationClassification::ProductionMutationForbidden => {
            "input artifact records a forbidden production mutation or exchange-truth claim; local shadow flow must halt".to_string()
        }
        ReadonlyReconciliationClassification::ManualReviewRequired => format!(
            "manual review required from shadow evidence: {}; {}",
            shadow_strategy_session.diagnostic, shadow_portfolio.diagnostic
        ),
    }
}

fn json_string_value(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn json_scalar_string_value(value: &serde_json::Value, field: &str) -> Option<String> {
    match value.get(field) {
        Some(serde_json::Value::String(text)) => Some(text.clone()),
        Some(serde_json::Value::Number(number)) => Some(number.to_string()),
        Some(serde_json::Value::Bool(flag)) => Some(flag.to_string()),
        _ => None,
    }
}

fn json_string_array(value: &serde_json::Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn json_bool_value(value: &serde_json::Value, field: &str) -> Option<bool> {
    value.get(field).and_then(serde_json::Value::as_bool)
}

fn json_u64_value(value: &serde_json::Value, field: &str) -> Option<u64> {
    value.get(field).and_then(serde_json::Value::as_u64)
}

fn file_fnv1a64_hash(path: &str) -> String {
    match fs::read(path) {
        Ok(bytes) => format!("fnv1a64:{:016x}", fnv1a64(&bytes)),
        Err(_) => "unavailable".to_string(),
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn parse_non_negative_decimal(value: &str) -> anyhow::Result<Decimal> {
    let value = value.trim();
    if value.is_empty() {
        anyhow::bail!("decimal value must not be empty");
    }
    if value.starts_with('-') {
        anyhow::bail!("decimal value must not be negative");
    }
    if value.matches('.').count() > 1
        || !value.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        || !value.chars().any(|ch| ch.is_ascii_digit())
    {
        anyhow::bail!("decimal value must be a plain decimal string");
    }
    value
        .parse::<Decimal>()
        .context("decimal value must parse as Decimal")
}

fn format_decimal(value: &Decimal) -> String {
    value.normalize().to_string()
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
            output_dir: &output_dir,
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

fn load_json_file<T>(path: &Path, label: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {label} '{}'", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {label} '{}'", path.display()))
}

fn load_json_value(path: &Path, label: &str) -> anyhow::Result<serde_json::Value> {
    load_json_file(path, label)
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

struct ProductionLiveAlphaOrderRequestInput<'a> {
    endpoint_path: &'a str,
    symbol: &'a str,
    side: &'a str,
    order_type: &'a str,
    quantity: &'a str,
    price: &'a str,
    time_in_force: &'a str,
    recv_window_ms: u64,
    timestamp_ms: u64,
}

fn build_production_live_alpha_signed_order_request_preview(
    input: &ProductionLiveAlphaOrderRequestInput<'_>,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> anyhow::Result<ProductionLiveAlphaSignedOrderRequestPreview> {
    let signing_credential = credentials.signing_credential()?;
    let query_without_signature = build_production_live_alpha_order_query(input);
    let signature =
        urlencoding::encode(&signing_credential.sign(&query_without_signature)).into_owned();
    let signed_query = format!("{query_without_signature}&signature={signature}");
    let request = ProductionLiveAlphaSignedOrderRequestPreview {
        method: "POST".to_string(),
        endpoint_path: input.endpoint_path.to_string(),
        endpoint_url_redacted: format!("{BINANCE_PRODUCTION_HTTP_BASE_URL}{}", input.endpoint_path),
        query_without_signature,
        signature,
        signed_query,
        api_key_header_name: BINANCE_API_KEY_HEADER.to_string(),
        api_key_header_value: signing_credential.api_key().to_string(),
    };
    request.ensure_memory_only_redacted(credentials)?;
    Ok(request)
}

fn build_production_live_alpha_order_query(
    input: &ProductionLiveAlphaOrderRequestInput<'_>,
) -> String {
    join_query_pair_vec(&[
        ("symbol".to_string(), input.symbol.to_string()),
        ("side".to_string(), input.side.to_string()),
        ("type".to_string(), input.order_type.to_string()),
        ("timeInForce".to_string(), input.time_in_force.to_string()),
        ("quantity".to_string(), input.quantity.to_string()),
        ("price".to_string(), input.price.to_string()),
        ("recvWindow".to_string(), input.recv_window_ms.to_string()),
        ("timestamp".to_string(), input.timestamp_ms.to_string()),
    ])
}

fn production_live_alpha_order_query_shape_without_signature() -> String {
    [
        "symbol",
        "side",
        "type",
        "timeInForce",
        "quantity",
        "price",
        "recvWindow",
        "timestamp",
    ]
    .join("&")
}

fn execute_production_mutation_guarded_send(
    request: &ProductionLiveAlphaSignedOrderRequestPreview,
) -> ProductionMutationGuardedSendHttpResult {
    std::thread::spawn({
        let endpoint_url = request.endpoint_url_redacted.clone();
        let signed_query = request.signed_query.clone();
        let api_key_header_name = request.api_key_header_name.clone();
        let api_key_header_value = request.api_key_header_value.clone();
        move || {
            execute_production_mutation_guarded_send_on_thread(
                &endpoint_url,
                &signed_query,
                &api_key_header_name,
                &api_key_header_value,
            )
        }
    })
    .join()
    .unwrap_or_else(|_| {
        ProductionMutationGuardedSendHttpResult::pre_http_failure("http_send_thread_panicked")
    })
}

fn execute_production_mutation_guarded_send_on_thread(
    endpoint_url: &str,
    signed_query: &str,
    api_key_header_name: &str,
    api_key_header_value: &str,
) -> ProductionMutationGuardedSendHttpResult {
    let started = Instant::now();
    let client = match reqwest::blocking::Client::builder()
        .timeout(PRODUCTION_ORDER_STATE_PROBE_TIMEOUT)
        .user_agent("NTPRO-v160-production-mutation-guarded-send")
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return ProductionMutationGuardedSendHttpResult::pre_http_failure(
                "http_client_build_failed",
            );
        }
    };

    let signed_url = format!("{endpoint_url}?{signed_query}");
    match client
        .post(signed_url)
        .header(api_key_header_name, api_key_header_value)
        .send()
    {
        Ok(response) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            let status = response.status().as_u16();
            if response.status().is_success() {
                ProductionMutationGuardedSendHttpResult::success(latency_ms, status)
            } else {
                ProductionMutationGuardedSendHttpResult::failure(
                    Some(latency_ms),
                    Some(status),
                    "http_status_not_success",
                )
            }
        }
        Err(error) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            ProductionMutationGuardedSendHttpResult::failure(
                Some(latency_ms),
                error.status().map(|status| status.as_u16()),
                classify_production_public_read_error(&error),
            )
        }
    }
}

fn normalize_production_live_alpha_order_endpoint_path(
    endpoint_path: &str,
) -> anyhow::Result<String> {
    let endpoint_path = endpoint_path.trim();
    if endpoint_path.is_empty() {
        anyhow::bail!("production live-alpha request preview endpoint must not be empty");
    }
    if endpoint_path.contains('?') {
        anyhow::bail!(
            "production live-alpha request preview endpoint must not include query parameters"
        );
    }
    if !endpoint_path.starts_with('/') {
        anyhow::bail!("production live-alpha request preview endpoint must start with '/'");
    }
    match endpoint_path {
        TESTNET_ORDER_ENDPOINT_ORDER => Ok(endpoint_path.to_string()),
        _ => anyhow::bail!(
            "production live-alpha request preview allowlist only includes POST /api/v3/order; got POST {endpoint_path}"
        ),
    }
}

fn endpoint_decision_label(decision: EndpointDecision) -> &'static str {
    match decision {
        EndpointDecision::AllowReadOnly => "allow_read_only",
        EndpointDecision::AllowRequestPreviewOnly => "allow_request_preview_only",
        EndpointDecision::Deny => "deny",
    }
}

fn required_json_string(value: &serde_json::Value, key: &str) -> anyhow::Result<String> {
    json_string_value(value, key).with_context(|| format!("missing or non-string JSON field {key}"))
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

fn join_query_pair_vec(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
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

fn write_production_order_state_readonly_report(
    path: &Path,
    value: &ProductionOrderStateReadOnlyProofReport,
    credentials: &EnvOnlyProductionReadCredentials,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    credentials.ensure_no_secret_values_absent(&path.display().to_string(), &body)?;
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_live_alpha_order_request_preview_report(
    path: &Path,
    value: &ProductionLiveAlphaOrderRequestPreviewArtifact,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    credentials.ensure_no_secret_values_absent(&path.display().to_string(), &body)?;
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_request_builder_artifact(
    path: &Path,
    value: &ProductionMutationRequestBuilderArtifact,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    credentials.ensure_no_secret_values_absent(&path.display().to_string(), &body)?;
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_guarded_send_artifact(
    path: &Path,
    value: &ProductionMutationGuardedSendArtifact,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    credentials.ensure_no_secret_values_absent(&path.display().to_string(), &body)?;
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_response_redaction_artifact(
    path: &Path,
    value: &ProductionMutationResponseRedactionArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_order_state_readback_artifact(
    path: &Path,
    value: &ProductionMutationOrderStateReadbackArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_audit_trail_artifact(
    path: &Path,
    value: &ProductionMutationAuditTrailArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_failure_semantics_artifact(
    path: &Path,
    value: &ProductionMutationFailureSemanticsArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
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

fn missing_production_order_state_cli_flags(
    opt: &LiveProductionOrderStateReadOnlyProofOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_order_state_read {
        missing.push("--allow-production-order-state-read");
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
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    missing
}

fn missing_production_live_alpha_dry_run_order_gate_cli_flags(
    opt: &LiveProductionLiveAlphaDryRunOrderGateOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_live_alpha_dry_run {
        missing.push("--allow-production-live-alpha-dry-run");
    }
    if !opt.confirm_owner_approved_dry_run {
        missing.push("--confirm-owner-approved-dry-run");
    }
    if !opt.confirm_no_production_order_submission {
        missing.push("--confirm-no-production-order-submission");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_no_execution_adapter_call {
        missing.push("--confirm-no-execution-adapter-call");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_real_funds {
        missing.push("--confirm-no-real-funds");
    }
    missing
}

fn missing_production_live_alpha_order_request_preview_cli_flags(
    opt: &LiveProductionLiveAlphaOrderRequestPreviewOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_live_alpha_request_preview {
        missing.push("--allow-production-live-alpha-request-preview");
    }
    if !opt.confirm_owner_approved_request_preview {
        missing.push("--confirm-owner-approved-request-preview");
    }
    if !opt.confirm_memory_only_signature {
        missing.push("--confirm-memory-only-signature");
    }
    if !opt.confirm_no_production_order_submission {
        missing.push("--confirm-no-production-order-submission");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_no_execution_adapter_call {
        missing.push("--confirm-no-execution-adapter-call");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_real_funds {
        missing.push("--confirm-no-real-funds");
    }
    missing
}

fn missing_production_live_alpha_order_request_preview_env_vars(
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> Vec<String> {
    let mut missing = credentials
        .production_signing_material_missing_gate_env_vars
        .clone();
    if !credentials.production_signing_material_gate_open
        && credentials.production_signing_material_gate_required
    {
        return missing;
    }
    if credentials
        .api_key_value
        .as_deref()
        .is_none_or(|value| value.is_empty())
    {
        missing.push(credentials.api_key_env.clone());
    }
    if credentials
        .api_secret_value
        .as_deref()
        .is_none_or(|value| value.is_empty())
    {
        missing.push(credentials.api_secret_env.clone());
    }
    missing
}

fn missing_production_live_alpha_execution_dry_run_cli_flags(
    opt: &LiveProductionLiveAlphaExecutionDryRunOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_live_alpha_execution_dry_run {
        missing.push("--allow-production-live-alpha-execution-dry-run");
    }
    if !opt.confirm_owner_approved_execution_dry_run {
        missing.push("--confirm-owner-approved-execution-dry-run");
    }
    if !opt.confirm_dry_run_adapter_only {
        missing.push("--confirm-dry-run-adapter-only");
    }
    if !opt.confirm_no_production_adapter {
        missing.push("--confirm-no-production-adapter");
    }
    if !opt.confirm_no_production_order_submission {
        missing.push("--confirm-no-production-order-submission");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_real_funds {
        missing.push("--confirm-no-real-funds");
    }
    missing
}

fn missing_production_live_alpha_kill_switch_runtime_gate_cli_flags(
    opt: &LiveProductionLiveAlphaKillSwitchRuntimeGateOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_live_alpha_kill_switch_runtime_gate {
        missing.push("--allow-production-live-alpha-kill-switch-runtime-gate");
    }
    if !opt.confirm_owner_approved_runtime_gate {
        missing.push("--confirm-owner-approved-runtime-gate");
    }
    if !opt.confirm_no_production_order_submission {
        missing.push("--confirm-no-production-order-submission");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_real_funds {
        missing.push("--confirm-no-real-funds");
    }
    missing
}

fn missing_production_mutation_runtime_gate_cli_flags(
    opt: &LiveProductionMutationRuntimeGateOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_runtime_gate {
        missing.push("--allow-production-mutation-runtime-gate");
    }
    if !opt.confirm_owner_approved_production_mutation {
        missing.push("--confirm-owner-approved-production-mutation");
    }
    if !opt.confirm_single_limit_gtc {
        missing.push("--confirm-single-limit-gtc");
    }
    if !opt.confirm_tiny_notional {
        missing.push("--confirm-tiny-notional");
    }
    if !opt.confirm_signing_approval_required {
        missing.push("--confirm-signing-approval-required");
    }
    if !opt.confirm_no_network_before_send {
        missing.push("--confirm-no-network-before-send");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    missing
}

fn missing_production_mutation_signing_approval_cli_flags(
    opt: &LiveProductionMutationSigningApprovalOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_signing_approval {
        missing.push("--allow-production-mutation-signing-approval");
    }
    if !opt.confirm_owner_approved_signing_material {
        missing.push("--confirm-owner-approved-signing-material");
    }
    if !opt.confirm_env_only_signing_material {
        missing.push("--confirm-env-only-signing-material");
    }
    if !opt.confirm_memory_only_signing {
        missing.push("--confirm-memory-only-signing");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_production_order_submission {
        missing.push("--confirm-no-production-order-submission");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    missing
}

fn missing_production_mutation_request_builder_cli_flags(
    opt: &LiveProductionMutationRequestBuilderOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_request_builder {
        missing.push("--allow-production-mutation-request-builder");
    }
    if !opt.confirm_owner_approved_request_builder {
        missing.push("--confirm-owner-approved-request-builder");
    }
    if !opt.confirm_single_limit_gtc {
        missing.push("--confirm-single-limit-gtc");
    }
    if !opt.confirm_tiny_notional {
        missing.push("--confirm-tiny-notional");
    }
    if !opt.confirm_signing_approval_ready {
        missing.push("--confirm-signing-approval-ready");
    }
    if !opt.confirm_memory_only_signing {
        missing.push("--confirm-memory-only-signing");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_production_order_submission {
        missing.push("--confirm-no-production-order-submission");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    missing
}

fn production_mutation_request_builder_missing_env_vars(
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> Vec<String> {
    let mut missing = credentials
        .production_signing_material_missing_gate_env_vars
        .clone();
    if credentials.production_signing_material_gate_open {
        if credentials
            .api_key_value
            .as_deref()
            .is_none_or(str::is_empty)
        {
            missing.push(credentials.api_key_env.clone());
        }
        if credentials
            .api_secret_value
            .as_deref()
            .is_none_or(str::is_empty)
        {
            missing.push(credentials.api_secret_env.clone());
        }
    }
    missing
}

fn missing_production_mutation_guarded_send_cli_flags(
    opt: &LiveProductionMutationGuardedSendOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_guarded_send {
        missing.push("--allow-production-mutation-guarded-send");
    }
    if !opt.confirm_owner_approved_guarded_send {
        missing.push("--confirm-owner-approved-guarded-send");
    }
    if !opt.confirm_single_limit_gtc {
        missing.push("--confirm-single-limit-gtc");
    }
    if !opt.confirm_tiny_notional {
        missing.push("--confirm-tiny-notional");
    }
    if !opt.confirm_single_shot {
        missing.push("--confirm-single-shot");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    if !opt.confirm_response_redacted {
        missing.push("--confirm-response-redacted");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    missing
}

fn production_mutation_guarded_send_missing_env_vars(
    opt: &LiveProductionMutationGuardedSendOpt,
    credentials: &EnvOnlyProductionMutationPreviewCredentials,
) -> Vec<String> {
    if !opt.manual_online {
        return Vec::new();
    }
    let mut missing = credentials
        .production_signing_material_missing_gate_env_vars
        .clone();
    if credentials.production_signing_material_gate_open {
        if credentials
            .api_key_value
            .as_deref()
            .is_none_or(str::is_empty)
        {
            missing.push(credentials.api_key_env.clone());
        }
        if credentials
            .api_secret_value
            .as_deref()
            .is_none_or(str::is_empty)
        {
            missing.push(credentials.api_secret_env.clone());
        }
    }
    missing
}

fn missing_production_mutation_response_redaction_cli_flags(
    opt: &LiveProductionMutationResponseRedactionOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_response_redaction {
        missing.push("--allow-production-mutation-response-redaction");
    }
    if !opt.confirm_owner_approved_response_redaction {
        missing.push("--confirm-owner-approved-response-redaction");
    }
    if !opt.confirm_no_raw_response_persistence {
        missing.push("--confirm-no-raw-response-persistence");
    }
    if !opt.confirm_no_headers_persistence {
        missing.push("--confirm-no-headers-persistence");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    if !opt.confirm_order_metadata_only {
        missing.push("--confirm-order-metadata-only");
    }
    if !opt.confirm_no_account_balances {
        missing.push("--confirm-no-account-balances");
    }
    if !opt.confirm_no_unrestricted_payload {
        missing.push("--confirm-no-unrestricted-payload");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    missing
}

fn missing_production_mutation_order_state_readback_cli_flags(
    opt: &LiveProductionMutationOrderStateReadbackOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_order_state_readback {
        missing.push("--allow-production-mutation-order-state-readback");
    }
    if !opt.confirm_owner_approved_order_state_readback {
        missing.push("--confirm-owner-approved-order-state-readback");
    }
    if !opt.confirm_known_order_identifier_only {
        missing.push("--confirm-known-order-identifier-only");
    }
    if !opt.confirm_read_only_get_order {
        missing.push("--confirm-read-only-get-order");
    }
    if !opt.confirm_response_redacted {
        missing.push("--confirm-response-redacted");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    missing
}

fn missing_production_mutation_audit_trail_cli_flags(
    opt: &LiveProductionMutationAuditTrailOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_audit_trail {
        missing.push("--allow-production-mutation-audit-trail");
    }
    if !opt.confirm_owner_approved_audit_trail {
        missing.push("--confirm-owner-approved-audit-trail");
    }
    if !opt.confirm_redacted_artifacts_only {
        missing.push("--confirm-redacted-artifacts-only");
    }
    if !opt.confirm_no_secret_or_raw_payload_persistence {
        missing.push("--confirm-no-secret-or-raw-payload-persistence");
    }
    if !opt.confirm_no_retry_or_followup_mutation {
        missing.push("--confirm-no-retry-or-followup-mutation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    missing
}

fn missing_production_mutation_failure_semantics_cli_flags(
    opt: &LiveProductionMutationFailureSemanticsOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_failure_semantics {
        missing.push("--allow-production-mutation-failure-semantics");
    }
    if !opt.confirm_evidence_only_failure_handling {
        missing.push("--confirm-evidence-only-failure-handling");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_automatic_cancel_replace_amend {
        missing.push("--confirm-no-automatic-cancel-replace-amend");
    }
    if !opt.confirm_no_correction_or_flatten {
        missing.push("--confirm-no-correction-or-flatten");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_strategy_continuation {
        missing.push("--confirm-no-strategy-continuation");
    }
    if !opt.confirm_no_listen_key_lifecycle {
        missing.push("--confirm-no-listen-key-lifecycle");
    }
    missing
}

fn production_mutation_order_state_readback_missing_env_vars<F>(
    opt: &LiveProductionMutationOrderStateReadbackOpt,
    read_env: &mut F,
) -> Vec<&'static str>
where
    F: FnMut(&str) -> Option<String>,
{
    if !opt.manual_online {
        return Vec::new();
    }
    let mut missing: Vec<&'static str> = [
        PRODUCTION_ORDER_STATE_ENV_ALLOW,
        PRODUCTION_ORDER_STATE_ENV_OWNER_APPROVED,
        PRODUCTION_ORDER_STATE_ENV_NO_ORDER_MUTATION,
        PRODUCTION_ORDER_STATE_ENV_NO_SECRET_PERSISTENCE,
        PRODUCTION_ORDER_STATE_ENV_NO_LISTEN_KEY,
        PRODUCTION_ORDER_STATE_ENV_DASHBOARD_DISABLED,
        PRODUCTION_ORDER_STATE_ENV_MANUAL_ONLINE,
    ]
    .into_iter()
    .filter(|name| read_env(name).as_deref() != Some("1"))
    .collect();
    if read_env(&opt.api_key_env)
        .as_deref()
        .is_none_or(str::is_empty)
    {
        missing.push("api_key_env_value");
    }
    if read_env(&opt.api_secret_env)
        .as_deref()
        .is_none_or(str::is_empty)
    {
        missing.push("api_secret_env_value");
    }
    missing
}

fn missing_production_live_alpha_risk_preflight_cli_flags(
    opt: &LiveProductionLiveAlphaRiskPreflightOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.confirm_hypothetical_dry_run_only {
        missing.push("--confirm-hypothetical-dry-run-only");
    }
    if !opt.confirm_no_execution_adapter_call {
        missing.push("--confirm-no-execution-adapter-call");
    }
    if !opt.confirm_no_production_order_submission {
        missing.push("--confirm-no-production-order-submission");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    missing
}

fn missing_production_order_state_env_gates<F>(
    read_env: &mut F,
    manual_online_requested: bool,
) -> Vec<&'static str>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut missing: Vec<&'static str> = [
        PRODUCTION_ORDER_STATE_ENV_ALLOW,
        PRODUCTION_ORDER_STATE_ENV_OWNER_APPROVED,
        PRODUCTION_ORDER_STATE_ENV_NO_ORDER_MUTATION,
        PRODUCTION_ORDER_STATE_ENV_NO_SECRET_PERSISTENCE,
        PRODUCTION_ORDER_STATE_ENV_NO_LISTEN_KEY,
        PRODUCTION_ORDER_STATE_ENV_DASHBOARD_DISABLED,
    ]
    .into_iter()
    .filter(|name| read_env(name).as_deref() != Some("1"))
    .collect();
    if manual_online_requested
        && read_env(PRODUCTION_ORDER_STATE_ENV_MANUAL_ONLINE).as_deref() != Some("1")
    {
        missing.push(PRODUCTION_ORDER_STATE_ENV_MANUAL_ONLINE);
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
    output_dir: &'a Path,
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
        kill_switch_approval_artifact_path: context
            .output_dir
            .join("v0_13")
            .join("kill_switch_approval_artifact.json"),
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
        kill_switch_approval_artifact_path: paths
            .output_dir
            .join("v0_13")
            .join("kill_switch_approval_artifact.json"),
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
                    output_dir,
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

fn optional_non_empty(field: &str, value: &Option<String>) -> anyhow::Result<Option<String>> {
    value
        .as_ref()
        .map(|raw| {
            validate_non_empty(field, raw)?;
            Ok(raw.trim().to_string())
        })
        .transpose()
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

    fn production_order_state_readonly_proof_opt(
        endpoint: ProductionOrderStateReadEndpoint,
        output: Option<PathBuf>,
        all_cli_gates: bool,
        manual_online: bool,
    ) -> LiveProductionOrderStateReadOnlyProofOpt {
        LiveProductionOrderStateReadOnlyProofOpt {
            endpoint,
            symbol: "BTCUSDT".to_string(),
            order_id: (endpoint == ProductionOrderStateReadEndpoint::Order).then_some(12_345),
            orig_client_order_id: None,
            output,
            manual_online,
            api_key_env: "NTPRO_V140001_API_KEY".to_string(),
            api_secret_env: "NTPRO_V140001_API_SECRET".to_string(),
            recv_window_ms: 5_000,
            allow_production_order_state_read: all_cli_gates,
            confirm_owner_approved_read_only: all_cli_gates,
            confirm_no_order_mutation: all_cli_gates,
            confirm_no_secret_persistence: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
        }
    }

    fn production_live_alpha_dry_run_order_gate_opt(
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionLiveAlphaDryRunOrderGateOpt {
        LiveProductionLiveAlphaDryRunOrderGateOpt {
            run_id: "v140-live-alpha-dry-run".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: "0.001".to_string(),
            notional: "10.00".to_string(),
            output,
            allow_production_live_alpha_dry_run: all_cli_gates,
            confirm_owner_approved_dry_run: all_cli_gates,
            confirm_no_production_order_submission: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_no_execution_adapter_call: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_real_funds: all_cli_gates,
        }
    }

    fn production_live_alpha_limit_dry_run_order_gate_opt(
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionLiveAlphaDryRunOrderGateOpt {
        LiveProductionLiveAlphaDryRunOrderGateOpt {
            run_id: "v150-live-alpha-request-preview".to_string(),
            session_id: Some("session-v150".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: "0.001".to_string(),
            notional: "10.00".to_string(),
            output,
            allow_production_live_alpha_dry_run: all_cli_gates,
            confirm_owner_approved_dry_run: all_cli_gates,
            confirm_no_production_order_submission: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_no_execution_adapter_call: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_real_funds: all_cli_gates,
        }
    }

    fn production_live_alpha_order_request_preview_opt(
        order_gate: PathBuf,
        manual_approval_lifecycle: PathBuf,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionLiveAlphaOrderRequestPreviewOpt {
        LiveProductionLiveAlphaOrderRequestPreviewOpt {
            run_id: "v150-live-alpha-request-preview".to_string(),
            order_gate,
            manual_approval_lifecycle,
            endpoint_path: TESTNET_ORDER_ENDPOINT_ORDER.to_string(),
            price: "10000.00".to_string(),
            time_in_force: TESTNET_ORDER_GTC_TIF.to_string(),
            timestamp_ms: 1_718_400_000_000,
            recv_window_ms: 5_000,
            api_key_env: "NTPRO_V150002_API_KEY".to_string(),
            api_secret_env: "NTPRO_V150002_API_SECRET".to_string(),
            credential_material: "synthetic".to_string(),
            output,
            allow_production_live_alpha_request_preview: all_cli_gates,
            confirm_owner_approved_request_preview: all_cli_gates,
            confirm_memory_only_signature: all_cli_gates,
            confirm_no_production_order_submission: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_no_execution_adapter_call: all_cli_gates,
            confirm_no_network: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_real_funds: all_cli_gates,
        }
    }

    struct ManualApprovalLifecycleFixture<'a> {
        approval_state: &'a str,
        run_id: &'a str,
        strategy_id: &'a str,
        symbol: &'a str,
        notional: &'a str,
        now_unix_ms: u64,
        expires_at_unix_ms: u64,
    }

    struct ManualApprovalLifecycleCase<'a> {
        name: &'a str,
        approval_state: &'a str,
        run_id: &'a str,
        symbol: &'a str,
        notional: &'a str,
        now_unix_ms: u64,
        expires_at_unix_ms: u64,
        expected_issue: &'a str,
    }

    fn production_live_alpha_manual_approval_lifecycle_opt(
        output: PathBuf,
        fixture: &ManualApprovalLifecycleFixture<'_>,
    ) -> LiveProductionLiveAlphaManualApprovalLifecycleOpt {
        LiveProductionLiveAlphaManualApprovalLifecycleOpt {
            run_id: fixture.run_id.to_string(),
            strategy_id: fixture.strategy_id.to_string(),
            symbol: fixture.symbol.to_string(),
            notional: fixture.notional.to_string(),
            approval_state: fixture.approval_state.to_string(),
            manual_approval_id: (fixture.approval_state != "pending")
                .then(|| "owner-approval-v150-005".to_string()),
            approved_by: (fixture.approval_state != "pending").then(|| "owner".to_string()),
            now_unix_ms: fixture.now_unix_ms,
            expires_at_unix_ms: fixture.expires_at_unix_ms,
            output,
            confirm_dry_run_request_preview_only: true,
            confirm_one_time_approval: true,
            confirm_no_production_mutation: true,
            confirm_dashboard_order_controls_disabled: true,
        }
    }

    fn production_live_alpha_execution_dry_run_opt(
        order_gate: PathBuf,
        risk_preflight: PathBuf,
        request_preview: PathBuf,
        kill_switch_runtime_gate: PathBuf,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionLiveAlphaExecutionDryRunOpt {
        LiveProductionLiveAlphaExecutionDryRunOpt {
            run_id: "v150-live-alpha-execution-dry-run".to_string(),
            order_gate,
            risk_preflight,
            request_preview,
            kill_switch_runtime_gate,
            output,
            allow_production_live_alpha_execution_dry_run: all_cli_gates,
            confirm_owner_approved_execution_dry_run: all_cli_gates,
            confirm_dry_run_adapter_only: all_cli_gates,
            confirm_no_production_adapter: all_cli_gates,
            confirm_no_production_order_submission: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_no_network: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_real_funds: all_cli_gates,
        }
    }

    fn production_live_alpha_kill_switch_runtime_gate_opt(
        kill_switch_approval: PathBuf,
        risk_preflight: PathBuf,
        request_preview: PathBuf,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionLiveAlphaKillSwitchRuntimeGateOpt {
        LiveProductionLiveAlphaKillSwitchRuntimeGateOpt {
            run_id: "v150-live-alpha-kill-switch-runtime-gate".to_string(),
            kill_switch_approval,
            risk_preflight,
            request_preview,
            output,
            allow_production_live_alpha_kill_switch_runtime_gate: all_cli_gates,
            confirm_owner_approved_runtime_gate: all_cli_gates,
            confirm_no_production_order_submission: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_no_network: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_real_funds: all_cli_gates,
        }
    }

    fn production_mutation_runtime_gate_opt(
        order_gate: PathBuf,
        risk_preflight: PathBuf,
        request_preview: PathBuf,
        kill_switch_runtime_gate: PathBuf,
        signing_approval: Option<PathBuf>,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionMutationRuntimeGateOpt {
        LiveProductionMutationRuntimeGateOpt {
            run_id: "v160-production-mutation-runtime-gate".to_string(),
            order_gate,
            risk_preflight,
            request_preview,
            kill_switch_runtime_gate,
            signing_approval,
            output,
            max_notional: "10.00".to_string(),
            allow_production_mutation_runtime_gate: all_cli_gates,
            confirm_owner_approved_production_mutation: all_cli_gates,
            confirm_single_limit_gtc: all_cli_gates,
            confirm_tiny_notional: all_cli_gates,
            confirm_signing_approval_required: all_cli_gates,
            confirm_no_network_before_send: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
            confirm_no_retry: all_cli_gates,
        }
    }

    fn production_mutation_signing_approval_opt(
        request_preview: PathBuf,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionMutationSigningApprovalOpt {
        LiveProductionMutationSigningApprovalOpt {
            run_id: "v160-production-mutation-signing-approval".to_string(),
            request_preview,
            approval_state: "approved".to_string(),
            manual_approval_id: Some("owner-approval-v160-003".to_string()),
            approved_by: Some("owner".to_string()),
            now_unix_ms: 1_718_400_000_000,
            expires_at_unix_ms: 1_718_400_060_000,
            output,
            allow_production_mutation_signing_approval: all_cli_gates,
            confirm_owner_approved_signing_material: all_cli_gates,
            confirm_env_only_signing_material: all_cli_gates,
            confirm_memory_only_signing: all_cli_gates,
            confirm_no_secret_persistence: all_cli_gates,
            confirm_no_network: all_cli_gates,
            confirm_no_production_order_submission: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
        }
    }

    fn production_mutation_request_builder_opt(
        runtime_gate: PathBuf,
        signing_approval: PathBuf,
        request_preview: PathBuf,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionMutationRequestBuilderOpt {
        LiveProductionMutationRequestBuilderOpt {
            run_id: "v160-production-mutation-request-builder".to_string(),
            runtime_gate,
            signing_approval,
            request_preview,
            api_key_env: "NTPRO_V150002_API_KEY".to_string(),
            api_secret_env: "NTPRO_V150002_API_SECRET".to_string(),
            timestamp_ms: 1_718_400_000_000,
            recv_window_ms: 5_000,
            max_notional: "10.00".to_string(),
            output,
            allow_production_mutation_request_builder: all_cli_gates,
            confirm_owner_approved_request_builder: all_cli_gates,
            confirm_single_limit_gtc: all_cli_gates,
            confirm_tiny_notional: all_cli_gates,
            confirm_signing_approval_ready: all_cli_gates,
            confirm_memory_only_signing: all_cli_gates,
            confirm_no_secret_persistence: all_cli_gates,
            confirm_no_network: all_cli_gates,
            confirm_no_production_order_submission: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
            confirm_no_retry: all_cli_gates,
        }
    }

    fn production_mutation_guarded_send_opt(
        request_builder: PathBuf,
        kill_switch_runtime_gate: PathBuf,
        request_preview: PathBuf,
        output: PathBuf,
        manual_online: bool,
        all_cli_gates: bool,
    ) -> LiveProductionMutationGuardedSendOpt {
        LiveProductionMutationGuardedSendOpt {
            run_id: "v160-production-mutation-guarded-send".to_string(),
            request_builder,
            kill_switch_runtime_gate,
            request_preview,
            api_key_env: "NTPRO_V150002_API_KEY".to_string(),
            api_secret_env: "NTPRO_V150002_API_SECRET".to_string(),
            timestamp_ms: 1_718_400_000_000,
            recv_window_ms: 5_000,
            max_notional: "10.00".to_string(),
            output,
            manual_online,
            allow_production_mutation_guarded_send: all_cli_gates,
            confirm_owner_approved_guarded_send: all_cli_gates,
            confirm_single_limit_gtc: all_cli_gates,
            confirm_tiny_notional: all_cli_gates,
            confirm_single_shot: all_cli_gates,
            confirm_no_retry: all_cli_gates,
            confirm_no_secret_persistence: all_cli_gates,
            confirm_response_redacted: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
        }
    }

    #[test]
    fn production_mutation_guarded_send_counters_separate_attempt_ack_and_platform_state() {
        let offline = production_mutation_guarded_send_counters(None);
        assert!(!offline.request_sent);
        assert!(!offline.network_attempted);
        assert!(!offline.production_order_request_attempted);
        assert!(!offline.http_send_attempted);
        assert!(!offline.exchange_ack_observed);
        assert!(!offline.confirmed_production_order_submission);
        assert_eq!(offline.production_order_submissions_attempted, 0);
        assert_eq!(offline.production_orders_submitted, 0);
        assert_eq!(offline.production_order_mutations_attempted, 0);
        assert!(!offline.production_trading_enabled);

        let rejected = ProductionMutationGuardedSendHttpResult::failure(
            Some(12),
            Some(400),
            "http_status_not_success",
        );
        let rejected_counters = production_mutation_guarded_send_counters(Some(&rejected));
        assert!(rejected_counters.request_sent);
        assert!(rejected_counters.network_attempted);
        assert!(rejected_counters.production_order_request_attempted);
        assert!(rejected_counters.http_send_attempted);
        assert!(!rejected_counters.exchange_ack_observed);
        assert!(!rejected_counters.confirmed_production_order_submission);
        assert_eq!(rejected_counters.production_order_submissions_attempted, 1);
        assert_eq!(rejected_counters.production_orders_submitted, 0);
        assert_eq!(rejected_counters.production_order_mutations_attempted, 1);
        assert!(!rejected_counters.production_trading_enabled);

        let acknowledged = ProductionMutationGuardedSendHttpResult::success(9, 200);
        let acknowledged_counters = production_mutation_guarded_send_counters(Some(&acknowledged));
        assert!(acknowledged_counters.request_sent);
        assert!(acknowledged_counters.network_attempted);
        assert!(acknowledged_counters.production_order_request_attempted);
        assert!(acknowledged_counters.http_send_attempted);
        assert!(acknowledged_counters.exchange_ack_observed);
        assert!(!acknowledged_counters.exchange_order_id_observed);
        assert!(!acknowledged_counters.exchange_order_status_observed);
        assert!(acknowledged_counters.confirmed_production_order_submission);
        assert_eq!(
            acknowledged_counters.production_order_submissions_attempted,
            1
        );
        assert_eq!(acknowledged_counters.production_orders_submitted, 1);
        assert_eq!(
            acknowledged_counters.production_order_mutations_attempted,
            1
        );
        assert!(acknowledged_counters.real_orders_submitted);
        assert!(!acknowledged_counters.platform_production_trading_enabled);
        assert!(!acknowledged_counters.production_trading_enabled);
    }

    fn production_mutation_response_redaction_opt(
        guarded_send: PathBuf,
        response: PathBuf,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionMutationResponseRedactionOpt {
        LiveProductionMutationResponseRedactionOpt {
            run_id: "v160-production-mutation-response-redaction".to_string(),
            guarded_send,
            response,
            output,
            allow_production_mutation_response_redaction: all_cli_gates,
            confirm_owner_approved_response_redaction: all_cli_gates,
            confirm_no_raw_response_persistence: all_cli_gates,
            confirm_no_headers_persistence: all_cli_gates,
            confirm_no_secret_persistence: all_cli_gates,
            confirm_order_metadata_only: all_cli_gates,
            confirm_no_account_balances: all_cli_gates,
            confirm_no_unrestricted_payload: all_cli_gates,
            confirm_no_retry: all_cli_gates,
        }
    }

    fn production_mutation_order_state_readback_opt(
        response_redaction: PathBuf,
        output: PathBuf,
        manual_online: bool,
        all_cli_gates: bool,
    ) -> LiveProductionMutationOrderStateReadbackOpt {
        LiveProductionMutationOrderStateReadbackOpt {
            run_id: "v160-production-mutation-order-state-readback".to_string(),
            response_redaction,
            output,
            manual_online,
            api_key_env: "NTPRO_V160007_API_KEY".to_string(),
            api_secret_env: "NTPRO_V160007_API_SECRET".to_string(),
            recv_window_ms: 5_000,
            allow_production_mutation_order_state_readback: all_cli_gates,
            confirm_owner_approved_order_state_readback: all_cli_gates,
            confirm_known_order_identifier_only: all_cli_gates,
            confirm_read_only_get_order: all_cli_gates,
            confirm_response_redacted: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_no_secret_persistence: all_cli_gates,
            confirm_no_retry: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
        }
    }

    fn production_mutation_audit_trail_opt(
        request_builder: PathBuf,
        guarded_send: PathBuf,
        response_redaction: PathBuf,
        order_state_readback: PathBuf,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionMutationAuditTrailOpt {
        LiveProductionMutationAuditTrailOpt {
            run_id: "v160-production-mutation-audit-trail".to_string(),
            request_builder,
            guarded_send,
            response_redaction,
            order_state_readback,
            output,
            allow_production_mutation_audit_trail: all_cli_gates,
            confirm_owner_approved_audit_trail: all_cli_gates,
            confirm_redacted_artifacts_only: all_cli_gates,
            confirm_no_secret_or_raw_payload_persistence: all_cli_gates,
            confirm_no_retry_or_followup_mutation: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
        }
    }

    fn production_mutation_failure_semantics_opt(
        audit_trail: PathBuf,
        output: PathBuf,
        failure_mode: ProductionMutationFailureMode,
        all_cli_gates: bool,
    ) -> LiveProductionMutationFailureSemanticsOpt {
        LiveProductionMutationFailureSemanticsOpt {
            run_id: "v160-production-mutation-failure-semantics".to_string(),
            audit_trail,
            failure_mode,
            output,
            allow_production_mutation_failure_semantics: all_cli_gates,
            confirm_evidence_only_failure_handling: all_cli_gates,
            confirm_no_retry: all_cli_gates,
            confirm_no_automatic_cancel_replace_amend: all_cli_gates,
            confirm_no_correction_or_flatten: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
            confirm_no_strategy_continuation: all_cli_gates,
            confirm_no_listen_key_lifecycle: all_cli_gates,
        }
    }

    fn write_kill_switch_approval_artifact(
        output: PathBuf,
        kill_switch_active: bool,
        approval_state: &str,
    ) {
        run_live_production_kill_switch_approval_artifact(
            &LiveProductionKillSwitchApprovalArtifactOpt {
                run_id: "v150-live-alpha-kill-switch-runtime-gate".to_string(),
                session_id: Some("session-v150".to_string()),
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                output,
                kill_switch_active,
                approval_state: approval_state.to_string(),
                manual_approval_id: (approval_state == "approved")
                    .then(|| "owner-approval-v150-004".to_string()),
                approved_by: (approval_state == "approved").then(|| "owner".to_string()),
                confirm_dry_run_only: true,
                confirm_no_production_mutation: true,
                confirm_dashboard_order_controls_disabled: true,
            },
        )
        .unwrap();
    }

    fn production_live_alpha_risk_preflight_opt(
        order_gate: PathBuf,
        input: PathBuf,
        output: PathBuf,
        all_cli_gates: bool,
    ) -> LiveProductionLiveAlphaRiskPreflightOpt {
        LiveProductionLiveAlphaRiskPreflightOpt {
            run_id: "v140-live-alpha-risk".to_string(),
            order_gate,
            input,
            output,
            confirm_hypothetical_dry_run_only: all_cli_gates,
            confirm_no_execution_adapter_call: all_cli_gates,
            confirm_no_production_order_submission: all_cli_gates,
            confirm_no_production_order_mutation: all_cli_gates,
            confirm_dashboard_order_controls_disabled: all_cli_gates,
        }
    }

    fn write_ready_live_alpha_artifact_chain(output_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let risk_input = output_dir.join("live_alpha_risk_input.json");
        let risk_preflight = output_dir.join("live_alpha_risk_preflight.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let request_preview = output_dir.join("live_alpha_order_request_preview.json");

        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        let mut input = passing_live_alpha_risk_input();
        input.order.order_type = "LIMIT".to_string();
        write_live_alpha_risk_input(&risk_input, &input);
        run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
            order_gate.clone(),
            risk_input,
            risk_preflight.clone(),
            true,
        ))
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();
        let request_opt = production_live_alpha_order_request_preview_opt(
            order_gate.clone(),
            manual_approval_lifecycle,
            request_preview.clone(),
            true,
        );
        run_live_production_live_alpha_order_request_preview_with_env(&request_opt, |name| {
            match name {
                "NTPRO_V150002_API_KEY" => {
                    Some("ntpro_v150003_synthetic_api_key_value".to_string())
                }
                "NTPRO_V150002_API_SECRET" => {
                    Some("ntpro_v150003_synthetic_api_secret_value".to_string())
                }
                _ => None,
            }
        })
        .unwrap();

        (order_gate, risk_preflight, request_preview)
    }

    fn write_ready_live_alpha_production_material_artifact_chain(
        output_dir: &Path,
    ) -> (PathBuf, PathBuf, PathBuf) {
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let risk_input = output_dir.join("live_alpha_risk_input.json");
        let risk_preflight = output_dir.join("live_alpha_risk_preflight.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let request_preview = output_dir.join("live_alpha_order_request_preview.json");

        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        let mut input = passing_live_alpha_risk_input();
        input.order.order_type = "LIMIT".to_string();
        write_live_alpha_risk_input(&risk_input, &input);
        run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
            order_gate.clone(),
            risk_input,
            risk_preflight.clone(),
            true,
        ))
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();
        let mut request_opt = production_live_alpha_order_request_preview_opt(
            order_gate.clone(),
            manual_approval_lifecycle,
            request_preview.clone(),
            true,
        );
        request_opt.credential_material = "production_live_alpha".to_string();
        let production_api_key = "ntpro_v160003_production_like_api_key_value";
        let production_api_secret = "ntpro_v160003_production_like_api_secret_value";
        run_live_production_live_alpha_order_request_preview_with_env(&request_opt, |name| {
            match name {
                PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
                | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
                "NTPRO_V150002_API_KEY" => Some(production_api_key.to_string()),
                "NTPRO_V150002_API_SECRET" => Some(production_api_secret.to_string()),
                _ => None,
            }
        })
        .unwrap();

        let body = fs::read_to_string(&request_preview).unwrap();
        assert!(!body.contains(production_api_key));
        assert!(!body.contains(production_api_secret));
        assert!(!body.contains("signature="));

        (order_gate, risk_preflight, request_preview)
    }

    fn write_ready_v160_request_builder_sources(output_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let (_, risk_preflight, request_preview) =
            write_ready_live_alpha_production_material_artifact_chain(output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
        let signing_approval = output_dir.join("production_mutation_signing_approval.json");
        let runtime_gate = output_dir.join("production_mutation_runtime_gate.json");

        write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight.clone(),
                request_preview.clone(),
                kill_switch_runtime_gate.clone(),
                true,
            ),
        )
        .unwrap();
        run_live_production_mutation_signing_approval(&production_mutation_signing_approval_opt(
            request_preview.clone(),
            signing_approval.clone(),
            true,
        ))
        .unwrap();
        run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
            output_dir.join("live_alpha_dry_run_order_gate.json"),
            risk_preflight,
            request_preview.clone(),
            kill_switch_runtime_gate,
            Some(signing_approval.clone()),
            runtime_gate.clone(),
            true,
        ))
        .unwrap();

        (runtime_gate, signing_approval, request_preview)
    }

    fn write_ready_v160_guarded_send_sources(output_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let (runtime_gate, signing_approval, request_preview) =
            write_ready_v160_request_builder_sources(output_dir);
        let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
        let request_builder = output_dir.join("production_mutation_request_builder.json");
        let production_api_key = "ntpro_v160005_production_like_api_key_value";
        let production_api_secret = "ntpro_v160005_production_like_api_secret_value";

        run_live_production_mutation_request_builder_with_env(
            &production_mutation_request_builder_opt(
                runtime_gate,
                signing_approval,
                request_preview.clone(),
                request_builder.clone(),
                true,
            ),
            |name| match name {
                PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
                | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
                "NTPRO_V150002_API_KEY" => Some(production_api_key.to_string()),
                "NTPRO_V150002_API_SECRET" => Some(production_api_secret.to_string()),
                _ => None,
            },
        )
        .unwrap();

        let body = fs::read_to_string(&request_builder).unwrap();
        assert!(!body.contains(production_api_key));
        assert!(!body.contains(production_api_secret));
        assert!(!body.contains("symbol=BTCUSDT"));

        (request_builder, request_preview, kill_switch_runtime_gate)
    }

    fn write_ready_v160_guarded_send_artifact(output_dir: &Path) -> PathBuf {
        let (request_builder, request_preview, kill_switch_runtime_gate) =
            write_ready_v160_guarded_send_sources(output_dir);
        let guarded_send = output_dir.join("production_mutation_guarded_send.json");
        run_live_production_mutation_guarded_send_with_env(
            &production_mutation_guarded_send_opt(
                request_builder,
                kill_switch_runtime_gate,
                request_preview,
                guarded_send.clone(),
                false,
                true,
            ),
            |_| None,
        )
        .unwrap();
        guarded_send
    }

    fn write_synthetic_production_mutation_response(path: &Path) {
        fs::write(
            path,
            serde_json::to_string_pretty(&serde_json::json!({
                "symbol": "BTCUSDT",
                "orderId": 123456789,
                "clientOrderId": "owner-approved-v160-single-shot",
                "transactTime": 1718400000000_u64,
                "workingTime": 1718400000001_u64,
                "status": "NEW",
                "type": "LIMIT",
                "side": "BUY",
                "timeInForce": "GTC"
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_forbidden_production_mutation_response(path: &Path) {
        fs::write(
            path,
            serde_json::to_string_pretty(&serde_json::json!({
                "symbol": "BTCUSDT",
                "orderId": 123456789,
                "clientOrderId": "owner-approved-v160-single-shot",
                "status": "NEW",
                "type": "LIMIT",
                "side": "BUY",
                "headers": {"X-MBX-APIKEY": "must_not_persist"},
                "signature": "signature=must_not_persist",
                "balances": [{"asset": "USDT", "free": "100.0"}],
                "payload": {"raw": "unrestricted"}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_ready_v160_response_redaction_artifact(output_dir: &Path) -> PathBuf {
        let guarded_send = write_ready_v160_guarded_send_artifact(output_dir);
        let response = output_dir.join("synthetic_order_response.json");
        let response_redaction = output_dir.join("production_mutation_response_redaction.json");
        write_synthetic_production_mutation_response(&response);
        run_live_production_mutation_response_redaction(
            &production_mutation_response_redaction_opt(
                guarded_send,
                response,
                response_redaction.clone(),
                true,
            ),
        )
        .unwrap();
        response_redaction
    }

    fn write_ready_v160_audit_trail_sources(
        output_dir: &Path,
    ) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
        let (request_builder, request_preview, kill_switch_runtime_gate) =
            write_ready_v160_guarded_send_sources(output_dir);
        let guarded_send = output_dir.join("production_mutation_guarded_send.json");
        run_live_production_mutation_guarded_send_with_env(
            &production_mutation_guarded_send_opt(
                request_builder.clone(),
                kill_switch_runtime_gate,
                request_preview,
                guarded_send.clone(),
                false,
                true,
            ),
            |_| None,
        )
        .unwrap();

        let response = output_dir.join("synthetic_order_response.json");
        let response_redaction = output_dir.join("production_mutation_response_redaction.json");
        write_synthetic_production_mutation_response(&response);
        run_live_production_mutation_response_redaction(
            &production_mutation_response_redaction_opt(
                guarded_send.clone(),
                response,
                response_redaction.clone(),
                true,
            ),
        )
        .unwrap();

        let order_state_readback = output_dir.join("production_mutation_order_state_readback.json");
        run_live_production_mutation_order_state_readback(
            &production_mutation_order_state_readback_opt(
                response_redaction.clone(),
                order_state_readback.clone(),
                false,
                true,
            ),
        )
        .unwrap();

        (
            request_builder,
            guarded_send,
            response_redaction,
            order_state_readback,
        )
    }

    fn write_ready_v160_audit_trail_artifact(output_dir: &Path) -> PathBuf {
        let (request_builder, guarded_send, response_redaction, order_state_readback) =
            write_ready_v160_audit_trail_sources(output_dir);
        let audit_trail = output_dir.join("production_mutation_audit_trail.json");
        run_live_production_mutation_audit_trail(&production_mutation_audit_trail_opt(
            request_builder,
            guarded_send,
            response_redaction,
            order_state_readback,
            audit_trail.clone(),
            true,
        ))
        .unwrap();
        audit_trail
    }

    fn passing_live_alpha_risk_input() -> ProductionLiveAlphaRiskPreflightInput {
        ProductionLiveAlphaRiskPreflightInput {
            schema_version: PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_INPUT_SCHEMA_VERSION.to_string(),
            session: ProductionLiveAlphaRiskPreflightSession {
                state: "running".to_string(),
            },
            market: ProductionLiveAlphaRiskPreflightMarket {
                symbol: "BTCUSDT".to_string(),
                last_event_at_unix_ms: 1_000,
                now_unix_ms: 1_500,
                max_age_ms: 1_000,
            },
            account: ProductionLiveAlphaRiskPreflightAccount {
                readable: true,
                account_id: "BINANCE-001".to_string(),
            },
            order_state: ProductionLiveAlphaRiskPreflightOrderState {
                readable: true,
                open_order_count: 0,
                last_read_at_unix_ms: None,
                now_unix_ms: None,
                max_age_ms: None,
            },
            risk: ProductionLiveAlphaRiskPreflightRisk {
                kill_switch_active: false,
                allowed_symbols: vec!["BTCUSDT".to_string()],
            },
            order: ProductionLiveAlphaRiskPreflightOrder {
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                order_type: "LIMIT".to_string(),
                quantity: "0.001".to_string(),
                notional: "10.00".to_string(),
            },
            limits: ProductionLiveAlphaRiskPreflightLimits {
                max_order_notional: "25.00".to_string(),
                current_position_notional: "50.00".to_string(),
                max_position_notional: "100.00".to_string(),
                max_open_orders: 5,
                max_clock_skew_ms: 100,
                observed_clock_skew_ms: 25,
            },
        }
    }

    fn write_live_alpha_risk_input(path: &Path, input: &ProductionLiveAlphaRiskPreflightInput) {
        fs::write(path, serde_json::to_string_pretty(input).unwrap()).unwrap();
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
        write_shadow_intent_with_notional(path, actual_submission, "10.00");
    }

    fn write_shadow_intent_with_notional(path: &Path, actual_submission: bool, notional: &str) {
        fs::write(
            path,
            format!(
                r#"{{"schema_version":"ntpro.v110_shadow_execution_intent.v1","run_id":"v120-shadow","intent_id":"intent-1","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","venue":"BINANCE","side":"buy","order_type":"market","quantity":"0.001","notional":"{notional}","mode":"production_shadow","submission_allowed":false,"actual_submission":{actual_submission},"submission_status":"blocked_by_v110_shadow_execution_boundary","execution_adapter_called":false,"order_endpoint_access_attempted":false,"production_order_mutation_attempted":false,"dashboard_order_controls_enabled":false}}
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], true);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], true);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
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
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
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
    fn production_order_state_readonly_proof_blocks_missing_gates_without_network() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-001-order-state-blocked-{}",
            std::process::id()
        ));
        let output = output_dir.join("order-state-proof.json");
        let opt = production_order_state_readonly_proof_opt(
            ProductionOrderStateReadEndpoint::OpenOrders,
            Some(output.clone()),
            false,
            false,
        );

        run_live_production_order_state_readonly_proof_with_env(&opt, |_| None).unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            report["schema_version"],
            PRODUCTION_ORDER_STATE_READONLY_SCHEMA_VERSION
        );
        assert_eq!(report["status"], "blocked_missing_gate");
        assert_eq!(report["endpoint"], "open_orders");
        assert_eq!(report["endpoint_class"], "production_order_state_read_only");
        assert_eq!(report["method"], "GET");
        assert_eq!(report["path"], "/api/v3/openOrders");
        assert_eq!(report["requires_api_key"], true);
        assert_eq!(report["requires_signature"], true);
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["order_state_read_attempted"], false);
        assert_eq!(report["production_order_state_reads_attempted"], 0);
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["listen_key_lifecycle_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
        assert_eq!(report["secrets_redacted"], true);
    }

    #[test]
    fn production_order_state_readonly_proof_writes_ready_offline_redacted_contract() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-001-order-state-ready-{}",
            std::process::id()
        ));
        let output = output_dir.join("order-state-proof.json");
        let opt = production_order_state_readonly_proof_opt(
            ProductionOrderStateReadEndpoint::OpenOrders,
            Some(output.clone()),
            true,
            false,
        );

        run_live_production_order_state_readonly_proof_with_env(&opt, |name| match name {
            PRODUCTION_ORDER_STATE_ENV_ALLOW
            | PRODUCTION_ORDER_STATE_ENV_OWNER_APPROVED
            | PRODUCTION_ORDER_STATE_ENV_NO_ORDER_MUTATION
            | PRODUCTION_ORDER_STATE_ENV_NO_SECRET_PERSISTENCE
            | PRODUCTION_ORDER_STATE_ENV_NO_LISTEN_KEY
            | PRODUCTION_ORDER_STATE_ENV_DASHBOARD_DISABLED => Some("1".to_string()),
            "NTPRO_V140001_API_KEY" => Some("ntpro_v140001_synthetic_api_key_value".to_string()),
            "NTPRO_V140001_API_SECRET" => {
                Some("ntpro_v140001_synthetic_api_secret_value".to_string())
            }
            _ => None,
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v140001_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v140001_synthetic_api_secret_value"));
        let report: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(report["status"], "ready_offline_contract");
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], true);
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
        assert_eq!(report["order_state_read_attempted"], false);
        assert_eq!(report["production_order_state_reads_attempted"], 0);
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
        assert_eq!(report["secrets_redacted"], true);
    }

    #[test]
    fn production_order_state_readonly_proof_blocks_manual_online_without_v14_owner_gate() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-001-order-state-online-blocked-{}",
            std::process::id()
        ));
        let output = output_dir.join("order-state-proof.json");
        let opt = production_order_state_readonly_proof_opt(
            ProductionOrderStateReadEndpoint::OpenOrders,
            Some(output.clone()),
            true,
            true,
        );
        let mut http_called = false;
        let mut read_env = |name: &str| match name {
            PRODUCTION_ORDER_STATE_ENV_MANUAL_ONLINE => None,
            PRODUCTION_ORDER_STATE_ENV_ALLOW
            | PRODUCTION_ORDER_STATE_ENV_OWNER_APPROVED
            | PRODUCTION_ORDER_STATE_ENV_NO_ORDER_MUTATION
            | PRODUCTION_ORDER_STATE_ENV_NO_SECRET_PERSISTENCE
            | PRODUCTION_ORDER_STATE_ENV_NO_LISTEN_KEY
            | PRODUCTION_ORDER_STATE_ENV_DASHBOARD_DISABLED => Some("1".to_string()),
            "NTPRO_V140001_API_KEY" => Some("ntpro_v140001_synthetic_api_key_value".to_string()),
            "NTPRO_V140001_API_SECRET" => {
                Some("ntpro_v140001_synthetic_api_secret_value".to_string())
            }
            _ => None,
        };

        run_live_production_order_state_readonly_proof_with_env_and_http(
            &opt,
            &mut read_env,
            |_opt, _credentials, _recv_window_ms| {
                http_called = true;
                ProductionOrderStateHttpResult::success(
                    ProductionOrderStateReadEndpoint::OpenOrders,
                    1,
                    200,
                )
            },
        )
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert!(!http_called);
        assert_eq!(report["status"], "blocked_missing_manual_online_gate");
        assert_eq!(report["manual_online_requested"], true);
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], false);
        assert_eq!(report["online_execution_supported"], true);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["order_state_read_attempted"], false);
        assert_eq!(report["production_order_state_reads_attempted"], 0);
        assert_eq!(report["error_code"], "not_attempted");
        assert_eq!(report["production_order_mutation_attempted"], false);
    }

    #[test]
    fn production_order_state_readonly_proof_records_owner_gated_online_success() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-001-order-state-online-success-{}",
            std::process::id()
        ));
        let output = output_dir.join("order-state-proof.json");
        let opt = production_order_state_readonly_proof_opt(
            ProductionOrderStateReadEndpoint::Order,
            Some(output.clone()),
            true,
            true,
        );
        let mut read_env = |name: &str| match name {
            "NTPRO_V140001_API_KEY" => Some("ntpro_v140001_synthetic_api_key_value".to_string()),
            "NTPRO_V140001_API_SECRET" => {
                Some("ntpro_v140001_synthetic_api_secret_value".to_string())
            }
            _ => all_env_enabled(name),
        };

        run_live_production_order_state_readonly_proof_with_env_and_http(
            &opt,
            &mut read_env,
            |proof_opt, credentials, recv_window_ms| {
                assert_eq!(proof_opt.endpoint, ProductionOrderStateReadEndpoint::Order);
                assert!(credentials.api_key_present());
                assert!(credentials.api_secret_present());
                assert_eq!(recv_window_ms, 5_000);
                ProductionOrderStateHttpResult::success(
                    ProductionOrderStateReadEndpoint::Order,
                    17,
                    200,
                )
            },
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v140001_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v140001_synthetic_api_secret_value"));
        let report: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(report["status"], "online_order_state_read_ok");
        assert_eq!(report["endpoint"], "order");
        assert_eq!(report["method"], "GET");
        assert_eq!(report["path"], "/api/v3/order");
        assert_eq!(report["endpoint_read_allowed"], true);
        assert_eq!(report["offline_contract_ready"], false);
        assert_eq!(report["read_allowed"], false);
        assert_eq!(report["contract_ready"], false);
        assert_eq!(report["online_read_allowed"], true);
        assert_eq!(report["online_execution_supported"], true);
        assert_eq!(report["network_attempted"], true);
        assert_eq!(report["order_state_read_attempted"], true);
        assert_eq!(report["production_order_state_reads_attempted"], 1);
        assert_eq!(report["response_status_code"], 200);
        assert_eq!(report["response_shape"], "binance_order_state_v1");
        assert_eq!(report["response_shape_validated"], true);
        assert_eq!(report["endpoint_shape_validated"], true);
        assert_eq!(report["order_entries_observed"], 1);
        assert_eq!(report["non_empty_order_state_observed"], true);
        assert_eq!(report["order_lifecycle_readiness"], true);
        assert_eq!(report["latency_ms"], 17);
        assert_eq!(report["error_code"], "none");
        assert_eq!(report["production_order_submission_attempted"], false);
        assert_eq!(report["production_order_mutation_attempted"], false);
        assert_eq!(report["listen_key_lifecycle_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
        assert_eq!(report["real_orders_submitted"], false);
        assert_eq!(report["production_trading_enabled"], false);
        assert_eq!(report["order_state_values_are_exchange_truth"], true);
        assert_eq!(report["shadow_values_are_exchange_truth"], false);
        assert_eq!(report["portfolio_values_are_exchange_truth"], false);
        assert_eq!(report["values_are_exchange_truth"], true);
    }

    #[test]
    fn production_order_state_signed_request_redacts_secret_values() {
        let credentials = EnvOnlyProductionReadCredentials::from_values(
            "NTPRO_V140001_API_KEY".to_string(),
            Some("ntpro_v140001_synthetic_api_key_value".to_string()),
            "NTPRO_V140001_API_SECRET".to_string(),
            Some("ntpro_v140001_synthetic_api_secret_value".to_string()),
        );
        let opt = production_order_state_readonly_proof_opt(
            ProductionOrderStateReadEndpoint::Order,
            None,
            true,
            true,
        );

        let request = build_production_order_state_signed_request(
            &opt,
            &credentials,
            1_718_400_000_000,
            5_000,
        )
        .unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.endpoint_path, "/api/v3/order");
        assert_eq!(request.api_key_header_name, BINANCE_API_KEY_HEADER);
        assert!(request.query_without_signature.contains("symbol=BTCUSDT"));
        assert!(request.query_without_signature.contains("orderId=12345"));
        assert!(request.signed_query.contains("timestamp=1718400000000"));
        assert_eq!(request.signature.len(), 64);
        assert!(
            request
                .signature
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        );

        let debug_body = format!("{request:?}");
        assert!(!debug_body.contains("ntpro_v140001_synthetic_api_key_value"));
        assert!(!debug_body.contains("ntpro_v140001_synthetic_api_secret_value"));
        assert!(!debug_body.contains(&request.signature));
        assert!(!debug_body.contains(&request.signed_query));
    }

    #[test]
    fn production_order_state_shape_summary_accepts_expected_shapes() {
        let open_orders = serde_json::json!([
            {"symbol": "BTCUSDT", "orderId": 12345, "status": "NEW"}
        ]);
        let single_order = serde_json::json!({
            "symbol": "BTCUSDT",
            "orderId": 12345,
            "status": "FILLED"
        });

        let open_summary = summarize_production_order_state_shape(
            ProductionOrderStateReadEndpoint::OpenOrders,
            &open_orders,
        );
        let order_summary = summarize_production_order_state_shape(
            ProductionOrderStateReadEndpoint::Order,
            &single_order,
        );

        assert!(open_summary.shape_validated);
        assert!(open_summary.endpoint_shape_validated);
        assert_eq!(open_summary.order_entry_count, Some(1));
        assert_eq!(open_summary.order_entries_observed, 1);
        assert!(open_summary.non_empty_order_state_observed);
        assert!(open_summary.order_lifecycle_readiness);
        assert!(!open_summary.raw_order_list_recorded);
        assert!(order_summary.shape_validated);
        assert!(order_summary.endpoint_shape_validated);
        assert_eq!(order_summary.order_entry_count, Some(1));
        assert_eq!(order_summary.order_entries_observed, 1);
        assert!(order_summary.non_empty_order_state_observed);
        assert!(order_summary.order_lifecycle_readiness);
        assert!(!order_summary.raw_order_response_recorded);

        let summary_body = serde_json::to_string(&open_summary).unwrap();
        assert!(!summary_body.contains("BTCUSDT"));
        assert!(!summary_body.contains("12345"));
        assert!(!summary_body.contains("NEW"));
    }

    #[test]
    fn production_order_state_shape_summary_classifies_empty_open_orders_as_shape_only() {
        let empty_open_orders = serde_json::json!([]);

        let summary = summarize_production_order_state_shape(
            ProductionOrderStateReadEndpoint::OpenOrders,
            &empty_open_orders,
        );

        assert!(summary.shape_validated);
        assert!(summary.endpoint_shape_validated);
        assert_eq!(summary.status, "accepted");
        assert_eq!(summary.order_entry_count, Some(0));
        assert_eq!(summary.order_entries_observed, 0);
        assert!(!summary.non_empty_order_state_observed);
        assert!(!summary.order_lifecycle_readiness);
        assert_eq!(summary.rejection_reason, "none");
        assert!(!summary.raw_order_list_recorded);
    }

    #[test]
    fn production_order_state_shape_summary_rejects_invalid_open_orders_shape() {
        let invalid_open_orders = serde_json::json!({"symbol": "BTCUSDT"});

        let summary = summarize_production_order_state_shape(
            ProductionOrderStateReadEndpoint::OpenOrders,
            &invalid_open_orders,
        );

        assert!(!summary.shape_validated);
        assert!(!summary.endpoint_shape_validated);
        assert_eq!(summary.status, "rejected");
        assert_eq!(summary.order_entry_count, None);
        assert_eq!(summary.order_entries_observed, 0);
        assert!(!summary.non_empty_order_state_observed);
        assert!(!summary.order_lifecycle_readiness);
        assert_eq!(summary.rejection_reason, "root_not_array");
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
        assert_eq!(
            runtime["notional_preflight"]["status"],
            "shadow_decimal_string_evidence_only"
        );
        assert_eq!(
            runtime["notional_preflight"]["aggregation"],
            "rust_decimal_string_sum"
        );
        assert_eq!(runtime["notional_preflight"]["decimal_string_sum"], "10");
        assert_eq!(runtime["notional_preflight"]["parsed_notional_count"], 1);
        assert_eq!(runtime["notional_preflight"]["f64_aggregation_used"], false);
        assert_eq!(
            runtime["notional_preflight"]["live_alpha_money_math_ready"],
            false
        );
        assert_eq!(
            runtime["notional_preflight"]["risk_or_execution_grade"],
            false
        );
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
    fn production_shadow_portfolio_runtime_preserves_decimal_string_notional_preflight() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v121-008-shadow-portfolio-decimal-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent_with_notional(&shadow_intent, false, "0.100000000000000001");
        fs::write(
            &shadow_intent,
            format!(
                "{}{}",
                fs::read_to_string(&shadow_intent).unwrap(),
                r#"{"schema_version":"ntpro.v110_shadow_execution_intent.v1","run_id":"v120-shadow","intent_id":"intent-2","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","venue":"BINANCE","side":"buy","order_type":"market","quantity":"0.001","notional":"0.200000000000000002","mode":"production_shadow","submission_allowed":false,"actual_submission":false,"submission_status":"blocked_by_v110_shadow_execution_boundary","execution_adapter_called":false,"order_endpoint_access_attempted":false,"production_order_mutation_attempted":false,"dashboard_order_controls_enabled":false}
"#
            ),
        )
        .unwrap();

        let report = build_production_shadow_portfolio_runtime_report(
            "v120-shadow",
            Some("portfolio-1"),
            &account_snapshot,
            &shadow_intent,
        )
        .unwrap();

        assert_eq!(
            report.notional_preflight.status,
            "shadow_decimal_string_evidence_only"
        );
        assert_eq!(
            report.notional_preflight.aggregation,
            "rust_decimal_string_sum"
        );
        assert_eq!(
            report.notional_preflight.decimal_string_sum.as_deref(),
            Some("0.300000000000000003")
        );
        assert_eq!(report.notional_preflight.parsed_notional_count, 2);
        assert!(!report.notional_preflight.f64_aggregation_used);
        assert!(!report.notional_preflight.live_alpha_money_math_ready);
        assert!(!report.notional_preflight.risk_or_execution_grade);
        assert_eq!(
            report.exposure.notional.as_deref(),
            Some("0.300000000000000003")
        );
    }

    #[test]
    fn v13_live_alpha_amount_boundary_uses_decimal_strings_without_f64() {
        let first = parse_non_negative_decimal("0.100000000000000001").unwrap();
        let second = parse_non_negative_decimal("0.200000000000000002").unwrap();
        let sum = first + second;
        let f64_sum = 0.1_f64 + 0.2_f64;

        assert_eq!(format_decimal(&sum), "0.300000000000000003");
        assert_eq!(format!("{sum}"), "0.300000000000000003");
        assert_ne!(format!("{sum}"), format!("{f64_sum}"));

        for invalid in ["", "-0.1", "1e-5", "NaN", "inf"] {
            assert!(
                parse_non_negative_decimal(invalid).is_err(),
                "v0.13 amount boundary must reject non-plain decimal string {invalid}",
            );
        }
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

    #[tokio::test]
    async fn production_shadow_preflight_session_writes_heartbeats_and_stops() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v130-002-shadow-preflight-session-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let preflight_events = output_dir.join("shadow_preflight_session.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);
        run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
            run_id: "v130-shadow".to_string(),
            snapshot_id: Some("portfolio-1".to_string()),
            account_snapshot,
            shadow_intent,
            output: portfolio_runtime.clone(),
            compat_snapshot_output: None,
        })
        .unwrap();

        let result = run_production_shadow_preflight_session_loop(
            &LiveProductionShadowPreflightSessionOpt {
                run_id: "v130-shadow".to_string(),
                session_id: Some("session-1".to_string()),
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                shadow_portfolio_runtime: portfolio_runtime,
                strategy_session_status: None,
                output: preflight_events.clone(),
                max_heartbeats: 2,
                heartbeat_interval_ms: 1,
                stale_after_ms: 60_000,
                stop_file: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(result.heartbeats_written, 2);
        assert_eq!(result.final_state, "stopped");
        assert!(!result.stop_file_observed);
        assert!(!result.stale_data_detected);
        let events = read_jsonl_values(&preflight_events);
        assert_eq!(events.len(), 4);
        assert_eq!(
            events[0]["schema_version"],
            PRODUCTION_SHADOW_PREFLIGHT_SESSION_EVENT_SCHEMA_VERSION
        );
        assert_eq!(events[0]["event_type"], "shadow_preflight_session_started");
        assert_eq!(
            events[1]["event_type"],
            "shadow_preflight_session_heartbeat"
        );
        assert_eq!(events[1]["heartbeat_seq"], 1);
        assert_eq!(
            events[2]["event_type"],
            "shadow_preflight_session_heartbeat"
        );
        assert_eq!(events[2]["heartbeat_seq"], 2);
        assert_eq!(events[3]["event_type"], "shadow_preflight_session_stopped");
        assert_eq!(events[3]["shutdown_reason"], "max_heartbeats_reached");
        for event in &events {
            assert_eq!(event["session_network_attempted"], false);
            assert_eq!(event["production_order_submissions_attempted"], 0);
            assert_eq!(event["production_orders_submitted"], 0);
            assert_eq!(event["production_order_mutations_attempted"], 0);
            assert_eq!(event["production_order_state_reads_attempted"], 0);
            assert_eq!(event["listen_key_lifecycle_attempted"], 0);
            assert_eq!(event["cancel_replace_amend_attempted"], false);
            assert_eq!(event["dashboard_order_controls_enabled"], false);
            assert_eq!(event["values_are_exchange_truth"], false);
        }
    }

    #[tokio::test]
    async fn production_shadow_preflight_session_stops_on_owner_stop_file() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v130-002-shadow-preflight-stop-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let preflight_events = output_dir.join("shadow_preflight_session.jsonl");
        let stop_file = output_dir.join("STOP");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);
        fs::write(&stop_file, "stop").unwrap();
        run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
            run_id: "v130-shadow-stop".to_string(),
            snapshot_id: Some("portfolio-1".to_string()),
            account_snapshot,
            shadow_intent,
            output: portfolio_runtime.clone(),
            compat_snapshot_output: None,
        })
        .unwrap();

        let result = run_production_shadow_preflight_session_loop(
            &LiveProductionShadowPreflightSessionOpt {
                run_id: "v130-shadow-stop".to_string(),
                session_id: Some("session-1".to_string()),
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                shadow_portfolio_runtime: portfolio_runtime,
                strategy_session_status: None,
                output: preflight_events.clone(),
                max_heartbeats: 5,
                heartbeat_interval_ms: 1,
                stale_after_ms: 60_000,
                stop_file: Some(stop_file.clone()),
            },
        )
        .await
        .unwrap();

        assert_eq!(result.heartbeats_written, 0);
        assert_eq!(result.final_state, "stopped");
        assert!(result.stop_file_observed);
        let events = read_jsonl_values(&preflight_events);
        assert_eq!(events.len(), 2);
        assert_eq!(events[1]["event_type"], "shadow_preflight_session_stopped");
        assert_eq!(events[1]["shutdown_reason"], "owner_stop_file");
        assert_eq!(events[1]["stop_file_observed"], true);
        assert_eq!(events[1]["stop_file_path"], stop_file.display().to_string());
        assert_eq!(events[1]["production_order_mutations_attempted"], 0);
    }

    #[tokio::test]
    async fn production_shadow_preflight_session_detects_stale_portfolio_runtime() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v130-002-shadow-preflight-stale-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let preflight_events = output_dir.join("shadow_preflight_session.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);
        run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
            run_id: "v130-shadow-stale".to_string(),
            snapshot_id: Some("portfolio-1".to_string()),
            account_snapshot,
            shadow_intent,
            output: portfolio_runtime.clone(),
            compat_snapshot_output: None,
        })
        .unwrap();
        sleep(Duration::from_millis(5)).await;

        let result = run_production_shadow_preflight_session_loop(
            &LiveProductionShadowPreflightSessionOpt {
                run_id: "v130-shadow-stale".to_string(),
                session_id: Some("session-1".to_string()),
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                shadow_portfolio_runtime: portfolio_runtime,
                strategy_session_status: None,
                output: preflight_events.clone(),
                max_heartbeats: 2,
                heartbeat_interval_ms: 1,
                stale_after_ms: 1,
                stop_file: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(result.heartbeats_written, 0);
        assert_eq!(result.final_state, "stale_data_halted");
        assert!(result.stale_data_detected);
        let events = read_jsonl_values(&preflight_events);
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[1]["event_type"],
            "shadow_preflight_stale_data_detected"
        );
        assert_eq!(events[1]["state"], "stale_data_halted");
        assert_eq!(events[1]["stale_data_detected"], true);
        assert_eq!(
            events[1]["shutdown_reason"],
            "stale_shadow_portfolio_runtime"
        );
        assert_eq!(events[1]["production_orders_submitted"], 0);
    }

    #[test]
    fn production_live_alpha_dry_run_order_gate_blocks_missing_owner_flags() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-003-live-alpha-dry-run-blocked-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let output = output_dir.join("live_alpha_dry_run_order_gate.json");

        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_dry_run_order_gate_opt(output.clone(), false),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["dry_run_order_gate_ready"], false);
        assert_eq!(artifact["dry_run_order_intent_recorded"], false);
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 8);
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["order_endpoint_access_attempted"], false);
        assert_eq!(artifact["execution_adapter_called"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["real_funds"], false);
    }

    #[test]
    fn production_live_alpha_dry_run_order_gate_rejects_market_order_type() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v151-004-live-alpha-market-order-gate-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let output = output_dir.join("live_alpha_market_order_gate.json");
        let mut opt = production_live_alpha_dry_run_order_gate_opt(output, true);
        opt.order_type = "MARKET".to_string();

        let err = run_live_production_live_alpha_dry_run_order_gate(&opt).unwrap_err();
        assert!(
            err.to_string().contains("only supports LIMIT order_type"),
            "{err:?}"
        );
    }

    #[test]
    fn production_live_alpha_dry_run_order_gate_records_ready_no_submission_contract() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-003-live-alpha-dry-run-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let output = output_dir.join("live_alpha_dry_run_order_gate.json");

        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_dry_run_order_gate_opt(output.clone(), true),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_dry_run_no_submission");
        assert_eq!(artifact["mode"], "production_live_alpha_dry_run");
        assert_eq!(artifact["symbol"], "BTCUSDT");
        assert_eq!(artifact["side"], "BUY");
        assert_eq!(artifact["order_type"], "LIMIT");
        assert_eq!(artifact["quantity"], "0.001");
        assert_eq!(artifact["notional"], "10.00");
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["dry_run_order_gate_ready"], true);
        assert_eq!(artifact["dry_run_order_intent_recorded"], true);
        assert_eq!(artifact["order_submission_mode"], "dry_run_no_submission");
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_order_state_reads_allowed"], false);
        assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["production_order_state_reads_attempted"], 0);
        assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
        assert_eq!(artifact["cancel_replace_amend_attempted"], false);
        assert_eq!(artifact["order_endpoint_access_attempted"], false);
        assert_eq!(artifact["execution_adapter_called"], false);
        assert_eq!(artifact["matching_engine_submission"], false);
        assert_eq!(artifact["actual_submission_count"], 0);
        assert_eq!(artifact["automatic_correction_orders_submitted"], 0);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["external_venue_connection"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["real_funds"], false);
        assert_eq!(artifact["production_trading_enabled"], false);
        assert_eq!(artifact["values_are_exchange_truth"], false);
    }

    #[test]
    fn production_live_alpha_order_request_preview_builds_redacted_metadata_only() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v150-002-request-preview-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let output = output_dir.join("live_alpha_order_request_preview.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();

        let opt = production_live_alpha_order_request_preview_opt(
            order_gate,
            manual_approval_lifecycle.clone(),
            output.clone(),
            true,
        );
        run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| {
            panic!("default synthetic signing material must not read env var {name}")
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains(PRODUCTION_MUTATION_PREVIEW_SYNTHETIC_API_KEY));
        assert!(!body.contains(PRODUCTION_MUTATION_PREVIEW_SYNTHETIC_API_SECRET));
        assert!(!body.contains("signature="));
        assert!(!body.contains("symbol=BTCUSDT"));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_request_preview_only");
        assert_eq!(
            artifact["endpoint_class"],
            "production_mutation_owner_approved_manual_only"
        );
        assert_eq!(artifact["endpoint_decision"], "allow_request_preview_only");
        assert_eq!(artifact["request_method"], "POST");
        assert_eq!(artifact["request_target"], TESTNET_ORDER_ENDPOINT_ORDER);
        assert_eq!(
            artifact["query_shape_without_signature"],
            "symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp"
        );
        assert_eq!(
            artifact["signature_preflight"],
            "created_in_memory_not_recorded"
        );
        assert_eq!(artifact["credential_material"], "synthetic");
        assert_eq!(artifact["production_signing_material_gate_required"], false);
        assert_eq!(artifact["production_signing_material_gate_open"], false);
        assert_eq!(artifact["production_signing_material_env_read"], false);
        assert_eq!(
            artifact["production_signing_material_missing_gate_env_vars"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["api_key_header_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["request_body_recorded"], false);
        assert_eq!(artifact["raw_request_body_recorded"], false);
        assert_eq!(
            artifact["manual_approval_lifecycle_status"],
            "approval_valid_for_dry_run_request_preview"
        );
        assert_eq!(artifact["manual_approval_lifecycle_state"], "approved");
        assert_eq!(artifact["manual_approval_lifecycle_valid"], true);
        assert_eq!(
            artifact["manual_approval_lifecycle_issues"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(artifact["manual_approval_one_time"], true);
        assert_eq!(artifact["manual_approval_used"], true);
        assert_eq!(artifact["manual_approval_consumed"], true);
        assert_eq!(
            artifact["manual_approval_consume_status"],
            "approval_consumed_after_request_preview_created"
        );
        assert_eq!(
            artifact["manual_approval_consume_transition"],
            "approved_to_request_preview_created_to_used"
        );
        assert_eq!(artifact["order_gate_ready"], true);
        assert_eq!(artifact["request_preview_allowed"], true);
        assert_eq!(artifact["request_preview_built"], true);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["order_endpoint_access_attempted"], false);
        assert_eq!(artifact["execution_adapter_called"], false);
        assert_eq!(artifact["production_adapter_called"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["real_funds"], false);
        assert_eq!(artifact["production_trading_enabled"], false);
        assert_eq!(artifact["signed_request_memory_only"], true);
        assert_eq!(artifact["secrets_redacted"], true);

        let consumed_approval: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(manual_approval_lifecycle).unwrap()).unwrap();
        assert_eq!(
            consumed_approval["status"],
            "approval_consumed_after_request_preview_created"
        );
        assert_eq!(consumed_approval["approval_state"], "used");
        assert_eq!(consumed_approval["approval_used"], true);
        assert_eq!(consumed_approval["approval_consumed"], true);
        assert_eq!(consumed_approval["request_preview_created"], true);
        assert_eq!(
            consumed_approval["consumed_by_request_preview_run_id"],
            "v150-live-alpha-request-preview"
        );
        assert_eq!(consumed_approval["approval_lifecycle_valid"], false);
    }

    #[test]
    fn production_live_alpha_order_request_preview_rejects_order_test_endpoint() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v151-005-order-test-preview-denied-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let output = output_dir.join("live_alpha_order_request_preview.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();

        let mut opt = production_live_alpha_order_request_preview_opt(
            order_gate,
            manual_approval_lifecycle,
            output,
            true,
        );
        opt.endpoint_path = TESTNET_ORDER_ENDPOINT_TEST.to_string();

        let err = run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| {
            panic!("denied /api/v3/order/test preview must not read env var {name}")
        })
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("allowlist only includes POST /api/v3/order"),
            "{err:?}"
        );
    }

    #[test]
    fn production_live_alpha_order_request_preview_blocks_production_material_without_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v151-003-production-material-blocked-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let output = output_dir.join("live_alpha_order_request_preview.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();

        let mut opt = production_live_alpha_order_request_preview_opt(
            order_gate,
            manual_approval_lifecycle,
            output.clone(),
            true,
        );
        opt.credential_material = "production_live_alpha".to_string();

        run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| match name {
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
            | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => None,
            "NTPRO_V150002_API_KEY" | "NTPRO_V150002_API_SECRET" => {
                panic!("blocked production signing material must not read {name}")
            }
            _ => None,
        })
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_endpoint_or_owner_scope");
        assert_eq!(artifact["credential_material"], "production_live_alpha");
        assert_eq!(artifact["production_signing_material_gate_required"], true);
        assert_eq!(artifact["production_signing_material_gate_open"], false);
        assert_eq!(artifact["production_signing_material_env_read"], false);
        assert_eq!(
            artifact["production_signing_material_missing_gate_env_vars"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(
            artifact["missing_env_vars"]
                .as_array()
                .unwrap()
                .iter()
                .any(|env| env.as_str() == Some(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW))
        );
        assert!(artifact["missing_env_vars"].as_array().unwrap().iter().any(
            |env| env.as_str() == Some(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED)
        ));
        assert_eq!(artifact["request_preview_allowed"], false);
        assert_eq!(artifact["request_preview_built"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
    }

    #[test]
    fn production_live_alpha_order_request_preview_uses_production_material_only_with_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v151-003-production-material-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let output = output_dir.join("live_alpha_order_request_preview.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();

        let mut opt = production_live_alpha_order_request_preview_opt(
            order_gate,
            manual_approval_lifecycle,
            output.clone(),
            true,
        );
        opt.credential_material = "production_live_alpha".to_string();

        let production_api_key = "ntpro_v151003_production_like_api_key_value";
        let production_api_secret = "ntpro_v151003_production_like_api_secret_value";
        run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| match name {
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
            | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
            "NTPRO_V150002_API_KEY" => Some(production_api_key.to_string()),
            "NTPRO_V150002_API_SECRET" => Some(production_api_secret.to_string()),
            _ => None,
        })
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains(production_api_key));
        assert!(!body.contains(production_api_secret));
        assert!(!body.contains("signature="));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(artifact["status"], "ready_request_preview_only");
        assert_eq!(artifact["credential_material"], "production_live_alpha");
        assert_eq!(artifact["production_signing_material_gate_required"], true);
        assert_eq!(artifact["production_signing_material_gate_open"], true);
        assert_eq!(artifact["production_signing_material_env_read"], true);
        assert_eq!(
            artifact["production_signing_material_missing_gate_env_vars"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["request_preview_built"], true);
        assert_eq!(artifact["signed_request_memory_only"], true);
        assert_eq!(artifact["secrets_redacted"], true);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
    }

    #[test]
    fn production_live_alpha_order_request_preview_consumes_one_time_manual_approval() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v151-002-approval-consume-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let first_preview = output_dir.join("live_alpha_order_request_preview_first.json");
        let second_preview = output_dir.join("live_alpha_order_request_preview_second.json");
        let risk_input = output_dir.join("live_alpha_risk_input.json");
        let risk_preflight = output_dir.join("live_alpha_risk_preflight.json");
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let runtime_gate = output_dir.join("live_alpha_kill_switch_runtime_gate.json");
        let execution_output = output_dir.join("live_alpha_execution_dry_run.json");

        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();

        let first_opt = production_live_alpha_order_request_preview_opt(
            order_gate.clone(),
            manual_approval_lifecycle.clone(),
            first_preview.clone(),
            true,
        );
        run_live_production_live_alpha_order_request_preview_with_env(
            &first_opt,
            |name| match name {
                "NTPRO_V150002_API_KEY" => {
                    Some("ntpro_v151002_synthetic_api_key_value".to_string())
                }
                "NTPRO_V150002_API_SECRET" => {
                    Some("ntpro_v151002_synthetic_api_secret_value".to_string())
                }
                _ => None,
            },
        )
        .unwrap();

        let first_artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&first_preview).unwrap()).unwrap();
        assert_eq!(first_artifact["status"], "ready_request_preview_only");
        assert_eq!(first_artifact["request_preview_built"], true);
        assert_eq!(first_artifact["manual_approval_consumed"], true);
        assert_eq!(first_artifact["manual_approval_used"], true);

        let consumed_approval: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&manual_approval_lifecycle).unwrap()).unwrap();
        assert_eq!(consumed_approval["approval_state"], "used");
        assert_eq!(consumed_approval["approval_used"], true);
        assert_eq!(consumed_approval["request_preview_created"], true);
        assert_eq!(consumed_approval["approval_lifecycle_valid"], false);

        let second_opt = production_live_alpha_order_request_preview_opt(
            order_gate.clone(),
            manual_approval_lifecycle,
            second_preview.clone(),
            true,
        );
        run_live_production_live_alpha_order_request_preview_with_env(
            &second_opt,
            |name| match name {
                "NTPRO_V150002_API_KEY" => {
                    Some("ntpro_v151002_synthetic_api_key_value".to_string())
                }
                "NTPRO_V150002_API_SECRET" => {
                    Some("ntpro_v151002_synthetic_api_secret_value".to_string())
                }
                _ => None,
            },
        )
        .unwrap();

        let second_artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&second_preview).unwrap()).unwrap();
        assert_eq!(
            second_artifact["status"],
            "blocked_manual_approval_lifecycle"
        );
        assert_eq!(second_artifact["request_preview_allowed"], false);
        assert_eq!(second_artifact["request_preview_built"], false);
        assert_eq!(second_artifact["manual_approval_lifecycle_valid"], false);
        assert_eq!(second_artifact["manual_approval_used"], true);
        assert_eq!(
            second_artifact["manual_approval_consume_status"],
            "approval_already_used"
        );
        assert!(
            second_artifact["manual_approval_lifecycle_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == "manual_approval_used")
        );
        assert_eq!(second_artifact["request_sent"], false);
        assert_eq!(second_artifact["production_orders_submitted"], 0);
        assert_eq!(second_artifact["production_order_mutations_attempted"], 0);
        assert_eq!(second_artifact["network_attempted"], false);

        let mut risk = passing_live_alpha_risk_input();
        risk.order.order_type = "LIMIT".to_string();
        write_live_alpha_risk_input(&risk_input, &risk);
        run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
            order_gate.clone(),
            risk_input,
            risk_preflight.clone(),
            true,
        ))
        .unwrap();
        write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight.clone(),
                second_preview.clone(),
                runtime_gate.clone(),
                true,
            ),
        )
        .unwrap();
        run_live_production_live_alpha_execution_dry_run(
            &production_live_alpha_execution_dry_run_opt(
                order_gate,
                risk_preflight,
                second_preview,
                runtime_gate,
                execution_output.clone(),
                true,
            ),
        )
        .unwrap();
        let execution: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(execution_output).unwrap()).unwrap();
        assert_eq!(execution["status"], "blocked_source_artifact");
        assert_eq!(execution["dry_run_execution_adapter_called"], false);
        assert_eq!(execution["production_adapter_called"], false);
        assert_eq!(execution["production_orders_submitted"], 0);
        assert_eq!(execution["production_order_mutations_attempted"], 0);
        assert_eq!(execution["network_attempted"], false);
        assert!(
            execution["source_artifact_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == "request_preview_not_built")
        );
    }

    #[test]
    fn production_live_alpha_order_request_preview_blocks_without_owner_scope() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v150-002-request-preview-blocked-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let output = output_dir.join("live_alpha_order_request_preview.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();

        let opt = production_live_alpha_order_request_preview_opt(
            order_gate,
            manual_approval_lifecycle,
            output.clone(),
            false,
        );
        run_live_production_live_alpha_order_request_preview_with_env(&opt, |_| None).unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_endpoint_or_owner_scope");
        assert_eq!(
            artifact["endpoint_class"],
            "production_mutation_scope_candidate"
        );
        assert_eq!(artifact["endpoint_decision"], "deny");
        assert_eq!(artifact["request_preview_allowed"], false);
        assert_eq!(artifact["request_preview_built"], false);
        assert_eq!(artifact["manual_approval_lifecycle_valid"], true);
        assert_eq!(artifact["signed_request_memory_only"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["execution_adapter_called"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 10);
        assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["credential_material"], "synthetic");
        assert_eq!(artifact["production_signing_material_env_read"], false);
    }

    #[test]
    fn production_live_alpha_order_request_preview_blocks_invalid_manual_approval_lifecycle() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v150-005-approval-lifecycle-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();

        for case in [
            ManualApprovalLifecycleCase {
                name: "pending",
                approval_state: "pending",
                run_id: "v150-live-alpha-request-preview",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
                expected_issue: "manual_approval_not_approved",
            },
            ManualApprovalLifecycleCase {
                name: "expired",
                approval_state: "expired",
                run_id: "v150-live-alpha-request-preview",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_070_000,
                expires_at_unix_ms: 1_718_400_060_000,
                expected_issue: "manual_approval_expired",
            },
            ManualApprovalLifecycleCase {
                name: "revoked",
                approval_state: "revoked",
                run_id: "v150-live-alpha-request-preview",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
                expected_issue: "manual_approval_revoked",
            },
            ManualApprovalLifecycleCase {
                name: "used",
                approval_state: "used",
                run_id: "v150-live-alpha-request-preview",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
                expected_issue: "manual_approval_used",
            },
            ManualApprovalLifecycleCase {
                name: "run-id-mismatch",
                approval_state: "approved",
                run_id: "wrong-run-id",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
                expected_issue: "manual_approval_run_id_mismatch",
            },
            ManualApprovalLifecycleCase {
                name: "symbol-mismatch",
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                symbol: "ETHUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
                expected_issue: "manual_approval_symbol_mismatch",
            },
            ManualApprovalLifecycleCase {
                name: "notional-mismatch",
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                symbol: "BTCUSDT",
                notional: "11.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
                expected_issue: "manual_approval_notional_mismatch",
            },
        ] {
            let approval = output_dir.join(format!("manual_approval_{}.json", case.name));
            let output = output_dir.join(format!("request_preview_{}.json", case.name));
            run_live_production_live_alpha_manual_approval_lifecycle(
                &production_live_alpha_manual_approval_lifecycle_opt(
                    approval.clone(),
                    &ManualApprovalLifecycleFixture {
                        approval_state: case.approval_state,
                        run_id: case.run_id,
                        strategy_id: "ema_cross_btcusdt_v1",
                        symbol: case.symbol,
                        notional: case.notional,
                        now_unix_ms: case.now_unix_ms,
                        expires_at_unix_ms: case.expires_at_unix_ms,
                    },
                ),
            )
            .unwrap();
            let opt = production_live_alpha_order_request_preview_opt(
                order_gate.clone(),
                approval,
                output.clone(),
                true,
            );
            run_live_production_live_alpha_order_request_preview_with_env(
                &opt,
                |name| match name {
                    "NTPRO_V150002_API_KEY" => {
                        Some("ntpro_v150005_synthetic_api_key_value".to_string())
                    }
                    "NTPRO_V150002_API_SECRET" => {
                        Some("ntpro_v150005_synthetic_api_secret_value".to_string())
                    }
                    _ => None,
                },
            )
            .unwrap();

            let artifact: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
            assert_eq!(artifact["status"], "blocked_manual_approval_lifecycle");
            assert_eq!(artifact["manual_approval_lifecycle_valid"], false);
            assert_eq!(artifact["request_preview_allowed"], false);
            assert_eq!(artifact["request_preview_built"], false);
            assert_eq!(artifact["request_sent"], false);
            assert_eq!(artifact["production_orders_submitted"], 0);
            assert_eq!(artifact["production_order_mutations_attempted"], 0);
            assert_eq!(artifact["network_attempted"], false);
            assert!(
                artifact["manual_approval_lifecycle_issues"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|issue| issue == case.expected_issue),
                "missing {} in {:?}",
                case.expected_issue,
                artifact["manual_approval_lifecycle_issues"]
            );
        }
    }

    #[test]
    fn production_live_alpha_execution_dry_run_routes_only_to_local_dry_run_adapter() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v150-003-execution-dry-run-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (order_gate, risk_preflight, request_preview) =
            write_ready_live_alpha_artifact_chain(&output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
        let output = output_dir.join("live_alpha_execution_dry_run.json");

        run_live_production_kill_switch_approval_artifact(
            &LiveProductionKillSwitchApprovalArtifactOpt {
                run_id: "v150-live-alpha-kill-switch-runtime-gate".to_string(),
                session_id: Some("session-v150".to_string()),
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                output: kill_switch_approval.clone(),
                kill_switch_active: false,
                approval_state: "approved".to_string(),
                manual_approval_id: Some("owner-approval-v150-004".to_string()),
                approved_by: Some("owner".to_string()),
                confirm_dry_run_only: true,
                confirm_no_production_mutation: true,
                confirm_dashboard_order_controls_disabled: true,
            },
        )
        .unwrap();
        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight.clone(),
                request_preview.clone(),
                kill_switch_runtime_gate.clone(),
                true,
            ),
        )
        .unwrap();

        run_live_production_live_alpha_execution_dry_run(
            &production_live_alpha_execution_dry_run_opt(
                order_gate,
                risk_preflight,
                request_preview,
                kill_switch_runtime_gate,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_LIVE_ALPHA_EXECUTION_DRY_RUN_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_dry_run_execution_adapter_only");
        assert_eq!(
            artifact["execution_decision"],
            "dry_run_adapter_artifact_only"
        );
        assert_eq!(
            artifact["execution_boundary_contract_version"],
            "ntpro.v151_execution_dry_run_adapter_boundary.v1"
        );
        assert_eq!(
            artifact["execution_boundary_flow"],
            "StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter"
        );
        assert_eq!(artifact["execution_boundary_contract_ready"], true);
        assert_eq!(artifact["strategy_intent_boundary"], "StrategyIntent");
        assert_eq!(artifact["risk_decision_boundary"], "RiskDecision");
        assert_eq!(artifact["execution_command_boundary"], "ExecutionCommand");
        assert_eq!(artifact["execution_command_created"], true);
        assert_eq!(artifact["execution_command_route"], "dry_run_adapter_only");
        assert_eq!(
            artifact["execution_command_destination"],
            "ntpro_local_artifact_dry_run_execution_adapter"
        );
        assert_eq!(
            artifact["dry_run_adapter_boundary"],
            "DryRunExecutionAdapter"
        );
        assert_eq!(artifact["dry_run_adapter_route_allowed"], true);
        assert_eq!(
            artifact["production_adapter_boundary"],
            "ProductionExecutionAdapter"
        );
        assert_eq!(artifact["production_adapter_route_allowed"], false);
        assert_eq!(artifact["production_adapter_instantiation_allowed"], false);
        assert_eq!(artifact["dry_run_execution_adapter_called"], true);
        assert_eq!(artifact["dry_run_execution_adapter_wrote_artifact"], true);
        assert_eq!(artifact["dry_run_adapter_artifact_only"], true);
        assert_eq!(artifact["real_execution_adapter_called"], false);
        assert_eq!(artifact["production_adapter_instantiated"], false);
        assert_eq!(artifact["production_adapter_called"], false);
        assert_eq!(artifact["strategy_intent_recorded"], true);
        assert_eq!(artifact["strategy_intent_reaches_risk_preflight"], true);
        assert_eq!(artifact["strategy_intent_reaches_dry_run_adapter"], true);
        assert_eq!(
            artifact["strategy_intent_reaches_production_adapter"],
            false
        );
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["order_gate_ready"], true);
        assert_eq!(artifact["risk_preflight_decision"], "dry_run_approved");
        assert_eq!(artifact["request_preview_built"], true);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(
            artifact["kill_switch_runtime_gate_status"],
            "ready_runtime_gate_open_for_dry_run_only"
        );
        assert_eq!(artifact["kill_switch_runtime_gate_open"], true);
        assert_eq!(artifact["symbol"], "BTCUSDT");
        assert_eq!(artifact["side"], "BUY");
        assert_eq!(artifact["order_type"], "LIMIT");
        assert_eq!(artifact["quantity"], "0.001");
        assert_eq!(artifact["price"], "10000.00");
        assert_eq!(artifact["time_in_force"], "GTC");
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["order_endpoint_access_attempted"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["real_funds"], false);
        assert_eq!(artifact["production_trading_enabled"], false);
        assert_eq!(artifact["values_are_exchange_truth"], false);
    }

    #[test]
    fn production_live_alpha_execution_dry_run_blocks_without_owner_scope() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v150-003-execution-dry-run-blocked-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (order_gate, risk_preflight, request_preview) =
            write_ready_live_alpha_artifact_chain(&output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
        let output = output_dir.join("live_alpha_execution_dry_run.json");

        run_live_production_kill_switch_approval_artifact(
            &LiveProductionKillSwitchApprovalArtifactOpt {
                run_id: "v150-live-alpha-kill-switch-runtime-gate".to_string(),
                session_id: Some("session-v150".to_string()),
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                output: kill_switch_approval.clone(),
                kill_switch_active: false,
                approval_state: "approved".to_string(),
                manual_approval_id: Some("owner-approval-v150-004".to_string()),
                approved_by: Some("owner".to_string()),
                confirm_dry_run_only: true,
                confirm_no_production_mutation: true,
                confirm_dashboard_order_controls_disabled: true,
            },
        )
        .unwrap();
        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight.clone(),
                request_preview.clone(),
                kill_switch_runtime_gate.clone(),
                true,
            ),
        )
        .unwrap();

        run_live_production_live_alpha_execution_dry_run(
            &production_live_alpha_execution_dry_run_opt(
                order_gate,
                risk_preflight,
                request_preview,
                kill_switch_runtime_gate,
                output.clone(),
                false,
            ),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["execution_decision"], "blocked_no_adapter_route");
        assert_eq!(
            artifact["execution_boundary_contract_version"],
            "ntpro.v151_execution_dry_run_adapter_boundary.v1"
        );
        assert_eq!(artifact["execution_boundary_contract_ready"], false);
        assert_eq!(artifact["execution_command_created"], false);
        assert_eq!(
            artifact["execution_command_route"],
            "blocked_before_execution_command"
        );
        assert_eq!(artifact["execution_command_destination"], "none");
        assert_eq!(artifact["dry_run_adapter_route_allowed"], false);
        assert_eq!(artifact["production_adapter_route_allowed"], false);
        assert_eq!(artifact["production_adapter_instantiation_allowed"], false);
        assert_eq!(artifact["dry_run_execution_adapter_called"], false);
        assert_eq!(artifact["dry_run_execution_adapter_wrote_artifact"], false);
        assert_eq!(artifact["dry_run_adapter_artifact_only"], false);
        assert_eq!(artifact["production_adapter_instantiated"], false);
        assert_eq!(artifact["production_adapter_called"], false);
        assert_eq!(
            artifact["strategy_intent_reaches_production_adapter"],
            false
        );
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 10);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
    }

    #[test]
    fn production_live_alpha_kill_switch_runtime_gate_blocks_active_switch() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v150-004-kill-switch-active-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (_, risk_preflight, request_preview) =
            write_ready_live_alpha_artifact_chain(&output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let output = output_dir.join("kill_switch_runtime_gate.json");
        write_kill_switch_approval_artifact(kill_switch_approval.clone(), true, "approved");

        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight,
                request_preview,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "blocked_kill_switch_active");
        assert_eq!(
            artifact["runtime_gate_decision"],
            "blocked_no_runtime_mutation"
        );
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["kill_switch_active"], true);
        assert_eq!(artifact["manual_approval_recorded"], true);
        assert_eq!(artifact["request_preview_built"], true);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert!(
            artifact["runtime_gate_reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "kill_switch_active")
        );
    }

    #[test]
    fn production_live_alpha_kill_switch_runtime_gate_blocks_missing_approval() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v150-004-kill-switch-missing-approval-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (_, risk_preflight, request_preview) =
            write_ready_live_alpha_artifact_chain(&output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let output = output_dir.join("kill_switch_runtime_gate.json");
        write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "pending");

        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight,
                request_preview,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_manual_approval");
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["kill_switch_active"], false);
        assert_eq!(artifact["approval_state"], "pending");
        assert_eq!(artifact["manual_approval_recorded"], false);
        assert_eq!(artifact["request_preview_built"], true);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert!(
            artifact["runtime_gate_reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "manual_approval_missing_or_not_approved")
        );
    }

    #[test]
    fn production_live_alpha_kill_switch_runtime_gate_blocks_request_preview() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v150-004-kill-switch-blocked-preview-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
        let risk_input = output_dir.join("live_alpha_risk_input.json");
        let risk_preflight = output_dir.join("live_alpha_risk_preflight.json");
        let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
        let request_preview = output_dir.join("live_alpha_order_request_preview.json");
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let output = output_dir.join("kill_switch_runtime_gate.json");

        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        let mut input = passing_live_alpha_risk_input();
        input.order.order_type = "LIMIT".to_string();
        write_live_alpha_risk_input(&risk_input, &input);
        run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
            order_gate.clone(),
            risk_input,
            risk_preflight.clone(),
            true,
        ))
        .unwrap();
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                manual_approval_lifecycle.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: "approved",
                    run_id: "v150-live-alpha-request-preview",
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: "BTCUSDT",
                    notional: "10.00",
                    now_unix_ms: 1_718_400_000_000,
                    expires_at_unix_ms: 1_718_400_060_000,
                },
            ),
        )
        .unwrap();
        let mut request_opt = production_live_alpha_order_request_preview_opt(
            order_gate,
            manual_approval_lifecycle,
            request_preview.clone(),
            true,
        );
        request_opt.credential_material = "production_live_alpha".to_string();
        run_live_production_live_alpha_order_request_preview_with_env(&request_opt, |name| {
            match name {
                PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
                | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => None,
                "NTPRO_V150002_API_KEY" | "NTPRO_V150002_API_SECRET" => {
                    panic!("blocked production signing material must not read {name}")
                }
                _ => None,
            }
        })
        .unwrap();
        write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");

        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight,
                request_preview,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_request_preview");
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["kill_switch_active"], false);
        assert_eq!(artifact["manual_approval_recorded"], true);
        assert_eq!(
            artifact["request_preview_status"],
            "blocked_endpoint_or_owner_scope"
        );
        assert_eq!(artifact["request_preview_built"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert!(
            artifact["runtime_gate_reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "request_preview_blocked")
        );
    }

    #[test]
    fn production_mutation_runtime_gate_blocks_missing_signing_approval() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-002-runtime-gate-signing-missing-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (order_gate, risk_preflight, request_preview) =
            write_ready_live_alpha_artifact_chain(&output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
        let output = output_dir.join("production_mutation_runtime_gate.json");
        write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight.clone(),
                request_preview.clone(),
                kill_switch_runtime_gate.clone(),
                true,
            ),
        )
        .unwrap();

        run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
            order_gate,
            risk_preflight,
            request_preview,
            kill_switch_runtime_gate,
            None,
            output.clone(),
            true,
        ))
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_RUNTIME_GATE_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "blocked_signing_approval");
        assert_eq!(
            artifact["capability"],
            "Minimum Owner-Approved Production Order Mutation Candidate"
        );
        assert_eq!(artifact["capability_expansion_from_v15"], true);
        assert_eq!(artifact["default_fail_closed"], true);
        assert_eq!(
            artifact["runtime_gate_decision"],
            "blocked_before_any_send_consideration"
        );
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["send_consideration_allowed"], false);
        assert_eq!(artifact["owner_approval_required"], true);
        assert_eq!(artifact["owner_approval_consumed"], true);
        assert_eq!(artifact["manual_approval_consumed"], true);
        assert_eq!(
            artifact["manual_approval_consume_status"],
            "approval_consumed_after_request_preview_created"
        );
        assert_eq!(artifact["kill_switch_checked_before_send"], true);
        assert_eq!(artifact["kill_switch_runtime_gate_open"], true);
        assert_eq!(artifact["kill_switch_active"], false);
        assert_eq!(artifact["risk_preflight_decision"], "dry_run_approved");
        assert_eq!(artifact["request_preview_built"], true);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["signing_approval_required"], true);
        assert_eq!(artifact["signing_approval_ready"], false);
        assert_eq!(artifact["explicit_send_gate_required"], true);
        assert_eq!(artifact["explicit_send_gate_open"], false);
        assert_eq!(artifact["single_order_candidate"], true);
        assert_eq!(artifact["tiny_notional_gate_ready"], true);
        assert_eq!(artifact["order_type"], "LIMIT");
        assert_eq!(artifact["time_in_force"], "GTC");
        assert_eq!(artifact["notional"], "10.00");
        assert_eq!(
            artifact["production_order_submission_allowed_policy"],
            "owner_approved_single_limit_gtc_only"
        );
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert!(
            artifact["runtime_gate_reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "signing_approval_missing")
        );
        assert!(
            artifact["runtime_gate_reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "explicit_send_gate_closed")
        );
    }

    #[test]
    fn production_mutation_signing_approval_ready_for_production_material_preview() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-003-signing-approval-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (_, _, request_preview) =
            write_ready_live_alpha_production_material_artifact_chain(&output_dir);
        let output = output_dir.join("production_mutation_signing_approval.json");

        run_live_production_mutation_signing_approval(&production_mutation_signing_approval_opt(
            request_preview,
            output.clone(),
            true,
        ))
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v160003_production_like_api_key_value"));
        assert!(!body.contains("ntpro_v160003_production_like_api_secret_value"));
        assert!(!body.contains("signature="));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_signing_material_approval");
        assert_eq!(artifact["credential_material"], "production_live_alpha");
        assert_eq!(artifact["approval_state"], "approved");
        assert_eq!(artifact["manual_approval_recorded"], true);
        assert_eq!(artifact["owner_approval_required"], true);
        assert_eq!(artifact["owner_approved_signing_material"], true);
        assert_eq!(artifact["signing_approval_ready"], true);
        assert_eq!(artifact["production_signing_material_gate_required"], true);
        assert_eq!(artifact["production_signing_material_gate_open"], true);
        assert_eq!(artifact["production_signing_material_env_read"], true);
        assert_eq!(
            artifact["production_signing_material_missing_gate_env_vars"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["request_body_recorded"], false);
        assert_eq!(artifact["raw_request_body_recorded"], false);
        assert_eq!(artifact["request_preview_built"], true);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["real_funds"], false);
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
    }

    #[test]
    fn production_mutation_signing_approval_blocks_synthetic_preview() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-003-signing-approval-synthetic-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (_, _, request_preview) = write_ready_live_alpha_artifact_chain(&output_dir);
        let output = output_dir.join("production_mutation_signing_approval.json");

        run_live_production_mutation_signing_approval(&production_mutation_signing_approval_opt(
            request_preview,
            output.clone(),
            true,
        ))
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_request_preview");
        assert_eq!(artifact["credential_material"], "synthetic");
        assert_eq!(artifact["signing_approval_ready"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert!(
            artifact["source_artifact_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == "request_preview_not_production_live_alpha_material")
        );
    }

    #[test]
    fn production_mutation_runtime_gate_accepts_ready_signing_approval_but_blocks_send_gate() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-003-runtime-gate-signing-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (order_gate, risk_preflight, request_preview) =
            write_ready_live_alpha_production_material_artifact_chain(&output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
        let signing_approval = output_dir.join("production_mutation_signing_approval.json");
        let output = output_dir.join("production_mutation_runtime_gate.json");
        write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight.clone(),
                request_preview.clone(),
                kill_switch_runtime_gate.clone(),
                true,
            ),
        )
        .unwrap();
        run_live_production_mutation_signing_approval(&production_mutation_signing_approval_opt(
            request_preview.clone(),
            signing_approval.clone(),
            true,
        ))
        .unwrap();

        run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
            order_gate,
            risk_preflight,
            request_preview,
            kill_switch_runtime_gate,
            Some(signing_approval.clone()),
            output.clone(),
            true,
        ))
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_explicit_send_gate");
        assert_eq!(artifact["signing_approval_required"], true);
        assert_eq!(artifact["signing_approval_ready"], true);
        assert_eq!(
            artifact["signing_approval_status"],
            "ready_signing_material_approval"
        );
        assert_eq!(
            artifact["source_signing_approval_path"],
            signing_approval.display().to_string()
        );
        assert_eq!(artifact["explicit_send_gate_required"], true);
        assert_eq!(artifact["explicit_send_gate_open"], false);
        assert_eq!(
            artifact["runtime_gate_decision"],
            "blocked_before_any_send_consideration"
        );
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["send_consideration_allowed"], false);
        assert_eq!(artifact["single_order_candidate"], true);
        assert_eq!(artifact["tiny_notional_gate_ready"], true);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["execution_adapter_called"], false);
        assert_eq!(artifact["production_adapter_called"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert!(
            artifact["runtime_gate_reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "explicit_send_gate_closed")
        );
    }

    #[test]
    fn production_mutation_request_builder_builds_redacted_limit_gtc_object_without_send() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-004-request-builder-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (runtime_gate, signing_approval, request_preview) =
            write_ready_v160_request_builder_sources(&output_dir);
        let output = output_dir.join("production_mutation_request_builder.json");
        let production_api_key = "ntpro_v160003_production_like_api_key_value";
        let production_api_secret = "ntpro_v160003_production_like_api_secret_value";

        run_live_production_mutation_request_builder_with_env(
            &production_mutation_request_builder_opt(
                runtime_gate,
                signing_approval,
                request_preview,
                output.clone(),
                true,
            ),
            |name| match name {
                PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
                | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
                "NTPRO_V150002_API_KEY" => Some(production_api_key.to_string()),
                "NTPRO_V150002_API_SECRET" => Some(production_api_secret.to_string()),
                _ => None,
            },
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains(production_api_key));
        assert!(!body.contains(production_api_secret));
        assert!(!body.contains("symbol=BTCUSDT"));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_request_object_built_no_send");
        assert_eq!(artifact["request_builder_ready"], true);
        assert_eq!(artifact["request_object_built"], true);
        assert_eq!(
            artifact["runtime_gate_status"],
            "blocked_explicit_send_gate"
        );
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["send_consideration_allowed"], false);
        assert_eq!(
            artifact["signing_approval_status"],
            "ready_signing_material_approval"
        );
        assert_eq!(artifact["signing_approval_ready"], true);
        assert_eq!(artifact["explicit_send_gate_open"], false);
        assert_eq!(artifact["credential_material"], "production_live_alpha");
        assert_eq!(artifact["production_signing_material_gate_required"], true);
        assert_eq!(artifact["production_signing_material_gate_open"], true);
        assert_eq!(artifact["production_signing_material_env_read"], true);
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["api_key_header_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["request_body_recorded"], false);
        assert_eq!(artifact["raw_request_body_recorded"], false);
        assert_eq!(artifact["request_method"], "POST");
        assert_eq!(artifact["request_target"], TESTNET_ORDER_ENDPOINT_ORDER);
        assert_eq!(
            artifact["query_shape_without_signature"],
            "symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp"
        );
        assert_eq!(
            artifact["signed_query_shape"],
            "symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp&signature=<redacted>"
        );
        assert_eq!(artifact["order_type"], "LIMIT");
        assert_eq!(artifact["time_in_force"], "GTC");
        assert_eq!(artifact["single_order_candidate"], true);
        assert_eq!(artifact["tiny_notional_gate_ready"], true);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn production_mutation_request_builder_blocks_missing_confirmations_and_env_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-004-request-builder-missing-gates-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (runtime_gate, signing_approval, request_preview) =
            write_ready_v160_request_builder_sources(&output_dir);
        let output = output_dir.join("production_mutation_request_builder.json");

        run_live_production_mutation_request_builder_with_env(
            &production_mutation_request_builder_opt(
                runtime_gate,
                signing_approval,
                request_preview,
                output.clone(),
                false,
            ),
            |_| None,
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["request_builder_ready"], false);
        assert_eq!(artifact["request_object_built"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--allow-production-mutation-request-builder")
        );
        assert!(
            artifact["missing_env_vars"]
                .as_array()
                .unwrap()
                .iter()
                .any(|env| env == PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW)
        );
    }

    #[test]
    fn production_mutation_request_builder_rejects_non_limit_request_preview() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-004-request-builder-market-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (runtime_gate, signing_approval, request_preview) =
            write_ready_v160_request_builder_sources(&output_dir);
        let mut preview: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&request_preview).unwrap()).unwrap();
        preview["order_type"] = serde_json::Value::String("MARKET".to_string());
        fs::write(
            &request_preview,
            serde_json::to_string_pretty(&preview).unwrap(),
        )
        .unwrap();
        let output = output_dir.join("production_mutation_request_builder.json");

        run_live_production_mutation_request_builder_with_env(
            &production_mutation_request_builder_opt(
                runtime_gate,
                signing_approval,
                request_preview,
                output.clone(),
                true,
            ),
            |name| match name {
                PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
                | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
                "NTPRO_V150002_API_KEY" => Some("ntpro_v160004_api_key".to_string()),
                "NTPRO_V150002_API_SECRET" => Some("ntpro_v160004_api_secret".to_string()),
                _ => None,
            },
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_source_artifact");
        assert_eq!(artifact["request_builder_ready"], false);
        assert_eq!(artifact["request_object_built"], false);
        assert_eq!(artifact["order_type"], "MARKET");
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert!(
            artifact["source_artifact_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == "request_preview_not_limit")
        );
    }

    #[test]
    fn production_mutation_guarded_send_offline_ready_without_network() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-005-guarded-send-ready-offline-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (request_builder, request_preview, kill_switch_runtime_gate) =
            write_ready_v160_guarded_send_sources(&output_dir);
        let output = output_dir.join("production_mutation_guarded_send.json");

        run_live_production_mutation_guarded_send_with_env(
            &production_mutation_guarded_send_opt(
                request_builder,
                kill_switch_runtime_gate,
                request_preview,
                output.clone(),
                false,
                true,
            ),
            |_| None,
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v160005_production_like_api_key_value"));
        assert!(!body.contains("ntpro_v160005_production_like_api_secret_value"));
        assert!(!body.contains("symbol=BTCUSDT"));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION
        );
        assert_eq!(
            artifact["status"],
            "ready_guarded_send_path_offline_no_network"
        );
        assert_eq!(artifact["manual_online_requested"], false);
        assert_eq!(artifact["guarded_send_ready"], true);
        assert_eq!(artifact["send_path_evaluated"], true);
        assert_eq!(artifact["kill_switch_enforcement_ready"], true);
        assert_eq!(artifact["kill_switch_checked_before_send"], true);
        assert_eq!(artifact["kill_switch_checked_after_send"], true);
        assert_eq!(
            artifact["pre_send_kill_switch_snapshot_source"],
            artifact["post_send_kill_switch_snapshot_source"]
        );
        assert_eq!(
            artifact["pre_send_kill_switch_snapshot_hash"],
            artifact["post_send_kill_switch_snapshot_hash"]
        );
        assert!(
            artifact["pre_send_kill_switch_checked_at"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        );
        assert!(
            artifact["post_send_kill_switch_checked_at"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        );
        assert_eq!(artifact["pre_send_kill_switch_runtime_gate_open"], true);
        assert_eq!(artifact["pre_send_kill_switch_active"], false);
        assert_eq!(artifact["post_send_kill_switch_runtime_gate_open"], true);
        assert_eq!(artifact["post_send_kill_switch_active"], false);
        assert_eq!(artifact["post_send_kill_switch_clean"], true);
        assert_eq!(artifact["kill_switch_blocked_send"], false);
        assert_eq!(artifact["post_send_progression_blocked"], false);
        assert_eq!(artifact["manual_review_required"], false);
        assert_eq!(artifact["new_orders_blocked"], false);
        assert_eq!(artifact["single_shot_send_allowed"], false);
        assert_eq!(
            artifact["request_builder_status"],
            "ready_request_object_built_no_send"
        );
        assert_eq!(artifact["request_object_built"], true);
        assert_eq!(artifact["request_method"], "POST");
        assert_eq!(artifact["request_target"], TESTNET_ORDER_ENDPOINT_ORDER);
        assert_eq!(artifact["order_type"], "LIMIT");
        assert_eq!(artifact["time_in_force"], "GTC");
        assert_eq!(artifact["credential_material"], "production_live_alpha");
        assert_eq!(artifact["production_signing_material_env_read"], false);
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["api_key_header_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["request_body_recorded"], false);
        assert_eq!(artifact["raw_request_body_recorded"], false);
        assert_eq!(artifact["raw_exchange_response_recorded"], false);
        assert_eq!(artifact["response_body_recorded"], false);
        assert_eq!(artifact["response_redacted"], true);
        assert_eq!(artifact["error_code"], "not_attempted_offline");
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_request_attempted"], false);
        assert_eq!(artifact["http_send_attempted"], false);
        assert_eq!(artifact["exchange_ack_observed"], false);
        assert_eq!(artifact["exchange_order_id_observed"], false);
        assert_eq!(artifact["exchange_order_status_observed"], false);
        assert_eq!(artifact["confirmed_production_order_submission"], false);
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_order_state_reads_allowed"], false);
        assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["production_order_state_reads_attempted"], 0);
        assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["real_funds"], false);
        assert_eq!(artifact["platform_production_trading_enabled"], false);
        assert_eq!(artifact["production_trading_enabled"], false);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn production_mutation_guarded_send_blocks_manual_online_without_env_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-005-guarded-send-manual-missing-env-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (request_builder, request_preview, kill_switch_runtime_gate) =
            write_ready_v160_guarded_send_sources(&output_dir);
        let output = output_dir.join("production_mutation_guarded_send.json");

        run_live_production_mutation_guarded_send_with_env(
            &production_mutation_guarded_send_opt(
                request_builder,
                kill_switch_runtime_gate,
                request_preview,
                output.clone(),
                true,
                true,
            ),
            |_| None,
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_manual_online_gate");
        assert_eq!(artifact["manual_online_requested"], true);
        assert_eq!(artifact["guarded_send_ready"], false);
        assert_eq!(artifact["kill_switch_enforcement_ready"], true);
        assert_eq!(artifact["kill_switch_blocked_send"], false);
        assert_eq!(artifact["single_shot_send_allowed"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_request_attempted"], false);
        assert_eq!(artifact["http_send_attempted"], false);
        assert_eq!(artifact["exchange_ack_observed"], false);
        assert_eq!(artifact["confirmed_production_order_submission"], false);
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["production_trading_enabled"], false);
        assert!(
            artifact["missing_env_vars"]
                .as_array()
                .unwrap()
                .iter()
                .any(|env| env == PRODUCTION_MUTATION_HTTP_SEND_ENV_ALLOW)
        );
        assert!(
            artifact["missing_env_vars"]
                .as_array()
                .unwrap()
                .iter()
                .any(|env| env == PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW)
        );
    }

    #[test]
    fn production_mutation_guarded_send_blocks_missing_confirmations() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-005-guarded-send-missing-flags-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (request_builder, request_preview, kill_switch_runtime_gate) =
            write_ready_v160_guarded_send_sources(&output_dir);
        let output = output_dir.join("production_mutation_guarded_send.json");

        run_live_production_mutation_guarded_send_with_env(
            &production_mutation_guarded_send_opt(
                request_builder,
                kill_switch_runtime_gate,
                request_preview,
                output.clone(),
                false,
                false,
            ),
            |_| None,
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["guarded_send_ready"], false);
        assert_eq!(artifact["kill_switch_enforcement_ready"], true);
        assert_eq!(artifact["kill_switch_blocked_send"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_request_attempted"], false);
        assert_eq!(artifact["http_send_attempted"], false);
        assert_eq!(artifact["exchange_ack_observed"], false);
        assert_eq!(artifact["confirmed_production_order_submission"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["production_trading_enabled"], false);
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--allow-production-mutation-guarded-send")
        );
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--confirm-owner-approved-guarded-send")
        );
    }

    #[test]
    fn production_mutation_guarded_send_blocks_active_kill_switch() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-008-guarded-send-kill-switch-active-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (request_builder, request_preview, kill_switch_runtime_gate) =
            write_ready_v160_guarded_send_sources(&output_dir);
        let mut gate: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&kill_switch_runtime_gate).unwrap()).unwrap();
        gate["status"] = serde_json::Value::String("blocked_kill_switch_active".to_string());
        gate["runtime_gate_open"] = serde_json::Value::Bool(false);
        gate["kill_switch_active"] = serde_json::Value::Bool(true);
        fs::write(
            &kill_switch_runtime_gate,
            serde_json::to_string_pretty(&gate).unwrap(),
        )
        .unwrap();
        let output = output_dir.join("production_mutation_guarded_send.json");

        run_live_production_mutation_guarded_send_with_env(
            &production_mutation_guarded_send_opt(
                request_builder,
                kill_switch_runtime_gate,
                request_preview,
                output.clone(),
                false,
                true,
            ),
            |_| None,
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_kill_switch_enforcement");
        assert_eq!(artifact["guarded_send_ready"], false);
        assert_eq!(artifact["kill_switch_enforcement_ready"], false);
        assert_eq!(artifact["kill_switch_checked_before_send"], true);
        assert_eq!(artifact["kill_switch_checked_after_send"], true);
        assert_eq!(artifact["pre_send_kill_switch_runtime_gate_open"], false);
        assert_eq!(artifact["pre_send_kill_switch_active"], true);
        assert_eq!(artifact["post_send_kill_switch_runtime_gate_open"], false);
        assert_eq!(artifact["post_send_kill_switch_active"], true);
        assert_eq!(artifact["post_send_kill_switch_clean"], false);
        assert_eq!(artifact["kill_switch_blocked_send"], true);
        assert_eq!(artifact["post_send_progression_blocked"], true);
        assert_eq!(artifact["manual_review_required"], true);
        assert_eq!(artifact["new_orders_blocked"], true);
        assert_eq!(artifact["single_shot_send_allowed"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_request_attempted"], false);
        assert_eq!(artifact["http_send_attempted"], false);
        assert_eq!(artifact["exchange_ack_observed"], false);
        assert_eq!(artifact["confirmed_production_order_submission"], false);
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["production_trading_enabled"], false);
        assert!(
            artifact["source_artifact_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == "kill_switch_runtime_gate_not_open")
        );
        assert!(
            artifact["source_artifact_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == "kill_switch_active_before_send")
        );
    }

    #[test]
    fn production_mutation_guarded_send_reads_post_send_kill_switch_after_http_boundary() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v161-002-guarded-send-post-kill-switch-read-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (request_builder, request_preview, kill_switch_runtime_gate) =
            write_ready_v160_guarded_send_sources(&output_dir);
        let output = output_dir.join("production_mutation_guarded_send.json");
        let opt = production_mutation_guarded_send_opt(
            request_builder,
            kill_switch_runtime_gate.clone(),
            request_preview,
            output,
            true,
            true,
        );
        let credentials =
            EnvOnlyProductionMutationPreviewCredentials::from_guarded_send_opt(&opt, |name| {
                match name {
                    PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
                    | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED
                    | PRODUCTION_MUTATION_HTTP_SEND_ENV_ALLOW
                    | PRODUCTION_MUTATION_HTTP_SEND_ENV_OWNER_APPROVED
                    | PRODUCTION_MUTATION_HTTP_SEND_ENV_SINGLE_SHOT => Some("1".to_string()),
                    "NTPRO_V150002_API_KEY" => Some("ntpro_v161002_api_key".to_string()),
                    "NTPRO_V150002_API_SECRET" => Some("ntpro_v161002_api_secret".to_string()),
                    _ => None,
                }
            });
        let mut executor_called = false;
        let post_gate = kill_switch_runtime_gate;

        let artifact = build_production_mutation_guarded_send_artifact_with_executor(
            &opt,
            &credentials,
            |_| {
                executor_called = true;
                let mut gate: serde_json::Value =
                    serde_json::from_str(&fs::read_to_string(&post_gate).unwrap()).unwrap();
                gate["status"] =
                    serde_json::Value::String("blocked_kill_switch_active_after_send".to_string());
                gate["runtime_gate_open"] = serde_json::Value::Bool(false);
                gate["kill_switch_active"] = serde_json::Value::Bool(true);
                fs::write(&post_gate, serde_json::to_string_pretty(&gate).unwrap()).unwrap();
                ProductionMutationGuardedSendHttpResult::success(7, 200)
            },
        )
        .unwrap();

        assert!(executor_called);
        assert_eq!(artifact.status, "manual_online_send_attempt_recorded");
        assert!(artifact.guarded_send_ready);
        assert!(!artifact.kill_switch_enforcement_ready);
        assert!(artifact.request_sent);
        assert!(artifact.exchange_ack_observed);
        assert!(artifact.confirmed_production_order_submission);
        assert_eq!(artifact.production_orders_submitted, 1);
        assert_eq!(artifact.production_order_mutations_attempted, 1);
        assert!(artifact.pre_send_kill_switch_runtime_gate_open);
        assert!(!artifact.pre_send_kill_switch_active);
        assert!(!artifact.post_send_kill_switch_runtime_gate_open);
        assert!(artifact.post_send_kill_switch_active);
        assert_ne!(
            artifact.pre_send_kill_switch_snapshot_hash,
            artifact.post_send_kill_switch_snapshot_hash
        );
        assert!(!artifact.post_send_kill_switch_clean);
        assert!(!artifact.kill_switch_blocked_send);
        assert!(artifact.post_send_progression_blocked);
        assert!(artifact.manual_review_required);
        assert!(artifact.new_orders_blocked);
        assert!(
            artifact
                .source_artifact_issues
                .iter()
                .any(|issue| { issue == "post_send_kill_switch_not_clean" })
        );
        assert!(!artifact.retry_attempted);
        assert!(!artifact.cancel_attempted);
        assert!(!artifact.replace_attempted);
        assert!(!artifact.amend_attempted);
        assert!(!artifact.flatten_attempted);
        assert!(!artifact.dashboard_order_controls_enabled);
    }

    #[test]
    fn production_mutation_response_redaction_persists_allowed_order_metadata_only() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-006-response-redaction-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let guarded_send = write_ready_v160_guarded_send_artifact(&output_dir);
        let response = output_dir.join("synthetic_order_response.json");
        let output = output_dir.join("production_mutation_response_redaction.json");
        write_synthetic_production_mutation_response(&response);

        run_live_production_mutation_response_redaction(
            &production_mutation_response_redaction_opt(
                guarded_send,
                response,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("X-MBX-APIKEY"));
        assert!(!body.contains("signature=must_not_persist"));
        assert!(!body.contains("\"headers\""));
        assert!(!body.contains("\"payload\""));
        assert!(!body.contains("\"balances\""));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_response_redacted");
        assert_eq!(artifact["response_redaction_ready"], true);
        assert_eq!(
            artifact["source_guarded_send_status"],
            "ready_guarded_send_path_offline_no_network"
        );
        assert_eq!(artifact["response_shape_validated"], true);
        assert_eq!(
            artifact["response_type"],
            "binance_order_response_redacted_metadata_v1"
        );
        assert_eq!(artifact["symbol"], "BTCUSDT");
        assert_eq!(artifact["side"], "BUY");
        assert_eq!(artifact["order_type"], "LIMIT");
        assert_eq!(artifact["time_in_force"], "GTC");
        assert_eq!(artifact["order_id"], "123456789");
        assert_eq!(
            artifact["client_order_id"],
            "owner-approved-v160-single-shot"
        );
        assert_eq!(artifact["exchange_status"], "NEW");
        assert_eq!(
            artifact["transact_time_shape"],
            "epoch_millis_present_redacted"
        );
        assert_eq!(
            artifact["working_time_shape"],
            "epoch_millis_present_redacted"
        );
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["api_key_header_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["request_body_recorded"], false);
        assert_eq!(artifact["raw_request_body_recorded"], false);
        assert_eq!(artifact["raw_exchange_response_recorded"], false);
        assert_eq!(artifact["response_body_recorded"], false);
        assert_eq!(artifact["response_headers_recorded"], false);
        assert_eq!(artifact["unrestricted_payload_recorded"], false);
        assert_eq!(artifact["account_balances_recorded"], false);
        assert_eq!(artifact["fills_recorded"], false);
        assert_eq!(artifact["response_redacted"], true);
        assert_eq!(
            artifact["forbidden_response_markers"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["real_funds"], false);
    }

    #[test]
    fn production_mutation_response_redaction_blocks_forbidden_response_markers() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-006-response-redaction-forbidden-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let guarded_send = write_ready_v160_guarded_send_artifact(&output_dir);
        let response = output_dir.join("forbidden_order_response.json");
        let output = output_dir.join("production_mutation_response_redaction.json");
        write_forbidden_production_mutation_response(&response);

        run_live_production_mutation_response_redaction(
            &production_mutation_response_redaction_opt(
                guarded_send,
                response,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_forbidden_response_marker");
        assert_eq!(artifact["response_redaction_ready"], false);
        assert_eq!(artifact["response_shape_validated"], false);
        assert!(
            artifact["forbidden_response_markers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|marker| marker.as_str().unwrap().contains("$.headers"))
        );
        assert!(
            artifact["forbidden_response_markers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|marker| marker.as_str().unwrap().contains("$.signature"))
        );
        assert_eq!(artifact["raw_exchange_response_recorded"], false);
        assert_eq!(artifact["response_headers_recorded"], false);
        assert_eq!(artifact["unrestricted_payload_recorded"], false);
        assert_eq!(artifact["account_balances_recorded"], false);
    }

    #[test]
    fn production_mutation_response_redaction_blocks_missing_confirmations() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-006-response-redaction-missing-flags-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let guarded_send = write_ready_v160_guarded_send_artifact(&output_dir);
        let response = output_dir.join("synthetic_order_response.json");
        let output = output_dir.join("production_mutation_response_redaction.json");
        write_synthetic_production_mutation_response(&response);

        run_live_production_mutation_response_redaction(
            &production_mutation_response_redaction_opt(
                guarded_send,
                response,
                output.clone(),
                false,
            ),
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["response_redaction_ready"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--allow-production-mutation-response-redaction")
        );
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--confirm-no-raw-response-persistence")
        );
    }

    #[test]
    fn production_mutation_order_state_readback_writes_ready_offline_known_order_contract() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-007-order-state-readback-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let response_redaction = write_ready_v160_response_redaction_artifact(&output_dir);
        let output = output_dir.join("production_mutation_order_state_readback.json");

        run_live_production_mutation_order_state_readback_with_env_and_http(
            &production_mutation_order_state_readback_opt(
                response_redaction,
                output.clone(),
                false,
                true,
            ),
            &mut |_| None,
            |_opt, _credentials, _recv_window_ms| {
                panic!("offline order-state readback must not call HTTP")
            },
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("X-MBX-APIKEY"));
        assert!(!body.contains("ntpro_v160007_api_key_value"));
        assert!(!body.contains("ntpro_v160007_api_secret_value"));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_ORDER_STATE_READBACK_SCHEMA_VERSION
        );
        assert_eq!(
            artifact["status"],
            "ready_offline_order_state_readback_contract"
        );
        assert_eq!(artifact["readback_contract_ready"], true);
        assert_eq!(
            artifact["source_response_redaction_status"],
            "ready_response_redacted"
        );
        assert_eq!(
            artifact["known_order_identifier_source"],
            "production_mutation_response_redaction"
        );
        assert_eq!(artifact["known_order_id"], "123456789");
        assert_eq!(
            artifact["known_client_order_id"],
            "owner-approved-v160-single-shot"
        );
        assert_eq!(artifact["symbol"], "BTCUSDT");
        assert_eq!(artifact["endpoint"], "order");
        assert_eq!(artifact["method"], "GET");
        assert_eq!(artifact["path"], "/api/v3/order");
        assert_eq!(artifact["manual_online_requested"], false);
        assert_eq!(artifact["order_state_read_allowed"], false);
        assert_eq!(artifact["order_state_read_attempted"], false);
        assert_eq!(artifact["response_shape"], "binance_order_state_v1");
        assert_eq!(artifact["response_shape_validated"], false);
        assert_eq!(artifact["strategy_success_inferred"], false);
        assert_eq!(
            artifact["strategy_success_proof"],
            "not_inferred_readback_is_observability_only"
        );
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["api_key_present"], false);
        assert_eq!(artifact["api_secret_present"], false);
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["api_key_header_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["raw_exchange_response_recorded"], false);
        assert_eq!(artifact["response_body_recorded"], false);
        assert_eq!(artifact["response_headers_recorded"], false);
        assert_eq!(artifact["response_redacted"], true);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_state_reads_allowed"], false);
        assert_eq!(artifact["production_order_state_reads_attempted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["production_trading_enabled"], false);
    }

    #[test]
    fn production_mutation_order_state_readback_blocks_missing_confirmations() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-007-order-state-readback-missing-flags-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let response_redaction = write_ready_v160_response_redaction_artifact(&output_dir);
        let output = output_dir.join("production_mutation_order_state_readback.json");

        run_live_production_mutation_order_state_readback_with_env_and_http(
            &production_mutation_order_state_readback_opt(
                response_redaction,
                output.clone(),
                false,
                false,
            ),
            &mut |_| None,
            |_opt, _credentials, _recv_window_ms| {
                panic!("blocked order-state readback must not call HTTP")
            },
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["readback_contract_ready"], false);
        assert_eq!(artifact["order_state_read_attempted"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_state_reads_attempted"], 0);
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--allow-production-mutation-order-state-readback")
        );
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--confirm-known-order-identifier-only")
        );
    }

    #[test]
    fn production_mutation_order_state_readback_blocks_manual_online_without_env_gates() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-007-order-state-readback-manual-missing-env-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let response_redaction = write_ready_v160_response_redaction_artifact(&output_dir);
        let output = output_dir.join("production_mutation_order_state_readback.json");
        let mut http_called = false;

        run_live_production_mutation_order_state_readback_with_env_and_http(
            &production_mutation_order_state_readback_opt(
                response_redaction,
                output.clone(),
                true,
                true,
            ),
            &mut |_| None,
            |_opt, _credentials, _recv_window_ms| {
                http_called = true;
                ProductionOrderStateHttpResult::success(
                    ProductionOrderStateReadEndpoint::Order,
                    1,
                    200,
                )
            },
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert!(!http_called);
        assert_eq!(artifact["status"], "blocked_missing_manual_online_gate");
        assert_eq!(artifact["manual_online_requested"], true);
        assert_eq!(artifact["readback_contract_ready"], false);
        assert_eq!(artifact["order_state_read_allowed"], false);
        assert_eq!(artifact["order_state_read_attempted"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_state_reads_attempted"], 0);
        assert!(
            artifact["missing_env_vars"]
                .as_array()
                .unwrap()
                .iter()
                .any(|env| env == PRODUCTION_ORDER_STATE_ENV_ALLOW)
        );
        assert!(
            artifact["missing_env_vars"]
                .as_array()
                .unwrap()
                .iter()
                .any(|env| env == PRODUCTION_ORDER_STATE_ENV_MANUAL_ONLINE)
        );
    }

    #[test]
    fn production_mutation_order_state_readback_records_owner_gated_online_success() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-007-order-state-readback-online-success-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let response_redaction = write_ready_v160_response_redaction_artifact(&output_dir);
        let output = output_dir.join("production_mutation_order_state_readback.json");
        let mut read_env = |name: &str| match name {
            "NTPRO_V160007_API_KEY" => Some("ntpro_v160007_api_key_value".to_string()),
            "NTPRO_V160007_API_SECRET" => Some("ntpro_v160007_api_secret_value".to_string()),
            _ => all_env_enabled(name),
        };

        run_live_production_mutation_order_state_readback_with_env_and_http(
            &production_mutation_order_state_readback_opt(
                response_redaction,
                output.clone(),
                true,
                true,
            ),
            &mut read_env,
            |proof_opt, credentials, recv_window_ms| {
                assert_eq!(proof_opt.endpoint, ProductionOrderStateReadEndpoint::Order);
                assert_eq!(proof_opt.symbol, "BTCUSDT");
                assert_eq!(proof_opt.order_id, Some(123_456_789));
                assert_eq!(
                    proof_opt.orig_client_order_id.as_deref(),
                    Some("owner-approved-v160-single-shot")
                );
                assert!(credentials.api_key_present());
                assert!(credentials.api_secret_present());
                assert_eq!(recv_window_ms, 5_000);
                ProductionOrderStateHttpResult::success(
                    ProductionOrderStateReadEndpoint::Order,
                    17,
                    200,
                )
            },
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v160007_api_key_value"));
        assert!(!body.contains("ntpro_v160007_api_secret_value"));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(artifact["status"], "online_order_state_read_ok");
        assert_eq!(artifact["readback_contract_ready"], true);
        assert_eq!(artifact["manual_online_requested"], true);
        assert_eq!(artifact["order_state_read_allowed"], true);
        assert_eq!(artifact["order_state_read_attempted"], true);
        assert_eq!(artifact["network_attempted"], true);
        assert_eq!(artifact["production_order_state_reads_allowed"], true);
        assert_eq!(artifact["production_order_state_reads_attempted"], 1);
        assert_eq!(artifact["response_status_code"], 200);
        assert_eq!(artifact["response_shape"], "binance_order_state_v1");
        assert_eq!(artifact["response_shape_validated"], true);
        assert_eq!(artifact["endpoint_shape_validated"], true);
        assert_eq!(artifact["order_entries_observed"], 1);
        assert_eq!(artifact["non_empty_order_state_observed"], true);
        assert_eq!(artifact["order_lifecycle_readiness"], true);
        assert_eq!(artifact["latency_ms"], 17);
        assert_eq!(artifact["error_code"], "none");
        assert_eq!(artifact["strategy_success_inferred"], false);
        assert_eq!(
            artifact["strategy_success_proof"],
            "not_inferred_readback_is_observability_only"
        );
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["real_orders_submitted"], false);
        assert_eq!(artifact["production_trading_enabled"], false);
    }

    #[test]
    fn production_mutation_audit_trail_links_redacted_candidate_chain() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-009-audit-trail-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (request_builder, guarded_send, response_redaction, order_state_readback) =
            write_ready_v160_audit_trail_sources(&output_dir);
        let output = output_dir.join("production_mutation_audit_trail.json");

        run_live_production_mutation_audit_trail(&production_mutation_audit_trail_opt(
            request_builder,
            guarded_send,
            response_redaction,
            order_state_readback,
            output.clone(),
            true,
        ))
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v160005_production_like_api_key_value"));
        assert!(!body.contains("ntpro_v160005_production_like_api_secret_value"));
        assert!(!body.contains("signature="));
        assert!(!body.contains("X-MBX-APIKEY"));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_redacted_audit_trail");
        assert_eq!(artifact["audit_trail_ready"], true);
        assert!(
            artifact["preview_hash"]
                .as_str()
                .unwrap()
                .starts_with("fnv1a64:")
        );
        assert_eq!(
            artifact["signing_approval_status"],
            "ready_signing_material_approval"
        );
        assert_eq!(artifact["approval_state"], "approved");
        assert_eq!(artifact["manual_approval_recorded"], true);
        assert_eq!(artifact["manual_approval_id"], "owner-approval-v160-003");
        assert_eq!(artifact["approved_by"], "owner");
        assert_eq!(
            artifact["runtime_gate_status"],
            "blocked_explicit_send_gate"
        );
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["send_consideration_allowed"], false);
        assert_eq!(
            artifact["guarded_send_status"],
            "ready_guarded_send_path_offline_no_network"
        );
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(
            artifact["response_redaction_status"],
            "ready_response_redacted"
        );
        assert_eq!(artifact["response_redaction_ready"], true);
        assert_eq!(
            artifact["order_state_readback_status"],
            "ready_offline_order_state_readback_contract"
        );
        assert_eq!(artifact["readback_contract_ready"], true);
        assert_eq!(artifact["order_state_read_attempted"], false);
        assert_eq!(artifact["kill_switch_checked_before_send"], true);
        assert_eq!(artifact["kill_switch_checked_after_send"], true);
        assert_eq!(artifact["pre_send_kill_switch_runtime_gate_open"], true);
        assert_eq!(artifact["pre_send_kill_switch_active"], false);
        assert_eq!(artifact["post_send_kill_switch_runtime_gate_open"], true);
        assert_eq!(artifact["post_send_kill_switch_active"], false);
        assert_eq!(artifact["kill_switch_blocked_send"], false);
        assert_eq!(artifact["symbol"], "BTCUSDT");
        assert_eq!(artifact["side"], "BUY");
        assert_eq!(artifact["order_type"], "LIMIT");
        assert_eq!(artifact["time_in_force"], "GTC");
        assert_eq!(artifact["order_id"], "123456789");
        assert_eq!(
            artifact["client_order_id"],
            "owner-approved-v160-single-shot"
        );
        assert_eq!(artifact["exchange_status"], "NEW");
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["failure_state"], "none_recorded");
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["raw_exchange_response_recorded"], false);
        assert_eq!(artifact["response_body_recorded"], false);
        assert_eq!(artifact["response_headers_recorded"], false);
        assert_eq!(artifact["unrestricted_payload_recorded"], false);
        assert_eq!(artifact["account_balances_recorded"], false);
        assert_eq!(artifact["response_redacted"], true);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_order_state_reads_allowed"], false);
        assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["production_order_state_reads_attempted"], 0);
        assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["production_trading_enabled"], false);
    }

    #[test]
    fn production_mutation_audit_trail_blocks_missing_confirmations() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-009-audit-trail-missing-flags-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (request_builder, guarded_send, response_redaction, order_state_readback) =
            write_ready_v160_audit_trail_sources(&output_dir);
        let output = output_dir.join("production_mutation_audit_trail.json");

        run_live_production_mutation_audit_trail(&production_mutation_audit_trail_opt(
            request_builder,
            guarded_send,
            response_redaction,
            order_state_readback,
            output.clone(),
            false,
        ))
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["audit_trail_ready"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["failure_state"], "blocked_missing_gate");
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--allow-production-mutation-audit-trail")
        );
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--confirm-redacted-artifacts-only")
        );
    }

    #[test]
    fn production_mutation_failure_semantics_records_no_retry_for_failure_modes() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-010-failure-semantics-ready-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let audit_trail = write_ready_v160_audit_trail_artifact(&output_dir);

        for (mode, expected_state) in [
            (
                ProductionMutationFailureMode::Timeout,
                "timeout_write_evidence_and_stop",
            ),
            (
                ProductionMutationFailureMode::Http4xx,
                "http_4xx_write_evidence_and_stop",
            ),
            (
                ProductionMutationFailureMode::Http5xx,
                "http_5xx_write_evidence_and_stop",
            ),
            (
                ProductionMutationFailureMode::MalformedResponse,
                "malformed_response_write_evidence_and_stop",
            ),
            (
                ProductionMutationFailureMode::ReadbackMismatch,
                "readback_mismatch_write_evidence_and_stop",
            ),
            (
                ProductionMutationFailureMode::KillSwitchTransition,
                "kill_switch_transition_write_evidence_and_stop",
            ),
        ] {
            let output = output_dir.join(format!("failure_semantics_{}.json", mode.as_str()));
            run_live_production_mutation_failure_semantics(
                &production_mutation_failure_semantics_opt(
                    audit_trail.clone(),
                    output.clone(),
                    mode,
                    true,
                ),
            )
            .unwrap();

            let body = fs::read_to_string(output).unwrap();
            assert!(!body.contains("ntpro_v160005_production_like_api_key_value"));
            assert!(!body.contains("ntpro_v160005_production_like_api_secret_value"));
            assert!(!body.contains("signature="));
            assert!(!body.contains("X-MBX-APIKEY"));
            let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(
                artifact["schema_version"],
                PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION
            );
            assert_eq!(artifact["status"], "ready_failure_semantics_evidence");
            assert_eq!(artifact["failure_semantics_ready"], true);
            assert_eq!(artifact["failure_mode"], mode.as_str());
            assert_eq!(artifact["failure_state"], expected_state);
            assert_eq!(artifact["terminal_action"], "write_evidence_and_stop");
            assert_eq!(artifact["evidence_written"], true);
            assert_eq!(artifact["stop_after_evidence"], true);
            assert_eq!(artifact["strategy_continuation_allowed"], false);
            assert_eq!(
                artifact["source_audit_trail_status"],
                "ready_redacted_audit_trail"
            );
            assert_eq!(artifact["source_audit_trail_ready"], true);
            assert_eq!(artifact["source_failure_state"], "none_recorded");
            assert_eq!(artifact["retry_allowed"], false);
            assert_eq!(artifact["retry_attempted"], false);
            assert_eq!(artifact["retry_attempts"], 0);
            assert_eq!(artifact["max_retry_attempts"], 0);
            assert_eq!(artifact["cancel_attempted"], false);
            assert_eq!(artifact["replace_attempted"], false);
            assert_eq!(artifact["amend_attempted"], false);
            assert_eq!(artifact["correction_attempted"], false);
            assert_eq!(artifact["flatten_attempted"], false);
            assert_eq!(artifact["remediation_attempted"], false);
            assert_eq!(artifact["automatic_remediation_allowed"], false);
            assert_eq!(artifact["dashboard_order_controls_enabled"], false);
            assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
            assert_eq!(artifact["production_order_mutations_attempted"], 0);
            assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
            assert_eq!(
                artifact["source_artifact_issues"].as_array().unwrap().len(),
                0
            );
            assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
            assert_eq!(artifact["evidence_only_failure_handling_confirmed"], true);
            assert_eq!(artifact["no_retry_confirmed"], true);
            assert_eq!(
                artifact["no_automatic_cancel_replace_amend_confirmed"],
                true
            );
            assert_eq!(artifact["no_correction_or_flatten_confirmed"], true);
            assert_eq!(artifact["dashboard_controls_disabled_confirmed"], true);
            assert_eq!(artifact["no_strategy_continuation_confirmed"], true);
            assert_eq!(artifact["no_listen_key_lifecycle_confirmed"], true);
        }
    }

    #[test]
    fn production_mutation_failure_semantics_blocks_missing_confirmations() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-010-failure-semantics-missing-flags-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let audit_trail = write_ready_v160_audit_trail_artifact(&output_dir);
        let output = output_dir.join("failure_semantics_missing_flags.json");

        run_live_production_mutation_failure_semantics(&production_mutation_failure_semantics_opt(
            audit_trail,
            output.clone(),
            ProductionMutationFailureMode::Timeout,
            false,
        ))
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["failure_semantics_ready"], false);
        assert_eq!(artifact["failure_state"], "blocked_missing_gate");
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["remediation_attempted"], false);
        assert_eq!(artifact["strategy_continuation_allowed"], false);
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--allow-production-mutation-failure-semantics")
        );
        assert!(
            artifact["missing_cli_flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|flag| flag == "--confirm-no-retry")
        );
    }

    #[test]
    fn production_mutation_runtime_gate_blocks_missing_owner_confirmations() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-002-runtime-gate-missing-flags-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (order_gate, risk_preflight, request_preview) =
            write_ready_live_alpha_artifact_chain(&output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
        let output = output_dir.join("production_mutation_runtime_gate.json");
        write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight.clone(),
                request_preview.clone(),
                kill_switch_runtime_gate.clone(),
                true,
            ),
        )
        .unwrap();

        run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
            order_gate,
            risk_preflight,
            request_preview,
            kill_switch_runtime_gate,
            None,
            output.clone(),
            false,
        ))
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_missing_gate");
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["send_consideration_allowed"], false);
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 9);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_mutation_runtime_gate_blocks_active_kill_switch() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v160-002-runtime-gate-active-kill-switch-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (order_gate, risk_preflight, request_preview) =
            write_ready_live_alpha_artifact_chain(&output_dir);
        let kill_switch_approval = output_dir.join("kill_switch_approval.json");
        let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
        let output = output_dir.join("production_mutation_runtime_gate.json");
        write_kill_switch_approval_artifact(kill_switch_approval.clone(), true, "approved");
        run_live_production_live_alpha_kill_switch_runtime_gate(
            &production_live_alpha_kill_switch_runtime_gate_opt(
                kill_switch_approval,
                risk_preflight.clone(),
                request_preview.clone(),
                kill_switch_runtime_gate.clone(),
                true,
            ),
        )
        .unwrap();

        run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
            order_gate,
            risk_preflight,
            request_preview,
            kill_switch_runtime_gate,
            None,
            output.clone(),
            true,
        ))
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_kill_switch_active");
        assert_eq!(artifact["kill_switch_checked_before_send"], true);
        assert_eq!(artifact["kill_switch_active"], true);
        assert_eq!(artifact["runtime_gate_open"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert!(
            artifact["runtime_gate_reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "kill_switch_active")
        );
    }

    #[test]
    fn production_live_alpha_risk_preflight_approves_hypothetical_order_without_submission() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-004-risk-approved-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("order_gate.json");
        let input = output_dir.join("risk_input.json");
        let output = output_dir.join("risk_preflight.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        write_live_alpha_risk_input(&input, &passing_live_alpha_risk_input());

        run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
            order_gate,
            input,
            output.clone(),
            true,
        ))
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            report["schema_version"],
            PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_REPORT_SCHEMA_VERSION
        );
        assert_eq!(report["status"], "approved");
        assert_eq!(report["risk_decision"], "dry_run_approved");
        assert_eq!(
            report["execution_decision"],
            "blocked_no_production_mutation"
        );
        assert_eq!(report["reasons"].as_array().unwrap().len(), 0);
        assert_eq!(report["order_gate_ready"], true);
        assert_eq!(report["projected_position_notional"], "60");
        assert_eq!(report["production_order_submission_allowed"], false);
        assert_eq!(report["production_order_mutation_allowed"], false);
        assert_eq!(report["production_order_submissions_attempted"], 0);
        assert_eq!(report["production_orders_submitted"], 0);
        assert_eq!(report["production_order_mutations_attempted"], 0);
        assert_eq!(report["execution_adapter_called"], false);
        assert_eq!(report["order_endpoint_access_attempted"], false);
        assert_eq!(report["network_attempted"], false);
        assert_eq!(report["dashboard_order_controls_enabled"], false);
        assert_eq!(report["real_orders_submitted"], false);
    }

    #[test]
    fn production_live_alpha_risk_preflight_blocks_missing_confirmations() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-004-risk-missing-flags-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("order_gate.json");
        let input = output_dir.join("risk_input.json");
        let output = output_dir.join("risk_preflight.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        write_live_alpha_risk_input(&input, &passing_live_alpha_risk_input());

        run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
            order_gate,
            input,
            output.clone(),
            false,
        ))
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(report["status"], "blocked_missing_gate");
        assert_eq!(report["risk_decision"], "dry_run_rejected");
        assert_eq!(
            report["execution_decision"],
            "blocked_no_production_mutation"
        );
        assert_eq!(report["missing_cli_flags"].as_array().unwrap().len(), 5);
        assert!(
            report["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "missing_owner_dry_run_confirmation")
        );
        assert_eq!(report["execution_adapter_called"], false);
        assert_eq!(report["production_orders_submitted"], 0);
    }

    #[test]
    fn production_live_alpha_risk_preflight_rejects_risk_and_state_failures() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-004-risk-rejected-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("order_gate.json");
        let input = output_dir.join("risk_input.json");
        let output = output_dir.join("risk_preflight.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        let mut preflight = passing_live_alpha_risk_input();
        preflight.market.now_unix_ms = 5_000;
        preflight.market.max_age_ms = 100;
        preflight.account.readable = false;
        preflight.order_state.readable = false;
        preflight.order_state.open_order_count = 5;
        preflight.risk.kill_switch_active = true;
        preflight.order.notional = "30.00".to_string();
        preflight.limits.max_order_notional = "25.00".to_string();
        preflight.limits.current_position_notional = "90.00".to_string();
        preflight.limits.max_position_notional = "100.00".to_string();
        write_live_alpha_risk_input(&input, &preflight);

        run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
            order_gate,
            input,
            output.clone(),
            true,
        ))
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(report["status"], "rejected");
        assert_eq!(report["risk_decision"], "dry_run_rejected");
        assert_eq!(
            report["execution_decision"],
            "blocked_no_production_mutation"
        );
        let reasons = report["reasons"].as_array().unwrap();
        for expected in [
            "market_stale",
            "account_read_failed",
            "order_state_read_failed",
            "kill_switch_active",
            "notional_limit_exceeded",
            "position_limit_exceeded",
            "open_order_limit_exceeded",
        ] {
            assert!(
                reasons.iter().any(|reason| reason == expected),
                "missing reason {expected}: {reasons:?}"
            );
        }
        assert_eq!(report["execution_adapter_called"], false);
        assert_eq!(report["order_endpoint_access_attempted"], false);
        assert_eq!(report["production_orders_submitted"], 0);
        assert_eq!(report["network_attempted"], false);
    }

    #[test]
    fn production_live_alpha_risk_preflight_rejects_mutating_order_gate() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v140-004-risk-mutating-gate-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let order_gate = output_dir.join("order_gate.json");
        let input = output_dir.join("risk_input.json");
        let output = output_dir.join("risk_preflight.json");
        run_live_production_live_alpha_dry_run_order_gate(
            &production_live_alpha_dry_run_order_gate_opt(order_gate.clone(), true),
        )
        .unwrap();
        let mut gate_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&order_gate).unwrap()).unwrap();
        gate_json["production_orders_submitted"] = serde_json::json!(1);
        fs::write(
            &order_gate,
            serde_json::to_string_pretty(&gate_json).unwrap(),
        )
        .unwrap();
        write_live_alpha_risk_input(&input, &passing_live_alpha_risk_input());

        run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
            order_gate,
            input,
            output.clone(),
            true,
        ))
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(report["status"], "rejected");
        assert!(
            report["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "order_gate_production_orders_submitted_nonzero")
        );
        assert_eq!(report["execution_adapter_called"], false);
        assert_eq!(report["production_order_mutations_attempted"], 0);
    }

    #[test]
    fn production_kill_switch_approval_artifact_writes_no_mutation_contract() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v130-004-kill-switch-approval-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let output = output_dir.join("kill_switch_approval.json");

        run_live_production_kill_switch_approval_artifact(
            &LiveProductionKillSwitchApprovalArtifactOpt {
                run_id: "v130-live-alpha-preflight".to_string(),
                session_id: Some("session-1".to_string()),
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                output: output.clone(),
                kill_switch_active: true,
                approval_state: "approved".to_string(),
                manual_approval_id: Some("owner-approval-001".to_string()),
                approved_by: Some("owner".to_string()),
                confirm_dry_run_only: true,
                confirm_no_production_mutation: true,
                confirm_dashboard_order_controls_disabled: true,
            },
        )
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "manual_approval_recorded");
        assert_eq!(artifact["kill_switch_enabled"], true);
        assert_eq!(artifact["kill_switch_active"], true);
        assert_eq!(artifact["kill_switch_dry_run"], true);
        assert_eq!(artifact["manual_approval_required"], true);
        assert_eq!(artifact["manual_approval_recorded"], true);
        assert_eq!(artifact["manual_approval_id"], "owner-approval-001");
        assert_eq!(artifact["approved_by"], "owner");
        assert_eq!(artifact["approval_state"], "approved");
        assert_eq!(artifact["approval_artifact_only"], true);
        assert_eq!(
            artifact["owner_approval_required_before_any_mutation"],
            true
        );
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["production_order_state_reads_allowed"], false);
        assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
        assert_eq!(artifact["production_order_submissions_attempted"], 0);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["production_order_state_reads_attempted"], 0);
        assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
        assert_eq!(artifact["cancel_replace_amend_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["values_are_exchange_truth"], false);
    }

    #[test]
    fn production_kill_switch_approval_artifact_requires_dry_run_confirmations() {
        let output = std::env::temp_dir().join(format!(
            "ntpro-v130-004-kill-switch-approval-missing-confirm-{}.json",
            std::process::id()
        ));
        let err = build_production_kill_switch_approval_artifact(
            &LiveProductionKillSwitchApprovalArtifactOpt {
                run_id: "v130-live-alpha-preflight".to_string(),
                session_id: None,
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                output,
                kill_switch_active: true,
                approval_state: "pending".to_string(),
                manual_approval_id: None,
                approved_by: None,
                confirm_dry_run_only: false,
                confirm_no_production_mutation: true,
                confirm_dashboard_order_controls_disabled: true,
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("--confirm-dry-run-only"));
    }

    #[test]
    fn production_kill_switch_approval_artifact_requires_approved_fields() {
        let output = std::env::temp_dir().join(format!(
            "ntpro-v130-004-kill-switch-approval-missing-fields-{}.json",
            std::process::id()
        ));
        let err = build_production_kill_switch_approval_artifact(
            &LiveProductionKillSwitchApprovalArtifactOpt {
                run_id: "v130-live-alpha-preflight".to_string(),
                session_id: None,
                strategy_id: "ema_cross_btcusdt_v1".to_string(),
                output,
                kill_switch_active: true,
                approval_state: "approved".to_string(),
                manual_approval_id: None,
                approved_by: Some("owner".to_string()),
                confirm_dry_run_only: true,
                confirm_no_production_mutation: true,
                confirm_dashboard_order_controls_disabled: true,
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("--manual-approval-id"));
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

    #[test]
    fn production_readonly_reconciliation_classifies_ok() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-006-reconciliation-ok-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let strategy_status = output_dir.join("strategy_session_status.json");
        let shadow_strategy_session = output_dir.join("shadow_strategy_session.jsonl");
        let reconciliation = output_dir.join("reconciliation_events.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);
        fs::write(
            &strategy_status,
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
        run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
            run_id: "v120-shadow".to_string(),
            snapshot_id: Some("portfolio-1".to_string()),
            account_snapshot: account_snapshot.clone(),
            shadow_intent: shadow_intent.clone(),
            output: portfolio_runtime.clone(),
            compat_snapshot_output: None,
        })
        .unwrap();
        run_live_production_shadow_strategy_session(&LiveProductionShadowStrategySessionOpt {
            run_id: "v120-shadow".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            shadow_portfolio_runtime: portfolio_runtime.clone(),
            strategy_session_status: Some(strategy_status),
            output: shadow_strategy_session.clone(),
            heartbeat_count: 1,
            stop_after_heartbeats: false,
            stop_file: None,
        })
        .unwrap();

        run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
            run_id: "v120-shadow".to_string(),
            account_snapshot: Some(account_snapshot),
            shadow_portfolio_runtime: Some(portfolio_runtime),
            shadow_strategy_session: Some(shadow_strategy_session),
            shadow_intent: Some(shadow_intent),
            output: reconciliation.clone(),
        })
        .unwrap();

        let events = read_jsonl_values(&reconciliation);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["classification"], "ok");
        assert_eq!(events[0]["event_type"], "observed_account_state");
        assert_eq!(events[0]["severity"], "info");
        assert_eq!(events[0]["recommended_action"], "record_only");
        assert_eq!(events[0]["risk_halted"], false);
        assert_eq!(events[0]["production_order_submissions_attempted"], 0);
        assert_eq!(events[0]["production_order_mutations_attempted"], 0);
        assert_eq!(events[0]["production_order_state_reads_attempted"], 0);
        assert_eq!(events[0]["listen_key_lifecycle_attempted"], 0);
        assert_eq!(events[0]["dashboard_order_controls_enabled"], false);
    }

    #[test]
    fn production_readonly_reconciliation_classifies_missing_account_snapshot() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-006-reconciliation-missing-account-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let reconciliation = output_dir.join("reconciliation_events.jsonl");
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

        run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
            run_id: "v120-shadow".to_string(),
            account_snapshot: None,
            shadow_portfolio_runtime: Some(portfolio_runtime),
            shadow_strategy_session: None,
            shadow_intent: None,
            output: reconciliation.clone(),
        })
        .unwrap();

        let events = read_jsonl_values(&reconciliation);
        assert_eq!(events[0]["classification"], "missing_account_snapshot");
        assert_eq!(events[0]["severity"], "degraded");
        assert_eq!(events[0]["recommended_action"], "mark_degraded");
        assert_eq!(events[0]["risk_halted"], true);
    }

    #[test]
    fn production_readonly_reconciliation_classifies_shadow_intent_without_portfolio() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-006-reconciliation-intent-no-portfolio-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let reconciliation = output_dir.join("reconciliation_events.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);

        run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
            run_id: "v120-shadow".to_string(),
            account_snapshot: Some(account_snapshot),
            shadow_portfolio_runtime: None,
            shadow_strategy_session: None,
            shadow_intent: Some(shadow_intent),
            output: reconciliation.clone(),
        })
        .unwrap();

        let events = read_jsonl_values(&reconciliation);
        assert_eq!(
            events[0]["classification"],
            "shadow_intent_without_portfolio"
        );
        assert_eq!(events[0]["event_type"], "shadow_mismatch");
        assert_eq!(events[0]["severity"], "halt");
        assert_eq!(events[0]["recommended_action"], "manual_review_required");
        assert_eq!(events[0]["risk_halted"], true);
    }

    #[test]
    fn production_readonly_reconciliation_classifies_production_mutation_forbidden() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-006-reconciliation-mutation-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let reconciliation = output_dir.join("reconciliation_events.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
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
                "real_orders_submitted": false,
                "provenance": {
                    "values_are_exchange_truth": false
                }
            }))
            .unwrap(),
        )
        .unwrap();

        run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
            run_id: "v120-shadow".to_string(),
            account_snapshot: Some(account_snapshot),
            shadow_portfolio_runtime: Some(portfolio_runtime),
            shadow_strategy_session: None,
            shadow_intent: None,
            output: reconciliation.clone(),
        })
        .unwrap();

        let events = read_jsonl_values(&reconciliation);
        assert_eq!(events[0]["classification"], "production_mutation_forbidden");
        assert_eq!(events[0]["event_type"], "risk_halt");
        assert_eq!(events[0]["severity"], "halt");
        assert_eq!(events[0]["recommended_action"], "halt_shadow_flow");
        assert_eq!(events[0]["production_orders_submitted"], 0);
    }

    #[test]
    fn production_readonly_reconciliation_classifies_manual_review_required() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v120-006-reconciliation-manual-review-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
        let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
        let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
        let shadow_strategy_session = output_dir.join("shadow_strategy_session.jsonl");
        let reconciliation = output_dir.join("reconciliation_events.jsonl");
        write_redacted_account_snapshot_report(&account_snapshot, true);
        write_shadow_intent(&shadow_intent, false);
        run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
            run_id: "v120-shadow".to_string(),
            snapshot_id: Some("portfolio-1".to_string()),
            account_snapshot: account_snapshot.clone(),
            shadow_intent: shadow_intent.clone(),
            output: portfolio_runtime.clone(),
            compat_snapshot_output: None,
        })
        .unwrap();
        run_live_production_shadow_strategy_session(&LiveProductionShadowStrategySessionOpt {
            run_id: "v120-shadow".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            shadow_portfolio_runtime: portfolio_runtime.clone(),
            strategy_session_status: None,
            output: shadow_strategy_session.clone(),
            heartbeat_count: 1,
            stop_after_heartbeats: false,
            stop_file: None,
        })
        .unwrap();

        run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
            run_id: "v120-shadow".to_string(),
            account_snapshot: Some(account_snapshot),
            shadow_portfolio_runtime: Some(portfolio_runtime),
            shadow_strategy_session: Some(shadow_strategy_session),
            shadow_intent: Some(shadow_intent),
            output: reconciliation.clone(),
        })
        .unwrap();

        let events = read_jsonl_values(&reconciliation);
        assert_eq!(events[0]["classification"], "manual_review_required");
        assert_eq!(events[0]["event_type"], "manual_remediation_required");
        assert_eq!(events[0]["severity"], "warning");
        assert_eq!(events[0]["manual_review_required"], true);
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
