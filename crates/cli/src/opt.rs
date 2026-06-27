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

use std::{net::SocketAddr, path::PathBuf};

use clap::{ArgAction, Parser, ValueEnum};

/// Command-line interface for NTPRO.
#[derive(Debug, Parser)]
#[clap(version, about, author)]
pub struct NautilusCli {
    #[clap(subcommand)]
    pub command: Commands,
}

/// Available top-level commands for the NTPRO CLI.
#[derive(Parser, Debug)]
pub enum Commands {
    Backtest(BacktestOpt),
    Sandbox(SandboxOpt),
    Live(LiveOpt),
    Data(DataOpt),
    Config(ConfigOpt),
    Supervisor(SupervisorOpt),
    Dashboard(DashboardOpt),
    Workflow(WorkflowOpt),
    Database(DatabaseOpt),
    #[cfg(feature = "defi")]
    Blockchain(BlockchainOpt),
}

/// Backtest operations and validation commands.
#[derive(Parser, Debug)]
#[command(about = "Backtest operations", long_about = None)]
pub struct BacktestOpt {
    #[clap(subcommand)]
    pub command: BacktestCommand,
}

/// Available backtest commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Backtest operations", long_about = None)]
pub enum BacktestCommand {
    /// Validates a Rust backtest config without running the engine.
    Validate(BacktestValidateOpt),
    /// Runs a Rust backtest smoke path, or metadata-only dry-run with --dry-run.
    Run(BacktestRunOpt),
}

/// Backtest validation options.
#[derive(Parser, Debug, Clone)]
pub struct BacktestValidateOpt {
    /// Path to the Rust backtest config file.
    #[arg(long)]
    pub config: PathBuf,
}

/// Backtest run options.
#[derive(Parser, Debug, Clone)]
pub struct BacktestRunOpt {
    /// Path to the Rust backtest config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional directory for run artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Writes metadata-only dry-run artifacts without starting the backtest engine.
    #[arg(long)]
    pub dry_run: bool,
}

/// Sandbox live-node operations and validation commands.
#[derive(Parser, Debug)]
#[command(about = "Sandbox live-node operations", long_about = None)]
pub struct SandboxOpt {
    #[clap(subcommand)]
    pub command: SandboxCommand,
}

/// Available sandbox commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Sandbox live-node operations", long_about = None)]
pub enum SandboxCommand {
    /// Validates a Rust sandbox config without starting a node.
    Validate(SandboxValidateOpt),
    /// Writes simulation-only sandbox demo artifacts; use live run for LiveNode smoke.
    Run(SandboxRunOpt),
}

/// Sandbox validation options.
#[derive(Parser, Debug, Clone)]
pub struct SandboxValidateOpt {
    /// Path to the Rust sandbox config file.
    #[arg(long)]
    pub config: PathBuf,
}

/// Sandbox run options.
#[derive(Parser, Debug, Clone)]
pub struct SandboxRunOpt {
    /// Path to the Rust sandbox config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional directory for run artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Local sandbox LiveNode validation and smoke commands.
#[derive(Parser, Debug)]
#[command(
    about = "Local live, production read-only, dry-run proof, and owner-gated mutation-candidate commands",
    long_about = None
)]
pub struct LiveOpt {
    #[clap(subcommand)]
    pub command: LiveCommand,
}

/// Available local live, production read-only, and dry-run proof commands.
#[derive(Parser, Debug, Clone)]
#[command(
    about = "Local live, production read-only, dry-run proof, and owner-gated mutation-candidate commands",
    long_about = None
)]
pub enum LiveCommand {
    /// Validates the Rust live-init smoke config.
    Validate(LiveValidateOpt),
    /// Runs a local sandbox LiveNode start/stop smoke path without external venue access or real orders.
    Run(LiveRunOpt),
    /// Checks the v0.10 manual Binance testnet order gate without opening network or submitting orders.
    TestnetOrderGate(LiveTestnetOrderGateOpt),
    /// Checks local v0.10 Binance testnet order risk preflight without network or orders.
    TestnetOrderPreflight(LiveTestnetOrderPreflightOpt),
    /// Builds a redacted v0.10 Binance testnet order request preview without network or orders.
    TestnetOrderRequestPreview(LiveTestnetOrderRequestPreviewOpt),
    /// Checks the redacted POST /api/v3/order/test preflight without network or orders.
    TestnetOrderTestPreflight(LiveTestnetOrderTestPreflightOpt),
    /// Writes the redacted v0.10 execution artifact contract without network or orders.
    TestnetExecutionArtifactContract(LiveTestnetExecutionArtifactContractOpt),
    /// Writes offline v0.10 reconciliation/orphan-order fixtures without network or orders.
    TestnetReconciliationFixture(LiveTestnetReconciliationFixtureOpt),
    /// Writes a v0.11 production public read-only probe contract; no production mutation.
    ProductionPublicReadProbe(LiveProductionPublicReadProbeOpt),
    /// Writes a v0.11 authenticated production account read-only snapshot; no production mutation.
    ProductionAccountSnapshotContract(LiveProductionAccountSnapshotContractOpt),
    /// Writes a v0.14 owner-gated production order-state read-only proof; no production mutation.
    ProductionOrderStateReadOnlyProof(LiveProductionOrderStateReadOnlyProofOpt),
    /// Writes a v0.14 live-alpha dry-run order gate artifact; no production mutation.
    ProductionLiveAlphaDryRunOrderGate(LiveProductionLiveAlphaDryRunOrderGateOpt),
    /// Builds a v0.15 redacted production live-alpha order request preview; no request execution and no production mutation.
    ProductionLiveAlphaOrderRequestPreview(LiveProductionLiveAlphaOrderRequestPreviewOpt),
    /// Writes a v0.15 one-time manual approval lifecycle artifact for request preview only; no production mutation.
    ProductionLiveAlphaManualApprovalLifecycle(LiveProductionLiveAlphaManualApprovalLifecycleOpt),
    /// Routes live-alpha intent into a local dry-run execution adapter artifact only; no production mutation.
    ProductionLiveAlphaExecutionDryRun(LiveProductionLiveAlphaExecutionDryRunOpt),
    /// Evaluates the v0.15 kill-switch runtime gate before any dry-run mutation progression; no production mutation.
    ProductionLiveAlphaKillSwitchRuntimeGate(LiveProductionLiveAlphaKillSwitchRuntimeGateOpt),
    /// Evaluates v0.16 owner-approved production mutation runtime gates; no request execution.
    ProductionMutationRuntimeGate(LiveProductionMutationRuntimeGateOpt),
    /// Writes a v0.16 owner approval artifact for env-only production signing material; no request execution.
    ProductionMutationSigningApproval(LiveProductionMutationSigningApprovalOpt),
    /// Builds a v0.16 single LIMIT GTC production order request object locally; no request execution.
    ProductionMutationRequestBuilder(LiveProductionMutationRequestBuilderOpt),
    /// Evaluates the v0.16 guarded single-shot production HTTP send path.
    ProductionMutationGuardedSend(LiveProductionMutationGuardedSendOpt),
    /// Redacts a v0.16 production mutation response artifact; no raw response persistence.
    ProductionMutationResponseRedaction(LiveProductionMutationResponseRedactionOpt),
    /// Proves a v0.16 post-submit order-state readback path from known order identifiers.
    ProductionMutationOrderStateReadback(LiveProductionMutationOrderStateReadbackOpt),
    /// Writes a v0.16 redacted production mutation audit trail artifact.
    ProductionMutationAuditTrail(LiveProductionMutationAuditTrailOpt),
    /// Writes v0.16 production mutation failure/no-retry semantics evidence.
    ProductionMutationFailureSemantics(LiveProductionMutationFailureSemanticsOpt),
    /// Writes a v0.17 local production order ledger for one mutation candidate lineage.
    ProductionMutationLocalOrderLedger(LiveProductionMutationLocalOrderLedgerOpt),
    /// Maps v0.17 redacted exchange readback metadata for one mutation candidate lineage.
    ProductionMutationExchangeReadbackMapper(LiveProductionMutationExchangeReadbackMapperOpt),
    /// Classifies v0.17 local-vs-exchange reconciliation for one mutation candidate lineage.
    ProductionMutationReconciliationClassifier(LiveProductionMutationReconciliationClassifierOpt),
    /// Detects v0.17 open/orphan order risk for one mutation candidate lineage.
    ProductionMutationOrphanOrderDetector(LiveProductionMutationOrphanOrderDetectorOpt),
    /// Builds a v0.18 cancel request preview from one v0.17 orphan-risk artifact; no cancel send.
    ProductionMutationCancelRequestPreview(LiveProductionMutationCancelRequestPreviewOpt),
    /// Evaluates a v0.18 cancel risk gate from one cancel preview artifact; no cancel send.
    ProductionMutationCancelRiskGate(LiveProductionMutationCancelRiskGateOpt),
    /// Writes a v0.18 one-time manual owner approval lifecycle artifact for one cancel candidate; no cancel send.
    ProductionMutationManualOwnerApprovalLifecycle(
        LiveProductionMutationManualOwnerApprovalLifecycleOpt,
    ),
    /// Writes a v0.19 single-use owner approval lifecycle artifact for actual cancel authorization; no cancel send.
    ProductionMutationActualCancelOwnerApprovalLifecycle(
        Box<LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt>,
    ),
    /// Writes a v0.19 actual cancel executor adapter boundary artifact; no cancel send.
    ProductionMutationActualCancelExecutorAdapterBoundary(
        Box<LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt>,
    ),
    /// Redacts a future v0.18 owner-approved cancel response artifact; no cancel send.
    ProductionMutationCancelResponseRedaction(LiveProductionMutationCancelResponseRedactionOpt),
    /// Classifies a future v0.18 post-cancel readback artifact; no network read or cancel send.
    ProductionMutationPostCancelReadback(LiveProductionMutationPostCancelReadbackOpt),
    /// Writes a v0.18 cancel recovery incident/audit closeout artifact; no cancel send.
    ProductionMutationCancelRecoveryIncidentAuditCloseout(
        LiveProductionMutationCancelRecoveryIncidentAuditCloseoutOpt,
    ),
    /// Writes a v0.14 hypothetical live-alpha dry-run risk preflight; no production mutation.
    ProductionLiveAlphaRiskPreflight(LiveProductionLiveAlphaRiskPreflightOpt),
    /// Builds local v0.12 shadow portfolio artifacts from read-only inputs; no production mutation.
    ProductionShadowPortfolioRuntime(LiveProductionShadowPortfolioRuntimeOpt),
    /// Writes local v0.12 shadow strategy session events from read-only artifacts; no production mutation.
    ProductionShadowStrategySession(LiveProductionShadowStrategySessionOpt),
    /// Runs a local v0.13 guarded-live-alpha dry-run preflight loop; no production mutation.
    ProductionShadowPreflightSession(LiveProductionShadowPreflightSessionOpt),
    /// Writes a local v0.13 kill-switch dry-run/manual-approval artifact; no production mutation.
    ProductionKillSwitchApprovalArtifact(LiveProductionKillSwitchApprovalArtifactOpt),
    /// Writes local v0.12 production read-only reconciliation events; no production mutation.
    ProductionReadonlyReconciliation(LiveProductionReadonlyReconciliationOpt),
}

/// Live validation options.
#[derive(Parser, Debug, Clone)]
pub struct LiveValidateOpt {
    /// Path to the Rust live config file.
    #[arg(long)]
    pub config: PathBuf,
}

/// Live run options.
#[derive(Parser, Debug, Clone)]
pub struct LiveRunOpt {
    /// Path to the Rust live config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional directory for run artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Binance testnet order gate options.
#[derive(Parser, Debug, Clone)]
pub struct LiveTestnetOrderGateOpt {
    /// Path to the Rust strategy-session config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Manual CLI gate for any Binance testnet order path.
    #[arg(long)]
    pub allow_testnet_order: bool,
    /// Confirms owner approval for the manual Binance testnet order proof.
    #[arg(long)]
    pub confirm_owner_approved_testnet_order: bool,
    /// Confirms the configured testnet order is tiny-notional only.
    #[arg(long)]
    pub confirm_tiny_notional: bool,
    /// Confirms the proof must cancel immediately after submit ack.
    #[arg(long)]
    pub confirm_cancel_after_submit: bool,
}

/// Binance testnet order preflight options.
#[derive(Parser, Debug, Clone)]
pub struct LiveTestnetOrderPreflightOpt {
    /// Path to the Rust strategy-session config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Path to the local order preflight input JSON.
    #[arg(long)]
    pub input: PathBuf,
    /// Optional JSON report output path.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Manual CLI gate for any Binance testnet order path.
    #[arg(long)]
    pub allow_testnet_order: bool,
    /// Confirms owner approval for the manual Binance testnet order proof.
    #[arg(long)]
    pub confirm_owner_approved_testnet_order: bool,
    /// Confirms the configured testnet order is tiny-notional only.
    #[arg(long)]
    pub confirm_tiny_notional: bool,
    /// Confirms the proof must cancel immediately after submit ack.
    #[arg(long)]
    pub confirm_cancel_after_submit: bool,
}

/// Binance testnet signed order request preview options.
#[derive(Parser, Debug, Clone)]
pub struct LiveTestnetOrderRequestPreviewOpt {
    /// Path to the Rust strategy-session config file.
    #[arg(long)]
    pub config: PathBuf,
    /// HTTP method for the signed order request preview.
    #[arg(long, default_value = "POST")]
    pub method: String,
    /// Binance order endpoint path to preview.
    #[arg(long, default_value = "/api/v3/order/test")]
    pub endpoint_path: String,
    /// Timestamp in milliseconds for deterministic signing.
    #[arg(long)]
    pub timestamp_ms: u64,
    /// Binance recvWindow in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Environment variable name containing the Binance testnet API key.
    #[arg(long, default_value = "BINANCE_TESTNET_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the Binance testnet API secret.
    #[arg(long, default_value = "BINANCE_TESTNET_API_SECRET")]
    pub api_secret_env: String,
    /// Optional original client order id for DELETE /api/v3/order previews.
    #[arg(long)]
    pub orig_client_order_id: Option<String>,
    /// Optional JSON report output path.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Manual CLI gate for any Binance testnet order path.
    #[arg(long)]
    pub allow_testnet_order: bool,
    /// Confirms owner approval for the manual Binance testnet order proof.
    #[arg(long)]
    pub confirm_owner_approved_testnet_order: bool,
    /// Confirms the configured testnet order is tiny-notional only.
    #[arg(long)]
    pub confirm_tiny_notional: bool,
    /// Confirms the proof must cancel immediately after submit ack.
    #[arg(long)]
    pub confirm_cancel_after_submit: bool,
}

/// Binance testnet POST /api/v3/order/test preflight options.
#[derive(Parser, Debug, Clone)]
pub struct LiveTestnetOrderTestPreflightOpt {
    /// Path to the Rust strategy-session config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Timestamp in milliseconds for deterministic signing.
    #[arg(long)]
    pub timestamp_ms: u64,
    /// Binance recvWindow in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Environment variable name containing the Binance testnet API key.
    #[arg(long, default_value = "BINANCE_TESTNET_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the Binance testnet API secret.
    #[arg(long, default_value = "BINANCE_TESTNET_API_SECRET")]
    pub api_secret_env: String,
    /// Optional JSON report output path.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Manual CLI gate for any Binance testnet order path.
    #[arg(long)]
    pub allow_testnet_order: bool,
    /// Confirms owner approval for the manual Binance testnet order proof.
    #[arg(long)]
    pub confirm_owner_approved_testnet_order: bool,
    /// Confirms the configured testnet order is tiny-notional only.
    #[arg(long)]
    pub confirm_tiny_notional: bool,
    /// Confirms the proof must cancel immediately after submit ack.
    #[arg(long)]
    pub confirm_cancel_after_submit: bool,
}

/// Binance testnet execution artifact contract options.
#[derive(Parser, Debug, Clone)]
pub struct LiveTestnetExecutionArtifactContractOpt {
    /// Path to the Rust strategy-session config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Timestamp in milliseconds for deterministic signing.
    #[arg(long)]
    pub timestamp_ms: u64,
    /// Binance recvWindow in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Environment variable name containing the Binance testnet API key.
    #[arg(long, default_value = "BINANCE_TESTNET_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the Binance testnet API secret.
    #[arg(long, default_value = "BINANCE_TESTNET_API_SECRET")]
    pub api_secret_env: String,
    /// Synthetic cancel client order id used only for the redacted contract.
    #[arg(long, default_value = "ntpro-v100-artifact-contract-only")]
    pub orig_client_order_id: String,
    /// Optional JSON report output path.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Manual CLI gate for any Binance testnet order path.
    #[arg(long)]
    pub allow_testnet_order: bool,
    /// Confirms owner approval for the manual Binance testnet order proof.
    #[arg(long)]
    pub confirm_owner_approved_testnet_order: bool,
    /// Confirms the configured testnet order is tiny-notional only.
    #[arg(long)]
    pub confirm_tiny_notional: bool,
    /// Confirms the proof must cancel immediately after submit ack.
    #[arg(long)]
    pub confirm_cancel_after_submit: bool,
}

/// Offline Binance testnet reconciliation fixture options.
#[derive(Parser, Debug, Clone)]
pub struct LiveTestnetReconciliationFixtureOpt {
    /// Path to the Rust strategy-session config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Reconciliation scenario to render.
    #[arg(long, value_enum, default_value_t = TestnetReconciliationScenario::All)]
    pub scenario: TestnetReconciliationScenario,
    /// Optional JSON report output path.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Offline Binance testnet reconciliation fixture scenarios.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum TestnetReconciliationScenario {
    /// Render every scoped inconsistent-state fixture.
    All,
    /// Submit was sent but no local submit ack was recorded.
    SubmitWithoutLocalAck,
    /// Cancel request timed out before terminal confirmation.
    CancelTimeout,
    /// Local state is open while exchange state is filled.
    LocalOpenExchangeFilled,
    /// Process restarted with unfinished testnet order state.
    RestartUnfinishedOrder,
}

/// Production public read-only probe contract options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionPublicReadProbeOpt {
    /// Production public read-only endpoint to classify.
    #[arg(long, value_enum, default_value_t = ProductionPublicReadEndpoint::ServerTime)]
    pub endpoint: ProductionPublicReadEndpoint,
    /// Optional JSON report output path.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Requests the future manual online read path. V110-002 records this as blocked.
    #[arg(long)]
    pub manual_online: bool,
    /// Manual CLI gate for any production public read path.
    #[arg(long)]
    pub allow_production_public_read: bool,
    /// Confirms the probe is read-only and must not use signed/authenticated endpoints.
    #[arg(long)]
    pub confirm_read_only: bool,
    /// Confirms production order submission or mutation remains forbidden.
    #[arg(long)]
    pub confirm_no_order_mutation: bool,
}

/// v0.11 production public read-only probe endpoints.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ProductionPublicReadEndpoint {
    /// `GET /api/v3/time`.
    ServerTime,
    /// `GET /api/v3/exchangeInfo`.
    ExchangeInfo,
}

/// v0.14 production order-state read-only proof endpoints.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ProductionOrderStateReadEndpoint {
    /// `GET /api/v3/openOrders`.
    OpenOrders,
    /// `GET /api/v3/order`.
    Order,
}

/// Authenticated production account snapshot contract options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionAccountSnapshotContractOpt {
    /// Optional JSON report output path.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Requests the future manual online account read path. V110-003 records this as blocked.
    #[arg(long)]
    pub manual_online: bool,
    /// Environment variable name containing the production read-only API key.
    #[arg(long, default_value = "BINANCE_PRODUCTION_READONLY_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the production read-only API secret.
    #[arg(long, default_value = "BINANCE_PRODUCTION_READONLY_API_SECRET")]
    pub api_secret_env: String,
    /// Binance recvWindow in milliseconds for the future signed read shape.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Manual CLI gate for authenticated production read-only account snapshots.
    #[arg(long)]
    pub allow_production_authenticated_read: bool,
    /// Confirms owner approval for authenticated production read-only evidence.
    #[arg(long)]
    pub confirm_owner_approved_read_only: bool,
    /// Confirms production order submission or mutation remains forbidden.
    #[arg(long)]
    pub confirm_no_order_mutation: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL must not be persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// Owner-gated production order-state read-only proof options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionOrderStateReadOnlyProofOpt {
    /// Production order-state read endpoint to classify.
    #[arg(long, value_enum, default_value_t = ProductionOrderStateReadEndpoint::OpenOrders)]
    pub endpoint: ProductionOrderStateReadEndpoint,
    /// Binance symbol used to bound the order-state read.
    #[arg(long, default_value = "BTCUSDT")]
    pub symbol: String,
    /// Optional Binance order id for `GET /api/v3/order`.
    #[arg(long)]
    pub order_id: Option<u64>,
    /// Optional original client order id for `GET /api/v3/order`.
    #[arg(long)]
    pub orig_client_order_id: Option<String>,
    /// Optional JSON report output path.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Requests the manual online order-state read path.
    #[arg(long)]
    pub manual_online: bool,
    /// Environment variable name containing the production read-only API key.
    #[arg(long, default_value = "BINANCE_PRODUCTION_READONLY_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the production read-only API secret.
    #[arg(long, default_value = "BINANCE_PRODUCTION_READONLY_API_SECRET")]
    pub api_secret_env: String,
    /// Binance recvWindow in milliseconds for the signed read.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Manual CLI gate for production order-state reads.
    #[arg(long)]
    pub allow_production_order_state_read: bool,
    /// Confirms owner approval for production order-state read-only evidence.
    #[arg(long)]
    pub confirm_owner_approved_read_only: bool,
    /// Confirms production order submission or mutation remains forbidden.
    #[arg(long)]
    pub confirm_no_order_mutation: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL must not be persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
}

/// Owner-gated live-alpha dry-run order gate options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionLiveAlphaDryRunOrderGateOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// Optional owner-visible session identifier; defaults to run_id.
    #[arg(long)]
    pub session_id: Option<String>,
    /// Owner-visible strategy identifier.
    #[arg(long, default_value = "ema_cross_btcusdt_v1")]
    pub strategy_id: String,
    /// Binance symbol used to describe the dry-run order intent.
    #[arg(long, default_value = "BTCUSDT")]
    pub symbol: String,
    /// Dry-run order side.
    #[arg(long, default_value = "BUY")]
    pub side: String,
    /// Dry-run order type.
    #[arg(long, default_value = "LIMIT")]
    pub order_type: String,
    /// Dry-run quantity as a decimal string.
    #[arg(long, default_value = "0.001")]
    pub quantity: String,
    /// Dry-run notional as a decimal string.
    #[arg(long, default_value = "10.00")]
    pub notional: String,
    /// v0.14 dry-run order gate JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for live-alpha dry-run order evidence.
    #[arg(long)]
    pub allow_production_live_alpha_dry_run: bool,
    /// Confirms owner approval for the dry-run evidence.
    #[arg(long)]
    pub confirm_owner_approved_dry_run: bool,
    /// Confirms no production order submission is allowed.
    #[arg(long)]
    pub confirm_no_production_order_submission: bool,
    /// Confirms no production order mutation is allowed.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms no execution adapter call is allowed.
    #[arg(long)]
    pub confirm_no_execution_adapter_call: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms no real funds or real orders are involved.
    #[arg(long)]
    pub confirm_no_real_funds: bool,
}

