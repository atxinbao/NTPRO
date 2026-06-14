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
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use nautilus_binance::{
    common::{
        enums::{BinanceEnvironment, BinanceProductType},
        urls::{get_http_base_url, get_ws_base_url},
    },
    mock_lifecycle::{BinanceMockOrderLifecycleSummary, load_v04_binance_mock_order_lifecycle},
    replay::{BinanceReplaySummary, load_v04_binance_spot_bar_replay},
};
use nautilus_risk::v04_rejection::{
    V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID, V04_BINANCE_RISK_REJECTION_FIXTURE_REASON,
    V04_BINANCE_RISK_REJECTION_INSTRUMENT_ID, V04_BINANCE_RISK_REJECTION_LIFECYCLE_ID,
    V04_BINANCE_RISK_REJECTION_REASON, V04_BINANCE_RISK_REJECTION_SMOKE_ID,
    V04BinanceRiskRejectionSummary,
};
use nautilus_trading::strategy::v04_smoke::{
    V04EmaSmokeSummary, V04RsiSmokeSummary, v04_ema_smoke_from_csv, v04_rsi_smoke_from_csv,
};
use serde::{Deserialize, Serialize};

use crate::{
    artifacts::{atomic_write_json, atomic_write_text},
    opt::{WorkflowCommand, WorkflowKind, WorkflowOpt, WorkflowRunMode, WorkflowRunOpt},
};

const WORKFLOW_ID: &str = "v05-binance-sandbox-local-workflow";
const MANIFEST_SCHEMA_VERSION: &str = "ntpro.workflow_manifest.v1";
const SUMMARY_SCHEMA_VERSION: &str = "ntpro.workflow_summary.v1";
const BOUNDARY_SCHEMA_VERSION: &str = "ntpro.workflow_boundary.v1";
const EVENT_SCHEMA_VERSION: &str = "ntpro.workflow_event.v1";
const TESTNET_CONFIG_SCHEMA_VERSION: &str = "ntpro.v06_binance_testnet_config.v1";
const TESTNET_CREDENTIAL_POLICY_SCHEMA_VERSION: &str =
    "ntpro.v06_binance_testnet_credential_policy.v1";
const TESTNET_CONNECTIVITY_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v06_binance_testnet_connectivity_probe.v1";
const TESTNET_ORDER_LIFECYCLE_SCHEMA_VERSION: &str = "ntpro.v06_binance_testnet_order_lifecycle.v1";
const TESTNET_RECONCILIATION_SCHEMA_VERSION: &str = "ntpro.v06_binance_testnet_reconciliation.v1";
const DEFAULT_RUN_ID: &str = "v05-binance-sandbox-local";
const BINANCE_SPOT_BARS_CSV: &str =
    include_str!("../../adapters/binance/test_data/v04/binance_spot_bars.csv");

/// Runs a local workflow CLI command.
///
/// # Errors
///
/// Returns an error if the workflow kind is unsupported, checked-in fixtures
/// are invalid, or any artifact cannot be written.
pub(crate) fn run_workflow_command(opt: WorkflowOpt) -> anyhow::Result<()> {
    match opt.command {
        WorkflowCommand::Run(run) => {
            let result = run_workflow(run)?;
            println!(
                "workflow.run status=ok workflow={} run_id={} output={} manifest={} summary={} events={} artifact_count={} requested_mode={} network_permission_requested={} network_attempted={} external_venue_connection={} real_funds={} production_trading={} real_orders_submitted={} testnet_connection={} runtime_status={}",
                result.workflow,
                result.run_id,
                result.output_dir.display(),
                result.manifest_path.display(),
                result.summary_path.display(),
                result.events_path.display(),
                result.artifact_paths.len(),
                result.requested_mode,
                result.network_permission_requested,
                result.network_attempted,
                result.external_venue_connection,
                result.real_funds,
                result.production_trading,
                result.real_orders_submitted,
                result.testnet_connection,
                result.runtime_status,
            );
        }
    }
    Ok(())
}

fn run_workflow(opt: WorkflowRunOpt) -> anyhow::Result<WorkflowRunResult> {
    match opt.workflow {
        WorkflowKind::BinanceSandbox => run_binance_sandbox_workflow(opt),
        WorkflowKind::BinanceTestnet => run_binance_testnet_workflow(opt),
    }
}

fn run_binance_sandbox_workflow(opt: WorkflowRunOpt) -> anyhow::Result<WorkflowRunResult> {
    if opt.mode != WorkflowRunMode::DryRun {
        anyhow::bail!("binance-sandbox workflow only supports --mode dry-run");
    }
    if opt.config.is_some() {
        anyhow::bail!("binance-sandbox workflow does not accept --config");
    }
    if opt.allow_testnet_network {
        anyhow::bail!("binance-sandbox workflow does not accept --allow-testnet-network");
    }

    let run_id = opt.run_id.unwrap_or_else(|| DEFAULT_RUN_ID.to_string());
    validate_run_id(&run_id)?;

    let output_dir = opt
        .output
        .unwrap_or_else(|| PathBuf::from("runs/workflows").join(&run_id));
    let artifact_paths = WorkflowArtifactPaths::new(&output_dir);

    let market_replay = load_v04_binance_spot_bar_replay()?.summary();
    let ema_strategy = v04_ema_smoke_from_csv(BINANCE_SPOT_BARS_CSV)?;
    let rsi_strategy = v04_rsi_smoke_from_csv(BINANCE_SPOT_BARS_CSV)?;
    let order_lifecycle = load_v04_binance_mock_order_lifecycle()?.summary();
    let risk_rejection = v04_risk_rejection_summary();
    let boundary = WorkflowBoundary::binance_sandbox();
    let summary = WorkflowSummary::new(
        &run_id,
        &market_replay,
        &ema_strategy,
        &rsi_strategy,
        &order_lifecycle,
        &risk_rejection,
        &boundary,
    );
    let events = WorkflowEvents::from_summary(&summary);
    let manifest = WorkflowManifest::new(&run_id, &artifact_paths, &summary, &events);

    let mut written = Vec::new();
    write_json_artifact(&artifact_paths.market_replay, &market_replay, &mut written)?;
    write_json_artifact(&artifact_paths.ema_strategy, &ema_strategy, &mut written)?;
    write_json_artifact(&artifact_paths.rsi_strategy, &rsi_strategy, &mut written)?;
    write_json_artifact(
        &artifact_paths.order_lifecycle,
        &order_lifecycle,
        &mut written,
    )?;
    write_json_artifact(
        &artifact_paths.risk_rejection,
        &risk_rejection,
        &mut written,
    )?;
    write_json_artifact(&artifact_paths.boundary, &boundary, &mut written)?;
    write_json_artifact(&artifact_paths.summary, &summary, &mut written)?;
    write_events_artifact(&artifact_paths.events, &events, &mut written)?;
    write_json_artifact(&artifact_paths.manifest, &manifest, &mut written)?;

    Ok(WorkflowRunResult {
        workflow: "binance-sandbox".to_string(),
        run_id,
        runtime_status: "completed".to_string(),
        output_dir,
        manifest_path: artifact_paths.manifest,
        summary_path: artifact_paths.summary,
        events_path: artifact_paths.events,
        artifact_paths: written,
        external_venue_connection: false,
        real_funds: false,
        production_trading: false,
        real_orders_submitted: false,
        testnet_connection: false,
        requested_mode: "dry-run".to_string(),
        network_permission_requested: false,
        network_attempted: false,
    })
}

