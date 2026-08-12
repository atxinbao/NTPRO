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
    collections::{BTreeSet, HashMap},
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use aws_lc_rs::digest;
use nautilus_binance::{
    common::{
        consts::BINANCE_API_KEY_HEADER,
        credential::SigningCredential,
        enums::{BinanceEnvironment, BinanceProductType},
    },
    config::{BinanceDataClientConfig, BinanceExecClientConfig},
    factories::{BinanceDataClientFactory, BinanceExecutionClientFactory},
};
use nautilus_common::{
    actor::{DataActor, DataActorCore, data_actor::DataActorConfig},
    enums::Environment,
    nautilus_actor,
};
use nautilus_core::string::urlencoding;
use nautilus_live::{
    config::LiveRiskEngineConfig,
    node::{LiveNode, NodeState},
    status::{
        ConnectionStatus, ExecutionStatus, LifecycleStatus, NodeStatus, ProcessMode, SnapshotValue,
    },
};
use nautilus_model::{
    data::{QuoteTick, TradeTick},
    enums::{OrderSide, TimeInForce},
    events::{
        OrderAccepted, OrderCanceled, OrderDenied, OrderExpired, OrderFilled, OrderRejected,
        OrderSubmitted,
    },
    identifiers::{AccountId, ClientId, ClientOrderId, InstrumentId, StrategyId, TraderId, Venue},
    instruments::{Instrument, InstrumentAny},
    orders::Order,
    types::{Money, Price, Quantity},
};
use nautilus_sandbox::{SandboxExecutionClientConfig, SandboxExecutionClientFactory};
use nautilus_trading::{
    nautilus_strategy,
    strategy::{Strategy, StrategyConfig, StrategyCore},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use tokio::time::{sleep, timeout};

use crate::{
    artifacts::{atomic_write_json, atomic_write_text},
    endpoint_classifier::{EndpointAuthKind, EndpointClassifier, EndpointDecision},
    opt::{
        LiveProductionAccountSnapshotContractOpt, LiveProductionKillSwitchApprovalArtifactOpt,
        LiveProductionLiveAlphaDryRunOrderGateOpt, LiveProductionLiveAlphaExecutionDryRunOpt,
        LiveProductionLiveAlphaKillSwitchRuntimeGateOpt,
        LiveProductionLiveAlphaManualApprovalLifecycleOpt,
        LiveProductionLiveAlphaOrderRequestPreviewOpt, LiveProductionLiveAlphaRiskPreflightOpt,
        LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt,
        LiveProductionMutationActualCancelFailureEvidenceOpt,
        LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt,
        LiveProductionMutationActualCancelReadbackReconciliationOpt,
        LiveProductionMutationActualCancelSingleShotOpt, LiveProductionMutationAuditTrailOpt,
        LiveProductionMutationCancelRecoveryIncidentAuditCloseoutOpt,
        LiveProductionMutationCancelRequestPreviewOpt,
        LiveProductionMutationCancelResponseRedactionOpt, LiveProductionMutationCancelRiskGateOpt,
        LiveProductionMutationExchangeReadbackMapperOpt, LiveProductionMutationFailureSemanticsOpt,
        LiveProductionMutationGuardedSendOpt, LiveProductionMutationLocalOrderLedgerOpt,
        LiveProductionMutationManualOwnerApprovalLifecycleOpt,
        LiveProductionMutationOrderStateReadbackOpt, LiveProductionMutationOrphanOrderDetectorOpt,
        LiveProductionMutationPostCancelReadbackOpt,
        LiveProductionMutationReconciliationClassifierOpt, LiveProductionMutationRequestBuilderOpt,
        LiveProductionMutationResponseRedactionOpt, LiveProductionMutationRuntimeGateOpt,
        LiveProductionMutationSigningApprovalOpt, LiveProductionOrderStateReadOnlyProofOpt,
        LiveProductionPublicReadProbeOpt, LiveProductionReadonlyReconciliationOpt,
        LiveProductionShadowPortfolioRuntimeOpt, LiveProductionShadowPreflightSessionOpt,
        LiveProductionShadowStrategySessionOpt, LiveRunOpt,
        LiveTestnetExecutionArtifactContractOpt, LiveTestnetOrderGateOpt,
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

mod command;
mod execution_strategy;
mod node_runtime;

use execution_strategy::ProductionSingleShotExecutionStrategy;

use node_runtime::{
    run_live_run_with_command, run_production_market_data_node_with_command,
    run_strategy_session_node_with_command,
};

pub(crate) use command::run_live_command;

const LIVE_INIT_SMOKE_MODE: &str = "live-init-smoke";
const PRODUCTION_MARKET_DATA_MODE: &str = "production-market-data";
const PRODUCTION_MARKET_DATA_SCHEMA_VERSION: &str = "ntpro.live_market_data_node.v1";
const PRODUCTION_EXECUTION_SCHEMA_VERSION: &str = "ntpro.s3.live_execution_node.v1";
const LIVE_ENVIRONMENT: &str = "live";
const BINANCE_SPOT_PRODUCT_TYPE: &str = "spot";
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
pub(crate) const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW: &str =
    "NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ";
pub(crate) const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED: &str =
    "NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY";
pub(crate) const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION: &str =
    "NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION";
pub(crate) const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE: &str =
    "NTPRO_CONFIRM_NO_SECRET_PERSISTENCE";
pub(crate) const PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE: &str = "NTPRO_V12_MANUAL_ONLINE";
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
const PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_owner_approval_lifecycle.v1";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_EXECUTOR_ADAPTER_BOUNDARY_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_executor_adapter_boundary.v1";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_single_shot.v1";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_readback_reconciliation.v1";
const PRODUCTION_MUTATION_ACTUAL_CANCEL_FAILURE_EVIDENCE_SCHEMA_VERSION: &str =
    "ntpro.v190_actual_cancel_failure_evidence.v1";
const PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION: &str =
    "ntpro.v180_cancel_response_redaction.v1";
const PRODUCTION_MUTATION_POST_CANCEL_READBACK_SCHEMA_VERSION: &str =
    "ntpro.v180_post_cancel_readback.v1";
const PRODUCTION_MUTATION_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_SCHEMA_VERSION: &str =
    "ntpro.v180_cancel_recovery_incident_audit_closeout.v1";
const PRODUCTION_MUTATION_EXCHANGE_ORDER_READBACK_SCHEMA_VERSION: &str =
    "ntpro.v170_redacted_binance_order_readback.v1";
const PRODUCTION_MUTATION_EXCHANGE_OPEN_ORDERS_READBACK_SCHEMA_VERSION: &str =
    "ntpro.v170_redacted_binance_open_orders_readback.v1";
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
#[serde(deny_unknown_fields)]
struct ProductionMarketDataNodeConfig {
    live_market_data: ProductionMarketDataSection,
    live_execution: Option<ProductionExecutionSection>,
    shutdown: LiveShutdownConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductionMarketDataSection {
    schema_version: String,
    mode: String,
    environment: String,
    node_id: String,
    trader_id: String,
    venue: String,
    product_type: String,
    symbols: Vec<String>,
    api_key_env: String,
    api_secret_env: String,
    execution_client_enabled: bool,
    order_endpoint_access_allowed: bool,
    order_submission_allowed: bool,
    automatic_reconnect_allowed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductionExecutionSection {
    schema_version: String,
    source_manifest_sha256: String,
    execution_admission_sha256: String,
    runtime_artifact_root: PathBuf,
    risk_policy_ref: String,
    owner_authority_ref: String,
    risk_authority_ref: String,
    operator_authority_ref: String,
    admission_id: String,
    strategy_version_id: String,
    account_id: String,
    instrument_id: String,
    side: String,
    order_type: String,
    time_in_force: String,
    price: String,
    quantity: String,
    max_notional: String,
    risk_policy_max_notional: String,
    expires_at_unix_ms: u64,
    api_key_env: String,
    api_secret_env: String,
    owner_confirmed: bool,
    risk_confirmed: bool,
    operator_confirmed: bool,
    kill_switch_active: bool,
    single_shot: bool,
    cancel_order_allowed: bool,
    replace_order_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_recovery_allowed: bool,
}

#[derive(Debug)]
struct ProductionMarketDataActor {
    core: DataActorCore,
    client_id: ClientId,
    instrument_ids: Vec<InstrumentId>,
    quote_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
    trade_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
    last_event_unix_ms: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

nautilus_actor!(ProductionMarketDataActor);

impl ProductionMarketDataActor {
    fn new(client_id: ClientId, instrument_ids: Vec<InstrumentId>) -> Self {
        Self {
            core: DataActorCore::new(DataActorConfig::default()),
            client_id,
            instrument_ids,
            quote_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            trade_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_event_unix_ms: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    fn counters(
        &self,
    ) -> (
        std::sync::Arc<std::sync::atomic::AtomicU64>,
        std::sync::Arc<std::sync::atomic::AtomicU64>,
        std::sync::Arc<std::sync::atomic::AtomicU64>,
    ) {
        (
            self.quote_count.clone(),
            self.trade_count.clone(),
            self.last_event_unix_ms.clone(),
        )
    }

    fn record_quote_event(&self) {
        self.quote_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.record_market_event_time();
    }

    fn record_trade_event(&self) {
        self.trade_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.record_market_event_time();
    }

    fn record_market_event_time(&self) {
        let observed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| millis_to_u64(duration.as_millis()));
        self.last_event_unix_ms
            .store(observed_at, std::sync::atomic::Ordering::Release);
    }
}

impl DataActor for ProductionMarketDataActor {
    fn on_start(&mut self) -> anyhow::Result<()> {
        for instrument_id in self.instrument_ids.clone() {
            self.subscribe_quotes(instrument_id, Some(self.client_id), None);
            self.subscribe_trades(instrument_id, Some(self.client_id), None);
        }
        Ok(())
    }

    fn on_quote(&mut self, _quote: &QuoteTick) -> anyhow::Result<()> {
        self.record_quote_event();
        Ok(())
    }

    fn on_trade(&mut self, _trade: &TradeTick) -> anyhow::Result<()> {
        self.record_trade_event();
        Ok(())
    }
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
    product_snapshot: Option<ProductLiveAccountSnapshot>,
    error_code: String,
    network_attempted: bool,
    diagnostic: String,
}

impl ProductionAccountSnapshotHttpResult {
    #[cfg(test)]
    fn success(latency_ms: u64, http_status: u16) -> Self {
        Self::success_with_product_snapshot(
            latency_ms,
            http_status,
            ProductionAccountSnapshotShapeSummary::accepted_fixture(),
            ProductLiveAccountSnapshot::accepted_fixture(),
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
            product_snapshot: None,
            error_code: "none".to_string(),
            network_attempted: true,
            diagnostic: format!(
                "V120 authenticated production account snapshot read succeeded with GET {PRODUCTION_ACCOUNT_SNAPSHOT_ENDPOINT} and HTTP {http_status}; raw account response, balances, uid, headers, signature, signed query, and signed URL were not recorded."
            ),
        }
    }

    fn success_with_product_snapshot(
        latency_ms: u64,
        http_status: u16,
        response_shape_summary: ProductionAccountSnapshotShapeSummary,
        product_snapshot: ProductLiveAccountSnapshot,
    ) -> Self {
        let mut result = Self::success_with_shape(latency_ms, http_status, response_shape_summary);
        result.product_snapshot = Some(product_snapshot);
        result
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
            product_snapshot: None,
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

/// 产品 API 可消费的生产账户只读观察结果。
///
/// 该类型只包含连接元数据和受限账户投影，不携带凭证、签名或原始响应。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProductLiveAccountReadObservation {
    pub status: String,
    pub network_attempted: bool,
    pub account_read_attempted: bool,
    pub response_status_code: Option<u16>,
    pub latency_ms: Option<u64>,
    pub response_shape: String,
    pub response_shape_validated: bool,
    pub account_type_present: bool,
    pub balance_entry_count: Option<usize>,
    pub permission_entry_count: Option<usize>,
    pub can_trade_present: bool,
    pub can_withdraw_present: bool,
    pub can_deposit_present: bool,
    pub account_snapshot: Option<ProductLiveAccountSnapshot>,
    pub error_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProductLiveAssetBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
    pub total: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProductLiveAccountSnapshot {
    pub account_type: String,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub source_balance_entry_count: usize,
    pub zero_balance_entry_count: usize,
    pub assets: Vec<ProductLiveAssetBalance>,
}

impl ProductLiveAccountSnapshot {
    #[cfg(test)]
    fn accepted_fixture() -> Self {
        Self {
            account_type: "SPOT".to_string(),
            can_trade: true,
            can_withdraw: false,
            can_deposit: true,
            source_balance_entry_count: 1,
            zero_balance_entry_count: 0,
            assets: vec![ProductLiveAssetBalance {
                asset: "BTC".to_string(),
                free: "0.125".to_string(),
                locked: "0".to_string(),
                total: "0.125".to_string(),
            }],
        }
    }
}

impl ProductLiveAccountReadObservation {
    pub(crate) fn blocked(error_code: &str) -> Self {
        Self {
            status: "blocked".to_string(),
            network_attempted: false,
            account_read_attempted: false,
            response_status_code: None,
            latency_ms: None,
            response_shape: production_account_snapshot_response_shape().to_string(),
            response_shape_validated: false,
            account_type_present: false,
            balance_entry_count: None,
            permission_entry_count: None,
            can_trade_present: false,
            can_withdraw_present: false,
            can_deposit_present: false,
            account_snapshot: None,
            error_code: error_code.to_string(),
        }
    }
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
    market_reference_source: String,
    market_reference_price: String,
    max_reference_price_distance_bps: String,
    price_distance_from_reference_bps: String,
    would_cross_spread: bool,
    non_marketable_price_preflight_ready: bool,
    owner_acknowledged_no_cancel_path: bool,
    price_safety_send_consideration_allowed: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
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
    response_redaction_source: String,
    source_guarded_send_run_id: String,
    source_guarded_send_hash: String,
    redacted_response_derived_from_actual_http_result: bool,
    synthetic_fixture_redaction_only: bool,
    owner_run_mutation_closure_evidence: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationLocalOrderLedgerSourceRef {
    path: String,
    hash: String,
    sha256: String,
    bytes: u64,
    source_command: String,
    source_commit: String,
    source_release_tag: String,
    schema_version: String,
    artifact_type: String,
    status: String,
    ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationLocalOrderLedgerArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion_from_v16: String,
    lineage_scope: String,
    current_local_state: String,
    default_fail_closed: bool,
    owner_gated_readback_required: bool,
    local_ledger_ready: bool,
    restart_readable: bool,
    request_builder_ref: ProductionMutationLocalOrderLedgerSourceRef,
    guarded_send_ref: ProductionMutationLocalOrderLedgerSourceRef,
    response_redaction_ref: ProductionMutationLocalOrderLedgerSourceRef,
    readback_ref: ProductionMutationLocalOrderLedgerSourceRef,
    audit_ref: ProductionMutationLocalOrderLedgerSourceRef,
    failure_ref: ProductionMutationLocalOrderLedgerSourceRef,
    symbol: String,
    side: String,
    order_type: String,
    time_in_force: String,
    order_id: String,
    client_order_id: String,
    exchange_status: String,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    exchange_readback_mapped: bool,
    reconciliation_classified: bool,
    orphan_risk_detected: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    network_attempted: bool,
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
    duplicate_submit_attempted: bool,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    no_network_confirmed: bool,
    no_duplicate_submit_confirmed: bool,
    no_retry_confirmed: bool,
    no_cancel_confirmed: bool,
    no_remediation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationExchangeReadbackMapperArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion_from_v16: String,
    lineage_scope: String,
    default_fail_closed: bool,
    owner_gated_readback_required: bool,
    local_ledger_ready: bool,
    exchange_readback_mapped: bool,
    reconciliation_classified: bool,
    orphan_risk_detected: bool,
    local_ledger_ref: ProductionMutationLocalOrderLedgerSourceRef,
    order_readback_ref: ProductionMutationLocalOrderLedgerSourceRef,
    open_orders_readback_ref: ProductionMutationLocalOrderLedgerSourceRef,
    known_order_id: String,
    known_client_order_id: String,
    symbol: String,
    exchange_order_status: String,
    exchange_order_state: String,
    open_order_observed: bool,
    terminal_state_observed: bool,
    order_found: bool,
    open_orders_count: usize,
    source_artifact_issues: Vec<String>,
    malformed_readback_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    manual_review_required: bool,
    new_orders_blocked: bool,
    network_attempted: bool,
    request_sent: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    duplicate_submit_attempted: bool,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    redacted_readback_metadata_only_confirmed: bool,
    known_order_identifier_only_confirmed: bool,
    read_only_reconciliation_scope_confirmed: bool,
    no_network_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    no_production_order_mutation_confirmed: bool,
    no_retry_confirmed: bool,
    no_cancel_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationReconciliationClassifierArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion_from_v16: String,
    lineage_scope: String,
    default_fail_closed: bool,
    owner_gated_readback_required: bool,
    exchange_readback_mapper_ref: ProductionMutationLocalOrderLedgerSourceRef,
    exchange_readback_mapped: bool,
    reconciliation_classified: bool,
    orphan_risk_detected: bool,
    local_request_sent: bool,
    exchange_order_status: String,
    exchange_order_state: String,
    open_order_observed: bool,
    terminal_state_observed: bool,
    order_found: bool,
    reconciliation_outcome: String,
    failure_mode: String,
    failure_state: String,
    terminal_action: String,
    failure_incident_outcome: String,
    failure_incident_severity: String,
    readback_required: bool,
    terminal_evidence_required: bool,
    incident_risk_halted: bool,
    incident_manual_review_required: bool,
    incident_new_orders_blocked: bool,
    failure_semantics_path: String,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    manual_review_required: bool,
    new_orders_blocked: bool,
    network_attempted: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    duplicate_submit_attempted: bool,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    single_v16_mutation_candidate_lineage_confirmed: bool,
    read_only_reconciliation_scope_confirmed: bool,
    no_retry_confirmed: bool,
    no_cancel_confirmed: bool,
    no_remediation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationOrphanOrderDetectorArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion_from_v16: String,
    lineage_scope: String,
    default_fail_closed: bool,
    owner_gated_readback_required: bool,
    reconciliation_classifier_ref: ProductionMutationLocalOrderLedgerSourceRef,
    reconciliation_classified: bool,
    orphan_detection_completed: bool,
    orphan_risk_detected: bool,
    orphan_detection_outcome: String,
    reconciliation_outcome: String,
    local_request_sent: bool,
    local_terminal_state: bool,
    exchange_order_status: String,
    exchange_order_state: String,
    open_order_observed: bool,
    terminal_state_observed: bool,
    order_found: bool,
    failure_mode: String,
    failure_incident_outcome: String,
    readback_required: bool,
    incident_risk_halted: bool,
    risk_halted: bool,
    new_orders_blocked: bool,
    manual_review_required: bool,
    stale_ledger_restart_required: bool,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    network_attempted: bool,
    production_order_submission_allowed: bool,
    production_order_mutation_allowed: bool,
    production_order_state_reads_allowed: bool,
    listen_key_lifecycle_allowed: bool,
    duplicate_submit_attempted: bool,
    retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    single_v16_mutation_candidate_lineage_confirmed: bool,
    read_only_reconciliation_scope_confirmed: bool,
    no_retry_confirmed: bool,
    no_cancel_confirmed: bool,
    no_remediation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationCancelRequestPreviewArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion: String,
    lineage_scope: String,
    default_fail_closed: bool,
    owner_approval_required: bool,
    owner_approval_lifecycle_recorded: bool,
    cancel_candidate_source: String,
    orphan_order_detector_ref: ProductionMutationLocalOrderLedgerSourceRef,
    reconciliation_classifier_ref: ProductionMutationLocalOrderLedgerSourceRef,
    exchange_readback_mapper_ref: ProductionMutationLocalOrderLedgerSourceRef,
    orphan_risk_detected: bool,
    risk_halted: bool,
    new_orders_blocked: bool,
    manual_review_required: bool,
    cancel_request_preview_ready: bool,
    order_identifier_known: bool,
    known_order_id: String,
    known_client_order_id: String,
    symbol: String,
    account_label: String,
    cancel_reason: String,
    orphan_detection_outcome: String,
    candidate_count: u64,
    multi_order_cancel_requested: bool,
    cancel_all_requested: bool,
    bulk_cancel_requested: bool,
    strategy_driven_cancel_requested: bool,
    multi_account_cancel_requested: bool,
    multi_venue_cancel_requested: bool,
    retry_requested: bool,
    replace_or_amend_requested: bool,
    flatten_requested: bool,
    dashboard_cancel_requested: bool,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    network_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    production_order_mutation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    single_v16_mutation_candidate_lineage_confirmed: bool,
    orphan_risk_halted_confirmed: bool,
    manual_review_required_confirmed: bool,
    known_order_identifier_only_confirmed: bool,
    no_retry_confirmed: bool,
    no_cancel_confirmed: bool,
    no_network_confirmed: bool,
    no_remediation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationCancelRiskGateArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion: String,
    lineage_scope: String,
    default_fail_closed: bool,
    cancel_request_preview_ref: ProductionMutationLocalOrderLedgerSourceRef,
    cancel_request_preview_ready: bool,
    cancel_risk_gate_ready: bool,
    orphan_risk_detected: bool,
    risk_halted: bool,
    new_orders_blocked: bool,
    manual_review_required: bool,
    order_identifier_known: bool,
    known_order_id: String,
    known_client_order_id: String,
    symbol: String,
    expected_symbol: String,
    symbol_matches_lineage: bool,
    account_label: String,
    expected_account_label: String,
    account_matches_lineage: bool,
    owner_approval_required: bool,
    owner_approval_lifecycle_recorded: bool,
    candidate_count: u64,
    multi_order_cancel_requested: bool,
    cancel_all_requested: bool,
    bulk_cancel_requested: bool,
    strategy_driven_cancel_requested: bool,
    multi_account_cancel_requested: bool,
    multi_venue_cancel_requested: bool,
    retry_requested: bool,
    replace_or_amend_requested: bool,
    flatten_requested: bool,
    dashboard_cancel_requested: bool,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    network_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    production_order_mutation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    single_v16_mutation_candidate_lineage_confirmed: bool,
    cancel_request_preview_ready_confirmed: bool,
    orphan_risk_halted_confirmed: bool,
    known_order_identifier_only_confirmed: bool,
    symbol_account_scope_confirmed: bool,
    owner_approval_required_confirmed: bool,
    no_cancel_all_or_bulk_confirmed: bool,
    no_retry_confirmed: bool,
    no_cancel_confirmed: bool,
    no_network_confirmed: bool,
    no_remediation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationManualOwnerApprovalLifecycleArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion: String,
    lineage_scope: String,
    default_fail_closed: bool,
    cancel_risk_gate_ref: ProductionMutationLocalOrderLedgerSourceRef,
    cancel_risk_gate_ready: bool,
    approval_scope: String,
    approval_source: String,
    approval_state: String,
    manual_approval_recorded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    now_unix_ms: u64,
    expires_at_unix_ms: u64,
    approval_expires: bool,
    approval_expired: bool,
    approval_revoked: bool,
    approval_used: bool,
    approval_reusable: bool,
    one_time_approval: bool,
    approval_lifecycle_valid: bool,
    owner_approval_required: bool,
    owner_approval_lifecycle_recorded: bool,
    approval_consumed: bool,
    approval_consumed_before_send: bool,
    approval_consumed_after_send: bool,
    known_order_id: String,
    known_client_order_id: String,
    symbol: String,
    account_label: String,
    candidate_count: u64,
    strategy_auto_approval_allowed: bool,
    strategy_auto_approval_attempted: bool,
    background_auto_approval_allowed: bool,
    background_auto_approval_attempted: bool,
    dashboard_auto_approval_allowed: bool,
    dashboard_auto_approval_attempted: bool,
    incident_handler_auto_approval_allowed: bool,
    incident_handler_auto_approval_attempted: bool,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    network_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    production_order_mutation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    lifecycle_issues: Vec<String>,
    one_order_cancel_candidate_confirmed: bool,
    one_time_approval_confirmed: bool,
    non_reusable_approval_confirmed: bool,
    approval_expiry_confirmed: bool,
    no_strategy_auto_approval_confirmed: bool,
    no_background_auto_approval_confirmed: bool,
    no_dashboard_cancel_approval_confirmed: bool,
    no_incident_handler_auto_approval_confirmed: bool,
    no_cancel_confirmed: bool,
    no_network_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationSourceFileRef {
    path: String,
    hash: String,
    sha256: String,
    bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationActualCancelOwnerApprovalLifecycleArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    execution_mode: String,
    approval_scope: String,
    default_fail_closed: bool,
    actual_cancel_safety_contract_ref: ProductionMutationSourceFileRef,
    release_manifest_ref: ProductionMutationSourceFileRef,
    cancel_risk_gate_ref: ProductionMutationLocalOrderLedgerSourceRef,
    safety_contract_ready: bool,
    release_provenance_ready: bool,
    cancel_risk_gate_ready: bool,
    approval_state: String,
    approval_lifecycle_valid: bool,
    approval_execution_authorized: bool,
    approval_failure_reason: String,
    manual_approval_recorded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approval_reason: Option<String>,
    now_unix_ms: u64,
    expires_at_unix_ms: u64,
    approval_created: bool,
    approval_approved: bool,
    approval_expired: bool,
    approval_used: bool,
    approval_rejected: bool,
    approval_audited: bool,
    approval_reusable: bool,
    one_time_approval: bool,
    single_order_required: bool,
    single_venue_required: bool,
    single_execution_attempt_required: bool,
    approval_consumed: bool,
    approval_consumed_before_send: bool,
    approval_consumed_after_send: bool,
    audit_evidence_recorded: bool,
    audit_event: String,
    known_order_id: String,
    known_client_order_id: String,
    symbol: String,
    account_label: String,
    venue: String,
    expected_release_tag: String,
    release_manifest_product_version: String,
    release_manifest_planned_tag: String,
    release_manifest_actual_cancel_scope: String,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    network_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    bulk_cancel_allowed: bool,
    multi_account_cancel_allowed: bool,
    multi_strategy_cancel_allowed: bool,
    multi_venue_cancel_allowed: bool,
    production_order_submit_lifecycle_included: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    dashboard_auto_approval_allowed: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    safety_contract_issues: Vec<String>,
    release_manifest_issues: Vec<String>,
    source_artifact_issues: Vec<String>,
    lifecycle_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    actual_cancel_safety_contract_confirmed: bool,
    one_order_one_venue_one_attempt_confirmed: bool,
    single_use_approval_confirmed: bool,
    approval_expiry_confirmed: bool,
    bind_order_risk_gate_release_provenance_confirmed: bool,
    audit_evidence_confirmed: bool,
    no_dashboard_approval_confirmed: bool,
    no_automatic_cancel_confirmed: bool,
    no_bulk_cancel_confirmed: bool,
    no_retry_confirmed: bool,
    no_submit_lifecycle_confirmed: bool,
    no_network_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationActualCancelExecutorAdapterBoundaryArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    execution_mode: String,
    adapter_boundary_scope: String,
    default_fail_closed: bool,
    owner_approval_lifecycle_ref: ProductionMutationLocalOrderLedgerSourceRef,
    adapter_capability_ref: ProductionMutationLocalOrderLedgerSourceRef,
    owner_approval_lifecycle_ready: bool,
    adapter_capability_ready: bool,
    adapter_boundary_ready: bool,
    actual_cancel_send_allowed_by_adapter_boundary: bool,
    adapter_id: String,
    venue: String,
    order_id_type: String,
    known_order_id: String,
    known_client_order_id: String,
    symbol: String,
    account_label: String,
    cancel_request_contract: String,
    cancel_response_contract: String,
    post_cancel_readback_contract: String,
    audit_contract: String,
    adapter_failure_taxonomy: Vec<String>,
    max_cancel_requests: u64,
    allowed_attempts: u64,
    allowed_order_count: u64,
    allowed_venue_count: u64,
    request_contract_ready: bool,
    response_contract_ready: bool,
    readback_contract_ready: bool,
    audit_contract_ready: bool,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    network_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    bulk_cancel_allowed: bool,
    cancel_all_allowed: bool,
    multi_account_cancel_allowed: bool,
    multi_strategy_cancel_allowed: bool,
    multi_venue_cancel_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    dashboard_execution_allowed: bool,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    source_artifact_issues: Vec<String>,
    adapter_capability_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    adapter_capability_confirmed: bool,
    request_response_readback_audit_contract_confirmed: bool,
    one_order_one_venue_one_attempt_confirmed: bool,
    fail_closed_unsupported_capability_confirmed: bool,
    no_bulk_cancel_confirmed: bool,
    no_retry_confirmed: bool,
    no_automatic_cancel_confirmed: bool,
    no_dashboard_execution_confirmed: bool,
    no_network_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationActualCancelSingleShotArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    execution_mode: String,
    default_fail_closed: bool,
    actual_cancel_safety_contract_ref: ProductionMutationSourceFileRef,
    release_manifest_ref: ProductionMutationSourceFileRef,
    cancel_risk_gate_ref: ProductionMutationLocalOrderLedgerSourceRef,
    owner_approval_lifecycle_ref: ProductionMutationLocalOrderLedgerSourceRef,
    adapter_boundary_ref: ProductionMutationLocalOrderLedgerSourceRef,
    adapter_capability_ref: ProductionMutationLocalOrderLedgerSourceRef,
    manual_online_requested: bool,
    actual_cancel_command_ready: bool,
    single_shot_cancel_allowed: bool,
    owner_approval_ready: bool,
    risk_gate_ready: bool,
    release_provenance_ready: bool,
    adapter_boundary_ready: bool,
    adapter_capability_ready: bool,
    approval_consumed_before_send: bool,
    approval_consumed_after_send: bool,
    approval_state_before_attempt: String,
    approval_state_after_attempt: String,
    request_id: String,
    request_method: String,
    request_target: String,
    request_contract: String,
    adapter_id: String,
    venue: String,
    order_id_type: String,
    known_order_id: String,
    known_client_order_id: String,
    cancel_order_identifier_ref: String,
    symbol: String,
    account_label: String,
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
    response_redacted: bool,
    venue_response_status: String,
    venue_response_source: String,
    venue_response_code: Option<u16>,
    venue_response_error_code: String,
    latency_ms: Option<u64>,
    local_audit_reference: String,
    readback_required: bool,
    readback_requirement: String,
    source_artifact_issues: Vec<String>,
    adapter_capability_issues: Vec<String>,
    safety_contract_issues: Vec<String>,
    release_manifest_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    missing_env_vars: Vec<String>,
    request_sent: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    production_order_mutations_attempted: u64,
    network_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    http_send_attempted: bool,
    venue_ack_observed: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    bulk_cancel_allowed: bool,
    cancel_all_allowed: bool,
    multi_account_cancel_allowed: bool,
    multi_strategy_cancel_allowed: bool,
    multi_venue_cancel_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    dashboard_execution_allowed: bool,
    owner_approval_confirmed: bool,
    risk_gate_confirmed: bool,
    release_provenance_confirmed: bool,
    adapter_boundary_confirmed: bool,
    single_shot_confirmed: bool,
    consume_approval_before_send_confirmed: bool,
    readback_required_confirmed: bool,
    no_bulk_cancel_confirmed: bool,
    no_retry_confirmed: bool,
    no_automatic_cancel_confirmed: bool,
    no_dashboard_execution_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationActualCancelReadbackReconciliationArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    source_actual_cancel_attempt_path: String,
    source_readback_path: String,
    source_actual_cancel_attempt_run_id: String,
    source_actual_cancel_attempt_hash: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    execution_mode: String,
    lineage_scope: String,
    default_fail_closed: bool,
    actual_cancel_attempt_ref: ProductionMutationLocalOrderLedgerSourceRef,
    actual_cancel_attempt_ready: bool,
    actual_cancel_attempt_recorded: bool,
    actual_cancel_request_sent: bool,
    actual_cancel_request_id: String,
    readback_required: bool,
    readback_evidence_present: bool,
    reconciliation_evidence_present: bool,
    reconciliation_ready: bool,
    readback_reconciliation_complete: bool,
    actual_cancel_followup_complete: bool,
    redacted_metadata_only: bool,
    venue: String,
    symbol: String,
    account_label: String,
    known_order_id: String,
    known_client_order_id: String,
    readback_type: String,
    readback_state: String,
    readback_result: String,
    reconciliation_status: String,
    venue_state: String,
    order_status: String,
    execution_fill_status: String,
    remaining_quantity_state: String,
    residual_risk_state: String,
    local_audit_state: String,
    readback_order_id: String,
    readback_client_order_id: String,
    readback_orig_client_order_id: String,
    readback_update_time_shape: String,
    partial_fill_observed: bool,
    already_cancelled_observed: bool,
    filled_before_cancel_observed: bool,
    timeout_observed: bool,
    unknown_observed: bool,
    inconsistent_observed: bool,
    degraded: bool,
    error_state: bool,
    terminal_state_observed: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    risk_halted: bool,
    dashboard_read_only_consumable: bool,
    dashboard_audit_view_ready: bool,
    allowed_readback_fields: Vec<String>,
    forbidden_readback_markers: Vec<String>,
    source_artifact_issues: Vec<String>,
    readback_lineage_issues: Vec<String>,
    unsupported_readback_states: Vec<String>,
    missing_cli_flags: Vec<String>,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    raw_readback_body_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    unrestricted_payload_recorded: bool,
    account_balances_recorded: bool,
    fills_recorded: bool,
    readback_execution_attempted: bool,
    order_state_read_attempted: bool,
    production_order_state_reads_attempted: u64,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    production_order_mutations_attempted: u64,
    network_attempted: bool,
    network_readback_endpoint_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    second_cancel_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    production_order_mutation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    actual_cancel_attempt_recorded_confirmed: bool,
    readback_required_confirmed: bool,
    readback_metadata_only_confirmed: bool,
    order_status_reconciled_confirmed: bool,
    execution_fill_status_reconciled_confirmed: bool,
    remaining_quantity_reconciled_confirmed: bool,
    risk_state_recorded_confirmed: bool,
    local_audit_state_recorded_confirmed: bool,
    dashboard_read_only_consumable_confirmed: bool,
    no_raw_readback_persistence_confirmed: bool,
    no_headers_persistence_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    no_retry_confirmed: bool,
    no_remediation_confirmed: bool,
    no_second_cancel_confirmed: bool,
    no_network_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationActualCancelFailureEvidenceArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    source_readback_reconciliation_path: String,
    source_request_ref_path: String,
    source_response_ref_path: String,
    source_readback_ref_path: String,
    source_audit_ref_path: String,
    source_readback_reconciliation_run_id: String,
    source_readback_reconciliation_hash: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    execution_mode: String,
    lineage_scope: String,
    default_fail_closed: bool,
    readback_reconciliation_ref: ProductionMutationLocalOrderLedgerSourceRef,
    request_ref: ProductionMutationLocalOrderLedgerSourceRef,
    response_ref: ProductionMutationLocalOrderLedgerSourceRef,
    readback_ref: ProductionMutationLocalOrderLedgerSourceRef,
    audit_ref: ProductionMutationLocalOrderLedgerSourceRef,
    request_ref_recorded: bool,
    response_ref_recorded: bool,
    readback_ref_recorded: bool,
    audit_ref_recorded: bool,
    references_ready: bool,
    evidence_ready: bool,
    failure_evidence_ready: bool,
    dashboard_read_only_consumable: bool,
    release_gate_consumable: bool,
    venue: String,
    symbol: String,
    account_label: String,
    readback_result: String,
    reconciliation_status: String,
    source_readback_state: String,
    source_venue_state: String,
    source_order_status: String,
    source_execution_fill_status: String,
    source_remaining_quantity_state: String,
    source_residual_risk_state: String,
    source_local_audit_state: String,
    cancel_outcome: String,
    outcome_category: String,
    failure_mode: String,
    partial_success_mode: String,
    operator_action: String,
    operator_action_required: bool,
    recovered: bool,
    degraded: bool,
    failed: bool,
    partial_success: bool,
    residual_risk_visible: bool,
    residual_risk_state: String,
    manual_review_required: bool,
    new_orders_blocked: bool,
    risk_halted: bool,
    outcome_cancel_confirmed: bool,
    outcome_already_cancelled: bool,
    outcome_rejected: bool,
    outcome_timeout: bool,
    outcome_unknown: bool,
    outcome_partial_fill: bool,
    outcome_filled_before_cancel: bool,
    outcome_venue_unavailable: bool,
    outcome_adapter_failure: bool,
    outcome_inconsistent: bool,
    outcome_failed: bool,
    actual_cancel_followup_complete: bool,
    unknown_not_recovered: bool,
    partial_fill_residual_risk_visible: bool,
    request_response_readback_audit_refs_recorded: bool,
    source_artifact_issues: Vec<String>,
    lineage_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    raw_readback_body_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    unrestricted_payload_recorded: bool,
    account_balances_recorded: bool,
    fills_recorded: bool,
    readback_execution_attempted: bool,
    order_state_read_attempted: bool,
    production_order_state_reads_attempted: u64,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    production_order_mutations_attempted: u64,
    network_attempted: bool,
    network_readback_endpoint_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    compensation_trade_attempted: bool,
    second_cancel_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    production_order_mutation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    request_ref_recorded_confirmed: bool,
    response_ref_recorded_confirmed: bool,
    readback_ref_recorded_confirmed: bool,
    audit_ref_recorded_confirmed: bool,
    failure_outcomes_classified_confirmed: bool,
    operator_action_model_confirmed: bool,
    unknown_not_recovered_confirmed: bool,
    partial_fill_residual_risk_confirmed: bool,
    dashboard_release_gate_consumable_confirmed: bool,
    no_retry_confirmed: bool,
    no_remediation_confirmed: bool,
    no_compensation_trade_confirmed: bool,
    no_network_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationCancelResponseRedactionArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    source_manual_owner_approval_lifecycle_path: String,
    source_response_path: String,
    source_manual_owner_approval_lifecycle_run_id: String,
    source_manual_owner_approval_lifecycle_hash: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion: String,
    lineage_scope: String,
    default_fail_closed: bool,
    manual_owner_approval_lifecycle_ref: ProductionMutationLocalOrderLedgerSourceRef,
    approval_lifecycle_valid: bool,
    approval_state: String,
    manual_approval_recorded: bool,
    approval_consumed: bool,
    response_redaction_ready: bool,
    cancel_response_redacted: bool,
    response_shape_validated: bool,
    response_type: String,
    known_order_id: String,
    known_client_order_id: String,
    cancel_order_id: String,
    cancel_client_order_id: String,
    orig_client_order_id: String,
    symbol: String,
    account_label: String,
    exchange_status: String,
    transact_time_shape: String,
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
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    network_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    production_order_mutation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    manual_owner_approval_lifecycle_ready_confirmed: bool,
    no_raw_response_persistence_confirmed: bool,
    no_headers_persistence_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    cancel_metadata_only_confirmed: bool,
    no_account_balances_confirmed: bool,
    no_unrestricted_payload_confirmed: bool,
    no_retry_confirmed: bool,
    no_cancel_confirmed: bool,
    no_network_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationPostCancelReadbackArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    source_cancel_response_redaction_path: String,
    source_readback_path: String,
    source_cancel_response_redaction_run_id: String,
    source_cancel_response_redaction_hash: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion: String,
    lineage_scope: String,
    default_fail_closed: bool,
    cancel_response_redaction_ref: ProductionMutationLocalOrderLedgerSourceRef,
    cancel_response_redaction_ready: bool,
    cancel_response_redacted: bool,
    post_cancel_readback_ready: bool,
    post_cancel_readback_classified: bool,
    redacted_metadata_only: bool,
    readback_type: String,
    readback_state: String,
    readback_state_class: String,
    readback_outcome: String,
    terminal_state_observed: bool,
    ambiguous_state_observed: bool,
    order_found: bool,
    order_lineage_preserved: bool,
    known_order_id: String,
    known_client_order_id: String,
    readback_order_id: String,
    readback_client_order_id: String,
    readback_orig_client_order_id: String,
    symbol: String,
    account_label: String,
    readback_update_time_shape: String,
    allowed_readback_fields: Vec<String>,
    forbidden_readback_markers: Vec<String>,
    source_artifact_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    unsupported_readback_states: Vec<String>,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    raw_readback_body_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    unrestricted_payload_recorded: bool,
    account_balances_recorded: bool,
    fills_recorded: bool,
    readback_execution_attempted: bool,
    order_state_read_attempted: bool,
    production_order_state_reads_attempted: u64,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    production_order_mutations_attempted: u64,
    network_attempted: bool,
    network_readback_endpoint_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    production_order_mutation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    cancel_response_redaction_ready_confirmed: bool,
    readback_metadata_only_confirmed: bool,
    terminal_and_ambiguous_classification_confirmed: bool,
    no_raw_readback_persistence_confirmed: bool,
    no_headers_persistence_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    no_mutation_confirmed: bool,
    no_retry_confirmed: bool,
    no_remediation_confirmed: bool,
    no_cancel_confirmed: bool,
    no_network_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProductionMutationCancelRecoveryIncidentAuditCloseoutArtifact {
    schema_version: String,
    run_id: String,
    order_lineage_id: String,
    source_cancel_risk_gate_path: String,
    source_manual_owner_approval_lifecycle_path: String,
    source_cancel_response_redaction_path: String,
    source_post_cancel_readback_path: String,
    source_cancel_risk_gate_run_id: String,
    source_manual_owner_approval_lifecycle_run_id: String,
    source_cancel_response_redaction_run_id: String,
    source_post_cancel_readback_run_id: String,
    source_cancel_risk_gate_hash: String,
    source_manual_owner_approval_lifecycle_hash: String,
    source_cancel_response_redaction_hash: String,
    source_post_cancel_readback_hash: String,
    artifact_type: String,
    status: String,
    created_at: String,
    mode: String,
    capability: String,
    capability_expansion: String,
    lineage_scope: String,
    default_fail_closed: bool,
    cancel_risk_gate_ref: ProductionMutationLocalOrderLedgerSourceRef,
    manual_owner_approval_lifecycle_ref: ProductionMutationLocalOrderLedgerSourceRef,
    cancel_response_redaction_ref: ProductionMutationLocalOrderLedgerSourceRef,
    post_cancel_readback_ref: ProductionMutationLocalOrderLedgerSourceRef,
    cancel_recovery_lineage_ready: bool,
    incident_closeout_ready: bool,
    audit_trail_ready: bool,
    audit_traceability_ready: bool,
    recovery_needed_reason: String,
    risk_gate_result: String,
    risk_gate_ready: bool,
    orphan_risk_detected: bool,
    risk_halted: bool,
    new_orders_blocked: bool,
    manual_review_required: bool,
    owner_approval_state: String,
    manual_approval_recorded: bool,
    approval_lifecycle_valid: bool,
    approval_consumed: bool,
    redaction_contract_state: String,
    cancel_response_redaction_ready: bool,
    cancel_response_redacted: bool,
    post_cancel_readback_ready: bool,
    readback_state: String,
    readback_state_class: String,
    readback_outcome: String,
    terminal_state_observed: bool,
    ambiguous_state_observed: bool,
    terminal_action_recommendation: String,
    remaining_risk: String,
    remaining_risk_requires_manual_review: bool,
    order_lineage_preserved: bool,
    candidate_count: u64,
    known_order_id: String,
    known_client_order_id: String,
    symbol: String,
    account_label: String,
    source_artifact_issues: Vec<String>,
    lineage_issues: Vec<String>,
    missing_cli_flags: Vec<String>,
    api_key_value_recorded: bool,
    api_secret_value_recorded: bool,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    raw_exchange_response_recorded: bool,
    raw_readback_body_recorded: bool,
    response_body_recorded: bool,
    response_headers_recorded: bool,
    unrestricted_payload_recorded: bool,
    account_balances_recorded: bool,
    fills_recorded: bool,
    readback_execution_attempted: bool,
    order_state_read_attempted: bool,
    production_order_state_reads_attempted: u64,
    actual_cancel_send_allowed: bool,
    cancel_attempted: bool,
    cancel_requests_sent: u64,
    production_order_mutations_attempted: u64,
    network_attempted: bool,
    network_readback_endpoint_attempted: bool,
    network_cancel_endpoint_attempted: bool,
    retry_attempted: bool,
    replace_attempted: bool,
    amend_attempted: bool,
    flatten_attempted: bool,
    remediation_attempted: bool,
    automatic_cancel_allowed: bool,
    automatic_remediation_allowed: bool,
    production_order_mutation_allowed: bool,
    dashboard_order_controls_enabled: bool,
    dashboard_cancel_controls_enabled: bool,
    cancel_recovery_lineage_confirmed: bool,
    risk_reason_recorded_confirmed: bool,
    risk_gate_result_recorded_confirmed: bool,
    owner_approval_state_recorded_confirmed: bool,
    redaction_contract_state_recorded_confirmed: bool,
    readback_state_recorded_confirmed: bool,
    terminal_action_recommendation_confirmed: bool,
    remaining_risk_recorded_confirmed: bool,
    no_mutation_confirmed: bool,
    no_cancel_confirmed: bool,
    no_network_confirmed: bool,
    no_retry_confirmed: bool,
    no_remediation_confirmed: bool,
    no_automatic_remediation_confirmed: bool,
    dashboard_controls_disabled_confirmed: bool,
    no_secret_persistence_confirmed: bool,
    diagnostic: String,
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
struct ProductionMutationPriceSafetyPreflight {
    market_reference_source: String,
    market_reference_price: String,
    max_reference_price_distance_bps: String,
    price_distance_from_reference_bps: String,
    preflight_ready: bool,
    source_artifact_issues: Vec<String>,
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

fn retired_production_mutation_guarded_send_counters() -> ProductionMutationGuardedSendCounters {
    ProductionMutationGuardedSendCounters {
        request_sent: false,
        network_attempted: false,
        production_order_request_attempted: false,
        http_send_attempted: false,
        exchange_ack_observed: false,
        exchange_order_id_observed: false,
        exchange_order_status_observed: false,
        confirmed_production_order_submission: false,
        production_order_submissions_attempted: 0,
        production_orders_submitted: 0,
        production_order_mutations_attempted: 0,
        real_orders_submitted: false,
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
    let artifact = build_production_mutation_guarded_send_artifact(opt)?;
    write_production_mutation_guarded_send_artifact(&opt.output, &artifact)?;
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

fn run_live_production_mutation_local_order_ledger(
    opt: &LiveProductionMutationLocalOrderLedgerOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_local_order_ledger_artifact(opt)?;
    write_production_mutation_local_order_ledger_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_local_order_ledger status={} run_id={} order_lineage_id={} output={} local_ledger_ready={} current_local_state={} duplicate_submit_attempted=false retry_attempted=false cancel_attempted=false remediation_attempted=false dashboard_order_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.local_ledger_ready,
        artifact.current_local_state,
    );
    Ok(())
}

fn run_live_production_mutation_exchange_readback_mapper(
    opt: &LiveProductionMutationExchangeReadbackMapperOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_exchange_readback_mapper_artifact(opt)?;
    write_production_mutation_exchange_readback_mapper_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_exchange_readback_mapper status={} run_id={} order_lineage_id={} output={} exchange_readback_mapped={} exchange_order_state={} exchange_order_status={} open_order_observed={} terminal_state_observed={} retry_attempted=false cancel_attempted=false remediation_attempted=false dashboard_order_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.exchange_readback_mapped,
        artifact.exchange_order_state,
        artifact.exchange_order_status,
        artifact.open_order_observed,
        artifact.terminal_state_observed,
    );
    Ok(())
}

fn run_live_production_mutation_reconciliation_classifier(
    opt: &LiveProductionMutationReconciliationClassifierOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_reconciliation_classifier_artifact(opt)?;
    write_production_mutation_reconciliation_classifier_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_reconciliation_classifier status={} run_id={} order_lineage_id={} output={} reconciliation_classified={} reconciliation_outcome={} manual_review_required={} new_orders_blocked={} retry_attempted=false cancel_attempted=false remediation_attempted=false dashboard_order_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.reconciliation_classified,
        artifact.reconciliation_outcome,
        artifact.manual_review_required,
        artifact.new_orders_blocked,
    );
    Ok(())
}

fn run_live_production_mutation_orphan_order_detector(
    opt: &LiveProductionMutationOrphanOrderDetectorOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_orphan_order_detector_artifact(opt)?;
    write_production_mutation_orphan_order_detector_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_orphan_order_detector status={} run_id={} order_lineage_id={} output={} orphan_detection_completed={} orphan_risk_detected={} orphan_detection_outcome={} risk_halted={} manual_review_required={} new_orders_blocked={} retry_attempted=false cancel_attempted=false remediation_attempted=false dashboard_order_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.orphan_detection_completed,
        artifact.orphan_risk_detected,
        artifact.orphan_detection_outcome,
        artifact.risk_halted,
        artifact.manual_review_required,
        artifact.new_orders_blocked,
    );
    Ok(())
}

fn run_live_production_mutation_cancel_request_preview(
    opt: &LiveProductionMutationCancelRequestPreviewOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_cancel_request_preview_artifact(opt)?;
    write_production_mutation_cancel_request_preview_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_cancel_request_preview status={} run_id={} order_lineage_id={} output={} cancel_request_preview_ready={} orphan_risk_detected={} risk_halted={} manual_review_required={} new_orders_blocked={} order_identifier_known={} candidate_count={} actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 network_attempted=false network_cancel_endpoint_attempted=false retry_attempted=false remediation_attempted=false dashboard_cancel_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.cancel_request_preview_ready,
        artifact.orphan_risk_detected,
        artifact.risk_halted,
        artifact.manual_review_required,
        artifact.new_orders_blocked,
        artifact.order_identifier_known,
        artifact.candidate_count,
    );
    Ok(())
}

fn run_live_production_mutation_cancel_risk_gate(
    opt: &LiveProductionMutationCancelRiskGateOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_cancel_risk_gate_artifact(opt)?;
    write_production_mutation_cancel_risk_gate_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_cancel_risk_gate status={} run_id={} order_lineage_id={} output={} cancel_risk_gate_ready={} cancel_request_preview_ready={} orphan_risk_detected={} risk_halted={} manual_review_required={} new_orders_blocked={} order_identifier_known={} symbol_matches_lineage={} account_matches_lineage={} owner_approval_required={} actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 network_attempted=false network_cancel_endpoint_attempted=false retry_attempted=false remediation_attempted=false dashboard_cancel_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.cancel_risk_gate_ready,
        artifact.cancel_request_preview_ready,
        artifact.orphan_risk_detected,
        artifact.risk_halted,
        artifact.manual_review_required,
        artifact.new_orders_blocked,
        artifact.order_identifier_known,
        artifact.symbol_matches_lineage,
        artifact.account_matches_lineage,
        artifact.owner_approval_required,
    );
    Ok(())
}

fn run_live_production_mutation_manual_owner_approval_lifecycle(
    opt: &LiveProductionMutationManualOwnerApprovalLifecycleOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_manual_owner_approval_lifecycle_artifact(opt)?;
    write_production_mutation_manual_owner_approval_lifecycle_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_manual_owner_approval_lifecycle status={} run_id={} order_lineage_id={} output={} approval_state={} approval_lifecycle_valid={} approval_scope={} approval_reusable=false approval_expires={} approval_consumed_before_send=false approval_consumed_after_send=false actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 network_attempted=false network_cancel_endpoint_attempted=false dashboard_cancel_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.approval_state,
        artifact.approval_lifecycle_valid,
        artifact.approval_scope,
        artifact.approval_expires,
    );
    Ok(())
}

fn run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
    opt: &LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_actual_cancel_owner_approval_lifecycle_artifact(opt)?;
    write_production_mutation_actual_cancel_owner_approval_lifecycle_artifact(
        &opt.output,
        &artifact,
    )?;
    println!(
        "live.production_mutation_actual_cancel_owner_approval_lifecycle status={} run_id={} order_lineage_id={} venue={} output={} approval_state={} approval_lifecycle_valid={} approval_execution_authorized={} approval_reusable=false one_order_one_venue_one_attempt=true actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 network_attempted=false dashboard_cancel_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        artifact.venue,
        opt.output.display(),
        artifact.approval_state,
        artifact.approval_lifecycle_valid,
        artifact.approval_execution_authorized,
    );
    Ok(())
}

fn run_live_production_mutation_actual_cancel_executor_adapter_boundary(
    opt: &LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_actual_cancel_executor_adapter_boundary_artifact(opt)?;
    write_production_mutation_actual_cancel_executor_adapter_boundary_artifact(
        &opt.output,
        &artifact,
    )?;
    println!(
        "live.production_mutation_actual_cancel_executor_adapter_boundary status={} run_id={} order_lineage_id={} adapter_id={} venue={} order_id_type={} output={} adapter_boundary_ready={} actual_cancel_send_allowed_by_adapter_boundary={} actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 network_attempted=false retry_attempted=false bulk_cancel_allowed=false dashboard_cancel_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        artifact.adapter_id,
        artifact.venue,
        artifact.order_id_type,
        opt.output.display(),
        artifact.adapter_boundary_ready,
        artifact.actual_cancel_send_allowed_by_adapter_boundary,
    );
    Ok(())
}

fn run_live_production_mutation_actual_cancel_single_shot(
    opt: &LiveProductionMutationActualCancelSingleShotOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_actual_cancel_single_shot_artifact(opt)?;
    write_production_mutation_actual_cancel_single_shot_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_actual_cancel_single_shot status={} run_id={} order_lineage_id={} venue={} order_id_type={} output={} manual_online_requested={} actual_cancel_command_ready={} single_shot_cancel_allowed={} request_sent={} cancel_attempted={} cancel_requests_sent={} network_attempted={} readback_required={} approval_state_after_attempt={} retry_attempted=false bulk_cancel_allowed=false dashboard_cancel_controls_enabled=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        artifact.venue,
        artifact.order_id_type,
        opt.output.display(),
        artifact.manual_online_requested,
        artifact.actual_cancel_command_ready,
        artifact.single_shot_cancel_allowed,
        artifact.request_sent,
        artifact.cancel_attempted,
        artifact.cancel_requests_sent,
        artifact.network_attempted,
        artifact.readback_required,
        artifact.approval_state_after_attempt,
    );
    Ok(())
}

fn run_live_production_mutation_actual_cancel_readback_reconciliation(
    opt: &LiveProductionMutationActualCancelReadbackReconciliationOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_actual_cancel_readback_reconciliation_artifact(opt)?;
    write_production_mutation_actual_cancel_readback_reconciliation_artifact(
        &opt.output,
        &artifact,
    )?;
    println!(
        "live.production_mutation_actual_cancel_readback_reconciliation status={} run_id={} order_lineage_id={} output={} reconciliation_ready={} readback_result={} reconciliation_status={} partial_fill_observed={} already_cancelled_observed={} timeout_observed={} unknown_observed={} inconsistent_observed={} manual_review_required={} new_orders_blocked={} risk_halted={} dashboard_read_only_consumable={} raw_exchange_response_recorded=false raw_readback_body_recorded=false response_headers_recorded=false unrestricted_payload_recorded=false actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 production_order_mutations_attempted=0 readback_execution_attempted=false order_state_read_attempted=false production_order_state_reads_attempted=0 network_attempted=false network_readback_endpoint_attempted=false network_cancel_endpoint_attempted=false retry_attempted=false remediation_attempted=false second_cancel_attempted=false dashboard_cancel_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.reconciliation_ready,
        artifact.readback_result,
        artifact.reconciliation_status,
        artifact.partial_fill_observed,
        artifact.already_cancelled_observed,
        artifact.timeout_observed,
        artifact.unknown_observed,
        artifact.inconsistent_observed,
        artifact.manual_review_required,
        artifact.new_orders_blocked,
        artifact.risk_halted,
        artifact.dashboard_read_only_consumable,
    );
    Ok(())
}

fn run_live_production_mutation_actual_cancel_failure_evidence(
    opt: &LiveProductionMutationActualCancelFailureEvidenceOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_actual_cancel_failure_evidence_artifact(opt)?;
    write_production_mutation_actual_cancel_failure_evidence_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_actual_cancel_failure_evidence status={} run_id={} order_lineage_id={} output={} evidence_ready={} cancel_outcome={} outcome_category={} operator_action_required={} recovered={} degraded={} failed={} partial_success={} residual_risk_visible={} dashboard_read_only_consumable={} release_gate_consumable={} request_response_readback_audit_refs_recorded={} unknown_not_recovered={} partial_fill_residual_risk_visible={} raw_exchange_response_recorded=false raw_readback_body_recorded=false response_headers_recorded=false unrestricted_payload_recorded=false actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 production_order_mutations_attempted=0 readback_execution_attempted=false order_state_read_attempted=false production_order_state_reads_attempted=0 network_attempted=false network_readback_endpoint_attempted=false network_cancel_endpoint_attempted=false retry_attempted=false remediation_attempted=false compensation_trade_attempted=false second_cancel_attempted=false dashboard_cancel_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.evidence_ready,
        artifact.cancel_outcome,
        artifact.outcome_category,
        artifact.operator_action_required,
        artifact.recovered,
        artifact.degraded,
        artifact.failed,
        artifact.partial_success,
        artifact.residual_risk_visible,
        artifact.dashboard_read_only_consumable,
        artifact.release_gate_consumable,
        artifact.request_response_readback_audit_refs_recorded,
        artifact.unknown_not_recovered,
        artifact.partial_fill_residual_risk_visible,
    );
    Ok(())
}

fn run_live_production_mutation_cancel_response_redaction(
    opt: &LiveProductionMutationCancelResponseRedactionOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_cancel_response_redaction_artifact(opt)?;
    write_production_mutation_cancel_response_redaction_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_cancel_response_redaction status={} run_id={} order_lineage_id={} output={} response_redaction_ready={} cancel_response_redacted={} raw_exchange_response_recorded=false response_headers_recorded=false unrestricted_payload_recorded=false account_balances_recorded=false actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 network_attempted=false network_cancel_endpoint_attempted=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.response_redaction_ready,
        artifact.cancel_response_redacted,
    );
    Ok(())
}

fn run_live_production_mutation_post_cancel_readback(
    opt: &LiveProductionMutationPostCancelReadbackOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_post_cancel_readback_artifact(opt)?;
    write_production_mutation_post_cancel_readback_artifact(&opt.output, &artifact)?;
    println!(
        "live.production_mutation_post_cancel_readback status={} run_id={} order_lineage_id={} output={} post_cancel_readback_ready={} readback_state={} readback_state_class={} terminal_state_observed={} ambiguous_state_observed={} raw_exchange_response_recorded=false raw_readback_body_recorded=false response_headers_recorded=false unrestricted_payload_recorded=false actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 production_order_mutations_attempted=0 readback_execution_attempted=false order_state_read_attempted=false production_order_state_reads_attempted=0 network_attempted=false network_readback_endpoint_attempted=false network_cancel_endpoint_attempted=false retry_attempted=false remediation_attempted=false dashboard_cancel_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.post_cancel_readback_ready,
        artifact.readback_state,
        artifact.readback_state_class,
        artifact.terminal_state_observed,
        artifact.ambiguous_state_observed,
    );
    Ok(())
}

fn run_live_production_mutation_cancel_recovery_incident_audit_closeout(
    opt: &LiveProductionMutationCancelRecoveryIncidentAuditCloseoutOpt,
) -> anyhow::Result<()> {
    let artifact = build_production_mutation_cancel_recovery_incident_audit_closeout_artifact(opt)?;
    write_production_mutation_cancel_recovery_incident_audit_closeout_artifact(
        &opt.output,
        &artifact,
    )?;
    println!(
        "live.production_mutation_cancel_recovery_incident_audit_closeout status={} run_id={} order_lineage_id={} output={} incident_closeout_ready={} audit_trail_ready={} recovery_needed_reason={} risk_gate_result={} owner_approval_state={} redaction_contract_state={} readback_state={} readback_state_class={} terminal_action_recommendation={} remaining_risk={} actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 production_order_mutations_attempted=0 readback_execution_attempted=false order_state_read_attempted=false production_order_state_reads_attempted=0 network_attempted=false network_readback_endpoint_attempted=false network_cancel_endpoint_attempted=false retry_attempted=false remediation_attempted=false automatic_remediation_allowed=false dashboard_cancel_controls_enabled=false signature_recorded=false signed_query_recorded=false signed_url_recorded=false api_key_value_recorded=false api_secret_value_recorded=false",
        artifact.status,
        artifact.run_id,
        artifact.order_lineage_id,
        opt.output.display(),
        artifact.incident_closeout_ready,
        artifact.audit_trail_ready,
        artifact.recovery_needed_reason,
        artifact.risk_gate_result,
        artifact.owner_approval_state,
        artifact.redaction_contract_state,
        artifact.readback_state,
        artifact.readback_state_class,
        artifact.terminal_action_recommendation,
        artifact.remaining_risk,
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

pub(crate) fn execute_product_live_account_read(
    api_key_env: &str,
    api_secret_env: &str,
    recv_window_ms: u64,
) -> ProductLiveAccountReadObservation {
    execute_product_live_account_read_with_env_and_http(
        api_key_env,
        api_secret_env,
        recv_window_ms,
        |name| std::env::var(name).ok(),
        execute_production_account_snapshot_read,
    )
}

fn execute_product_live_account_read_with_env_and_http<F, H>(
    api_key_env: &str,
    api_secret_env: &str,
    recv_window_ms: u64,
    mut read_env: F,
    mut execute_http: H,
) -> ProductLiveAccountReadObservation
where
    F: FnMut(&str) -> Option<String>,
    H: FnMut(&EnvOnlyProductionReadCredentials, u64) -> ProductionAccountSnapshotHttpResult,
{
    let runtime_gates_open = [
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW,
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED,
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION,
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE,
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE,
    ]
    .into_iter()
    .all(|name| read_env(name).as_deref() == Some("1"));
    if !runtime_gates_open {
        return ProductLiveAccountReadObservation::blocked("runtime_gate_changed");
    }

    let credentials = EnvOnlyProductionReadCredentials::from_values(
        api_key_env.to_string(),
        read_env(api_key_env),
        api_secret_env.to_string(),
        read_env(api_secret_env),
    );
    if !credentials.api_key_present() || !credentials.api_secret_present() {
        return ProductLiveAccountReadObservation::blocked("credential_state_changed");
    }

    let result = execute_http(&credentials, recv_window_ms);
    let product_result_ready = result.response_shape_validated && result.product_snapshot.is_some();
    let error_code = if result.response_shape_validated
        && result.product_snapshot.is_none()
        && result.error_code == "none"
    {
        "account_result_missing".to_string()
    } else {
        result.error_code
    };
    ProductLiveAccountReadObservation {
        status: if product_result_ready {
            "connected".to_string()
        } else {
            "failed".to_string()
        },
        network_attempted: result.network_attempted,
        account_read_attempted: result.network_attempted,
        response_status_code: result.http_status,
        latency_ms: result.latency_ms,
        response_shape: result.response_shape,
        response_shape_validated: result.response_shape_validated,
        account_type_present: result.response_shape_summary.account_type_present,
        balance_entry_count: result.response_shape_summary.balance_entry_count,
        permission_entry_count: result.response_shape_summary.permission_entry_count,
        can_trade_present: result.response_shape_summary.can_trade_present,
        can_withdraw_present: result.response_shape_summary.can_withdraw_present,
        can_deposit_present: result.response_shape_summary.can_deposit_present,
        account_snapshot: result.product_snapshot,
        error_code,
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
        .redirect(reqwest::redirect::Policy::none())
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
                let content_length = response.content_length();
                match read_product_live_account_response_body(response, content_length).and_then(
                    |bytes| {
                        serde_json::from_slice::<serde_json::Value>(&bytes)
                            .map_err(|_| "response_shape_invalid")
                    },
                ) {
                    Ok(body) => {
                        let shape_summary = summarize_production_account_snapshot_shape(&body);
                        if shape_summary.shape_validated {
                            match project_product_live_account_snapshot(&body) {
                                Ok(product_snapshot) => {
                                    ProductionAccountSnapshotHttpResult::success_with_product_snapshot(
                                        latency_ms,
                                        status,
                                        shape_summary,
                                        product_snapshot,
                                    )
                                }
                                Err(error_code) => {
                                    ProductionAccountSnapshotHttpResult::failure_with_shape(
                                        Some(latency_ms),
                                        Some(status),
                                        error_code,
                                        shape_summary,
                                    )
                                }
                            }
                        } else {
                            ProductionAccountSnapshotHttpResult::failure_with_shape(
                                Some(latency_ms),
                                Some(status),
                                "response_shape_invalid",
                                shape_summary,
                            )
                        }
                    }
                    Err(error_code) => ProductionAccountSnapshotHttpResult::failure(
                        Some(latency_ms),
                        Some(status),
                        error_code,
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

const PRODUCT_LIVE_ACCOUNT_MAX_RESPONSE_BYTES: usize = 1_048_576;
const PRODUCT_LIVE_ACCOUNT_MAX_SOURCE_BALANCES: usize = 2_048;
const PRODUCT_LIVE_ACCOUNT_MAX_NON_ZERO_ASSETS: usize = 256;

fn read_product_live_account_response_body<R>(
    reader: R,
    content_length: Option<u64>,
) -> Result<Vec<u8>, &'static str>
where
    R: Read,
{
    if content_length.is_some_and(|length| length > PRODUCT_LIVE_ACCOUNT_MAX_RESPONSE_BYTES as u64)
    {
        return Err("account_result_limit_exceeded");
    }
    let mut bytes = Vec::with_capacity(
        content_length
            .unwrap_or_default()
            .min(PRODUCT_LIVE_ACCOUNT_MAX_RESPONSE_BYTES as u64) as usize,
    );
    reader
        .take(PRODUCT_LIVE_ACCOUNT_MAX_RESPONSE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "body_error")?;
    if bytes.len() > PRODUCT_LIVE_ACCOUNT_MAX_RESPONSE_BYTES {
        return Err("account_result_limit_exceeded");
    }
    Ok(bytes)
}

fn project_product_live_account_snapshot(
    body: &serde_json::Value,
) -> Result<ProductLiveAccountSnapshot, &'static str> {
    let object = body.as_object().ok_or("account_result_invalid")?;
    let account_type = object
        .get("accountType")
        .and_then(serde_json::Value::as_str)
        .filter(|value| product_live_account_token_is_valid(value))
        .ok_or("account_result_invalid")?
        .to_string();
    let can_trade = object
        .get("canTrade")
        .and_then(serde_json::Value::as_bool)
        .ok_or("account_result_invalid")?;
    let can_withdraw = object
        .get("canWithdraw")
        .and_then(serde_json::Value::as_bool)
        .ok_or("account_result_invalid")?;
    let can_deposit = object
        .get("canDeposit")
        .and_then(serde_json::Value::as_bool)
        .ok_or("account_result_invalid")?;
    let balances = object
        .get("balances")
        .and_then(serde_json::Value::as_array)
        .ok_or("account_result_invalid")?;
    if balances.len() > PRODUCT_LIVE_ACCOUNT_MAX_SOURCE_BALANCES {
        return Err("account_result_limit_exceeded");
    }

    let mut seen_assets = BTreeSet::new();
    let mut assets = Vec::new();
    for balance in balances {
        let balance = balance.as_object().ok_or("account_result_invalid")?;
        let asset = balance
            .get("asset")
            .and_then(serde_json::Value::as_str)
            .filter(|value| product_live_asset_code_is_valid(value))
            .ok_or("account_result_invalid")?;
        if !seen_assets.insert(asset.to_string()) {
            return Err("account_result_duplicate_asset");
        }
        let free_text = balance
            .get("free")
            .and_then(serde_json::Value::as_str)
            .filter(|value| value.len() <= 64)
            .ok_or("account_result_invalid")?;
        let locked_text = balance
            .get("locked")
            .and_then(serde_json::Value::as_str)
            .filter(|value| value.len() <= 64)
            .ok_or("account_result_invalid")?;
        let free = parse_product_live_account_decimal(free_text)?;
        let locked = parse_product_live_account_decimal(locked_text)?;
        let total = free.checked_add(locked).ok_or("account_result_invalid")?;
        if total == Decimal::ZERO {
            continue;
        }
        if assets.len() >= PRODUCT_LIVE_ACCOUNT_MAX_NON_ZERO_ASSETS {
            return Err("account_result_limit_exceeded");
        }
        assets.push(ProductLiveAssetBalance {
            asset: asset.to_string(),
            free: format_decimal(&free),
            locked: format_decimal(&locked),
            total: format_decimal(&total),
        });
    }
    assets.sort_by(|left, right| left.asset.cmp(&right.asset));

    Ok(ProductLiveAccountSnapshot {
        account_type,
        can_trade,
        can_withdraw,
        can_deposit,
        source_balance_entry_count: balances.len(),
        zero_balance_entry_count: balances.len().saturating_sub(assets.len()),
        assets,
    })
}

fn product_live_account_token_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .chars()
            .all(|character| character.is_ascii_uppercase() || character == '_')
}

fn product_live_asset_code_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .chars()
            .all(|character| character.is_ascii_uppercase() || character.is_ascii_digit())
}

fn parse_product_live_account_decimal(value: &str) -> Result<Decimal, &'static str> {
    let (integer, fraction) = match value.split_once('.') {
        Some((integer, fraction)) if !fraction.contains('.') => (integer, Some(fraction)),
        Some(_) => return Err("account_result_invalid"),
        None => (value, None),
    };
    let integer_is_canonical = !integer.is_empty()
        && (integer == "0"
            || (!integer.starts_with('0')
                && integer.chars().all(|character| character.is_ascii_digit())));
    let fraction_is_canonical = fraction.is_none_or(|fraction| {
        !fraction.is_empty() && fraction.chars().all(|character| character.is_ascii_digit())
    });
    if !integer_is_canonical || !fraction_is_canonical {
        return Err("account_result_invalid");
    }
    Decimal::from_str_exact(value).map_err(|_| "account_result_invalid")
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
    let mut source_artifact_issues = production_mutation_request_builder_source_issues(
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
    let price_safety = production_mutation_request_builder_price_safety_preflight(&price, opt);
    source_artifact_issues.extend(price_safety.source_artifact_issues.clone());
    source_artifact_issues.sort();
    source_artifact_issues.dedup();
    let price_safety_send_consideration_allowed = price_safety.preflight_ready
        && opt.confirm_non_marketable_price
        && opt.confirm_owner_acknowledged_no_cancel_path;
    let price_safety_manual_review_required = !price_safety_send_consideration_allowed;
    let price_safety_new_orders_blocked = !price_safety_send_consideration_allowed;
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
        && price_safety_send_consideration_allowed
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
        market_reference_source: price_safety.market_reference_source,
        market_reference_price: price_safety.market_reference_price,
        max_reference_price_distance_bps: price_safety.max_reference_price_distance_bps,
        price_distance_from_reference_bps: price_safety.price_distance_from_reference_bps,
        would_cross_spread: opt.would_cross_spread,
        non_marketable_price_preflight_ready: price_safety.preflight_ready,
        owner_acknowledged_no_cancel_path: opt.confirm_owner_acknowledged_no_cancel_path,
        price_safety_send_consideration_allowed,
        manual_review_required: price_safety_manual_review_required,
        new_orders_blocked: price_safety_new_orders_blocked,
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
) -> anyhow::Result<ProductionMutationGuardedSendArtifact> {
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
    let missing_env_vars = Vec::new();
    let mut source_artifact_issues = production_mutation_guarded_send_source_issues(
        &request_builder,
        &pre_send_kill_switch_runtime_gate,
        &request_preview,
        &opt.max_notional,
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
        && pre_send_kill_switch_clean;
    let single_shot_send_allowed = false;
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
    let counters = retired_production_mutation_guarded_send_counters();
    let status = if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
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
        mode: "retired_guarded_send_offline_evaluation".to_string(),
        capability: "Historical Production Mutation Artifact Evaluation".to_string(),
        manual_online_requested: false,
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
        credential_material: "retired_not_read".to_string(),
        production_signing_material_gate_required: false,
        production_signing_material_gate_open: false,
        production_signing_material_env_read: false,
        production_signing_material_missing_gate_env_vars: vec![
            "production_mutation_executor_retired_after_v0.32.0".to_string(),
        ],
        api_key_env: "retired".to_string(),
        api_secret_env: "retired".to_string(),
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
        http_status_code: None,
        latency_ms: None,
        error_code: "not_attempted_executor_retired".to_string(),
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
    let redacted_response_derived_from_actual_http_result =
        production_mutation_response_redaction_has_actual_http_result(&guarded_send);
    let synthetic_fixture_redaction_only = !redacted_response_derived_from_actual_http_result;
    let owner_run_mutation_closure_evidence =
        response_redaction_ready && redacted_response_derived_from_actual_http_result;

    Ok(ProductionMutationResponseRedactionArtifact {
        schema_version: PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        source_guarded_send_path: opt.guarded_send.display().to_string(),
        source_response_path: opt.response.display().to_string(),
        response_redaction_source: if redacted_response_derived_from_actual_http_result {
            "actual_guarded_send_http_result"
        } else {
            "synthetic_fixture"
        }
        .to_string(),
        source_guarded_send_run_id: json_string_value(&guarded_send, "run_id")
            .unwrap_or_else(|| "unknown".to_string()),
        source_guarded_send_hash: file_fnv1a64_hash(&opt.guarded_send.display().to_string()),
        redacted_response_derived_from_actual_http_result,
        synthetic_fixture_redaction_only,
        owner_run_mutation_closure_evidence,
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

fn build_production_mutation_local_order_ledger_artifact(
    opt: &LiveProductionMutationLocalOrderLedgerOpt,
) -> anyhow::Result<ProductionMutationLocalOrderLedgerArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("order_lineage_id", &opt.order_lineage_id)?;

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
    let audit_trail = load_json_value(&opt.audit_trail, "production mutation audit trail")?;
    let failure_semantics = load_json_value(
        &opt.failure_semantics,
        "production mutation failure semantics",
    )?;

    let missing_cli_flags = missing_production_mutation_local_order_ledger_cli_flags(opt);
    let source_artifact_issues = production_mutation_local_order_ledger_source_issues(
        &request_builder,
        &guarded_send,
        &response_redaction,
        &order_state_readback,
        &audit_trail,
        &failure_semantics,
    );
    let local_ledger_ready = missing_cli_flags.is_empty() && source_artifact_issues.is_empty();
    let status = if local_ledger_ready {
        "ready_local_order_ledger"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else {
        "blocked_source_artifact"
    };
    let current_local_state = if local_ledger_ready {
        "local_ledger_pending_exchange_reconciliation"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else {
        "blocked_source_artifact"
    };
    let request_sent = json_bool_value(&audit_trail, "request_sent")
        .or_else(|| json_bool_value(&guarded_send, "request_sent"))
        .unwrap_or(false);
    let network_attempted = json_bool_value(&audit_trail, "network_attempted")
        .or_else(|| json_bool_value(&guarded_send, "network_attempted"))
        .unwrap_or(false);
    let production_order_state_reads_allowed = json_bool_value(
        &order_state_readback,
        "production_order_state_reads_allowed",
    )
    .unwrap_or(false);
    let production_order_state_reads_attempted = json_u64_value(
        &order_state_readback,
        "production_order_state_reads_attempted",
    )
    .unwrap_or(0);

    Ok(ProductionMutationLocalOrderLedgerArtifact {
        schema_version: PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id: opt.order_lineage_id.clone(),
        artifact_type: "production_mutation_local_order_ledger".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_local_reconciliation_ledger".to_string(),
        capability: "Production Reconciliation And Orphan Recovery Evidence".to_string(),
        capability_expansion_from_v16: "reconciliation_evidence_only".to_string(),
        lineage_scope: "single_v16_mutation_candidate".to_string(),
        current_local_state: current_local_state.to_string(),
        default_fail_closed: true,
        owner_gated_readback_required: true,
        local_ledger_ready,
        restart_readable: local_ledger_ready,
        request_builder_ref: production_mutation_local_order_ledger_source_ref(
            &opt.request_builder,
            &request_builder,
            "request_builder_ready",
        ),
        guarded_send_ref: production_mutation_local_order_ledger_source_ref(
            &opt.guarded_send,
            &guarded_send,
            "guarded_send_ready",
        ),
        response_redaction_ref: production_mutation_local_order_ledger_source_ref(
            &opt.response_redaction,
            &response_redaction,
            "response_redaction_ready",
        ),
        readback_ref: production_mutation_local_order_ledger_source_ref(
            &opt.order_state_readback,
            &order_state_readback,
            "readback_contract_ready",
        ),
        audit_ref: production_mutation_local_order_ledger_source_ref(
            &opt.audit_trail,
            &audit_trail,
            "audit_trail_ready",
        ),
        failure_ref: production_mutation_local_order_ledger_source_ref(
            &opt.failure_semantics,
            &failure_semantics,
            "failure_semantics_ready",
        ),
        symbol: json_scalar_string_value(&audit_trail, "symbol")
            .or_else(|| json_scalar_string_value(&response_redaction, "symbol"))
            .unwrap_or_else(|| "unknown".to_string()),
        side: json_scalar_string_value(&audit_trail, "side")
            .or_else(|| json_scalar_string_value(&response_redaction, "side"))
            .unwrap_or_else(|| "unknown".to_string()),
        order_type: json_scalar_string_value(&audit_trail, "order_type")
            .or_else(|| json_scalar_string_value(&response_redaction, "order_type"))
            .unwrap_or_else(|| "unknown".to_string()),
        time_in_force: json_scalar_string_value(&audit_trail, "time_in_force")
            .or_else(|| json_scalar_string_value(&response_redaction, "time_in_force"))
            .unwrap_or_else(|| "unknown".to_string()),
        order_id: json_scalar_string_value(&audit_trail, "order_id")
            .or_else(|| json_scalar_string_value(&response_redaction, "order_id"))
            .unwrap_or_else(|| "missing".to_string()),
        client_order_id: json_scalar_string_value(&audit_trail, "client_order_id")
            .or_else(|| json_scalar_string_value(&response_redaction, "client_order_id"))
            .unwrap_or_else(|| "missing".to_string()),
        exchange_status: json_scalar_string_value(&audit_trail, "exchange_status")
            .or_else(|| json_scalar_string_value(&response_redaction, "exchange_status"))
            .unwrap_or_else(|| "unknown".to_string()),
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        exchange_readback_mapped: false,
        reconciliation_classified: false,
        orphan_risk_detected: false,
        manual_review_required: !local_ledger_ready,
        new_orders_blocked: !local_ledger_ready,
        network_attempted,
        request_sent,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed,
        listen_key_lifecycle_allowed: false,
        production_order_submissions_attempted: json_u64_value(
            &audit_trail,
            "production_order_submissions_attempted",
        )
        .unwrap_or_else(|| {
            json_u64_value(&guarded_send, "production_order_submissions_attempted").unwrap_or(0)
        }),
        production_orders_submitted: json_u64_value(&audit_trail, "production_orders_submitted")
            .unwrap_or_else(|| {
                json_u64_value(&guarded_send, "production_orders_submitted").unwrap_or(0)
            }),
        production_order_mutations_attempted: json_u64_value(
            &audit_trail,
            "production_order_mutations_attempted",
        )
        .unwrap_or_else(|| {
            json_u64_value(&guarded_send, "production_order_mutations_attempted").unwrap_or(0)
        }),
        production_order_state_reads_attempted,
        listen_key_lifecycle_attempted: 0,
        duplicate_submit_attempted: false,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        no_network_confirmed: opt.confirm_no_network,
        no_duplicate_submit_confirmed: opt.confirm_no_duplicate_submit,
        no_retry_confirmed: opt.confirm_no_retry,
        no_cancel_confirmed: opt.confirm_no_cancel,
        no_remediation_confirmed: opt.confirm_no_remediation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        diagnostic: if local_ledger_ready {
            "local ledger links the single v0.16 mutation candidate evidence chain for later read-only reconciliation; no duplicate submit, retry, cancel, remediation, or Dashboard order control is enabled"
        } else {
            "local order ledger is blocked because required confirmations or source artifact evidence are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_exchange_readback_mapper_artifact(
    opt: &LiveProductionMutationExchangeReadbackMapperOpt,
) -> anyhow::Result<ProductionMutationExchangeReadbackMapperArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let local_order_ledger = load_json_value(
        &opt.local_order_ledger,
        "production mutation local order ledger",
    )?;
    let order_readback = load_json_value(&opt.order_readback, "redacted order readback")?;
    let open_orders_readback =
        load_json_value(&opt.open_orders_readback, "redacted openOrders readback")?;
    let missing_cli_flags = missing_production_mutation_exchange_readback_mapper_cli_flags(opt);
    let source_artifact_issues = production_mutation_exchange_readback_mapper_source_issues(
        &local_order_ledger,
        &order_readback,
        &open_orders_readback,
    );
    let malformed_readback_issues =
        production_mutation_exchange_readback_mapper_malformed_issues(&order_readback);
    let mapping_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && malformed_readback_issues.is_empty();
    let order_lineage_id = json_string_value(&local_order_ledger, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_order_id = json_scalar_string_value(&local_order_ledger, "order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id = json_scalar_string_value(&local_order_ledger, "client_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let symbol = json_scalar_string_value(&local_order_ledger, "symbol")
        .unwrap_or_else(|| "unknown".to_string());
    let order_found = json_bool_value(&order_readback, "order_found").unwrap_or(true);
    let exchange_order_status = if order_found {
        json_scalar_string_value(&order_readback, "exchange_status").map_or_else(
            || "MALFORMED".to_string(),
            |status| status.to_ascii_uppercase(),
        )
    } else {
        "MISSING".to_string()
    };
    let exchange_order_state = normalized_exchange_order_state(&exchange_order_status);
    let open_orders_count = json_array_len(&open_orders_readback, "open_orders").unwrap_or(0);
    let open_order_observed = exchange_open_order_observed(
        &open_orders_readback,
        &known_order_id,
        &known_client_order_id,
        &symbol,
    );
    let terminal_state_observed = exchange_order_status_is_terminal(&exchange_order_status);
    let status = if mapping_ready {
        "ready_exchange_readback_mapped"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !malformed_readback_issues.is_empty() {
        "blocked_malformed_exchange_readback"
    } else {
        "blocked_source_artifact"
    };

    Ok(ProductionMutationExchangeReadbackMapperArtifact {
        schema_version: PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id,
        artifact_type: "production_mutation_exchange_readback_mapper".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_exchange_readback_mapper".to_string(),
        capability: "Production Reconciliation And Orphan Recovery Evidence".to_string(),
        capability_expansion_from_v16: "reconciliation_evidence_only".to_string(),
        lineage_scope: "single_v16_mutation_candidate".to_string(),
        default_fail_closed: true,
        owner_gated_readback_required: true,
        local_ledger_ready: json_bool_value(&local_order_ledger, "local_ledger_ready")
            .unwrap_or(false),
        exchange_readback_mapped: mapping_ready,
        reconciliation_classified: false,
        orphan_risk_detected: false,
        local_ledger_ref: production_mutation_local_order_ledger_source_ref(
            &opt.local_order_ledger,
            &local_order_ledger,
            "local_ledger_ready",
        ),
        order_readback_ref: production_mutation_local_order_ledger_source_ref(
            &opt.order_readback,
            &order_readback,
            "response_redacted",
        ),
        open_orders_readback_ref: production_mutation_local_order_ledger_source_ref(
            &opt.open_orders_readback,
            &open_orders_readback,
            "response_redacted",
        ),
        known_order_id,
        known_client_order_id,
        symbol,
        exchange_order_status,
        exchange_order_state: exchange_order_state.to_string(),
        open_order_observed,
        terminal_state_observed,
        order_found,
        open_orders_count,
        source_artifact_issues,
        malformed_readback_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        manual_review_required: !mapping_ready,
        new_orders_blocked: !mapping_ready,
        network_attempted: false,
        request_sent: false,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        duplicate_submit_attempted: false,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        redacted_readback_metadata_only_confirmed: opt.confirm_redacted_readback_metadata_only,
        known_order_identifier_only_confirmed: opt.confirm_known_order_identifier_only,
        read_only_reconciliation_scope_confirmed: opt.confirm_read_only_reconciliation_scope,
        no_network_confirmed: opt.confirm_no_network,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        no_production_order_mutation_confirmed: opt.confirm_no_production_order_mutation,
        no_retry_confirmed: opt.confirm_no_retry,
        no_cancel_confirmed: opt.confirm_no_cancel,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        diagnostic: if mapping_ready {
            "redacted exchange readback metadata was normalized for the single local lineage; classification and orphan decisions remain later v0.17 steps"
        } else {
            "exchange readback mapper is blocked because required confirmations, source artifacts, or redacted readback shape are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_reconciliation_classifier_artifact(
    opt: &LiveProductionMutationReconciliationClassifierOpt,
) -> anyhow::Result<ProductionMutationReconciliationClassifierArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let exchange_readback_mapper =
        load_json_value(&opt.exchange_readback_mapper, "exchange readback mapper")?;
    let missing_cli_flags = missing_production_mutation_reconciliation_classifier_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_reconciliation_classifier_source_issues(&exchange_readback_mapper);
    let classification_ready = missing_cli_flags.is_empty() && source_artifact_issues.is_empty();
    let (reconciliation_outcome, outcome_manual_review_required, outcome_new_orders_blocked) =
        classify_production_mutation_reconciliation_outcome(&exchange_readback_mapper);
    let failure_semantics =
        production_mutation_failure_semantics_from_exchange_mapper(&exchange_readback_mapper);
    let failure_incident =
        classify_production_mutation_failure_incident(failure_semantics.as_ref());
    let status = if classification_ready {
        "ready_reconciliation_classified"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else {
        "blocked_source_artifact"
    };
    let exchange_order_status =
        json_scalar_string_value(&exchange_readback_mapper, "exchange_order_status")
            .unwrap_or_else(|| "UNKNOWN".to_string());
    let exchange_order_state = json_string_value(&exchange_readback_mapper, "exchange_order_state")
        .unwrap_or_else(|| "unknown".to_string());
    let local_request_sent =
        json_bool_value(&exchange_readback_mapper, "request_sent").unwrap_or(false);
    let manual_review_required = !classification_ready
        || outcome_manual_review_required
        || failure_incident.manual_review_required;
    let new_orders_blocked =
        !classification_ready || outcome_new_orders_blocked || failure_incident.new_orders_blocked;

    Ok(ProductionMutationReconciliationClassifierArtifact {
        schema_version: PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id: json_string_value(&exchange_readback_mapper, "order_lineage_id")
            .unwrap_or_else(|| "missing".to_string()),
        artifact_type: "production_mutation_reconciliation_classifier".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_reconciliation_classifier".to_string(),
        capability: "Production Reconciliation And Orphan Recovery Evidence".to_string(),
        capability_expansion_from_v16: "reconciliation_evidence_only".to_string(),
        lineage_scope: "single_v16_mutation_candidate".to_string(),
        default_fail_closed: true,
        owner_gated_readback_required: true,
        exchange_readback_mapper_ref: production_mutation_local_order_ledger_source_ref(
            &opt.exchange_readback_mapper,
            &exchange_readback_mapper,
            "exchange_readback_mapped",
        ),
        exchange_readback_mapped: json_bool_value(
            &exchange_readback_mapper,
            "exchange_readback_mapped",
        )
        .unwrap_or(false),
        reconciliation_classified: classification_ready,
        orphan_risk_detected: false,
        local_request_sent,
        exchange_order_status,
        exchange_order_state,
        open_order_observed: json_bool_value(&exchange_readback_mapper, "open_order_observed")
            .unwrap_or(false),
        terminal_state_observed: json_bool_value(
            &exchange_readback_mapper,
            "terminal_state_observed",
        )
        .unwrap_or(false),
        order_found: json_bool_value(&exchange_readback_mapper, "order_found").unwrap_or(false),
        reconciliation_outcome: reconciliation_outcome.to_string(),
        failure_mode: failure_incident.failure_mode.clone(),
        failure_state: failure_incident.failure_state.clone(),
        terminal_action: failure_incident.terminal_action.clone(),
        failure_incident_outcome: failure_incident.outcome.to_string(),
        failure_incident_severity: failure_incident.severity.to_string(),
        readback_required: failure_incident.readback_required,
        terminal_evidence_required: failure_incident.terminal_evidence_required,
        incident_risk_halted: failure_incident.risk_halted,
        incident_manual_review_required: failure_incident.manual_review_required,
        incident_new_orders_blocked: failure_incident.new_orders_blocked,
        failure_semantics_path: failure_incident.source_path,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        manual_review_required,
        new_orders_blocked,
        network_attempted: false,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        duplicate_submit_attempted: false,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        single_v16_mutation_candidate_lineage_confirmed: opt
            .confirm_single_v16_mutation_candidate_lineage,
        read_only_reconciliation_scope_confirmed: opt.confirm_read_only_reconciliation_scope,
        no_retry_confirmed: opt.confirm_no_retry,
        no_cancel_confirmed: opt.confirm_no_cancel,
        no_remediation_confirmed: opt.confirm_no_remediation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        diagnostic: if classification_ready {
            "local ledger evidence and exchange readback mapper evidence were classified for one mutation candidate lineage; orphan detection and remediation remain later v0.17 steps"
        } else {
            "reconciliation classification is blocked because required confirmations or source mapper evidence are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_orphan_order_detector_artifact(
    opt: &LiveProductionMutationOrphanOrderDetectorOpt,
) -> anyhow::Result<ProductionMutationOrphanOrderDetectorArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let reconciliation_classifier =
        load_json_value(&opt.reconciliation_classifier, "reconciliation classifier")?;
    let missing_cli_flags = missing_production_mutation_orphan_order_detector_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_orphan_order_detector_source_issues(&reconciliation_classifier);
    let detection_ready = missing_cli_flags.is_empty() && source_artifact_issues.is_empty();
    let detection = detect_production_mutation_orphan_order_risk(&reconciliation_classifier);
    let status = if detection_ready {
        "ready_orphan_order_detection_completed"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else {
        "blocked_source_artifact"
    };
    let incident_risk_halted =
        json_bool_value(&reconciliation_classifier, "incident_risk_halted").unwrap_or(false);
    let classifier_manual_review_required =
        json_bool_value(&reconciliation_classifier, "manual_review_required").unwrap_or(false);
    let classifier_new_orders_blocked =
        json_bool_value(&reconciliation_classifier, "new_orders_blocked").unwrap_or(false);
    let risk_halted = !detection_ready || detection.risk_halted || incident_risk_halted;
    let manual_review_required =
        !detection_ready || detection.manual_review_required || classifier_manual_review_required;
    let new_orders_blocked =
        !detection_ready || detection.new_orders_blocked || classifier_new_orders_blocked;

    Ok(ProductionMutationOrphanOrderDetectorArtifact {
        schema_version: PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id: json_string_value(&reconciliation_classifier, "order_lineage_id")
            .unwrap_or_else(|| "missing".to_string()),
        artifact_type: "production_mutation_orphan_order_detector".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_orphan_order_detector".to_string(),
        capability: "Production Reconciliation And Orphan Recovery Evidence".to_string(),
        capability_expansion_from_v16: "orphan_detection_evidence_only".to_string(),
        lineage_scope: "single_v16_mutation_candidate".to_string(),
        default_fail_closed: true,
        owner_gated_readback_required: true,
        reconciliation_classifier_ref: production_mutation_local_order_ledger_source_ref(
            &opt.reconciliation_classifier,
            &reconciliation_classifier,
            "reconciliation_classified",
        ),
        reconciliation_classified: json_bool_value(
            &reconciliation_classifier,
            "reconciliation_classified",
        )
        .unwrap_or(false),
        orphan_detection_completed: detection_ready,
        orphan_risk_detected: detection.orphan_risk_detected,
        orphan_detection_outcome: detection.outcome.to_string(),
        reconciliation_outcome: json_string_value(
            &reconciliation_classifier,
            "reconciliation_outcome",
        )
        .unwrap_or_else(|| "unknown".to_string()),
        local_request_sent: json_bool_value(&reconciliation_classifier, "local_request_sent")
            .unwrap_or(false),
        local_terminal_state: detection.local_terminal_state,
        exchange_order_status: json_scalar_string_value(
            &reconciliation_classifier,
            "exchange_order_status",
        )
        .unwrap_or_else(|| "UNKNOWN".to_string()),
        exchange_order_state: json_string_value(
            &reconciliation_classifier,
            "exchange_order_state",
        )
        .unwrap_or_else(|| "unknown".to_string()),
        open_order_observed: json_bool_value(&reconciliation_classifier, "open_order_observed")
            .unwrap_or(false),
        terminal_state_observed: json_bool_value(
            &reconciliation_classifier,
            "terminal_state_observed",
        )
        .unwrap_or(false),
        order_found: json_bool_value(&reconciliation_classifier, "order_found").unwrap_or(false),
        failure_mode: json_string_value(&reconciliation_classifier, "failure_mode")
            .unwrap_or_else(|| "none".to_string()),
        failure_incident_outcome: json_string_value(
            &reconciliation_classifier,
            "failure_incident_outcome",
        )
        .unwrap_or_else(|| "not_linked".to_string()),
        readback_required: json_bool_value(&reconciliation_classifier, "readback_required")
            .unwrap_or(false),
        incident_risk_halted: json_bool_value(&reconciliation_classifier, "incident_risk_halted")
            .unwrap_or(false),
        risk_halted,
        new_orders_blocked,
        manual_review_required,
        stale_ledger_restart_required: detection.stale_ledger_restart_required,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        network_attempted: false,
        production_order_submission_allowed: false,
        production_order_mutation_allowed: false,
        production_order_state_reads_allowed: false,
        listen_key_lifecycle_allowed: false,
        duplicate_submit_attempted: false,
        retry_attempted: false,
        cancel_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        single_v16_mutation_candidate_lineage_confirmed: opt
            .confirm_single_v16_mutation_candidate_lineage,
        read_only_reconciliation_scope_confirmed: opt.confirm_read_only_reconciliation_scope,
        no_retry_confirmed: opt.confirm_no_retry,
        no_cancel_confirmed: opt.confirm_no_cancel,
        no_remediation_confirmed: opt.confirm_no_remediation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        diagnostic: if detection_ready {
            "orphan order risk was detected from local reconciliation evidence only; cancel/retry/remediation remain disabled"
        } else {
            "orphan order detection is blocked because required confirmations or classifier evidence are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_cancel_request_preview_artifact(
    opt: &LiveProductionMutationCancelRequestPreviewOpt,
) -> anyhow::Result<ProductionMutationCancelRequestPreviewArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("account_label", &opt.account_label)?;

    let orphan_order_detector =
        load_json_value(&opt.orphan_order_detector, "orphan order detector")?;
    let reconciliation_classifier_path = source_ref_path(
        &opt.orphan_order_detector,
        &orphan_order_detector,
        "reconciliation_classifier_ref",
        "orphan order detector reconciliation classifier ref",
    )?;
    let reconciliation_classifier =
        load_json_value(&reconciliation_classifier_path, "reconciliation classifier")?;
    let exchange_readback_mapper_path = source_ref_path(
        &reconciliation_classifier_path,
        &reconciliation_classifier,
        "exchange_readback_mapper_ref",
        "reconciliation classifier exchange readback mapper ref",
    )?;
    let exchange_readback_mapper =
        load_json_value(&exchange_readback_mapper_path, "exchange readback mapper")?;

    let missing_cli_flags = missing_production_mutation_cancel_request_preview_cli_flags(opt);
    let source_artifact_issues = production_mutation_cancel_request_preview_source_issues(
        &orphan_order_detector,
        &reconciliation_classifier,
        &exchange_readback_mapper,
    );
    let preview_ready = missing_cli_flags.is_empty() && source_artifact_issues.is_empty();
    let status = if preview_ready {
        "ready_cancel_request_preview"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else {
        "blocked_source_artifact"
    };
    let order_lineage_id = json_string_value(&orphan_order_detector, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    let raw_order_id = json_scalar_string_value(&exchange_readback_mapper, "known_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let raw_client_order_id =
        json_scalar_string_value(&exchange_readback_mapper, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string());
    let order_identifier_known = cancel_preview_identifier_known(&raw_order_id)
        || cancel_preview_identifier_known(&raw_client_order_id);
    let orphan_detection_outcome =
        json_string_value(&orphan_order_detector, "orphan_detection_outcome")
            .unwrap_or_else(|| "unknown".to_string());

    Ok(ProductionMutationCancelRequestPreviewArtifact {
        schema_version: PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id,
        artifact_type: "cancel_request_preview".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_cancel_request_preview".to_string(),
        capability: "Owner-Approved Cancel Recovery Preview".to_string(),
        capability_expansion: "preview_gate_approval_only".to_string(),
        lineage_scope: "single_v16_mutation_candidate".to_string(),
        default_fail_closed: true,
        owner_approval_required: true,
        owner_approval_lifecycle_recorded: false,
        cancel_candidate_source: "production_mutation_orphan_order_detector".to_string(),
        orphan_order_detector_ref: production_mutation_local_order_ledger_source_ref(
            &opt.orphan_order_detector,
            &orphan_order_detector,
            "orphan_detection_completed",
        ),
        reconciliation_classifier_ref: production_mutation_local_order_ledger_source_ref(
            &reconciliation_classifier_path,
            &reconciliation_classifier,
            "reconciliation_classified",
        ),
        exchange_readback_mapper_ref: production_mutation_local_order_ledger_source_ref(
            &exchange_readback_mapper_path,
            &exchange_readback_mapper,
            "exchange_readback_mapped",
        ),
        orphan_risk_detected: json_bool_value(&orphan_order_detector, "orphan_risk_detected")
            .unwrap_or(false),
        risk_halted: json_bool_value(&orphan_order_detector, "risk_halted").unwrap_or(false),
        new_orders_blocked: json_bool_value(&orphan_order_detector, "new_orders_blocked")
            .unwrap_or(false),
        manual_review_required: json_bool_value(
            &orphan_order_detector,
            "manual_review_required",
        )
        .unwrap_or(false),
        cancel_request_preview_ready: preview_ready,
        order_identifier_known,
        known_order_id: redact_cancel_preview_identifier("order_id", &raw_order_id),
        known_client_order_id: redact_cancel_preview_identifier(
            "client_order_id",
            &raw_client_order_id,
        ),
        symbol: json_scalar_string_value(&exchange_readback_mapper, "symbol")
            .unwrap_or_else(|| "unknown".to_string()),
        account_label: opt.account_label.clone(),
        cancel_reason: cancel_request_preview_reason(&orphan_detection_outcome).to_string(),
        orphan_detection_outcome,
        candidate_count: 1,
        multi_order_cancel_requested: false,
        cancel_all_requested: false,
        bulk_cancel_requested: false,
        strategy_driven_cancel_requested: false,
        multi_account_cancel_requested: false,
        multi_venue_cancel_requested: false,
        retry_requested: false,
        replace_or_amend_requested: false,
        flatten_requested: false,
        dashboard_cancel_requested: false,
        actual_cancel_send_allowed: false,
        cancel_attempted: false,
        cancel_requests_sent: 0,
        network_attempted: false,
        network_cancel_endpoint_attempted: false,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        production_order_mutation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        single_v16_mutation_candidate_lineage_confirmed: opt
            .confirm_single_v16_mutation_candidate_lineage,
        orphan_risk_halted_confirmed: opt.confirm_orphan_risk_halted,
        manual_review_required_confirmed: opt.confirm_manual_review_required,
        known_order_identifier_only_confirmed: opt.confirm_known_order_identifier_only,
        no_retry_confirmed: opt.confirm_no_retry,
        no_cancel_confirmed: opt.confirm_no_cancel,
        no_network_confirmed: opt.confirm_no_network,
        no_remediation_confirmed: opt.confirm_no_remediation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        diagnostic: if preview_ready {
            "cancel request preview is ready for one redacted known-order candidate; no cancel send, network endpoint, retry, remediation, or Dashboard cancel control is enabled"
        } else {
            "cancel request preview is blocked because required confirmations or v0.17 orphan-risk source evidence are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_cancel_risk_gate_artifact(
    opt: &LiveProductionMutationCancelRiskGateOpt,
) -> anyhow::Result<ProductionMutationCancelRiskGateArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("expected_symbol", &opt.expected_symbol)?;
    validate_non_empty("expected_account_label", &opt.expected_account_label)?;

    let cancel_request_preview =
        load_json_value(&opt.cancel_request_preview, "cancel request preview")?;
    let missing_cli_flags = missing_production_mutation_cancel_risk_gate_cli_flags(opt);
    let source_artifact_issues = production_mutation_cancel_risk_gate_source_issues(
        &cancel_request_preview,
        &opt.expected_symbol,
        &opt.expected_account_label,
    );
    let gate_ready = missing_cli_flags.is_empty() && source_artifact_issues.is_empty();
    let status = if gate_ready {
        "ready_cancel_risk_gate"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else {
        "blocked_source_artifact"
    };
    let order_lineage_id = json_string_value(&cancel_request_preview, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    let symbol = json_scalar_string_value(&cancel_request_preview, "symbol")
        .unwrap_or_else(|| "unknown".to_string());
    let account_label = json_scalar_string_value(&cancel_request_preview, "account_label")
        .unwrap_or_else(|| "unknown".to_string());

    Ok(ProductionMutationCancelRiskGateArtifact {
        schema_version: PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id,
        artifact_type: "cancel_risk_gate".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_cancel_risk_gate".to_string(),
        capability: "Owner-Approved Cancel Recovery Preview".to_string(),
        capability_expansion: "preview_gate_approval_only".to_string(),
        lineage_scope: json_string_value(&cancel_request_preview, "lineage_scope")
            .unwrap_or_else(|| "unknown".to_string()),
        default_fail_closed: true,
        cancel_request_preview_ref: production_mutation_local_order_ledger_source_ref(
            &opt.cancel_request_preview,
            &cancel_request_preview,
            "ready_cancel_request_preview",
        ),
        cancel_request_preview_ready: json_bool_value(
            &cancel_request_preview,
            "cancel_request_preview_ready",
        )
        .unwrap_or(false),
        cancel_risk_gate_ready: gate_ready,
        orphan_risk_detected: json_bool_value(&cancel_request_preview, "orphan_risk_detected")
            .unwrap_or(false),
        risk_halted: json_bool_value(&cancel_request_preview, "risk_halted").unwrap_or(false),
        new_orders_blocked: json_bool_value(&cancel_request_preview, "new_orders_blocked")
            .unwrap_or(false),
        manual_review_required: json_bool_value(&cancel_request_preview, "manual_review_required")
            .unwrap_or(false),
        order_identifier_known: json_bool_value(&cancel_request_preview, "order_identifier_known")
            .unwrap_or(false),
        known_order_id: json_scalar_string_value(&cancel_request_preview, "known_order_id")
            .unwrap_or_else(|| "missing".to_string()),
        known_client_order_id: json_scalar_string_value(
            &cancel_request_preview,
            "known_client_order_id",
        )
        .unwrap_or_else(|| "missing".to_string()),
        symbol: symbol.clone(),
        expected_symbol: opt.expected_symbol.clone(),
        symbol_matches_lineage: cancel_risk_gate_field_matches(&symbol, &opt.expected_symbol),
        account_label: account_label.clone(),
        expected_account_label: opt.expected_account_label.clone(),
        account_matches_lineage: cancel_risk_gate_field_matches(
            &account_label,
            &opt.expected_account_label,
        ),
        owner_approval_required: json_bool_value(
            &cancel_request_preview,
            "owner_approval_required",
        )
        .unwrap_or(false),
        owner_approval_lifecycle_recorded: json_bool_value(
            &cancel_request_preview,
            "owner_approval_lifecycle_recorded",
        )
        .unwrap_or(false),
        candidate_count: json_u64_value(&cancel_request_preview, "candidate_count").unwrap_or(0),
        multi_order_cancel_requested: json_bool_value(
            &cancel_request_preview,
            "multi_order_cancel_requested",
        )
        .unwrap_or(false),
        cancel_all_requested: json_bool_value(&cancel_request_preview, "cancel_all_requested")
            .unwrap_or(false),
        bulk_cancel_requested: json_bool_value(&cancel_request_preview, "bulk_cancel_requested")
            .unwrap_or(false),
        strategy_driven_cancel_requested: json_bool_value(
            &cancel_request_preview,
            "strategy_driven_cancel_requested",
        )
        .unwrap_or(false),
        multi_account_cancel_requested: json_bool_value(
            &cancel_request_preview,
            "multi_account_cancel_requested",
        )
        .unwrap_or(false),
        multi_venue_cancel_requested: json_bool_value(
            &cancel_request_preview,
            "multi_venue_cancel_requested",
        )
        .unwrap_or(false),
        retry_requested: json_bool_value(&cancel_request_preview, "retry_requested")
            .unwrap_or(false),
        replace_or_amend_requested: json_bool_value(
            &cancel_request_preview,
            "replace_or_amend_requested",
        )
        .unwrap_or(false),
        flatten_requested: json_bool_value(&cancel_request_preview, "flatten_requested")
            .unwrap_or(false),
        dashboard_cancel_requested: json_bool_value(
            &cancel_request_preview,
            "dashboard_cancel_requested",
        )
        .unwrap_or(false),
        actual_cancel_send_allowed: false,
        cancel_attempted: false,
        cancel_requests_sent: 0,
        network_attempted: false,
        network_cancel_endpoint_attempted: false,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        production_order_mutation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        single_v16_mutation_candidate_lineage_confirmed: opt
            .confirm_single_v16_mutation_candidate_lineage,
        cancel_request_preview_ready_confirmed: opt.confirm_cancel_request_preview_ready,
        orphan_risk_halted_confirmed: opt.confirm_orphan_risk_halted,
        known_order_identifier_only_confirmed: opt.confirm_known_order_identifier_only,
        symbol_account_scope_confirmed: opt.confirm_symbol_account_scope,
        owner_approval_required_confirmed: opt.confirm_owner_approval_required,
        no_cancel_all_or_bulk_confirmed: opt.confirm_no_cancel_all_or_bulk,
        no_retry_confirmed: opt.confirm_no_retry,
        no_cancel_confirmed: opt.confirm_no_cancel,
        no_network_confirmed: opt.confirm_no_network,
        no_remediation_confirmed: opt.confirm_no_remediation,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        diagnostic: if gate_ready {
            "cancel risk gate is ready for one previewed known-order candidate; symbol/account scope matches and no cancel send, network endpoint, retry, remediation, bulk cancel, or Dashboard cancel control is enabled"
        } else {
            "cancel risk gate is blocked because required confirmations, preview readiness, lineage scope, symbol/account match, or forbidden-control checks are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_manual_owner_approval_lifecycle_artifact(
    opt: &LiveProductionMutationManualOwnerApprovalLifecycleOpt,
) -> anyhow::Result<ProductionMutationManualOwnerApprovalLifecycleArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    if opt.now_unix_ms == 0 {
        anyhow::bail!("manual owner approval lifecycle now_unix_ms must be positive");
    }
    if opt.expires_at_unix_ms == 0 {
        anyhow::bail!("manual owner approval lifecycle expires_at_unix_ms must be positive");
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

    let cancel_risk_gate = load_json_value(&opt.cancel_risk_gate, "cancel risk gate")?;
    let missing_cli_flags =
        missing_production_mutation_manual_owner_approval_lifecycle_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_manual_owner_approval_lifecycle_source_issues(&cancel_risk_gate);
    let approval_expired = approval_state == "expired" || opt.now_unix_ms > opt.expires_at_unix_ms;
    let approval_revoked = approval_state == "revoked";
    let approval_used = approval_state == "used";
    let manual_approval_recorded =
        manual_approval_id.is_some() && approved_by.is_some() && approval_state != "pending";

    let mut lifecycle_issues = Vec::new();
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

    let approval_lifecycle_valid = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && lifecycle_issues.is_empty();
    let status = if approval_lifecycle_valid {
        "approval_lifecycle_recorded_for_cancel_candidate"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
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
    let order_lineage_id = json_string_value(&cancel_risk_gate, "order_lineage_id")
        .unwrap_or_else(|| "missing".into());

    Ok(ProductionMutationManualOwnerApprovalLifecycleArtifact {
        schema_version: PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION
            .to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id,
        artifact_type: "manual_owner_approval_lifecycle".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_manual_owner_approval_lifecycle".to_string(),
        capability: "Owner-Approved Cancel Recovery Preview".to_string(),
        capability_expansion: "preview_gate_approval_only".to_string(),
        lineage_scope: json_string_value(&cancel_risk_gate, "lineage_scope")
            .unwrap_or_else(|| "unknown".to_string()),
        default_fail_closed: true,
        cancel_risk_gate_ref: production_mutation_local_order_ledger_source_ref(
            &opt.cancel_risk_gate,
            &cancel_risk_gate,
            "ready_cancel_risk_gate",
        ),
        cancel_risk_gate_ready: json_bool_value(&cancel_risk_gate, "cancel_risk_gate_ready")
            .unwrap_or(false),
        approval_scope: "one_order_cancel_candidate".to_string(),
        approval_source: "owner_manual_action".to_string(),
        approval_state: approval_state.to_string(),
        manual_approval_recorded,
        manual_approval_id,
        approved_by,
        now_unix_ms: opt.now_unix_ms,
        expires_at_unix_ms: opt.expires_at_unix_ms,
        approval_expires: true,
        approval_expired,
        approval_revoked,
        approval_used,
        approval_reusable: false,
        one_time_approval: true,
        approval_lifecycle_valid,
        owner_approval_required: json_bool_value(&cancel_risk_gate, "owner_approval_required")
            .unwrap_or(false),
        owner_approval_lifecycle_recorded: true,
        approval_consumed: false,
        approval_consumed_before_send: false,
        approval_consumed_after_send: false,
        known_order_id: json_scalar_string_value(&cancel_risk_gate, "known_order_id")
            .unwrap_or_else(|| "missing".to_string()),
        known_client_order_id: json_scalar_string_value(&cancel_risk_gate, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string()),
        symbol: json_scalar_string_value(&cancel_risk_gate, "symbol")
            .unwrap_or_else(|| "unknown".to_string()),
        account_label: json_scalar_string_value(&cancel_risk_gate, "account_label")
            .unwrap_or_else(|| "unknown".to_string()),
        candidate_count: json_u64_value(&cancel_risk_gate, "candidate_count").unwrap_or(0),
        strategy_auto_approval_allowed: false,
        strategy_auto_approval_attempted: false,
        background_auto_approval_allowed: false,
        background_auto_approval_attempted: false,
        dashboard_auto_approval_allowed: false,
        dashboard_auto_approval_attempted: false,
        incident_handler_auto_approval_allowed: false,
        incident_handler_auto_approval_attempted: false,
        actual_cancel_send_allowed: false,
        cancel_attempted: false,
        cancel_requests_sent: 0,
        network_attempted: false,
        network_cancel_endpoint_attempted: false,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        production_order_mutation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        lifecycle_issues,
        one_order_cancel_candidate_confirmed: opt.confirm_one_order_cancel_candidate,
        one_time_approval_confirmed: opt.confirm_one_time_approval,
        non_reusable_approval_confirmed: opt.confirm_non_reusable_approval,
        approval_expiry_confirmed: opt.confirm_approval_expiry,
        no_strategy_auto_approval_confirmed: opt.confirm_no_strategy_auto_approval,
        no_background_auto_approval_confirmed: opt.confirm_no_background_auto_approval,
        no_dashboard_cancel_approval_confirmed: opt.confirm_no_dashboard_cancel_approval,
        no_incident_handler_auto_approval_confirmed: opt
            .confirm_no_incident_handler_auto_approval,
        no_cancel_confirmed: opt.confirm_no_cancel,
        no_network_confirmed: opt.confirm_no_network,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        diagnostic: if approval_lifecycle_valid {
            "manual owner approval lifecycle is recorded for one cancel candidate only; approval is one-time, non-reusable, expirable, not consumed, and no cancel send is enabled"
        } else {
            "manual owner approval lifecycle is blocked because confirmations, source gate readiness, approval state, expiry, or one-time owner evidence are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_actual_cancel_owner_approval_lifecycle_artifact(
    opt: &LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt,
) -> anyhow::Result<ProductionMutationActualCancelOwnerApprovalLifecycleArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("expected_order_lineage_id", &opt.expected_order_lineage_id)?;
    validate_non_empty("expected_symbol", &opt.expected_symbol)?;
    validate_non_empty("expected_account_label", &opt.expected_account_label)?;
    validate_non_empty("venue", &opt.venue)?;
    validate_non_empty("expected_release_tag", &opt.expected_release_tag)?;
    if opt.now_unix_ms == 0 {
        anyhow::bail!("actual cancel owner approval lifecycle now_unix_ms must be positive");
    }
    if opt.expires_at_unix_ms == 0 {
        anyhow::bail!("actual cancel owner approval lifecycle expires_at_unix_ms must be positive");
    }

    let approval_state = opt.approval_state.trim();
    if !matches!(
        approval_state,
        "created" | "approved" | "expired" | "used" | "rejected" | "audited"
    ) {
        anyhow::bail!(
            "approval_state must be created, approved, expired, used, rejected, or audited"
        );
    }
    let manual_approval_id = optional_non_empty("manual_approval_id", &opt.manual_approval_id)?;
    let approved_by = optional_non_empty("approved_by", &opt.approved_by)?;
    let approval_reason = optional_non_empty("approval_reason", &opt.approval_reason)?;

    let safety_contract_raw =
        fs::read_to_string(&opt.actual_cancel_safety_contract).with_context(|| {
            format!(
                "failed to read actual cancel safety contract '{}'",
                opt.actual_cancel_safety_contract.display()
            )
        })?;
    let release_manifest = load_json_value(&opt.release_manifest, "release manifest")?;
    let cancel_risk_gate = load_json_value(&opt.cancel_risk_gate, "cancel risk gate")?;

    let missing_cli_flags =
        missing_production_mutation_actual_cancel_owner_approval_lifecycle_cli_flags(opt);
    let safety_contract_issues =
        actual_cancel_owner_approval_safety_contract_issues(&safety_contract_raw);
    let release_manifest_issues =
        actual_cancel_owner_approval_release_manifest_issues(&release_manifest, opt);
    let source_artifact_issues =
        production_mutation_actual_cancel_owner_approval_lifecycle_source_issues(
            &cancel_risk_gate,
            opt,
        );

    let approval_created = approval_state == "created";
    let approval_approved = approval_state == "approved";
    let approval_expired = approval_state == "expired" || opt.now_unix_ms > opt.expires_at_unix_ms;
    let approval_used = approval_state == "used";
    let approval_rejected = approval_state == "rejected";
    let approval_audited = approval_state == "audited";
    let approval_consumed = approval_used || approval_audited;
    let audit_evidence_recorded = (approval_used || approval_rejected || approval_audited)
        && manual_approval_id.is_some()
        && approved_by.is_some()
        && approval_reason.is_some();
    let manual_approval_recorded =
        manual_approval_id.is_some() && approved_by.is_some() && approval_reason.is_some();

    let mut lifecycle_issues = Vec::new();
    if !approval_approved {
        lifecycle_issues.push(format!("approval_state_{approval_state}"));
    }
    if approval_expired {
        lifecycle_issues.push("owner_approval_expired".to_string());
    }
    if approval_used {
        lifecycle_issues.push("owner_approval_reused".to_string());
    }
    if approval_rejected {
        lifecycle_issues.push("owner_approval_rejected".to_string());
    }
    if approval_audited {
        lifecycle_issues.push("owner_approval_already_audited".to_string());
    }
    if !manual_approval_recorded {
        lifecycle_issues.push("missing_owner_approval".to_string());
    }
    if (approval_used || approval_rejected || approval_audited) && !audit_evidence_recorded {
        lifecycle_issues.push("approval_audit_evidence_missing".to_string());
    }

    let approval_lifecycle_valid = missing_cli_flags.is_empty()
        && safety_contract_issues.is_empty()
        && release_manifest_issues.is_empty()
        && source_artifact_issues.is_empty()
        && lifecycle_issues.is_empty();
    let approval_execution_authorized = approval_lifecycle_valid && approval_approved;
    let status = if approval_execution_authorized {
        "approval_execution_lifecycle_ready"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !safety_contract_issues.is_empty() {
        "blocked_safety_contract"
    } else if !release_manifest_issues.is_empty() {
        "blocked_release_provenance"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
    } else if approval_created {
        "approval_created"
    } else if approval_expired {
        "approval_expired"
    } else if approval_used {
        "approval_used"
    } else if approval_rejected {
        "approval_rejected"
    } else if approval_audited {
        "approval_audited"
    } else {
        "approval_invalid"
    };
    let approval_failure_reason = first_actual_cancel_owner_approval_failure_reason(
        &missing_cli_flags,
        &safety_contract_issues,
        &release_manifest_issues,
        &source_artifact_issues,
        &lifecycle_issues,
    );
    let order_lineage_id = json_string_value(&cancel_risk_gate, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    let release_manifest_product_version = json_string_value(&release_manifest, "product_version")
        .unwrap_or_else(|| "missing".to_string());
    let release_manifest_planned_tag =
        json_pointer_string_value(&release_manifest, "/patch_release/planned_tag")
            .unwrap_or_else(|| "missing".to_string());
    let release_manifest_actual_cancel_scope =
        json_pointer_string_value(&release_manifest, "/capability/actual_cancel_scope")
            .unwrap_or_else(|| "missing".to_string());

    Ok(
        ProductionMutationActualCancelOwnerApprovalLifecycleArtifact {
            schema_version:
                PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION
                    .to_string(),
            run_id: opt.run_id.clone(),
            order_lineage_id,
            artifact_type: "actual_cancel_owner_approval_lifecycle".to_string(),
            status: status.to_string(),
            created_at: now_millis(),
            mode: "single_shot_actual_cancel_owner_approval_lifecycle".to_string(),
            capability: "Owner-Approved Single-Shot Actual Cancel".to_string(),
            execution_mode: "owner_approved_single_shot_manual_only".to_string(),
            approval_scope: "one_order_one_venue_one_attempt".to_string(),
            default_fail_closed: true,
            actual_cancel_safety_contract_ref: production_mutation_source_file_ref(
                &opt.actual_cancel_safety_contract,
            ),
            release_manifest_ref: production_mutation_source_file_ref(&opt.release_manifest),
            cancel_risk_gate_ref: production_mutation_local_order_ledger_source_ref(
                &opt.cancel_risk_gate,
                &cancel_risk_gate,
                "cancel_risk_gate_ready",
            ),
            safety_contract_ready: safety_contract_issues.is_empty(),
            release_provenance_ready: release_manifest_issues.is_empty(),
            cancel_risk_gate_ready: json_bool_value(&cancel_risk_gate, "cancel_risk_gate_ready")
                .unwrap_or(false),
            approval_state: approval_state.to_string(),
            approval_lifecycle_valid,
            approval_execution_authorized,
            approval_failure_reason,
            manual_approval_recorded,
            manual_approval_id,
            approved_by,
            approval_reason,
            now_unix_ms: opt.now_unix_ms,
            expires_at_unix_ms: opt.expires_at_unix_ms,
            approval_created,
            approval_approved,
            approval_expired,
            approval_used,
            approval_rejected,
            approval_audited,
            approval_reusable: false,
            one_time_approval: true,
            single_order_required: true,
            single_venue_required: true,
            single_execution_attempt_required: true,
            approval_consumed,
            approval_consumed_before_send: approval_consumed,
            approval_consumed_after_send: approval_consumed,
            audit_evidence_recorded,
            audit_event: if audit_evidence_recorded {
                format!("approval_{approval_state}_audited")
            } else {
                "not_recorded".to_string()
            },
            known_order_id: json_scalar_string_value(&cancel_risk_gate, "known_order_id")
                .unwrap_or_else(|| "missing".to_string()),
            known_client_order_id: json_scalar_string_value(
                &cancel_risk_gate,
                "known_client_order_id",
            )
            .unwrap_or_else(|| "missing".to_string()),
            symbol: json_scalar_string_value(&cancel_risk_gate, "symbol")
                .unwrap_or_else(|| "unknown".to_string()),
            account_label: json_scalar_string_value(&cancel_risk_gate, "account_label")
                .unwrap_or_else(|| "unknown".to_string()),
            venue: opt.venue.clone(),
            expected_release_tag: opt.expected_release_tag.clone(),
            release_manifest_product_version,
            release_manifest_planned_tag,
            release_manifest_actual_cancel_scope,
            actual_cancel_send_allowed: false,
            cancel_attempted: false,
            cancel_requests_sent: 0,
            network_attempted: false,
            network_cancel_endpoint_attempted: false,
            retry_attempted: false,
            replace_attempted: false,
            amend_attempted: false,
            flatten_attempted: false,
            remediation_attempted: false,
            automatic_cancel_allowed: false,
            automatic_remediation_allowed: false,
            bulk_cancel_allowed: false,
            multi_account_cancel_allowed: false,
            multi_strategy_cancel_allowed: false,
            multi_venue_cancel_allowed: false,
            production_order_submit_lifecycle_included: false,
            dashboard_order_controls_enabled: false,
            dashboard_cancel_controls_enabled: false,
            dashboard_auto_approval_allowed: false,
            api_key_value_recorded: false,
            api_secret_value_recorded: false,
            api_key_header_value_recorded: false,
            signature_recorded: false,
            signed_query_recorded: false,
            signed_url_recorded: false,
            raw_exchange_response_recorded: false,
            response_body_recorded: false,
            response_headers_recorded: false,
            safety_contract_issues,
            release_manifest_issues,
            source_artifact_issues,
            lifecycle_issues,
            missing_cli_flags: missing_cli_flags
                .iter()
                .map(|flag| (*flag).to_string())
                .collect(),
            actual_cancel_safety_contract_confirmed: opt.confirm_actual_cancel_safety_contract,
            one_order_one_venue_one_attempt_confirmed: opt
                .confirm_one_order_one_venue_one_attempt,
            single_use_approval_confirmed: opt.confirm_single_use_approval,
            approval_expiry_confirmed: opt.confirm_approval_expiry,
            bind_order_risk_gate_release_provenance_confirmed: opt
                .confirm_bind_order_risk_gate_release_provenance,
            audit_evidence_confirmed: opt.confirm_audit_evidence,
            no_dashboard_approval_confirmed: opt.confirm_no_dashboard_approval,
            no_automatic_cancel_confirmed: opt.confirm_no_automatic_cancel,
            no_bulk_cancel_confirmed: opt.confirm_no_bulk_cancel,
            no_retry_confirmed: opt.confirm_no_retry,
            no_submit_lifecycle_confirmed: opt.confirm_no_submit_lifecycle,
            no_network_confirmed: opt.confirm_no_network,
            no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
            diagnostic: if approval_execution_authorized {
                "owner approval authorizes exactly one future actual cancel attempt for the bound order, venue, risk gate, and release provenance; this artifact does not send the cancel"
            } else {
                "owner approval lifecycle is fail-closed before any cancel send because required gates, provenance, source artifacts, or lifecycle state are incomplete"
            }
            .to_string(),
        },
    )
}

fn build_production_mutation_actual_cancel_executor_adapter_boundary_artifact(
    opt: &LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt,
) -> anyhow::Result<ProductionMutationActualCancelExecutorAdapterBoundaryArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("adapter_id", &opt.adapter_id)?;
    validate_non_empty("venue", &opt.venue)?;
    validate_non_empty("order_id_type", &opt.order_id_type)?;
    validate_non_empty("expected_order_lineage_id", &opt.expected_order_lineage_id)?;
    validate_non_empty("expected_symbol", &opt.expected_symbol)?;
    validate_non_empty("expected_account_label", &opt.expected_account_label)?;

    let owner_approval_lifecycle =
        load_json_value(&opt.owner_approval_lifecycle, "owner approval lifecycle")?;
    let adapter_capability = load_json_value(&opt.adapter_capability, "adapter capability")?;

    let missing_cli_flags =
        missing_production_mutation_actual_cancel_executor_adapter_boundary_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_actual_cancel_executor_adapter_boundary_source_issues(
            &owner_approval_lifecycle,
            opt,
        );
    let adapter_capability_issues =
        production_mutation_actual_cancel_executor_adapter_boundary_capability_issues(
            &adapter_capability,
            opt,
        );
    let adapter_boundary_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && adapter_capability_issues.is_empty();
    let status = if adapter_boundary_ready {
        "adapter_boundary_ready"
    } else if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() {
        "blocked_owner_approval_lifecycle"
    } else {
        "blocked_adapter_capability"
    };
    let order_lineage_id = json_string_value(&owner_approval_lifecycle, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());

    Ok(ProductionMutationActualCancelExecutorAdapterBoundaryArtifact {
        schema_version: PRODUCTION_MUTATION_ACTUAL_CANCEL_EXECUTOR_ADAPTER_BOUNDARY_SCHEMA_VERSION
            .to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id,
        artifact_type: "actual_cancel_executor_adapter_boundary".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_shot_actual_cancel_executor_adapter_boundary".to_string(),
        capability: "Owner-Approved Single-Shot Actual Cancel Adapter Boundary".to_string(),
        execution_mode: "owner_approved_single_shot_manual_only".to_string(),
        adapter_boundary_scope: "one_order_one_venue_one_attempt".to_string(),
        default_fail_closed: true,
        owner_approval_lifecycle_ref: production_mutation_local_order_ledger_source_ref(
            &opt.owner_approval_lifecycle,
            &owner_approval_lifecycle,
            "approval_execution_authorized",
        ),
        adapter_capability_ref: production_mutation_local_order_ledger_source_ref(
            &opt.adapter_capability,
            &adapter_capability,
            "actual_cancel_supported",
        ),
        owner_approval_lifecycle_ready: source_artifact_issues.is_empty(),
        adapter_capability_ready: adapter_capability_issues.is_empty(),
        adapter_boundary_ready,
        actual_cancel_send_allowed_by_adapter_boundary: adapter_boundary_ready,
        adapter_id: opt.adapter_id.clone(),
        venue: opt.venue.clone(),
        order_id_type: opt.order_id_type.clone(),
        known_order_id: json_scalar_string_value(&owner_approval_lifecycle, "known_order_id")
            .unwrap_or_else(|| "missing".to_string()),
        known_client_order_id: json_scalar_string_value(
            &owner_approval_lifecycle,
            "known_client_order_id",
        )
        .unwrap_or_else(|| "missing".to_string()),
        symbol: json_scalar_string_value(&owner_approval_lifecycle, "symbol")
            .unwrap_or_else(|| "unknown".to_string()),
        account_label: json_scalar_string_value(&owner_approval_lifecycle, "account_label")
            .unwrap_or_else(|| "unknown".to_string()),
        cancel_request_contract: "single_order_cancel_request_v1".to_string(),
        cancel_response_contract: "single_order_cancel_response_metadata_v1".to_string(),
        post_cancel_readback_contract: "single_order_post_cancel_readback_required_v1"
            .to_string(),
        audit_contract: "single_order_cancel_audit_event_required_v1".to_string(),
        adapter_failure_taxonomy: vec![
            "rejected".to_string(),
            "timeout".to_string(),
            "unknown".to_string(),
            "already_cancelled".to_string(),
            "venue_unavailable".to_string(),
            "transport_failure".to_string(),
        ],
        max_cancel_requests: 1,
        allowed_attempts: 1,
        allowed_order_count: 1,
        allowed_venue_count: 1,
        request_contract_ready: adapter_boundary_ready,
        response_contract_ready: adapter_boundary_ready,
        readback_contract_ready: adapter_boundary_ready,
        audit_contract_ready: adapter_boundary_ready,
        actual_cancel_send_allowed: false,
        cancel_attempted: false,
        cancel_requests_sent: 0,
        network_attempted: false,
        network_cancel_endpoint_attempted: false,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        bulk_cancel_allowed: false,
        cancel_all_allowed: false,
        multi_account_cancel_allowed: false,
        multi_strategy_cancel_allowed: false,
        multi_venue_cancel_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        dashboard_execution_allowed: false,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        source_artifact_issues,
        adapter_capability_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        adapter_capability_confirmed: opt.confirm_adapter_capability,
        request_response_readback_audit_contract_confirmed: opt
            .confirm_request_response_readback_audit_contract,
        one_order_one_venue_one_attempt_confirmed: opt.confirm_one_order_one_venue_one_attempt,
        fail_closed_unsupported_capability_confirmed: opt
            .confirm_fail_closed_unsupported_capability,
        no_bulk_cancel_confirmed: opt.confirm_no_bulk_cancel,
        no_retry_confirmed: opt.confirm_no_retry,
        no_automatic_cancel_confirmed: opt.confirm_no_automatic_cancel,
        no_dashboard_execution_confirmed: opt.confirm_no_dashboard_execution,
        no_network_confirmed: opt.confirm_no_network,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        diagnostic: if adapter_boundary_ready {
            "adapter boundary is ready for one future owner-approved single-order cancel attempt; this artifact does not send the cancel"
        } else {
            "adapter boundary is fail-closed before any cancel send because owner approval, adapter capability, scope, or CLI confirmations are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_actual_cancel_single_shot_artifact(
    opt: &LiveProductionMutationActualCancelSingleShotOpt,
) -> anyhow::Result<ProductionMutationActualCancelSingleShotArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("expected_order_lineage_id", &opt.expected_order_lineage_id)?;
    validate_non_empty("expected_symbol", &opt.expected_symbol)?;
    validate_non_empty("expected_account_label", &opt.expected_account_label)?;
    validate_non_empty("venue", &opt.venue)?;
    validate_non_empty("order_id_type", &opt.order_id_type)?;
    validate_non_empty("expected_release_tag", &opt.expected_release_tag)?;
    if !matches!(
        opt.order_id_type.as_str(),
        "exchange_order_id" | "client_order_id"
    ) {
        anyhow::bail!("order_id_type must be exchange_order_id or client_order_id");
    }
    if opt.timestamp_ms == 0 {
        anyhow::bail!("actual cancel single-shot timestamp_ms must be positive");
    }
    if opt.recv_window_ms == 0 {
        anyhow::bail!("actual cancel single-shot recvWindow must be positive");
    }

    let safety_contract_raw =
        fs::read_to_string(&opt.actual_cancel_safety_contract).with_context(|| {
            format!(
                "failed to read actual cancel safety contract '{}'",
                opt.actual_cancel_safety_contract.display()
            )
        })?;
    let release_manifest = load_json_value(&opt.release_manifest, "release manifest")?;
    let cancel_risk_gate = load_json_value(&opt.cancel_risk_gate, "cancel risk gate")?;
    let owner_approval_lifecycle =
        load_json_value(&opt.owner_approval_lifecycle, "owner approval lifecycle")?;
    let adapter_boundary = load_json_value(&opt.adapter_boundary, "adapter boundary")?;
    let adapter_capability = load_json_value(&opt.adapter_capability, "adapter capability")?;

    let missing_cli_flags = missing_production_mutation_actual_cancel_single_shot_cli_flags(opt);
    let missing_env_vars = Vec::new();
    let safety_contract_issues =
        actual_cancel_owner_approval_safety_contract_issues(&safety_contract_raw);
    let release_manifest_issues =
        actual_cancel_single_shot_release_manifest_issues(&release_manifest, opt);
    let mut source_artifact_issues = production_mutation_actual_cancel_single_shot_source_issues(
        &cancel_risk_gate,
        &owner_approval_lifecycle,
        &adapter_boundary,
        opt,
    );
    let adapter_capability_issues =
        production_mutation_actual_cancel_single_shot_adapter_capability_issues(
            &adapter_capability,
            opt,
        );
    let (cancel_order_identifier_ref, raw_order_identifier_issue, _raw_order_identifier) =
        actual_cancel_single_shot_order_identifier(opt, &owner_approval_lifecycle);
    if let Some(issue) = raw_order_identifier_issue {
        source_artifact_issues.push(issue);
    }

    let actual_cancel_command_ready = missing_cli_flags.is_empty()
        && safety_contract_issues.is_empty()
        && release_manifest_issues.is_empty()
        && source_artifact_issues.is_empty()
        && adapter_capability_issues.is_empty();
    let single_shot_cancel_allowed = false;
    let request_id = format!("actual-cancel:{}:{}", opt.run_id, opt.timestamp_ms);
    let approval_state_before_attempt =
        json_string_value(&owner_approval_lifecycle, "approval_state")
            .unwrap_or_else(|| "unknown".to_string());
    let owner_approval_authorized_before_attempt =
        json_bool_value(&owner_approval_lifecycle, "approval_execution_authorized")
            .unwrap_or(false);
    let request_sent = false;
    let cancel_attempted = false;
    let network_attempted = false;
    let http_send_attempted = false;
    let venue_ack_observed = false;
    let status = if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !safety_contract_issues.is_empty() {
        "blocked_safety_contract"
    } else if !release_manifest_issues.is_empty() {
        "blocked_release_provenance"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
    } else if !adapter_capability_issues.is_empty() {
        "blocked_adapter_capability"
    } else {
        "ready_actual_cancel_command_offline_no_send"
    };
    let approval_state_after_attempt = approval_state_before_attempt.clone();
    let readback_required = false;
    let local_audit_reference = format!(
        "actual_cancel_audit:{}:{}:{}",
        opt.run_id, request_id, status
    );
    let order_lineage_id = json_string_value(&owner_approval_lifecycle, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_order_id = json_scalar_string_value(&owner_approval_lifecycle, "known_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id =
        json_scalar_string_value(&owner_approval_lifecycle, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string());

    Ok(ProductionMutationActualCancelSingleShotArtifact {
        schema_version: PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id,
        artifact_type: "actual_cancel_single_shot".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "retired_actual_cancel_offline_evaluation".to_string(),
        capability: "Historical Actual Cancel Artifact Evaluation".to_string(),
        execution_mode: "offline_only_executor_retired".to_string(),
        default_fail_closed: true,
        actual_cancel_safety_contract_ref: production_mutation_source_file_ref(
            &opt.actual_cancel_safety_contract,
        ),
        release_manifest_ref: production_mutation_source_file_ref(&opt.release_manifest),
        cancel_risk_gate_ref: production_mutation_local_order_ledger_source_ref(
            &opt.cancel_risk_gate,
            &cancel_risk_gate,
            "cancel_risk_gate_ready",
        ),
        owner_approval_lifecycle_ref: production_mutation_local_order_ledger_source_ref(
            &opt.owner_approval_lifecycle,
            &owner_approval_lifecycle,
            if cancel_attempted {
                "approval_consumed"
            } else {
                "approval_execution_authorized"
            },
        ),
        adapter_boundary_ref: production_mutation_local_order_ledger_source_ref(
            &opt.adapter_boundary,
            &adapter_boundary,
            "adapter_boundary_ready",
        ),
        adapter_capability_ref: production_mutation_local_order_ledger_source_ref(
            &opt.adapter_capability,
            &adapter_capability,
            "actual_cancel_supported",
        ),
        manual_online_requested: false,
        actual_cancel_command_ready,
        single_shot_cancel_allowed,
        owner_approval_ready: source_artifact_issues.is_empty()
            && owner_approval_authorized_before_attempt,
        risk_gate_ready: json_bool_value(&cancel_risk_gate, "cancel_risk_gate_ready")
            .unwrap_or(false),
        release_provenance_ready: release_manifest_issues.is_empty(),
        adapter_boundary_ready: json_bool_value(&adapter_boundary, "adapter_boundary_ready")
            .unwrap_or(false),
        adapter_capability_ready: adapter_capability_issues.is_empty(),
        approval_consumed_before_send: cancel_attempted,
        approval_consumed_after_send: cancel_attempted,
        approval_state_before_attempt,
        approval_state_after_attempt,
        request_id,
        request_method: TESTNET_ORDER_METHOD_DELETE.to_string(),
        request_target: TESTNET_ORDER_ENDPOINT_ORDER.to_string(),
        request_contract: "single_order_cancel_request_v1".to_string(),
        adapter_id: json_string_value(&adapter_boundary, "adapter_id")
            .unwrap_or_else(|| "unknown".to_string()),
        venue: opt.venue.clone(),
        order_id_type: opt.order_id_type.clone(),
        known_order_id,
        known_client_order_id,
        cancel_order_identifier_ref,
        symbol: opt.expected_symbol.clone(),
        account_label: opt.expected_account_label.clone(),
        recv_window_ms: opt.recv_window_ms,
        timestamp_recorded: false,
        timestamp_shape: "epoch_millis_present_redacted".to_string(),
        credential_material: "retired_not_read".to_string(),
        production_signing_material_gate_required: false,
        production_signing_material_gate_open: false,
        production_signing_material_env_read: false,
        production_signing_material_missing_gate_env_vars: vec![
            "production_mutation_executor_retired_after_v0.32.0".to_string(),
        ],
        api_key_env: "retired".to_string(),
        api_secret_env: "retired".to_string(),
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
        response_redacted: true,
        venue_response_status: "not_attempted".to_string(),
        venue_response_source: "executor_retired_offline".to_string(),
        venue_response_code: None,
        venue_response_error_code: "not_attempted_executor_retired".to_string(),
        latency_ms: None,
        local_audit_reference,
        readback_required,
        readback_requirement: if readback_required {
            "post_cancel_readback_required_before_any_retry_or_followup".to_string()
        } else {
            "not_required_without_send_attempt".to_string()
        },
        source_artifact_issues,
        adapter_capability_issues,
        safety_contract_issues,
        release_manifest_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        missing_env_vars,
        request_sent,
        cancel_attempted,
        cancel_requests_sent: u64::from(request_sent),
        production_order_mutations_attempted: u64::from(cancel_attempted),
        network_attempted,
        network_cancel_endpoint_attempted: network_attempted,
        http_send_attempted,
        venue_ack_observed,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        bulk_cancel_allowed: false,
        cancel_all_allowed: false,
        multi_account_cancel_allowed: false,
        multi_strategy_cancel_allowed: false,
        multi_venue_cancel_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        dashboard_execution_allowed: false,
        owner_approval_confirmed: opt.confirm_owner_approval,
        risk_gate_confirmed: opt.confirm_risk_gate,
        release_provenance_confirmed: opt.confirm_release_provenance,
        adapter_boundary_confirmed: opt.confirm_adapter_boundary,
        single_shot_confirmed: opt.confirm_single_shot,
        consume_approval_before_send_confirmed: opt.confirm_consume_approval_before_send,
        readback_required_confirmed: opt.confirm_readback_required,
        no_bulk_cancel_confirmed: opt.confirm_no_bulk_cancel,
        no_retry_confirmed: opt.confirm_no_retry,
        no_automatic_cancel_confirmed: opt.confirm_no_automatic_cancel,
        no_dashboard_execution_confirmed: opt.confirm_no_dashboard_execution,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        diagnostic: "actual cancel executor is retired; historical contracts were evaluated offline without credential reads, signing, network, approval consumption, cancel, retry, bulk cancel, automatic cancel, or Dashboard execution".to_string(),
    })
}

fn build_production_mutation_actual_cancel_readback_reconciliation_artifact(
    opt: &LiveProductionMutationActualCancelReadbackReconciliationOpt,
) -> anyhow::Result<ProductionMutationActualCancelReadbackReconciliationArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("expected_order_lineage_id", &opt.expected_order_lineage_id)?;
    validate_non_empty("expected_symbol", &opt.expected_symbol)?;
    validate_non_empty("expected_account_label", &opt.expected_account_label)?;
    validate_non_empty("venue", &opt.venue)?;

    let actual_cancel_attempt =
        load_json_value(&opt.actual_cancel_attempt, "actual cancel attempt artifact")?;
    let readback = load_json_value(&opt.readback, "actual cancel post-cancel readback metadata")?;
    let missing_cli_flags =
        missing_production_mutation_actual_cancel_readback_reconciliation_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_actual_cancel_readback_reconciliation_source_issues(
            &actual_cancel_attempt,
            opt,
        );
    let forbidden_readback_markers = production_mutation_response_forbidden_markers(&readback);
    let readback_state = production_mutation_actual_cancel_readback_reconciliation_state(&readback);
    let reconciliation = classify_production_mutation_actual_cancel_readback_reconciliation(
        &readback_state,
        &readback,
    );
    let readback_lineage_issues =
        production_mutation_actual_cancel_readback_reconciliation_lineage_issues(
            &actual_cancel_attempt,
            &readback,
            opt,
        );
    let unsupported_readback_states = if reconciliation.readback_result == "unsupported" {
        vec![readback_state.clone()]
    } else {
        Vec::new()
    };
    let reconciliation_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && forbidden_readback_markers.is_empty()
        && readback_lineage_issues.is_empty()
        && unsupported_readback_states.is_empty();
    let status = if !missing_cli_flags.is_empty() {
        "blocked_missing_gate".to_string()
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact".to_string()
    } else if !forbidden_readback_markers.is_empty() {
        "blocked_forbidden_readback_marker".to_string()
    } else if !readback_lineage_issues.is_empty() {
        "blocked_readback_lineage".to_string()
    } else if !unsupported_readback_states.is_empty() {
        "blocked_unsupported_readback_state".to_string()
    } else {
        reconciliation.artifact_status.clone()
    };

    let known_order_id = json_scalar_string_value(&actual_cancel_attempt, "known_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id =
        json_scalar_string_value(&actual_cancel_attempt, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string());
    let raw_readback_order_id =
        json_scalar_string_value(&readback, "orderId").unwrap_or_else(|| "missing".to_string());
    let raw_readback_client_order_id = json_scalar_string_value(&readback, "clientOrderId")
        .unwrap_or_else(|| "missing".to_string());
    let raw_readback_orig_client_order_id =
        json_scalar_string_value(&readback, "origClientOrderId")
            .unwrap_or_else(|| "missing".to_string());

    Ok(ProductionMutationActualCancelReadbackReconciliationArtifact {
        schema_version:
            PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id: json_string_value(&actual_cancel_attempt, "order_lineage_id")
            .unwrap_or_else(|| "missing".to_string()),
        source_actual_cancel_attempt_path: opt.actual_cancel_attempt.display().to_string(),
        source_readback_path: opt.readback.display().to_string(),
        source_actual_cancel_attempt_run_id: json_string_value(&actual_cancel_attempt, "run_id")
            .unwrap_or_else(|| "unknown".to_string()),
        source_actual_cancel_attempt_hash: file_fnv1a64_hash(
            &opt.actual_cancel_attempt.display().to_string(),
        ),
        artifact_type: "actual_cancel_readback_reconciliation".to_string(),
        status,
        created_at: now_millis(),
        mode: "owner_approved_single_shot_actual_cancel_readback_reconciliation".to_string(),
        capability: "Owner-Approved Single-Shot Actual Cancel Readback Reconciliation"
            .to_string(),
        execution_mode: "post_actual_cancel_readback_evidence_only".to_string(),
        lineage_scope: "single_actual_cancel_attempt".to_string(),
        default_fail_closed: true,
        actual_cancel_attempt_ref: production_mutation_local_order_ledger_source_ref(
            &opt.actual_cancel_attempt,
            &actual_cancel_attempt,
            "readback_required",
        ),
        actual_cancel_attempt_ready: source_artifact_issues.is_empty(),
        actual_cancel_attempt_recorded: json_bool_value(
            &actual_cancel_attempt,
            "cancel_attempted",
        )
        .unwrap_or(false),
        actual_cancel_request_sent: json_bool_value(&actual_cancel_attempt, "request_sent")
            .unwrap_or(false),
        actual_cancel_request_id: json_string_value(&actual_cancel_attempt, "request_id")
            .unwrap_or_else(|| "unknown".to_string()),
        readback_required: json_bool_value(&actual_cancel_attempt, "readback_required")
            .unwrap_or(false),
        readback_evidence_present: reconciliation_ready,
        reconciliation_evidence_present: reconciliation_ready,
        reconciliation_ready,
        readback_reconciliation_complete: reconciliation_ready
            && reconciliation.readback_reconciliation_complete,
        actual_cancel_followup_complete: reconciliation_ready
            && reconciliation.actual_cancel_followup_complete,
        redacted_metadata_only: reconciliation_ready,
        venue: opt.venue.clone(),
        symbol: json_scalar_string_value(&readback, "symbol")
            .or_else(|| json_scalar_string_value(&actual_cancel_attempt, "symbol"))
            .unwrap_or_else(|| "unknown".to_string()),
        account_label: json_scalar_string_value(&actual_cancel_attempt, "account_label")
            .unwrap_or_else(|| "unknown".to_string()),
        known_order_id,
        known_client_order_id,
        readback_type: "binance_actual_cancel_post_readback_reconciliation_metadata_v1"
            .to_string(),
        readback_state,
        readback_result: reconciliation.readback_result,
        reconciliation_status: reconciliation.reconciliation_status,
        venue_state: reconciliation.venue_state,
        order_status: reconciliation.order_status,
        execution_fill_status: reconciliation.execution_fill_status,
        remaining_quantity_state: reconciliation.remaining_quantity_state,
        residual_risk_state: reconciliation.residual_risk_state,
        local_audit_state: reconciliation.local_audit_state,
        readback_order_id: redact_cancel_preview_identifier(
            "readback_order_id",
            &raw_readback_order_id,
        ),
        readback_client_order_id: redact_cancel_preview_identifier(
            "readback_client_order_id",
            &raw_readback_client_order_id,
        ),
        readback_orig_client_order_id: redact_cancel_preview_identifier(
            "readback_orig_client_order_id",
            &raw_readback_orig_client_order_id,
        ),
        readback_update_time_shape: production_mutation_response_time_shape(&readback, "updateTime"),
        partial_fill_observed: reconciliation.partial_fill_observed,
        already_cancelled_observed: reconciliation.already_cancelled_observed,
        filled_before_cancel_observed: reconciliation.filled_before_cancel_observed,
        timeout_observed: reconciliation.timeout_observed,
        unknown_observed: reconciliation.unknown_observed,
        inconsistent_observed: reconciliation.inconsistent_observed,
        degraded: reconciliation.degraded,
        error_state: reconciliation.error_state,
        terminal_state_observed: reconciliation.terminal_state_observed,
        manual_review_required: reconciliation.manual_review_required,
        new_orders_blocked: reconciliation.new_orders_blocked,
        risk_halted: reconciliation.risk_halted,
        dashboard_read_only_consumable: reconciliation_ready,
        dashboard_audit_view_ready: reconciliation_ready,
        allowed_readback_fields:
            production_mutation_actual_cancel_readback_reconciliation_allowed_fields(),
        forbidden_readback_markers,
        source_artifact_issues,
        readback_lineage_issues,
        unsupported_readback_states,
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
        raw_exchange_response_recorded: false,
        raw_readback_body_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        unrestricted_payload_recorded: false,
        account_balances_recorded: false,
        fills_recorded: false,
        readback_execution_attempted: false,
        order_state_read_attempted: false,
        production_order_state_reads_attempted: 0,
        actual_cancel_send_allowed: false,
        cancel_attempted: false,
        cancel_requests_sent: 0,
        production_order_mutations_attempted: 0,
        network_attempted: false,
        network_readback_endpoint_attempted: false,
        network_cancel_endpoint_attempted: false,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        second_cancel_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        production_order_mutation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        actual_cancel_attempt_recorded_confirmed: opt.confirm_actual_cancel_attempt_recorded,
        readback_required_confirmed: opt.confirm_readback_required,
        readback_metadata_only_confirmed: opt.confirm_readback_metadata_only,
        order_status_reconciled_confirmed: opt.confirm_order_status_reconciled,
        execution_fill_status_reconciled_confirmed: opt
            .confirm_execution_fill_status_reconciled,
        remaining_quantity_reconciled_confirmed: opt.confirm_remaining_quantity_reconciled,
        risk_state_recorded_confirmed: opt.confirm_risk_state_recorded,
        local_audit_state_recorded_confirmed: opt.confirm_local_audit_state_recorded,
        dashboard_read_only_consumable_confirmed: opt.confirm_dashboard_read_only_consumable,
        no_raw_readback_persistence_confirmed: opt.confirm_no_raw_readback_persistence,
        no_headers_persistence_confirmed: opt.confirm_no_headers_persistence,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        no_retry_confirmed: opt.confirm_no_retry,
        no_remediation_confirmed: opt.confirm_no_remediation,
        no_second_cancel_confirmed: opt.confirm_no_second_cancel,
        no_network_confirmed: opt.confirm_no_network,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        diagnostic: if reconciliation_ready {
            "actual cancel attempt has post-cancel readback reconciliation evidence; degraded readback outcomes remain explicit and do not enable retry, remediation, second cancel, network readback, raw persistence, or Dashboard cancel controls"
        } else {
            "actual cancel readback reconciliation blocked because the actual cancel attempt, readback metadata, lineage, or manual confirmations are incomplete"
        }
        .to_string(),
    })
}

fn build_production_mutation_actual_cancel_failure_evidence_artifact(
    opt: &LiveProductionMutationActualCancelFailureEvidenceOpt,
) -> anyhow::Result<ProductionMutationActualCancelFailureEvidenceArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;
    validate_non_empty("expected_order_lineage_id", &opt.expected_order_lineage_id)?;
    validate_non_empty("expected_symbol", &opt.expected_symbol)?;
    validate_non_empty("expected_account_label", &opt.expected_account_label)?;
    validate_non_empty("venue", &opt.venue)?;

    let reconciliation = load_json_value(
        &opt.readback_reconciliation,
        "actual cancel readback reconciliation artifact",
    )?;
    let request_ref = load_json_value(&opt.request_ref, "actual cancel request reference")?;
    let response_ref = load_json_value(&opt.response_ref, "actual cancel response reference")?;
    let readback_ref = load_json_value(&opt.readback_ref, "actual cancel readback reference")?;
    let audit_ref = load_json_value(&opt.audit_ref, "actual cancel audit reference")?;
    let missing_cli_flags =
        missing_production_mutation_actual_cancel_failure_evidence_cli_flags(opt);
    let source_artifact_issues = production_mutation_actual_cancel_failure_evidence_source_issues(
        &reconciliation,
        &request_ref,
        &response_ref,
        &readback_ref,
        &audit_ref,
    );
    let lineage_issues =
        production_mutation_actual_cancel_failure_evidence_lineage_issues(&reconciliation, opt);
    let decision = classify_production_mutation_actual_cancel_failure_evidence(
        &reconciliation,
        &request_ref,
        &response_ref,
        &readback_ref,
        &audit_ref,
    );
    let evidence_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && lineage_issues.is_empty();
    let status = if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
    } else if !lineage_issues.is_empty() {
        "blocked_lineage"
    } else {
        decision.artifact_status
    };
    let request_ref_recorded = true;
    let response_ref_recorded = true;
    let readback_ref_recorded = true;
    let audit_ref_recorded = true;
    let refs_recorded = request_ref_recorded
        && response_ref_recorded
        && readback_ref_recorded
        && audit_ref_recorded;
    let dashboard_consumable = evidence_ready;
    let release_gate_consumable = evidence_ready;
    let source_residual_risk_state = json_string_value(&reconciliation, "residual_risk_state")
        .unwrap_or_else(|| "unknown".to_string());

    Ok(
        ProductionMutationActualCancelFailureEvidenceArtifact {
            schema_version:
                PRODUCTION_MUTATION_ACTUAL_CANCEL_FAILURE_EVIDENCE_SCHEMA_VERSION.to_string(),
            run_id: opt.run_id.clone(),
            order_lineage_id: json_string_value(&reconciliation, "order_lineage_id")
                .unwrap_or_else(|| "missing".to_string()),
            source_readback_reconciliation_path: opt.readback_reconciliation.display().to_string(),
            source_request_ref_path: opt.request_ref.display().to_string(),
            source_response_ref_path: opt.response_ref.display().to_string(),
            source_readback_ref_path: opt.readback_ref.display().to_string(),
            source_audit_ref_path: opt.audit_ref.display().to_string(),
            source_readback_reconciliation_run_id: json_string_value(&reconciliation, "run_id")
                .unwrap_or_else(|| "unknown".to_string()),
            source_readback_reconciliation_hash: file_fnv1a64_hash(
                &opt.readback_reconciliation.display().to_string(),
            ),
            artifact_type: "actual_cancel_failure_evidence".to_string(),
            status: status.to_string(),
            created_at: now_millis(),
            mode: "owner_approved_single_shot_actual_cancel_failure_partial_success_evidence"
                .to_string(),
            capability:
                "Owner-Approved Single-Shot Actual Cancel Failure and Partial-Success Evidence"
                    .to_string(),
            execution_mode: "post_actual_cancel_evidence_only".to_string(),
            lineage_scope: "single_actual_cancel_attempt".to_string(),
            default_fail_closed: true,
            readback_reconciliation_ref: production_mutation_local_order_ledger_source_ref(
                &opt.readback_reconciliation,
                &reconciliation,
                "reconciliation_ready",
            ),
            request_ref: production_mutation_local_order_ledger_source_ref(
                &opt.request_ref,
                &request_ref,
                "ready",
            ),
            response_ref: production_mutation_local_order_ledger_source_ref(
                &opt.response_ref,
                &response_ref,
                "ready",
            ),
            readback_ref: production_mutation_local_order_ledger_source_ref(
                &opt.readback_ref,
                &readback_ref,
                "ready",
            ),
            audit_ref: production_mutation_local_order_ledger_source_ref(
                &opt.audit_ref,
                &audit_ref,
                "ready",
            ),
            request_ref_recorded,
            response_ref_recorded,
            readback_ref_recorded,
            audit_ref_recorded,
            references_ready: refs_recorded && source_artifact_issues.is_empty(),
            evidence_ready,
            failure_evidence_ready: evidence_ready,
            dashboard_read_only_consumable: dashboard_consumable,
            release_gate_consumable,
            venue: json_string_value(&reconciliation, "venue")
                .unwrap_or_else(|| "unknown".to_string()),
            symbol: json_scalar_string_value(&reconciliation, "symbol")
                .unwrap_or_else(|| "unknown".to_string()),
            account_label: json_scalar_string_value(&reconciliation, "account_label")
                .unwrap_or_else(|| "unknown".to_string()),
            readback_result: json_string_value(&reconciliation, "readback_result")
                .unwrap_or_else(|| "unknown".to_string()),
            reconciliation_status: json_string_value(&reconciliation, "reconciliation_status")
                .unwrap_or_else(|| "unknown".to_string()),
            source_readback_state: json_string_value(&reconciliation, "readback_state")
                .unwrap_or_else(|| "unknown".to_string()),
            source_venue_state: json_string_value(&reconciliation, "venue_state")
                .unwrap_or_else(|| "unknown".to_string()),
            source_order_status: json_string_value(&reconciliation, "order_status")
                .unwrap_or_else(|| "unknown".to_string()),
            source_execution_fill_status: json_string_value(
                &reconciliation,
                "execution_fill_status",
            )
            .unwrap_or_else(|| "unknown".to_string()),
            source_remaining_quantity_state: json_string_value(
                &reconciliation,
                "remaining_quantity_state",
            )
            .unwrap_or_else(|| "unknown".to_string()),
            source_residual_risk_state: source_residual_risk_state.clone(),
            source_local_audit_state: json_string_value(&reconciliation, "local_audit_state")
                .unwrap_or_else(|| "unknown".to_string()),
            cancel_outcome: decision.cancel_outcome.to_string(),
            outcome_category: decision.outcome_category.to_string(),
            failure_mode: decision.failure_mode.to_string(),
            partial_success_mode: decision.partial_success_mode.to_string(),
            operator_action: decision.operator_action.to_string(),
            operator_action_required: decision.operator_action_required,
            recovered: decision.recovered,
            degraded: decision.degraded,
            failed: decision.failed,
            partial_success: decision.partial_success,
            residual_risk_visible: decision.residual_risk_visible,
            residual_risk_state: if decision.residual_risk_state == "source" {
                source_residual_risk_state
            } else {
                decision.residual_risk_state.to_string()
            },
            manual_review_required: decision.manual_review_required,
            new_orders_blocked: decision.new_orders_blocked,
            risk_halted: decision.risk_halted,
            outcome_cancel_confirmed: decision.cancel_outcome == "cancel_confirmed",
            outcome_already_cancelled: decision.cancel_outcome == "already_cancelled",
            outcome_rejected: decision.cancel_outcome == "rejected",
            outcome_timeout: decision.cancel_outcome == "timeout",
            outcome_unknown: decision.cancel_outcome == "unknown",
            outcome_partial_fill: decision.cancel_outcome == "partial_fill",
            outcome_filled_before_cancel: decision.cancel_outcome == "filled_before_cancel",
            outcome_venue_unavailable: decision.cancel_outcome == "venue_unavailable",
            outcome_adapter_failure: decision.cancel_outcome == "adapter_failure",
            outcome_inconsistent: decision.cancel_outcome == "inconsistent",
            outcome_failed: decision.failed,
            actual_cancel_followup_complete: evidence_ready && decision.recovered,
            unknown_not_recovered: !(decision.cancel_outcome == "unknown" && decision.recovered),
            partial_fill_residual_risk_visible: decision.cancel_outcome != "partial_fill"
                || decision.residual_risk_visible,
            request_response_readback_audit_refs_recorded: refs_recorded,
            source_artifact_issues,
            lineage_issues,
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
            raw_exchange_response_recorded: false,
            raw_readback_body_recorded: false,
            response_body_recorded: false,
            response_headers_recorded: false,
            unrestricted_payload_recorded: false,
            account_balances_recorded: false,
            fills_recorded: false,
            readback_execution_attempted: false,
            order_state_read_attempted: false,
            production_order_state_reads_attempted: 0,
            actual_cancel_send_allowed: false,
            cancel_attempted: false,
            cancel_requests_sent: 0,
            production_order_mutations_attempted: 0,
            network_attempted: false,
            network_readback_endpoint_attempted: false,
            network_cancel_endpoint_attempted: false,
            retry_attempted: false,
            replace_attempted: false,
            amend_attempted: false,
            flatten_attempted: false,
            remediation_attempted: false,
            compensation_trade_attempted: false,
            second_cancel_attempted: false,
            automatic_cancel_allowed: false,
            automatic_remediation_allowed: false,
            production_order_mutation_allowed: false,
            dashboard_order_controls_enabled: false,
            dashboard_cancel_controls_enabled: false,
            request_ref_recorded_confirmed: opt.confirm_request_ref_recorded,
            response_ref_recorded_confirmed: opt.confirm_response_ref_recorded,
            readback_ref_recorded_confirmed: opt.confirm_readback_ref_recorded,
            audit_ref_recorded_confirmed: opt.confirm_audit_ref_recorded,
            failure_outcomes_classified_confirmed: opt.confirm_failure_outcomes_classified,
            operator_action_model_confirmed: opt.confirm_operator_action_model,
            unknown_not_recovered_confirmed: opt.confirm_unknown_not_recovered,
            partial_fill_residual_risk_confirmed: opt.confirm_partial_fill_residual_risk,
            dashboard_release_gate_consumable_confirmed: opt
                .confirm_dashboard_release_gate_consumable,
            no_retry_confirmed: opt.confirm_no_retry,
            no_remediation_confirmed: opt.confirm_no_remediation,
            no_compensation_trade_confirmed: opt.confirm_no_compensation_trade,
            no_network_confirmed: opt.confirm_no_network,
            dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
            no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
            diagnostic: if evidence_ready {
                "actual cancel outcome evidence classifies failure and partial-success paths from redacted request, response, readback, and audit references; no retry, remediation, compensation trade, network call, second cancel, raw persistence, or Dashboard order control is enabled"
            } else {
                "actual cancel outcome evidence blocked because source references, lineage, or manual confirmations are incomplete"
            }
            .to_string(),
        },
    )
}

fn build_production_mutation_cancel_response_redaction_artifact(
    opt: &LiveProductionMutationCancelResponseRedactionOpt,
) -> anyhow::Result<ProductionMutationCancelResponseRedactionArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let approval_lifecycle = load_json_value(
        &opt.manual_owner_approval_lifecycle,
        "manual owner approval lifecycle",
    )?;
    let response = load_json_value(&opt.response, "production mutation cancel response")?;
    let missing_cli_flags = missing_production_mutation_cancel_response_redaction_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_cancel_response_redaction_source_issues(&approval_lifecycle);
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
        "ready_cancel_response_redacted"
    };

    let order_lineage_id = json_string_value(&approval_lifecycle, "order_lineage_id")
        .unwrap_or_else(|| "missing".into());
    let known_order_id = json_scalar_string_value(&approval_lifecycle, "known_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id =
        json_scalar_string_value(&approval_lifecycle, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string());
    let raw_cancel_order_id =
        json_scalar_string_value(&response, "orderId").unwrap_or_else(|| "missing".to_string());
    let raw_cancel_client_order_id = json_scalar_string_value(&response, "clientOrderId")
        .unwrap_or_else(|| "missing".to_string());
    let raw_orig_client_order_id = json_scalar_string_value(&response, "origClientOrderId")
        .unwrap_or_else(|| "missing".to_string());

    Ok(ProductionMutationCancelResponseRedactionArtifact {
        schema_version: PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id,
        source_manual_owner_approval_lifecycle_path: opt
            .manual_owner_approval_lifecycle
            .display()
            .to_string(),
        source_response_path: opt.response.display().to_string(),
        source_manual_owner_approval_lifecycle_run_id: json_string_value(
            &approval_lifecycle,
            "run_id",
        )
        .unwrap_or_else(|| "unknown".to_string()),
        source_manual_owner_approval_lifecycle_hash: file_fnv1a64_hash(
            &opt.manual_owner_approval_lifecycle.display().to_string(),
        ),
        artifact_type: "cancel_response_redaction".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_cancel_response_redaction_contract".to_string(),
        capability: "Owner-Approved Cancel Recovery Preview".to_string(),
        capability_expansion: "future_cancel_response_redaction_only".to_string(),
        lineage_scope: json_string_value(&approval_lifecycle, "lineage_scope")
            .unwrap_or_else(|| "unknown".to_string()),
        default_fail_closed: true,
        manual_owner_approval_lifecycle_ref: production_mutation_local_order_ledger_source_ref(
            &opt.manual_owner_approval_lifecycle,
            &approval_lifecycle,
            "approval_lifecycle_valid",
        ),
        approval_lifecycle_valid: json_bool_value(&approval_lifecycle, "approval_lifecycle_valid")
            .unwrap_or(false),
        approval_state: json_string_value(&approval_lifecycle, "approval_state")
            .unwrap_or_else(|| "unknown".to_string()),
        manual_approval_recorded: json_bool_value(
            &approval_lifecycle,
            "manual_approval_recorded",
        )
        .unwrap_or(false),
        approval_consumed: json_bool_value(&approval_lifecycle, "approval_consumed")
            .unwrap_or(false),
        response_redaction_ready,
        cancel_response_redacted: true,
        response_shape_validated: response_redaction_ready,
        response_type: "binance_cancel_response_redacted_metadata_v1".to_string(),
        known_order_id,
        known_client_order_id,
        cancel_order_id: redact_cancel_preview_identifier("cancel_order_id", &raw_cancel_order_id),
        cancel_client_order_id: redact_cancel_preview_identifier(
            "cancel_client_order_id",
            &raw_cancel_client_order_id,
        ),
        orig_client_order_id: redact_cancel_preview_identifier(
            "orig_client_order_id",
            &raw_orig_client_order_id,
        ),
        symbol: json_scalar_string_value(&response, "symbol")
            .or_else(|| json_scalar_string_value(&approval_lifecycle, "symbol"))
            .unwrap_or_else(|| "unknown".to_string()),
        account_label: json_scalar_string_value(&approval_lifecycle, "account_label")
            .unwrap_or_else(|| "unknown".to_string()),
        exchange_status: json_scalar_string_value(&response, "status")
            .unwrap_or_else(|| "unknown".to_string()),
        transact_time_shape: production_mutation_response_time_shape(&response, "transactTime"),
        allowed_response_fields: production_mutation_cancel_response_allowed_fields(),
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
        actual_cancel_send_allowed: false,
        cancel_attempted: false,
        cancel_requests_sent: 0,
        network_attempted: false,
        network_cancel_endpoint_attempted: false,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        production_order_mutation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        manual_owner_approval_lifecycle_ready_confirmed: opt
            .confirm_manual_owner_approval_lifecycle_ready,
        no_raw_response_persistence_confirmed: opt.confirm_no_raw_response_persistence,
        no_headers_persistence_confirmed: opt.confirm_no_headers_persistence,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        cancel_metadata_only_confirmed: opt.confirm_cancel_metadata_only,
        no_account_balances_confirmed: opt.confirm_no_account_balances,
        no_unrestricted_payload_confirmed: opt.confirm_no_unrestricted_payload,
        no_retry_confirmed: opt.confirm_no_retry,
        no_cancel_confirmed: opt.confirm_no_cancel,
        no_network_confirmed: opt.confirm_no_network,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        diagnostic: if response_redaction_ready {
            "future cancel response was reduced to allowed cancel metadata only; raw body, headers, secrets, balances, fills, unrestricted payload, cancel send, network, retry, and Dashboard cancel controls were not persisted or enabled"
        } else {
            "future cancel response redaction contract blocked before persisting unrestricted response material"
        }
        .to_string(),
    })
}

fn build_production_mutation_post_cancel_readback_artifact(
    opt: &LiveProductionMutationPostCancelReadbackOpt,
) -> anyhow::Result<ProductionMutationPostCancelReadbackArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let cancel_response_redaction = load_json_value(
        &opt.cancel_response_redaction,
        "cancel response redaction artifact",
    )?;
    let readback = load_json_value(&opt.readback, "post-cancel readback metadata")?;
    let missing_cli_flags = missing_production_mutation_post_cancel_readback_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_post_cancel_readback_source_issues(&cancel_response_redaction);
    let forbidden_readback_markers = production_mutation_response_forbidden_markers(&readback);
    let readback_state = production_mutation_post_cancel_readback_state(&readback);
    let (readback_state_class, readback_outcome, terminal_state_observed, ambiguous_state_observed) =
        production_mutation_post_cancel_readback_classification(&readback_state);
    let unsupported_readback_states = if readback_state_class == "unsupported" {
        vec![readback_state.clone()]
    } else {
        Vec::new()
    };
    let post_cancel_readback_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && forbidden_readback_markers.is_empty()
        && unsupported_readback_states.is_empty();
    let status = if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
    } else if !forbidden_readback_markers.is_empty() {
        "blocked_forbidden_readback_marker"
    } else if !unsupported_readback_states.is_empty() {
        "blocked_unsupported_readback_state"
    } else {
        "ready_post_cancel_readback_classified"
    };

    let order_lineage_id = json_string_value(&cancel_response_redaction, "order_lineage_id")
        .unwrap_or_else(|| "missing".into());
    let known_order_id = json_scalar_string_value(&cancel_response_redaction, "known_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id =
        json_scalar_string_value(&cancel_response_redaction, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string());
    let raw_readback_order_id =
        json_scalar_string_value(&readback, "orderId").unwrap_or_else(|| "missing".to_string());
    let raw_readback_client_order_id = json_scalar_string_value(&readback, "clientOrderId")
        .unwrap_or_else(|| "missing".to_string());
    let raw_readback_orig_client_order_id =
        json_scalar_string_value(&readback, "origClientOrderId")
            .unwrap_or_else(|| "missing".to_string());
    let symbol = json_scalar_string_value(&readback, "symbol")
        .or_else(|| json_scalar_string_value(&cancel_response_redaction, "symbol"))
        .unwrap_or_else(|| "unknown".to_string());
    let account_label = json_scalar_string_value(&cancel_response_redaction, "account_label")
        .unwrap_or_else(|| "unknown".to_string());
    let order_found = readback_state != "MISSING";
    let order_lineage_preserved =
        post_cancel_readback_ready && cancel_preview_identifier_known(&order_lineage_id);

    Ok(ProductionMutationPostCancelReadbackArtifact {
        schema_version: PRODUCTION_MUTATION_POST_CANCEL_READBACK_SCHEMA_VERSION.to_string(),
        run_id: opt.run_id.clone(),
        order_lineage_id,
        source_cancel_response_redaction_path: opt.cancel_response_redaction.display().to_string(),
        source_readback_path: opt.readback.display().to_string(),
        source_cancel_response_redaction_run_id: json_string_value(
            &cancel_response_redaction,
            "run_id",
        )
        .unwrap_or_else(|| "unknown".to_string()),
        source_cancel_response_redaction_hash: file_fnv1a64_hash(
            &opt.cancel_response_redaction.display().to_string(),
        ),
        artifact_type: "post_cancel_readback".to_string(),
        status: status.to_string(),
        created_at: now_millis(),
        mode: "single_mutation_candidate_post_cancel_readback_contract".to_string(),
        capability: "Owner-Approved Cancel Recovery Preview".to_string(),
        capability_expansion: "future_post_cancel_readback_only".to_string(),
        lineage_scope: json_string_value(&cancel_response_redaction, "lineage_scope")
            .unwrap_or_else(|| "unknown".to_string()),
        default_fail_closed: true,
        cancel_response_redaction_ref: production_mutation_local_order_ledger_source_ref(
            &opt.cancel_response_redaction,
            &cancel_response_redaction,
            "response_redaction_ready",
        ),
        cancel_response_redaction_ready: json_bool_value(
            &cancel_response_redaction,
            "response_redaction_ready",
        )
        .unwrap_or(false),
        cancel_response_redacted: json_bool_value(
            &cancel_response_redaction,
            "cancel_response_redacted",
        )
        .unwrap_or(false),
        post_cancel_readback_ready,
        post_cancel_readback_classified: post_cancel_readback_ready,
        redacted_metadata_only: post_cancel_readback_ready,
        readback_type: "binance_post_cancel_order_readback_redacted_metadata_v1".to_string(),
        readback_state,
        readback_state_class: readback_state_class.to_string(),
        readback_outcome: readback_outcome.to_string(),
        terminal_state_observed,
        ambiguous_state_observed,
        order_found,
        order_lineage_preserved,
        known_order_id,
        known_client_order_id,
        readback_order_id: redact_cancel_preview_identifier(
            "readback_order_id",
            &raw_readback_order_id,
        ),
        readback_client_order_id: redact_cancel_preview_identifier(
            "readback_client_order_id",
            &raw_readback_client_order_id,
        ),
        readback_orig_client_order_id: redact_cancel_preview_identifier(
            "readback_orig_client_order_id",
            &raw_readback_orig_client_order_id,
        ),
        symbol,
        account_label,
        readback_update_time_shape: production_mutation_response_time_shape(&readback, "updateTime"),
        allowed_readback_fields: production_mutation_post_cancel_readback_allowed_fields(),
        forbidden_readback_markers,
        source_artifact_issues,
        missing_cli_flags: missing_cli_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        unsupported_readback_states,
        api_key_value_recorded: false,
        api_secret_value_recorded: false,
        api_key_header_value_recorded: false,
        signature_recorded: false,
        signed_query_recorded: false,
        signed_url_recorded: false,
        raw_exchange_response_recorded: false,
        raw_readback_body_recorded: false,
        response_body_recorded: false,
        response_headers_recorded: false,
        unrestricted_payload_recorded: false,
        account_balances_recorded: false,
        fills_recorded: false,
        readback_execution_attempted: false,
        order_state_read_attempted: false,
        production_order_state_reads_attempted: 0,
        actual_cancel_send_allowed: false,
        cancel_attempted: false,
        cancel_requests_sent: 0,
        production_order_mutations_attempted: 0,
        network_attempted: false,
        network_readback_endpoint_attempted: false,
        network_cancel_endpoint_attempted: false,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        remediation_attempted: false,
        automatic_cancel_allowed: false,
        automatic_remediation_allowed: false,
        production_order_mutation_allowed: false,
        dashboard_order_controls_enabled: false,
        dashboard_cancel_controls_enabled: false,
        cancel_response_redaction_ready_confirmed: opt.confirm_cancel_response_redaction_ready,
        readback_metadata_only_confirmed: opt.confirm_readback_metadata_only,
        terminal_and_ambiguous_classification_confirmed: opt
            .confirm_terminal_and_ambiguous_classification,
        no_raw_readback_persistence_confirmed: opt.confirm_no_raw_readback_persistence,
        no_headers_persistence_confirmed: opt.confirm_no_headers_persistence,
        no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
        no_mutation_confirmed: opt.confirm_no_mutation,
        no_retry_confirmed: opt.confirm_no_retry,
        no_remediation_confirmed: opt.confirm_no_remediation,
        no_cancel_confirmed: opt.confirm_no_cancel,
        no_network_confirmed: opt.confirm_no_network,
        dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
        diagnostic: if post_cancel_readback_ready {
            "future post-cancel readback metadata was classified without raw response persistence, network readback, cancel send, retry, remediation, mutation, or Dashboard cancel controls"
        } else {
            "future post-cancel readback contract blocked before persisting unrestricted readback material or enabling retry/remediation controls"
        }
        .to_string(),
    })
}

fn build_production_mutation_cancel_recovery_incident_audit_closeout_artifact(
    opt: &LiveProductionMutationCancelRecoveryIncidentAuditCloseoutOpt,
) -> anyhow::Result<ProductionMutationCancelRecoveryIncidentAuditCloseoutArtifact> {
    validate_non_empty("run_id", &opt.run_id)?;

    let cancel_risk_gate = load_json_value(&opt.cancel_risk_gate, "cancel risk gate")?;
    let manual_owner_approval_lifecycle = load_json_value(
        &opt.manual_owner_approval_lifecycle,
        "manual owner approval lifecycle",
    )?;
    let cancel_response_redaction =
        load_json_value(&opt.cancel_response_redaction, "cancel response redaction")?;
    let post_cancel_readback = load_json_value(&opt.post_cancel_readback, "post-cancel readback")?;
    let missing_cli_flags =
        missing_production_mutation_cancel_recovery_incident_audit_closeout_cli_flags(opt);
    let source_artifact_issues =
        production_mutation_cancel_recovery_incident_audit_closeout_source_issues(
            &cancel_risk_gate,
            &manual_owner_approval_lifecycle,
            &cancel_response_redaction,
            &post_cancel_readback,
        );
    let lineage_issues = production_mutation_cancel_recovery_incident_audit_closeout_lineage_issues(
        &cancel_risk_gate,
        &manual_owner_approval_lifecycle,
        &cancel_response_redaction,
        &post_cancel_readback,
    );
    let closeout_ready = missing_cli_flags.is_empty()
        && source_artifact_issues.is_empty()
        && lineage_issues.is_empty();
    let status = if !missing_cli_flags.is_empty() {
        "blocked_missing_gate"
    } else if !source_artifact_issues.is_empty() {
        "blocked_source_artifact"
    } else if !lineage_issues.is_empty() {
        "blocked_lineage_mismatch"
    } else {
        "ready_cancel_recovery_incident_audit_closeout"
    };

    let order_lineage_id = json_string_value(&post_cancel_readback, "order_lineage_id")
        .or_else(|| json_string_value(&cancel_response_redaction, "order_lineage_id"))
        .or_else(|| json_string_value(&manual_owner_approval_lifecycle, "order_lineage_id"))
        .or_else(|| json_string_value(&cancel_risk_gate, "order_lineage_id"))
        .unwrap_or_else(|| "missing".to_string());
    let readback_state_class = json_string_value(&post_cancel_readback, "readback_state_class")
        .unwrap_or_else(|| "unknown".to_string());
    let terminal_action_recommendation =
        cancel_recovery_terminal_action_recommendation(&readback_state_class);
    let (remaining_risk, remaining_risk_requires_manual_review) =
        cancel_recovery_remaining_risk(&readback_state_class);
    let risk_gate_ready =
        json_bool_value(&cancel_risk_gate, "cancel_risk_gate_ready").unwrap_or(false);
    let redaction_ready =
        json_bool_value(&cancel_response_redaction, "response_redaction_ready").unwrap_or(false);
    let cancel_response_redacted =
        json_bool_value(&cancel_response_redaction, "cancel_response_redacted").unwrap_or(false);

    Ok(
        ProductionMutationCancelRecoveryIncidentAuditCloseoutArtifact {
            schema_version:
                PRODUCTION_MUTATION_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_SCHEMA_VERSION
                    .to_string(),
            run_id: opt.run_id.clone(),
            order_lineage_id,
            source_cancel_risk_gate_path: opt.cancel_risk_gate.display().to_string(),
            source_manual_owner_approval_lifecycle_path: opt
                .manual_owner_approval_lifecycle
                .display()
                .to_string(),
            source_cancel_response_redaction_path: opt
                .cancel_response_redaction
                .display()
                .to_string(),
            source_post_cancel_readback_path: opt.post_cancel_readback.display().to_string(),
            source_cancel_risk_gate_run_id: json_string_value(&cancel_risk_gate, "run_id")
                .unwrap_or_else(|| "unknown".to_string()),
            source_manual_owner_approval_lifecycle_run_id: json_string_value(
                &manual_owner_approval_lifecycle,
                "run_id",
            )
            .unwrap_or_else(|| "unknown".to_string()),
            source_cancel_response_redaction_run_id: json_string_value(
                &cancel_response_redaction,
                "run_id",
            )
            .unwrap_or_else(|| "unknown".to_string()),
            source_post_cancel_readback_run_id: json_string_value(
                &post_cancel_readback,
                "run_id",
            )
            .unwrap_or_else(|| "unknown".to_string()),
            source_cancel_risk_gate_hash: file_fnv1a64_hash(
                &opt.cancel_risk_gate.display().to_string(),
            ),
            source_manual_owner_approval_lifecycle_hash: file_fnv1a64_hash(
                &opt.manual_owner_approval_lifecycle.display().to_string(),
            ),
            source_cancel_response_redaction_hash: file_fnv1a64_hash(
                &opt.cancel_response_redaction.display().to_string(),
            ),
            source_post_cancel_readback_hash: file_fnv1a64_hash(
                &opt.post_cancel_readback.display().to_string(),
            ),
            artifact_type: "cancel_recovery_incident_audit_closeout".to_string(),
            status: status.to_string(),
            created_at: now_millis(),
            mode: "single_mutation_candidate_cancel_recovery_incident_audit_closeout".to_string(),
            capability: "Owner-Approved Cancel Recovery Preview".to_string(),
            capability_expansion: "incident_audit_closeout_only".to_string(),
            lineage_scope: json_string_value(&cancel_risk_gate, "lineage_scope")
                .unwrap_or_else(|| "unknown".to_string()),
            default_fail_closed: true,
            cancel_risk_gate_ref: production_mutation_local_order_ledger_source_ref(
                &opt.cancel_risk_gate,
                &cancel_risk_gate,
                "cancel_risk_gate_ready",
            ),
            manual_owner_approval_lifecycle_ref:
                production_mutation_local_order_ledger_source_ref(
                    &opt.manual_owner_approval_lifecycle,
                    &manual_owner_approval_lifecycle,
                    "approval_lifecycle_valid",
                ),
            cancel_response_redaction_ref: production_mutation_local_order_ledger_source_ref(
                &opt.cancel_response_redaction,
                &cancel_response_redaction,
                "response_redaction_ready",
            ),
            post_cancel_readback_ref: production_mutation_local_order_ledger_source_ref(
                &opt.post_cancel_readback,
                &post_cancel_readback,
                "post_cancel_readback_ready",
            ),
            cancel_recovery_lineage_ready: source_artifact_issues.is_empty()
                && lineage_issues.is_empty(),
            incident_closeout_ready: closeout_ready,
            audit_trail_ready: closeout_ready,
            audit_traceability_ready: closeout_ready,
            recovery_needed_reason: cancel_recovery_needed_reason(&cancel_risk_gate).to_string(),
            risk_gate_result: if risk_gate_ready {
                "ready_owner_approval_required"
            } else {
                "blocked"
            }
            .to_string(),
            risk_gate_ready,
            orphan_risk_detected: json_bool_value(&cancel_risk_gate, "orphan_risk_detected")
                .unwrap_or(false),
            risk_halted: json_bool_value(&cancel_risk_gate, "risk_halted").unwrap_or(false),
            new_orders_blocked: json_bool_value(&cancel_risk_gate, "new_orders_blocked")
                .unwrap_or(false),
            manual_review_required: json_bool_value(&cancel_risk_gate, "manual_review_required")
                .unwrap_or(false),
            owner_approval_state: json_string_value(
                &manual_owner_approval_lifecycle,
                "approval_state",
            )
            .unwrap_or_else(|| "unknown".to_string()),
            manual_approval_recorded: json_bool_value(
                &manual_owner_approval_lifecycle,
                "manual_approval_recorded",
            )
            .unwrap_or(false),
            approval_lifecycle_valid: json_bool_value(
                &manual_owner_approval_lifecycle,
                "approval_lifecycle_valid",
            )
            .unwrap_or(false),
            approval_consumed: json_bool_value(
                &manual_owner_approval_lifecycle,
                "approval_consumed",
            )
            .unwrap_or(false),
            redaction_contract_state: if redaction_ready && cancel_response_redacted {
                "ready_redacted_metadata_only"
            } else {
                "blocked"
            }
            .to_string(),
            cancel_response_redaction_ready: redaction_ready,
            cancel_response_redacted,
            post_cancel_readback_ready: json_bool_value(
                &post_cancel_readback,
                "post_cancel_readback_ready",
            )
            .unwrap_or(false),
            readback_state: json_string_value(&post_cancel_readback, "readback_state")
                .unwrap_or_else(|| "unknown".to_string()),
            readback_state_class,
            readback_outcome: json_string_value(&post_cancel_readback, "readback_outcome")
                .unwrap_or_else(|| "unknown".to_string()),
            terminal_state_observed: json_bool_value(
                &post_cancel_readback,
                "terminal_state_observed",
            )
            .unwrap_or(false),
            ambiguous_state_observed: json_bool_value(
                &post_cancel_readback,
                "ambiguous_state_observed",
            )
            .unwrap_or(false),
            terminal_action_recommendation: terminal_action_recommendation.to_string(),
            remaining_risk: remaining_risk.to_string(),
            remaining_risk_requires_manual_review,
            order_lineage_preserved: json_bool_value(
                &post_cancel_readback,
                "order_lineage_preserved",
            )
            .unwrap_or(false),
            candidate_count: json_u64_value(&cancel_risk_gate, "candidate_count").unwrap_or(0),
            known_order_id: json_scalar_string_value(&cancel_risk_gate, "known_order_id")
                .unwrap_or_else(|| "missing".to_string()),
            known_client_order_id: json_scalar_string_value(
                &cancel_risk_gate,
                "known_client_order_id",
            )
            .unwrap_or_else(|| "missing".to_string()),
            symbol: json_scalar_string_value(&cancel_risk_gate, "symbol")
                .unwrap_or_else(|| "unknown".to_string()),
            account_label: json_scalar_string_value(&cancel_risk_gate, "account_label")
                .unwrap_or_else(|| "unknown".to_string()),
            source_artifact_issues,
            lineage_issues,
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
            raw_exchange_response_recorded: false,
            raw_readback_body_recorded: false,
            response_body_recorded: false,
            response_headers_recorded: false,
            unrestricted_payload_recorded: false,
            account_balances_recorded: false,
            fills_recorded: false,
            readback_execution_attempted: false,
            order_state_read_attempted: false,
            production_order_state_reads_attempted: 0,
            actual_cancel_send_allowed: false,
            cancel_attempted: false,
            cancel_requests_sent: 0,
            production_order_mutations_attempted: 0,
            network_attempted: false,
            network_readback_endpoint_attempted: false,
            network_cancel_endpoint_attempted: false,
            retry_attempted: false,
            replace_attempted: false,
            amend_attempted: false,
            flatten_attempted: false,
            remediation_attempted: false,
            automatic_cancel_allowed: false,
            automatic_remediation_allowed: false,
            production_order_mutation_allowed: false,
            dashboard_order_controls_enabled: false,
            dashboard_cancel_controls_enabled: false,
            cancel_recovery_lineage_confirmed: opt.confirm_cancel_recovery_lineage,
            risk_reason_recorded_confirmed: opt.confirm_risk_reason_recorded,
            risk_gate_result_recorded_confirmed: opt.confirm_risk_gate_result_recorded,
            owner_approval_state_recorded_confirmed: opt
                .confirm_owner_approval_state_recorded,
            redaction_contract_state_recorded_confirmed: opt
                .confirm_redaction_contract_state_recorded,
            readback_state_recorded_confirmed: opt.confirm_readback_state_recorded,
            terminal_action_recommendation_confirmed: opt
                .confirm_terminal_action_recommendation,
            remaining_risk_recorded_confirmed: opt.confirm_remaining_risk_recorded,
            no_mutation_confirmed: opt.confirm_no_mutation,
            no_cancel_confirmed: opt.confirm_no_cancel,
            no_network_confirmed: opt.confirm_no_network,
            no_retry_confirmed: opt.confirm_no_retry,
            no_remediation_confirmed: opt.confirm_no_remediation,
            no_automatic_remediation_confirmed: opt.confirm_no_automatic_remediation,
            dashboard_controls_disabled_confirmed: opt.confirm_dashboard_order_controls_disabled,
            no_secret_persistence_confirmed: opt.confirm_no_secret_persistence,
            diagnostic: if closeout_ready {
                "cancel recovery incident/audit closeout links risk gate, owner approval, response redaction, and post-cancel readback evidence while preserving no-send, no-network, no-retry, no-remediation, and Dashboard cancel-control boundaries"
            } else {
                "cancel recovery incident/audit closeout is blocked because confirmations, source artifacts, or lineage traceability are incomplete"
            }
            .to_string(),
        },
    )
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

fn production_mutation_request_builder_price_safety_preflight(
    price: &str,
    opt: &LiveProductionMutationRequestBuilderOpt,
) -> ProductionMutationPriceSafetyPreflight {
    let market_reference_source = opt.market_reference_source.trim().to_string();
    let market_reference_price = opt.market_reference_price.trim().to_string();
    let max_reference_price_distance_bps = opt.max_reference_price_distance_bps.trim().to_string();
    let mut source_artifact_issues = Vec::new();

    if market_reference_source.is_empty() {
        source_artifact_issues.push("market_reference_source_missing".to_string());
    }

    let price_distance_from_reference_bps = match (
        parse_non_negative_decimal(price),
        parse_non_negative_decimal(&market_reference_price),
        parse_non_negative_decimal(&max_reference_price_distance_bps),
    ) {
        (Ok(limit_price), Ok(reference_price), Ok(max_distance_bps))
            if limit_price > Decimal::ZERO
                && reference_price > Decimal::ZERO
                && max_distance_bps >= Decimal::ZERO =>
        {
            let distance = if limit_price >= reference_price {
                limit_price - reference_price
            } else {
                reference_price - limit_price
            };
            let distance_bps = distance * Decimal::new(10_000, 0) / reference_price;
            if distance_bps > max_distance_bps {
                source_artifact_issues.push("price_distance_exceeds_reference_limit".to_string());
            }
            format_decimal(&distance_bps)
        }
        (Err(_), _, _) => {
            source_artifact_issues.push("limit_price_parse_failed".to_string());
            "unavailable".to_string()
        }
        (_, Err(_), _) => {
            source_artifact_issues.push("market_reference_price_missing_or_invalid".to_string());
            "unavailable".to_string()
        }
        (_, _, Err(_)) => {
            source_artifact_issues.push("max_reference_price_distance_bps_invalid".to_string());
            "unavailable".to_string()
        }
        (Ok(limit_price), Ok(reference_price), Ok(max_distance_bps)) => {
            if limit_price <= Decimal::ZERO {
                source_artifact_issues.push("limit_price_not_positive".to_string());
            }
            if reference_price <= Decimal::ZERO {
                source_artifact_issues.push("market_reference_price_not_positive".to_string());
            }
            if max_distance_bps < Decimal::ZERO {
                source_artifact_issues
                    .push("max_reference_price_distance_bps_negative".to_string());
            }
            "unavailable".to_string()
        }
    };

    if opt.would_cross_spread {
        source_artifact_issues.push("limit_price_would_cross_spread".to_string());
    }

    source_artifact_issues.sort();
    source_artifact_issues.dedup();
    let preflight_ready = source_artifact_issues.is_empty();

    ProductionMutationPriceSafetyPreflight {
        market_reference_source,
        market_reference_price,
        max_reference_price_distance_bps,
        price_distance_from_reference_bps,
        preflight_ready,
        source_artifact_issues,
    }
}

fn production_mutation_guarded_send_source_issues(
    request_builder: &serde_json::Value,
    kill_switch_runtime_gate: &serde_json::Value,
    request_preview: &serde_json::Value,
    max_notional: &str,
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

fn production_mutation_response_redaction_has_actual_http_result(
    guarded_send: &serde_json::Value,
) -> bool {
    json_bool_value(guarded_send, "request_sent").unwrap_or(false)
        && json_bool_value(guarded_send, "http_send_attempted").unwrap_or(false)
        && json_bool_value(guarded_send, "exchange_ack_observed").unwrap_or(false)
        && json_bool_value(guarded_send, "confirmed_production_order_submission").unwrap_or(false)
        && json_u64_value(guarded_send, "production_order_submissions_attempted").unwrap_or(0) > 0
        && json_u64_value(guarded_send, "production_orders_submitted").unwrap_or(0) > 0
        && json_u64_value(guarded_send, "production_order_mutations_attempted").unwrap_or(0) > 0
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

fn production_mutation_local_order_ledger_source_ref(
    path: &Path,
    artifact: &serde_json::Value,
    ready_field: &str,
) -> ProductionMutationLocalOrderLedgerSourceRef {
    let artifact_type =
        json_string_value(artifact, "artifact_type").unwrap_or_else(|| "missing".to_string());
    ProductionMutationLocalOrderLedgerSourceRef {
        path: path.display().to_string(),
        hash: file_fnv1a64_hash(&path.display().to_string()),
        sha256: file_sha256_hash(path),
        bytes: file_byte_len(path),
        source_command: production_mutation_source_command(&artifact_type).to_string(),
        source_commit: std::env::var("NTPRO_SOURCE_COMMIT")
            .unwrap_or_else(|_| "unknown".to_string()),
        source_release_tag: std::env::var("NTPRO_SOURCE_RELEASE_TAG")
            .unwrap_or_else(|_| "unreleased".to_string()),
        schema_version: json_string_value(artifact, "schema_version")
            .unwrap_or_else(|| "missing".to_string()),
        artifact_type,
        status: json_string_value(artifact, "status").unwrap_or_else(|| "unknown".to_string()),
        ready: json_bool_value(artifact, ready_field).unwrap_or(false),
    }
}

fn production_mutation_source_file_ref(path: &Path) -> ProductionMutationSourceFileRef {
    ProductionMutationSourceFileRef {
        path: path.display().to_string(),
        hash: file_fnv1a64_hash(&path.display().to_string()),
        sha256: file_sha256_hash(path),
        bytes: file_byte_len(path),
    }
}

fn production_mutation_source_command(artifact_type: &str) -> &'static str {
    match artifact_type {
        "production_mutation_request_builder" => {
            "nautilus live production-mutation-request-builder"
        }
        "production_mutation_guarded_send" => "nautilus live production-mutation-guarded-send",
        "production_mutation_response_redaction" => {
            "nautilus live production-mutation-response-redaction"
        }
        "production_mutation_order_state_readback" => {
            "nautilus live production-mutation-order-state-readback"
        }
        "production_mutation_audit_trail" => "nautilus live production-mutation-audit-trail",
        "production_mutation_failure_semantics" => {
            "nautilus live production-mutation-failure-semantics"
        }
        "production_mutation_local_order_ledger" => {
            "nautilus live production-mutation-local-order-ledger"
        }
        "production_mutation_exchange_readback_mapper" => {
            "nautilus live production-mutation-exchange-readback-mapper"
        }
        "production_mutation_reconciliation_classifier" => {
            "nautilus live production-mutation-reconciliation-classifier"
        }
        "production_mutation_orphan_order_detector" => {
            "nautilus live production-mutation-orphan-order-detector"
        }
        "cancel_request_preview" => "nautilus live production-mutation-cancel-request-preview",
        "cancel_risk_gate" => "nautilus live production-mutation-cancel-risk-gate",
        "manual_owner_approval_lifecycle" => {
            "nautilus live production-mutation-manual-owner-approval-lifecycle"
        }
        "actual_cancel_owner_approval_lifecycle" => {
            "nautilus live production-mutation-actual-cancel-owner-approval-lifecycle"
        }
        "actual_cancel_executor_adapter_boundary" => {
            "nautilus live production-mutation-actual-cancel-executor-adapter-boundary"
        }
        "actual_cancel_single_shot" => {
            "nautilus live production-mutation-actual-cancel-single-shot"
        }
        "actual_cancel_readback_reconciliation" => {
            "nautilus live production-mutation-actual-cancel-readback-reconciliation"
        }
        "actual_cancel_failure_evidence" => {
            "nautilus live production-mutation-actual-cancel-failure-evidence"
        }
        "cancel_response_redaction" => {
            "nautilus live production-mutation-cancel-response-redaction"
        }
        "post_cancel_readback" => "nautilus live production-mutation-post-cancel-readback",
        "cancel_recovery_incident_audit_closeout" => {
            "nautilus live production-mutation-cancel-recovery-incident-audit-closeout"
        }
        "redacted_binance_order_readback" => {
            "nautilus live production-mutation-exchange-readback-mapper --order-readback"
        }
        "redacted_binance_open_orders_readback" => {
            "nautilus live production-mutation-exchange-readback-mapper --open-orders-readback"
        }
        _ => "unknown",
    }
}

fn production_mutation_local_order_ledger_source_issues(
    request_builder: &serde_json::Value,
    guarded_send: &serde_json::Value,
    response_redaction: &serde_json::Value,
    order_state_readback: &serde_json::Value,
    audit_trail: &serde_json::Value,
    failure_semantics: &serde_json::Value,
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
    if json_string_value(audit_trail, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION)
    {
        issues.push("audit_trail_schema_mismatch".to_string());
    }
    if json_string_value(failure_semantics, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION)
    {
        issues.push("failure_semantics_schema_mismatch".to_string());
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
    if json_string_value(audit_trail, "status").as_deref() != Some("ready_redacted_audit_trail") {
        issues.push("audit_trail_not_ready".to_string());
    }
    if json_string_value(failure_semantics, "status").as_deref()
        != Some("ready_failure_semantics_evidence")
    {
        issues.push("failure_semantics_not_ready".to_string());
    }
    for (label, artifact) in [
        ("request_builder", request_builder),
        ("guarded_send", guarded_send),
        ("response_redaction", response_redaction),
        ("order_state_readback", order_state_readback),
        ("audit_trail", audit_trail),
        ("failure_semantics", failure_semantics),
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
            "retry_attempted",
            "cancel_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
            "remediation_attempted",
            "dashboard_order_controls_enabled",
        ] {
            if json_bool_value(artifact, field).unwrap_or(false) {
                issues.push(format!("{label}_{field}_true"));
            }
        }
    }
    if json_u64_value(audit_trail, "production_order_mutations_attempted").unwrap_or(0) > 1 {
        issues.push("audit_trail_multiple_mutation_attempts".to_string());
    }
    if json_u64_value(guarded_send, "production_order_mutations_attempted").unwrap_or(0) > 1 {
        issues.push("guarded_send_multiple_mutation_attempts".to_string());
    }
    if !json_bool_value(failure_semantics, "stop_after_evidence").unwrap_or(false) {
        issues.push("failure_semantics_does_not_stop_after_evidence".to_string());
    }
    if json_bool_value(failure_semantics, "strategy_continuation_allowed").unwrap_or(false) {
        issues.push("failure_semantics_allows_strategy_continuation".to_string());
    }
    issues
}

fn production_mutation_exchange_readback_mapper_source_issues(
    local_order_ledger: &serde_json::Value,
    order_readback: &serde_json::Value,
    open_orders_readback: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(local_order_ledger, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_SCHEMA_VERSION)
    {
        issues.push("local_order_ledger_schema_mismatch".to_string());
    }
    if json_string_value(local_order_ledger, "status").as_deref()
        != Some("ready_local_order_ledger")
    {
        issues.push("local_order_ledger_not_ready".to_string());
    }
    if !json_bool_value(local_order_ledger, "local_ledger_ready").unwrap_or(false) {
        issues.push("local_order_ledger_ready_false".to_string());
    }
    if json_string_value(order_readback, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_EXCHANGE_ORDER_READBACK_SCHEMA_VERSION)
    {
        issues.push("order_readback_schema_mismatch".to_string());
    }
    if json_string_value(open_orders_readback, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_EXCHANGE_OPEN_ORDERS_READBACK_SCHEMA_VERSION)
    {
        issues.push("open_orders_readback_schema_mismatch".to_string());
    }
    if !json_bool_value(order_readback, "response_redacted").unwrap_or(false) {
        issues.push("order_readback_not_redacted".to_string());
    }
    if !json_bool_value(open_orders_readback, "response_redacted").unwrap_or(false) {
        issues.push("open_orders_readback_not_redacted".to_string());
    }
    for (label, artifact) in [
        ("local_order_ledger", local_order_ledger),
        ("order_readback", order_readback),
        ("open_orders_readback", open_orders_readback),
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
            "request_sent",
            "network_attempted",
            "retry_attempted",
            "cancel_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
            "remediation_attempted",
            "dashboard_order_controls_enabled",
        ] {
            if json_bool_value(artifact, field).unwrap_or(false) {
                issues.push(format!("{label}_{field}_true"));
            }
        }
    }
    issues
}

fn production_mutation_reconciliation_classifier_source_issues(
    exchange_readback_mapper: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(exchange_readback_mapper, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION)
    {
        issues.push("exchange_readback_mapper_schema_mismatch".to_string());
    }
    let source_status = json_string_value(exchange_readback_mapper, "status")
        .unwrap_or_else(|| "unknown".to_string());
    if !matches!(
        source_status.as_str(),
        "ready_exchange_readback_mapped" | "blocked_malformed_exchange_readback"
    ) {
        issues.push(format!("exchange_readback_mapper_status_{source_status}"));
    }
    let exchange_readback_mapped =
        json_bool_value(exchange_readback_mapper, "exchange_readback_mapped").unwrap_or(false);
    if !exchange_readback_mapped && source_status != "blocked_malformed_exchange_readback" {
        issues.push("exchange_readback_mapper_not_mapped".to_string());
    }
    if json_bool_value(exchange_readback_mapper, "reconciliation_classified").unwrap_or(false) {
        issues.push("exchange_readback_mapper_already_classified".to_string());
    }
    if json_bool_value(exchange_readback_mapper, "orphan_risk_detected").unwrap_or(false) {
        issues.push("exchange_readback_mapper_orphan_risk_already_detected".to_string());
    }
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
        "network_attempted",
        "retry_attempted",
        "cancel_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
    ] {
        if json_bool_value(exchange_readback_mapper, field).unwrap_or(false) {
            issues.push(format!("exchange_readback_mapper_{field}_true"));
        }
    }
    issues
}

fn production_mutation_orphan_order_detector_source_issues(
    reconciliation_classifier: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(reconciliation_classifier, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION)
    {
        issues.push("reconciliation_classifier_schema_mismatch".to_string());
    }
    let source_status = json_string_value(reconciliation_classifier, "status")
        .unwrap_or_else(|| "unknown".to_string());
    if source_status != "ready_reconciliation_classified" {
        issues.push(format!("reconciliation_classifier_status_{source_status}"));
    }
    if !json_bool_value(reconciliation_classifier, "reconciliation_classified").unwrap_or(false) {
        issues.push("reconciliation_classifier_not_classified".to_string());
    }
    if json_bool_value(reconciliation_classifier, "orphan_risk_detected").unwrap_or(false) {
        issues.push("reconciliation_classifier_orphan_risk_already_detected".to_string());
    }
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
        "network_attempted",
        "retry_attempted",
        "cancel_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
    ] {
        if json_bool_value(reconciliation_classifier, field).unwrap_or(false) {
            issues.push(format!("reconciliation_classifier_{field}_true"));
        }
    }
    issues
}

fn production_mutation_cancel_request_preview_source_issues(
    orphan_order_detector: &serde_json::Value,
    reconciliation_classifier: &serde_json::Value,
    exchange_readback_mapper: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(orphan_order_detector, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_SCHEMA_VERSION)
    {
        issues.push("orphan_order_detector_schema_mismatch".to_string());
    }
    if json_string_value(orphan_order_detector, "status").as_deref()
        != Some("ready_orphan_order_detection_completed")
    {
        let status = json_string_value(orphan_order_detector, "status")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("orphan_order_detector_status_{status}"));
    }
    if !json_bool_value(orphan_order_detector, "orphan_detection_completed").unwrap_or(false) {
        issues.push("orphan_order_detector_not_completed".to_string());
    }
    for field in [
        "orphan_risk_detected",
        "risk_halted",
        "new_orders_blocked",
        "manual_review_required",
    ] {
        if !json_bool_value(orphan_order_detector, field).unwrap_or(false) {
            issues.push(format!("orphan_order_detector_{field}_not_true"));
        }
    }
    if json_string_value(orphan_order_detector, "lineage_scope").as_deref()
        != Some("single_v16_mutation_candidate")
    {
        issues.push("orphan_order_detector_lineage_scope_mismatch".to_string());
    }
    if json_string_value(reconciliation_classifier, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION)
    {
        issues.push("reconciliation_classifier_schema_mismatch".to_string());
    }
    if json_string_value(reconciliation_classifier, "status").as_deref()
        != Some("ready_reconciliation_classified")
    {
        let status = json_string_value(reconciliation_classifier, "status")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("reconciliation_classifier_status_{status}"));
    }
    if !json_bool_value(reconciliation_classifier, "reconciliation_classified").unwrap_or(false) {
        issues.push("reconciliation_classifier_not_classified".to_string());
    }
    if json_string_value(exchange_readback_mapper, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION)
    {
        issues.push("exchange_readback_mapper_schema_mismatch".to_string());
    }
    if json_string_value(exchange_readback_mapper, "status").as_deref()
        != Some("ready_exchange_readback_mapped")
    {
        let status = json_string_value(exchange_readback_mapper, "status")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("exchange_readback_mapper_status_{status}"));
    }
    if !json_bool_value(exchange_readback_mapper, "exchange_readback_mapped").unwrap_or(false) {
        issues.push("exchange_readback_mapper_not_mapped".to_string());
    }

    let orphan_lineage = json_string_value(orphan_order_detector, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    let classifier_lineage = json_string_value(reconciliation_classifier, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    let mapper_lineage = json_string_value(exchange_readback_mapper, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    if orphan_lineage == "missing" || orphan_lineage.trim().is_empty() {
        issues.push("order_lineage_id_missing".to_string());
    }
    if orphan_lineage != classifier_lineage || orphan_lineage != mapper_lineage {
        issues.push("order_lineage_id_mismatch".to_string());
    }

    let known_order_id = json_scalar_string_value(exchange_readback_mapper, "known_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id =
        json_scalar_string_value(exchange_readback_mapper, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string());
    if !cancel_preview_identifier_known(&known_order_id)
        && !cancel_preview_identifier_known(&known_client_order_id)
    {
        issues.push("known_order_identifier_missing".to_string());
    }
    let symbol = json_scalar_string_value(exchange_readback_mapper, "symbol")
        .unwrap_or_else(|| "unknown".to_string());
    if !cancel_preview_identifier_known(&symbol) {
        issues.push("symbol_missing".to_string());
    }

    for (label, artifact) in [
        ("orphan_order_detector", orphan_order_detector),
        ("reconciliation_classifier", reconciliation_classifier),
        ("exchange_readback_mapper", exchange_readback_mapper),
    ] {
        append_true_marker_issues(
            &mut issues,
            label,
            artifact,
            &[
                "api_key_value_recorded",
                "api_secret_value_recorded",
                "api_key_header_value_recorded",
                "signature_recorded",
                "signed_query_recorded",
                "signed_url_recorded",
                "raw_exchange_response_recorded",
                "response_body_recorded",
                "response_headers_recorded",
                "network_attempted",
                "actual_cancel_send_allowed",
                "cancel_attempted",
                "retry_attempted",
                "replace_attempted",
                "amend_attempted",
                "flatten_attempted",
                "remediation_attempted",
                "automatic_cancel_allowed",
                "automatic_remediation_allowed",
                "production_order_mutation_allowed",
                "dashboard_order_controls_enabled",
                "dashboard_cancel_controls_enabled",
            ],
        );
        append_nonzero_marker_issues(
            &mut issues,
            label,
            artifact,
            &[
                "cancel_requests_sent",
                "production_order_mutations_attempted",
            ],
        );
    }

    issues
}

fn production_mutation_cancel_risk_gate_source_issues(
    cancel_request_preview: &serde_json::Value,
    expected_symbol: &str,
    expected_account_label: &str,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(cancel_request_preview, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_SCHEMA_VERSION)
    {
        issues.push("cancel_request_preview_schema_mismatch".to_string());
    }
    if json_string_value(cancel_request_preview, "artifact_type").as_deref()
        != Some("cancel_request_preview")
    {
        issues.push("cancel_request_preview_artifact_type_mismatch".to_string());
    }
    if json_string_value(cancel_request_preview, "status").as_deref()
        != Some("ready_cancel_request_preview")
    {
        let status = json_string_value(cancel_request_preview, "status")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("cancel_request_preview_status_{status}"));
    }
    if !json_bool_value(cancel_request_preview, "cancel_request_preview_ready").unwrap_or(false) {
        issues.push("cancel_request_preview_not_ready".to_string());
    }
    for field in [
        "orphan_risk_detected",
        "risk_halted",
        "new_orders_blocked",
        "manual_review_required",
        "order_identifier_known",
        "owner_approval_required",
    ] {
        if !json_bool_value(cancel_request_preview, field).unwrap_or(false) {
            issues.push(format!("cancel_request_preview_{field}_not_true"));
        }
    }
    if json_string_value(cancel_request_preview, "lineage_scope").as_deref()
        != Some("single_v16_mutation_candidate")
    {
        issues.push("cancel_request_preview_lineage_scope_mismatch".to_string());
    }
    let order_lineage_id = json_string_value(cancel_request_preview, "order_lineage_id")
        .unwrap_or_else(|| "missing".to_string());
    if !cancel_preview_identifier_known(&order_lineage_id) {
        issues.push("order_lineage_id_missing".to_string());
    }
    if json_u64_value(cancel_request_preview, "candidate_count").unwrap_or(0) != 1 {
        issues.push("cancel_request_preview_candidate_count_not_one".to_string());
    }
    let known_order_id = json_scalar_string_value(cancel_request_preview, "known_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id =
        json_scalar_string_value(cancel_request_preview, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string());
    if !cancel_preview_identifier_known(&known_order_id)
        && !cancel_preview_identifier_known(&known_client_order_id)
    {
        issues.push("known_order_identifier_missing".to_string());
    }
    let symbol = json_scalar_string_value(cancel_request_preview, "symbol")
        .unwrap_or_else(|| "unknown".to_string());
    if !cancel_risk_gate_field_matches(&symbol, expected_symbol) {
        issues.push("symbol_mismatch".to_string());
    }
    let account_label = json_scalar_string_value(cancel_request_preview, "account_label")
        .unwrap_or_else(|| "unknown".to_string());
    if !cancel_risk_gate_field_matches(&account_label, expected_account_label) {
        issues.push("account_label_mismatch".to_string());
    }
    for field in ["source_artifact_issues", "missing_cli_flags"] {
        if json_array_has_items(cancel_request_preview, field) {
            issues.push(format!("cancel_request_preview_{field}_not_empty"));
        }
    }

    append_true_marker_issues(
        &mut issues,
        "cancel_request_preview",
        cancel_request_preview,
        &[
            "multi_order_cancel_requested",
            "cancel_all_requested",
            "bulk_cancel_requested",
            "strategy_driven_cancel_requested",
            "multi_account_cancel_requested",
            "multi_venue_cancel_requested",
            "retry_requested",
            "replace_or_amend_requested",
            "flatten_requested",
            "dashboard_cancel_requested",
            "api_key_value_recorded",
            "api_secret_value_recorded",
            "api_key_header_value_recorded",
            "signature_recorded",
            "signed_query_recorded",
            "signed_url_recorded",
            "raw_exchange_response_recorded",
            "response_body_recorded",
            "response_headers_recorded",
            "network_attempted",
            "network_cancel_endpoint_attempted",
            "actual_cancel_send_allowed",
            "cancel_attempted",
            "retry_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
            "remediation_attempted",
            "automatic_cancel_allowed",
            "automatic_remediation_allowed",
            "production_order_mutation_allowed",
            "dashboard_order_controls_enabled",
            "dashboard_cancel_controls_enabled",
        ],
    );
    append_nonzero_marker_issues(
        &mut issues,
        "cancel_request_preview",
        cancel_request_preview,
        &[
            "cancel_requests_sent",
            "production_order_mutations_attempted",
        ],
    );

    issues
}

fn production_mutation_manual_owner_approval_lifecycle_source_issues(
    cancel_risk_gate: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(cancel_risk_gate, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION)
    {
        issues.push("cancel_risk_gate_schema_mismatch".to_string());
    }
    if json_string_value(cancel_risk_gate, "artifact_type").as_deref() != Some("cancel_risk_gate") {
        issues.push("cancel_risk_gate_artifact_type_mismatch".to_string());
    }
    if json_string_value(cancel_risk_gate, "status").as_deref() != Some("ready_cancel_risk_gate") {
        let status =
            json_string_value(cancel_risk_gate, "status").unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("cancel_risk_gate_status_{status}"));
    }
    for field in [
        "cancel_request_preview_ready",
        "cancel_risk_gate_ready",
        "orphan_risk_detected",
        "risk_halted",
        "new_orders_blocked",
        "manual_review_required",
        "order_identifier_known",
        "symbol_matches_lineage",
        "account_matches_lineage",
        "owner_approval_required",
    ] {
        if !json_bool_value(cancel_risk_gate, field).unwrap_or(false) {
            issues.push(format!("cancel_risk_gate_{field}_not_true"));
        }
    }
    if json_string_value(cancel_risk_gate, "lineage_scope").as_deref()
        != Some("single_v16_mutation_candidate")
    {
        issues.push("cancel_risk_gate_lineage_scope_mismatch".to_string());
    }
    let order_lineage_id =
        json_string_value(cancel_risk_gate, "order_lineage_id").unwrap_or_else(|| "missing".into());
    if !cancel_preview_identifier_known(&order_lineage_id) {
        issues.push("order_lineage_id_missing".to_string());
    }
    if json_u64_value(cancel_risk_gate, "candidate_count").unwrap_or(0) != 1 {
        issues.push("cancel_risk_gate_candidate_count_not_one".to_string());
    }
    if json_bool_value(cancel_risk_gate, "owner_approval_lifecycle_recorded").unwrap_or(false) {
        issues.push("cancel_risk_gate_owner_approval_lifecycle_already_recorded".to_string());
    }
    for field in ["source_artifact_issues", "missing_cli_flags"] {
        if json_array_has_items(cancel_risk_gate, field) {
            issues.push(format!("cancel_risk_gate_{field}_not_empty"));
        }
    }

    append_true_marker_issues(
        &mut issues,
        "cancel_risk_gate",
        cancel_risk_gate,
        &[
            "multi_order_cancel_requested",
            "cancel_all_requested",
            "bulk_cancel_requested",
            "strategy_driven_cancel_requested",
            "multi_account_cancel_requested",
            "multi_venue_cancel_requested",
            "retry_requested",
            "replace_or_amend_requested",
            "flatten_requested",
            "dashboard_cancel_requested",
            "api_key_value_recorded",
            "api_secret_value_recorded",
            "api_key_header_value_recorded",
            "signature_recorded",
            "signed_query_recorded",
            "signed_url_recorded",
            "raw_exchange_response_recorded",
            "response_body_recorded",
            "response_headers_recorded",
            "network_attempted",
            "network_cancel_endpoint_attempted",
            "actual_cancel_send_allowed",
            "cancel_attempted",
            "retry_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
            "remediation_attempted",
            "automatic_cancel_allowed",
            "automatic_remediation_allowed",
            "production_order_mutation_allowed",
            "dashboard_order_controls_enabled",
            "dashboard_cancel_controls_enabled",
        ],
    );
    append_nonzero_marker_issues(
        &mut issues,
        "cancel_risk_gate",
        cancel_risk_gate,
        &[
            "cancel_requests_sent",
            "production_order_mutations_attempted",
        ],
    );

    issues
}

fn actual_cancel_owner_approval_safety_contract_issues(raw: &str) -> Vec<String> {
    let mut issues = Vec::new();
    for (token, issue) in [
        (
            "ntpro.v190_actual_cancel_safety_contract.v1",
            "missing_v190_safety_contract_schema",
        ),
        (
            "Owner-Approved Single-Shot Actual Cancel",
            "missing_actual_cancel_capability",
        ),
        (
            "one_order_one_venue_one_attempt",
            "missing_one_order_one_venue_one_attempt_scope",
        ),
        (
            "missing_owner_approval",
            "missing_missing_owner_approval_reason",
        ),
        (
            "owner_approval_reused",
            "missing_owner_approval_reused_reason",
        ),
        (
            "bulk_cancel_requested",
            "missing_bulk_cancel_forbidden_reason",
        ),
        (
            "retry_or_repair_requested",
            "missing_retry_forbidden_reason",
        ),
        (
            "dashboard_operation_requested",
            "missing_dashboard_operation_forbidden_reason",
        ),
    ] {
        if !raw.contains(token) {
            issues.push(issue.to_string());
        }
    }
    issues
}

fn actual_cancel_owner_approval_release_manifest_issues(
    release_manifest: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(release_manifest, "schema_version").as_deref()
        != Some("ntpro.v181_release_manifest.v1")
    {
        issues.push("release_manifest_schema_mismatch".to_string());
    }
    if json_string_value(release_manifest, "product_version").as_deref() != Some("v0.18.1") {
        issues.push("release_manifest_product_version_mismatch".to_string());
    }
    if json_pointer_string_value(release_manifest, "/patch_release/planned_tag").as_deref()
        != Some(opt.expected_release_tag.as_str())
    {
        issues.push("release_manifest_planned_tag_mismatch".to_string());
    }
    if json_pointer_string_value(release_manifest, "/capability/actual_cancel_scope").as_deref()
        != Some("not_included")
    {
        issues.push("release_manifest_actual_cancel_scope_mismatch".to_string());
    }
    for (pointer, issue) in [
        (
            "/boundary_flags/actual_cancel_send_allowed",
            "release_manifest_actual_cancel_send_allowed_true",
        ),
        (
            "/boundary_flags/automatic_cancel_allowed",
            "release_manifest_automatic_cancel_allowed_true",
        ),
        (
            "/boundary_flags/dashboard_cancel_controls_enabled",
            "release_manifest_dashboard_cancel_controls_enabled_true",
        ),
        (
            "/boundary_flags/network_cancel_endpoint_attempted",
            "release_manifest_network_cancel_endpoint_attempted_true",
        ),
    ] {
        if json_pointer_bool_value(release_manifest, pointer).unwrap_or(true) {
            issues.push(issue.to_string());
        }
    }
    issues
}

fn production_mutation_actual_cancel_owner_approval_lifecycle_source_issues(
    cancel_risk_gate: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt,
) -> Vec<String> {
    let mut issues =
        production_mutation_manual_owner_approval_lifecycle_source_issues(cancel_risk_gate);
    if json_string_value(cancel_risk_gate, "order_lineage_id").as_deref()
        != Some(opt.expected_order_lineage_id.as_str())
    {
        issues.push("order_lineage_id_mismatch".to_string());
    }
    if json_scalar_string_value(cancel_risk_gate, "symbol").as_deref()
        != Some(opt.expected_symbol.as_str())
    {
        issues.push("symbol_mismatch".to_string());
    }
    if json_scalar_string_value(cancel_risk_gate, "account_label").as_deref()
        != Some(opt.expected_account_label.as_str())
    {
        issues.push("account_label_mismatch".to_string());
    }
    issues
}

fn production_mutation_actual_cancel_executor_adapter_boundary_source_issues(
    owner_approval_lifecycle: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(owner_approval_lifecycle, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION)
    {
        issues.push("owner_approval_lifecycle_schema_mismatch".to_string());
    }
    if json_string_value(owner_approval_lifecycle, "artifact_type").as_deref()
        != Some("actual_cancel_owner_approval_lifecycle")
    {
        issues.push("owner_approval_lifecycle_artifact_type_mismatch".to_string());
    }
    if !json_bool_value(owner_approval_lifecycle, "approval_execution_authorized").unwrap_or(false)
    {
        issues.push("owner_approval_not_authorized".to_string());
    }
    if !json_bool_value(owner_approval_lifecycle, "single_order_required").unwrap_or(false) {
        issues.push("owner_approval_single_order_not_required".to_string());
    }
    if !json_bool_value(owner_approval_lifecycle, "single_venue_required").unwrap_or(false) {
        issues.push("owner_approval_single_venue_not_required".to_string());
    }
    if !json_bool_value(
        owner_approval_lifecycle,
        "single_execution_attempt_required",
    )
    .unwrap_or(false)
    {
        issues.push("owner_approval_single_attempt_not_required".to_string());
    }
    if json_bool_value(owner_approval_lifecycle, "approval_reusable").unwrap_or(true) {
        issues.push("owner_approval_reusable".to_string());
    }
    if json_string_value(owner_approval_lifecycle, "order_lineage_id").as_deref()
        != Some(opt.expected_order_lineage_id.as_str())
    {
        issues.push("order_lineage_id_mismatch".to_string());
    }
    if json_scalar_string_value(owner_approval_lifecycle, "symbol").as_deref()
        != Some(opt.expected_symbol.as_str())
    {
        issues.push("symbol_mismatch".to_string());
    }
    if json_scalar_string_value(owner_approval_lifecycle, "account_label").as_deref()
        != Some(opt.expected_account_label.as_str())
    {
        issues.push("account_label_mismatch".to_string());
    }
    if json_string_value(owner_approval_lifecycle, "venue").as_deref() != Some(opt.venue.as_str()) {
        issues.push("venue_mismatch".to_string());
    }
    if json_bool_value(owner_approval_lifecycle, "bulk_cancel_allowed").unwrap_or(true) {
        issues.push("owner_approval_bulk_cancel_allowed".to_string());
    }
    if json_bool_value(owner_approval_lifecycle, "retry_attempted").unwrap_or(true) {
        issues.push("owner_approval_retry_attempted".to_string());
    }
    if json_bool_value(
        owner_approval_lifecycle,
        "dashboard_cancel_controls_enabled",
    )
    .unwrap_or(true)
    {
        issues.push("owner_approval_dashboard_cancel_controls_enabled".to_string());
    }
    issues
}

fn production_mutation_actual_cancel_executor_adapter_boundary_capability_issues(
    adapter_capability: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(adapter_capability, "schema_version").as_deref()
        != Some("ntpro.v190_actual_cancel_adapter_capability.v1")
    {
        issues.push("adapter_capability_schema_mismatch".to_string());
    }
    if json_string_value(adapter_capability, "artifact_type").as_deref()
        != Some("actual_cancel_adapter_capability")
    {
        issues.push("adapter_capability_artifact_type_mismatch".to_string());
    }
    if json_string_value(adapter_capability, "adapter_id").as_deref()
        != Some(opt.adapter_id.as_str())
    {
        issues.push("adapter_id_mismatch".to_string());
    }
    if !json_bool_value(adapter_capability, "actual_cancel_supported").unwrap_or(false) {
        issues.push("adapter_actual_cancel_unsupported".to_string());
    }
    let supported_venues = json_string_array(adapter_capability, "supported_venues");
    if !supported_venues.iter().any(|venue| venue == &opt.venue) {
        issues.push("adapter_venue_unsupported".to_string());
    }
    let supported_order_id_types =
        json_string_array(adapter_capability, "supported_order_id_types");
    if !supported_order_id_types
        .iter()
        .any(|order_id_type| order_id_type == &opt.order_id_type)
    {
        issues.push("adapter_order_id_type_unsupported".to_string());
    }
    for (field, issue) in [
        (
            "bulk_cancel_supported",
            "adapter_bulk_cancel_supported_forbidden",
        ),
        (
            "cancel_all_supported",
            "adapter_cancel_all_supported_forbidden",
        ),
        ("retry_supported", "adapter_retry_supported_forbidden"),
        (
            "automatic_cancel_supported",
            "adapter_automatic_cancel_supported_forbidden",
        ),
        (
            "multi_venue_supported",
            "adapter_multi_venue_supported_forbidden",
        ),
    ] {
        if json_bool_value(adapter_capability, field).unwrap_or(true) {
            issues.push(issue.to_string());
        }
    }
    issues
}

fn actual_cancel_single_shot_release_manifest_issues(
    release_manifest: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelSingleShotOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(release_manifest, "schema_version").as_deref()
        != Some("ntpro.v181_release_manifest.v1")
    {
        issues.push("release_manifest_schema_mismatch".to_string());
    }
    if json_string_value(release_manifest, "product_version").as_deref() != Some("v0.18.1") {
        issues.push("release_manifest_product_version_mismatch".to_string());
    }
    if json_pointer_string_value(release_manifest, "/patch_release/planned_tag").as_deref()
        != Some(opt.expected_release_tag.as_str())
    {
        issues.push("release_manifest_planned_tag_mismatch".to_string());
    }
    if json_pointer_string_value(release_manifest, "/capability/actual_cancel_scope").as_deref()
        != Some("not_included")
    {
        issues.push("release_manifest_actual_cancel_scope_mismatch".to_string());
    }
    issues
}

fn production_mutation_actual_cancel_single_shot_source_issues(
    cancel_risk_gate: &serde_json::Value,
    owner_approval_lifecycle: &serde_json::Value,
    adapter_boundary: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelSingleShotOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(cancel_risk_gate, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION)
    {
        issues.push("cancel_risk_gate_schema_mismatch".to_string());
    }
    if !json_bool_value(cancel_risk_gate, "cancel_risk_gate_ready").unwrap_or(false) {
        issues.push("risk_gate_not_passed".to_string());
    }
    if !json_bool_value(cancel_risk_gate, "new_orders_blocked").unwrap_or(false) {
        issues.push("risk_gate_new_orders_not_blocked".to_string());
    }
    if json_string_value(cancel_risk_gate, "order_lineage_id").as_deref()
        != Some(opt.expected_order_lineage_id.as_str())
    {
        issues.push("order_lineage_id_mismatch".to_string());
    }
    if json_scalar_string_value(cancel_risk_gate, "symbol").as_deref()
        != Some(opt.expected_symbol.as_str())
    {
        issues.push("symbol_mismatch".to_string());
    }
    if json_scalar_string_value(cancel_risk_gate, "account_label").as_deref()
        != Some(opt.expected_account_label.as_str())
    {
        issues.push("account_label_mismatch".to_string());
    }

    if json_string_value(owner_approval_lifecycle, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION)
    {
        issues.push("owner_approval_lifecycle_schema_mismatch".to_string());
    }
    if !json_bool_value(owner_approval_lifecycle, "approval_execution_authorized").unwrap_or(false)
    {
        issues.push("missing_owner_approval".to_string());
    }
    if json_bool_value(owner_approval_lifecycle, "approval_consumed").unwrap_or(false)
        || json_bool_value(owner_approval_lifecycle, "approval_used").unwrap_or(false)
    {
        issues.push("owner_approval_reused".to_string());
    }
    if json_string_value(owner_approval_lifecycle, "order_lineage_id").as_deref()
        != Some(opt.expected_order_lineage_id.as_str())
    {
        issues.push("owner_approval_scope_mismatch".to_string());
    }
    if json_string_value(owner_approval_lifecycle, "venue").as_deref() != Some(opt.venue.as_str()) {
        issues.push("venue_mismatch".to_string());
    }

    if json_string_value(adapter_boundary, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_ACTUAL_CANCEL_EXECUTOR_ADAPTER_BOUNDARY_SCHEMA_VERSION)
    {
        issues.push("adapter_boundary_schema_mismatch".to_string());
    }
    if !json_bool_value(adapter_boundary, "adapter_boundary_ready").unwrap_or(false) {
        issues.push("adapter_capability_missing".to_string());
    }
    if !json_bool_value(
        adapter_boundary,
        "actual_cancel_send_allowed_by_adapter_boundary",
    )
    .unwrap_or(false)
    {
        issues.push("adapter_boundary_not_authorized".to_string());
    }
    if json_string_value(adapter_boundary, "order_lineage_id").as_deref()
        != Some(opt.expected_order_lineage_id.as_str())
    {
        issues.push("adapter_boundary_order_lineage_mismatch".to_string());
    }
    if json_string_value(adapter_boundary, "venue").as_deref() != Some(opt.venue.as_str()) {
        issues.push("adapter_boundary_venue_mismatch".to_string());
    }
    if json_string_value(adapter_boundary, "order_id_type").as_deref()
        != Some(opt.order_id_type.as_str())
    {
        issues.push("adapter_boundary_order_id_type_mismatch".to_string());
    }
    for (field, issue) in [
        ("bulk_cancel_allowed", "bulk_cancel_requested"),
        ("cancel_all_allowed", "bulk_cancel_requested"),
        ("retry_attempted", "retry_or_repair_requested"),
        ("automatic_cancel_allowed", "automatic_cancel_requested"),
        (
            "dashboard_execution_allowed",
            "dashboard_operation_requested",
        ),
        (
            "dashboard_cancel_controls_enabled",
            "dashboard_operation_requested",
        ),
    ] {
        if json_bool_value(adapter_boundary, field).unwrap_or(false) {
            issues.push(issue.to_string());
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

fn production_mutation_actual_cancel_single_shot_adapter_capability_issues(
    adapter_capability: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelSingleShotOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(adapter_capability, "schema_version").as_deref()
        != Some("ntpro.v190_actual_cancel_adapter_capability.v1")
    {
        issues.push("adapter_capability_schema_mismatch".to_string());
    }
    if json_string_value(adapter_capability, "artifact_type").as_deref()
        != Some("actual_cancel_adapter_capability")
    {
        issues.push("adapter_capability_artifact_type_mismatch".to_string());
    }
    if !json_bool_value(adapter_capability, "actual_cancel_supported").unwrap_or(false) {
        issues.push("adapter_capability_missing".to_string());
    }
    let supported_venues = json_string_array(adapter_capability, "supported_venues");
    if !supported_venues.iter().any(|venue| venue == &opt.venue) {
        issues.push("venue_mismatch".to_string());
    }
    let supported_order_id_types =
        json_string_array(adapter_capability, "supported_order_id_types");
    if !supported_order_id_types
        .iter()
        .any(|order_id_type| order_id_type == &opt.order_id_type)
    {
        issues.push("order_identity_mismatch".to_string());
    }
    for (field, issue) in [
        ("bulk_cancel_supported", "bulk_cancel_requested"),
        ("cancel_all_supported", "bulk_cancel_requested"),
        ("retry_supported", "retry_or_repair_requested"),
        ("automatic_cancel_supported", "automatic_cancel_requested"),
        ("multi_venue_supported", "scope_expansion_requested"),
    ] {
        if json_bool_value(adapter_capability, field).unwrap_or(true) {
            issues.push(issue.to_string());
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

fn production_mutation_actual_cancel_readback_reconciliation_source_issues(
    actual_cancel_attempt: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelReadbackReconciliationOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(actual_cancel_attempt, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_SCHEMA_VERSION)
    {
        issues.push("actual_cancel_attempt_schema_mismatch".to_string());
    }
    if json_string_value(actual_cancel_attempt, "artifact_type").as_deref()
        != Some("actual_cancel_single_shot")
    {
        issues.push("actual_cancel_attempt_artifact_type_mismatch".to_string());
    }
    if json_string_value(actual_cancel_attempt, "status").as_deref()
        != Some("actual_cancel_attempt_recorded")
    {
        let status = json_string_value(actual_cancel_attempt, "status")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("actual_cancel_attempt_status_{status}"));
    }
    if json_string_value(actual_cancel_attempt, "order_lineage_id").as_deref()
        != Some(opt.expected_order_lineage_id.as_str())
    {
        issues.push("actual_cancel_attempt_order_lineage_mismatch".to_string());
    }
    if json_scalar_string_value(actual_cancel_attempt, "symbol").as_deref()
        != Some(opt.expected_symbol.as_str())
    {
        issues.push("actual_cancel_attempt_symbol_mismatch".to_string());
    }
    if json_scalar_string_value(actual_cancel_attempt, "account_label").as_deref()
        != Some(opt.expected_account_label.as_str())
    {
        issues.push("actual_cancel_attempt_account_label_mismatch".to_string());
    }
    if json_string_value(actual_cancel_attempt, "venue").as_deref() != Some(opt.venue.as_str()) {
        issues.push("actual_cancel_attempt_venue_mismatch".to_string());
    }
    for field in [
        "actual_cancel_command_ready",
        "single_shot_cancel_allowed",
        "owner_approval_ready",
        "risk_gate_ready",
        "release_provenance_ready",
        "adapter_boundary_ready",
        "adapter_capability_ready",
        "approval_consumed_before_send",
        "approval_consumed_after_send",
        "request_sent",
        "cancel_attempted",
        "http_send_attempted",
        "readback_required",
    ] {
        if !json_bool_value(actual_cancel_attempt, field).unwrap_or(false) {
            issues.push(format!("actual_cancel_attempt_{field}_not_true"));
        }
    }
    if json_u64_value(actual_cancel_attempt, "cancel_requests_sent").unwrap_or(0) != 1 {
        issues.push("actual_cancel_attempt_cancel_requests_sent_not_one".to_string());
    }
    if json_u64_value(
        actual_cancel_attempt,
        "production_order_mutations_attempted",
    )
    .unwrap_or(0)
        != 1
    {
        issues.push("actual_cancel_attempt_mutations_attempted_not_one".to_string());
    }
    if json_string_value(actual_cancel_attempt, "readback_requirement").as_deref()
        != Some("post_cancel_readback_required_before_any_retry_or_followup")
    {
        issues.push("actual_cancel_attempt_readback_requirement_mismatch".to_string());
    }
    for field in [
        "source_artifact_issues",
        "adapter_capability_issues",
        "safety_contract_issues",
        "release_manifest_issues",
        "missing_cli_flags",
        "missing_env_vars",
    ] {
        if json_array_has_items(actual_cancel_attempt, field) {
            issues.push(format!("actual_cancel_attempt_{field}_not_empty"));
        }
    }
    append_true_marker_issues(
        &mut issues,
        "actual_cancel_attempt",
        actual_cancel_attempt,
        &[
            "retry_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
            "remediation_attempted",
            "automatic_cancel_allowed",
            "automatic_remediation_allowed",
            "bulk_cancel_allowed",
            "cancel_all_allowed",
            "multi_account_cancel_allowed",
            "multi_strategy_cancel_allowed",
            "multi_venue_cancel_allowed",
            "dashboard_order_controls_enabled",
            "dashboard_cancel_controls_enabled",
            "dashboard_execution_allowed",
            "api_key_value_recorded",
            "api_secret_value_recorded",
            "api_key_header_value_recorded",
            "signature_recorded",
            "signed_query_recorded",
            "signed_url_recorded",
            "request_body_recorded",
            "raw_request_body_recorded",
            "raw_exchange_response_recorded",
            "response_body_recorded",
            "response_headers_recorded",
        ],
    );

    issues.sort();
    issues.dedup();
    issues
}

fn production_mutation_actual_cancel_readback_reconciliation_lineage_issues(
    actual_cancel_attempt: &serde_json::Value,
    readback: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelReadbackReconciliationOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_scalar_string_value(readback, "symbol").as_deref() != Some(opt.expected_symbol.as_str())
    {
        issues.push("readback_symbol_mismatch".to_string());
    }

    let readback_order_id_ref = redact_cancel_preview_identifier(
        "readback_order_id",
        &json_scalar_string_value(readback, "orderId").unwrap_or_else(|| "missing".to_string()),
    );
    let readback_client_order_id_ref = redact_cancel_preview_identifier(
        "readback_client_order_id",
        &json_scalar_string_value(readback, "clientOrderId")
            .unwrap_or_else(|| "missing".to_string()),
    );
    let readback_orig_client_order_id_ref = redact_cancel_preview_identifier(
        "readback_orig_client_order_id",
        &json_scalar_string_value(readback, "origClientOrderId")
            .unwrap_or_else(|| "missing".to_string()),
    );
    let known_order_id = json_scalar_string_value(actual_cancel_attempt, "known_order_id")
        .unwrap_or_else(|| "missing".to_string());
    let known_client_order_id =
        json_scalar_string_value(actual_cancel_attempt, "known_client_order_id")
            .unwrap_or_else(|| "missing".to_string());
    let cancel_order_identifier_ref =
        json_scalar_string_value(actual_cancel_attempt, "cancel_order_identifier_ref")
            .unwrap_or_else(|| "missing".to_string());
    let readback_timeout =
        production_mutation_actual_cancel_readback_reconciliation_timeout(readback);
    let order_identity_matches =
        redacted_cancel_identifiers_match(&readback_order_id_ref, &known_order_id)
            || redacted_cancel_identifiers_match(
                &readback_order_id_ref,
                &cancel_order_identifier_ref,
            )
            || redacted_cancel_identifiers_match(
                &readback_client_order_id_ref,
                &known_client_order_id,
            )
            || redacted_cancel_identifiers_match(
                &readback_orig_client_order_id_ref,
                &known_client_order_id,
            )
            || redacted_cancel_identifiers_match(
                &readback_orig_client_order_id_ref,
                &cancel_order_identifier_ref,
            );
    if !order_identity_matches && !readback_timeout {
        issues.push("readback_order_identity_mismatch".to_string());
    }

    issues.sort();
    issues.dedup();
    issues
}

fn production_mutation_actual_cancel_failure_evidence_source_issues(
    reconciliation: &serde_json::Value,
    request_ref: &serde_json::Value,
    response_ref: &serde_json::Value,
    readback_ref: &serde_json::Value,
    audit_ref: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(reconciliation, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_SCHEMA_VERSION)
    {
        issues.push("readback_reconciliation_schema_mismatch".to_string());
    }
    if json_string_value(reconciliation, "artifact_type").as_deref()
        != Some("actual_cancel_readback_reconciliation")
    {
        issues.push("readback_reconciliation_artifact_type_mismatch".to_string());
    }
    if !json_bool_value(reconciliation, "reconciliation_ready").unwrap_or(false) {
        issues.push("readback_reconciliation_ready_false".to_string());
    }
    if !json_bool_value(reconciliation, "dashboard_read_only_consumable").unwrap_or(false) {
        issues.push("readback_reconciliation_dashboard_consumable_false".to_string());
    }
    if json_string_value(reconciliation, "readback_result").is_none() {
        issues.push("readback_reconciliation_missing_readback_result".to_string());
    }
    for (label, artifact) in [
        ("request_ref", request_ref),
        ("response_ref", response_ref),
        ("readback_ref", readback_ref),
        ("audit_ref", audit_ref),
    ] {
        if !artifact.is_object() {
            issues.push(format!("{label}_not_object"));
        }
        if json_string_value(artifact, "artifact_type").is_none() {
            issues.push(format!("{label}_missing_artifact_type"));
        }
        if json_string_value(artifact, "status").is_none() {
            issues.push(format!("{label}_missing_status"));
        }
        for marker in production_mutation_response_forbidden_markers(artifact) {
            issues.push(format!("{label}_forbidden_marker_{marker}"));
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

fn production_mutation_actual_cancel_failure_evidence_lineage_issues(
    reconciliation: &serde_json::Value,
    opt: &LiveProductionMutationActualCancelFailureEvidenceOpt,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(reconciliation, "order_lineage_id").as_deref()
        != Some(opt.expected_order_lineage_id.as_str())
    {
        issues.push("readback_reconciliation_order_lineage_mismatch".to_string());
    }
    if json_scalar_string_value(reconciliation, "symbol").as_deref()
        != Some(opt.expected_symbol.as_str())
    {
        issues.push("readback_reconciliation_symbol_mismatch".to_string());
    }
    if json_scalar_string_value(reconciliation, "account_label").as_deref()
        != Some(opt.expected_account_label.as_str())
    {
        issues.push("readback_reconciliation_account_label_mismatch".to_string());
    }
    if json_string_value(reconciliation, "venue").as_deref() != Some(opt.venue.as_str()) {
        issues.push("readback_reconciliation_venue_mismatch".to_string());
    }
    issues.sort();
    issues.dedup();
    issues
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProductionMutationActualCancelFailureEvidenceDecision {
    artifact_status: &'static str,
    cancel_outcome: &'static str,
    outcome_category: &'static str,
    failure_mode: &'static str,
    partial_success_mode: &'static str,
    operator_action: &'static str,
    residual_risk_state: &'static str,
    operator_action_required: bool,
    recovered: bool,
    degraded: bool,
    failed: bool,
    partial_success: bool,
    residual_risk_visible: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    risk_halted: bool,
}

fn classify_production_mutation_actual_cancel_failure_evidence(
    reconciliation: &serde_json::Value,
    request_ref: &serde_json::Value,
    response_ref: &serde_json::Value,
    readback_ref: &serde_json::Value,
    audit_ref: &serde_json::Value,
) -> ProductionMutationActualCancelFailureEvidenceDecision {
    let refs = [request_ref, response_ref, readback_ref, audit_ref];
    if production_mutation_actual_cancel_failure_evidence_has_marker(
        &refs,
        &[
            "adapterFailure",
            "adapter_failure",
            "adapter_failure_observed",
            "adapter_failure_mode",
            "failure_mode",
            "error_code",
            "diagnostic",
        ],
        &["adapter_failure", "adapter failure", "adapter-unavailable"],
    ) {
        return actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_adapter_failure",
            "adapter_failure",
            "failed",
            "adapter_failure",
            "none",
            "operator_review_adapter_failure_before_any_followup",
            "adapter_failure_manual_review",
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        );
    }
    if production_mutation_actual_cancel_failure_evidence_has_marker(
        &refs,
        &[
            "venueUnavailable",
            "venue_unavailable",
            "venue_disposition",
            "venueDisposition",
            "failure_mode",
            "error_code",
            "diagnostic",
        ],
        &[
            "venue_unavailable",
            "venue unavailable",
            "exchange_unavailable",
        ],
    ) {
        return actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_venue_unavailable",
            "venue_unavailable",
            "failed",
            "venue_unavailable",
            "none",
            "operator_confirm_venue_state_before_any_followup",
            "venue_unavailable_manual_review",
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        );
    }

    let readback_result = json_string_value(reconciliation, "readback_result")
        .unwrap_or_else(|| "unknown".to_string());
    let readback_state = json_string_value(reconciliation, "readback_state")
        .unwrap_or_else(|| "unknown".to_string())
        .to_ascii_uppercase();
    let order_status = json_string_value(reconciliation, "order_status")
        .unwrap_or_else(|| "unknown".to_string())
        .to_ascii_lowercase();
    let partial_fill_observed =
        json_bool_value(reconciliation, "partial_fill_observed").unwrap_or(false);
    if readback_result == "rejected" || readback_state == "REJECTED" || order_status == "rejected" {
        return actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_rejected",
            "rejected",
            "failed",
            "rejected",
            "none",
            "operator_review_rejection_and_exchange_state",
            "rejected_manual_review",
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        );
    }

    match readback_result.as_str() {
        "cancel_confirmed" => actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_recovered_cancel_confirmed",
            "cancel_confirmed",
            "recovered",
            "none",
            "none",
            "no_operator_action_required_cancel_confirmed",
            "none",
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ),
        "already_cancelled" => actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_recovered_already_cancelled",
            "already_cancelled",
            "recovered",
            "none",
            "idempotent_terminal_already_cancelled",
            "no_operator_action_required_already_cancelled",
            "none",
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ),
        "filled_before_cancel" => actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_partial_success_filled_before_cancel",
            "filled_before_cancel",
            "partial_success",
            "none",
            "filled_before_cancel",
            "operator_review_filled_position_and_residual_risk",
            "filled_position_review_required",
            true,
            false,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        "timeout" => actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_timeout",
            "timeout",
            "failed",
            "timeout",
            "none",
            "operator_confirm_exchange_state_after_timeout",
            "timeout_manual_review",
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        "unknown" => actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_unknown",
            "unknown",
            "failed",
            "unknown_state",
            "none",
            "operator_confirm_unknown_exchange_state",
            "unknown_manual_review",
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        "inconsistent" if partial_fill_observed => actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_partial_success_partial_fill",
            "partial_fill",
            "partial_success",
            "none",
            "partial_fill",
            "operator_review_partial_fill_residual_risk",
            "partial_fill_residual_risk_manual_review",
            true,
            false,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        "inconsistent" => actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_inconsistent",
            "inconsistent",
            "failed",
            "inconsistent_readback",
            "none",
            "operator_reconcile_inconsistent_exchange_state",
            "source",
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        _ => actual_cancel_failure_evidence_decision(
            "ready_actual_cancel_failure_unknown",
            "unknown",
            "failed",
            "unknown_state",
            "none",
            "operator_confirm_unknown_exchange_state",
            "unknown_manual_review",
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn actual_cancel_failure_evidence_decision(
    artifact_status: &'static str,
    cancel_outcome: &'static str,
    outcome_category: &'static str,
    failure_mode: &'static str,
    partial_success_mode: &'static str,
    operator_action: &'static str,
    residual_risk_state: &'static str,
    operator_action_required: bool,
    recovered: bool,
    degraded: bool,
    failed: bool,
    partial_success: bool,
    residual_risk_visible: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    risk_halted: bool,
) -> ProductionMutationActualCancelFailureEvidenceDecision {
    ProductionMutationActualCancelFailureEvidenceDecision {
        artifact_status,
        cancel_outcome,
        outcome_category,
        failure_mode,
        partial_success_mode,
        operator_action,
        residual_risk_state,
        operator_action_required,
        recovered,
        degraded,
        failed,
        partial_success,
        residual_risk_visible,
        manual_review_required,
        new_orders_blocked,
        risk_halted,
    }
}

fn production_mutation_actual_cancel_failure_evidence_has_marker(
    artifacts: &[&serde_json::Value],
    fields: &[&str],
    markers: &[&str],
) -> bool {
    artifacts.iter().any(|artifact| {
        fields.iter().any(|field| {
            json_bool_value(artifact, field).unwrap_or(false)
                || json_scalar_string_value(artifact, field).is_some_and(|value| {
                    let normalized = value.trim().to_ascii_lowercase();
                    markers.iter().any(|marker| normalized.contains(marker))
                })
        })
    })
}

fn actual_cancel_single_shot_order_identifier(
    opt: &LiveProductionMutationActualCancelSingleShotOpt,
    owner_approval_lifecycle: &serde_json::Value,
) -> (String, Option<String>, String) {
    let (raw, source, kind, missing_issue) = if opt.order_id_type == "exchange_order_id" {
        (
            opt.cancel_order_id.as_deref(),
            json_scalar_string_value(owner_approval_lifecycle, "known_order_id")
                .unwrap_or_else(|| "missing".to_string()),
            "order_id",
            "cancel_order_id_missing",
        )
    } else {
        (
            opt.cancel_orig_client_order_id.as_deref(),
            json_scalar_string_value(owner_approval_lifecycle, "known_client_order_id")
                .unwrap_or_else(|| "missing".to_string()),
            "client_order_id",
            "cancel_orig_client_order_id_missing",
        )
    };
    let raw = raw.map(str::trim).unwrap_or_default();
    if raw.is_empty() {
        return (
            "missing".to_string(),
            Some(missing_issue.to_string()),
            String::new(),
        );
    }
    let redacted = redact_cancel_preview_identifier(kind, raw);
    let issue = if source == raw || source == redacted {
        None
    } else {
        Some("order_identity_mismatch".to_string())
    };
    (redacted, issue, raw.to_string())
}

fn production_mutation_cancel_response_redaction_source_issues(
    approval_lifecycle: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(approval_lifecycle, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION)
    {
        issues.push("manual_owner_approval_lifecycle_schema_mismatch".to_string());
    }
    if json_string_value(approval_lifecycle, "artifact_type").as_deref()
        != Some("manual_owner_approval_lifecycle")
    {
        issues.push("manual_owner_approval_lifecycle_artifact_type_mismatch".to_string());
    }
    if json_string_value(approval_lifecycle, "status").as_deref()
        != Some("approval_lifecycle_recorded_for_cancel_candidate")
    {
        let status = json_string_value(approval_lifecycle, "status")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("manual_owner_approval_lifecycle_status_{status}"));
    }
    for field in [
        "approval_lifecycle_valid",
        "manual_approval_recorded",
        "owner_approval_required",
        "owner_approval_lifecycle_recorded",
        "one_time_approval",
        "approval_expires",
    ] {
        if !json_bool_value(approval_lifecycle, field).unwrap_or(false) {
            issues.push(format!("manual_owner_approval_lifecycle_{field}_not_true"));
        }
    }
    if json_string_value(approval_lifecycle, "approval_state").as_deref() != Some("approved") {
        let state = json_string_value(approval_lifecycle, "approval_state")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("manual_owner_approval_lifecycle_state_{state}"));
    }
    if json_string_value(approval_lifecycle, "approval_scope").as_deref()
        != Some("one_order_cancel_candidate")
    {
        issues.push("manual_owner_approval_lifecycle_scope_mismatch".to_string());
    }
    if json_string_value(approval_lifecycle, "lineage_scope").as_deref()
        != Some("single_v16_mutation_candidate")
    {
        issues.push("manual_owner_approval_lifecycle_lineage_scope_mismatch".to_string());
    }
    let order_lineage_id = json_string_value(approval_lifecycle, "order_lineage_id")
        .unwrap_or_else(|| "missing".into());
    if !cancel_preview_identifier_known(&order_lineage_id) {
        issues.push("order_lineage_id_missing".to_string());
    }
    if json_u64_value(approval_lifecycle, "candidate_count").unwrap_or(0) != 1 {
        issues.push("manual_owner_approval_lifecycle_candidate_count_not_one".to_string());
    }
    for field in [
        "source_artifact_issues",
        "missing_cli_flags",
        "lifecycle_issues",
    ] {
        if json_array_has_items(approval_lifecycle, field) {
            issues.push(format!("manual_owner_approval_lifecycle_{field}_not_empty"));
        }
    }

    append_true_marker_issues(
        &mut issues,
        "manual_owner_approval_lifecycle",
        approval_lifecycle,
        &[
            "approval_expired",
            "approval_revoked",
            "approval_used",
            "approval_reusable",
            "approval_consumed",
            "approval_consumed_before_send",
            "approval_consumed_after_send",
            "strategy_auto_approval_allowed",
            "strategy_auto_approval_attempted",
            "background_auto_approval_allowed",
            "background_auto_approval_attempted",
            "dashboard_auto_approval_allowed",
            "dashboard_auto_approval_attempted",
            "incident_handler_auto_approval_allowed",
            "incident_handler_auto_approval_attempted",
            "api_key_value_recorded",
            "api_secret_value_recorded",
            "api_key_header_value_recorded",
            "signature_recorded",
            "signed_query_recorded",
            "signed_url_recorded",
            "raw_exchange_response_recorded",
            "response_body_recorded",
            "response_headers_recorded",
            "network_attempted",
            "network_cancel_endpoint_attempted",
            "actual_cancel_send_allowed",
            "cancel_attempted",
            "retry_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
            "remediation_attempted",
            "automatic_cancel_allowed",
            "automatic_remediation_allowed",
            "production_order_mutation_allowed",
            "dashboard_order_controls_enabled",
            "dashboard_cancel_controls_enabled",
        ],
    );
    append_nonzero_marker_issues(
        &mut issues,
        "manual_owner_approval_lifecycle",
        approval_lifecycle,
        &[
            "cancel_requests_sent",
            "production_order_mutations_attempted",
        ],
    );

    issues
}

fn production_mutation_post_cancel_readback_source_issues(
    cancel_response_redaction: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(cancel_response_redaction, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION)
    {
        issues.push("cancel_response_redaction_schema_mismatch".to_string());
    }
    if json_string_value(cancel_response_redaction, "artifact_type").as_deref()
        != Some("cancel_response_redaction")
    {
        issues.push("cancel_response_redaction_artifact_type_mismatch".to_string());
    }
    if json_string_value(cancel_response_redaction, "status").as_deref()
        != Some("ready_cancel_response_redacted")
    {
        let status = json_string_value(cancel_response_redaction, "status")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("cancel_response_redaction_status_{status}"));
    }
    for field in [
        "response_redaction_ready",
        "cancel_response_redacted",
        "response_shape_validated",
        "approval_lifecycle_valid",
        "manual_approval_recorded",
    ] {
        if !json_bool_value(cancel_response_redaction, field).unwrap_or(false) {
            issues.push(format!("cancel_response_redaction_{field}_not_true"));
        }
    }
    if json_string_value(cancel_response_redaction, "lineage_scope").as_deref()
        != Some("single_v16_mutation_candidate")
    {
        issues.push("cancel_response_redaction_lineage_scope_mismatch".to_string());
    }
    let order_lineage_id = json_string_value(cancel_response_redaction, "order_lineage_id")
        .unwrap_or_else(|| "missing".into());
    if !cancel_preview_identifier_known(&order_lineage_id) {
        issues.push("order_lineage_id_missing".to_string());
    }
    for field in [
        "source_artifact_issues",
        "missing_cli_flags",
        "forbidden_response_markers",
    ] {
        if json_array_has_items(cancel_response_redaction, field) {
            issues.push(format!("cancel_response_redaction_{field}_not_empty"));
        }
    }

    append_true_marker_issues(
        &mut issues,
        "cancel_response_redaction",
        cancel_response_redaction,
        &[
            "approval_consumed",
            "api_key_value_recorded",
            "api_secret_value_recorded",
            "api_key_header_value_recorded",
            "signature_recorded",
            "signed_query_recorded",
            "signed_url_recorded",
            "request_body_recorded",
            "raw_request_body_recorded",
            "raw_exchange_response_recorded",
            "response_body_recorded",
            "response_headers_recorded",
            "unrestricted_payload_recorded",
            "account_balances_recorded",
            "fills_recorded",
            "network_attempted",
            "network_cancel_endpoint_attempted",
            "actual_cancel_send_allowed",
            "cancel_attempted",
            "retry_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
            "remediation_attempted",
            "automatic_cancel_allowed",
            "automatic_remediation_allowed",
            "production_order_mutation_allowed",
            "dashboard_order_controls_enabled",
            "dashboard_cancel_controls_enabled",
        ],
    );
    append_nonzero_marker_issues(
        &mut issues,
        "cancel_response_redaction",
        cancel_response_redaction,
        &[
            "cancel_requests_sent",
            "production_order_mutations_attempted",
        ],
    );

    issues
}

fn production_mutation_cancel_recovery_incident_audit_closeout_source_issues(
    cancel_risk_gate: &serde_json::Value,
    approval_lifecycle: &serde_json::Value,
    cancel_response_redaction: &serde_json::Value,
    post_cancel_readback: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    issues.extend(
        production_mutation_manual_owner_approval_lifecycle_source_issues(cancel_risk_gate)
            .into_iter()
            .map(|issue| format!("source_cancel_risk_gate_{issue}")),
    );
    issues.extend(
        production_mutation_cancel_response_redaction_source_issues(approval_lifecycle)
            .into_iter()
            .map(|issue| format!("source_manual_owner_approval_lifecycle_{issue}")),
    );
    issues.extend(
        production_mutation_post_cancel_readback_source_issues(cancel_response_redaction)
            .into_iter()
            .map(|issue| format!("source_cancel_response_redaction_{issue}")),
    );
    issues.extend(
        production_mutation_cancel_recovery_post_cancel_readback_source_issues(
            post_cancel_readback,
        )
        .into_iter()
        .map(|issue| format!("source_post_cancel_readback_{issue}")),
    );
    issues
}

fn production_mutation_cancel_recovery_post_cancel_readback_source_issues(
    post_cancel_readback: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    if json_string_value(post_cancel_readback, "schema_version").as_deref()
        != Some(PRODUCTION_MUTATION_POST_CANCEL_READBACK_SCHEMA_VERSION)
    {
        issues.push("post_cancel_readback_schema_mismatch".to_string());
    }
    if json_string_value(post_cancel_readback, "artifact_type").as_deref()
        != Some("post_cancel_readback")
    {
        issues.push("post_cancel_readback_artifact_type_mismatch".to_string());
    }
    if json_string_value(post_cancel_readback, "status").as_deref()
        != Some("ready_post_cancel_readback_classified")
    {
        let status = json_string_value(post_cancel_readback, "status")
            .unwrap_or_else(|| "unknown".to_string());
        issues.push(format!("post_cancel_readback_status_{status}"));
    }
    for field in [
        "cancel_response_redaction_ready",
        "cancel_response_redacted",
        "post_cancel_readback_ready",
        "post_cancel_readback_classified",
        "redacted_metadata_only",
        "order_lineage_preserved",
    ] {
        if !json_bool_value(post_cancel_readback, field).unwrap_or(false) {
            issues.push(format!("post_cancel_readback_{field}_not_true"));
        }
    }
    if json_string_value(post_cancel_readback, "lineage_scope").as_deref()
        != Some("single_v16_mutation_candidate")
    {
        issues.push("post_cancel_readback_lineage_scope_mismatch".to_string());
    }
    let order_lineage_id = json_string_value(post_cancel_readback, "order_lineage_id")
        .unwrap_or_else(|| "missing".into());
    if !cancel_preview_identifier_known(&order_lineage_id) {
        issues.push("order_lineage_id_missing".to_string());
    }
    for field in [
        "source_artifact_issues",
        "missing_cli_flags",
        "forbidden_readback_markers",
        "unsupported_readback_states",
    ] {
        if json_array_has_items(post_cancel_readback, field) {
            issues.push(format!("post_cancel_readback_{field}_not_empty"));
        }
    }

    append_true_marker_issues(
        &mut issues,
        "post_cancel_readback",
        post_cancel_readback,
        &[
            "api_key_value_recorded",
            "api_secret_value_recorded",
            "api_key_header_value_recorded",
            "signature_recorded",
            "signed_query_recorded",
            "signed_url_recorded",
            "raw_exchange_response_recorded",
            "raw_readback_body_recorded",
            "response_body_recorded",
            "response_headers_recorded",
            "unrestricted_payload_recorded",
            "account_balances_recorded",
            "fills_recorded",
            "readback_execution_attempted",
            "order_state_read_attempted",
            "network_attempted",
            "network_readback_endpoint_attempted",
            "network_cancel_endpoint_attempted",
            "actual_cancel_send_allowed",
            "cancel_attempted",
            "retry_attempted",
            "replace_attempted",
            "amend_attempted",
            "flatten_attempted",
            "remediation_attempted",
            "automatic_cancel_allowed",
            "automatic_remediation_allowed",
            "production_order_mutation_allowed",
            "dashboard_order_controls_enabled",
            "dashboard_cancel_controls_enabled",
        ],
    );
    append_nonzero_marker_issues(
        &mut issues,
        "post_cancel_readback",
        post_cancel_readback,
        &[
            "production_order_state_reads_attempted",
            "cancel_requests_sent",
            "production_order_mutations_attempted",
        ],
    );

    issues
}

fn production_mutation_cancel_recovery_incident_audit_closeout_lineage_issues(
    cancel_risk_gate: &serde_json::Value,
    approval_lifecycle: &serde_json::Value,
    cancel_response_redaction: &serde_json::Value,
    post_cancel_readback: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    let lineage_fields = [
        ("cancel_risk_gate", cancel_risk_gate),
        ("manual_owner_approval_lifecycle", approval_lifecycle),
        ("cancel_response_redaction", cancel_response_redaction),
        ("post_cancel_readback", post_cancel_readback),
    ];
    let mut expected_lineage: Option<String> = None;
    for (label, artifact) in lineage_fields {
        let order_lineage_id =
            json_string_value(artifact, "order_lineage_id").unwrap_or_else(|| "missing".into());
        if !cancel_preview_identifier_known(&order_lineage_id) {
            issues.push(format!("{label}_order_lineage_id_missing"));
        } else if let Some(expected) = &expected_lineage {
            if expected != &order_lineage_id {
                issues.push(format!("{label}_order_lineage_id_mismatch"));
            }
        } else {
            expected_lineage = Some(order_lineage_id);
        }
        if json_string_value(artifact, "lineage_scope").as_deref()
            != Some("single_v16_mutation_candidate")
        {
            issues.push(format!("{label}_lineage_scope_mismatch"));
        }
    }

    for (field, issue) in [
        ("symbol", "symbol_mismatch"),
        ("account_label", "account_label_mismatch"),
        ("known_order_id", "known_order_id_mismatch"),
        ("known_client_order_id", "known_client_order_id_mismatch"),
    ] {
        let expected = json_scalar_string_value(cancel_risk_gate, field)
            .unwrap_or_else(|| "missing".to_string());
        for (label, artifact) in [
            ("manual_owner_approval_lifecycle", approval_lifecycle),
            ("cancel_response_redaction", cancel_response_redaction),
            ("post_cancel_readback", post_cancel_readback),
        ] {
            let observed =
                json_scalar_string_value(artifact, field).unwrap_or_else(|| "missing".to_string());
            if observed != expected {
                issues.push(format!("{label}_{issue}"));
            }
        }
    }

    issues
}

fn cancel_recovery_needed_reason(cancel_risk_gate: &serde_json::Value) -> &'static str {
    if json_bool_value(cancel_risk_gate, "orphan_risk_detected").unwrap_or(false) {
        "orphan_risk_detected"
    } else if json_bool_value(cancel_risk_gate, "manual_review_required").unwrap_or(false)
        && json_bool_value(cancel_risk_gate, "risk_halted").unwrap_or(false)
    {
        "manual_review_required_risk_halt"
    } else {
        "unknown_recovery_reason"
    }
}

fn cancel_recovery_terminal_action_recommendation(readback_state_class: &str) -> &'static str {
    match readback_state_class {
        "terminal_canceled" => "close_incident_cancel_confirmed",
        "terminal_filled" => "manual_position_and_fill_review",
        "terminal_rejected" => "close_incident_rejected_no_retry",
        "terminal_expired" => "close_incident_expired_no_retry",
        "ambiguous_missing" => "manual_exchange_and_local_ledger_review",
        "ambiguous_unknown" => "manual_exchange_state_review",
        _ => "manual_review_unsupported_readback_state",
    }
}

fn cancel_recovery_remaining_risk(readback_state_class: &str) -> (&'static str, bool) {
    match readback_state_class {
        "terminal_canceled" => ("none_cancel_confirmed", false),
        "terminal_filled" => ("fill_or_position_risk_requires_review", true),
        "terminal_rejected" => ("none_terminal_rejected_no_retry", false),
        "terminal_expired" => ("none_terminal_expired_no_retry", false),
        "ambiguous_missing" => ("exchange_state_missing_manual_review_required", true),
        "ambiguous_unknown" => ("exchange_state_unknown_manual_review_required", true),
        _ => ("unsupported_readback_state_manual_review_required", true),
    }
}

fn source_ref_path(
    source_path: &Path,
    artifact: &serde_json::Value,
    ref_field: &str,
    label: &str,
) -> anyhow::Result<PathBuf> {
    let ref_path = artifact
        .get(ref_field)
        .and_then(|source_ref| source_ref.get("path"))
        .and_then(serde_json::Value::as_str)
        .filter(|path| !path.trim().is_empty())
        .with_context(|| format!("{label} is missing path"))?;
    let path = PathBuf::from(ref_path);
    if path.is_absolute() || path.exists() {
        return Ok(path);
    }
    if let Some(parent) = source_path.parent() {
        let sibling = parent.join(&path);
        if sibling.exists() {
            return Ok(sibling);
        }
    }
    Ok(path)
}

fn append_true_marker_issues(
    issues: &mut Vec<String>,
    label: &str,
    artifact: &serde_json::Value,
    fields: &[&str],
) {
    for field in fields {
        if json_bool_value(artifact, field).unwrap_or(false) {
            issues.push(format!("{label}_{field}_true"));
        }
    }
}

fn append_nonzero_marker_issues(
    issues: &mut Vec<String>,
    label: &str,
    artifact: &serde_json::Value,
    fields: &[&str],
) {
    for field in fields {
        if json_u64_value(artifact, field).unwrap_or(0) > 0 {
            issues.push(format!("{label}_{field}_nonzero"));
        }
    }
}

fn cancel_preview_identifier_known(value: &str) -> bool {
    let normalized = value.trim();
    !normalized.is_empty()
        && !matches!(
            normalized.to_ascii_lowercase().as_str(),
            "missing" | "unknown" | "none" | "null"
        )
}

fn cancel_risk_gate_field_matches(actual: &str, expected: &str) -> bool {
    !actual.trim().is_empty() && actual.trim() == expected.trim()
}

fn json_array_has_items(value: &serde_json::Value, field: &str) -> bool {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| !items.is_empty())
}

fn redact_cancel_preview_identifier(kind: &str, value: &str) -> String {
    if !cancel_preview_identifier_known(value) {
        return "missing".to_string();
    }
    let digest = digest::digest(&digest::SHA256, value.trim().as_bytes());
    let hex = lowercase_hex(digest.as_ref());
    format!("{kind}:sha256:{}:len={}", &hex[..16], value.trim().len())
}

fn redacted_cancel_identifiers_match(left: &str, right: &str) -> bool {
    let Some(left_signature) = redacted_cancel_identifier_signature(left) else {
        return false;
    };
    let Some(right_signature) = redacted_cancel_identifier_signature(right) else {
        return false;
    };
    left_signature == right_signature
}

fn redacted_cancel_identifier_signature(value: &str) -> Option<&str> {
    value
        .trim()
        .split_once(":sha256:")
        .map(|(_, suffix)| suffix)
}

fn cancel_request_preview_reason(orphan_detection_outcome: &str) -> &'static str {
    match orphan_detection_outcome {
        "local_missing_exchange_seen" => "local_missing_exchange_seen",
        "stale_ledger_restart_required" => "stale_restart_review",
        _ => "orphan_risk_detected",
    }
}

fn production_mutation_exchange_readback_mapper_malformed_issues(
    order_readback: &serde_json::Value,
) -> Vec<String> {
    let mut issues = Vec::new();
    let order_found = json_bool_value(order_readback, "order_found").unwrap_or(true);
    if order_found {
        match json_scalar_string_value(order_readback, "exchange_status") {
            Some(status) if exchange_order_status_supported(&status) => {}
            Some(status) => issues.push(format!("unsupported_exchange_status_{status}")),
            None => issues.push("exchange_status_missing".to_string()),
        }
    }
    issues
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProductionMutationOrphanDetection {
    outcome: &'static str,
    orphan_risk_detected: bool,
    risk_halted: bool,
    new_orders_blocked: bool,
    manual_review_required: bool,
    stale_ledger_restart_required: bool,
    local_terminal_state: bool,
}

fn detect_production_mutation_orphan_order_risk(
    reconciliation_classifier: &serde_json::Value,
) -> ProductionMutationOrphanDetection {
    let reconciliation_outcome =
        json_string_value(reconciliation_classifier, "reconciliation_outcome")
            .unwrap_or_else(|| "unknown".to_string());
    let open_order_observed =
        json_bool_value(reconciliation_classifier, "open_order_observed").unwrap_or(false);
    let terminal_state_observed =
        json_bool_value(reconciliation_classifier, "terminal_state_observed").unwrap_or(false);
    let stale_ledger_restart_required =
        !json_bool_value(reconciliation_classifier, "restart_readable").unwrap_or(true)
            || json_bool_value(reconciliation_classifier, "stale_ledger_restart_required")
                .unwrap_or(false);
    let incident_risk_halted =
        json_bool_value(reconciliation_classifier, "incident_risk_halted").unwrap_or(false);
    let local_terminal_state = matches!(
        reconciliation_outcome.as_str(),
        "local_sent_exchange_filled"
            | "local_sent_exchange_canceled"
            | "local_sent_exchange_rejected"
    ) && terminal_state_observed;

    if stale_ledger_restart_required {
        return ProductionMutationOrphanDetection {
            outcome: "stale_ledger_restart_required",
            orphan_risk_detected: true,
            risk_halted: true,
            new_orders_blocked: true,
            manual_review_required: true,
            stale_ledger_restart_required: true,
            local_terminal_state,
        };
    }
    if incident_risk_halted {
        return ProductionMutationOrphanDetection {
            outcome: "failure_incident_risk_halt",
            orphan_risk_detected: true,
            risk_halted: true,
            new_orders_blocked: true,
            manual_review_required: true,
            stale_ledger_restart_required: false,
            local_terminal_state,
        };
    }
    if open_order_observed && !local_terminal_state {
        return ProductionMutationOrphanDetection {
            outcome: "open_orphan_risk",
            orphan_risk_detected: true,
            risk_halted: true,
            new_orders_blocked: true,
            manual_review_required: true,
            stale_ledger_restart_required: false,
            local_terminal_state,
        };
    }
    if reconciliation_outcome == "local_no_send_exchange_order_seen" {
        return ProductionMutationOrphanDetection {
            outcome: "local_missing_exchange_seen",
            orphan_risk_detected: true,
            risk_halted: true,
            new_orders_blocked: true,
            manual_review_required: true,
            stale_ledger_restart_required: false,
            local_terminal_state,
        };
    }
    if matches!(
        reconciliation_outcome.as_str(),
        "readback_failed" | "local_sent_exchange_unknown" | "local_sent_exchange_missing"
    ) {
        return ProductionMutationOrphanDetection {
            outcome: "readback_or_lineage_ambiguous",
            orphan_risk_detected: true,
            risk_halted: true,
            new_orders_blocked: true,
            manual_review_required: true,
            stale_ledger_restart_required: false,
            local_terminal_state,
        };
    }

    ProductionMutationOrphanDetection {
        outcome: "clean_terminal",
        orphan_risk_detected: false,
        risk_halted: false,
        new_orders_blocked: false,
        manual_review_required: false,
        stale_ledger_restart_required: false,
        local_terminal_state,
    }
}

#[derive(Debug, Clone)]
struct LoadedProductionMutationFailureSemantics {
    path: PathBuf,
    artifact: serde_json::Value,
}

#[derive(Debug, Clone)]
struct ProductionMutationFailureIncident {
    failure_mode: String,
    failure_state: String,
    terminal_action: String,
    outcome: &'static str,
    severity: &'static str,
    readback_required: bool,
    terminal_evidence_required: bool,
    risk_halted: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    source_path: String,
}

fn production_mutation_failure_semantics_from_exchange_mapper(
    exchange_readback_mapper: &serde_json::Value,
) -> Option<LoadedProductionMutationFailureSemantics> {
    let local_ledger_path = exchange_readback_mapper
        .get("local_ledger_ref")
        .and_then(|value| value.get("path"))
        .and_then(serde_json::Value::as_str)?;
    let local_ledger =
        load_json_file_if_present(Path::new(local_ledger_path), "local order ledger")?;
    let failure_semantics_path = local_ledger
        .get("failure_ref")
        .and_then(|value| value.get("path"))
        .and_then(serde_json::Value::as_str)?;
    let path = PathBuf::from(failure_semantics_path);
    let artifact = load_json_file_if_present(&path, "failure semantics")?;
    Some(LoadedProductionMutationFailureSemantics { path, artifact })
}

fn load_json_file_if_present(path: &Path, label: &str) -> Option<serde_json::Value> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw)
        .with_context(|| format!("parse {label} JSON at {}", path.display()))
        .ok()
}

fn classify_production_mutation_failure_incident(
    failure_semantics: Option<&LoadedProductionMutationFailureSemantics>,
) -> ProductionMutationFailureIncident {
    let Some(loaded) = failure_semantics else {
        return ProductionMutationFailureIncident {
            failure_mode: "none".to_string(),
            failure_state: "none".to_string(),
            terminal_action: "none".to_string(),
            outcome: "not_linked",
            severity: "info",
            readback_required: false,
            terminal_evidence_required: false,
            risk_halted: false,
            manual_review_required: false,
            new_orders_blocked: false,
            source_path: String::new(),
        };
    };
    let artifact = &loaded.artifact;
    let failure_mode =
        json_string_value(artifact, "failure_mode").unwrap_or_else(|| "unknown".to_string());
    let failure_state =
        json_string_value(artifact, "failure_state").unwrap_or_else(|| "unknown".to_string());
    let terminal_action =
        json_string_value(artifact, "terminal_action").unwrap_or_else(|| "unknown".to_string());
    let source_path = loaded.path.display().to_string();
    let schema_ok = json_string_value(artifact, "schema_version").as_deref()
        == Some(PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION);
    let ready = schema_ok
        && json_string_value(artifact, "status").as_deref()
            == Some("ready_failure_semantics_evidence")
        && json_bool_value(artifact, "failure_semantics_ready").unwrap_or(false);
    if !ready {
        return ProductionMutationFailureIncident {
            failure_mode,
            failure_state,
            terminal_action,
            outcome: "failure_semantics_unavailable",
            severity: "warning",
            readback_required: true,
            terminal_evidence_required: false,
            risk_halted: false,
            manual_review_required: true,
            new_orders_blocked: true,
            source_path,
        };
    }

    match failure_mode.as_str() {
        "timeout" => ProductionMutationFailureIncident {
            failure_mode,
            failure_state,
            terminal_action,
            outcome: "timeout_readback_required",
            severity: "warning",
            readback_required: true,
            terminal_evidence_required: false,
            risk_halted: false,
            manual_review_required: true,
            new_orders_blocked: true,
            source_path,
        },
        "http-4xx" => ProductionMutationFailureIncident {
            failure_mode,
            failure_state,
            terminal_action,
            outcome: "http_4xx_terminal_evidence",
            severity: "info",
            readback_required: false,
            terminal_evidence_required: true,
            risk_halted: false,
            manual_review_required: false,
            new_orders_blocked: false,
            source_path,
        },
        "http-5xx" => ProductionMutationFailureIncident {
            failure_mode,
            failure_state,
            terminal_action,
            outcome: "http_5xx_readback_required",
            severity: "warning",
            readback_required: true,
            terminal_evidence_required: false,
            risk_halted: false,
            manual_review_required: true,
            new_orders_blocked: true,
            source_path,
        },
        "malformed-response" => ProductionMutationFailureIncident {
            failure_mode,
            failure_state,
            terminal_action,
            outcome: "malformed_response_manual_review",
            severity: "warning",
            readback_required: false,
            terminal_evidence_required: false,
            risk_halted: false,
            manual_review_required: true,
            new_orders_blocked: true,
            source_path,
        },
        "readback-mismatch" => ProductionMutationFailureIncident {
            failure_mode,
            failure_state,
            terminal_action,
            outcome: "readback_mismatch_risk_halt",
            severity: "critical",
            readback_required: false,
            terminal_evidence_required: false,
            risk_halted: true,
            manual_review_required: true,
            new_orders_blocked: true,
            source_path,
        },
        "kill-switch-transition" => ProductionMutationFailureIncident {
            failure_mode,
            failure_state,
            terminal_action,
            outcome: "kill_switch_transition_halt",
            severity: "critical",
            readback_required: false,
            terminal_evidence_required: false,
            risk_halted: true,
            manual_review_required: true,
            new_orders_blocked: true,
            source_path,
        },
        _ => ProductionMutationFailureIncident {
            failure_mode,
            failure_state,
            terminal_action,
            outcome: "unknown_failure_manual_review",
            severity: "warning",
            readback_required: true,
            terminal_evidence_required: false,
            risk_halted: false,
            manual_review_required: true,
            new_orders_blocked: true,
            source_path,
        },
    }
}

fn classify_production_mutation_reconciliation_outcome(
    exchange_readback_mapper: &serde_json::Value,
) -> (&'static str, bool, bool) {
    let source_status = json_string_value(exchange_readback_mapper, "status")
        .unwrap_or_else(|| "unknown".to_string());
    let exchange_readback_mapped =
        json_bool_value(exchange_readback_mapper, "exchange_readback_mapped").unwrap_or(false);
    if source_status == "blocked_malformed_exchange_readback" || !exchange_readback_mapped {
        return ("readback_failed", true, true);
    }

    let local_request_sent =
        json_bool_value(exchange_readback_mapper, "request_sent").unwrap_or(false);
    let exchange_order_state = json_string_value(exchange_readback_mapper, "exchange_order_state")
        .unwrap_or_else(|| "unknown".to_string());
    let order_found = json_bool_value(exchange_readback_mapper, "order_found").unwrap_or(false);
    let open_order_observed =
        json_bool_value(exchange_readback_mapper, "open_order_observed").unwrap_or(false);
    let exchange_order_seen = order_found
        || open_order_observed
        || !matches!(
            exchange_order_state.as_str(),
            "missing" | "unknown" | "malformed"
        );

    if !local_request_sent && exchange_order_seen {
        return ("local_no_send_exchange_order_seen", true, true);
    }

    if !local_request_sent {
        return ("readback_failed", true, true);
    }

    match exchange_order_state.as_str() {
        "open" => ("local_sent_exchange_new", true, true),
        "filled" => ("local_sent_exchange_filled", false, false),
        "canceled" | "expired" => ("local_sent_exchange_canceled", false, false),
        "rejected" => ("local_sent_exchange_rejected", false, false),
        "missing" => ("local_sent_exchange_missing", true, true),
        _ => ("local_sent_exchange_unknown", true, true),
    }
}

fn exchange_order_status_supported(status: &str) -> bool {
    matches!(
        status.to_ascii_uppercase().as_str(),
        "NEW"
            | "PARTIALLY_FILLED"
            | "FILLED"
            | "CANCELED"
            | "CANCELLED"
            | "REJECTED"
            | "EXPIRED"
            | "MISSING"
    )
}

fn normalized_exchange_order_state(status: &str) -> &'static str {
    match status.to_ascii_uppercase().as_str() {
        "NEW" | "PARTIALLY_FILLED" => "open",
        "FILLED" => "filled",
        "CANCELED" | "CANCELLED" => "canceled",
        "REJECTED" => "rejected",
        "EXPIRED" => "expired",
        "MISSING" => "missing",
        _ => "malformed",
    }
}

fn exchange_order_status_is_terminal(status: &str) -> bool {
    matches!(
        status.to_ascii_uppercase().as_str(),
        "FILLED" | "CANCELED" | "CANCELLED" | "REJECTED" | "EXPIRED"
    )
}

fn exchange_open_order_observed(
    open_orders_readback: &serde_json::Value,
    known_order_id: &str,
    known_client_order_id: &str,
    symbol: &str,
) -> bool {
    open_orders_readback
        .get("open_orders")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|orders| {
            orders.iter().any(|order| {
                let order_id_matches =
                    json_scalar_string_value(order, "order_id").as_deref() == Some(known_order_id);
                let client_order_id_matches = json_scalar_string_value(order, "client_order_id")
                    .as_deref()
                    == Some(known_client_order_id);
                let symbol_matches =
                    json_scalar_string_value(order, "symbol").as_deref() == Some(symbol);
                symbol_matches && (order_id_matches || client_order_id_matches)
            })
        })
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

fn production_mutation_cancel_response_allowed_fields() -> Vec<String> {
    [
        "symbol",
        "orderId",
        "clientOrderId",
        "origClientOrderId",
        "status",
        "transactTime",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

fn production_mutation_post_cancel_readback_allowed_fields() -> Vec<String> {
    [
        "symbol",
        "orderId",
        "clientOrderId",
        "origClientOrderId",
        "status",
        "updateTime",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

fn production_mutation_post_cancel_readback_state(readback: &serde_json::Value) -> String {
    json_scalar_string_value(readback, "status")
        .unwrap_or_else(|| "UNKNOWN".to_string())
        .trim()
        .to_ascii_uppercase()
}

fn production_mutation_post_cancel_readback_classification(
    readback_state: &str,
) -> (&'static str, &'static str, bool, bool) {
    match readback_state {
        "CANCELED" | "CANCELLED" => ("terminal_canceled", "cancel_confirmed", true, false),
        "FILLED" => (
            "terminal_filled",
            "filled_before_or_during_cancel",
            true,
            false,
        ),
        "REJECTED" => ("terminal_rejected", "cancel_or_order_rejected", true, false),
        "EXPIRED" => ("terminal_expired", "order_expired", true, false),
        "MISSING" => (
            "ambiguous_missing",
            "order_missing_manual_review",
            false,
            true,
        ),
        "UNKNOWN" => (
            "ambiguous_unknown",
            "unknown_state_manual_review",
            false,
            true,
        ),
        _ => ("unsupported", "unsupported_state_blocked", false, true),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProductionMutationActualCancelReadbackReconciliationDecision {
    readback_result: String,
    reconciliation_status: String,
    artifact_status: String,
    venue_state: String,
    order_status: String,
    execution_fill_status: String,
    remaining_quantity_state: String,
    residual_risk_state: String,
    local_audit_state: String,
    partial_fill_observed: bool,
    already_cancelled_observed: bool,
    filled_before_cancel_observed: bool,
    timeout_observed: bool,
    unknown_observed: bool,
    inconsistent_observed: bool,
    degraded: bool,
    error_state: bool,
    terminal_state_observed: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    risk_halted: bool,
    readback_reconciliation_complete: bool,
    actual_cancel_followup_complete: bool,
}

fn production_mutation_actual_cancel_readback_reconciliation_allowed_fields() -> Vec<String> {
    [
        "symbol",
        "orderId",
        "clientOrderId",
        "origClientOrderId",
        "status",
        "readbackStatus",
        "readbackResult",
        "cancelResult",
        "reconciliationHint",
        "venueDisposition",
        "alreadyCancelled",
        "already_cancelled",
        "executedQty",
        "origQty",
        "remainingQty",
        "updateTime",
        "localAuditState",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

fn production_mutation_actual_cancel_readback_reconciliation_state(
    readback: &serde_json::Value,
) -> String {
    json_scalar_string_value(readback, "readbackStatus")
        .or_else(|| json_scalar_string_value(readback, "status"))
        .unwrap_or_else(|| "UNKNOWN".to_string())
        .trim()
        .to_ascii_uppercase()
}

fn classify_production_mutation_actual_cancel_readback_reconciliation(
    readback_state: &str,
    readback: &serde_json::Value,
) -> ProductionMutationActualCancelReadbackReconciliationDecision {
    let timeout = production_mutation_actual_cancel_readback_reconciliation_timeout(readback)
        || readback_state == "TIMEOUT";
    if timeout {
        return actual_cancel_readback_reconciliation_decision(
            "timeout",
            "degraded_timeout",
            "degraded_actual_cancel_readback_timeout",
            "readback_timeout",
            "timeout",
            "unknown_timeout",
            "unknown_timeout",
            "unknown_timeout_manual_review",
            "readback_timeout_recorded",
            false,
            false,
            false,
            true,
            false,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            false,
            false,
        );
    }

    let already_cancelled = production_mutation_actual_cancel_readback_already_cancelled(readback);
    let executed_qty = production_mutation_readback_decimal_field(readback, "executedQty");
    let orig_qty = production_mutation_readback_decimal_field(readback, "origQty");
    let partial_fill_observed = readback_state == "PARTIALLY_FILLED"
        || matches!(
            (executed_qty, orig_qty),
            (Some(executed), Some(orig)) if executed > Decimal::ZERO && executed < orig
        );
    let filled_before_cancel_observed = readback_state == "FILLED"
        || matches!(
            (executed_qty, orig_qty),
            (Some(executed), Some(orig)) if orig > Decimal::ZERO && executed >= orig
        );

    if partial_fill_observed {
        return actual_cancel_readback_reconciliation_decision(
            "inconsistent",
            "degraded_inconsistent",
            "degraded_actual_cancel_readback_inconsistent",
            "partial_fill_observed_after_cancel",
            "partially_filled",
            "partial_fill",
            "remaining_quantity_open_or_unknown",
            "partial_fill_residual_risk_manual_review",
            "partial_fill_inconsistent_recorded",
            true,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            false,
            false,
        );
    }

    if already_cancelled {
        return actual_cancel_readback_reconciliation_decision(
            "already_cancelled",
            "ready_already_cancelled",
            "ready_actual_cancel_readback_already_cancelled",
            "terminal_cancelled_before_or_by_attempt",
            "cancelled",
            "no_fill_observed",
            "zero_remaining_or_terminal",
            "none_terminal_already_cancelled",
            "already_cancelled_readback_recorded",
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            true,
            true,
        );
    }

    if matches!(readback_state, "CANCELED" | "CANCELLED") {
        return actual_cancel_readback_reconciliation_decision(
            "cancel_confirmed",
            "ready_cancel_confirmed",
            "ready_actual_cancel_readback_cancel_confirmed",
            "terminal_cancelled",
            "cancelled",
            "no_fill_observed",
            "zero_remaining_or_terminal",
            "none_terminal_cancel_confirmed",
            "cancel_confirmed_readback_recorded",
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            true,
            true,
        );
    }

    if filled_before_cancel_observed {
        return actual_cancel_readback_reconciliation_decision(
            "filled_before_cancel",
            "ready_filled_before_cancel",
            "ready_actual_cancel_readback_filled_before_cancel",
            "terminal_filled",
            "filled",
            "filled_before_cancel",
            "zero_remaining_terminal_fill",
            "filled_position_review_required",
            "filled_before_cancel_readback_recorded",
            false,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }

    if readback_state == "UNKNOWN" {
        return actual_cancel_readback_reconciliation_decision(
            "unknown",
            "degraded_unknown",
            "degraded_actual_cancel_readback_unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown_manual_review",
            "unknown_readback_recorded",
            false,
            false,
            false,
            false,
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            true,
            false,
            false,
        );
    }

    match readback_state {
        "NEW" | "PENDING_CANCEL" | "REJECTED" | "EXPIRED" | "MISSING" => {
            actual_cancel_readback_reconciliation_decision(
                "inconsistent",
                "degraded_inconsistent",
                "degraded_actual_cancel_readback_inconsistent",
                "post_cancel_state_not_cancelled",
                &readback_state.to_ascii_lowercase(),
                "not_fully_reconciled",
                "remaining_quantity_open_or_unknown",
                "inconsistent_manual_review",
                "inconsistent_readback_recorded",
                false,
                false,
                false,
                false,
                false,
                true,
                true,
                true,
                false,
                true,
                true,
                true,
                false,
                false,
            )
        }
        _ => actual_cancel_readback_reconciliation_decision(
            "unsupported",
            "blocked_unsupported",
            "blocked_unsupported_readback_state",
            "unsupported",
            "unsupported",
            "unsupported",
            "unsupported",
            "unsupported_manual_review",
            "unsupported_readback_recorded",
            false,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            false,
            false,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn actual_cancel_readback_reconciliation_decision(
    readback_result: &str,
    reconciliation_status: &str,
    artifact_status: &str,
    venue_state: &str,
    order_status: &str,
    execution_fill_status: &str,
    remaining_quantity_state: &str,
    residual_risk_state: &str,
    local_audit_state: &str,
    partial_fill_observed: bool,
    already_cancelled_observed: bool,
    filled_before_cancel_observed: bool,
    timeout_observed: bool,
    unknown_observed: bool,
    inconsistent_observed: bool,
    degraded: bool,
    error_state: bool,
    terminal_state_observed: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    risk_halted: bool,
    readback_reconciliation_complete: bool,
    actual_cancel_followup_complete: bool,
) -> ProductionMutationActualCancelReadbackReconciliationDecision {
    ProductionMutationActualCancelReadbackReconciliationDecision {
        readback_result: readback_result.to_string(),
        reconciliation_status: reconciliation_status.to_string(),
        artifact_status: artifact_status.to_string(),
        venue_state: venue_state.to_string(),
        order_status: order_status.to_string(),
        execution_fill_status: execution_fill_status.to_string(),
        remaining_quantity_state: remaining_quantity_state.to_string(),
        residual_risk_state: residual_risk_state.to_string(),
        local_audit_state: local_audit_state.to_string(),
        partial_fill_observed,
        already_cancelled_observed,
        filled_before_cancel_observed,
        timeout_observed,
        unknown_observed,
        inconsistent_observed,
        degraded,
        error_state,
        terminal_state_observed,
        manual_review_required,
        new_orders_blocked,
        risk_halted,
        readback_reconciliation_complete,
        actual_cancel_followup_complete,
    }
}

fn production_mutation_actual_cancel_readback_reconciliation_timeout(
    readback: &serde_json::Value,
) -> bool {
    json_bool_value(readback, "timeout").unwrap_or(false)
        || json_scalar_string_value(readback, "readbackResult")
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("timeout"))
        || json_scalar_string_value(readback, "readbackStatus")
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("timeout"))
}

fn production_mutation_actual_cancel_readback_already_cancelled(
    readback: &serde_json::Value,
) -> bool {
    json_bool_value(readback, "alreadyCancelled").unwrap_or(false)
        || json_bool_value(readback, "already_cancelled").unwrap_or(false)
        || [
            "cancelResult",
            "readbackResult",
            "reconciliationHint",
            "venueDisposition",
        ]
        .into_iter()
        .filter_map(|field| json_scalar_string_value(readback, field))
        .any(|value| {
            matches!(
                value
                    .trim()
                    .replace([' ', '-'], "_")
                    .to_ascii_uppercase()
                    .as_str(),
                "ALREADY_CANCELLED" | "ALREADY_CANCELED"
            )
        })
}

fn production_mutation_readback_decimal_field(
    readback: &serde_json::Value,
    field: &str,
) -> Option<Decimal> {
    let value = json_scalar_string_value(readback, field)?;
    parse_non_negative_decimal(&value).ok()
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

fn json_pointer_string_value(value: &serde_json::Value, pointer: &str) -> Option<String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn json_pointer_bool_value(value: &serde_json::Value, pointer: &str) -> Option<bool> {
    value.pointer(pointer).and_then(serde_json::Value::as_bool)
}

fn json_array_len(value: &serde_json::Value, field: &str) -> Option<usize> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
}

fn file_fnv1a64_hash(path: &str) -> String {
    match fs::read(path) {
        Ok(bytes) => format!("fnv1a64:{:016x}", fnv1a64(&bytes)),
        Err(_) => "unavailable".to_string(),
    }
}

fn file_sha256_hash(path: &Path) -> String {
    match fs::read(path) {
        Ok(bytes) => {
            let digest = digest::digest(&digest::SHA256, &bytes);
            format!("sha256:{}", lowercase_hex(digest.as_ref()))
        }
        Err(_) => "unavailable".to_string(),
    }
}

fn file_byte_len(path: &Path) -> u64 {
    fs::metadata(path).map_or(0, |metadata| metadata.len())
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
    if is_production_market_data_node_config(&config)? {
        return run_production_market_data_node_with_command(
            &config,
            run_id.as_deref(),
            output.as_deref(),
            stop_file.as_deref(),
            controls,
        )
        .await;
    }
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

fn is_production_market_data_node_config(path: &Path) -> anyhow::Result<bool> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read ntpro-node config '{}'", path.display()))?;
    let value: toml::Value = toml::from_str(&raw)
        .with_context(|| format!("failed to parse ntpro-node config '{}'", path.display()))?;
    Ok(value.get("live_market_data").is_some())
}

fn load_production_market_data_node_config(
    path: &Path,
) -> anyhow::Result<ProductionMarketDataNodeConfig> {
    let raw = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read production market data config '{}'",
            path.display()
        )
    })?;
    let config: ProductionMarketDataNodeConfig = toml::from_str(&raw).with_context(|| {
        format!(
            "failed to parse production market data config '{}'",
            path.display()
        )
    })?;
    validate_production_market_data_node_config(&config)?;
    Ok(config)
}

fn validate_production_market_data_node_config(
    config: &ProductionMarketDataNodeConfig,
) -> anyhow::Result<()> {
    let market = &config.live_market_data;
    validate_exact(
        "live_market_data.schema_version",
        &market.schema_version,
        PRODUCTION_MARKET_DATA_SCHEMA_VERSION,
    )?;
    validate_exact(
        "live_market_data.mode",
        &market.mode,
        PRODUCTION_MARKET_DATA_MODE,
    )?;
    validate_exact(
        "live_market_data.environment",
        &market.environment,
        LIVE_ENVIRONMENT,
    )?;
    validate_non_empty("live_market_data.node_id", &market.node_id)?;
    validate_non_empty("live_market_data.trader_id", &market.trader_id)?;
    validate_exact("live_market_data.venue", &market.venue, "BINANCE")?;
    validate_exact(
        "live_market_data.product_type",
        &market.product_type,
        BINANCE_SPOT_PRODUCT_TYPE,
    )?;
    if market.symbols.is_empty() || market.symbols.len() > 32 {
        anyhow::bail!("production market data symbols must contain between 1 and 32 entries");
    }
    let mut symbols = BTreeSet::new();
    for symbol in &market.symbols {
        let instrument_id = InstrumentId::from_str(symbol)
            .with_context(|| format!("invalid production market data symbol '{symbol}'"))?;
        if instrument_id.venue != Venue::from("BINANCE") || !symbols.insert(symbol) {
            anyhow::bail!("production market data symbols must be unique Binance instruments");
        }
    }
    validate_exact(
        "live_market_data.api_key_env",
        &market.api_key_env,
        "NTPRO_BINANCE_LIVE_API_KEY",
    )?;
    validate_exact(
        "live_market_data.api_secret_env",
        &market.api_secret_env,
        "NTPRO_BINANCE_LIVE_API_SECRET",
    )?;
    if market.execution_client_enabled
        || market.order_endpoint_access_allowed
        || market.order_submission_allowed
        || market.automatic_reconnect_allowed
    {
        anyhow::bail!(
            "production market data Runtime requires execution, order access, submission and automatic reconnect to remain false"
        );
    }
    if let Some(execution) = &config.live_execution {
        validate_exact(
            "live_execution.schema_version",
            &execution.schema_version,
            PRODUCTION_EXECUTION_SCHEMA_VERSION,
        )?;
        for (field, value) in [
            (
                "live_execution.source_manifest_sha256",
                execution.source_manifest_sha256.as_str(),
            ),
            (
                "live_execution.execution_admission_sha256",
                execution.execution_admission_sha256.as_str(),
            ),
            (
                "live_execution.risk_policy_ref",
                execution.risk_policy_ref.as_str(),
            ),
            (
                "live_execution.owner_authority_ref",
                execution.owner_authority_ref.as_str(),
            ),
            (
                "live_execution.risk_authority_ref",
                execution.risk_authority_ref.as_str(),
            ),
            (
                "live_execution.operator_authority_ref",
                execution.operator_authority_ref.as_str(),
            ),
            (
                "live_execution.admission_id",
                execution.admission_id.as_str(),
            ),
            (
                "live_execution.strategy_version_id",
                execution.strategy_version_id.as_str(),
            ),
            ("live_execution.account_id", execution.account_id.as_str()),
            (
                "live_execution.instrument_id",
                execution.instrument_id.as_str(),
            ),
        ] {
            validate_non_empty(field, value)?;
        }
        if !valid_prefixed_sha256(&execution.source_manifest_sha256)
            || !valid_prefixed_sha256(&execution.execution_admission_sha256)
            || !execution.runtime_artifact_root.is_absolute()
            || execution.owner_authority_ref == execution.risk_authority_ref
            || execution.owner_authority_ref == execution.operator_authority_ref
            || execution.risk_authority_ref == execution.operator_authority_ref
        {
            anyhow::bail!("live_execution authority binding is invalid");
        }
        let instrument_id = InstrumentId::from_str(&execution.instrument_id)
            .context("invalid live_execution.instrument_id")?;
        if instrument_id.venue != Venue::from("BINANCE")
            || !market.symbols.contains(&execution.instrument_id)
        {
            anyhow::bail!(
                "live_execution.instrument_id must be one of the admitted Binance market symbols"
            );
        }
        validate_exact("live_execution.order_type", &execution.order_type, "LIMIT")?;
        validate_exact(
            "live_execution.time_in_force",
            &execution.time_in_force,
            "GTC",
        )?;
        if !matches!(execution.side.as_str(), "BUY" | "SELL") {
            anyhow::bail!("live_execution.side must be BUY or SELL");
        }
        let price = Decimal::from_str_exact(&execution.price)
            .context("live_execution.price must be a decimal")?;
        let quantity = Decimal::from_str_exact(&execution.quantity)
            .context("live_execution.quantity must be a decimal")?;
        let max_notional = Decimal::from_str_exact(&execution.max_notional)
            .context("live_execution.max_notional must be a decimal")?;
        let risk_policy_max_notional = Decimal::from_str_exact(&execution.risk_policy_max_notional)
            .context("live_execution.risk_policy_max_notional must be a decimal")?;
        if price <= Decimal::ZERO
            || quantity <= Decimal::ZERO
            || max_notional <= Decimal::ZERO
            || risk_policy_max_notional <= Decimal::ZERO
            || price * quantity > max_notional
            || max_notional > risk_policy_max_notional
        {
            anyhow::bail!("live_execution order must be positive and within max_notional");
        }
        if execution.expires_at_unix_ms <= current_unix_timestamp_millis() {
            anyhow::bail!("live_execution admission has expired");
        }
        validate_exact(
            "live_execution.api_key_env",
            &execution.api_key_env,
            "NTPRO_BINANCE_LIVE_API_KEY",
        )?;
        validate_exact(
            "live_execution.api_secret_env",
            &execution.api_secret_env,
            "NTPRO_BINANCE_LIVE_API_SECRET",
        )?;
        if !execution.owner_confirmed
            || !execution.risk_confirmed
            || !execution.operator_confirmed
            || execution.kill_switch_active
            || !execution.single_shot
            || execution.cancel_order_allowed
            || execution.replace_order_allowed
            || execution.automatic_retry_allowed
            || execution.automatic_recovery_allowed
        {
            anyhow::bail!(
                "live_execution requires three-party single-shot approval with kill switch clear and all follow-up mutations disabled"
            );
        }
    }
    validate_exact("shutdown.mode", &config.shutdown.mode, START_STOP_SHUTDOWN)?;
    if config.shutdown.connection_timeout_secs == 0
        || config.shutdown.disconnection_timeout_secs == 0
    {
        anyhow::bail!("production market data connection timeouts must be greater than zero");
    }
    Ok(())
}

fn current_unix_timestamp_millis() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}

fn valid_prefixed_sha256(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
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
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
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

fn write_production_mutation_local_order_ledger_artifact(
    path: &Path,
    value: &ProductionMutationLocalOrderLedgerArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_exchange_readback_mapper_artifact(
    path: &Path,
    value: &ProductionMutationExchangeReadbackMapperArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_reconciliation_classifier_artifact(
    path: &Path,
    value: &ProductionMutationReconciliationClassifierArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_orphan_order_detector_artifact(
    path: &Path,
    value: &ProductionMutationOrphanOrderDetectorArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_cancel_request_preview_artifact(
    path: &Path,
    value: &ProductionMutationCancelRequestPreviewArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_cancel_risk_gate_artifact(
    path: &Path,
    value: &ProductionMutationCancelRiskGateArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_manual_owner_approval_lifecycle_artifact(
    path: &Path,
    value: &ProductionMutationManualOwnerApprovalLifecycleArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_actual_cancel_owner_approval_lifecycle_artifact(
    path: &Path,
    value: &ProductionMutationActualCancelOwnerApprovalLifecycleArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_actual_cancel_executor_adapter_boundary_artifact(
    path: &Path,
    value: &ProductionMutationActualCancelExecutorAdapterBoundaryArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_actual_cancel_single_shot_artifact(
    path: &Path,
    value: &ProductionMutationActualCancelSingleShotArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_actual_cancel_readback_reconciliation_artifact(
    path: &Path,
    value: &ProductionMutationActualCancelReadbackReconciliationArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_actual_cancel_failure_evidence_artifact(
    path: &Path,
    value: &ProductionMutationActualCancelFailureEvidenceArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_cancel_response_redaction_artifact(
    path: &Path,
    value: &ProductionMutationCancelResponseRedactionArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_post_cancel_readback_artifact(
    path: &Path,
    value: &ProductionMutationPostCancelReadbackArtifact,
) -> anyhow::Result<()> {
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    atomic_write_text(path, &body)?;
    Ok(())
}

fn write_production_mutation_cancel_recovery_incident_audit_closeout_artifact(
    path: &Path,
    value: &ProductionMutationCancelRecoveryIncidentAuditCloseoutArtifact,
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
    if !opt.confirm_non_marketable_price {
        missing.push("--confirm-non-marketable-price");
    }
    if !opt.confirm_owner_acknowledged_no_cancel_path {
        missing.push("--confirm-owner-acknowledged-no-cancel-path");
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

fn missing_production_mutation_local_order_ledger_cli_flags(
    opt: &LiveProductionMutationLocalOrderLedgerOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_local_order_ledger {
        missing.push("--allow-production-mutation-local-order-ledger");
    }
    if !opt.confirm_single_v16_mutation_candidate_lineage {
        missing.push("--confirm-single-v16-mutation-candidate-lineage");
    }
    if !opt.confirm_read_only_reconciliation_scope {
        missing.push("--confirm-read-only-reconciliation-scope");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_duplicate_submit {
        missing.push("--confirm-no-duplicate-submit");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_exchange_readback_mapper_cli_flags(
    opt: &LiveProductionMutationExchangeReadbackMapperOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_exchange_readback_mapper {
        missing.push("--allow-production-mutation-exchange-readback-mapper");
    }
    if !opt.confirm_redacted_readback_metadata_only {
        missing.push("--confirm-redacted-readback-metadata-only");
    }
    if !opt.confirm_known_order_identifier_only {
        missing.push("--confirm-known-order-identifier-only");
    }
    if !opt.confirm_read_only_reconciliation_scope {
        missing.push("--confirm-read-only-reconciliation-scope");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    if !opt.confirm_no_production_order_mutation {
        missing.push("--confirm-no-production-order-mutation");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    missing
}

fn missing_production_mutation_reconciliation_classifier_cli_flags(
    opt: &LiveProductionMutationReconciliationClassifierOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_reconciliation_classifier {
        missing.push("--allow-production-mutation-reconciliation-classifier");
    }
    if !opt.confirm_single_v16_mutation_candidate_lineage {
        missing.push("--confirm-single-v16-mutation-candidate-lineage");
    }
    if !opt.confirm_read_only_reconciliation_scope {
        missing.push("--confirm-read-only-reconciliation-scope");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_orphan_order_detector_cli_flags(
    opt: &LiveProductionMutationOrphanOrderDetectorOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_orphan_order_detector {
        missing.push("--allow-production-mutation-orphan-order-detector");
    }
    if !opt.confirm_single_v16_mutation_candidate_lineage {
        missing.push("--confirm-single-v16-mutation-candidate-lineage");
    }
    if !opt.confirm_read_only_reconciliation_scope {
        missing.push("--confirm-read-only-reconciliation-scope");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_cancel_request_preview_cli_flags(
    opt: &LiveProductionMutationCancelRequestPreviewOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_cancel_request_preview {
        missing.push("--allow-production-mutation-cancel-request-preview");
    }
    if !opt.confirm_single_v16_mutation_candidate_lineage {
        missing.push("--confirm-single-v16-mutation-candidate-lineage");
    }
    if !opt.confirm_orphan_risk_halted {
        missing.push("--confirm-orphan-risk-halted");
    }
    if !opt.confirm_manual_review_required {
        missing.push("--confirm-manual-review-required");
    }
    if !opt.confirm_known_order_identifier_only {
        missing.push("--confirm-known-order-identifier-only");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_cancel_risk_gate_cli_flags(
    opt: &LiveProductionMutationCancelRiskGateOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_cancel_risk_gate {
        missing.push("--allow-production-mutation-cancel-risk-gate");
    }
    if !opt.confirm_single_v16_mutation_candidate_lineage {
        missing.push("--confirm-single-v16-mutation-candidate-lineage");
    }
    if !opt.confirm_cancel_request_preview_ready {
        missing.push("--confirm-cancel-request-preview-ready");
    }
    if !opt.confirm_orphan_risk_halted {
        missing.push("--confirm-orphan-risk-halted");
    }
    if !opt.confirm_known_order_identifier_only {
        missing.push("--confirm-known-order-identifier-only");
    }
    if !opt.confirm_symbol_account_scope {
        missing.push("--confirm-symbol-account-scope");
    }
    if !opt.confirm_owner_approval_required {
        missing.push("--confirm-owner-approval-required");
    }
    if !opt.confirm_no_cancel_all_or_bulk {
        missing.push("--confirm-no-cancel-all-or-bulk");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_manual_owner_approval_lifecycle_cli_flags(
    opt: &LiveProductionMutationManualOwnerApprovalLifecycleOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_manual_owner_approval_lifecycle {
        missing.push("--allow-production-mutation-manual-owner-approval-lifecycle");
    }
    if !opt.confirm_one_order_cancel_candidate {
        missing.push("--confirm-one-order-cancel-candidate");
    }
    if !opt.confirm_one_time_approval {
        missing.push("--confirm-one-time-approval");
    }
    if !opt.confirm_non_reusable_approval {
        missing.push("--confirm-non-reusable-approval");
    }
    if !opt.confirm_approval_expiry {
        missing.push("--confirm-approval-expiry");
    }
    if !opt.confirm_no_strategy_auto_approval {
        missing.push("--confirm-no-strategy-auto-approval");
    }
    if !opt.confirm_no_background_auto_approval {
        missing.push("--confirm-no-background-auto-approval");
    }
    if !opt.confirm_no_dashboard_cancel_approval {
        missing.push("--confirm-no-dashboard-cancel-approval");
    }
    if !opt.confirm_no_incident_handler_auto_approval {
        missing.push("--confirm-no-incident-handler-auto-approval");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_actual_cancel_owner_approval_lifecycle_cli_flags(
    opt: &LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_actual_cancel_owner_approval_lifecycle {
        missing.push("--allow-production-mutation-actual-cancel-owner-approval-lifecycle");
    }
    if !opt.confirm_actual_cancel_safety_contract {
        missing.push("--confirm-actual-cancel-safety-contract");
    }
    if !opt.confirm_one_order_one_venue_one_attempt {
        missing.push("--confirm-one-order-one-venue-one-attempt");
    }
    if !opt.confirm_single_use_approval {
        missing.push("--confirm-single-use-approval");
    }
    if !opt.confirm_approval_expiry {
        missing.push("--confirm-approval-expiry");
    }
    if !opt.confirm_bind_order_risk_gate_release_provenance {
        missing.push("--confirm-bind-order-risk-gate-release-provenance");
    }
    if !opt.confirm_audit_evidence {
        missing.push("--confirm-audit-evidence");
    }
    if !opt.confirm_no_dashboard_approval {
        missing.push("--confirm-no-dashboard-approval");
    }
    if !opt.confirm_no_automatic_cancel {
        missing.push("--confirm-no-automatic-cancel");
    }
    if !opt.confirm_no_bulk_cancel {
        missing.push("--confirm-no-bulk-cancel");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_submit_lifecycle {
        missing.push("--confirm-no-submit-lifecycle");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_actual_cancel_executor_adapter_boundary_cli_flags(
    opt: &LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_actual_cancel_executor_adapter_boundary {
        missing.push("--allow-production-mutation-actual-cancel-executor-adapter-boundary");
    }
    if !opt.confirm_adapter_capability {
        missing.push("--confirm-adapter-capability");
    }
    if !opt.confirm_request_response_readback_audit_contract {
        missing.push("--confirm-request-response-readback-audit-contract");
    }
    if !opt.confirm_one_order_one_venue_one_attempt {
        missing.push("--confirm-one-order-one-venue-one-attempt");
    }
    if !opt.confirm_fail_closed_unsupported_capability {
        missing.push("--confirm-fail-closed-unsupported-capability");
    }
    if !opt.confirm_no_bulk_cancel {
        missing.push("--confirm-no-bulk-cancel");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_automatic_cancel {
        missing.push("--confirm-no-automatic-cancel");
    }
    if !opt.confirm_no_dashboard_execution {
        missing.push("--confirm-no-dashboard-execution");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_actual_cancel_single_shot_cli_flags(
    opt: &LiveProductionMutationActualCancelSingleShotOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_actual_cancel_single_shot {
        missing.push("--allow-production-mutation-actual-cancel-single-shot");
    }
    if !opt.confirm_owner_approval {
        missing.push("--confirm-owner-approval");
    }
    if !opt.confirm_risk_gate {
        missing.push("--confirm-risk-gate");
    }
    if !opt.confirm_release_provenance {
        missing.push("--confirm-release-provenance");
    }
    if !opt.confirm_adapter_boundary {
        missing.push("--confirm-adapter-boundary");
    }
    if !opt.confirm_single_shot {
        missing.push("--confirm-single-shot");
    }
    if !opt.confirm_consume_approval_before_send {
        missing.push("--confirm-consume-approval-before-send");
    }
    if !opt.confirm_readback_required {
        missing.push("--confirm-readback-required");
    }
    if !opt.confirm_no_bulk_cancel {
        missing.push("--confirm-no-bulk-cancel");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_automatic_cancel {
        missing.push("--confirm-no-automatic-cancel");
    }
    if !opt.confirm_no_dashboard_execution {
        missing.push("--confirm-no-dashboard-execution");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn missing_production_mutation_actual_cancel_readback_reconciliation_cli_flags(
    opt: &LiveProductionMutationActualCancelReadbackReconciliationOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_actual_cancel_readback_reconciliation {
        missing.push("--allow-production-mutation-actual-cancel-readback-reconciliation");
    }
    if !opt.confirm_actual_cancel_attempt_recorded {
        missing.push("--confirm-actual-cancel-attempt-recorded");
    }
    if !opt.confirm_readback_required {
        missing.push("--confirm-readback-required");
    }
    if !opt.confirm_readback_metadata_only {
        missing.push("--confirm-readback-metadata-only");
    }
    if !opt.confirm_order_status_reconciled {
        missing.push("--confirm-order-status-reconciled");
    }
    if !opt.confirm_execution_fill_status_reconciled {
        missing.push("--confirm-execution-fill-status-reconciled");
    }
    if !opt.confirm_remaining_quantity_reconciled {
        missing.push("--confirm-remaining-quantity-reconciled");
    }
    if !opt.confirm_risk_state_recorded {
        missing.push("--confirm-risk-state-recorded");
    }
    if !opt.confirm_local_audit_state_recorded {
        missing.push("--confirm-local-audit-state-recorded");
    }
    if !opt.confirm_dashboard_read_only_consumable {
        missing.push("--confirm-dashboard-read-only-consumable");
    }
    if !opt.confirm_no_raw_readback_persistence {
        missing.push("--confirm-no-raw-readback-persistence");
    }
    if !opt.confirm_no_headers_persistence {
        missing.push("--confirm-no-headers-persistence");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_no_second_cancel {
        missing.push("--confirm-no-second-cancel");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    missing
}

fn missing_production_mutation_actual_cancel_failure_evidence_cli_flags(
    opt: &LiveProductionMutationActualCancelFailureEvidenceOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_actual_cancel_failure_evidence {
        missing.push("--allow-production-mutation-actual-cancel-failure-evidence");
    }
    if !opt.confirm_request_ref_recorded {
        missing.push("--confirm-request-ref-recorded");
    }
    if !opt.confirm_response_ref_recorded {
        missing.push("--confirm-response-ref-recorded");
    }
    if !opt.confirm_readback_ref_recorded {
        missing.push("--confirm-readback-ref-recorded");
    }
    if !opt.confirm_audit_ref_recorded {
        missing.push("--confirm-audit-ref-recorded");
    }
    if !opt.confirm_failure_outcomes_classified {
        missing.push("--confirm-failure-outcomes-classified");
    }
    if !opt.confirm_operator_action_model {
        missing.push("--confirm-operator-action-model");
    }
    if !opt.confirm_unknown_not_recovered {
        missing.push("--confirm-unknown-not-recovered");
    }
    if !opt.confirm_partial_fill_residual_risk {
        missing.push("--confirm-partial-fill-residual-risk");
    }
    if !opt.confirm_dashboard_release_gate_consumable {
        missing.push("--confirm-dashboard-release-gate-consumable");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_no_compensation_trade {
        missing.push("--confirm-no-compensation-trade");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    missing
}

fn first_actual_cancel_owner_approval_failure_reason(
    missing_cli_flags: &[&'static str],
    safety_contract_issues: &[String],
    release_manifest_issues: &[String],
    source_artifact_issues: &[String],
    lifecycle_issues: &[String],
) -> String {
    if !missing_cli_flags.is_empty() {
        return "missing_cli_flags".to_string();
    }
    safety_contract_issues
        .first()
        .or_else(|| release_manifest_issues.first())
        .or_else(|| source_artifact_issues.first())
        .or_else(|| lifecycle_issues.first())
        .cloned()
        .unwrap_or_else(|| "none".to_string())
}

fn missing_production_mutation_cancel_response_redaction_cli_flags(
    opt: &LiveProductionMutationCancelResponseRedactionOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_cancel_response_redaction {
        missing.push("--allow-production-mutation-cancel-response-redaction");
    }
    if !opt.confirm_manual_owner_approval_lifecycle_ready {
        missing.push("--confirm-manual-owner-approval-lifecycle-ready");
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
    if !opt.confirm_cancel_metadata_only {
        missing.push("--confirm-cancel-metadata-only");
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
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    missing
}

fn missing_production_mutation_post_cancel_readback_cli_flags(
    opt: &LiveProductionMutationPostCancelReadbackOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_post_cancel_readback {
        missing.push("--allow-production-mutation-post-cancel-readback");
    }
    if !opt.confirm_cancel_response_redaction_ready {
        missing.push("--confirm-cancel-response-redaction-ready");
    }
    if !opt.confirm_readback_metadata_only {
        missing.push("--confirm-readback-metadata-only");
    }
    if !opt.confirm_terminal_and_ambiguous_classification {
        missing.push("--confirm-terminal-and-ambiguous-classification");
    }
    if !opt.confirm_no_raw_readback_persistence {
        missing.push("--confirm-no-raw-readback-persistence");
    }
    if !opt.confirm_no_headers_persistence {
        missing.push("--confirm-no-headers-persistence");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
    }
    if !opt.confirm_no_mutation {
        missing.push("--confirm-no-mutation");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    missing
}

fn missing_production_mutation_cancel_recovery_incident_audit_closeout_cli_flags(
    opt: &LiveProductionMutationCancelRecoveryIncidentAuditCloseoutOpt,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !opt.allow_production_mutation_cancel_recovery_incident_audit_closeout {
        missing.push("--allow-production-mutation-cancel-recovery-incident-audit-closeout");
    }
    if !opt.confirm_cancel_recovery_lineage {
        missing.push("--confirm-cancel-recovery-lineage");
    }
    if !opt.confirm_risk_reason_recorded {
        missing.push("--confirm-risk-reason-recorded");
    }
    if !opt.confirm_risk_gate_result_recorded {
        missing.push("--confirm-risk-gate-result-recorded");
    }
    if !opt.confirm_owner_approval_state_recorded {
        missing.push("--confirm-owner-approval-state-recorded");
    }
    if !opt.confirm_redaction_contract_state_recorded {
        missing.push("--confirm-redaction-contract-state-recorded");
    }
    if !opt.confirm_readback_state_recorded {
        missing.push("--confirm-readback-state-recorded");
    }
    if !opt.confirm_terminal_action_recommendation {
        missing.push("--confirm-terminal-action-recommendation");
    }
    if !opt.confirm_remaining_risk_recorded {
        missing.push("--confirm-remaining-risk-recorded");
    }
    if !opt.confirm_no_mutation {
        missing.push("--confirm-no-mutation");
    }
    if !opt.confirm_no_cancel {
        missing.push("--confirm-no-cancel");
    }
    if !opt.confirm_no_network {
        missing.push("--confirm-no-network");
    }
    if !opt.confirm_no_retry {
        missing.push("--confirm-no-retry");
    }
    if !opt.confirm_no_remediation {
        missing.push("--confirm-no-remediation");
    }
    if !opt.confirm_no_automatic_remediation {
        missing.push("--confirm-no-automatic-remediation");
    }
    if !opt.confirm_dashboard_order_controls_disabled {
        missing.push("--confirm-dashboard-order-controls-disabled");
    }
    if !opt.confirm_no_secret_persistence {
        missing.push("--confirm-no-secret-persistence");
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
#[path = "live/tests.rs"]
mod tests;