/// Owner-gated production live-alpha order request preview options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionLiveAlphaOrderRequestPreviewOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.14/v0.15 dry-run order gate JSON input.
    #[arg(long)]
    pub order_gate: PathBuf,
    /// v0.15 one-time manual approval lifecycle JSON input.
    #[arg(long)]
    pub manual_approval_lifecycle: PathBuf,
    /// Production order endpoint path to preview.
    #[arg(long, default_value = "/api/v3/order")]
    pub endpoint_path: String,
    /// Limit order price as a decimal string.
    #[arg(long)]
    pub price: String,
    /// Limit order time-in-force.
    #[arg(long, default_value = "GTC")]
    pub time_in_force: String,
    /// Timestamp in milliseconds for deterministic memory-only signing.
    #[arg(long)]
    pub timestamp_ms: u64,
    /// Binance recvWindow in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Environment variable name containing the Binance production API key.
    #[arg(long, default_value = "BINANCE_PRODUCTION_LIVE_ALPHA_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the Binance production API secret.
    #[arg(long, default_value = "BINANCE_PRODUCTION_LIVE_ALPHA_API_SECRET")]
    pub api_secret_env: String,
    /// Signing material used for dry-run request preview.
    #[arg(
        long,
        default_value = "synthetic",
        value_parser = ["synthetic", "production_live_alpha"]
    )]
    pub credential_material: String,
    /// v0.15 redacted request preview JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for production live-alpha request preview only.
    #[arg(long)]
    pub allow_production_live_alpha_request_preview: bool,
    /// Confirms owner approval for request preview only.
    #[arg(long)]
    pub confirm_owner_approved_request_preview: bool,
    /// Confirms signatures and signed queries remain memory-only.
    #[arg(long)]
    pub confirm_memory_only_signature: bool,
    /// Confirms no production order submission is allowed.
    #[arg(long)]
    pub confirm_no_production_order_submission: bool,
    /// Confirms no production order mutation is allowed.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms no execution adapter call is allowed.
    #[arg(long)]
    pub confirm_no_execution_adapter_call: bool,
    /// Confirms no network request is allowed.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms no real funds or real orders are involved.
    #[arg(long)]
    pub confirm_no_real_funds: bool,
}

/// One-time manual approval lifecycle artifact options for request preview only.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionLiveAlphaManualApprovalLifecycleOpt {
    /// Owner-visible run identifier that the approval binds to.
    #[arg(long)]
    pub run_id: String,
    /// Owner-visible strategy identifier that the approval binds to.
    #[arg(long, default_value = "ema_cross_btcusdt_v1")]
    pub strategy_id: String,
    /// Symbol that the approval binds to.
    #[arg(long, default_value = "BTCUSDT")]
    pub symbol: String,
    /// Dry-run notional that the approval binds to.
    #[arg(long)]
    pub notional: String,
    /// Manual approval state: pending, approved, expired, revoked, or used.
    #[arg(long, default_value = "pending")]
    pub approval_state: String,
    /// Optional owner approval identifier for non-pending states.
    #[arg(long)]
    pub manual_approval_id: Option<String>,
    /// Optional owner/operator name for non-pending states.
    #[arg(long)]
    pub approved_by: Option<String>,
    /// Deterministic current time in milliseconds for lifecycle evaluation.
    #[arg(long)]
    pub now_unix_ms: u64,
    /// Approval expiry in milliseconds.
    #[arg(long)]
    pub expires_at_unix_ms: u64,
    /// v0.15 manual approval lifecycle JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Confirms this approval is scoped to request preview only.
    #[arg(long)]
    pub confirm_dry_run_request_preview_only: bool,
    /// Confirms the approval is one-time use only.
    #[arg(long)]
    pub confirm_one_time_approval: bool,
    /// Confirms no production order submission or mutation is allowed.
    #[arg(long)]
    pub confirm_no_production_mutation: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
}

/// Owner-gated production live-alpha execution dry-run isolation options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionLiveAlphaExecutionDryRunOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.14 dry-run order gate JSON input.
    #[arg(long)]
    pub order_gate: PathBuf,
    /// v0.14 risk preflight JSON input.
    #[arg(long)]
    pub risk_preflight: PathBuf,
    /// v0.15 request preview JSON input.
    #[arg(long)]
    pub request_preview: PathBuf,
    /// v0.15 kill-switch runtime gate JSON input.
    #[arg(long)]
    pub kill_switch_runtime_gate: PathBuf,
    /// v0.15 execution dry-run isolation JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for production live-alpha execution dry-run only.
    #[arg(long)]
    pub allow_production_live_alpha_execution_dry_run: bool,
    /// Confirms owner approval for execution dry-run only.
    #[arg(long)]
    pub confirm_owner_approved_execution_dry_run: bool,
    /// Confirms the local dry-run execution adapter is the only reachable adapter.
    #[arg(long)]
    pub confirm_dry_run_adapter_only: bool,
    /// Confirms no production execution adapter is instantiated or called.
    #[arg(long)]
    pub confirm_no_production_adapter: bool,
    /// Confirms no production order submission is allowed.
    #[arg(long)]
    pub confirm_no_production_order_submission: bool,
    /// Confirms no production order mutation is allowed.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms no network request is allowed.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms no real funds or real orders are involved.
    #[arg(long)]
    pub confirm_no_real_funds: bool,
}

/// Owner-gated production live-alpha kill-switch runtime gate options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionLiveAlphaKillSwitchRuntimeGateOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.13 kill-switch/manual-approval JSON input.
    #[arg(long)]
    pub kill_switch_approval: PathBuf,
    /// v0.14 risk preflight JSON input.
    #[arg(long)]
    pub risk_preflight: PathBuf,
    /// v0.15 request preview JSON input.
    #[arg(long)]
    pub request_preview: PathBuf,
    /// v0.15 kill-switch runtime gate JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for production live-alpha kill-switch runtime gate only.
    #[arg(long)]
    pub allow_production_live_alpha_kill_switch_runtime_gate: bool,
    /// Confirms owner approval for the runtime gate only.
    #[arg(long)]
    pub confirm_owner_approved_runtime_gate: bool,
    /// Confirms no production order submission is allowed.
    #[arg(long)]
    pub confirm_no_production_order_submission: bool,
    /// Confirms no production order mutation is allowed.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms no network request is allowed.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms no real funds or real orders are involved.
    #[arg(long)]
    pub confirm_no_real_funds: bool,
}

/// Owner-approved v0.16 production mutation runtime gate options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationRuntimeGateOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.14/v0.15 dry-run order gate JSON input.
    #[arg(long)]
    pub order_gate: PathBuf,
    /// v0.14 risk preflight JSON input.
    #[arg(long)]
    pub risk_preflight: PathBuf,
    /// v0.15 request preview JSON input.
    #[arg(long)]
    pub request_preview: PathBuf,
    /// v0.15 kill-switch runtime gate JSON input.
    #[arg(long)]
    pub kill_switch_runtime_gate: PathBuf,
    /// Optional v0.16 production signing material approval JSON input.
    #[arg(long)]
    pub signing_approval: Option<PathBuf>,
    /// v0.16 production mutation runtime gate JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Maximum allowed tiny order notional for this runtime gate.
    #[arg(long, default_value = "10.00")]
    pub max_notional: String,
    /// Manual CLI gate for v0.16 production mutation runtime-gate evaluation.
    #[arg(long)]
    pub allow_production_mutation_runtime_gate: bool,
    /// Confirms owner approval is required immediately before any send.
    #[arg(long)]
    pub confirm_owner_approved_production_mutation: bool,
    /// Confirms the candidate is a single LIMIT GTC order only.
    #[arg(long)]
    pub confirm_single_limit_gtc: bool,
    /// Confirms the candidate notional is tiny and owner-capped.
    #[arg(long)]
    pub confirm_tiny_notional: bool,
    /// Confirms a separate signing approval artifact is required before send.
    #[arg(long)]
    pub confirm_signing_approval_required: bool,
    /// Confirms no network request can occur before the explicit send gate.
    #[arg(long)]
    pub confirm_no_network_before_send: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
    /// Confirms retry, correction, cancel, replace, amend, and flatten remain forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
}

/// Owner-approved v0.16 production signing material approval artifact options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationSigningApprovalOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.15 production live-alpha request preview JSON input.
    #[arg(long)]
    pub request_preview: PathBuf,
    /// Approval state: pending, approved, expired, or revoked.
    #[arg(long, default_value = "pending")]
    pub approval_state: String,
    /// Optional owner approval identifier for approved/expired/revoked states.
    #[arg(long)]
    pub manual_approval_id: Option<String>,
    /// Optional owner/operator name for approved/expired/revoked states.
    #[arg(long)]
    pub approved_by: Option<String>,
    /// Deterministic current time in milliseconds for lifecycle evaluation.
    #[arg(long)]
    pub now_unix_ms: u64,
    /// Signing approval expiry in milliseconds.
    #[arg(long)]
    pub expires_at_unix_ms: u64,
    /// v0.16 signing approval JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for production signing approval artifact creation.
    #[arg(long)]
    pub allow_production_mutation_signing_approval: bool,
    /// Confirms owner approved production_live_alpha signing material for this candidate.
    #[arg(long)]
    pub confirm_owner_approved_signing_material: bool,
    /// Confirms signing material remains env-only and is not stored.
    #[arg(long)]
    pub confirm_env_only_signing_material: bool,
    /// Confirms signatures and signed queries remain memory-only.
    #[arg(long)]
    pub confirm_memory_only_signing: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms no network request is allowed by this approval artifact.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms no production order submission is allowed by this approval artifact.
    #[arg(long)]
    pub confirm_no_production_order_submission: bool,
    /// Confirms no production order mutation is allowed by this approval artifact.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
}

/// Owner-approved v0.16 single LIMIT GTC request builder options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationRequestBuilderOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.16 production mutation runtime gate JSON input.
    #[arg(long)]
    pub runtime_gate: PathBuf,
    /// v0.16 production signing material approval JSON input.
    #[arg(long)]
    pub signing_approval: PathBuf,
    /// v0.15 production live-alpha request preview JSON input.
    #[arg(long)]
    pub request_preview: PathBuf,
    /// Environment variable name containing the Binance production API key.
    #[arg(long, default_value = "BINANCE_PRODUCTION_LIVE_ALPHA_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the Binance production API secret.
    #[arg(long, default_value = "BINANCE_PRODUCTION_LIVE_ALPHA_API_SECRET")]
    pub api_secret_env: String,
    /// Timestamp in milliseconds for deterministic memory-only signing.
    #[arg(long)]
    pub timestamp_ms: u64,
    /// Binance recvWindow in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Maximum allowed tiny order notional for this request builder.
    #[arg(long, default_value = "10.00")]
    pub max_notional: String,
    /// Market reference source used to prove the LIMIT price is not marketable.
    #[arg(long, default_value = "")]
    pub market_reference_source: String,
    /// Market reference price used for non-marketable LIMIT price safety checks.
    #[arg(long, default_value = "")]
    pub market_reference_price: String,
    /// Maximum allowed LIMIT price distance from the reference in basis points.
    #[arg(long, default_value = "50")]
    pub max_reference_price_distance_bps: String,
    /// Marks that the LIMIT price would cross the spread if the spread check is available.
    #[arg(long, default_value_t = false)]
    pub would_cross_spread: bool,
    /// v0.16 request builder JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for production mutation request object building.
    #[arg(long)]
    pub allow_production_mutation_request_builder: bool,
    /// Confirms owner approved building the redacted request object.
    #[arg(long)]
    pub confirm_owner_approved_request_builder: bool,
    /// Confirms the candidate is a single LIMIT GTC order only.
    #[arg(long)]
    pub confirm_single_limit_gtc: bool,
    /// Confirms the candidate notional is tiny and owner-capped.
    #[arg(long)]
    pub confirm_tiny_notional: bool,
    /// Confirms the LIMIT price passed the non-marketable price preflight.
    #[arg(long)]
    pub confirm_non_marketable_price: bool,
    /// Confirms the owner acknowledges v0.16 has no automatic cancel path.
    #[arg(long)]
    pub confirm_owner_acknowledged_no_cancel_path: bool,
    /// Confirms the signing approval artifact is required and ready.
    #[arg(long)]
    pub confirm_signing_approval_ready: bool,
    /// Confirms signatures and signed queries remain memory-only.
    #[arg(long)]
    pub confirm_memory_only_signing: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms no network request is allowed by this builder.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms no production order submission is allowed by this builder.
    #[arg(long)]
    pub confirm_no_production_order_submission: bool,
    /// Confirms no production order mutation is allowed by this builder.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
    /// Confirms retry, correction, cancel, replace, amend, and flatten remain forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
}

/// Owner-approved v0.16 guarded production HTTP send options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationGuardedSendOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.16 request builder JSON input.
    #[arg(long)]
    pub request_builder: PathBuf,
    /// v0.15 kill-switch runtime gate JSON input checked around send.
    #[arg(long)]
    pub kill_switch_runtime_gate: PathBuf,
    /// v0.15 production live-alpha request preview JSON input.
    #[arg(long)]
    pub request_preview: PathBuf,
    /// Environment variable name containing the Binance production API key.
    #[arg(long, default_value = "BINANCE_PRODUCTION_LIVE_ALPHA_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the Binance production API secret.
    #[arg(long, default_value = "BINANCE_PRODUCTION_LIVE_ALPHA_API_SECRET")]
    pub api_secret_env: String,
    /// Timestamp in milliseconds for deterministic memory-only signing.
    #[arg(long)]
    pub timestamp_ms: u64,
    /// Binance recvWindow in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Maximum allowed tiny order notional for this guarded send path.
    #[arg(long, default_value = "10.00")]
    pub max_notional: String,
    /// v0.16 guarded send JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Requests the manual online single-shot send path.
    #[arg(long)]
    pub manual_online: bool,
    /// Manual CLI gate for guarded production send evaluation.
    #[arg(long)]
    pub allow_production_mutation_guarded_send: bool,
    /// Confirms owner approved this guarded send evaluation.
    #[arg(long)]
    pub confirm_owner_approved_guarded_send: bool,
    /// Confirms the candidate is a single LIMIT GTC order only.
    #[arg(long)]
    pub confirm_single_limit_gtc: bool,
    /// Confirms the candidate notional is tiny and owner-capped.
    #[arg(long)]
    pub confirm_tiny_notional: bool,
    /// Confirms this path is single-shot only.
    #[arg(long)]
    pub confirm_single_shot: bool,
    /// Confirms no retry, correction, cancel, replace, amend, or flatten is allowed.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms API key, secret, signature, signed query, signed URL, raw response, and raw body are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms raw exchange response must not be persisted.
    #[arg(long)]
    pub confirm_response_redacted: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
}

/// Owner-approved v0.16 production mutation response redaction options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationResponseRedactionOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.16 guarded-send JSON input.
    #[arg(long)]
    pub guarded_send: PathBuf,
    /// Synthetic or manually supplied production mutation response JSON input.
    #[arg(long)]
    pub response: PathBuf,
    /// v0.16 response redaction JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for response redaction contract evaluation.
    #[arg(long)]
    pub allow_production_mutation_response_redaction: bool,
    /// Confirms owner approved response redaction contract evaluation.
    #[arg(long)]
    pub confirm_owner_approved_response_redaction: bool,
    /// Confirms raw response bodies must not be persisted.
    #[arg(long)]
    pub confirm_no_raw_response_persistence: bool,
    /// Confirms HTTP headers must not be persisted.
    #[arg(long)]
    pub confirm_no_headers_persistence: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms only order metadata fields are allowed.
    #[arg(long)]
    pub confirm_order_metadata_only: bool,
    /// Confirms account balances are not persisted in the response artifact.
    #[arg(long)]
    pub confirm_no_account_balances: bool,
    /// Confirms unrestricted payload capture is forbidden.
    #[arg(long)]
    pub confirm_no_unrestricted_payload: bool,
    /// Confirms retry, correction, cancel, replace, amend, and flatten remain forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
}

/// Owner-approved v0.16 post-submit order-state readback options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationOrderStateReadbackOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.16 response redaction JSON input.
    #[arg(long)]
    pub response_redaction: PathBuf,
    /// v0.16 order-state readback JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Requests the manual online readback path.
    #[arg(long)]
    pub manual_online: bool,
    /// Environment variable name containing the production read-only API key.
    #[arg(long, default_value = "BINANCE_PRODUCTION_READONLY_API_KEY")]
    pub api_key_env: String,
    /// Environment variable name containing the production read-only API secret.
    #[arg(long, default_value = "BINANCE_PRODUCTION_READONLY_API_SECRET")]
    pub api_secret_env: String,
    /// Binance recvWindow in milliseconds for the signed readback.
    #[arg(long, default_value_t = 5_000)]
    pub recv_window_ms: u64,
    /// Manual CLI gate for production mutation order-state readback.
    #[arg(long)]
    pub allow_production_mutation_order_state_readback: bool,
    /// Confirms owner approved the order-state readback proof.
    #[arg(long)]
    pub confirm_owner_approved_order_state_readback: bool,
    /// Confirms readback uses only known order identifiers from the mutation artifact.
    #[arg(long)]
    pub confirm_known_order_identifier_only: bool,
    /// Confirms only GET /api/v3/order is allowed.
    #[arg(long)]
    pub confirm_read_only_get_order: bool,
    /// Confirms readback response metadata remains redacted.
    #[arg(long)]
    pub confirm_response_redacted: bool,
    /// Confirms production order submission or mutation remains forbidden.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms retry, correction, cancel, replace, amend, and flatten remain forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
}

/// Owner-approved v0.16 production mutation audit trail options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationAuditTrailOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.16 request builder JSON input.
    #[arg(long)]
    pub request_builder: PathBuf,
    /// v0.16 guarded-send JSON input.
    #[arg(long)]
    pub guarded_send: PathBuf,
    /// v0.16 response-redaction JSON input.
    #[arg(long)]
    pub response_redaction: PathBuf,
    /// v0.16 order-state readback JSON input.
    #[arg(long)]
    pub order_state_readback: PathBuf,
    /// v0.16 audit trail JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for production mutation audit trail evaluation.
    #[arg(long)]
    pub allow_production_mutation_audit_trail: bool,
    /// Confirms owner approved audit trail creation for this candidate.
    #[arg(long)]
    pub confirm_owner_approved_audit_trail: bool,
    /// Confirms request, response, and readback artifacts are redacted.
    #[arg(long)]
    pub confirm_redacted_artifacts_only: bool,
    /// Confirms no raw secrets, signatures, signed URLs, or raw payloads are persisted.
    #[arg(long)]
    pub confirm_no_secret_or_raw_payload_persistence: bool,
    /// Confirms retry, cancel, replace, amend, correction, and flatten remain forbidden.
    #[arg(long)]
    pub confirm_no_retry_or_followup_mutation: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
}

/// v0.16 production mutation failure mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProductionMutationFailureMode {
    #[value(name = "timeout")]
    Timeout,
    #[value(name = "http-4xx")]
    Http4xx,
    #[value(name = "http-5xx")]
    Http5xx,
    #[value(name = "malformed-response")]
    MalformedResponse,
    #[value(name = "readback-mismatch")]
    ReadbackMismatch,
    #[value(name = "kill-switch-transition")]
    KillSwitchTransition,
}

/// Owner-approved v0.16 production mutation failure/no-retry semantics options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationFailureSemanticsOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.16 audit trail JSON input.
    #[arg(long)]
    pub audit_trail: PathBuf,
    /// Simulated or observed failure mode to classify.
    #[arg(long, value_enum)]
    pub failure_mode: ProductionMutationFailureMode,
    /// v0.16 failure semantics JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for production mutation failure semantics evaluation.
    #[arg(long)]
    pub allow_production_mutation_failure_semantics: bool,
    /// Confirms failure handling writes evidence only and stops.
    #[arg(long)]
    pub confirm_evidence_only_failure_handling: bool,
    /// Confirms no retry is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms no automatic cancel, replace, or amend is attempted.
    #[arg(long)]
    pub confirm_no_automatic_cancel_replace_amend: bool,
    /// Confirms no correction or flatten remediation is attempted.
    #[arg(long)]
    pub confirm_no_correction_or_flatten: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms strategy execution does not continue after failure evidence.
    #[arg(long)]
    pub confirm_no_strategy_continuation: bool,
    /// Confirms listenKey lifecycle remains forbidden.
    #[arg(long)]
    pub confirm_no_listen_key_lifecycle: bool,
}