fn run_binance_testnet_workflow(opt: WorkflowRunOpt) -> anyhow::Result<WorkflowRunResult> {
    let config_path = opt
        .config
        .as_ref()
        .context("binance-testnet workflow requires --config")?;
    let config = load_testnet_workflow_config(config_path)?;
    config.validate()?;
    let run_id = resolve_testnet_run_id(opt.run_id.as_deref(), &config)?;

    let output_dir = opt
        .output
        .unwrap_or_else(|| PathBuf::from("runs/workflows").join(&run_id));
    let artifact_paths = WorkflowArtifactPaths::new(&output_dir);
    let testnet_paths = TestnetArtifactPaths::new(&output_dir);

    let credential_policy = TestnetCredentialPolicy::from_config(&config);
    let connectivity_probe =
        TestnetConnectivityProbe::from_config(&config, opt.mode, opt.allow_testnet_network);
    let order_lifecycle = TestnetOrderLifecycle::from_config(&run_id, &config);
    let reconciliation = TestnetReconciliation::from_order_lifecycle(&run_id, &order_lifecycle);
    let boundary =
        WorkflowBoundary::binance_testnet_dry_run(&credential_policy, &connectivity_probe);
    let summary = WorkflowSummary::new_binance_testnet(
        &run_id,
        &config,
        &credential_policy,
        &connectivity_probe,
        &order_lifecycle,
        &reconciliation,
        &boundary,
    );
    let events = WorkflowEvents::from_artifacts(
        &summary,
        [
            ("workflow.testnet_config.ready", "testnet/config.json"),
            (
                "workflow.credential_policy.ready",
                "testnet/credential_policy.json",
            ),
            (
                "workflow.connectivity_probe.ready",
                "testnet/connectivity_probe.json",
            ),
            (
                "workflow.testnet_order_lifecycle.ready",
                "orders/testnet_dry_run_lifecycle.json",
            ),
            (
                "workflow.testnet_reconciliation.ready",
                "orders/reconciliation.json",
            ),
            ("workflow.summary.ready", "summary.json"),
            ("workflow.events.ready", "events.jsonl"),
        ],
    );
    let manifest = WorkflowManifest::new_with_artifacts(
        &run_id,
        &artifact_paths,
        &summary,
        vec![
            WorkflowManifestArtifact::new(
                relative_artifact_path(&testnet_paths.config, &artifact_paths.output_dir),
                TESTNET_CONFIG_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(
                    &testnet_paths.credential_policy,
                    &artifact_paths.output_dir,
                ),
                TESTNET_CREDENTIAL_POLICY_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(
                    &testnet_paths.connectivity_probe,
                    &artifact_paths.output_dir,
                ),
                TESTNET_CONNECTIVITY_PROBE_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&testnet_paths.order_lifecycle, &artifact_paths.output_dir),
                TESTNET_ORDER_LIFECYCLE_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&testnet_paths.reconciliation, &artifact_paths.output_dir),
                TESTNET_RECONCILIATION_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&artifact_paths.boundary, &artifact_paths.output_dir),
                BOUNDARY_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&artifact_paths.summary, &artifact_paths.output_dir),
                SUMMARY_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&artifact_paths.events, &artifact_paths.output_dir),
                EVENT_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&artifact_paths.manifest, &artifact_paths.output_dir),
                MANIFEST_SCHEMA_VERSION,
            ),
        ],
    );

    let mut written = Vec::new();
    write_json_artifact(
        &testnet_paths.config,
        &config.to_artifact(&run_id),
        &mut written,
    )?;
    write_json_artifact(
        &testnet_paths.credential_policy,
        &credential_policy,
        &mut written,
    )?;
    write_json_artifact(
        &testnet_paths.connectivity_probe,
        &connectivity_probe,
        &mut written,
    )?;
    write_json_artifact(
        &testnet_paths.order_lifecycle,
        &order_lifecycle,
        &mut written,
    )?;
    write_json_artifact(&testnet_paths.reconciliation, &reconciliation, &mut written)?;
    write_json_artifact(&artifact_paths.boundary, &boundary, &mut written)?;
    write_json_artifact(&artifact_paths.summary, &summary, &mut written)?;
    write_events_artifact(&artifact_paths.events, &events, &mut written)?;
    write_json_artifact(&artifact_paths.manifest, &manifest, &mut written)?;

    Ok(WorkflowRunResult {
        workflow: "binance-testnet".to_string(),
        run_id,
        runtime_status: summary.runtime_status.clone(),
        output_dir,
        manifest_path: artifact_paths.manifest,
        summary_path: artifact_paths.summary,
        events_path: artifact_paths.events,
        artifact_paths: written,
        external_venue_connection: boundary.external_venue_connection,
        real_funds: boundary.real_funds,
        production_trading: boundary.production_trading,
        real_orders_submitted: boundary.real_orders_submitted,
        testnet_connection: boundary.testnet_connection,
        requested_mode: connectivity_probe.requested_mode,
        network_permission_requested: connectivity_probe.network_permission_requested,
        network_attempted: connectivity_probe.network_attempted,
    })
}