/// v0.17 local production order ledger options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationLocalOrderLedgerOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// Stable local lineage identifier for one v0.16 mutation candidate.
    #[arg(long)]
    pub order_lineage_id: String,
    /// v0.16 request builder JSON input.
    #[arg(long)]
    pub request_builder: PathBuf,
    /// v0.16 guarded-send JSON input.
    #[arg(long)]
    pub guarded_send: PathBuf,
    /// v0.16 response-redaction JSON input.
    #[arg(long)]
    pub response_redaction: PathBuf,
    /// v0.16 order-state readback JSON input.
    #[arg(long)]
    pub order_state_readback: PathBuf,
    /// v0.16 audit trail JSON input.
    #[arg(long)]
    pub audit_trail: PathBuf,
    /// v0.16 failure semantics JSON input.
    #[arg(long)]
    pub failure_semantics: PathBuf,
    /// v0.17 local order ledger JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for local production order ledger creation.
    #[arg(long)]
    pub allow_production_mutation_local_order_ledger: bool,
    /// Confirms the ledger covers one v0.16 mutation candidate lineage only.
    #[arg(long)]
    pub confirm_single_v16_mutation_candidate_lineage: bool,
    /// Confirms v0.17 reconciliation is read-only evidence only.
    #[arg(long)]
    pub confirm_read_only_reconciliation_scope: bool,
    /// Confirms the ledger is local/offline and performs no network calls.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms no duplicate submit is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_duplicate_submit: bool,
    /// Confirms no retry is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms no cancel is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no replace, amend, correction, flatten, or remediation is attempted.
    #[arg(long)]
    pub confirm_no_remediation: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// v0.17 exchange readback mapper options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationExchangeReadbackMapperOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.17 local order ledger JSON input.
    #[arg(long)]
    pub local_order_ledger: PathBuf,
    /// Redacted GET /api/v3/order readback metadata JSON input.
    #[arg(long)]
    pub order_readback: PathBuf,
    /// Redacted GET /api/v3/openOrders readback metadata JSON input.
    #[arg(long)]
    pub open_orders_readback: PathBuf,
    /// v0.17 exchange readback mapper JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for exchange readback mapping.
    #[arg(long)]
    pub allow_production_mutation_exchange_readback_mapper: bool,
    /// Confirms readback metadata is already redacted.
    #[arg(long)]
    pub confirm_redacted_readback_metadata_only: bool,
    /// Confirms mapping uses only known order identifiers from the local lineage.
    #[arg(long)]
    pub confirm_known_order_identifier_only: bool,
    /// Confirms v0.17 reconciliation is read-only evidence only.
    #[arg(long)]
    pub confirm_read_only_reconciliation_scope: bool,
    /// Confirms mapper is local/offline and performs no network calls.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms production order submission and mutation remain forbidden.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms no retry is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms no cancel is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
}

/// v0.17 reconciliation classifier options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationReconciliationClassifierOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.17 exchange readback mapper JSON input.
    #[arg(long)]
    pub exchange_readback_mapper: PathBuf,
    /// v0.17 reconciliation classifier JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for reconciliation classification.
    #[arg(long)]
    pub allow_production_mutation_reconciliation_classifier: bool,
    /// Confirms classification is limited to one v0.16 mutation candidate lineage.
    #[arg(long)]
    pub confirm_single_v16_mutation_candidate_lineage: bool,
    /// Confirms v0.17 reconciliation is read-only evidence only.
    #[arg(long)]
    pub confirm_read_only_reconciliation_scope: bool,
    /// Confirms no retry is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms no cancel is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no replace, amend, correction, flatten, or remediation is attempted.
    #[arg(long)]
    pub confirm_no_remediation: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// v0.17 orphan order detector options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationOrphanOrderDetectorOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.17 reconciliation classifier JSON input.
    #[arg(long)]
    pub reconciliation_classifier: PathBuf,
    /// v0.17 orphan order detector JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for orphan order detection.
    #[arg(long)]
    pub allow_production_mutation_orphan_order_detector: bool,
    /// Confirms detection is limited to one v0.16 mutation candidate lineage.
    #[arg(long)]
    pub confirm_single_v16_mutation_candidate_lineage: bool,
    /// Confirms v0.17 orphan detection is read-only evidence only.
    #[arg(long)]
    pub confirm_read_only_reconciliation_scope: bool,
    /// Confirms no retry is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms no cancel is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no replace, amend, correction, flatten, or remediation is attempted.
    #[arg(long)]
    pub confirm_no_remediation: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// v0.18 cancel request preview options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationCancelRequestPreviewOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.17 orphan order detector JSON input.
    #[arg(long)]
    pub orphan_order_detector: PathBuf,
    /// Owner-selected account label for the single cancel candidate scope.
    #[arg(long)]
    pub account_label: String,
    /// v0.18 cancel request preview JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for cancel request preview generation.
    #[arg(long)]
    pub allow_production_mutation_cancel_request_preview: bool,
    /// Confirms preview is limited to one v0.16 mutation candidate lineage.
    #[arg(long)]
    pub confirm_single_v16_mutation_candidate_lineage: bool,
    /// Confirms the source orphan detector halted risk and blocked new orders.
    #[arg(long)]
    pub confirm_orphan_risk_halted: bool,
    /// Confirms owner/manual review remains required before any future cancel scope.
    #[arg(long)]
    pub confirm_manual_review_required: bool,
    /// Confirms the preview uses known order identifiers only.
    #[arg(long)]
    pub confirm_known_order_identifier_only: bool,
    /// Confirms no retry is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms no cancel is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no network endpoint is attempted.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms no replace, amend, correction, flatten, or remediation is attempted.
    #[arg(long)]
    pub confirm_no_remediation: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// v0.18 cancel risk gate options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationCancelRiskGateOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.18 cancel request preview JSON input.
    #[arg(long)]
    pub cancel_request_preview: PathBuf,
    /// Expected owner-selected symbol for the single cancel candidate scope.
    #[arg(long)]
    pub expected_symbol: String,
    /// Expected owner-selected account label for the single cancel candidate scope.
    #[arg(long)]
    pub expected_account_label: String,
    /// v0.18 cancel risk gate JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for cancel risk gate evaluation.
    #[arg(long)]
    pub allow_production_mutation_cancel_risk_gate: bool,
    /// Confirms the gate is limited to one v0.16 mutation candidate lineage.
    #[arg(long)]
    pub confirm_single_v16_mutation_candidate_lineage: bool,
    /// Confirms the cancel request preview must already be ready.
    #[arg(long)]
    pub confirm_cancel_request_preview_ready: bool,
    /// Confirms the source orphan risk is halted and new orders are blocked.
    #[arg(long)]
    pub confirm_orphan_risk_halted: bool,
    /// Confirms known order identifiers are required for the scoped candidate.
    #[arg(long)]
    pub confirm_known_order_identifier_only: bool,
    /// Confirms symbol and account label must match the selected lineage scope.
    #[arg(long)]
    pub confirm_symbol_account_scope: bool,
    /// Confirms owner approval remains required before any future send.
    #[arg(long)]
    pub confirm_owner_approval_required: bool,
    /// Confirms cancel-all, bulk, and multi-order cancel requests are forbidden.
    #[arg(long)]
    pub confirm_no_cancel_all_or_bulk: bool,
    /// Confirms no retry is requested, attempted, or scheduled.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms no cancel is attempted or scheduled.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no network endpoint is attempted.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms no replace, amend, correction, flatten, or remediation is attempted.
    #[arg(long)]
    pub confirm_no_remediation: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// v0.18 manual owner approval lifecycle options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationManualOwnerApprovalLifecycleOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.18 cancel risk gate JSON input.
    #[arg(long)]
    pub cancel_risk_gate: PathBuf,
    /// Approval state: pending, approved, expired, revoked, or used.
    #[arg(long, default_value = "pending")]
    pub approval_state: String,
    /// Optional owner approval identifier for non-pending states.
    #[arg(long)]
    pub manual_approval_id: Option<String>,
    /// Optional owner/operator name for non-pending states.
    #[arg(long)]
    pub approved_by: Option<String>,
    /// Deterministic current time in milliseconds for lifecycle evaluation.
    #[arg(long)]
    pub now_unix_ms: u64,
    /// Approval expiry in milliseconds.
    #[arg(long)]
    pub expires_at_unix_ms: u64,
    /// v0.18 manual owner approval lifecycle JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for manual owner approval lifecycle evidence.
    #[arg(long)]
    pub allow_production_mutation_manual_owner_approval_lifecycle: bool,
    /// Confirms approval is scoped to exactly one cancel candidate.
    #[arg(long)]
    pub confirm_one_order_cancel_candidate: bool,
    /// Confirms approval is one-time only.
    #[arg(long)]
    pub confirm_one_time_approval: bool,
    /// Confirms approval is non-reusable after this candidate scope.
    #[arg(long)]
    pub confirm_non_reusable_approval: bool,
    /// Confirms approval expiry is required and enforced.
    #[arg(long)]
    pub confirm_approval_expiry: bool,
    /// Confirms strategy code cannot auto-approve.
    #[arg(long)]
    pub confirm_no_strategy_auto_approval: bool,
    /// Confirms background processes cannot auto-approve.
    #[arg(long)]
    pub confirm_no_background_auto_approval: bool,
    /// Confirms Dashboard buttons cannot auto-approve or cancel.
    #[arg(long)]
    pub confirm_no_dashboard_cancel_approval: bool,
    /// Confirms incident handlers cannot auto-approve.
    #[arg(long)]
    pub confirm_no_incident_handler_auto_approval: bool,
    /// Confirms no cancel send is attempted or allowed.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no network endpoint is attempted.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// v0.19 actual cancel owner approval lifecycle options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// V190-002 actual cancel safety contract Markdown input.
    #[arg(long)]
    pub actual_cancel_safety_contract: PathBuf,
    /// v0.18/v0.19 release manifest JSON input that anchors approval provenance.
    #[arg(long)]
    pub release_manifest: PathBuf,
    /// v0.18 cancel risk gate JSON input for the same order lineage.
    #[arg(long)]
    pub cancel_risk_gate: PathBuf,
    /// Expected order lineage identifier to bind this approval.
    #[arg(long)]
    pub expected_order_lineage_id: String,
    /// Expected symbol to bind this approval.
    #[arg(long)]
    pub expected_symbol: String,
    /// Expected account label to bind this approval.
    #[arg(long)]
    pub expected_account_label: String,
    /// Expected venue to bind this approval.
    #[arg(long)]
    pub venue: String,
    /// Expected release tag/provenance anchor.
    #[arg(long)]
    pub expected_release_tag: String,
    /// Approval state: created, approved, expired, used, rejected, or audited.
    #[arg(long, default_value = "created")]
    pub approval_state: String,
    /// Optional owner approval identifier for approved/expired/used/rejected/audited states.
    #[arg(long)]
    pub manual_approval_id: Option<String>,
    /// Optional owner/operator identity.
    #[arg(long)]
    pub approved_by: Option<String>,
    /// Optional owner-visible reason for this actual cancel approval decision.
    #[arg(long)]
    pub approval_reason: Option<String>,
    /// Deterministic current time in milliseconds for lifecycle evaluation.
    #[arg(long)]
    pub now_unix_ms: u64,
    /// Approval expiry in milliseconds.
    #[arg(long)]
    pub expires_at_unix_ms: u64,
    /// v0.19 actual cancel owner approval lifecycle JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for actual cancel owner approval lifecycle evidence.
    #[arg(long)]
    pub allow_production_mutation_actual_cancel_owner_approval_lifecycle: bool,
    /// Confirms the V190-002 actual cancel safety contract must be present.
    #[arg(long)]
    pub confirm_actual_cancel_safety_contract: bool,
    /// Confirms approval is scoped to one order, one venue, and one attempt.
    #[arg(long)]
    pub confirm_one_order_one_venue_one_attempt: bool,
    /// Confirms approval is single-use and cannot be reused.
    #[arg(long)]
    pub confirm_single_use_approval: bool,
    /// Confirms approval expiry is required and enforced.
    #[arg(long)]
    pub confirm_approval_expiry: bool,
    /// Confirms order, risk gate, and release provenance are bound together.
    #[arg(long)]
    pub confirm_bind_order_risk_gate_release_provenance: bool,
    /// Confirms used/rejected/audited states leave audit evidence.
    #[arg(long)]
    pub confirm_audit_evidence: bool,
    /// Confirms Dashboard cannot approve or trigger the actual cancel.
    #[arg(long)]
    pub confirm_no_dashboard_approval: bool,
    /// Confirms automatic cancel remains forbidden.
    #[arg(long)]
    pub confirm_no_automatic_cancel: bool,
    /// Confirms bulk and cancel-all paths remain forbidden.
    #[arg(long)]
    pub confirm_no_bulk_cancel: bool,
    /// Confirms retry, replace, amend, flatten, and remediation remain forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms this lifecycle does not introduce production order submission.
    #[arg(long)]
    pub confirm_no_submit_lifecycle: bool,
    /// Confirms this lifecycle does not open network access.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// v0.19 actual cancel executor adapter boundary options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// V190-003 owner approval lifecycle JSON input.
    #[arg(long)]
    pub owner_approval_lifecycle: PathBuf,
    /// Adapter capability declaration JSON input.
    #[arg(long)]
    pub adapter_capability: PathBuf,
    /// Expected adapter identifier.
    #[arg(long)]
    pub adapter_id: String,
    /// Expected venue for the single cancel attempt.
    #[arg(long)]
    pub venue: String,
    /// Expected order id type used by the adapter request.
    #[arg(long)]
    pub order_id_type: String,
    /// Expected order lineage identifier.
    #[arg(long)]
    pub expected_order_lineage_id: String,
    /// Expected symbol.
    #[arg(long)]
    pub expected_symbol: String,
    /// Expected account label.
    #[arg(long)]
    pub expected_account_label: String,
    /// v0.19 actual cancel executor adapter boundary JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for actual cancel adapter boundary evidence.
    #[arg(long)]
    pub allow_production_mutation_actual_cancel_executor_adapter_boundary: bool,
    /// Confirms adapter capability declaration must be present and matched.
    #[arg(long)]
    pub confirm_adapter_capability: bool,
    /// Confirms cancel request/response/readback/audit contracts are recorded.
    #[arg(long)]
    pub confirm_request_response_readback_audit_contract: bool,
    /// Confirms the boundary is scoped to one order, one venue, and one attempt.
    #[arg(long)]
    pub confirm_one_order_one_venue_one_attempt: bool,
    /// Confirms unsupported venue or adapter capability fails closed.
    #[arg(long)]
    pub confirm_fail_closed_unsupported_capability: bool,
    /// Confirms bulk and cancel-all paths remain forbidden.
    #[arg(long)]
    pub confirm_no_bulk_cancel: bool,
    /// Confirms retry, replace, amend, flatten, and remediation remain forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms automatic cancel remains forbidden.
    #[arg(long)]
    pub confirm_no_automatic_cancel: bool,
    /// Confirms Dashboard cannot trigger the adapter boundary.
    #[arg(long)]
    pub confirm_no_dashboard_execution: bool,
    /// Confirms this boundary command does not open network access.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// v0.18 cancel response redaction options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationCancelResponseRedactionOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.18 manual owner approval lifecycle JSON input.
    #[arg(long)]
    pub manual_owner_approval_lifecycle: PathBuf,
    /// Synthetic or manually supplied future cancel response JSON input.
    #[arg(long)]
    pub response: PathBuf,
    /// v0.18 cancel response redaction JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for cancel response redaction contract evaluation.
    #[arg(long)]
    pub allow_production_mutation_cancel_response_redaction: bool,
    /// Confirms the manual owner approval lifecycle must already be valid.
    #[arg(long)]
    pub confirm_manual_owner_approval_lifecycle_ready: bool,
    /// Confirms raw response bodies must not be persisted.
    #[arg(long)]
    pub confirm_no_raw_response_persistence: bool,
    /// Confirms HTTP headers must not be persisted.
    #[arg(long)]
    pub confirm_no_headers_persistence: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms only cancel response metadata fields are allowed.
    #[arg(long)]
    pub confirm_cancel_metadata_only: bool,
    /// Confirms account balances are not persisted in the response artifact.
    #[arg(long)]
    pub confirm_no_account_balances: bool,
    /// Confirms unrestricted payload capture is forbidden.
    #[arg(long)]
    pub confirm_no_unrestricted_payload: bool,
    /// Confirms retry, correction, replace, amend, flatten, and remediation remain forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms no cancel send is attempted or allowed.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no network endpoint is attempted.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
}

/// v0.18 post-cancel readback contract options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationPostCancelReadbackOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.18 cancel response redaction JSON input.
    #[arg(long)]
    pub cancel_response_redaction: PathBuf,
    /// Synthetic or manually supplied read-only post-cancel readback JSON input.
    #[arg(long)]
    pub readback: PathBuf,
    /// v0.18 post-cancel readback JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for post-cancel readback contract evaluation.
    #[arg(long)]
    pub allow_production_mutation_post_cancel_readback: bool,
    /// Confirms the cancel response redaction artifact must already be valid.
    #[arg(long)]
    pub confirm_cancel_response_redaction_ready: bool,
    /// Confirms only redacted readback metadata fields are allowed.
    #[arg(long)]
    pub confirm_readback_metadata_only: bool,
    /// Confirms canceled, filled, rejected, expired, missing, and unknown states are classified.
    #[arg(long)]
    pub confirm_terminal_and_ambiguous_classification: bool,
    /// Confirms raw readback bodies must not be persisted.
    #[arg(long)]
    pub confirm_no_raw_readback_persistence: bool,
    /// Confirms HTTP headers must not be persisted.
    #[arg(long)]
    pub confirm_no_headers_persistence: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
    /// Confirms no production mutation is attempted or allowed.
    #[arg(long)]
    pub confirm_no_mutation: bool,
    /// Confirms retry remains forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms remediation remains forbidden.
    #[arg(long)]
    pub confirm_no_remediation: bool,
    /// Confirms no cancel send is attempted or allowed.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no network endpoint is attempted.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
}

/// v0.18 cancel recovery incident/audit closeout contract options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionMutationCancelRecoveryIncidentAuditCloseoutOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.18 cancel risk gate JSON input.
    #[arg(long)]
    pub cancel_risk_gate: PathBuf,
    /// v0.18 manual owner approval lifecycle JSON input.
    #[arg(long)]
    pub manual_owner_approval_lifecycle: PathBuf,
    /// v0.18 cancel response redaction JSON input.
    #[arg(long)]
    pub cancel_response_redaction: PathBuf,
    /// v0.18 post-cancel readback JSON input.
    #[arg(long)]
    pub post_cancel_readback: PathBuf,
    /// v0.18 incident/audit closeout JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Manual CLI gate for incident/audit closeout contract evaluation.
    #[arg(long)]
    pub allow_production_mutation_cancel_recovery_incident_audit_closeout: bool,
    /// Confirms all source artifacts must share one cancel recovery lineage.
    #[arg(long)]
    pub confirm_cancel_recovery_lineage: bool,
    /// Confirms the recovery-needed reason is recorded.
    #[arg(long)]
    pub confirm_risk_reason_recorded: bool,
    /// Confirms the cancel risk gate result is recorded.
    #[arg(long)]
    pub confirm_risk_gate_result_recorded: bool,
    /// Confirms the owner approval state is recorded.
    #[arg(long)]
    pub confirm_owner_approval_state_recorded: bool,
    /// Confirms the cancel response redaction contract state is recorded.
    #[arg(long)]
    pub confirm_redaction_contract_state_recorded: bool,
    /// Confirms the post-cancel readback state is recorded.
    #[arg(long)]
    pub confirm_readback_state_recorded: bool,
    /// Confirms a terminal action recommendation is recorded.
    #[arg(long)]
    pub confirm_terminal_action_recommendation: bool,
    /// Confirms remaining risk is recorded.
    #[arg(long)]
    pub confirm_remaining_risk_recorded: bool,
    /// Confirms no production mutation is attempted or allowed.
    #[arg(long)]
    pub confirm_no_mutation: bool,
    /// Confirms no cancel send is attempted or allowed.
    #[arg(long)]
    pub confirm_no_cancel: bool,
    /// Confirms no network endpoint is attempted.
    #[arg(long)]
    pub confirm_no_network: bool,
    /// Confirms retry remains forbidden.
    #[arg(long)]
    pub confirm_no_retry: bool,
    /// Confirms manual remediation is not executed by this artifact.
    #[arg(long)]
    pub confirm_no_remediation: bool,
    /// Confirms automatic remediation remains forbidden.
    #[arg(long)]
    pub confirm_no_automatic_remediation: bool,
    /// Confirms Dashboard order and cancel controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
    /// Confirms API key, secret, signature, signed query, and signed URL are not persisted.
    #[arg(long)]
    pub confirm_no_secret_persistence: bool,
}

/// Hypothetical live-alpha risk preflight options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionLiveAlphaRiskPreflightOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// v0.14 dry-run order gate JSON input.
    #[arg(long)]
    pub order_gate: PathBuf,
    /// v0.14 hypothetical risk preflight input JSON.
    #[arg(long)]
    pub input: PathBuf,
    /// v0.14 risk preflight JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Confirms this command only evaluates hypothetical dry-run risk.
    #[arg(long)]
    pub confirm_hypothetical_dry_run_only: bool,
    /// Confirms no execution adapter call is allowed.
    #[arg(long)]
    pub confirm_no_execution_adapter_call: bool,
    /// Confirms no production order submission is allowed.
    #[arg(long)]
    pub confirm_no_production_order_submission: bool,
    /// Confirms no production order mutation is allowed.
    #[arg(long)]
    pub confirm_no_production_order_mutation: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
}

/// Local shadow portfolio runtime artifact options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionShadowPortfolioRuntimeOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// Optional deterministic snapshot identifier.
    #[arg(long)]
    pub snapshot_id: Option<String>,
    /// Redacted production account snapshot contract/report JSON.
    #[arg(long)]
    pub account_snapshot: PathBuf,
    /// Local shadow execution intent JSONL.
    #[arg(long)]
    pub shadow_intent: PathBuf,
    /// v0.12 shadow portfolio runtime JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Optional v0.11-compatible shadow_portfolio_snapshot.json output path for existing Dashboard readers.
    #[arg(long)]
    pub compat_snapshot_output: Option<PathBuf>,
}

/// Local persistent shadow strategy session artifact options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionShadowStrategySessionOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// Optional owner-visible session identifier; defaults to run_id.
    #[arg(long)]
    pub session_id: Option<String>,
    /// Owner-visible strategy identifier.
    #[arg(long, default_value = "ema_cross_btcusdt_v1")]
    pub strategy_id: String,
    /// v0.12 shadow portfolio runtime JSON input.
    #[arg(long)]
    pub shadow_portfolio_runtime: PathBuf,
    /// Optional existing strategy/session status JSON input.
    #[arg(long)]
    pub strategy_session_status: Option<PathBuf>,
    /// v0.12 shadow strategy session JSONL output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Number of local heartbeat events to write.
    #[arg(long, default_value_t = 2)]
    pub heartbeat_count: u64,
    /// Records a local owner stop marker after the configured heartbeat count.
    #[arg(long)]
    pub stop_after_heartbeats: bool,
    /// Optional local owner stop-file. If present, the command records a stop marker.
    #[arg(long)]
    pub stop_file: Option<PathBuf>,
}