fn validate_run_id(run_id: &str) -> anyhow::Result<()> {
    if run_id.trim().is_empty() {
        anyhow::bail!("workflow run_id must not be empty");
    }
    if !run_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        anyhow::bail!("workflow run_id may only contain ASCII letters, digits, '-', '_', and '.'");
    }
    Ok(())
}

fn resolve_testnet_run_id(
    cli_run_id: Option<&str>,
    config: &TestnetWorkflowConfig,
) -> anyhow::Result<String> {
    let run_id = cli_run_id.unwrap_or(&config.run.id).to_string();
    validate_run_id(&run_id)?;
    Ok(run_id)
}

fn requested_mode_label(mode: WorkflowRunMode) -> &'static str {
    match mode {
        WorkflowRunMode::DryRun => "dry-run",
        WorkflowRunMode::ConnectivityProbe => "connectivity-probe",
    }
}

fn runtime_status_for_testnet_mode(mode: WorkflowRunMode) -> &'static str {
    match mode {
        WorkflowRunMode::DryRun => "dry_run_completed",
        WorkflowRunMode::ConnectivityProbe => "offline_probe_validated",
    }
}

fn load_testnet_workflow_config(path: &Path) -> anyhow::Result<TestnetWorkflowConfig> {
    let raw = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read Binance testnet workflow config '{}'",
            path.display()
        )
    })?;
    let mut config: TestnetWorkflowConfig = toml::from_str(&raw).with_context(|| {
        format!(
            "failed to parse Binance testnet workflow config '{}'",
            path.display()
        )
    })?;
    config.source_path = Some(path.display().to_string());
    Ok(config)
}

fn write_json_artifact<T>(path: &Path, value: &T, written: &mut Vec<PathBuf>) -> anyhow::Result<()>
where
    T: Serialize,
{
    atomic_write_json(path, value)?;
    written.push(path.to_path_buf());
    Ok(())
}

fn write_events_artifact(
    path: &Path,
    events: &WorkflowEvents,
    written: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    let mut body = String::new();
    for event in &events.events {
        body.push_str(&serde_json::to_string(event)?);
        body.push('\n');
    }
    atomic_write_text(path, &body)?;
    written.push(path.to_path_buf());
    Ok(())
}

fn v04_risk_rejection_summary() -> V04BinanceRiskRejectionSummary {
    V04BinanceRiskRejectionSummary {
        smoke_id: V04_BINANCE_RISK_REJECTION_SMOKE_ID.to_string(),
        lifecycle_id: V04_BINANCE_RISK_REJECTION_LIFECYCLE_ID.to_string(),
        instrument_id: V04_BINANCE_RISK_REJECTION_INSTRUMENT_ID.to_string(),
        client_order_id: V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID.to_string(),
        fixture_reason: V04_BINANCE_RISK_REJECTION_FIXTURE_REASON.to_string(),
        risk_reason: V04_BINANCE_RISK_REJECTION_REASON.to_string(),
        order_status: "denied".to_string(),
        forwarded_to_execution: false,
        external_adapter: false,
        real_exchange_connection: false,
        real_orders_submitted: false,
        checksum: "60b0dc50f47caea8".to_string(),
    }
}

#[derive(Debug, Clone)]
struct WorkflowArtifactPaths {
    output_dir: PathBuf,
    market_replay: PathBuf,
    ema_strategy: PathBuf,
    rsi_strategy: PathBuf,
    order_lifecycle: PathBuf,
    risk_rejection: PathBuf,
    boundary: PathBuf,
    summary: PathBuf,
    events: PathBuf,
    manifest: PathBuf,
}

impl WorkflowArtifactPaths {
    fn new(output_dir: &Path) -> Self {
        Self {
            output_dir: output_dir.to_path_buf(),
            market_replay: output_dir.join("market/replay.json"),
            ema_strategy: output_dir.join("strategies/ema.json"),
            rsi_strategy: output_dir.join("strategies/rsi.json"),
            order_lifecycle: output_dir.join("orders/mock_lifecycle.json"),
            risk_rejection: output_dir.join("risk/rejection.json"),
            boundary: output_dir.join("boundary.json"),
            summary: output_dir.join("summary.json"),
            events: output_dir.join("events.jsonl"),
            manifest: output_dir.join("manifest.json"),
        }
    }
}

#[derive(Debug, Clone)]
struct TestnetArtifactPaths {
    config: PathBuf,
    credential_policy: PathBuf,
    connectivity_probe: PathBuf,
    order_lifecycle: PathBuf,
    reconciliation: PathBuf,
}