/// Local guarded-live-alpha shadow preflight loop options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionShadowPreflightSessionOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// Optional owner-visible session identifier; defaults to run_id.
    #[arg(long)]
    pub session_id: Option<String>,
    /// Owner-visible strategy identifier.
    #[arg(long, default_value = "ema_cross_btcusdt_v1")]
    pub strategy_id: String,
    /// v0.12 shadow portfolio runtime JSON input.
    #[arg(long)]
    pub shadow_portfolio_runtime: PathBuf,
    /// Optional existing strategy/session status JSON input.
    #[arg(long)]
    pub strategy_session_status: Option<PathBuf>,
    /// v0.13 shadow preflight session JSONL output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Maximum local heartbeat events before the preflight loop stops.
    #[arg(long, default_value_t = 3)]
    pub max_heartbeats: u64,
    /// Delay between heartbeat checks.
    #[arg(long, default_value_t = 1_000)]
    pub heartbeat_interval_ms: u64,
    /// Marks source data stale when the shadow portfolio artifact is older than this threshold.
    #[arg(long, default_value_t = 30_000)]
    pub stale_after_ms: u64,
    /// Optional local owner stop-file. If present, the loop stops without production mutation.
    #[arg(long)]
    pub stop_file: Option<PathBuf>,
}

/// Local guarded-live-alpha kill-switch dry-run/manual-approval artifact options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionKillSwitchApprovalArtifactOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// Optional owner-visible session identifier; defaults to run_id.
    #[arg(long)]
    pub session_id: Option<String>,
    /// Owner-visible strategy identifier.
    #[arg(long, default_value = "ema_cross_btcusdt_v1")]
    pub strategy_id: String,
    /// v0.13 kill-switch/manual-approval JSON output path.
    #[arg(long)]
    pub output: PathBuf,
    /// Dry-run kill-switch active state to record in the artifact.
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub kill_switch_active: bool,
    /// Manual approval state to record: pending, approved, or rejected.
    #[arg(long, default_value = "pending")]
    pub approval_state: String,
    /// Optional owner approval identifier when approval_state=approved.
    #[arg(long)]
    pub manual_approval_id: Option<String>,
    /// Optional owner/operator name when approval_state=approved.
    #[arg(long)]
    pub approved_by: Option<String>,
    /// Confirms this command is a local dry-run artifact only.
    #[arg(long)]
    pub confirm_dry_run_only: bool,
    /// Confirms no production order submission or mutation is allowed.
    #[arg(long)]
    pub confirm_no_production_mutation: bool,
    /// Confirms Dashboard order controls remain disabled.
    #[arg(long)]
    pub confirm_dashboard_order_controls_disabled: bool,
}

/// Local production read-only reconciliation artifact options.
#[derive(Parser, Debug, Clone)]
pub struct LiveProductionReadonlyReconciliationOpt {
    /// Owner-visible run identifier.
    #[arg(long)]
    pub run_id: String,
    /// Optional redacted production account snapshot JSON input.
    #[arg(long)]
    pub account_snapshot: Option<PathBuf>,
    /// Optional v0.12 shadow portfolio runtime JSON input.
    #[arg(long)]
    pub shadow_portfolio_runtime: Option<PathBuf>,
    /// Optional v0.12 shadow strategy session JSONL input.
    #[arg(long)]
    pub shadow_strategy_session: Option<PathBuf>,
    /// Optional local shadow execution intent JSONL input.
    #[arg(long)]
    pub shadow_intent: Option<PathBuf>,
    /// v0.12 production read-only reconciliation JSONL output path.
    #[arg(long)]
    pub output: PathBuf,
}

/// Local supervisor controls for sandbox-only node artifacts and processes.
#[derive(Parser, Debug)]
#[command(
    about = "Local supervisor controls for sandbox-only ntpro-node processes",
    long_about = None
)]
pub struct SupervisorOpt {
    #[clap(subcommand)]
    pub command: SupervisorCommand,
}

/// Available local supervisor commands.
#[derive(Parser, Debug, Clone)]
#[command(
    about = "Local supervisor controls for sandbox-only ntpro-node processes",
    long_about = None
)]
pub enum SupervisorCommand {
    /// Registers or replaces a local sandbox node record.
    Register(SupervisorRegisterOpt),
    /// Lists registered local nodes.
    List(SupervisorListOpt),
    /// Starts a registered local sandbox-only ntpro-node process.
    Start(SupervisorStartOpt),
    /// Stops a registered local ntpro-node process.
    Stop(SupervisorStopOpt),
    /// Pauses a running local sandbox node at the supervisor artifact layer.
    Pause(SupervisorNodeOpt),
    /// Resumes a paused local sandbox node at the supervisor artifact layer.
    Resume(SupervisorNodeOpt),
    /// Records a local sandbox data-source reconnect result.
    ReconnectData(SupervisorNodeOpt),
    /// Records a local sandbox execution-gateway reconnect result.
    ReconnectExecution(SupervisorNodeOpt),
    /// Reads the latest node status artifact.
    Status(SupervisorNodeOpt),
    /// Reads data/execution connection status from node artifacts.
    Connections(SupervisorNodeOpt),
    /// Reads execution summary from node artifacts.
    Execution(SupervisorNodeOpt),
    /// Reads risk summary from node artifacts.
    Risk(SupervisorNodeOpt),
    /// Prints per-node log artifact paths.
    Logs(SupervisorNodeOpt),
    /// Reads minimal per-node metrics JSON.
    Metrics(SupervisorNodeOpt),
    /// Reads supervisor-managed shadow runtime status without enabling order controls.
    ShadowRuntime(SupervisorNodeOpt),
}

/// Common supervisor registry option.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorRegistryOpt {
    /// Path to the local supervisor registry JSON file.
    #[arg(long)]
    pub registry: PathBuf,
}

/// Supervisor node registration options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorRegisterOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
    /// Stable local node identifier.
    #[arg(long)]
    pub node_id: String,
    /// Path to the Rust live-init smoke config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional directory for node artifacts.
    #[arg(long)]
    pub artifact_root: Option<PathBuf>,
}

/// Supervisor node listing options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorListOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
}

/// Supervisor node start options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorStartOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
    /// Registered local node identifier.
    #[arg(long)]
    pub node_id: String,
    /// Path to the ntpro-node binary.
    #[arg(long)]
    pub ntpro_node_bin: PathBuf,
    /// Startup wait timeout in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub startup_timeout_ms: u64,
    /// Maximum runtime passed to the spawned ntpro-node process.
    #[arg(long, default_value_t = 3_600_000)]
    pub node_max_runtime_ms: u64,
    /// Heartbeat interval passed to the spawned ntpro-node process.
    #[arg(long, default_value_t = 1_000)]
    pub node_heartbeat_interval_ms: u64,
    /// Optional parent process PID passed to the spawned ntpro-node process.
    #[arg(long)]
    pub node_parent_pid: Option<u32>,
    /// Shutdown timeout passed to the spawned ntpro-node process.
    #[arg(long, default_value_t = 5_000)]
    pub node_shutdown_timeout_ms: u64,
}

/// Supervisor node stop options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorStopOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
    /// Registered local node identifier.
    #[arg(long)]
    pub node_id: String,
    /// Stop wait timeout in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub stop_timeout_ms: u64,
}

/// Supervisor single-node query options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorNodeOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
    /// Registered local node identifier.
    #[arg(long)]
    pub node_id: String,
}

/// Local dashboard HTTP server commands.
#[derive(Parser, Debug)]
#[command(about = "Local dashboard HTTP server", long_about = None)]
pub struct DashboardOpt {
    #[clap(subcommand)]
    pub command: DashboardCommand,
}

/// Available local dashboard commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Local dashboard HTTP server", long_about = None)]
pub enum DashboardCommand {
    /// Serves the static dashboard shell and local JSON API.
    Serve(DashboardServeOpt),
}

/// Local dashboard server options.
#[derive(Parser, Debug, Clone)]
pub struct DashboardServeOpt {
    /// Path to the local supervisor registry JSON file.
    #[arg(long)]
    pub registry: PathBuf,
    /// Optional local workflow artifact root to scan independently of supervisor registry.
    #[arg(long)]
    pub workflow_root: Option<PathBuf>,
    /// Local loopback address for the dashboard HTTP server.
    #[arg(long, default_value = "127.0.0.1:5173")]
    pub bind: SocketAddr,
    /// Optional path to the local ntpro-node binary used by start controls.
    #[arg(long)]
    pub ntpro_node_bin: Option<PathBuf>,
}

/// Local workflow artifact commands for sandbox/testnet product smokes.
#[derive(Parser, Debug)]
#[command(
    about = "Local sandbox/testnet workflow artifact commands",
    long_about = None
)]
pub struct WorkflowOpt {
    #[clap(subcommand)]
    pub command: WorkflowCommand,
}

/// Available local workflow artifact commands.
#[derive(Parser, Debug, Clone)]
#[command(
    about = "Local sandbox/testnet workflow artifact commands",
    long_about = None
)]
pub enum WorkflowCommand {
    /// Runs a local Binance sandbox/testnet workflow and writes dashboard-readable artifacts.
    Run(WorkflowRunOpt),
}

/// Supported local workflow kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WorkflowKind {
    /// Local Binance sandbox workflow using checked-in fixtures and mock execution only.
    BinanceSandbox,
    /// Local Binance testnet workflow contract with fail-closed v0.7 network gating.
    BinanceTestnet,
}

/// Supported local workflow run modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WorkflowRunMode {
    /// Validate config and write local dry-run artifacts only.
    DryRun,
    /// Record connectivity-probe intent behind the fail-closed testnet network gate.
    ConnectivityProbe,
}

/// Local workflow run options.
#[derive(Parser, Debug, Clone)]
pub struct WorkflowRunOpt {
    /// Workflow kind to run.
    #[arg(long, value_enum, default_value_t = WorkflowKind::BinanceSandbox)]
    pub workflow: WorkflowKind,
    /// Workflow run mode. Default dry-run opens no sockets.
    #[arg(long, value_enum, default_value_t = WorkflowRunMode::DryRun)]
    pub mode: WorkflowRunMode,
    /// Optional workflow config. Required for the Binance testnet workflow.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Fail-closed opt-in for Binance testnet read-only networking; also requires NTPRO_ALLOW_TESTNET_NETWORK=1 and read-only config.
    #[arg(long)]
    pub allow_testnet_network: bool,
    /// Owner-visible run identifier used in the artifact directory.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional output directory for workflow artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Data catalog inspection, validation, and loading commands.
#[derive(Parser, Debug)]
#[command(about = "Data catalog operations", long_about = None)]
pub struct DataOpt {
    #[clap(subcommand)]
    pub command: DataCommand,
}

/// Available data catalog commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Data catalog operations", long_about = None)]
pub enum DataCommand {
    /// Inspects local data/catalog metadata without running a workflow.
    Inspect(DataInspectOpt),
    /// Validates local data/catalog readability and query shape.
    Validate(DataValidateOpt),
    /// Loads a local QuoteTick fixture into a Rust catalog directory.
    Load(DataLoadOpt),
}

/// Data catalog inspection options.
#[derive(Parser, Debug, Clone)]
pub struct DataInspectOpt {
    /// Path to the Rust data/catalog config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional directory for inspection artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Data catalog validation options.
#[derive(Parser, Debug, Clone)]
pub struct DataValidateOpt {
    /// Path to the Rust data/catalog config file.
    #[arg(long)]
    pub config: PathBuf,
}

/// Data catalog load options.
#[derive(Parser, Debug, Clone)]
pub struct DataLoadOpt {
    /// Path to the Rust data/catalog config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional directory for load artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Shared Rust config validation commands.
#[derive(Parser, Debug)]
#[command(about = "Rust config validation", long_about = None)]
pub struct ConfigOpt {
    #[clap(subcommand)]
    pub command: ConfigCommand,
}

/// Available Rust config validation commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Rust config validation", long_about = None)]
pub enum ConfigCommand {
    /// Validates a Rust workflow config without running the workflow.
    Validate(ConfigValidateOpt),
}

/// Supported workflow config kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ConfigKind {
    Backtest,
    Sandbox,
    Live,
    Data,
    StrategySession,
}

/// Shared Rust config validation options.
#[derive(Parser, Debug, Clone)]
pub struct ConfigValidateOpt {
    /// Workflow config kind.
    #[arg(long, value_enum)]
    pub kind: ConfigKind,
    /// Path to the Rust config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional directory for validation artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Database management options and subcommands.
#[derive(Parser, Debug)]
#[command(about = "Postgres database operations", long_about = None)]
pub struct DatabaseOpt {
    #[clap(subcommand)]
    pub command: DatabaseCommand,
}

/// Configuration parameters for database connection and operations.
#[derive(Parser, Debug, Clone)]
pub struct DatabaseConfig {
    /// Hostname or IP address of the database server.
    #[arg(long)]
    pub host: Option<String>,
    /// Port number of the database server.
    #[arg(long)]
    pub port: Option<u16>,
    /// Username for connecting to the database.
    #[arg(long)]
    pub username: Option<String>,
    /// Name of the database.
    #[arg(long)]
    pub database: Option<String>,
    /// Password for connecting to the database.
    #[arg(long)]
    pub password: Option<String>,
    /// Directory path to the schema files.
    #[arg(long)]
    pub schema: Option<String>,
}

/// Available database management commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Postgres database operations", long_about = None)]
pub enum DatabaseCommand {
    /// Initializes a new Postgres database with the latest schema.
    Init(DatabaseConfig),
    /// Drops roles, privileges and deletes all data from the database.
    Drop(DatabaseConfig),
}

#[cfg(feature = "defi")]
/// Blockchain management options and subcommands.
#[derive(Parser, Debug)]
#[command(about = "Blockchain operations", long_about = None)]
pub struct BlockchainOpt {
    #[clap(subcommand)]
    pub command: BlockchainCommand,
}

#[cfg(feature = "defi")]
/// Available blockchain management commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Blockchain operations", long_about = None)]
pub enum BlockchainCommand {
    /// Syncs blockchain blocks.
    SyncBlocks {
        /// The blockchain chain name (case-insensitive). Examples: ethereum, arbitrum, base, polygon, bsc
        #[arg(long)]
        chain: String,
        /// Starting block number to sync from (optional)
        #[arg(long)]
        from_block: Option<u64>,
        /// Ending block number to sync to (optional, defaults to current chain head)
        #[arg(long)]
        to_block: Option<u64>,
        /// Database configuration options
        #[clap(flatten)]
        database: DatabaseConfig,
    },
    /// Sync DEX pools.
    SyncDex {
        /// The blockchain chain name (case-insensitive). Examples: ethereum, arbitrum, base, polygon, bsc
        #[arg(long)]
        chain: String,
        /// The DEX name (case-insensitive). Examples: `UniswapV3`, uniswapv3, `SushiSwapV2`, `PancakeSwapV3`
        #[arg(long)]
        dex: String,
        /// RPC HTTP URL for blockchain calls (optional, falls back to `RPC_HTTP_URL` env var)
        #[arg(long)]
        rpc_url: Option<String>,
        /// Reset sync progress and start from the beginning, ignoring last synced block
        #[arg(long)]
        reset: bool,
        /// Maximum number of Multicall calls per RPC request (optional, defaults to 100)
        #[arg(long)]
        multicall_calls_per_rpc_request: Option<u32>,
        /// Database configuration options
        #[clap(flatten)]
        database: DatabaseConfig,
    },
    /// Analyze a specific DEX pool.
    AnalyzePool {
        /// The blockchain chain name (case-insensitive). Examples: ethereum, arbitrum, base, polygon, bsc
        #[arg(long)]
        chain: String,
        /// The DEX name (case-insensitive). Examples: UniswapV3, uniswapv3, SushiSwapV2, PancakeSwapV3
        #[arg(long)]
        dex: String,
        /// The pool contract address
        #[arg(long)]
        address: String,
        /// Starting block number to sync from (optional)
        #[arg(long)]
        from_block: Option<u64>,
        /// Ending block number to sync to (optional, defaults to current chain head)
        #[arg(long)]
        to_block: Option<u64>,
        /// RPC HTTP URL for blockchain calls (optional, falls back to RPC_HTTP_URL env var)
        #[arg(long)]
        rpc_url: Option<String>,
        /// Reset sync progress and start from the beginning, ignoring last synced block
        #[arg(long)]
        reset: bool,
        /// Maximum number of Multicall calls per RPC request (optional, defaults to 100)
        #[arg(long)]
        multicall_calls_per_rpc_request: Option<u32>,
        /// Database configuration options
        #[clap(flatten)]
        database: DatabaseConfig,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    fn render_subcommand_help(path: &[&str]) -> String {
        let mut command = NautilusCli::command();
        let mut current = &mut command;
        for name in path {
            current = current
                .find_subcommand_mut(name)
                .unwrap_or_else(|| panic!("{name} command should exist"));
        }
        current.render_help().to_string()
    }

    #[test]
    fn top_level_help_lists_backtest() {
        let help = NautilusCli::command().render_help().to_string();

        assert!(help.contains("backtest"));
        assert!(help.contains("sandbox"));
        assert!(help.contains("live"));
        assert!(help.contains("data"));
        assert!(help.contains("config"));
        assert!(help.contains("supervisor"));
        assert!(help.contains("dashboard"));
        assert!(help.contains("workflow"));
        assert!(help.contains("database"));
    }

    #[test]
    fn backtest_help_lists_validate_and_run() {
        let mut command = NautilusCli::command();
        let backtest = command
            .find_subcommand_mut("backtest")
            .expect("backtest command should exist");
        let help = backtest.render_help().to_string();

        assert!(help.contains("validate"));
        assert!(help.contains("run"));
    }

    #[test]
    fn parses_backtest_validate_config_path() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "backtest",
            "validate",
            "--config",
            "config/backtest.toml",
        ])
        .expect("backtest validate should parse");

        let Commands::Backtest(backtest) = parsed.command else {
            panic!("expected backtest command");
        };
        let BacktestCommand::Validate(validate) = backtest.command else {
            panic!("expected validate command");
        };

        assert_eq!(validate.config, PathBuf::from("config/backtest.toml"));
    }