impl TestnetArtifactPaths {
    fn new(output_dir: &Path) -> Self {
        Self {
            config: output_dir.join("testnet/config.json"),
            credential_policy: output_dir.join("testnet/credential_policy.json"),
            connectivity_probe: output_dir.join("testnet/connectivity_probe.json"),
            order_lifecycle: output_dir.join("orders/testnet_dry_run_lifecycle.json"),
            reconciliation: output_dir.join("orders/reconciliation.json"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TestnetWorkflowConfig {
    #[serde(skip)]
    source_path: Option<String>,
    run: TestnetRunConfig,
    venue: TestnetVenueConfig,
    credentials: TestnetCredentialConfig,
    connectivity: TestnetConnectivityConfig,
    execution: TestnetExecutionConfig,
}

impl TestnetWorkflowConfig {
    fn validate(&self) -> anyhow::Result<()> {
        ensure_field("run.id", &self.run.id)?;
        ensure_equals("run.mode", &self.run.mode, "dry-run")?;
        ensure_equals("venue.name", &self.venue.name, "BINANCE")?;
        ensure_equals("venue.environment", &self.venue.environment, "testnet")?;
        ensure_equals("venue.product", &self.venue.product, "spot")?;
        ensure_field("credentials.api_key_env", &self.credentials.api_key_env)?;
        ensure_field(
            "credentials.api_secret_env",
            &self.credentials.api_secret_env,
        )?;
        if self.credentials.values_in_file {
            anyhow::bail!("credentials.values_in_file must be false for the V06 testnet workflow");
        }
        if self.connectivity.network_attempted {
            anyhow::bail!("connectivity.network_attempted must be false for checked-in V06 config");
        }
        ensure_equals("connectivity.mode", &self.connectivity.mode, "dry-run")?;
        ensure_field(
            "connectivity.http_base_url",
            &self.connectivity.http_base_url,
        )?;
        ensure_field("connectivity.ws_base_url", &self.connectivity.ws_base_url)?;
        ensure_equals(
            "connectivity.http_base_url",
            &self.connectivity.http_base_url,
            get_http_base_url(BinanceProductType::Spot, BinanceEnvironment::Testnet),
        )?;
        ensure_equals(
            "connectivity.ws_base_url",
            &self.connectivity.ws_base_url,
            get_ws_base_url(BinanceProductType::Spot, BinanceEnvironment::Testnet),
        )?;
        ensure_equals(
            "execution.order_submission",
            &self.execution.order_submission,
            "disabled",
        )?;
        ensure_equals(
            "execution.reconciliation",
            &self.execution.reconciliation,
            "artifact-only",
        )?;
        if self.execution.real_orders_submitted {
            anyhow::bail!("execution.real_orders_submitted must be false");
        }
        Ok(())
    }

    fn to_artifact(&self, effective_run_id: &str) -> TestnetConfigArtifact {
        TestnetConfigArtifact {
            schema_version: TESTNET_CONFIG_SCHEMA_VERSION.to_string(),
            source_path: self
                .source_path
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            run_id: effective_run_id.to_string(),
            config_declared_run_id: self.run.id.clone(),
            mode: self.run.mode.clone(),
            venue: self.venue.name.clone(),
            product: self.venue.product.clone(),
            environment: self.venue.environment.clone(),
            http_base_url: self.connectivity.http_base_url.clone(),
            ws_base_url: self.connectivity.ws_base_url.clone(),
            order_submission: self.execution.order_submission.clone(),
            reconciliation: self.execution.reconciliation.clone(),
            real_orders_submitted: self.execution.real_orders_submitted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TestnetRunConfig {
    id: String,
    mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TestnetVenueConfig {
    name: String,
    product: String,
    environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TestnetCredentialConfig {
    api_key_env: String,
    api_secret_env: String,
    values_in_file: bool,
    required_for_network: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TestnetConnectivityConfig {
    mode: String,
    http_base_url: String,
    ws_base_url: String,
    network_attempted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TestnetExecutionConfig {
    order_submission: String,
    reconciliation: String,
    real_orders_submitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TestnetConfigArtifact {
    schema_version: String,
    source_path: String,
    run_id: String,
    config_declared_run_id: String,
    mode: String,
    venue: String,
    product: String,
    environment: String,
    http_base_url: String,
    ws_base_url: String,
    order_submission: String,
    reconciliation: String,
    real_orders_submitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TestnetCredentialPolicy {
    schema_version: String,
    policy: String,
    api_key_env: String,
    api_secret_env: String,
    values_in_file: bool,
    values_recorded: bool,
    secrets_redacted: bool,
    required_for_network: bool,
    api_key_present: bool,
    api_secret_present: bool,
}

impl TestnetCredentialPolicy {
    fn from_config(config: &TestnetWorkflowConfig) -> Self {
        Self {
            schema_version: TESTNET_CREDENTIAL_POLICY_SCHEMA_VERSION.to_string(),
            policy: "env-var-only-no-secret-persistence".to_string(),
            api_key_env: config.credentials.api_key_env.clone(),
            api_secret_env: config.credentials.api_secret_env.clone(),
            values_in_file: config.credentials.values_in_file,
            values_recorded: false,
            secrets_redacted: true,
            required_for_network: config.credentials.required_for_network,
            api_key_present: std::env::var(&config.credentials.api_key_env).is_ok(),
            api_secret_present: std::env::var(&config.credentials.api_secret_env).is_ok(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TestnetConnectivityProbe {
    schema_version: String,
    mode: String,
    requested_mode: String,
    http_base_url: String,
    ws_base_url: String,
    network_permission_requested: bool,
    network_attempted: bool,
    testnet_connection: bool,
    status: String,
    diagnostic: String,
}

impl TestnetConnectivityProbe {
    fn from_config(
        config: &TestnetWorkflowConfig,
        mode: WorkflowRunMode,
        allow_testnet_network: bool,
    ) -> Self {
        let requested_mode = requested_mode_label(mode);
        let status = runtime_status_for_testnet_mode(mode);
        let diagnostic = match (mode, allow_testnet_network) {
            (WorkflowRunMode::ConnectivityProbe, true) => {
                "Connectivity-probe intent and network permission were recorded, but v0.6.1 remains offline-only; no socket is opened."
            }
            (WorkflowRunMode::ConnectivityProbe, false) => {
                "Connectivity-probe intent was recorded, but v0.6.1 remains offline-only; no socket is opened."
            }
            (WorkflowRunMode::DryRun, true) => {
                "Network permission was recorded with dry-run mode, but v0.6.1 remains offline-only; no socket is opened."
            }
            (WorkflowRunMode::DryRun, false) => {
                "V06 validates the Binance testnet runtime contract offline; no socket is opened."
            }
        };
        Self {
            schema_version: TESTNET_CONNECTIVITY_PROBE_SCHEMA_VERSION.to_string(),
            mode: config.connectivity.mode.clone(),
            requested_mode: requested_mode.to_string(),
            http_base_url: config.connectivity.http_base_url.clone(),
            ws_base_url: config.connectivity.ws_base_url.clone(),
            network_permission_requested: allow_testnet_network,
            network_attempted: false,
            testnet_connection: false,
            status: status.to_string(),
            diagnostic: diagnostic.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TestnetOrderLifecycle {
    schema_version: String,
    lifecycle_id: String,
    mode: String,
    order_submission: String,
    submitted_count: u64,
    accepted_count: u64,
    filled_count: u64,
    canceled_count: u64,
    rejected_count: u64,
    real_orders_submitted: bool,
    external_venue_connection: bool,
    checksum: String,
}

impl TestnetOrderLifecycle {
    fn from_config(run_id: &str, config: &TestnetWorkflowConfig) -> Self {
        Self {
            schema_version: TESTNET_ORDER_LIFECYCLE_SCHEMA_VERSION.to_string(),
            lifecycle_id: format!("v06-binance-testnet-dry-run-{run_id}"),
            mode: config.run.mode.clone(),
            order_submission: config.execution.order_submission.clone(),
            submitted_count: 0,
            accepted_count: 0,
            filled_count: 0,
            canceled_count: 0,
            rejected_count: 1,
            real_orders_submitted: false,
            external_venue_connection: false,
            checksum: "v06-testnet-dry-run-no-real-orders".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TestnetReconciliation {
    schema_version: String,
    reconciliation_id: String,
    mode: String,
    matched_orders: u64,
    unmatched_orders: u64,
    external_account_state_loaded: bool,
    real_orders_submitted: bool,
    status: String,
}

impl TestnetReconciliation {
    fn from_order_lifecycle(run_id: &str, lifecycle: &TestnetOrderLifecycle) -> Self {
        Self {
            schema_version: TESTNET_RECONCILIATION_SCHEMA_VERSION.to_string(),
            reconciliation_id: format!("v06-binance-testnet-reconciliation-{run_id}"),
            mode: "artifact-only".to_string(),
            matched_orders: lifecycle.submitted_count,
            unmatched_orders: 0,
            external_account_state_loaded: false,
            real_orders_submitted: lifecycle.real_orders_submitted,
            status: "ok".to_string(),
        }
    }
}

fn ensure_field(name: &str, value: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{name} must not be empty");
    }
    Ok(())
}

fn ensure_equals(name: &str, value: &str, expected: &str) -> anyhow::Result<()> {
    if value != expected {
        anyhow::bail!("{name} must be '{expected}', got '{value}'");
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WorkflowBoundary {
    schema_version: String,
    sandbox_only: bool,
    fixture_replay: bool,
    mock_execution: bool,
    external_venue_connection: bool,
    real_funds: bool,
    production_trading: bool,
    real_orders_submitted: bool,
    testnet_connection: bool,
    network_attempted: bool,
    credential_policy: String,
    connectivity_mode: String,
    order_submission_mode: String,
    reconciliation_mode: String,
    notes: Vec<String>,
}

impl WorkflowBoundary {
    fn binance_sandbox() -> Self {
        Self {
            schema_version: BOUNDARY_SCHEMA_VERSION.to_string(),
            sandbox_only: true,
            fixture_replay: true,
            mock_execution: true,
            external_venue_connection: false,
            real_funds: false,
            production_trading: false,
            real_orders_submitted: false,
            testnet_connection: false,
            network_attempted: false,
            credential_policy: "not-required-for-local-fixture-workflow".to_string(),
            connectivity_mode: "offline-fixture".to_string(),
            order_submission_mode: "mock-only".to_string(),
            reconciliation_mode: "not-applicable".to_string(),
            notes: vec![
                "Binance sandbox-only local workflow; no real venue connection.".to_string(),
                "Uses checked-in fixture replay and mock order lifecycle evidence.".to_string(),
                "No real funds, no production trading, no real orders.".to_string(),
            ],
        }
    }

    fn binance_testnet_dry_run(
        credential_policy: &TestnetCredentialPolicy,
        connectivity_probe: &TestnetConnectivityProbe,
    ) -> Self {
        Self {
            schema_version: BOUNDARY_SCHEMA_VERSION.to_string(),
            sandbox_only: true,
            fixture_replay: false,
            mock_execution: true,
            external_venue_connection: false,
            real_funds: false,
            production_trading: false,
            real_orders_submitted: false,
            testnet_connection: connectivity_probe.testnet_connection,
            network_attempted: connectivity_probe.network_attempted,
            credential_policy: credential_policy.policy.clone(),
            connectivity_mode: connectivity_probe.mode.clone(),
            order_submission_mode: "disabled".to_string(),
            reconciliation_mode: "artifact-only".to_string(),
            notes: vec![
                "Binance testnet workflow foundation runs as offline dry-run evidence.".to_string(),
                "No socket is opened and no Binance credential value is recorded.".to_string(),
                "No real funds, no production trading, no real orders.".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WorkflowSummary {
    schema_version: String,
    workflow_id: String,
    workflow: String,
    run_id: String,
    runtime_status: String,
    market_fixture_id: String,
    market_bar_count: usize,
    market_checksum: String,
    ema_smoke_id: String,
    ema_signals_emitted: usize,
    ema_checksum: String,
    rsi_smoke_id: String,
    rsi_signals_emitted: usize,
    rsi_checksum: String,
    order_lifecycle_id: String,
    order_event_count: usize,
    order_checksum: String,
    risk_smoke_id: String,
    risk_checksum: String,
    sandbox_only: bool,
    fixture_replay: bool,
    mock_execution: bool,
    external_venue_connection: bool,
    real_funds: bool,
    production_trading: bool,
    real_orders_submitted: bool,
    testnet_connection: bool,
    network_attempted: bool,
    requested_mode: String,
    network_permission_requested: bool,
    credential_policy: String,
    connectivity_mode: String,
    order_submission_mode: String,
    reconciliation_mode: String,
}

impl WorkflowSummary {
    fn new(
        run_id: &str,
        market_replay: &BinanceReplaySummary,
        ema_strategy: &V04EmaSmokeSummary,
        rsi_strategy: &V04RsiSmokeSummary,
        order_lifecycle: &BinanceMockOrderLifecycleSummary,
        risk_rejection: &V04BinanceRiskRejectionSummary,
        boundary: &WorkflowBoundary,
    ) -> Self {
        Self {
            schema_version: SUMMARY_SCHEMA_VERSION.to_string(),
            workflow_id: WORKFLOW_ID.to_string(),
            workflow: "binance-sandbox".to_string(),
            run_id: run_id.to_string(),
            runtime_status: "completed".to_string(),
            market_fixture_id: market_replay.fixture_id.clone(),
            market_bar_count: market_replay.bar_count,
            market_checksum: market_replay.checksum.clone(),
            ema_smoke_id: ema_strategy.smoke_id.clone(),
            ema_signals_emitted: ema_strategy.signals_emitted,
            ema_checksum: ema_strategy.checksum.clone(),
            rsi_smoke_id: rsi_strategy.smoke_id.clone(),
            rsi_signals_emitted: rsi_strategy.signals_emitted,
            rsi_checksum: rsi_strategy.checksum.clone(),
            order_lifecycle_id: order_lifecycle.lifecycle_id.clone(),
            order_event_count: order_lifecycle.event_count,
            order_checksum: order_lifecycle.checksum.clone(),
            risk_smoke_id: risk_rejection.smoke_id.clone(),
            risk_checksum: risk_rejection.checksum.clone(),
            sandbox_only: boundary.sandbox_only,
            fixture_replay: boundary.fixture_replay,
            mock_execution: boundary.mock_execution,
            external_venue_connection: boundary.external_venue_connection,
            real_funds: boundary.real_funds,
            production_trading: boundary.production_trading,
            real_orders_submitted: boundary.real_orders_submitted,
            testnet_connection: boundary.testnet_connection,
            network_attempted: boundary.network_attempted,
            requested_mode: "dry-run".to_string(),
            network_permission_requested: false,
            credential_policy: boundary.credential_policy.clone(),
            connectivity_mode: boundary.connectivity_mode.clone(),
            order_submission_mode: boundary.order_submission_mode.clone(),
            reconciliation_mode: boundary.reconciliation_mode.clone(),
        }
    }

    fn new_binance_testnet(
        run_id: &str,
        config: &TestnetWorkflowConfig,
        credential_policy: &TestnetCredentialPolicy,
        connectivity_probe: &TestnetConnectivityProbe,
        order_lifecycle: &TestnetOrderLifecycle,
        reconciliation: &TestnetReconciliation,
        boundary: &WorkflowBoundary,
    ) -> Self {
        Self {
            schema_version: SUMMARY_SCHEMA_VERSION.to_string(),
            workflow_id: "v06-binance-testnet-runtime-foundation".to_string(),
            workflow: "binance-testnet".to_string(),
            run_id: run_id.to_string(),
            runtime_status: connectivity_probe.status.clone(),
            market_fixture_id: "not-applicable-testnet-dry-run".to_string(),
            market_bar_count: 0,
            market_checksum: connectivity_probe.status.clone(),
            ema_smoke_id: "not-applicable-testnet-dry-run".to_string(),
            ema_signals_emitted: 0,
            ema_checksum: "not-applicable".to_string(),
            rsi_smoke_id: "not-applicable-testnet-dry-run".to_string(),
            rsi_signals_emitted: 0,
            rsi_checksum: "not-applicable".to_string(),
            order_lifecycle_id: order_lifecycle.lifecycle_id.clone(),
            order_event_count: usize::try_from(order_lifecycle.rejected_count).unwrap_or(0),
            order_checksum: order_lifecycle.checksum.clone(),
            risk_smoke_id: reconciliation.reconciliation_id.clone(),
            risk_checksum: reconciliation.status.clone(),
            sandbox_only: boundary.sandbox_only,
            fixture_replay: boundary.fixture_replay,
            mock_execution: boundary.mock_execution,
            external_venue_connection: boundary.external_venue_connection,
            real_funds: boundary.real_funds,
            production_trading: boundary.production_trading,
            real_orders_submitted: boundary.real_orders_submitted,
            testnet_connection: boundary.testnet_connection,
            network_attempted: boundary.network_attempted,
            requested_mode: connectivity_probe.requested_mode.clone(),
            network_permission_requested: connectivity_probe.network_permission_requested,
            credential_policy: credential_policy.policy.clone(),
            connectivity_mode: config.connectivity.mode.clone(),
            order_submission_mode: config.execution.order_submission.clone(),
            reconciliation_mode: config.execution.reconciliation.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WorkflowEvent {
    schema_version: String,
    workflow_id: String,
    run_id: String,
    sequence: u64,
    event_type: String,
    status: String,
    artifact: String,
    sandbox_only: bool,
    real_orders_submitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WorkflowEvents {
    events: Vec<WorkflowEvent>,
}

impl WorkflowEvents {
    fn from_summary(summary: &WorkflowSummary) -> Self {
        Self::from_artifacts(
            summary,
            [
                ("workflow.market_replay.ready", "market/replay.json"),
                ("workflow.strategy_ema.ready", "strategies/ema.json"),
                ("workflow.strategy_rsi.ready", "strategies/rsi.json"),
                ("workflow.orders.ready", "orders/mock_lifecycle.json"),
                ("workflow.risk.ready", "risk/rejection.json"),
                ("workflow.summary.ready", "summary.json"),
                ("workflow.events.ready", "events.jsonl"),
            ],
        )
    }

    fn from_artifacts<const N: usize>(
        summary: &WorkflowSummary,
        artifacts: [(&str, &str); N],
    ) -> Self {
        let events = artifacts
            .into_iter()
            .enumerate()
            .map(|(index, (event_type, artifact))| WorkflowEvent {
                schema_version: EVENT_SCHEMA_VERSION.to_string(),
                workflow_id: summary.workflow_id.clone(),
                run_id: summary.run_id.clone(),
                sequence: u64::try_from(index + 1).unwrap_or(u64::MAX),
                event_type: event_type.to_string(),
                status: "ok".to_string(),
                artifact: artifact.to_string(),
                sandbox_only: summary.sandbox_only,
                real_orders_submitted: summary.real_orders_submitted,
            })
            .collect();
        Self { events }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WorkflowManifestArtifact {
    path: String,
    schema_version: String,
}

impl WorkflowManifestArtifact {
    fn new(path: String, schema_version: &str) -> Self {
        Self {
            path,
            schema_version: schema_version.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WorkflowManifest {
    schema_version: String,
    workflow_id: String,
    workflow: String,
    run_id: String,
    runtime_status: String,
    artifact_count: usize,
    artifacts: Vec<WorkflowManifestArtifact>,
    summary: WorkflowSummary,
}

impl WorkflowManifest {
    fn new(
        run_id: &str,
        paths: &WorkflowArtifactPaths,
        summary: &WorkflowSummary,
        _events: &WorkflowEvents,
    ) -> Self {
        let artifacts = vec![
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.market_replay, &paths.output_dir),
                "nautilus.binance_replay_summary.v1",
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.ema_strategy, &paths.output_dir),
                "nautilus.v04_ema_smoke_summary.v1",
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.rsi_strategy, &paths.output_dir),
                "nautilus.v04_rsi_smoke_summary.v1",
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.order_lifecycle, &paths.output_dir),
                "nautilus.binance_mock_order_lifecycle_summary.v1",
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.risk_rejection, &paths.output_dir),
                "nautilus.v04_risk_rejection_summary.v1",
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.boundary, &paths.output_dir),
                BOUNDARY_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.summary, &paths.output_dir),
                SUMMARY_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.events, &paths.output_dir),
                EVENT_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&paths.manifest, &paths.output_dir),
                MANIFEST_SCHEMA_VERSION,
            ),
        ];

        Self::new_with_artifacts(run_id, paths, summary, artifacts)
    }

    fn new_with_artifacts(
        run_id: &str,
        _paths: &WorkflowArtifactPaths,
        summary: &WorkflowSummary,
        artifacts: Vec<WorkflowManifestArtifact>,
    ) -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION.to_string(),
            workflow_id: summary.workflow_id.clone(),
            workflow: summary.workflow.clone(),
            run_id: run_id.to_string(),
            runtime_status: summary.runtime_status.clone(),
            artifact_count: artifacts.len(),
            artifacts,
            summary: summary.clone(),
        }
    }
}

fn relative_artifact_path(path: &Path, output_dir: &Path) -> String {
    path.strip_prefix(output_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowRunResult {
    workflow: String,
    run_id: String,
    runtime_status: String,
    output_dir: PathBuf,
    manifest_path: PathBuf,
    summary_path: PathBuf,
    events_path: PathBuf,
    artifact_paths: Vec<PathBuf>,
    external_venue_connection: bool,
    real_funds: bool,
    production_trading: bool,
    real_orders_submitted: bool,
    testnet_connection: bool,
    requested_mode: String,
    network_permission_requested: bool,
    network_attempted: bool,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("ntpro-v05-workflow-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn sandbox_workflow_opt(run_id: &str, output: PathBuf) -> WorkflowRunOpt {
        WorkflowRunOpt {
            workflow: WorkflowKind::BinanceSandbox,
            mode: WorkflowRunMode::DryRun,
            config: None,
            allow_testnet_network: false,
            run_id: Some(run_id.to_string()),
            output: Some(output),
        }
    }

    fn testnet_config() -> String {
        format!(
            r#"[run]
id = "v06-binance-testnet-dry-run"
mode = "dry-run"

[venue]
name = "BINANCE"
product = "spot"
environment = "testnet"

[credentials]
api_key_env = "BINANCE_TESTNET_API_KEY"
api_secret_env = "BINANCE_TESTNET_API_SECRET"
values_in_file = false
required_for_network = true

[connectivity]
mode = "dry-run"
http_base_url = "{}"
ws_base_url = "{}"
network_attempted = false

[execution]
order_submission = "disabled"
reconciliation = "artifact-only"
real_orders_submitted = false
"#,
            get_http_base_url(BinanceProductType::Spot, BinanceEnvironment::Testnet),
            get_ws_base_url(BinanceProductType::Spot, BinanceEnvironment::Testnet)
        )
    }

    fn write_testnet_config(dir: &Path) -> PathBuf {
        let path = dir.join("testnet.toml");
        fs::write(&path, testnet_config()).unwrap();
        path
    }

    #[test]
    fn workflow_run_writes_manifest_last_and_all_artifacts() {
        let output = temp_root("manifest-last");
        let result = run_workflow(sandbox_workflow_opt("v05-test", output)).unwrap();

        assert_eq!(result.artifact_paths.len(), 9);
        assert_eq!(result.artifact_paths.last(), Some(&result.manifest_path));
        for artifact in &result.artifact_paths {
            assert!(artifact.exists(), "{} should exist", artifact.display());
        }
    }

    #[test]
    fn workflow_summary_keeps_v04_sandbox_evidence_boundaries() {
        let output = temp_root("summary-boundary");
        let result = run_workflow(sandbox_workflow_opt("v05-summary", output)).unwrap();
        let summary = fs::read_to_string(&result.summary_path).unwrap();
        let summary: WorkflowSummary = serde_json::from_str(&summary).unwrap();

        assert_eq!(summary.schema_version, SUMMARY_SCHEMA_VERSION);
        assert_eq!(summary.workflow_id, WORKFLOW_ID);
        assert_eq!(summary.market_fixture_id, "v04-binance-spot-bars");
        assert_eq!(
            summary.order_lifecycle_id,
            "v04-binance-mock-order-lifecycle"
        );
        assert_eq!(summary.risk_smoke_id, V04_BINANCE_RISK_REJECTION_SMOKE_ID);
        assert!(summary.sandbox_only);
        assert!(summary.fixture_replay);
        assert!(summary.mock_execution);
        assert!(!summary.external_venue_connection);
        assert!(!summary.real_funds);
        assert!(!summary.production_trading);
        assert!(!summary.real_orders_submitted);
        assert!(!summary.testnet_connection);
        assert!(!summary.network_attempted);
    }

    #[test]
    fn workflow_events_are_valid_jsonl_and_reference_event_completion() {
        let output = temp_root("events");
        let result = run_workflow(sandbox_workflow_opt("v05-events", output)).unwrap();
        let events = fs::read_to_string(&result.events_path).unwrap();
        let parsed = events
            .lines()
            .map(serde_json::from_str::<WorkflowEvent>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(parsed.len(), 7);
        assert_eq!(
            parsed.last().map(|event| event.event_type.as_str()),
            Some("workflow.events.ready")
        );
        assert!(parsed.iter().all(|event| event.sandbox_only));
        assert!(parsed.iter().all(|event| !event.real_orders_submitted));
    }

    #[test]
    fn workflow_run_rejects_empty_run_id() {
        let output = temp_root("empty-run-id");
        let error = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceSandbox,
            mode: WorkflowRunMode::DryRun,
            config: None,
            allow_testnet_network: false,
            run_id: Some(" ".to_string()),
            output: Some(output),
        })
        .unwrap_err();

        assert!(error.to_string().contains("run_id must not be empty"));
    }

    #[test]
    fn binance_testnet_workflow_writes_dry_run_artifacts() {
        let root = temp_root("testnet-dry-run");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceTestnet,
            mode: WorkflowRunMode::DryRun,
            config: Some(config),
            allow_testnet_network: false,
            run_id: Some("v06-smoke".to_string()),
            output: Some(output),
        })
        .unwrap();

        assert_eq!(result.workflow, "binance-testnet");
        assert_eq!(result.runtime_status, "dry_run_completed");
        assert_eq!(result.requested_mode, "dry-run");
        assert!(!result.network_permission_requested);
        assert!(!result.network_attempted);
        assert_eq!(result.artifact_paths.len(), 9);
        assert_eq!(result.artifact_paths.last(), Some(&result.manifest_path));
        assert!(!result.testnet_connection);
        assert!(!result.external_venue_connection);
        assert!(!result.real_orders_submitted);

        let manifest: WorkflowManifest =
            serde_json::from_str(&fs::read_to_string(&result.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.workflow, "binance-testnet");
        assert_eq!(manifest.runtime_status, "dry_run_completed");
        assert_eq!(manifest.artifact_count, 9);
        assert_eq!(manifest.summary.requested_mode, "dry-run");
        assert!(!manifest.summary.network_permission_requested);
        assert_eq!(
            manifest.summary.order_lifecycle_id,
            "v06-binance-testnet-dry-run-v06-smoke"
        );
        assert_eq!(manifest.summary.connectivity_mode, "dry-run");
        assert_eq!(manifest.summary.order_submission_mode, "disabled");
        assert_eq!(manifest.summary.reconciliation_mode, "artifact-only");
        assert!(!manifest.summary.testnet_connection);
        assert!(!manifest.summary.network_attempted);
        assert!(!manifest.summary.real_orders_submitted);

        let events = fs::read_to_string(&result.events_path).unwrap();
        let parsed = events
            .lines()
            .map(serde_json::from_str::<WorkflowEvent>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(parsed.len(), 7);
        assert_eq!(
            parsed.first().map(|event| event.event_type.as_str()),
            Some("workflow.testnet_config.ready")
        );
        assert_eq!(
            parsed.last().map(|event| event.event_type.as_str()),
            Some("workflow.events.ready")
        );
        assert!(parsed.iter().all(|event| !event.real_orders_submitted));
    }

    #[test]
    fn binance_testnet_connectivity_probe_records_offline_probe_semantics() {
        let root = temp_root("testnet-connectivity-probe");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceTestnet,
            mode: WorkflowRunMode::ConnectivityProbe,
            config: Some(config),
            allow_testnet_network: true,
            run_id: Some("probe-run".to_string()),
            output: Some(output),
        })
        .unwrap();

        assert_eq!(result.runtime_status, "offline_probe_validated");
        assert_eq!(result.requested_mode, "connectivity-probe");
        assert!(result.network_permission_requested);
        assert!(!result.network_attempted);
        assert!(!result.testnet_connection);
        assert!(!result.external_venue_connection);
        assert!(!result.real_orders_submitted);

        let manifest: WorkflowManifest =
            serde_json::from_str(&fs::read_to_string(&result.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.runtime_status, "offline_probe_validated");
        assert_eq!(manifest.summary.runtime_status, "offline_probe_validated");
        assert_eq!(manifest.summary.requested_mode, "connectivity-probe");
        assert!(manifest.summary.network_permission_requested);
        assert!(!manifest.summary.network_attempted);
        assert!(!manifest.summary.testnet_connection);
        assert!(!manifest.summary.real_orders_submitted);

        let probe_path = result.output_dir.join("testnet/connectivity_probe.json");
        let probe: TestnetConnectivityProbe =
            serde_json::from_str(&fs::read_to_string(probe_path).unwrap()).unwrap();
        assert_eq!(probe.requested_mode, "connectivity-probe");
        assert!(probe.network_permission_requested);
        assert!(!probe.network_attempted);
        assert!(!probe.testnet_connection);
        assert_eq!(probe.status, "offline_probe_validated");
        assert!(probe.diagnostic.contains("offline-only"));
    }

    #[test]
    fn binance_testnet_workflow_uses_cli_run_id_as_single_artifact_identity() {
        let root = temp_root("testnet-effective-run-id");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceTestnet,
            mode: WorkflowRunMode::DryRun,
            config: Some(config),
            allow_testnet_network: false,
            run_id: Some("custom-run-id".to_string()),
            output: Some(output),
        })
        .unwrap();

        assert_eq!(result.run_id, "custom-run-id");

        let manifest: WorkflowManifest =
            serde_json::from_str(&fs::read_to_string(&result.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.run_id, "custom-run-id");
        assert_eq!(manifest.summary.run_id, "custom-run-id");
        assert_eq!(
            manifest.summary.order_lifecycle_id,
            "v06-binance-testnet-dry-run-custom-run-id"
        );
        assert_eq!(
            manifest.summary.risk_smoke_id,
            "v06-binance-testnet-reconciliation-custom-run-id"
        );

        let config_artifact_path = result.output_dir.join("testnet/config.json");
        let config_artifact: TestnetConfigArtifact =
            serde_json::from_str(&fs::read_to_string(config_artifact_path).unwrap()).unwrap();
        assert_eq!(config_artifact.run_id, "custom-run-id");
        assert_eq!(
            config_artifact.config_declared_run_id,
            "v06-binance-testnet-dry-run"
        );

        let events = fs::read_to_string(&result.events_path).unwrap();
        let parsed = events
            .lines()
            .map(serde_json::from_str::<WorkflowEvent>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(parsed.iter().all(|event| event.run_id == "custom-run-id"));
    }

    #[test]
    fn binance_testnet_workflow_uses_config_run_id_when_cli_run_id_absent() {
        let root = temp_root("testnet-config-run-id");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceTestnet,
            mode: WorkflowRunMode::DryRun,
            config: Some(config),
            allow_testnet_network: false,
            run_id: None,
            output: Some(output),
        })
        .unwrap();

        assert_eq!(result.run_id, "v06-binance-testnet-dry-run");

        let manifest: WorkflowManifest =
            serde_json::from_str(&fs::read_to_string(&result.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.run_id, "v06-binance-testnet-dry-run");
        assert_eq!(manifest.summary.run_id, "v06-binance-testnet-dry-run");
    }

    #[test]
    fn binance_testnet_workflow_requires_config() {
        let output = temp_root("testnet-missing-config");
        let error = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceTestnet,
            mode: WorkflowRunMode::DryRun,
            config: None,
            allow_testnet_network: false,
            run_id: Some("v06-missing-config".to_string()),
            output: Some(output),
        })
        .unwrap_err();

        assert!(error.to_string().contains("requires --config"));
    }
}