    #[test]
    fn parses_backtest_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "backtest",
            "run",
            "--config",
            "config/backtest.toml",
            "--run-id",
            "ema-cross",
            "--output",
            "runs/ema-cross",
            "--dry-run",
        ])
        .expect("backtest run should parse");

        let Commands::Backtest(backtest) = parsed.command else {
            panic!("expected backtest command");
        };
        let BacktestCommand::Run(run) = backtest.command else {
            panic!("expected run command");
        };

        assert_eq!(run.config, PathBuf::from("config/backtest.toml"));
        assert_eq!(run.run_id.as_deref(), Some("ema-cross"));
        assert_eq!(run.output, Some(PathBuf::from("runs/ema-cross")));
        assert!(run.dry_run);
    }

    #[test]
    fn backtest_run_help_describes_dry_run_boundary() {
        let help = render_subcommand_help(&["backtest", "run"]);

        assert!(help.contains("Rust backtest smoke path"));
        assert!(help.contains("metadata-only dry-run"));
        assert!(help.contains("--dry-run"));
    }

    #[test]
    fn sandbox_help_lists_validate_and_run() {
        let mut command = NautilusCli::command();
        let sandbox = command
            .find_subcommand_mut("sandbox")
            .expect("sandbox command should exist");
        let help = sandbox.render_help().to_string();

        assert!(help.contains("validate"));
        assert!(help.contains("run"));
    }

    #[test]
    fn parses_sandbox_validate_config_path() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "sandbox",
            "validate",
            "--config",
            "config/sandbox.toml",
        ])
        .expect("sandbox validate should parse");

        let Commands::Sandbox(sandbox) = parsed.command else {
            panic!("expected sandbox command");
        };
        let SandboxCommand::Validate(validate) = sandbox.command else {
            panic!("expected validate command");
        };

        assert_eq!(validate.config, PathBuf::from("config/sandbox.toml"));
    }

    #[test]
    fn parses_sandbox_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "sandbox",
            "run",
            "--config",
            "config/sandbox.toml",
            "--run-id",
            "sandbox-smoke",
            "--output",
            "runs/sandbox-smoke",
        ])
        .expect("sandbox run should parse");

        let Commands::Sandbox(sandbox) = parsed.command else {
            panic!("expected sandbox command");
        };
        let SandboxCommand::Run(run) = sandbox.command else {
            panic!("expected run command");
        };

        assert_eq!(run.config, PathBuf::from("config/sandbox.toml"));
        assert_eq!(run.run_id.as_deref(), Some("sandbox-smoke"));
        assert_eq!(run.output, Some(PathBuf::from("runs/sandbox-smoke")));
    }

    #[test]
    fn sandbox_run_help_describes_simulated_boundary() {
        let help = render_subcommand_help(&["sandbox", "run"]);

        assert!(help.contains("simulation-only sandbox demo artifacts"));
        assert!(help.contains("use live run for LiveNode smoke"));
    }

    #[test]
    fn live_help_lists_validate_and_run() {
        let mut command = NautilusCli::command();
        let live = command
            .find_subcommand_mut("live")
            .expect("live command should exist");
        let help = live.render_help().to_string();

        assert!(help.contains("validate"));
        assert!(help.contains("run"));
    }

    #[test]
    fn live_help_describes_owner_gated_mutation_candidate_boundary() {
        let help = render_subcommand_help(&["live"]);

        assert!(help.contains("owner-gated mutation-candidate commands"));
        assert!(!help.contains("(no production order mutation)"));
    }

    #[test]
    fn parses_live_validate_config_path() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "validate",
            "--config",
            "config/live.toml",
        ])
        .expect("live validate should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::Validate(validate) = live.command else {
            panic!("expected validate command");
        };

        assert_eq!(validate.config, PathBuf::from("config/live.toml"));
    }

    #[test]
    fn parses_live_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "run",
            "--config",
            "config/live.toml",
            "--run-id",
            "live-dry-run",
            "--output",
            "runs/live-dry-run",
        ])
        .expect("live run should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::Run(run) = live.command else {
            panic!("expected run command");
        };

        assert_eq!(run.config, PathBuf::from("config/live.toml"));
        assert_eq!(run.run_id.as_deref(), Some("live-dry-run"));
        assert_eq!(run.output, Some(PathBuf::from("runs/live-dry-run")));
    }

    #[test]
    fn parses_live_production_live_alpha_dry_run_order_gate_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-live-alpha-dry-run-order-gate",
            "--run-id",
            "v140-dry-run",
            "--session-id",
            "session-1",
            "--strategy-id",
            "ema_cross_btcusdt_v1",
            "--symbol",
            "BTCUSDT",
            "--side",
            "BUY",
            "--order-type",
            "LIMIT",
            "--quantity",
            "0.001",
            "--notional",
            "10.00",
            "--output",
            "runs/v140/live-alpha-dry-run-order-gate.json",
            "--allow-production-live-alpha-dry-run",
            "--confirm-owner-approved-dry-run",
            "--confirm-no-production-order-submission",
            "--confirm-no-production-order-mutation",
            "--confirm-no-execution-adapter-call",
            "--confirm-no-listen-key-lifecycle",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-real-funds",
        ])
        .expect("live production-live-alpha-dry-run-order-gate should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionLiveAlphaDryRunOrderGate(gate) = live.command else {
            panic!("expected production-live-alpha-dry-run-order-gate command");
        };

        assert_eq!(gate.run_id, "v140-dry-run");
        assert_eq!(gate.session_id.as_deref(), Some("session-1"));
        assert_eq!(gate.strategy_id, "ema_cross_btcusdt_v1");
        assert_eq!(gate.symbol, "BTCUSDT");
        assert_eq!(gate.side, "BUY");
        assert_eq!(gate.order_type, "LIMIT");
        assert_eq!(gate.quantity, "0.001");
        assert_eq!(gate.notional, "10.00");
        assert_eq!(
            gate.output,
            PathBuf::from("runs/v140/live-alpha-dry-run-order-gate.json")
        );
        assert!(gate.allow_production_live_alpha_dry_run);
        assert!(gate.confirm_owner_approved_dry_run);
        assert!(gate.confirm_no_production_order_submission);
        assert!(gate.confirm_no_production_order_mutation);
        assert!(gate.confirm_no_execution_adapter_call);
        assert!(gate.confirm_no_listen_key_lifecycle);
        assert!(gate.confirm_dashboard_order_controls_disabled);
        assert!(gate.confirm_no_real_funds);
    }

    #[test]
    fn parses_live_production_live_alpha_order_request_preview_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-live-alpha-order-request-preview",
            "--run-id",
            "v150-request-preview",
            "--order-gate",
            "runs/v150/order-gate.json",
            "--manual-approval-lifecycle",
            "runs/v150/manual-approval-lifecycle.json",
            "--endpoint-path",
            "/api/v3/order",
            "--price",
            "10000.00",
            "--time-in-force",
            "GTC",
            "--timestamp-ms",
            "1718400000000",
            "--recv-window-ms",
            "5000",
            "--api-key-env",
            "NTPRO_V150002_API_KEY",
            "--api-secret-env",
            "NTPRO_V150002_API_SECRET",
            "--output",
            "runs/v150/order-request-preview.json",
            "--allow-production-live-alpha-request-preview",
            "--confirm-owner-approved-request-preview",
            "--confirm-memory-only-signature",
            "--confirm-no-production-order-submission",
            "--confirm-no-production-order-mutation",
            "--confirm-no-execution-adapter-call",
            "--confirm-no-network",
            "--confirm-no-listen-key-lifecycle",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-real-funds",
        ])
        .expect("live production-live-alpha-order-request-preview should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionLiveAlphaOrderRequestPreview(preview) = live.command else {
            panic!("expected production-live-alpha-order-request-preview command");
        };

        assert_eq!(preview.run_id, "v150-request-preview");
        assert_eq!(
            preview.order_gate,
            PathBuf::from("runs/v150/order-gate.json")
        );
        assert_eq!(
            preview.manual_approval_lifecycle,
            PathBuf::from("runs/v150/manual-approval-lifecycle.json")
        );
        assert_eq!(preview.endpoint_path, "/api/v3/order");
        assert_eq!(preview.price, "10000.00");
        assert_eq!(preview.time_in_force, "GTC");
        assert_eq!(preview.timestamp_ms, 1_718_400_000_000);
        assert_eq!(preview.recv_window_ms, 5_000);
        assert_eq!(preview.api_key_env, "NTPRO_V150002_API_KEY");
        assert_eq!(preview.api_secret_env, "NTPRO_V150002_API_SECRET");
        assert_eq!(
            preview.output,
            PathBuf::from("runs/v150/order-request-preview.json")
        );
        assert!(preview.allow_production_live_alpha_request_preview);
        assert!(preview.confirm_owner_approved_request_preview);
        assert!(preview.confirm_memory_only_signature);
        assert!(preview.confirm_no_production_order_submission);
        assert!(preview.confirm_no_production_order_mutation);
        assert!(preview.confirm_no_execution_adapter_call);
        assert!(preview.confirm_no_network);
        assert!(preview.confirm_no_listen_key_lifecycle);
        assert!(preview.confirm_dashboard_order_controls_disabled);
        assert!(preview.confirm_no_real_funds);
    }

    #[test]
    fn parses_live_production_live_alpha_manual_approval_lifecycle_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-live-alpha-manual-approval-lifecycle",
            "--run-id",
            "v150-request-preview",
            "--strategy-id",
            "ema_cross_btcusdt_v1",
            "--symbol",
            "BTCUSDT",
            "--notional",
            "10.00",
            "--approval-state",
            "approved",
            "--manual-approval-id",
            "owner-approval-v150-005",
            "--approved-by",
            "owner",
            "--now-unix-ms",
            "1718400000000",
            "--expires-at-unix-ms",
            "1718400060000",
            "--output",
            "runs/v150/manual-approval-lifecycle.json",
            "--confirm-dry-run-request-preview-only",
            "--confirm-one-time-approval",
            "--confirm-no-production-mutation",
            "--confirm-dashboard-order-controls-disabled",
        ])
        .expect("live production-live-alpha-manual-approval-lifecycle should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionLiveAlphaManualApprovalLifecycle(approval) = live.command else {
            panic!("expected production-live-alpha-manual-approval-lifecycle command");
        };

        assert_eq!(approval.run_id, "v150-request-preview");
        assert_eq!(approval.strategy_id, "ema_cross_btcusdt_v1");
        assert_eq!(approval.symbol, "BTCUSDT");
        assert_eq!(approval.notional, "10.00");
        assert_eq!(approval.approval_state, "approved");
        assert_eq!(
            approval.manual_approval_id.as_deref(),
            Some("owner-approval-v150-005")
        );
        assert_eq!(approval.approved_by.as_deref(), Some("owner"));
        assert_eq!(approval.now_unix_ms, 1_718_400_000_000);
        assert_eq!(approval.expires_at_unix_ms, 1_718_400_060_000);
        assert_eq!(
            approval.output,
            PathBuf::from("runs/v150/manual-approval-lifecycle.json")
        );
        assert!(approval.confirm_dry_run_request_preview_only);
        assert!(approval.confirm_one_time_approval);
        assert!(approval.confirm_no_production_mutation);
        assert!(approval.confirm_dashboard_order_controls_disabled);
    }

    #[test]
    fn parses_live_production_live_alpha_execution_dry_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-live-alpha-execution-dry-run",
            "--run-id",
            "v150-execution-dry-run",
            "--order-gate",
            "runs/v150/order-gate.json",
            "--risk-preflight",
            "runs/v150/risk-preflight.json",
            "--request-preview",
            "runs/v150/request-preview.json",
            "--kill-switch-runtime-gate",
            "runs/v150/kill-switch-runtime-gate.json",
            "--output",
            "runs/v150/execution-dry-run.json",
            "--allow-production-live-alpha-execution-dry-run",
            "--confirm-owner-approved-execution-dry-run",
            "--confirm-dry-run-adapter-only",
            "--confirm-no-production-adapter",
            "--confirm-no-production-order-submission",
            "--confirm-no-production-order-mutation",
            "--confirm-no-network",
            "--confirm-no-listen-key-lifecycle",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-real-funds",
        ])
        .expect("live production-live-alpha-execution-dry-run should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionLiveAlphaExecutionDryRun(dry_run) = live.command else {
            panic!("expected production-live-alpha-execution-dry-run command");
        };

        assert_eq!(dry_run.run_id, "v150-execution-dry-run");
        assert_eq!(
            dry_run.order_gate,
            PathBuf::from("runs/v150/order-gate.json")
        );
        assert_eq!(
            dry_run.risk_preflight,
            PathBuf::from("runs/v150/risk-preflight.json")
        );
        assert_eq!(
            dry_run.request_preview,
            PathBuf::from("runs/v150/request-preview.json")
        );
        assert_eq!(
            dry_run.kill_switch_runtime_gate,
            PathBuf::from("runs/v150/kill-switch-runtime-gate.json")
        );
        assert_eq!(
            dry_run.output,
            PathBuf::from("runs/v150/execution-dry-run.json")
        );
        assert!(dry_run.allow_production_live_alpha_execution_dry_run);
        assert!(dry_run.confirm_owner_approved_execution_dry_run);
        assert!(dry_run.confirm_dry_run_adapter_only);
        assert!(dry_run.confirm_no_production_adapter);
        assert!(dry_run.confirm_no_production_order_submission);
        assert!(dry_run.confirm_no_production_order_mutation);
        assert!(dry_run.confirm_no_network);
        assert!(dry_run.confirm_no_listen_key_lifecycle);
        assert!(dry_run.confirm_dashboard_order_controls_disabled);
        assert!(dry_run.confirm_no_real_funds);
    }

    #[test]
    fn parses_live_production_live_alpha_kill_switch_runtime_gate_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-live-alpha-kill-switch-runtime-gate",
            "--run-id",
            "v150-kill-switch-runtime-gate",
            "--kill-switch-approval",
            "runs/v150/kill-switch-approval.json",
            "--risk-preflight",
            "runs/v150/risk-preflight.json",
            "--request-preview",
            "runs/v150/request-preview.json",
            "--output",
            "runs/v150/kill-switch-runtime-gate.json",
            "--allow-production-live-alpha-kill-switch-runtime-gate",
            "--confirm-owner-approved-runtime-gate",
            "--confirm-no-production-order-submission",
            "--confirm-no-production-order-mutation",
            "--confirm-no-network",
            "--confirm-no-listen-key-lifecycle",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-real-funds",
        ])
        .expect("live production-live-alpha-kill-switch-runtime-gate should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionLiveAlphaKillSwitchRuntimeGate(gate) = live.command else {
            panic!("expected production-live-alpha-kill-switch-runtime-gate command");
        };

        assert_eq!(gate.run_id, "v150-kill-switch-runtime-gate");
        assert_eq!(
            gate.kill_switch_approval,
            PathBuf::from("runs/v150/kill-switch-approval.json")
        );
        assert_eq!(
            gate.risk_preflight,
            PathBuf::from("runs/v150/risk-preflight.json")
        );
        assert_eq!(
            gate.request_preview,
            PathBuf::from("runs/v150/request-preview.json")
        );
        assert_eq!(
            gate.output,
            PathBuf::from("runs/v150/kill-switch-runtime-gate.json")
        );
        assert!(gate.allow_production_live_alpha_kill_switch_runtime_gate);
        assert!(gate.confirm_owner_approved_runtime_gate);
        assert!(gate.confirm_no_production_order_submission);
        assert!(gate.confirm_no_production_order_mutation);
        assert!(gate.confirm_no_network);
        assert!(gate.confirm_no_listen_key_lifecycle);
        assert!(gate.confirm_dashboard_order_controls_disabled);
        assert!(gate.confirm_no_real_funds);
    }

    #[test]
    fn parses_live_production_mutation_runtime_gate_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-runtime-gate",
            "--run-id",
            "v160-runtime-gate",
            "--order-gate",
            "runs/v160/order-gate.json",
            "--risk-preflight",
            "runs/v160/risk-preflight.json",
            "--request-preview",
            "runs/v160/request-preview.json",
            "--kill-switch-runtime-gate",
            "runs/v160/kill-switch-runtime-gate.json",
            "--signing-approval",
            "runs/v160/signing-approval.json",
            "--output",
            "runs/v160/production-mutation-runtime-gate.json",
            "--max-notional",
            "10.00",
            "--allow-production-mutation-runtime-gate",
            "--confirm-owner-approved-production-mutation",
            "--confirm-single-limit-gtc",
            "--confirm-tiny-notional",
            "--confirm-signing-approval-required",
            "--confirm-no-network-before-send",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-listen-key-lifecycle",
            "--confirm-no-retry",
        ])
        .expect("live production-mutation-runtime-gate should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationRuntimeGate(gate) = live.command else {
            panic!("expected production-mutation-runtime-gate command");
        };

        assert_eq!(gate.run_id, "v160-runtime-gate");
        assert_eq!(gate.order_gate, PathBuf::from("runs/v160/order-gate.json"));
        assert_eq!(
            gate.risk_preflight,
            PathBuf::from("runs/v160/risk-preflight.json")
        );
        assert_eq!(
            gate.request_preview,
            PathBuf::from("runs/v160/request-preview.json")
        );
        assert_eq!(
            gate.kill_switch_runtime_gate,
            PathBuf::from("runs/v160/kill-switch-runtime-gate.json")
        );
        assert_eq!(
            gate.signing_approval,
            Some(PathBuf::from("runs/v160/signing-approval.json"))
        );
        assert_eq!(
            gate.output,
            PathBuf::from("runs/v160/production-mutation-runtime-gate.json")
        );
        assert_eq!(gate.max_notional, "10.00");
        assert!(gate.allow_production_mutation_runtime_gate);
        assert!(gate.confirm_owner_approved_production_mutation);
        assert!(gate.confirm_single_limit_gtc);
        assert!(gate.confirm_tiny_notional);
        assert!(gate.confirm_signing_approval_required);
        assert!(gate.confirm_no_network_before_send);
        assert!(gate.confirm_dashboard_order_controls_disabled);
        assert!(gate.confirm_no_listen_key_lifecycle);
        assert!(gate.confirm_no_retry);
    }

    #[test]
    fn parses_live_production_mutation_signing_approval_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-signing-approval",
            "--run-id",
            "v160-signing-approval",
            "--request-preview",
            "runs/v160/request-preview.json",
            "--approval-state",
            "approved",
            "--manual-approval-id",
            "owner-approval-v160-003",
            "--approved-by",
            "owner",
            "--now-unix-ms",
            "1718400000000",
            "--expires-at-unix-ms",
            "1718400060000",
            "--output",
            "runs/v160/signing-approval.json",
            "--allow-production-mutation-signing-approval",
            "--confirm-owner-approved-signing-material",
            "--confirm-env-only-signing-material",
            "--confirm-memory-only-signing",
            "--confirm-no-secret-persistence",
            "--confirm-no-network",
            "--confirm-no-production-order-submission",
            "--confirm-no-production-order-mutation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-listen-key-lifecycle",
        ])
        .expect("live production-mutation-signing-approval should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationSigningApproval(approval) = live.command else {
            panic!("expected production-mutation-signing-approval command");
        };

        assert_eq!(approval.run_id, "v160-signing-approval");
        assert_eq!(
            approval.request_preview,
            PathBuf::from("runs/v160/request-preview.json")
        );
        assert_eq!(approval.approval_state, "approved");
        assert_eq!(
            approval.manual_approval_id.as_deref(),
            Some("owner-approval-v160-003")
        );
        assert_eq!(approval.approved_by.as_deref(), Some("owner"));
        assert_eq!(approval.now_unix_ms, 1_718_400_000_000);
        assert_eq!(approval.expires_at_unix_ms, 1_718_400_060_000);
        assert_eq!(
            approval.output,
            PathBuf::from("runs/v160/signing-approval.json")
        );
        assert!(approval.allow_production_mutation_signing_approval);
        assert!(approval.confirm_owner_approved_signing_material);
        assert!(approval.confirm_env_only_signing_material);
        assert!(approval.confirm_memory_only_signing);
        assert!(approval.confirm_no_secret_persistence);
        assert!(approval.confirm_no_network);
        assert!(approval.confirm_no_production_order_submission);
        assert!(approval.confirm_no_production_order_mutation);
        assert!(approval.confirm_dashboard_order_controls_disabled);
        assert!(approval.confirm_no_listen_key_lifecycle);
    }

    #[test]
    fn parses_live_production_mutation_request_builder_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-request-builder",
            "--run-id",
            "v160-request-builder",
            "--runtime-gate",
            "runs/v160/runtime-gate.json",
            "--signing-approval",
            "runs/v160/signing-approval.json",
            "--request-preview",
            "runs/v160/request-preview.json",
            "--api-key-env",
            "NTPRO_V160004_API_KEY",
            "--api-secret-env",
            "NTPRO_V160004_API_SECRET",
            "--timestamp-ms",
            "1718400000000",
            "--recv-window-ms",
            "5000",
            "--max-notional",
            "10.00",
            "--market-reference-source",
            "fixture_mid_price",
            "--market-reference-price",
            "10001.00",
            "--max-reference-price-distance-bps",
            "50",
            "--output",
            "runs/v160/request-builder.json",
            "--allow-production-mutation-request-builder",
            "--confirm-owner-approved-request-builder",
            "--confirm-single-limit-gtc",
            "--confirm-tiny-notional",
            "--confirm-non-marketable-price",
            "--confirm-owner-acknowledged-no-cancel-path",
            "--confirm-signing-approval-ready",
            "--confirm-memory-only-signing",
            "--confirm-no-secret-persistence",
            "--confirm-no-network",
            "--confirm-no-production-order-submission",
            "--confirm-no-production-order-mutation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-listen-key-lifecycle",
            "--confirm-no-retry",
        ])
        .expect("live production-mutation-request-builder should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationRequestBuilder(builder) = live.command else {
            panic!("expected production-mutation-request-builder command");
        };

        assert_eq!(builder.run_id, "v160-request-builder");
        assert_eq!(
            builder.runtime_gate,
            PathBuf::from("runs/v160/runtime-gate.json")
        );
        assert_eq!(
            builder.signing_approval,
            PathBuf::from("runs/v160/signing-approval.json")
        );
        assert_eq!(
            builder.request_preview,
            PathBuf::from("runs/v160/request-preview.json")
        );
        assert_eq!(builder.api_key_env, "NTPRO_V160004_API_KEY");
        assert_eq!(builder.api_secret_env, "NTPRO_V160004_API_SECRET");
        assert_eq!(builder.timestamp_ms, 1_718_400_000_000);
        assert_eq!(builder.recv_window_ms, 5_000);
        assert_eq!(builder.max_notional, "10.00");
        assert_eq!(builder.market_reference_source, "fixture_mid_price");
        assert_eq!(builder.market_reference_price, "10001.00");
        assert_eq!(builder.max_reference_price_distance_bps, "50");
        assert!(!builder.would_cross_spread);
        assert_eq!(
            builder.output,
            PathBuf::from("runs/v160/request-builder.json")
        );
        assert!(builder.allow_production_mutation_request_builder);
        assert!(builder.confirm_owner_approved_request_builder);
        assert!(builder.confirm_single_limit_gtc);
        assert!(builder.confirm_tiny_notional);
        assert!(builder.confirm_non_marketable_price);
        assert!(builder.confirm_owner_acknowledged_no_cancel_path);
        assert!(builder.confirm_signing_approval_ready);
        assert!(builder.confirm_memory_only_signing);
        assert!(builder.confirm_no_secret_persistence);
        assert!(builder.confirm_no_network);
        assert!(builder.confirm_no_production_order_submission);
        assert!(builder.confirm_no_production_order_mutation);
        assert!(builder.confirm_dashboard_order_controls_disabled);
        assert!(builder.confirm_no_listen_key_lifecycle);
        assert!(builder.confirm_no_retry);
    }

    #[test]
    fn parses_live_production_mutation_guarded_send_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-guarded-send",
            "--run-id",
            "v160-guarded-send",
            "--request-builder",
            "runs/v160/request-builder.json",
            "--kill-switch-runtime-gate",
            "runs/v160/kill-switch-runtime-gate.json",
            "--request-preview",
            "runs/v160/request-preview.json",
            "--api-key-env",
            "NTPRO_V160005_API_KEY",
            "--api-secret-env",
            "NTPRO_V160005_API_SECRET",
            "--timestamp-ms",
            "1718400000000",
            "--recv-window-ms",
            "5000",
            "--max-notional",
            "10.00",
            "--output",
            "runs/v160/guarded-send.json",
            "--manual-online",
            "--allow-production-mutation-guarded-send",
            "--confirm-owner-approved-guarded-send",
            "--confirm-single-limit-gtc",
            "--confirm-tiny-notional",
            "--confirm-single-shot",
            "--confirm-no-retry",
            "--confirm-no-secret-persistence",
            "--confirm-response-redacted",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-listen-key-lifecycle",
        ])
        .expect("live production-mutation-guarded-send should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationGuardedSend(send) = live.command else {
            panic!("expected production-mutation-guarded-send command");
        };

        assert_eq!(send.run_id, "v160-guarded-send");
        assert_eq!(
            send.request_builder,
            PathBuf::from("runs/v160/request-builder.json")
        );
        assert_eq!(
            send.kill_switch_runtime_gate,
            PathBuf::from("runs/v160/kill-switch-runtime-gate.json")
        );
        assert_eq!(
            send.request_preview,
            PathBuf::from("runs/v160/request-preview.json")
        );
        assert_eq!(send.api_key_env, "NTPRO_V160005_API_KEY");
        assert_eq!(send.api_secret_env, "NTPRO_V160005_API_SECRET");
        assert_eq!(send.timestamp_ms, 1_718_400_000_000);
        assert_eq!(send.recv_window_ms, 5_000);
        assert_eq!(send.max_notional, "10.00");
        assert_eq!(send.output, PathBuf::from("runs/v160/guarded-send.json"));
        assert!(send.manual_online);
        assert!(send.allow_production_mutation_guarded_send);
        assert!(send.confirm_owner_approved_guarded_send);
        assert!(send.confirm_single_limit_gtc);
        assert!(send.confirm_tiny_notional);
        assert!(send.confirm_single_shot);
        assert!(send.confirm_no_retry);
        assert!(send.confirm_no_secret_persistence);
        assert!(send.confirm_response_redacted);
        assert!(send.confirm_dashboard_order_controls_disabled);
        assert!(send.confirm_no_listen_key_lifecycle);
    }

    #[test]
    fn parses_live_production_mutation_response_redaction_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-response-redaction",
            "--run-id",
            "v160-response-redaction",
            "--guarded-send",
            "runs/v160/guarded-send.json",
            "--response",
            "runs/v160/synthetic-response.json",
            "--output",
            "runs/v160/response-redaction.json",
            "--allow-production-mutation-response-redaction",
            "--confirm-owner-approved-response-redaction",
            "--confirm-no-raw-response-persistence",
            "--confirm-no-headers-persistence",
            "--confirm-no-secret-persistence",
            "--confirm-order-metadata-only",
            "--confirm-no-account-balances",
            "--confirm-no-unrestricted-payload",
            "--confirm-no-retry",
        ])
        .expect("live production-mutation-response-redaction should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationResponseRedaction(redaction) = live.command else {
            panic!("expected production-mutation-response-redaction command");
        };

        assert_eq!(redaction.run_id, "v160-response-redaction");
        assert_eq!(
            redaction.guarded_send,
            PathBuf::from("runs/v160/guarded-send.json")
        );
        assert_eq!(
            redaction.response,
            PathBuf::from("runs/v160/synthetic-response.json")
        );
        assert_eq!(
            redaction.output,
            PathBuf::from("runs/v160/response-redaction.json")
        );
        assert!(redaction.allow_production_mutation_response_redaction);
        assert!(redaction.confirm_owner_approved_response_redaction);
        assert!(redaction.confirm_no_raw_response_persistence);
        assert!(redaction.confirm_no_headers_persistence);
        assert!(redaction.confirm_no_secret_persistence);
        assert!(redaction.confirm_order_metadata_only);
        assert!(redaction.confirm_no_account_balances);
        assert!(redaction.confirm_no_unrestricted_payload);
        assert!(redaction.confirm_no_retry);
    }

    #[test]
    fn parses_live_production_mutation_order_state_readback_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-order-state-readback",
            "--run-id",
            "v160-order-state-readback",
            "--response-redaction",
            "runs/v160/response-redaction.json",
            "--output",
            "runs/v160/order-state-readback.json",
            "--manual-online",
            "--api-key-env",
            "NTPRO_V160007_API_KEY",
            "--api-secret-env",
            "NTPRO_V160007_API_SECRET",
            "--recv-window-ms",
            "5000",
            "--allow-production-mutation-order-state-readback",
            "--confirm-owner-approved-order-state-readback",
            "--confirm-known-order-identifier-only",
            "--confirm-read-only-get-order",
            "--confirm-response-redacted",
            "--confirm-no-production-order-mutation",
            "--confirm-no-secret-persistence",
            "--confirm-no-retry",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-listen-key-lifecycle",
        ])
        .expect("live production-mutation-order-state-readback should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationOrderStateReadback(readback) = live.command else {
            panic!("expected production-mutation-order-state-readback command");
        };

        assert_eq!(readback.run_id, "v160-order-state-readback");
        assert_eq!(
            readback.response_redaction,
            PathBuf::from("runs/v160/response-redaction.json")
        );
        assert_eq!(
            readback.output,
            PathBuf::from("runs/v160/order-state-readback.json")
        );
        assert!(readback.manual_online);
        assert_eq!(readback.api_key_env, "NTPRO_V160007_API_KEY");
        assert_eq!(readback.api_secret_env, "NTPRO_V160007_API_SECRET");
        assert_eq!(readback.recv_window_ms, 5_000);
        assert!(readback.allow_production_mutation_order_state_readback);
        assert!(readback.confirm_owner_approved_order_state_readback);
        assert!(readback.confirm_known_order_identifier_only);
        assert!(readback.confirm_read_only_get_order);
        assert!(readback.confirm_response_redacted);
        assert!(readback.confirm_no_production_order_mutation);
        assert!(readback.confirm_no_secret_persistence);
        assert!(readback.confirm_no_retry);
        assert!(readback.confirm_dashboard_order_controls_disabled);
        assert!(readback.confirm_no_listen_key_lifecycle);
    }

    #[test]
    fn parses_live_production_mutation_audit_trail_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-audit-trail",
            "--run-id",
            "v160-audit-trail",
            "--request-builder",
            "runs/v160/request-builder.json",
            "--guarded-send",
            "runs/v160/guarded-send.json",
            "--response-redaction",
            "runs/v160/response-redaction.json",
            "--order-state-readback",
            "runs/v160/order-state-readback.json",
            "--output",
            "runs/v160/audit-trail.json",
            "--allow-production-mutation-audit-trail",
            "--confirm-owner-approved-audit-trail",
            "--confirm-redacted-artifacts-only",
            "--confirm-no-secret-or-raw-payload-persistence",
            "--confirm-no-retry-or-followup-mutation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-listen-key-lifecycle",
        ])
        .expect("live production-mutation-audit-trail should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationAuditTrail(audit) = live.command else {
            panic!("expected production-mutation-audit-trail command");
        };

        assert_eq!(audit.run_id, "v160-audit-trail");
        assert_eq!(
            audit.request_builder,
            PathBuf::from("runs/v160/request-builder.json")
        );
        assert_eq!(
            audit.guarded_send,
            PathBuf::from("runs/v160/guarded-send.json")
        );
        assert_eq!(
            audit.response_redaction,
            PathBuf::from("runs/v160/response-redaction.json")
        );
        assert_eq!(
            audit.order_state_readback,
            PathBuf::from("runs/v160/order-state-readback.json")
        );
        assert_eq!(audit.output, PathBuf::from("runs/v160/audit-trail.json"));
        assert!(audit.allow_production_mutation_audit_trail);
        assert!(audit.confirm_owner_approved_audit_trail);
        assert!(audit.confirm_redacted_artifacts_only);
        assert!(audit.confirm_no_secret_or_raw_payload_persistence);
        assert!(audit.confirm_no_retry_or_followup_mutation);
        assert!(audit.confirm_dashboard_order_controls_disabled);
        assert!(audit.confirm_no_listen_key_lifecycle);
    }

    #[test]
    fn parses_live_production_mutation_failure_semantics_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-failure-semantics",
            "--run-id",
            "v160-failure-semantics",
            "--audit-trail",
            "runs/v160/audit-trail.json",
            "--failure-mode",
            "readback-mismatch",
            "--output",
            "runs/v160/failure-semantics.json",
            "--allow-production-mutation-failure-semantics",
            "--confirm-evidence-only-failure-handling",
            "--confirm-no-retry",
            "--confirm-no-automatic-cancel-replace-amend",
            "--confirm-no-correction-or-flatten",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-strategy-continuation",
            "--confirm-no-listen-key-lifecycle",
        ])
        .expect("live production-mutation-failure-semantics should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationFailureSemantics(failure) = live.command else {
            panic!("expected production-mutation-failure-semantics command");
        };

        assert_eq!(failure.run_id, "v160-failure-semantics");
        assert_eq!(
            failure.audit_trail,
            PathBuf::from("runs/v160/audit-trail.json")
        );
        assert_eq!(
            failure.failure_mode,
            ProductionMutationFailureMode::ReadbackMismatch
        );
        assert_eq!(
            failure.output,
            PathBuf::from("runs/v160/failure-semantics.json")
        );
        assert!(failure.allow_production_mutation_failure_semantics);
        assert!(failure.confirm_evidence_only_failure_handling);
        assert!(failure.confirm_no_retry);
        assert!(failure.confirm_no_automatic_cancel_replace_amend);
        assert!(failure.confirm_no_correction_or_flatten);
        assert!(failure.confirm_dashboard_order_controls_disabled);
        assert!(failure.confirm_no_strategy_continuation);
        assert!(failure.confirm_no_listen_key_lifecycle);
    }

    #[test]
    fn parses_live_production_mutation_local_order_ledger_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-local-order-ledger",
            "--run-id",
            "v170-local-order-ledger",
            "--order-lineage-id",
            "lineage-v160-single-shot",
            "--request-builder",
            "runs/v160/request-builder.json",
            "--guarded-send",
            "runs/v160/guarded-send.json",
            "--response-redaction",
            "runs/v160/response-redaction.json",
            "--order-state-readback",
            "runs/v160/order-state-readback.json",
            "--audit-trail",
            "runs/v160/audit-trail.json",
            "--failure-semantics",
            "runs/v160/failure-semantics.json",
            "--output",
            "runs/v170/local-order-ledger.json",
            "--allow-production-mutation-local-order-ledger",
            "--confirm-single-v16-mutation-candidate-lineage",
            "--confirm-read-only-reconciliation-scope",
            "--confirm-no-network",
            "--confirm-no-duplicate-submit",
            "--confirm-no-retry",
            "--confirm-no-cancel",
            "--confirm-no-remediation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-local-order-ledger should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationLocalOrderLedger(ledger) = live.command else {
            panic!("expected production-mutation-local-order-ledger command");
        };

        assert_eq!(ledger.run_id, "v170-local-order-ledger");
        assert_eq!(ledger.order_lineage_id, "lineage-v160-single-shot");
        assert_eq!(
            ledger.request_builder,
            PathBuf::from("runs/v160/request-builder.json")
        );
        assert_eq!(
            ledger.guarded_send,
            PathBuf::from("runs/v160/guarded-send.json")
        );
        assert_eq!(
            ledger.response_redaction,
            PathBuf::from("runs/v160/response-redaction.json")
        );
        assert_eq!(
            ledger.order_state_readback,
            PathBuf::from("runs/v160/order-state-readback.json")
        );
        assert_eq!(
            ledger.audit_trail,
            PathBuf::from("runs/v160/audit-trail.json")
        );
        assert_eq!(
            ledger.failure_semantics,
            PathBuf::from("runs/v160/failure-semantics.json")
        );
        assert_eq!(
            ledger.output,
            PathBuf::from("runs/v170/local-order-ledger.json")
        );
        assert!(ledger.allow_production_mutation_local_order_ledger);
        assert!(ledger.confirm_single_v16_mutation_candidate_lineage);
        assert!(ledger.confirm_read_only_reconciliation_scope);
        assert!(ledger.confirm_no_network);
        assert!(ledger.confirm_no_duplicate_submit);
        assert!(ledger.confirm_no_retry);
        assert!(ledger.confirm_no_cancel);
        assert!(ledger.confirm_no_remediation);
        assert!(ledger.confirm_dashboard_order_controls_disabled);
        assert!(ledger.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_mutation_exchange_readback_mapper_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-exchange-readback-mapper",
            "--run-id",
            "v170-exchange-readback-mapper",
            "--local-order-ledger",
            "runs/v170/local-order-ledger.json",
            "--order-readback",
            "runs/v170/order-readback-new.json",
            "--open-orders-readback",
            "runs/v170/open-orders-new.json",
            "--output",
            "runs/v170/exchange-readback-mapper.json",
            "--allow-production-mutation-exchange-readback-mapper",
            "--confirm-redacted-readback-metadata-only",
            "--confirm-known-order-identifier-only",
            "--confirm-read-only-reconciliation-scope",
            "--confirm-no-network",
            "--confirm-no-secret-persistence",
            "--confirm-no-production-order-mutation",
            "--confirm-no-retry",
            "--confirm-no-cancel",
            "--confirm-dashboard-order-controls-disabled",
        ])
        .expect("live production-mutation-exchange-readback-mapper should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationExchangeReadbackMapper(mapper) = live.command else {
            panic!("expected production-mutation-exchange-readback-mapper command");
        };

        assert_eq!(mapper.run_id, "v170-exchange-readback-mapper");
        assert_eq!(
            mapper.local_order_ledger,
            PathBuf::from("runs/v170/local-order-ledger.json")
        );
        assert_eq!(
            mapper.order_readback,
            PathBuf::from("runs/v170/order-readback-new.json")
        );
        assert_eq!(
            mapper.open_orders_readback,
            PathBuf::from("runs/v170/open-orders-new.json")
        );
        assert_eq!(
            mapper.output,
            PathBuf::from("runs/v170/exchange-readback-mapper.json")
        );
        assert!(mapper.allow_production_mutation_exchange_readback_mapper);
        assert!(mapper.confirm_redacted_readback_metadata_only);
        assert!(mapper.confirm_known_order_identifier_only);
        assert!(mapper.confirm_read_only_reconciliation_scope);
        assert!(mapper.confirm_no_network);
        assert!(mapper.confirm_no_secret_persistence);
        assert!(mapper.confirm_no_production_order_mutation);
        assert!(mapper.confirm_no_retry);
        assert!(mapper.confirm_no_cancel);
        assert!(mapper.confirm_dashboard_order_controls_disabled);
    }

    #[test]
    fn parses_live_production_mutation_reconciliation_classifier_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-reconciliation-classifier",
            "--run-id",
            "v170-reconciliation-classifier",
            "--exchange-readback-mapper",
            "runs/v170/exchange-readback-mapper.json",
            "--output",
            "runs/v170/reconciliation-classifier.json",
            "--allow-production-mutation-reconciliation-classifier",
            "--confirm-single-v16-mutation-candidate-lineage",
            "--confirm-read-only-reconciliation-scope",
            "--confirm-no-retry",
            "--confirm-no-cancel",
            "--confirm-no-remediation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-reconciliation-classifier should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationReconciliationClassifier(classifier) = live.command
        else {
            panic!("expected production-mutation-reconciliation-classifier command");
        };

        assert_eq!(classifier.run_id, "v170-reconciliation-classifier");
        assert_eq!(
            classifier.exchange_readback_mapper,
            PathBuf::from("runs/v170/exchange-readback-mapper.json")
        );
        assert_eq!(
            classifier.output,
            PathBuf::from("runs/v170/reconciliation-classifier.json")
        );
        assert!(classifier.allow_production_mutation_reconciliation_classifier);
        assert!(classifier.confirm_single_v16_mutation_candidate_lineage);
        assert!(classifier.confirm_read_only_reconciliation_scope);
        assert!(classifier.confirm_no_retry);
        assert!(classifier.confirm_no_cancel);
        assert!(classifier.confirm_no_remediation);
        assert!(classifier.confirm_dashboard_order_controls_disabled);
        assert!(classifier.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_mutation_orphan_order_detector_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-orphan-order-detector",
            "--run-id",
            "v170-orphan-order-detector",
            "--reconciliation-classifier",
            "runs/v170/reconciliation-classifier.json",
            "--output",
            "runs/v170/orphan-order-detector.json",
            "--allow-production-mutation-orphan-order-detector",
            "--confirm-single-v16-mutation-candidate-lineage",
            "--confirm-read-only-reconciliation-scope",
            "--confirm-no-retry",
            "--confirm-no-cancel",
            "--confirm-no-remediation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-orphan-order-detector should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationOrphanOrderDetector(detector) = live.command else {
            panic!("expected production-mutation-orphan-order-detector command");
        };

        assert_eq!(detector.run_id, "v170-orphan-order-detector");
        assert_eq!(
            detector.reconciliation_classifier,
            PathBuf::from("runs/v170/reconciliation-classifier.json")
        );
        assert_eq!(
            detector.output,
            PathBuf::from("runs/v170/orphan-order-detector.json")
        );
        assert!(detector.allow_production_mutation_orphan_order_detector);
        assert!(detector.confirm_single_v16_mutation_candidate_lineage);
        assert!(detector.confirm_read_only_reconciliation_scope);
        assert!(detector.confirm_no_retry);
        assert!(detector.confirm_no_cancel);
        assert!(detector.confirm_no_remediation);
        assert!(detector.confirm_dashboard_order_controls_disabled);
        assert!(detector.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_mutation_cancel_request_preview_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-cancel-request-preview",
            "--run-id",
            "v180-cancel-request-preview",
            "--orphan-order-detector",
            "runs/v170/orphan-order-detector.json",
            "--account-label",
            "prod-account-redacted",
            "--output",
            "runs/v180/cancel-request-preview.json",
            "--allow-production-mutation-cancel-request-preview",
            "--confirm-single-v16-mutation-candidate-lineage",
            "--confirm-orphan-risk-halted",
            "--confirm-manual-review-required",
            "--confirm-known-order-identifier-only",
            "--confirm-no-retry",
            "--confirm-no-cancel",
            "--confirm-no-network",
            "--confirm-no-remediation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-cancel-request-preview should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationCancelRequestPreview(preview) = live.command else {
            panic!("expected production-mutation-cancel-request-preview command");
        };

        assert_eq!(preview.run_id, "v180-cancel-request-preview");
        assert_eq!(
            preview.orphan_order_detector,
            PathBuf::from("runs/v170/orphan-order-detector.json")
        );
        assert_eq!(preview.account_label, "prod-account-redacted");
        assert_eq!(
            preview.output,
            PathBuf::from("runs/v180/cancel-request-preview.json")
        );
        assert!(preview.allow_production_mutation_cancel_request_preview);
        assert!(preview.confirm_single_v16_mutation_candidate_lineage);
        assert!(preview.confirm_orphan_risk_halted);
        assert!(preview.confirm_manual_review_required);
        assert!(preview.confirm_known_order_identifier_only);
        assert!(preview.confirm_no_retry);
        assert!(preview.confirm_no_cancel);
        assert!(preview.confirm_no_network);
        assert!(preview.confirm_no_remediation);
        assert!(preview.confirm_dashboard_order_controls_disabled);
        assert!(preview.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_mutation_cancel_risk_gate_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-cancel-risk-gate",
            "--run-id",
            "v180-cancel-risk-gate",
            "--cancel-request-preview",
            "runs/v180/cancel-request-preview.json",
            "--expected-symbol",
            "BTCUSDT",
            "--expected-account-label",
            "prod-account-redacted",
            "--output",
            "runs/v180/cancel-risk-gate.json",
            "--allow-production-mutation-cancel-risk-gate",
            "--confirm-single-v16-mutation-candidate-lineage",
            "--confirm-cancel-request-preview-ready",
            "--confirm-orphan-risk-halted",
            "--confirm-known-order-identifier-only",
            "--confirm-symbol-account-scope",
            "--confirm-owner-approval-required",
            "--confirm-no-cancel-all-or-bulk",
            "--confirm-no-retry",
            "--confirm-no-cancel",
            "--confirm-no-network",
            "--confirm-no-remediation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-cancel-risk-gate should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationCancelRiskGate(gate) = live.command else {
            panic!("expected production-mutation-cancel-risk-gate command");
        };

        assert_eq!(gate.run_id, "v180-cancel-risk-gate");
        assert_eq!(
            gate.cancel_request_preview,
            PathBuf::from("runs/v180/cancel-request-preview.json")
        );
        assert_eq!(gate.expected_symbol, "BTCUSDT");
        assert_eq!(gate.expected_account_label, "prod-account-redacted");
        assert_eq!(
            gate.output,
            PathBuf::from("runs/v180/cancel-risk-gate.json")
        );
        assert!(gate.allow_production_mutation_cancel_risk_gate);
        assert!(gate.confirm_single_v16_mutation_candidate_lineage);
        assert!(gate.confirm_cancel_request_preview_ready);
        assert!(gate.confirm_orphan_risk_halted);
        assert!(gate.confirm_known_order_identifier_only);
        assert!(gate.confirm_symbol_account_scope);
        assert!(gate.confirm_owner_approval_required);
        assert!(gate.confirm_no_cancel_all_or_bulk);
        assert!(gate.confirm_no_retry);
        assert!(gate.confirm_no_cancel);
        assert!(gate.confirm_no_network);
        assert!(gate.confirm_no_remediation);
        assert!(gate.confirm_dashboard_order_controls_disabled);
        assert!(gate.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_mutation_manual_owner_approval_lifecycle_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-manual-owner-approval-lifecycle",
            "--run-id",
            "v180-manual-owner-approval-lifecycle",
            "--cancel-risk-gate",
            "runs/v180/cancel-risk-gate.json",
            "--approval-state",
            "approved",
            "--manual-approval-id",
            "owner-approval-v180-005",
            "--approved-by",
            "owner",
            "--now-unix-ms",
            "1718400000000",
            "--expires-at-unix-ms",
            "1718400060000",
            "--output",
            "runs/v180/manual-owner-approval-lifecycle.json",
            "--allow-production-mutation-manual-owner-approval-lifecycle",
            "--confirm-one-order-cancel-candidate",
            "--confirm-one-time-approval",
            "--confirm-non-reusable-approval",
            "--confirm-approval-expiry",
            "--confirm-no-strategy-auto-approval",
            "--confirm-no-background-auto-approval",
            "--confirm-no-dashboard-cancel-approval",
            "--confirm-no-incident-handler-auto-approval",
            "--confirm-no-cancel",
            "--confirm-no-network",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-manual-owner-approval-lifecycle should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationManualOwnerApprovalLifecycle(approval) = live.command
        else {
            panic!("expected production-mutation-manual-owner-approval-lifecycle command");
        };

        assert_eq!(approval.run_id, "v180-manual-owner-approval-lifecycle");
        assert_eq!(
            approval.cancel_risk_gate,
            PathBuf::from("runs/v180/cancel-risk-gate.json")
        );
        assert_eq!(approval.approval_state, "approved");
        assert_eq!(
            approval.manual_approval_id.as_deref(),
            Some("owner-approval-v180-005")
        );
        assert_eq!(approval.approved_by.as_deref(), Some("owner"));
        assert_eq!(approval.now_unix_ms, 1_718_400_000_000);
        assert_eq!(approval.expires_at_unix_ms, 1_718_400_060_000);
        assert_eq!(
            approval.output,
            PathBuf::from("runs/v180/manual-owner-approval-lifecycle.json")
        );
        assert!(approval.allow_production_mutation_manual_owner_approval_lifecycle);
        assert!(approval.confirm_one_order_cancel_candidate);
        assert!(approval.confirm_one_time_approval);
        assert!(approval.confirm_non_reusable_approval);
        assert!(approval.confirm_approval_expiry);
        assert!(approval.confirm_no_strategy_auto_approval);
        assert!(approval.confirm_no_background_auto_approval);
        assert!(approval.confirm_no_dashboard_cancel_approval);
        assert!(approval.confirm_no_incident_handler_auto_approval);
        assert!(approval.confirm_no_cancel);
        assert!(approval.confirm_no_network);
        assert!(approval.confirm_dashboard_order_controls_disabled);
        assert!(approval.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_mutation_actual_cancel_owner_approval_lifecycle_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-actual-cancel-owner-approval-lifecycle",
            "--run-id",
            "v190-owner-approval-lifecycle",
            "--actual-cancel-safety-contract",
            "docs/rust-cutover/release/v0_19_0_actual_cancel_safety_contract.md",
            "--release-manifest",
            "docs/rust-cutover/release/v0_18_1_release_manifest.json",
            "--cancel-risk-gate",
            "runs/v180/cancel-risk-gate.json",
            "--expected-order-lineage-id",
            "lineage-v160-single-shot",
            "--expected-symbol",
            "BTCUSDT",
            "--expected-account-label",
            "prod-account-redacted",
            "--venue",
            "binance_spot",
            "--expected-release-tag",
            "ntpro-rust-only-v0.18.1",
            "--approval-state",
            "approved",
            "--manual-approval-id",
            "owner-approval-v190-003",
            "--approved-by",
            "owner",
            "--approval-reason",
            "orphan-risk-single-order-cancel",
            "--now-unix-ms",
            "1718400000000",
            "--expires-at-unix-ms",
            "1718400060000",
            "--output",
            "runs/v190/actual-cancel-owner-approval-lifecycle.json",
            "--allow-production-mutation-actual-cancel-owner-approval-lifecycle",
            "--confirm-actual-cancel-safety-contract",
            "--confirm-one-order-one-venue-one-attempt",
            "--confirm-single-use-approval",
            "--confirm-approval-expiry",
            "--confirm-bind-order-risk-gate-release-provenance",
            "--confirm-audit-evidence",
            "--confirm-no-dashboard-approval",
            "--confirm-no-automatic-cancel",
            "--confirm-no-bulk-cancel",
            "--confirm-no-retry",
            "--confirm-no-submit-lifecycle",
            "--confirm-no-network",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-actual-cancel-owner-approval-lifecycle should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationActualCancelOwnerApprovalLifecycle(approval) =
            live.command
        else {
            panic!("expected production-mutation-actual-cancel-owner-approval-lifecycle command");
        };

        assert_eq!(approval.run_id, "v190-owner-approval-lifecycle");
        assert_eq!(
            approval.actual_cancel_safety_contract,
            PathBuf::from("docs/rust-cutover/release/v0_19_0_actual_cancel_safety_contract.md")
        );
        assert_eq!(
            approval.release_manifest,
            PathBuf::from("docs/rust-cutover/release/v0_18_1_release_manifest.json")
        );
        assert_eq!(
            approval.cancel_risk_gate,
            PathBuf::from("runs/v180/cancel-risk-gate.json")
        );
        assert_eq!(
            approval.expected_order_lineage_id,
            "lineage-v160-single-shot"
        );
        assert_eq!(approval.expected_symbol, "BTCUSDT");
        assert_eq!(approval.expected_account_label, "prod-account-redacted");
        assert_eq!(approval.venue, "binance_spot");
        assert_eq!(approval.expected_release_tag, "ntpro-rust-only-v0.18.1");
        assert_eq!(approval.approval_state, "approved");
        assert_eq!(
            approval.manual_approval_id.as_deref(),
            Some("owner-approval-v190-003")
        );
        assert_eq!(approval.approved_by.as_deref(), Some("owner"));
        assert_eq!(
            approval.approval_reason.as_deref(),
            Some("orphan-risk-single-order-cancel")
        );
        assert_eq!(approval.now_unix_ms, 1_718_400_000_000);
        assert_eq!(approval.expires_at_unix_ms, 1_718_400_060_000);
        assert_eq!(
            approval.output,
            PathBuf::from("runs/v190/actual-cancel-owner-approval-lifecycle.json")
        );
        assert!(approval.allow_production_mutation_actual_cancel_owner_approval_lifecycle);
        assert!(approval.confirm_actual_cancel_safety_contract);
        assert!(approval.confirm_one_order_one_venue_one_attempt);
        assert!(approval.confirm_single_use_approval);
        assert!(approval.confirm_approval_expiry);
        assert!(approval.confirm_bind_order_risk_gate_release_provenance);
        assert!(approval.confirm_audit_evidence);
        assert!(approval.confirm_no_dashboard_approval);
        assert!(approval.confirm_no_automatic_cancel);
        assert!(approval.confirm_no_bulk_cancel);
        assert!(approval.confirm_no_retry);
        assert!(approval.confirm_no_submit_lifecycle);
        assert!(approval.confirm_no_network);
        assert!(approval.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_mutation_actual_cancel_executor_adapter_boundary_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-actual-cancel-executor-adapter-boundary",
            "--run-id",
            "v190-cancel-executor-adapter-boundary",
            "--owner-approval-lifecycle",
            "runs/v190/actual-cancel-owner-approval-lifecycle.json",
            "--adapter-capability",
            "runs/v190/adapter-capability.json",
            "--adapter-id",
            "binance_spot_cancel_adapter",
            "--venue",
            "binance_spot",
            "--order-id-type",
            "exchange_order_id",
            "--expected-order-lineage-id",
            "lineage-v160-single-shot",
            "--expected-symbol",
            "BTCUSDT",
            "--expected-account-label",
            "prod-account-redacted",
            "--output",
            "runs/v190/actual-cancel-executor-adapter-boundary.json",
            "--allow-production-mutation-actual-cancel-executor-adapter-boundary",
            "--confirm-adapter-capability",
            "--confirm-request-response-readback-audit-contract",
            "--confirm-one-order-one-venue-one-attempt",
            "--confirm-fail-closed-unsupported-capability",
            "--confirm-no-bulk-cancel",
            "--confirm-no-retry",
            "--confirm-no-automatic-cancel",
            "--confirm-no-dashboard-execution",
            "--confirm-no-network",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-actual-cancel-executor-adapter-boundary should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationActualCancelExecutorAdapterBoundary(boundary) =
            live.command
        else {
            panic!("expected production-mutation-actual-cancel-executor-adapter-boundary command");
        };

        assert_eq!(boundary.run_id, "v190-cancel-executor-adapter-boundary");
        assert_eq!(
            boundary.owner_approval_lifecycle,
            PathBuf::from("runs/v190/actual-cancel-owner-approval-lifecycle.json")
        );
        assert_eq!(
            boundary.adapter_capability,
            PathBuf::from("runs/v190/adapter-capability.json")
        );
        assert_eq!(boundary.adapter_id, "binance_spot_cancel_adapter");
        assert_eq!(boundary.venue, "binance_spot");
        assert_eq!(boundary.order_id_type, "exchange_order_id");
        assert_eq!(
            boundary.expected_order_lineage_id,
            "lineage-v160-single-shot"
        );
        assert_eq!(boundary.expected_symbol, "BTCUSDT");
        assert_eq!(boundary.expected_account_label, "prod-account-redacted");
        assert_eq!(
            boundary.output,
            PathBuf::from("runs/v190/actual-cancel-executor-adapter-boundary.json")
        );
        assert!(boundary.allow_production_mutation_actual_cancel_executor_adapter_boundary);
        assert!(boundary.confirm_adapter_capability);
        assert!(boundary.confirm_request_response_readback_audit_contract);
        assert!(boundary.confirm_one_order_one_venue_one_attempt);
        assert!(boundary.confirm_fail_closed_unsupported_capability);
        assert!(boundary.confirm_no_bulk_cancel);
        assert!(boundary.confirm_no_retry);
        assert!(boundary.confirm_no_automatic_cancel);
        assert!(boundary.confirm_no_dashboard_execution);
        assert!(boundary.confirm_no_network);
        assert!(boundary.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_mutation_cancel_response_redaction_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-cancel-response-redaction",
            "--run-id",
            "v180-cancel-response-redaction",
            "--manual-owner-approval-lifecycle",
            "runs/v180/manual-owner-approval-lifecycle.json",
            "--response",
            "runs/v180/synthetic-cancel-response.json",
            "--output",
            "runs/v180/cancel-response-redaction.json",
            "--allow-production-mutation-cancel-response-redaction",
            "--confirm-manual-owner-approval-lifecycle-ready",
            "--confirm-no-raw-response-persistence",
            "--confirm-no-headers-persistence",
            "--confirm-no-secret-persistence",
            "--confirm-cancel-metadata-only",
            "--confirm-no-account-balances",
            "--confirm-no-unrestricted-payload",
            "--confirm-no-retry",
            "--confirm-no-cancel",
            "--confirm-no-network",
            "--confirm-dashboard-order-controls-disabled",
        ])
        .expect("live production-mutation-cancel-response-redaction should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationCancelResponseRedaction(redaction) = live.command else {
            panic!("expected production-mutation-cancel-response-redaction command");
        };

        assert_eq!(redaction.run_id, "v180-cancel-response-redaction");
        assert_eq!(
            redaction.manual_owner_approval_lifecycle,
            PathBuf::from("runs/v180/manual-owner-approval-lifecycle.json")
        );
        assert_eq!(
            redaction.response,
            PathBuf::from("runs/v180/synthetic-cancel-response.json")
        );
        assert_eq!(
            redaction.output,
            PathBuf::from("runs/v180/cancel-response-redaction.json")
        );
        assert!(redaction.allow_production_mutation_cancel_response_redaction);
        assert!(redaction.confirm_manual_owner_approval_lifecycle_ready);
        assert!(redaction.confirm_no_raw_response_persistence);
        assert!(redaction.confirm_no_headers_persistence);
        assert!(redaction.confirm_no_secret_persistence);
        assert!(redaction.confirm_cancel_metadata_only);
        assert!(redaction.confirm_no_account_balances);
        assert!(redaction.confirm_no_unrestricted_payload);
        assert!(redaction.confirm_no_retry);
        assert!(redaction.confirm_no_cancel);
        assert!(redaction.confirm_no_network);
        assert!(redaction.confirm_dashboard_order_controls_disabled);
    }

    #[test]
    fn parses_live_production_mutation_post_cancel_readback_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-post-cancel-readback",
            "--run-id",
            "v180-post-cancel-readback",
            "--cancel-response-redaction",
            "runs/v180/cancel-response-redaction.json",
            "--readback",
            "runs/v180/post-cancel-readback-canceled.json",
            "--output",
            "runs/v180/post-cancel-readback.json",
            "--allow-production-mutation-post-cancel-readback",
            "--confirm-cancel-response-redaction-ready",
            "--confirm-readback-metadata-only",
            "--confirm-terminal-and-ambiguous-classification",
            "--confirm-no-raw-readback-persistence",
            "--confirm-no-headers-persistence",
            "--confirm-no-secret-persistence",
            "--confirm-no-mutation",
            "--confirm-no-retry",
            "--confirm-no-remediation",
            "--confirm-no-cancel",
            "--confirm-no-network",
            "--confirm-dashboard-order-controls-disabled",
        ])
        .expect("live production-mutation-post-cancel-readback should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationPostCancelReadback(readback) = live.command else {
            panic!("expected production-mutation-post-cancel-readback command");
        };

        assert_eq!(readback.run_id, "v180-post-cancel-readback");
        assert_eq!(
            readback.cancel_response_redaction,
            PathBuf::from("runs/v180/cancel-response-redaction.json")
        );
        assert_eq!(
            readback.readback,
            PathBuf::from("runs/v180/post-cancel-readback-canceled.json")
        );
        assert_eq!(
            readback.output,
            PathBuf::from("runs/v180/post-cancel-readback.json")
        );
        assert!(readback.allow_production_mutation_post_cancel_readback);
        assert!(readback.confirm_cancel_response_redaction_ready);
        assert!(readback.confirm_readback_metadata_only);
        assert!(readback.confirm_terminal_and_ambiguous_classification);
        assert!(readback.confirm_no_raw_readback_persistence);
        assert!(readback.confirm_no_headers_persistence);
        assert!(readback.confirm_no_secret_persistence);
        assert!(readback.confirm_no_mutation);
        assert!(readback.confirm_no_retry);
        assert!(readback.confirm_no_remediation);
        assert!(readback.confirm_no_cancel);
        assert!(readback.confirm_no_network);
        assert!(readback.confirm_dashboard_order_controls_disabled);
    }

    #[test]
    fn parses_live_production_mutation_cancel_recovery_incident_audit_closeout_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-mutation-cancel-recovery-incident-audit-closeout",
            "--run-id",
            "v180-incident-audit-closeout",
            "--cancel-risk-gate",
            "runs/v180/cancel-risk-gate.json",
            "--manual-owner-approval-lifecycle",
            "runs/v180/manual-owner-approval-lifecycle.json",
            "--cancel-response-redaction",
            "runs/v180/cancel-response-redaction.json",
            "--post-cancel-readback",
            "runs/v180/post-cancel-readback.json",
            "--output",
            "runs/v180/incident-audit-closeout.json",
            "--allow-production-mutation-cancel-recovery-incident-audit-closeout",
            "--confirm-cancel-recovery-lineage",
            "--confirm-risk-reason-recorded",
            "--confirm-risk-gate-result-recorded",
            "--confirm-owner-approval-state-recorded",
            "--confirm-redaction-contract-state-recorded",
            "--confirm-readback-state-recorded",
            "--confirm-terminal-action-recommendation",
            "--confirm-remaining-risk-recorded",
            "--confirm-no-mutation",
            "--confirm-no-cancel",
            "--confirm-no-network",
            "--confirm-no-retry",
            "--confirm-no-remediation",
            "--confirm-no-automatic-remediation",
            "--confirm-dashboard-order-controls-disabled",
            "--confirm-no-secret-persistence",
        ])
        .expect("live production-mutation-cancel-recovery-incident-audit-closeout should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionMutationCancelRecoveryIncidentAuditCloseout(closeout) =
            live.command
        else {
            panic!("expected production-mutation-cancel-recovery-incident-audit-closeout command");
        };

        assert_eq!(closeout.run_id, "v180-incident-audit-closeout");
        assert_eq!(
            closeout.cancel_risk_gate,
            PathBuf::from("runs/v180/cancel-risk-gate.json")
        );
        assert_eq!(
            closeout.manual_owner_approval_lifecycle,
            PathBuf::from("runs/v180/manual-owner-approval-lifecycle.json")
        );
        assert_eq!(
            closeout.cancel_response_redaction,
            PathBuf::from("runs/v180/cancel-response-redaction.json")
        );
        assert_eq!(
            closeout.post_cancel_readback,
            PathBuf::from("runs/v180/post-cancel-readback.json")
        );
        assert_eq!(
            closeout.output,
            PathBuf::from("runs/v180/incident-audit-closeout.json")
        );
        assert!(closeout.allow_production_mutation_cancel_recovery_incident_audit_closeout);
        assert!(closeout.confirm_cancel_recovery_lineage);
        assert!(closeout.confirm_risk_reason_recorded);
        assert!(closeout.confirm_risk_gate_result_recorded);
        assert!(closeout.confirm_owner_approval_state_recorded);
        assert!(closeout.confirm_redaction_contract_state_recorded);
        assert!(closeout.confirm_readback_state_recorded);
        assert!(closeout.confirm_terminal_action_recommendation);
        assert!(closeout.confirm_remaining_risk_recorded);
        assert!(closeout.confirm_no_mutation);
        assert!(closeout.confirm_no_cancel);
        assert!(closeout.confirm_no_network);
        assert!(closeout.confirm_no_retry);
        assert!(closeout.confirm_no_remediation);
        assert!(closeout.confirm_no_automatic_remediation);
        assert!(closeout.confirm_dashboard_order_controls_disabled);
        assert!(closeout.confirm_no_secret_persistence);
    }

    #[test]
    fn parses_live_production_live_alpha_risk_preflight_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "production-live-alpha-risk-preflight",
            "--run-id",
            "v140-risk",
            "--order-gate",
            "runs/v140/order-gate.json",
            "--input",
            "runs/v140/risk-input.json",
            "--output",
            "runs/v140/risk-preflight.json",
            "--confirm-hypothetical-dry-run-only",
            "--confirm-no-execution-adapter-call",
            "--confirm-no-production-order-submission",
            "--confirm-no-production-order-mutation",
            "--confirm-dashboard-order-controls-disabled",
        ])
        .expect("live production-live-alpha-risk-preflight should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::ProductionLiveAlphaRiskPreflight(preflight) = live.command else {
            panic!("expected production-live-alpha-risk-preflight command");
        };

        assert_eq!(preflight.run_id, "v140-risk");
        assert_eq!(
            preflight.order_gate,
            PathBuf::from("runs/v140/order-gate.json")
        );
        assert_eq!(preflight.input, PathBuf::from("runs/v140/risk-input.json"));
        assert_eq!(
            preflight.output,
            PathBuf::from("runs/v140/risk-preflight.json")
        );
        assert!(preflight.confirm_hypothetical_dry_run_only);
        assert!(preflight.confirm_no_execution_adapter_call);
        assert!(preflight.confirm_no_production_order_submission);
        assert!(preflight.confirm_no_production_order_mutation);
        assert!(preflight.confirm_dashboard_order_controls_disabled);
    }

    #[test]
    fn parses_live_testnet_order_gate_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "testnet-order-gate",
            "--config",
            "configs/nodes/btc-ema-shadow.toml",
            "--allow-testnet-order",
            "--confirm-owner-approved-testnet-order",
            "--confirm-tiny-notional",
            "--confirm-cancel-after-submit",
        ])
        .expect("live testnet-order-gate should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::TestnetOrderGate(gate) = live.command else {
            panic!("expected testnet-order-gate command");
        };

        assert_eq!(
            gate.config,
            PathBuf::from("configs/nodes/btc-ema-shadow.toml")
        );
        assert!(gate.allow_testnet_order);
        assert!(gate.confirm_owner_approved_testnet_order);
        assert!(gate.confirm_tiny_notional);
        assert!(gate.confirm_cancel_after_submit);
    }

    #[test]
    fn parses_live_testnet_order_preflight_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "testnet-order-preflight",
            "--config",
            "configs/nodes/btc-ema-shadow.toml",
            "--input",
            "runs/v100/preflight-input.json",
            "--output",
            "runs/v100/preflight-report.json",
            "--allow-testnet-order",
            "--confirm-owner-approved-testnet-order",
            "--confirm-tiny-notional",
            "--confirm-cancel-after-submit",
        ])
        .expect("live testnet-order-preflight should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::TestnetOrderPreflight(preflight) = live.command else {
            panic!("expected testnet-order-preflight command");
        };

        assert_eq!(
            preflight.config,
            PathBuf::from("configs/nodes/btc-ema-shadow.toml")
        );
        assert_eq!(
            preflight.input,
            PathBuf::from("runs/v100/preflight-input.json")
        );
        assert_eq!(
            preflight.output,
            Some(PathBuf::from("runs/v100/preflight-report.json"))
        );
        assert!(preflight.allow_testnet_order);
        assert!(preflight.confirm_owner_approved_testnet_order);
        assert!(preflight.confirm_tiny_notional);
        assert!(preflight.confirm_cancel_after_submit);
    }

    #[test]
    fn parses_live_testnet_order_request_preview_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "testnet-order-request-preview",
            "--config",
            "configs/nodes/btc-ema-shadow.toml",
            "--method",
            "DELETE",
            "--endpoint-path",
            "/api/v3/order",
            "--timestamp-ms",
            "1718400000000",
            "--recv-window-ms",
            "2500",
            "--api-key-env",
            "NTPRO_TEST_KEY",
            "--api-secret-env",
            "NTPRO_TEST_SECRET",
            "--orig-client-order-id",
            "ntpro-cancel-001",
            "--output",
            "runs/v100/request-preview.json",
            "--allow-testnet-order",
            "--confirm-owner-approved-testnet-order",
            "--confirm-tiny-notional",
            "--confirm-cancel-after-submit",
        ])
        .expect("live testnet-order-request-preview should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::TestnetOrderRequestPreview(preview) = live.command else {
            panic!("expected testnet-order-request-preview command");
        };

        assert_eq!(
            preview.config,
            PathBuf::from("configs/nodes/btc-ema-shadow.toml")
        );
        assert_eq!(preview.method, "DELETE");
        assert_eq!(preview.endpoint_path, "/api/v3/order");
        assert_eq!(preview.timestamp_ms, 1_718_400_000_000);
        assert_eq!(preview.recv_window_ms, 2_500);
        assert_eq!(preview.api_key_env, "NTPRO_TEST_KEY");
        assert_eq!(preview.api_secret_env, "NTPRO_TEST_SECRET");
        assert_eq!(
            preview.orig_client_order_id.as_deref(),
            Some("ntpro-cancel-001")
        );
        assert_eq!(
            preview.output,
            Some(PathBuf::from("runs/v100/request-preview.json"))
        );
        assert!(preview.allow_testnet_order);
        assert!(preview.confirm_owner_approved_testnet_order);
        assert!(preview.confirm_tiny_notional);
        assert!(preview.confirm_cancel_after_submit);
    }

    #[test]
    fn parses_live_testnet_order_test_preflight_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "testnet-order-test-preflight",
            "--config",
            "configs/nodes/btc-ema-shadow.toml",
            "--timestamp-ms",
            "1718400000000",
            "--recv-window-ms",
            "2500",
            "--api-key-env",
            "NTPRO_TEST_KEY",
            "--api-secret-env",
            "NTPRO_TEST_SECRET",
            "--output",
            "runs/v100/order-test-preflight.json",
            "--allow-testnet-order",
            "--confirm-owner-approved-testnet-order",
            "--confirm-tiny-notional",
            "--confirm-cancel-after-submit",
        ])
        .expect("live testnet-order-test-preflight should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::TestnetOrderTestPreflight(preflight) = live.command else {
            panic!("expected testnet-order-test-preflight command");
        };

        assert_eq!(
            preflight.config,
            PathBuf::from("configs/nodes/btc-ema-shadow.toml")
        );
        assert_eq!(preflight.timestamp_ms, 1_718_400_000_000);
        assert_eq!(preflight.recv_window_ms, 2_500);
        assert_eq!(preflight.api_key_env, "NTPRO_TEST_KEY");
        assert_eq!(preflight.api_secret_env, "NTPRO_TEST_SECRET");
        assert_eq!(
            preflight.output,
            Some(PathBuf::from("runs/v100/order-test-preflight.json"))
        );
        assert!(preflight.allow_testnet_order);
        assert!(preflight.confirm_owner_approved_testnet_order);
        assert!(preflight.confirm_tiny_notional);
        assert!(preflight.confirm_cancel_after_submit);
    }

    #[test]
    fn parses_live_testnet_execution_artifact_contract_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "testnet-execution-artifact-contract",
            "--config",
            "configs/nodes/btc-ema-shadow.toml",
            "--timestamp-ms",
            "1718400000000",
            "--recv-window-ms",
            "2500",
            "--api-key-env",
            "NTPRO_TEST_KEY",
            "--api-secret-env",
            "NTPRO_TEST_SECRET",
            "--orig-client-order-id",
            "ntpro-cancel-001",
            "--output",
            "runs/v100/execution-artifact-contract.json",
            "--allow-testnet-order",
            "--confirm-owner-approved-testnet-order",
            "--confirm-tiny-notional",
            "--confirm-cancel-after-submit",
        ])
        .expect("live testnet-execution-artifact-contract should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::TestnetExecutionArtifactContract(contract) = live.command else {
            panic!("expected testnet-execution-artifact-contract command");
        };

        assert_eq!(
            contract.config,
            PathBuf::from("configs/nodes/btc-ema-shadow.toml")
        );
        assert_eq!(contract.timestamp_ms, 1_718_400_000_000);
        assert_eq!(contract.recv_window_ms, 2_500);
        assert_eq!(contract.api_key_env, "NTPRO_TEST_KEY");
        assert_eq!(contract.api_secret_env, "NTPRO_TEST_SECRET");
        assert_eq!(contract.orig_client_order_id, "ntpro-cancel-001");
        assert_eq!(
            contract.output,
            Some(PathBuf::from("runs/v100/execution-artifact-contract.json"))
        );
        assert!(contract.allow_testnet_order);
        assert!(contract.confirm_owner_approved_testnet_order);
        assert!(contract.confirm_tiny_notional);
        assert!(contract.confirm_cancel_after_submit);
    }

    #[test]
    fn parses_live_testnet_reconciliation_fixture_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "testnet-reconciliation-fixture",
            "--config",
            "configs/nodes/btc-ema-shadow.toml",
            "--scenario",
            "cancel-timeout",
            "--output",
            "runs/v100/reconciliation-fixture.json",
        ])
        .expect("live testnet-reconciliation-fixture should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::TestnetReconciliationFixture(fixture) = live.command else {
            panic!("expected testnet-reconciliation-fixture command");
        };

        assert_eq!(
            fixture.config,
            PathBuf::from("configs/nodes/btc-ema-shadow.toml")
        );
        assert_eq!(
            fixture.scenario,
            TestnetReconciliationScenario::CancelTimeout
        );
        assert_eq!(
            fixture.output,
            Some(PathBuf::from("runs/v100/reconciliation-fixture.json"))
        );
    }

    #[test]
    fn live_help_describes_live_init_smoke_boundary() {
        let validate_help = render_subcommand_help(&["live", "validate"]);
        let run_help = render_subcommand_help(&["live", "run"]);
        let gate_help = render_subcommand_help(&["live", "testnet-order-gate"]);
        let preflight_help = render_subcommand_help(&["live", "testnet-order-preflight"]);
        let request_preview_help =
            render_subcommand_help(&["live", "testnet-order-request-preview"]);
        let production_request_preview_help =
            render_subcommand_help(&["live", "production-live-alpha-order-request-preview"]);
        let production_manual_approval_lifecycle_help =
            render_subcommand_help(&["live", "production-live-alpha-manual-approval-lifecycle"]);
        let production_execution_dry_run_help =
            render_subcommand_help(&["live", "production-live-alpha-execution-dry-run"]);
        let production_kill_switch_runtime_gate_help =
            render_subcommand_help(&["live", "production-live-alpha-kill-switch-runtime-gate"]);
        let order_test_preflight_help =
            render_subcommand_help(&["live", "testnet-order-test-preflight"]);
        let artifact_contract_help =
            render_subcommand_help(&["live", "testnet-execution-artifact-contract"]);
        let reconciliation_fixture_help =
            render_subcommand_help(&["live", "testnet-reconciliation-fixture"]);

        assert!(validate_help.contains("live-init smoke config"));
        assert!(run_help.contains("LiveNode start/stop smoke path"));
        assert!(run_help.contains("without external venue access"));
        assert!(run_help.contains("real orders"));
        assert!(gate_help.contains("without opening network or submitting orders"));
        assert!(gate_help.contains("--allow-testnet-order"));
        assert!(gate_help.contains("--confirm-owner-approved-testnet-order"));
        assert!(preflight_help.contains("without network or orders"));
        assert!(preflight_help.contains("--input"));
        assert!(preflight_help.contains("--allow-testnet-order"));
        assert!(request_preview_help.contains("without network or orders"));
        assert!(request_preview_help.contains("--endpoint-path"));
        assert!(request_preview_help.contains("--timestamp-ms"));
        assert!(request_preview_help.contains("--api-secret-env"));
        assert!(production_request_preview_help.contains("no request execution"));
        assert!(production_request_preview_help.contains("--order-gate"));
        assert!(production_request_preview_help.contains("--manual-approval-lifecycle"));
        assert!(production_request_preview_help.contains("--confirm-memory-only-signature"));
        assert!(production_request_preview_help.contains("--confirm-no-network"));
        assert!(production_manual_approval_lifecycle_help.contains("one-time manual approval"));
        assert!(production_manual_approval_lifecycle_help.contains("--expires-at-unix-ms"));
        assert!(production_manual_approval_lifecycle_help.contains("--confirm-one-time-approval"));
        assert!(production_execution_dry_run_help.contains("dry-run execution adapter"));
        assert!(production_execution_dry_run_help.contains("--risk-preflight"));
        assert!(production_execution_dry_run_help.contains("--request-preview"));
        assert!(production_execution_dry_run_help.contains("--kill-switch-runtime-gate"));
        assert!(production_execution_dry_run_help.contains("--confirm-no-production-adapter"));
        assert!(production_kill_switch_runtime_gate_help.contains("kill-switch runtime gate"));
        assert!(production_kill_switch_runtime_gate_help.contains("--kill-switch-approval"));
        assert!(production_kill_switch_runtime_gate_help.contains("--request-preview"));
        assert!(production_kill_switch_runtime_gate_help.contains("--confirm-no-network"));
        assert!(order_test_preflight_help.contains("without network or orders"));
        assert!(order_test_preflight_help.contains("--timestamp-ms"));
        assert!(order_test_preflight_help.contains("--api-secret-env"));
        assert!(artifact_contract_help.contains("without network or orders"));
        assert!(artifact_contract_help.contains("--orig-client-order-id"));
        assert!(artifact_contract_help.contains("--api-secret-env"));
        assert!(reconciliation_fixture_help.contains("without network or orders"));
        assert!(reconciliation_fixture_help.contains("--scenario"));
        assert!(reconciliation_fixture_help.contains("--output"));
    }

    #[test]
    fn live_help_describes_production_readonly_and_dry_run_boundaries() {
        let live_help = render_subcommand_help(&["live"]);

        assert!(
            live_help.contains(
                "Local live, production read-only, dry-run proof, and owner-gated mutation-candidate commands"
            )
        );
        assert!(!live_help.contains("Local sandbox LiveNode smoke commands"));
        assert!(live_help.contains("production-public-read-probe"));
        assert!(live_help.contains("production-order-state-read-only-proof"));
        assert!(live_help.contains("production-live-alpha-dry-run-order-gate"));
        assert!(live_help.contains("production-live-alpha-order-request-preview"));
        assert!(live_help.contains("production-live-alpha-execution-dry-run"));
        assert!(live_help.contains("no production mutation"));

        for command in [
            "production-public-read-probe",
            "production-account-snapshot-contract",
            "production-order-state-read-only-proof",
            "production-live-alpha-dry-run-order-gate",
            "production-live-alpha-order-request-preview",
            "production-live-alpha-manual-approval-lifecycle",
            "production-live-alpha-execution-dry-run",
            "production-live-alpha-kill-switch-runtime-gate",
            "production-live-alpha-risk-preflight",
            "production-shadow-portfolio-runtime",
            "production-shadow-strategy-session",
            "production-shadow-preflight-session",
            "production-kill-switch-approval-artifact",
            "production-readonly-reconciliation",
        ] {
            let help = render_subcommand_help(&["live", command]);
            assert!(
                help.contains("no production mutation"),
                "{command} help should state the no-production-mutation boundary"
            );
        }
    }

    #[test]
    fn supervisor_help_lists_local_artifact_controls() {
        let mut command = NautilusCli::command();
        let supervisor = command
            .find_subcommand_mut("supervisor")
            .expect("supervisor command should exist");
        let help = supervisor.render_help().to_string();

        for command in [
            "register",
            "list",
            "start",
            "stop",
            "pause",
            "resume",
            "reconnect-data",
            "reconnect-execution",
            "status",
            "connections",
            "execution",
            "risk",
            "logs",
            "metrics",
            "shadow-runtime",
        ] {
            assert!(help.contains(command), "{command} should be listed");
        }
    }

    #[test]
    fn supervisor_help_describes_local_boundary() {
        let help = render_subcommand_help(&["supervisor", "start"]);

        assert!(help.contains("registered local sandbox-only ntpro-node process"));
        assert!(help.contains("sandbox-only"));
        assert!(help.contains("--registry"));
        assert!(help.contains("--node-id"));
        assert!(help.contains("--ntpro-node-bin"));
    }

    #[test]
    fn parses_supervisor_register_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "supervisor",
            "register",
            "--registry",
            "runs/supervisor/registry.json",
            "--node-id",
            "sandbox-a",
            "--config",
            "config/live.toml",
            "--artifact-root",
            "runs/supervisor/nodes/sandbox-a",
        ])
        .expect("supervisor register should parse");

        let Commands::Supervisor(supervisor) = parsed.command else {
            panic!("expected supervisor command");
        };
        let SupervisorCommand::Register(register) = supervisor.command else {
            panic!("expected register command");
        };

        assert_eq!(
            register.registry.registry,
            PathBuf::from("runs/supervisor/registry.json")
        );
        assert_eq!(register.node_id, "sandbox-a");
        assert_eq!(register.config, PathBuf::from("config/live.toml"));
        assert_eq!(
            register.artifact_root,
            Some(PathBuf::from("runs/supervisor/nodes/sandbox-a"))
        );
    }

    #[test]
    fn parses_supervisor_start_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "supervisor",
            "start",
            "--registry",
            "runs/supervisor/registry.json",
            "--node-id",
            "sandbox-a",
            "--ntpro-node-bin",
            "target/debug/ntpro-node",
            "--startup-timeout-ms",
            "7500",
        ])
        .expect("supervisor start should parse");

        let Commands::Supervisor(supervisor) = parsed.command else {
            panic!("expected supervisor command");
        };
        let SupervisorCommand::Start(start) = supervisor.command else {
            panic!("expected start command");
        };

        assert_eq!(
            start.registry.registry,
            PathBuf::from("runs/supervisor/registry.json")
        );
        assert_eq!(start.node_id, "sandbox-a");
        assert_eq!(
            start.ntpro_node_bin,
            PathBuf::from("target/debug/ntpro-node")
        );
        assert_eq!(start.startup_timeout_ms, 7_500);
        assert_eq!(start.node_max_runtime_ms, 3_600_000);
        assert_eq!(start.node_heartbeat_interval_ms, 1_000);
        assert_eq!(start.node_parent_pid, None);
        assert_eq!(start.node_shutdown_timeout_ms, 5_000);
    }

    #[test]
    fn parses_supervisor_query_options() {
        for command in [
            "pause",
            "resume",
            "reconnect-data",
            "reconnect-execution",
            "status",
            "connections",
            "execution",
            "risk",
            "logs",
            "metrics",
            "shadow-runtime",
        ] {
            let parsed = NautilusCli::try_parse_from([
                "nautilus",
                "supervisor",
                command,
                "--registry",
                "runs/supervisor/registry.json",
                "--node-id",
                "sandbox-a",
            ])
            .unwrap_or_else(|_| panic!("supervisor {command} should parse"));

            let Commands::Supervisor(supervisor) = parsed.command else {
                panic!("expected supervisor command");
            };
            match supervisor.command {
                SupervisorCommand::Status(node)
                | SupervisorCommand::Pause(node)
                | SupervisorCommand::Resume(node)
                | SupervisorCommand::ReconnectData(node)
                | SupervisorCommand::ReconnectExecution(node)
                | SupervisorCommand::Connections(node)
                | SupervisorCommand::Execution(node)
                | SupervisorCommand::Risk(node)
                | SupervisorCommand::Logs(node)
                | SupervisorCommand::Metrics(node)
                | SupervisorCommand::ShadowRuntime(node) => {
                    assert_eq!(
                        node.registry.registry,
                        PathBuf::from("runs/supervisor/registry.json")
                    );
                    assert_eq!(node.node_id, "sandbox-a");
                }
                _ => panic!("expected query command"),
            }
        }
    }

    #[test]
    fn dashboard_help_lists_local_http_server() {
        let mut command = NautilusCli::command();
        let dashboard = command
            .find_subcommand_mut("dashboard")
            .expect("dashboard command should exist");
        let help = dashboard.render_help().to_string();

        assert!(help.contains("serve"));
        assert!(help.contains("Local dashboard HTTP server"));
    }

    #[test]
    fn dashboard_serve_help_describes_local_boundary() {
        let help = render_subcommand_help(&["dashboard", "serve"]);

        assert!(help.contains("static dashboard shell"));
        assert!(help.contains("local JSON API"));
        assert!(help.contains("--registry"));
        assert!(help.contains("--workflow-root"));
        assert!(help.contains("--bind"));
    }

    #[test]
    fn parses_dashboard_serve_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "dashboard",
            "serve",
            "--registry",
            "runs/supervisor/registry.json",
            "--workflow-root",
            "runs/workflows",
            "--bind",
            "127.0.0.1:5174",
        ])
        .expect("dashboard serve should parse");

        let Commands::Dashboard(dashboard) = parsed.command else {
            panic!("expected dashboard command");
        };
        let DashboardCommand::Serve(serve) = dashboard.command;

        assert_eq!(
            serve.registry,
            PathBuf::from("runs/supervisor/registry.json")
        );
        assert_eq!(serve.workflow_root, Some(PathBuf::from("runs/workflows")));
        assert_eq!(serve.bind.to_string(), "127.0.0.1:5174");
        assert!(serve.ntpro_node_bin.is_none());
    }

    #[test]
    fn dashboard_serve_defaults_to_loopback() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "dashboard",
            "serve",
            "--registry",
            "runs/supervisor/registry.json",
        ])
        .expect("dashboard serve should parse with default bind");

        let Commands::Dashboard(dashboard) = parsed.command else {
            panic!("expected dashboard command");
        };
        let DashboardCommand::Serve(serve) = dashboard.command;

        assert_eq!(serve.bind.to_string(), "127.0.0.1:5173");
    }

    #[test]
    fn parses_dashboard_serve_ntpro_node_bin() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "dashboard",
            "serve",
            "--registry",
            "runs/supervisor/registry.json",
            "--ntpro-node-bin",
            "target/debug/ntpro-node",
        ])
        .expect("dashboard serve should parse with ntpro-node binary");

        let Commands::Dashboard(dashboard) = parsed.command else {
            panic!("expected dashboard command");
        };
        let DashboardCommand::Serve(serve) = dashboard.command;

        assert_eq!(
            serve.ntpro_node_bin,
            Some(PathBuf::from("target/debug/ntpro-node"))
        );
    }

    #[test]
    fn workflow_help_lists_local_artifact_run() {
        let mut command = NautilusCli::command();
        let workflow = command
            .find_subcommand_mut("workflow")
            .expect("workflow command should exist");
        let help = workflow.render_help().to_string();

        assert!(help.contains("run"));
        assert!(help.contains("Local sandbox/testnet workflow artifact commands"));
    }

    #[test]
    fn workflow_run_help_describes_sandbox_boundary() {
        let help = render_subcommand_help(&["workflow", "run"]);

        assert!(help.contains("local Binance sandbox/testnet workflow"));
        assert!(help.contains("dashboard-readable artifacts"));
        assert!(help.contains("Default dry-run opens no sockets"));
        assert!(help.contains("Fail-closed"));
        assert!(help.contains("read-only networking"));
        assert!(help.contains("NTPRO_ALLOW_TESTNET_NETWORK=1"));
        assert!(help.contains("read-only config"));
        assert!(help.contains("--workflow"));
        assert!(help.contains("--mode"));
        assert!(help.contains("--config"));
        assert!(help.contains("--allow-testnet-network"));
        assert!(help.contains("--run-id"));
        assert!(help.contains("--output"));
    }

    #[test]
    fn parses_workflow_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "workflow",
            "run",
            "--workflow",
            "binance-sandbox",
            "--run-id",
            "v05-smoke",
            "--output",
            "runs/workflows/v05-smoke",
        ])
        .expect("workflow run should parse");

        let Commands::Workflow(workflow) = parsed.command else {
            panic!("expected workflow command");
        };
        let WorkflowCommand::Run(run) = workflow.command;

        assert_eq!(run.workflow, WorkflowKind::BinanceSandbox);
        assert_eq!(run.mode, WorkflowRunMode::DryRun);
        assert_eq!(run.config, None);
        assert!(!run.allow_testnet_network);
        assert_eq!(run.run_id.as_deref(), Some("v05-smoke"));
        assert_eq!(run.output, Some(PathBuf::from("runs/workflows/v05-smoke")));
    }

    #[test]
    fn parses_workflow_binance_testnet_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "workflow",
            "run",
            "--workflow",
            "binance-testnet",
            "--mode",
            "connectivity-probe",
            "--config",
            "examples/rust/binance/testnet_dry_run.toml",
            "--allow-testnet-network",
            "--run-id",
            "v06-smoke",
            "--output",
            "runs/workflows/v06-smoke",
        ])
        .expect("workflow run should parse");

        let Commands::Workflow(workflow) = parsed.command else {
            panic!("expected workflow command");
        };
        let WorkflowCommand::Run(run) = workflow.command;

        assert_eq!(run.workflow, WorkflowKind::BinanceTestnet);
        assert_eq!(run.mode, WorkflowRunMode::ConnectivityProbe);
        assert_eq!(
            run.config,
            Some(PathBuf::from("examples/rust/binance/testnet_dry_run.toml"))
        );
        assert!(run.allow_testnet_network);
        assert_eq!(run.run_id.as_deref(), Some("v06-smoke"));
        assert_eq!(run.output, Some(PathBuf::from("runs/workflows/v06-smoke")));
    }

    #[test]
    fn data_help_lists_inspect_validate_and_load() {
        let mut command = NautilusCli::command();
        let data = command
            .find_subcommand_mut("data")
            .expect("data command should exist");
        let help = data.render_help().to_string();

        assert!(help.contains("inspect"));
        assert!(help.contains("validate"));
        assert!(help.contains("load"));
    }

    #[test]
    fn parses_data_inspect_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "data",
            "inspect",
            "--config",
            "config/data.toml",
            "--output",
            "runs/catalog-audit",
        ])
        .expect("data inspect should parse");

        let Commands::Data(data) = parsed.command else {
            panic!("expected data command");
        };
        let DataCommand::Inspect(inspect) = data.command else {
            panic!("expected inspect command");
        };

        assert_eq!(inspect.config, PathBuf::from("config/data.toml"));
        assert_eq!(inspect.output, Some(PathBuf::from("runs/catalog-audit")));
    }

    #[test]
    fn parses_data_validate_config_path() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "data",
            "validate",
            "--config",
            "config/data.toml",
        ])
        .expect("data validate should parse");

        let Commands::Data(data) = parsed.command else {
            panic!("expected data command");
        };
        let DataCommand::Validate(validate) = data.command else {
            panic!("expected validate command");
        };

        assert_eq!(validate.config, PathBuf::from("config/data.toml"));
    }

    #[test]
    fn parses_data_load_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "data",
            "load",
            "--config",
            "config/data.toml",
            "--run-id",
            "load-quotes",
            "--output",
            "runs/load-quotes",
        ])
        .expect("data load should parse");

        let Commands::Data(data) = parsed.command else {
            panic!("expected data command");
        };
        let DataCommand::Load(load) = data.command else {
            panic!("expected load command");
        };

        assert_eq!(load.config, PathBuf::from("config/data.toml"));
        assert_eq!(load.run_id.as_deref(), Some("load-quotes"));
        assert_eq!(load.output, Some(PathBuf::from("runs/load-quotes")));
    }

    #[test]
    fn data_help_describes_supported_and_deferred_boundaries() {
        let inspect_help = render_subcommand_help(&["data", "inspect"]);
        let validate_help = render_subcommand_help(&["data", "validate"]);
        let load_help = render_subcommand_help(&["data", "load"]);

        assert!(inspect_help.contains("Inspects local data/catalog metadata"));
        assert!(validate_help.contains("Validates local data/catalog readability"));
        assert!(load_help.contains("Loads a local QuoteTick fixture"));
    }

    #[test]
    fn config_help_lists_validate() {
        let mut command = NautilusCli::command();
        let config = command
            .find_subcommand_mut("config")
            .expect("config command should exist");
        let help = config.render_help().to_string();

        assert!(help.contains("validate"));
    }

    #[test]
    fn config_validate_help_lists_kind_config_and_output() {
        let mut command = NautilusCli::command();
        let config = command
            .find_subcommand_mut("config")
            .expect("config command should exist");
        let validate = config
            .find_subcommand_mut("validate")
            .expect("config validate command should exist");
        let help = validate.render_help().to_string();

        assert!(help.contains("--kind"));
        assert!(help.contains("--config"));
        assert!(help.contains("--output"));
    }

    #[test]
    fn config_validate_help_describes_validation_boundary() {
        let help = render_subcommand_help(&["config", "validate"]);

        assert!(help.contains("Validates a Rust workflow config"));
        assert!(help.contains("--kind"));
        assert!(help.contains("--config"));
    }

    #[test]
    fn parses_config_validate_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "config",
            "validate",
            "--kind",
            "backtest",
            "--config",
            "config/backtest.toml",
            "--output",
            "runs/config-validate",
        ])
        .expect("config validate should parse");

        let Commands::Config(config) = parsed.command else {
            panic!("expected config command");
        };
        let ConfigCommand::Validate(validate) = config.command;

        assert_eq!(validate.kind, ConfigKind::Backtest);
        assert_eq!(validate.config, PathBuf::from("config/backtest.toml"));
        assert_eq!(validate.output, Some(PathBuf::from("runs/config-validate")));
    }

    #[test]
    fn database_help_lists_init_and_drop() {
        let mut command = NautilusCli::command();
        let database = command
            .find_subcommand_mut("database")
            .expect("database command should exist");
        let help = database.render_help().to_string();

        assert!(help.contains("init"));
        assert!(help.contains("drop"));
    }

    #[test]
    fn parses_database_init_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "database",
            "init",
            "--host",
            "localhost",
            "--port",
            "5432",
            "--username",
            "ntpro",
            "--database",
            "ntpro",
            "--password",
            "secret",
            "--schema",
            "schema/postgres",
        ])
        .expect("database init should parse");

        let Commands::Database(database) = parsed.command else {
            panic!("expected database command");
        };
        let DatabaseCommand::Init(config) = database.command else {
            panic!("expected init command");
        };

        assert_eq!(config.host.as_deref(), Some("localhost"));
        assert_eq!(config.port, Some(5432));
        assert_eq!(config.username.as_deref(), Some("ntpro"));
        assert_eq!(config.database.as_deref(), Some("ntpro"));
        assert_eq!(config.password.as_deref(), Some("secret"));
        assert_eq!(config.schema.as_deref(), Some("schema/postgres"));
    }

    #[test]
    fn parses_database_drop_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "database",
            "drop",
            "--host",
            "localhost",
            "--port",
            "5432",
            "--username",
            "ntpro",
            "--database",
            "ntpro",
        ])
        .expect("database drop should parse");

        let Commands::Database(database) = parsed.command else {
            panic!("expected database command");
        };
        let DatabaseCommand::Drop(config) = database.command else {
            panic!("expected drop command");
        };

        assert_eq!(config.host.as_deref(), Some("localhost"));
        assert_eq!(config.port, Some(5432));
        assert_eq!(config.username.as_deref(), Some("ntpro"));
        assert_eq!(config.database.as_deref(), Some("ntpro"));
        assert_eq!(config.password, None);
        assert_eq!(config.schema, None);
    }
}
