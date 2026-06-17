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
use nautilus_binance::{
    common::{
        consts::BINANCE_API_KEY_HEADER,
        credential::SigningCredential,
        enums::{BinanceEnvironment, BinanceProductType},
        urls::{get_http_base_url, get_ws_base_url},
    },
    mock_lifecycle::{BinanceMockOrderLifecycleSummary, load_v04_binance_mock_order_lifecycle},
    replay::{BinanceReplaySummary, load_v04_binance_spot_bar_replay},
};
use nautilus_core::string::urlencoding;
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
    workflow_contract::{
        BOUNDARY_SCHEMA_VERSION, EVENT_SCHEMA_VERSION, MANIFEST_SCHEMA_VERSION,
        SUMMARY_SCHEMA_VERSION, TESTNET_AUTHENTICATED_READONLY_PROBE_ARTIFACT_PATH,
        TESTNET_AUTHENTICATED_READONLY_PROBE_SCHEMA_VERSION, TESTNET_CONFIG_SCHEMA_VERSION,
        TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH, TESTNET_CONNECTIVITY_PROBE_SCHEMA_VERSION,
        TESTNET_CREDENTIAL_POLICY_SCHEMA_VERSION, TESTNET_HTTP_CONNECTIVITY_PROBE_ARTIFACT_PATH,
        TESTNET_HTTP_CONNECTIVITY_PROBE_SCHEMA_VERSION, TESTNET_ORDER_LIFECYCLE_SCHEMA_VERSION,
        TESTNET_RECONCILIATION_SCHEMA_VERSION, TESTNET_WEBSOCKET_PROBE_ARTIFACT_PATH,
        TESTNET_WEBSOCKET_PROBE_SCHEMA_VERSION, TestnetAuthenticatedReadOnlyProbe,
        TestnetConfigArtifact, TestnetConnectivityProbe, TestnetCredentialPolicy,
        TestnetHttpConnectivityProbe, TestnetOrderLifecycle, TestnetReconciliation,
        TestnetWebSocketConnectivityProbe, WorkflowBoundary, WorkflowEvent, WorkflowManifest,
        WorkflowManifestArtifact, WorkflowSummary,
    },
};

const WORKFLOW_ID: &str = "v05-binance-sandbox-local-workflow";
const DEFAULT_RUN_ID: &str = "v05-binance-sandbox-local";
const TESTNET_NETWORK_OPT_IN_ENV: &str = "NTPRO_ALLOW_TESTNET_NETWORK";
const TESTNET_AUTHENTICATED_MANUAL_ONLINE_ENV: &str = "NTPRO_V08_MANUAL_ONLINE";
const TESTNET_HTTP_READ_ONLY_ENDPOINT: &str = "/api/v3/time";
const TESTNET_AUTHENTICATED_READ_ONLY_ENDPOINT: &str = "/api/v3/account";
const TESTNET_AUTHENTICATED_RECV_WINDOW_MS: u64 = 5_000;
const TESTNET_HTTP_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const TESTNET_CREDENTIAL_LEGACY_REQUIRED_FOR_NETWORK_WARNING: &str = "credentials.required_for_network is deprecated; use credentials.required_for_public_read_only_probe and credentials.required_for_authenticated_read_only_probe";
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
                "workflow.run status=ok workflow={} run_id={} output={} manifest={} summary={} events={} artifact_count={} requested_mode={} network_permission_requested={} network_attempted={} external_venue_connection={} production_venue_connection={} testnet_public_network_connection={} external_network_attempted={} real_funds={} production_trading={} real_orders_submitted={} testnet_connection={} runtime_status={}",
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
                result.production_venue_connection,
                result.testnet_public_network_connection,
                result.external_network_attempted,
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
        WorkflowKind::BinanceTestnet => run_binance_testnet_workflow_with_env_permission(
            opt,
            testnet_network_env_permission_enabled(),
        ),
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
        production_venue_connection: false,
        testnet_public_network_connection: false,
        external_network_attempted: false,
        real_funds: false,
        production_trading: false,
        real_orders_submitted: false,
        testnet_connection: false,
        requested_mode: "dry-run".to_string(),
        network_permission_requested: false,
        network_attempted: false,
    })
}

fn run_binance_testnet_workflow_with_env_permission(
    opt: WorkflowRunOpt,
    env_network_permission: bool,
) -> anyhow::Result<WorkflowRunResult> {
    run_binance_testnet_workflow_with_env_permission_and_http_probe(
        opt,
        env_network_permission,
        execute_testnet_http_read_only_probe,
    )
}

fn run_binance_testnet_workflow_with_env_permission_and_http_probe<F>(
    opt: WorkflowRunOpt,
    env_network_permission: bool,
    http_probe: F,
) -> anyhow::Result<WorkflowRunResult>
where
    F: Fn(&TestnetWorkflowConfig) -> TestnetHttpReadOnlyProbeResult,
{
    run_binance_testnet_workflow_with_env_permission_and_probes(
        opt,
        env_network_permission,
        http_probe,
        execute_testnet_authenticated_read_only_probe,
    )
}

fn run_binance_testnet_workflow_with_env_permission_and_probes<F, G>(
    opt: WorkflowRunOpt,
    env_network_permission: bool,
    http_probe: F,
    authenticated_probe: G,
) -> anyhow::Result<WorkflowRunResult>
where
    F: Fn(&TestnetWorkflowConfig) -> TestnetHttpReadOnlyProbeResult,
    G: Fn(
        &TestnetWorkflowConfig,
        &EnvOnlyTestnetCredentials,
    ) -> TestnetAuthenticatedReadOnlyProbeResult,
{
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

    let env_credentials = EnvOnlyTestnetCredentials::load(&config.credentials);
    let credential_policy =
        TestnetCredentialPolicy::from_config_and_credentials(&config, &env_credentials);
    let network_gate =
        TestnetNetworkGate::evaluate(&config, opt.allow_testnet_network, env_network_permission);
    let http_probe_result =
        should_attempt_testnet_http_probe(opt.mode, &network_gate).then(|| http_probe(&config));
    let authenticated_probe_result = should_attempt_testnet_authenticated_read_only_probe(
        opt.mode,
        &network_gate,
        &env_credentials,
        testnet_authenticated_manual_online_enabled(),
    )
    .then(|| authenticated_probe(&config, &env_credentials));
    let connectivity_probe = TestnetConnectivityProbe::from_config(
        &config,
        opt.mode,
        opt.allow_testnet_network,
        &network_gate,
        &credential_policy,
        http_probe_result.as_ref(),
    );
    let http_connectivity_probe = TestnetHttpConnectivityProbe::from_config(
        &run_id,
        &config,
        opt.allow_testnet_network,
        &network_gate,
        &credential_policy,
        &connectivity_probe,
    );
    let websocket_probe = TestnetWebSocketConnectivityProbe::from_config(
        &run_id,
        &config,
        opt.mode,
        opt.allow_testnet_network,
        &network_gate,
        &credential_policy,
        None,
    );
    let authenticated_readonly_probe = TestnetAuthenticatedReadOnlyProbe::from_config(
        &run_id,
        &config,
        opt.mode,
        opt.allow_testnet_network,
        &network_gate,
        &credential_policy,
        authenticated_probe_result.as_ref(),
    );
    let order_lifecycle = TestnetOrderLifecycle::from_config(&run_id, &config);
    let reconciliation = TestnetReconciliation::from_order_lifecycle(&run_id, &order_lifecycle);
    let boundary =
        WorkflowBoundary::binance_testnet_dry_run(&credential_policy, &connectivity_probe);
    let summary = WorkflowSummary::new_binance_testnet(
        &run_id,
        &config,
        &connectivity_probe,
        &authenticated_readonly_probe,
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
                TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH,
            ),
            (
                "workflow.http_connectivity_probe.ready",
                TESTNET_HTTP_CONNECTIVITY_PROBE_ARTIFACT_PATH,
            ),
            (
                "workflow.websocket_probe.ready",
                TESTNET_WEBSOCKET_PROBE_ARTIFACT_PATH,
            ),
            (
                "workflow.authenticated_readonly_probe.ready",
                TESTNET_AUTHENTICATED_READONLY_PROBE_ARTIFACT_PATH,
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
                relative_artifact_path(
                    &testnet_paths.http_connectivity_probe,
                    &artifact_paths.output_dir,
                ),
                TESTNET_HTTP_CONNECTIVITY_PROBE_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(&testnet_paths.websocket_probe, &artifact_paths.output_dir),
                TESTNET_WEBSOCKET_PROBE_SCHEMA_VERSION,
            ),
            WorkflowManifestArtifact::new(
                relative_artifact_path(
                    &testnet_paths.authenticated_readonly_probe,
                    &artifact_paths.output_dir,
                ),
                TESTNET_AUTHENTICATED_READONLY_PROBE_SCHEMA_VERSION,
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
    write_secret_redacted_json_artifact(
        &testnet_paths.config,
        &config.to_artifact(&run_id),
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &testnet_paths.credential_policy,
        &credential_policy,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &testnet_paths.connectivity_probe,
        &connectivity_probe,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &testnet_paths.http_connectivity_probe,
        &http_connectivity_probe,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &testnet_paths.websocket_probe,
        &websocket_probe,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &testnet_paths.authenticated_readonly_probe,
        &authenticated_readonly_probe,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &testnet_paths.order_lifecycle,
        &order_lifecycle,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &testnet_paths.reconciliation,
        &reconciliation,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &artifact_paths.boundary,
        &boundary,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &artifact_paths.summary,
        &summary,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_events_artifact(
        &artifact_paths.events,
        &events,
        &mut written,
        &env_credentials,
    )?;
    write_secret_redacted_json_artifact(
        &artifact_paths.manifest,
        &manifest,
        &mut written,
        &env_credentials,
    )?;

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
        production_venue_connection: boundary.production_venue_connection,
        testnet_public_network_connection: boundary.testnet_public_network_connection,
        external_network_attempted: boundary.external_network_attempted,
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

fn write_secret_redacted_json_artifact<T>(
    path: &Path,
    value: &T,
    written: &mut Vec<PathBuf>,
    credentials: &EnvOnlyTestnetCredentials,
) -> anyhow::Result<()>
where
    T: Serialize,
{
    let raw = serde_json::to_string_pretty(value)?;
    let body = format!("{raw}\n");
    credentials.ensure_no_secret_values_absent(&path.display().to_string(), &body)?;
    atomic_write_text(path, &body)?;
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

fn write_secret_redacted_events_artifact(
    path: &Path,
    events: &WorkflowEvents,
    written: &mut Vec<PathBuf>,
    credentials: &EnvOnlyTestnetCredentials,
) -> anyhow::Result<()> {
    let mut body = String::new();
    for event in &events.events {
        body.push_str(&serde_json::to_string(event)?);
        body.push('\n');
    }
    credentials.ensure_no_secret_values_absent(&path.display().to_string(), &body)?;
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
    http_connectivity_probe: PathBuf,
    websocket_probe: PathBuf,
    authenticated_readonly_probe: PathBuf,
    order_lifecycle: PathBuf,
    reconciliation: PathBuf,
}

impl TestnetArtifactPaths {
    fn new(output_dir: &Path) -> Self {
        Self {
            config: output_dir.join("testnet/config.json"),
            credential_policy: output_dir.join("testnet/credential_policy.json"),
            connectivity_probe: output_dir.join(TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH),
            http_connectivity_probe: output_dir.join(TESTNET_HTTP_CONNECTIVITY_PROBE_ARTIFACT_PATH),
            websocket_probe: output_dir.join(TESTNET_WEBSOCKET_PROBE_ARTIFACT_PATH),
            authenticated_readonly_probe: output_dir
                .join(TESTNET_AUTHENTICATED_READONLY_PROBE_ARTIFACT_PATH),
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
            anyhow::bail!("credentials.values_in_file must be false for the testnet workflow");
        }
        if self
            .credentials
            .public_read_only_probe_requires_credentials()
        {
            anyhow::bail!(
                "credentials.required_for_public_read_only_probe must be false for public read-only testnet probes"
            );
        }
        if !self
            .credentials
            .authenticated_read_only_probe_requires_credentials()
        {
            anyhow::bail!(
                "credentials.required_for_authenticated_read_only_probe must be true for authenticated testnet online probes"
            );
        }
        if self.connectivity.network_attempted {
            anyhow::bail!("connectivity.network_attempted must be false for checked-in config");
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
    #[serde(default)]
    required_for_network: Option<bool>,
    #[serde(default)]
    required_for_public_read_only_probe: Option<bool>,
    #[serde(default)]
    required_for_authenticated_read_only_probe: Option<bool>,
}

impl TestnetCredentialConfig {
    fn legacy_required_for_network_present(&self) -> bool {
        self.required_for_network.is_some()
    }

    fn legacy_required_for_network_value(&self) -> bool {
        self.required_for_network.unwrap_or(false)
    }

    fn public_read_only_probe_requires_credentials(&self) -> bool {
        self.required_for_public_read_only_probe.unwrap_or(false)
    }

    fn authenticated_read_only_probe_requires_credentials(&self) -> bool {
        self.required_for_authenticated_read_only_probe
            .or(self.required_for_network)
            .unwrap_or(true)
    }

    fn credential_config_migration_warning(&self) -> String {
        if self.legacy_required_for_network_present() {
            TESTNET_CREDENTIAL_LEGACY_REQUIRED_FOR_NETWORK_WARNING.to_string()
        } else {
            String::new()
        }
    }
}

struct EnvOnlyTestnetCredentials {
    api_key_env: String,
    api_secret_env: String,
    api_key_value: Option<String>,
    api_secret_value: Option<String>,
    api_key_present: bool,
    api_secret_present: bool,
    sensitive_values: Vec<String>,
}

impl EnvOnlyTestnetCredentials {
    fn load(config: &TestnetCredentialConfig) -> Self {
        let api_key_value = read_env_secret_value(&config.api_key_env);
        let api_secret_value = read_env_secret_value(&config.api_secret_env);
        Self::from_values(
            config.api_key_env.clone(),
            api_key_value,
            config.api_secret_env.clone(),
            api_secret_value,
        )
    }

    #[cfg(test)]
    fn from_presence(
        api_key_env: String,
        api_key_present: bool,
        api_secret_env: String,
        api_secret_present: bool,
    ) -> Self {
        Self {
            api_key_env,
            api_secret_env,
            api_key_value: None,
            api_secret_value: None,
            api_key_present,
            api_secret_present,
            sensitive_values: Vec::new(),
        }
    }

    fn from_values(
        api_key_env: String,
        api_key_value: Option<String>,
        api_secret_env: String,
        api_secret_value: Option<String>,
    ) -> Self {
        let api_key_present = api_key_value
            .as_ref()
            .is_some_and(|value| !value.is_empty());
        let api_secret_present = api_secret_value
            .as_ref()
            .is_some_and(|value| !value.is_empty());
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
            api_key_present,
            api_secret_present,
            sensitive_values,
        }
    }

    fn authenticated_read_only_ready(&self) -> bool {
        self.api_key_present && self.api_secret_present
    }

    fn ensure_no_secret_values_absent(&self, label: &str, body: &str) -> anyhow::Result<()> {
        for secret_value in &self.sensitive_values {
            if body.contains(secret_value) {
                anyhow::bail!(
                    "testnet secret redaction guard blocked secret value leak in {label}"
                );
            }
        }
        Ok(())
    }

    fn signing_credential(&self) -> anyhow::Result<SigningCredential> {
        let api_key = self
            .api_key_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("authenticated read-only request requires API key env value")?;
        let api_secret = self
            .api_secret_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("authenticated read-only request requires API secret env value")?;

        Ok(SigningCredential::new(
            api_key.to_string(),
            api_secret.to_string(),
        ))
    }
}

fn read_env_secret_value(env_var: &str) -> Option<String> {
    std::env::var(env_var)
        .ok()
        .filter(|value| !value.is_empty())
}

struct TestnetSignedAuthenticatedGetRequest {
    method: String,
    endpoint_path: String,
    endpoint_url_redacted: String,
    query_without_signature: String,
    signature: String,
    signed_query: String,
    api_key_header_name: String,
    api_key_header_value: String,
}

impl Debug for TestnetSignedAuthenticatedGetRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TestnetSignedAuthenticatedGetRequest")
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestnetSignedAuthenticatedGetRequestPreview {
    endpoint_class: String,
    endpoint_url_redacted: String,
    request_method: String,
    request_target: String,
    query_shape: String,
    api_key_header_name: String,
    api_key_header_value_recorded: bool,
    signature_recorded: bool,
    signed_query_recorded: bool,
    signed_url_recorded: bool,
    request_body_recorded: bool,
    order_submission: String,
    account_mutation: bool,
    production_binance_connectivity: bool,
    real_funds: bool,
    production_trading: bool,
    diagnostic: String,
}

impl TestnetSignedAuthenticatedGetRequest {
    fn signed_url_for_execution(&self) -> String {
        format!("{}?{}", self.endpoint_url_redacted, self.signed_query)
    }

    fn redacted_preview(&self) -> TestnetSignedAuthenticatedGetRequestPreview {
        TestnetSignedAuthenticatedGetRequestPreview {
            endpoint_class: "binance-testnet-authenticated-read-only-account".to_string(),
            endpoint_url_redacted: self.endpoint_url_redacted.clone(),
            request_method: self.method.clone(),
            request_target: self.endpoint_path.clone(),
            query_shape: "timestamp=<ms>&recvWindow=<ms>&signature=<redacted>".to_string(),
            api_key_header_name: self.api_key_header_name.clone(),
            api_key_header_value_recorded: false,
            signature_recorded: false,
            signed_query_recorded: false,
            signed_url_recorded: false,
            request_body_recorded: false,
            order_submission: "disabled".to_string(),
            account_mutation: false,
            production_binance_connectivity: false,
            real_funds: false,
            production_trading: false,
            diagnostic: "V080 authenticated read-only GET request builder prepared Binance testnet /api/v3/account request metadata; API key header value, signature, signed query, signed URL, and response body stay memory-only and redacted.".to_string(),
        }
    }

    fn ensure_preview_redacted(
        &self,
        credentials: &EnvOnlyTestnetCredentials,
    ) -> anyhow::Result<()> {
        let preview = self.redacted_preview();
        let body = serde_json::to_string(&preview)?;
        credentials.ensure_no_secret_values_absent("authenticated-read-only-preview", &body)?;
        for (label, sensitive_value) in [
            ("signature", self.signature.as_str()),
            ("signed query", self.signed_query.as_str()),
            ("API key header value", self.api_key_header_value.as_str()),
        ] {
            if !sensitive_value.is_empty() && body.contains(sensitive_value) {
                anyhow::bail!("authenticated read-only preview leaked {label}");
            }
        }
        Ok(())
    }
}

fn build_testnet_authenticated_read_only_get_request(
    config: &TestnetWorkflowConfig,
    credentials: &EnvOnlyTestnetCredentials,
    timestamp_ms: u64,
) -> anyhow::Result<TestnetSignedAuthenticatedGetRequest> {
    build_testnet_authenticated_read_only_request(
        config,
        credentials,
        "GET",
        TESTNET_AUTHENTICATED_READ_ONLY_ENDPOINT,
        timestamp_ms,
        TESTNET_AUTHENTICATED_RECV_WINDOW_MS,
    )
}

fn build_testnet_authenticated_read_only_request(
    config: &TestnetWorkflowConfig,
    credentials: &EnvOnlyTestnetCredentials,
    method: &str,
    endpoint_path: &str,
    timestamp_ms: u64,
    recv_window_ms: u64,
) -> anyhow::Result<TestnetSignedAuthenticatedGetRequest> {
    let endpoint_path = normalize_testnet_authenticated_endpoint_path(endpoint_path)?;
    ensure_testnet_authenticated_request_allowed(method, &endpoint_path)?;
    if recv_window_ms == 0 {
        anyhow::bail!("authenticated read-only request recvWindow must be positive");
    }

    let signing_credential = credentials.signing_credential()?;
    let query_without_signature = format!("timestamp={timestamp_ms}&recvWindow={recv_window_ms}");
    let signature =
        urlencoding::encode(&signing_credential.sign(&query_without_signature)).into_owned();
    let signed_query = format!("{query_without_signature}&signature={signature}");
    let request = TestnetSignedAuthenticatedGetRequest {
        method: "GET".to_string(),
        endpoint_path: endpoint_path.clone(),
        endpoint_url_redacted: format!(
            "{}{}",
            config.connectivity.http_base_url.trim_end_matches('/'),
            endpoint_path,
        ),
        query_without_signature,
        signature,
        signed_query,
        api_key_header_name: BINANCE_API_KEY_HEADER.to_string(),
        api_key_header_value: signing_credential.api_key().to_string(),
    };
    request.ensure_preview_redacted(credentials)?;
    Ok(request)
}

fn normalize_testnet_authenticated_endpoint_path(endpoint_path: &str) -> anyhow::Result<String> {
    let endpoint_path = endpoint_path.trim();
    if endpoint_path.is_empty() {
        anyhow::bail!("authenticated read-only request endpoint must not be empty");
    }
    if endpoint_path.contains('?') {
        anyhow::bail!("authenticated read-only request endpoint must not include query parameters");
    }
    if !endpoint_path.starts_with('/') {
        anyhow::bail!("authenticated read-only request endpoint must start with '/'");
    }
    Ok(endpoint_path.to_string())
}

fn ensure_testnet_authenticated_request_allowed(
    method: &str,
    endpoint_path: &str,
) -> anyhow::Result<()> {
    if method != "GET" {
        anyhow::bail!("authenticated read-only request builder only allows GET, got {method}");
    }
    if endpoint_path.starts_with("/api/v3/order") {
        anyhow::bail!(
            "authenticated read-only request builder rejects order mutation endpoint {endpoint_path}"
        );
    }
    if endpoint_path != TESTNET_AUTHENTICATED_READ_ONLY_ENDPOINT {
        anyhow::bail!(
            "authenticated read-only request builder allowlist only includes {TESTNET_AUTHENTICATED_READ_ONLY_ENDPOINT}, got {endpoint_path}"
        );
    }
    Ok(())
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

impl TestnetCredentialPolicy {
    #[cfg(test)]
    fn from_config_with_presence(
        config: &TestnetWorkflowConfig,
        api_key_present: bool,
        api_secret_present: bool,
    ) -> Self {
        let credentials = EnvOnlyTestnetCredentials::from_presence(
            config.credentials.api_key_env.clone(),
            api_key_present,
            config.credentials.api_secret_env.clone(),
            api_secret_present,
        );
        Self::from_config_and_credentials(config, &credentials)
    }

    fn from_config_and_credentials(
        config: &TestnetWorkflowConfig,
        credentials: &EnvOnlyTestnetCredentials,
    ) -> Self {
        Self {
            schema_version: TESTNET_CREDENTIAL_POLICY_SCHEMA_VERSION.to_string(),
            policy: "env-var-only-no-secret-persistence".to_string(),
            credential_source: "environment_variables_only".to_string(),
            api_key_env: credentials.api_key_env.clone(),
            api_secret_env: credentials.api_secret_env.clone(),
            values_in_file: config.credentials.values_in_file,
            values_recorded: false,
            api_key_value_recorded: false,
            api_secret_value_recorded: false,
            secrets_redacted: true,
            required_for_network: config.credentials.legacy_required_for_network_value(),
            required_for_public_read_only_probe: config
                .credentials
                .public_read_only_probe_requires_credentials(),
            required_for_authenticated_read_only_probe: config
                .credentials
                .authenticated_read_only_probe_requires_credentials(),
            legacy_required_for_network_present: config
                .credentials
                .legacy_required_for_network_present(),
            credential_config_migration_warning: config
                .credentials
                .credential_config_migration_warning(),
            public_read_only_probe_requires_credentials: config
                .credentials
                .public_read_only_probe_requires_credentials(),
            authenticated_read_only_probe_requires_credentials: config
                .credentials
                .authenticated_read_only_probe_requires_credentials(),
            authenticated_read_only_probe_gate: "manual-online-only".to_string(),
            authenticated_read_only_probe_status: authenticated_probe_status(
                credentials.api_key_present,
                credentials.api_secret_present,
            )
            .to_string(),
            authenticated_read_only_probe_fail_closed: !credentials.authenticated_read_only_ready(),
            api_key_present: credentials.api_key_present,
            api_secret_present: credentials.api_secret_present,
        }
    }
}

fn authenticated_probe_status(api_key_present: bool, api_secret_present: bool) -> &'static str {
    if api_key_present && api_secret_present {
        "manual_gate_ready"
    } else {
        "manual_gate_blocked_missing_credentials"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestnetNetworkGate {
    env_network_permission: bool,
    status: String,
    reasons: Vec<String>,
}

impl TestnetNetworkGate {
    fn evaluate(
        config: &TestnetWorkflowConfig,
        allow_testnet_network: bool,
        env_network_permission: bool,
    ) -> Self {
        let mut reasons = Vec::new();
        if !allow_testnet_network {
            reasons.push("missing --allow-testnet-network".to_string());
        }
        if !env_network_permission {
            reasons.push(format!("{TESTNET_NETWORK_OPT_IN_ENV}=1 is not set"));
        }
        if config.venue.environment != "testnet" {
            reasons.push(format!(
                "venue.environment must be 'testnet', got '{}'",
                config.venue.environment
            ));
        }
        if config.execution.order_submission != "disabled" {
            reasons.push(format!(
                "execution.order_submission must be 'disabled', got '{}'",
                config.execution.order_submission
            ));
        }
        if config.execution.real_orders_submitted {
            reasons.push("execution.real_orders_submitted must be false".to_string());
        }

        let status = if reasons.is_empty() {
            "allowed"
        } else {
            "blocked"
        };

        Self {
            env_network_permission,
            status: status.to_string(),
            reasons,
        }
    }

    fn is_allowed(&self) -> bool {
        self.status == "allowed"
    }
}

fn testnet_network_env_permission_enabled() -> bool {
    matches!(
        std::env::var(TESTNET_NETWORK_OPT_IN_ENV).as_deref(),
        Ok("1")
    )
}

fn testnet_authenticated_manual_online_enabled() -> bool {
    matches!(
        std::env::var(TESTNET_AUTHENTICATED_MANUAL_ONLINE_ENV).as_deref(),
        Ok("1")
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestnetHttpReadOnlyProbeResult {
    endpoint_class: String,
    latency_ms: Option<u64>,
    http_status: Option<u16>,
    response_shape: String,
    response_shape_validated: bool,
    error_code: String,
    network_attempted: bool,
    testnet_connection: bool,
    status: String,
    diagnostic: String,
}

impl TestnetHttpReadOnlyProbeResult {
    fn success(latency_ms: u64, http_status: u16, response_shape: &str) -> Self {
        Self {
            endpoint_class: "binance-testnet-public-http-time".to_string(),
            latency_ms: Some(latency_ms),
            http_status: Some(http_status),
            response_shape: response_shape.to_string(),
            response_shape_validated: true,
            error_code: "none".to_string(),
            network_attempted: true,
            testnet_connection: true,
            status: "http_read_only_probe_ok".to_string(),
            diagnostic: format!(
                "V070 HTTP read-only probe succeeded against Binance testnet public time endpoint with HTTP {http_status}."
            ),
        }
    }

    fn failure(latency_ms: Option<u64>, http_status: Option<u16>, error_code: &str) -> Self {
        let status_detail = http_status
            .map(|status| format!(" HTTP {status}"))
            .unwrap_or_default();
        Self {
            endpoint_class: "binance-testnet-public-http-time".to_string(),
            latency_ms,
            http_status,
            response_shape: "binance_server_time_v1".to_string(),
            response_shape_validated: false,
            error_code: error_code.to_string(),
            network_attempted: true,
            testnet_connection: false,
            status: "http_read_only_probe_failed".to_string(),
            diagnostic: format!(
                "V070 HTTP read-only probe attempted Binance testnet public time endpoint and failed with {error_code}.{status_detail}"
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BinanceServerTimeResponse {
    #[serde(rename = "serverTime")]
    server_time: u64,
}

fn validates_binance_server_time_response_shape(body: &BinanceServerTimeResponse) -> bool {
    body.server_time > 0
}

fn should_attempt_testnet_http_probe(
    mode: WorkflowRunMode,
    network_gate: &TestnetNetworkGate,
) -> bool {
    mode == WorkflowRunMode::ConnectivityProbe && network_gate.is_allowed()
}

fn should_attempt_testnet_authenticated_read_only_probe(
    mode: WorkflowRunMode,
    network_gate: &TestnetNetworkGate,
    credentials: &EnvOnlyTestnetCredentials,
    authenticated_manual_online_permission: bool,
) -> bool {
    mode == WorkflowRunMode::ConnectivityProbe
        && network_gate.is_allowed()
        && credentials.authenticated_read_only_ready()
        && authenticated_manual_online_permission
}

fn current_unix_timestamp_ms() -> anyhow::Result<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_millis();
    u64::try_from(millis).context("current UNIX timestamp milliseconds exceeds u64")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestnetAuthenticatedReadOnlyProbeResult {
    endpoint_class: String,
    latency_ms: Option<u64>,
    http_status: Option<u16>,
    response_shape: String,
    response_shape_validated: bool,
    error_code: String,
    network_attempted: bool,
    testnet_connection: bool,
    status: String,
    diagnostic: String,
}

impl TestnetAuthenticatedReadOnlyProbeResult {
    fn success(latency_ms: u64, http_status: u16) -> Self {
        Self {
            endpoint_class: "binance-testnet-authenticated-readonly-account".to_string(),
            latency_ms: Some(latency_ms),
            http_status: Some(http_status),
            response_shape: "binance_account_v1".to_string(),
            response_shape_validated: true,
            error_code: "none".to_string(),
            network_attempted: true,
            testnet_connection: true,
            status: "authenticated_readonly_probe_ok".to_string(),
            diagnostic: format!(
                "V080 authenticated read-only probe validated Binance testnet account response shape with HTTP {http_status}; raw account body, balances, uid, headers, signature, and signed URL were not recorded."
            ),
        }
    }

    fn failure(latency_ms: Option<u64>, http_status: Option<u16>, error_code: &str) -> Self {
        let status_detail = http_status
            .map(|status| format!(" HTTP {status}"))
            .unwrap_or_default();
        Self {
            endpoint_class: "binance-testnet-authenticated-readonly-account".to_string(),
            latency_ms,
            http_status,
            response_shape: "binance_account_v1".to_string(),
            response_shape_validated: false,
            error_code: error_code.to_string(),
            network_attempted: true,
            testnet_connection: false,
            status: "authenticated_readonly_probe_failed".to_string(),
            diagnostic: format!(
                "V080 authenticated read-only probe attempted Binance testnet account endpoint and failed with {error_code}.{status_detail} Raw account body, balances, uid, headers, signature, and signed URL were not recorded."
            ),
        }
    }
}

fn execute_testnet_authenticated_read_only_probe(
    config: &TestnetWorkflowConfig,
    credentials: &EnvOnlyTestnetCredentials,
) -> TestnetAuthenticatedReadOnlyProbeResult {
    match build_testnet_authenticated_read_only_get_request(
        config,
        credentials,
        current_unix_timestamp_ms().unwrap_or(0),
    ) {
        Ok(request) => {
            let url = request.signed_url_for_execution();
            let header_name = request.api_key_header_name;
            let header_value = request.api_key_header_value;
            std::thread::spawn(move || {
                execute_testnet_authenticated_read_only_probe_on_thread(
                    &url,
                    &header_name,
                    &header_value,
                )
            })
            .join()
            .unwrap_or_else(|_| {
                TestnetAuthenticatedReadOnlyProbeResult::failure(
                    None,
                    None,
                    "authenticated_probe_thread_panicked",
                )
            })
        }
        Err(_) => TestnetAuthenticatedReadOnlyProbeResult::failure(
            None,
            None,
            "signed_request_builder_failed",
        ),
    }
}

fn execute_testnet_authenticated_read_only_probe_on_thread(
    signed_url: &str,
    api_key_header_name: &str,
    api_key_header_value: &str,
) -> TestnetAuthenticatedReadOnlyProbeResult {
    let started = Instant::now();
    let client = match reqwest::blocking::Client::builder()
        .timeout(TESTNET_HTTP_PROBE_TIMEOUT)
        .user_agent("NTPRO-v080-authenticated-readonly-probe")
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return TestnetAuthenticatedReadOnlyProbeResult::failure(
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
                    Ok(body) if validates_binance_account_response_shape(&body) => {
                        TestnetAuthenticatedReadOnlyProbeResult::success(latency_ms, status)
                    }
                    Ok(_) | Err(_) => TestnetAuthenticatedReadOnlyProbeResult::failure(
                        Some(latency_ms),
                        Some(status),
                        "response_shape_invalid",
                    ),
                }
            } else {
                TestnetAuthenticatedReadOnlyProbeResult::failure(
                    Some(latency_ms),
                    Some(status),
                    "http_status_not_success",
                )
            }
        }
        Err(error) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            TestnetAuthenticatedReadOnlyProbeResult::failure(
                Some(latency_ms),
                error.status().map(|status| status.as_u16()),
                classify_http_probe_error(&error),
            )
        }
    }
}

fn validates_binance_account_response_shape(body: &serde_json::Value) -> bool {
    let Some(object) = body.as_object() else {
        return false;
    };
    object
        .get("accountType")
        .is_some_and(serde_json::Value::is_string)
        && object
            .get("balances")
            .is_some_and(serde_json::Value::is_array)
        && object
            .get("canTrade")
            .is_some_and(serde_json::Value::is_boolean)
}

fn testnet_http_read_only_probe_url(config: &TestnetWorkflowConfig) -> String {
    format!(
        "{}{}",
        config.connectivity.http_base_url.trim_end_matches('/'),
        TESTNET_HTTP_READ_ONLY_ENDPOINT
    )
}

fn execute_testnet_http_read_only_probe(
    config: &TestnetWorkflowConfig,
) -> TestnetHttpReadOnlyProbeResult {
    let url = testnet_http_read_only_probe_url(config);
    std::thread::spawn(move || execute_testnet_http_read_only_probe_on_thread(&url))
        .join()
        .unwrap_or_else(|_| {
            TestnetHttpReadOnlyProbeResult::failure(None, None, "http_probe_thread_panicked")
        })
}

fn execute_testnet_http_read_only_probe_on_thread(url: &str) -> TestnetHttpReadOnlyProbeResult {
    let started = Instant::now();
    let client = match reqwest::blocking::Client::builder()
        .timeout(TESTNET_HTTP_PROBE_TIMEOUT)
        .user_agent("NTPRO-v070-read-only-probe")
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return TestnetHttpReadOnlyProbeResult::failure(None, None, "http_client_build_failed");
        }
    };

    match client.get(url).send() {
        Ok(response) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            let status = response.status().as_u16();
            if response.status().is_success() {
                match response.json::<BinanceServerTimeResponse>() {
                    Ok(body) if validates_binance_server_time_response_shape(&body) => {
                        TestnetHttpReadOnlyProbeResult::success(
                            latency_ms,
                            status,
                            "binance_server_time_v1",
                        )
                    }
                    Ok(_) => TestnetHttpReadOnlyProbeResult::failure(
                        Some(latency_ms),
                        Some(status),
                        "response_shape_invalid",
                    ),
                    Err(_) => TestnetHttpReadOnlyProbeResult::failure(
                        Some(latency_ms),
                        Some(status),
                        "response_shape_invalid",
                    ),
                }
            } else {
                TestnetHttpReadOnlyProbeResult::failure(
                    Some(latency_ms),
                    Some(status),
                    "http_status_not_success",
                )
            }
        }
        Err(error) => {
            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            let error_code = classify_http_probe_error(&error);
            TestnetHttpReadOnlyProbeResult::failure(
                Some(latency_ms),
                error.status().map(|s| s.as_u16()),
                error_code,
            )
        }
    }
}

fn classify_http_probe_error(error: &reqwest::Error) -> &'static str {
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

impl TestnetConnectivityProbe {
    fn from_config(
        config: &TestnetWorkflowConfig,
        mode: WorkflowRunMode,
        allow_testnet_network: bool,
        network_gate: &TestnetNetworkGate,
        credential_policy: &TestnetCredentialPolicy,
        http_probe_result: Option<&TestnetHttpReadOnlyProbeResult>,
    ) -> Self {
        let requested_mode = requested_mode_label(mode);
        let status = http_probe_result.map_or_else(
            || runtime_status_for_testnet_mode(mode),
            |result| result.status.as_str(),
        );
        let diagnostic = http_probe_result.map_or_else(
            || network_gate_diagnostic(mode, network_gate, credential_policy),
            |result| result.diagnostic.clone(),
        );
        Self {
            schema_version: TESTNET_CONNECTIVITY_PROBE_SCHEMA_VERSION.to_string(),
            mode: config.connectivity.mode.clone(),
            requested_mode: requested_mode.to_string(),
            public_read_only_probe_status: "available_without_credentials".to_string(),
            authenticated_read_only_probe_status: credential_policy
                .authenticated_read_only_probe_status
                .clone(),
            authenticated_read_only_probe_gate: credential_policy
                .authenticated_read_only_probe_gate
                .clone(),
            authenticated_read_only_probe_requires_credentials: credential_policy
                .authenticated_read_only_probe_requires_credentials,
            http_base_url: config.connectivity.http_base_url.clone(),
            ws_base_url: config.connectivity.ws_base_url.clone(),
            endpoint_class: http_probe_result.map_or_else(
                || "binance-testnet-public-http-time".to_string(),
                |result| result.endpoint_class.clone(),
            ),
            latency_ms: http_probe_result.and_then(|result| result.latency_ms),
            http_status: http_probe_result.and_then(|result| result.http_status),
            response_shape: http_probe_result.map_or_else(
                || "binance_server_time_v1".to_string(),
                |result| result.response_shape.clone(),
            ),
            response_shape_validated: http_probe_result
                .is_some_and(|result| result.response_shape_validated),
            error_code: http_probe_result.map_or_else(
                || "not_attempted".to_string(),
                |result| result.error_code.clone(),
            ),
            network_permission_requested: allow_testnet_network,
            env_network_permission: network_gate.env_network_permission,
            network_gate_status: network_gate.status.clone(),
            network_gate_reasons: network_gate.reasons.clone(),
            network_attempted: http_probe_result.is_some_and(|result| result.network_attempted),
            testnet_connection: http_probe_result.is_some_and(|result| result.testnet_connection),
            status: status.to_string(),
            diagnostic,
        }
    }
}

impl TestnetHttpConnectivityProbe {
    fn from_config(
        run_id: &str,
        config: &TestnetWorkflowConfig,
        allow_testnet_network: bool,
        network_gate: &TestnetNetworkGate,
        credential_policy: &TestnetCredentialPolicy,
        connectivity_probe: &TestnetConnectivityProbe,
    ) -> Self {
        Self {
            schema_version: TESTNET_HTTP_CONNECTIVITY_PROBE_SCHEMA_VERSION.to_string(),
            run_id: run_id.to_string(),
            environment: config.venue.environment.clone(),
            product: config.venue.product.clone(),
            endpoint_kind: "http_read_only".to_string(),
            endpoint_url_redacted: testnet_http_read_only_probe_url(config),
            network_gate_status: network_gate.status.clone(),
            network_gate_reasons: network_gate.reasons.clone(),
            network_permission_requested: allow_testnet_network,
            env_network_permission: network_gate.env_network_permission,
            network_attempted: connectivity_probe.network_attempted,
            testnet_connection: connectivity_probe.testnet_connection,
            order_submission: config.execution.order_submission.clone(),
            real_orders_submitted: false,
            credential_policy: credential_policy.policy.clone(),
            api_key_present: credential_policy.api_key_present,
            api_secret_present: credential_policy.api_secret_present,
            request_method: "GET".to_string(),
            request_target: TESTNET_HTTP_READ_ONLY_ENDPOINT.to_string(),
            response_status_code: connectivity_probe.http_status,
            response_shape: connectivity_probe.response_shape.clone(),
            response_shape_validated: connectivity_probe.response_shape_validated,
            latency_ms: connectivity_probe.latency_ms,
            error_code: connectivity_probe.error_code.clone(),
            status: connectivity_probe.status.clone(),
            diagnostic: connectivity_probe.diagnostic.clone(),
            generated_at: workflow_generated_at(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestnetWebSocketReadOnlyProbeResult {
    error_code: String,
    network_attempted: bool,
    testnet_connection: bool,
    status: String,
    diagnostic: String,
}

impl TestnetWebSocketReadOnlyProbeResult {
    #[cfg(test)]
    fn classified_failure(error_code: &str) -> Self {
        Self {
            error_code: error_code.to_string(),
            network_attempted: true,
            testnet_connection: false,
            status: "websocket_read_only_probe_failed".to_string(),
            diagnostic: format!(
                "V070 WebSocket read-only probe fixture classified the manual failure as {error_code}. No subscription was attempted and no order API was used."
            ),
        }
    }
}

impl TestnetWebSocketConnectivityProbe {
    fn from_config(
        run_id: &str,
        config: &TestnetWorkflowConfig,
        mode: WorkflowRunMode,
        allow_testnet_network: bool,
        network_gate: &TestnetNetworkGate,
        credential_policy: &TestnetCredentialPolicy,
        websocket_probe_result: Option<&TestnetWebSocketReadOnlyProbeResult>,
    ) -> Self {
        let (status, error_code, diagnostic) = websocket_probe_result.map_or_else(
            || websocket_probe_not_attempted_result(mode, network_gate),
            |result| {
                (
                    result.status.clone(),
                    result.error_code.clone(),
                    result.diagnostic.clone(),
                )
            },
        );
        let network_attempted =
            websocket_probe_result.is_some_and(|result| result.network_attempted);
        let testnet_connection =
            websocket_probe_result.is_some_and(|result| result.testnet_connection);

        Self {
            schema_version: TESTNET_WEBSOCKET_PROBE_SCHEMA_VERSION.to_string(),
            run_id: run_id.to_string(),
            mode: config.connectivity.mode.clone(),
            requested_mode: requested_mode_label(mode).to_string(),
            endpoint_kind: "websocket_read_only".to_string(),
            endpoint_class: "binance-testnet-public-websocket-handshake".to_string(),
            ws_base_url: config.connectivity.ws_base_url.clone(),
            network_gate_status: network_gate.status.clone(),
            network_gate_reasons: network_gate.reasons.clone(),
            network_permission_requested: allow_testnet_network,
            env_network_permission: network_gate.env_network_permission,
            websocket_probe_gate: "manual-online-only".to_string(),
            websocket_attempted: network_attempted,
            network_attempted,
            testnet_connection,
            subscription_attempted: false,
            message_count: 0,
            order_submission: config.execution.order_submission.clone(),
            real_orders_submitted: false,
            values_recorded: credential_policy.values_recorded,
            secrets_redacted: credential_policy.secrets_redacted,
            status,
            error_code,
            diagnostic,
            generated_at: workflow_generated_at(),
        }
    }
}

impl TestnetAuthenticatedReadOnlyProbe {
    fn from_config(
        run_id: &str,
        config: &TestnetWorkflowConfig,
        mode: WorkflowRunMode,
        allow_testnet_network: bool,
        network_gate: &TestnetNetworkGate,
        credential_policy: &TestnetCredentialPolicy,
        authenticated_probe_result: Option<&TestnetAuthenticatedReadOnlyProbeResult>,
    ) -> Self {
        let (status, error_code, diagnostic) = authenticated_probe_result.map_or_else(
            || {
                authenticated_readonly_probe_not_attempted_result(
                    mode,
                    network_gate,
                    credential_policy,
                )
            },
            |result| {
                (
                    result.status.clone(),
                    result.error_code.clone(),
                    result.diagnostic.clone(),
                )
            },
        );
        let network_attempted =
            authenticated_probe_result.is_some_and(|result| result.network_attempted);
        let testnet_connection =
            authenticated_probe_result.is_some_and(|result| result.testnet_connection);
        let response_shape_validated =
            authenticated_probe_result.is_some_and(|result| result.response_shape_validated);

        Self {
            schema_version: TESTNET_AUTHENTICATED_READONLY_PROBE_SCHEMA_VERSION.to_string(),
            run_id: run_id.to_string(),
            environment: config.venue.environment.clone(),
            product: config.venue.product.clone(),
            endpoint_kind: "authenticated_http_read_only".to_string(),
            endpoint_class: authenticated_probe_result.map_or_else(
                || "binance-testnet-authenticated-readonly-account".to_string(),
                |result| result.endpoint_class.clone(),
            ),
            endpoint_url_redacted: format!(
                "{}{}",
                config.connectivity.http_base_url.trim_end_matches('/'),
                TESTNET_AUTHENTICATED_READ_ONLY_ENDPOINT,
            ),
            network_gate_status: network_gate.status.clone(),
            network_gate_reasons: network_gate.reasons.clone(),
            network_permission_requested: allow_testnet_network,
            env_network_permission: network_gate.env_network_permission,
            network_attempted,
            testnet_connection,
            credential_policy: credential_policy.policy.clone(),
            api_key_present: credential_policy.api_key_present,
            api_secret_present: credential_policy.api_secret_present,
            request_method: "GET".to_string(),
            request_target: TESTNET_AUTHENTICATED_READ_ONLY_ENDPOINT.to_string(),
            query_shape: "timestamp=<ms>&recvWindow=<ms>&signature=<redacted>".to_string(),
            api_key_header_name: BINANCE_API_KEY_HEADER.to_string(),
            api_key_header_value_recorded: false,
            signature_recorded: false,
            signed_query_recorded: false,
            signed_url_recorded: false,
            raw_response_recorded: false,
            balances_recorded: false,
            uid_recorded: false,
            account_mutation: false,
            order_submission: config.execution.order_submission.clone(),
            real_orders_submitted: false,
            production_venue_connection: false,
            real_funds: false,
            production_trading: false,
            response_status_code: authenticated_probe_result.and_then(|result| result.http_status),
            response_shape: authenticated_probe_result.map_or_else(
                || "binance_account_v1".to_string(),
                |result| result.response_shape.clone(),
            ),
            response_shape_validated,
            latency_ms: authenticated_probe_result.and_then(|result| result.latency_ms),
            error_code,
            status,
            diagnostic,
            generated_at: workflow_generated_at(),
        }
    }
}

fn workflow_generated_at() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| "unix:0".to_string(),
        |duration| format!("unix:{}", duration.as_secs()),
    )
}

fn authenticated_readonly_probe_not_attempted_result(
    mode: WorkflowRunMode,
    network_gate: &TestnetNetworkGate,
    credential_policy: &TestnetCredentialPolicy,
) -> (String, String, String) {
    if network_gate.status == "blocked" {
        return (
            "authenticated_readonly_probe_deferred".to_string(),
            "network_gate_blocked".to_string(),
            format!(
                "V080 authenticated read-only probe skipped before signed request execution: {}. No raw account body, balances, uid, header value, signature, signed URL, or order API is recorded.",
                network_gate.reasons.join("; ")
            ),
        );
    }
    if credential_policy.authenticated_read_only_probe_fail_closed {
        return (
            "authenticated_readonly_probe_blocked_missing_credentials".to_string(),
            "missing_credentials".to_string(),
            "V080 authenticated read-only probe fail-closed because env-only API key or secret is missing. No signed request is sent and no raw account body, balances, uid, header value, signature, signed URL, or order API is recorded.".to_string(),
        );
    }

    match mode {
        WorkflowRunMode::ConnectivityProbe => (
            "authenticated_readonly_probe_manual_not_run".to_string(),
            "manual_authenticated_probe_not_enabled".to_string(),
            "V080 authenticated read-only probe is manual-online-only and was not executed by this run. No raw account body, balances, uid, header value, signature, signed URL, or order API is recorded.".to_string(),
        ),
        WorkflowRunMode::DryRun => (
            "authenticated_readonly_probe_not_requested".to_string(),
            "not_requested".to_string(),
            "V080 authenticated read-only probe is not requested for dry-run mode. No signed request is sent and no raw account body, balances, uid, header value, signature, signed URL, or order API is recorded.".to_string(),
        ),
    }
}

fn websocket_probe_not_attempted_result(
    mode: WorkflowRunMode,
    network_gate: &TestnetNetworkGate,
) -> (String, String, String) {
    if network_gate.status == "blocked" {
        return (
            "websocket_read_only_probe_deferred".to_string(),
            "network_gate_blocked".to_string(),
            format!(
                "V070 WebSocket read-only probe skipped before socket creation: {}. WebSocket proof is optional/manual; no subscription, no orders, and no secrets are recorded.",
                network_gate.reasons.join("; ")
            ),
        );
    }

    match mode {
        WorkflowRunMode::ConnectivityProbe => (
            "websocket_read_only_probe_manual_not_run".to_string(),
            "manual_websocket_probe_not_enabled".to_string(),
            "V070 WebSocket read-only probe is optional/manual and is not opened by default CI. HTTP read-only probe remains the primary online connectivity proof; no subscription or order API is used.".to_string(),
        ),
        WorkflowRunMode::DryRun => (
            "websocket_read_only_probe_not_requested".to_string(),
            "not_requested".to_string(),
            "V070 WebSocket read-only probe is not requested for dry-run mode; no socket is opened, no subscription is attempted, and no orders are submitted.".to_string(),
        ),
    }
}

fn network_gate_diagnostic(
    mode: WorkflowRunMode,
    network_gate: &TestnetNetworkGate,
    credential_policy: &TestnetCredentialPolicy,
) -> String {
    if network_gate.status == "blocked" {
        return format!(
            "V070 network gate blocked before socket creation: {}. Public read-only probe does not require credentials; authenticated read-only probe is {}. No socket is opened.",
            network_gate.reasons.join("; "),
            credential_policy.authenticated_read_only_probe_status,
        );
    }

    match mode {
        WorkflowRunMode::ConnectivityProbe => {
            format!(
                "V070 network gate allowed, but V070-002 only records env-only credential policy. Public read-only probe does not require credentials; authenticated read-only probe is {}. HTTP read-only probe is implemented by V070-003. No socket is opened.",
                credential_policy.authenticated_read_only_probe_status,
            )
        }
        WorkflowRunMode::DryRun => {
            format!(
                "V070 network gate allowed for future online probes, but dry-run mode stays offline. Public read-only probe does not require credentials; authenticated read-only probe is {}. No socket is opened.",
                credential_policy.authenticated_read_only_probe_status,
            )
        }
    }
}

impl TestnetOrderLifecycle {
    fn from_config(run_id: &str, config: &TestnetWorkflowConfig) -> Self {
        Self {
            schema_version: TESTNET_ORDER_LIFECYCLE_SCHEMA_VERSION.to_string(),
            lifecycle_id: format!("binance-testnet-readonly-no-order-lifecycle-{run_id}"),
            mode: config.run.mode.clone(),
            order_submission: config.execution.order_submission.clone(),
            submitted_count: 0,
            accepted_count: 0,
            filled_count: 0,
            canceled_count: 0,
            rejected_count: 1,
            real_orders_submitted: false,
            external_venue_connection: false,
            checksum: "binance-testnet-readonly-no-real-orders".to_string(),
        }
    }
}

impl TestnetReconciliation {
    fn from_order_lifecycle(run_id: &str, lifecycle: &TestnetOrderLifecycle) -> Self {
        Self {
            schema_version: TESTNET_RECONCILIATION_SCHEMA_VERSION.to_string(),
            reconciliation_id: format!("binance-testnet-artifact-only-reconciliation-{run_id}"),
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

impl WorkflowBoundary {
    fn binance_sandbox() -> Self {
        Self {
            schema_version: BOUNDARY_SCHEMA_VERSION.to_string(),
            sandbox_only: true,
            fixture_replay: true,
            mock_execution: true,
            external_venue_connection: false,
            production_venue_connection: false,
            testnet_public_network_connection: false,
            external_network_attempted: false,
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
            production_venue_connection: false,
            testnet_public_network_connection: connectivity_probe.testnet_connection,
            external_network_attempted: connectivity_probe.network_attempted,
            real_funds: false,
            production_trading: false,
            real_orders_submitted: false,
            testnet_connection: connectivity_probe.testnet_connection,
            network_attempted: connectivity_probe.network_attempted,
            credential_policy: credential_policy.policy.clone(),
            connectivity_mode: connectivity_probe.mode.clone(),
            order_submission_mode: "disabled".to_string(),
            reconciliation_mode: "artifact-only".to_string(),
            notes: binance_testnet_boundary_notes(connectivity_probe),
        }
    }
}

fn binance_testnet_boundary_notes(connectivity_probe: &TestnetConnectivityProbe) -> Vec<String> {
    let foundation_note = if connectivity_probe.network_attempted {
        "Binance testnet workflow foundation records explicit opt-in public HTTP read-only evidence."
            .to_string()
    } else {
        "Binance testnet workflow foundation runs as offline dry-run evidence.".to_string()
    };
    let connectivity_note = if connectivity_probe.network_attempted {
        "A Binance testnet public HTTP read-only socket was opened after explicit opt-in; no Binance credential value is recorded.".to_string()
    } else {
        "No socket is opened and no Binance credential value is recorded.".to_string()
    };

    vec![
        foundation_note,
        connectivity_note,
        "No real funds, no production trading, no real orders.".to_string(),
    ]
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
            production_venue_connection: boundary.production_venue_connection,
            testnet_public_network_connection: boundary.testnet_public_network_connection,
            external_network_attempted: boundary.external_network_attempted,
            real_funds: boundary.real_funds,
            production_trading: boundary.production_trading,
            real_orders_submitted: boundary.real_orders_submitted,
            testnet_connection: boundary.testnet_connection,
            network_attempted: boundary.network_attempted,
            requested_mode: "dry-run".to_string(),
            network_permission_requested: false,
            authenticated_probe_attempted: false,
            authenticated_readonly_probe_status: "not_applicable".to_string(),
            authenticated_response_shape_validated: false,
            authenticated_connectivity_proof: false,
            credential_policy: boundary.credential_policy.clone(),
            connectivity_mode: boundary.connectivity_mode.clone(),
            order_submission_mode: boundary.order_submission_mode.clone(),
            reconciliation_mode: boundary.reconciliation_mode.clone(),
        }
    }

    fn new_binance_testnet(
        run_id: &str,
        config: &TestnetWorkflowConfig,
        connectivity_probe: &TestnetConnectivityProbe,
        authenticated_readonly_probe: &TestnetAuthenticatedReadOnlyProbe,
        order_lifecycle: &TestnetOrderLifecycle,
        reconciliation: &TestnetReconciliation,
        boundary: &WorkflowBoundary,
    ) -> Self {
        Self {
            schema_version: SUMMARY_SCHEMA_VERSION.to_string(),
            workflow_id: "binance-testnet-readonly-connectivity-foundation".to_string(),
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
            production_venue_connection: boundary.production_venue_connection,
            testnet_public_network_connection: boundary.testnet_public_network_connection,
            external_network_attempted: boundary.external_network_attempted,
            real_funds: boundary.real_funds,
            production_trading: boundary.production_trading,
            real_orders_submitted: boundary.real_orders_submitted,
            testnet_connection: boundary.testnet_connection,
            network_attempted: boundary.network_attempted,
            requested_mode: connectivity_probe.requested_mode.clone(),
            network_permission_requested: connectivity_probe.network_permission_requested,
            authenticated_probe_attempted: authenticated_readonly_probe.network_attempted,
            authenticated_readonly_probe_status: authenticated_readonly_probe.status.clone(),
            authenticated_response_shape_validated: authenticated_readonly_probe
                .response_shape_validated,
            authenticated_connectivity_proof: authenticated_readonly_probe.testnet_connection
                && authenticated_readonly_probe.response_shape_validated,
            credential_policy: boundary.credential_policy.clone(),
            connectivity_mode: config.connectivity.mode.clone(),
            order_submission_mode: config.execution.order_submission.clone(),
            reconciliation_mode: config.execution.reconciliation.clone(),
        }
    }
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
            authenticated_probe_attempted: summary.authenticated_probe_attempted,
            authenticated_readonly_probe_status: summary
                .authenticated_readonly_probe_status
                .clone(),
            authenticated_response_shape_validated: summary.authenticated_response_shape_validated,
            authenticated_connectivity_proof: summary.authenticated_connectivity_proof,
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
    production_venue_connection: bool,
    testnet_public_network_connection: bool,
    external_network_attempted: bool,
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
required_for_public_read_only_probe = false
required_for_authenticated_read_only_probe = true
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
        write_testnet_config_body(dir, &testnet_config())
    }

    fn write_testnet_config_body(dir: &Path, body: &str) -> PathBuf {
        let path = dir.join("testnet.toml");
        fs::write(&path, body).unwrap();
        path
    }

    fn parsed_testnet_config() -> TestnetWorkflowConfig {
        toml::from_str(&testnet_config()).unwrap()
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
        assert!(!summary.production_venue_connection);
        assert!(!summary.testnet_public_network_connection);
        assert!(!summary.external_network_attempted);
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
            run_id: Some("v07-readonly-smoke".to_string()),
            output: Some(output),
        })
        .unwrap();

        assert_eq!(result.workflow, "binance-testnet");
        assert_eq!(result.runtime_status, "dry_run_completed");
        assert_eq!(result.requested_mode, "dry-run");
        assert!(!result.network_permission_requested);
        assert!(!result.network_attempted);
        assert_eq!(result.artifact_paths.len(), 12);
        assert_eq!(result.artifact_paths.last(), Some(&result.manifest_path));
        assert!(!result.testnet_connection);
        assert!(!result.external_venue_connection);
        assert!(!result.production_venue_connection);
        assert!(!result.testnet_public_network_connection);
        assert!(!result.external_network_attempted);
        assert!(!result.real_orders_submitted);

        let manifest: WorkflowManifest =
            serde_json::from_str(&fs::read_to_string(&result.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.workflow, "binance-testnet");
        assert_eq!(manifest.runtime_status, "dry_run_completed");
        assert_eq!(manifest.artifact_count, 12);
        assert_eq!(manifest.summary.requested_mode, "dry-run");
        assert!(!manifest.summary.network_permission_requested);
        assert_eq!(
            manifest.summary.order_lifecycle_id,
            "binance-testnet-readonly-no-order-lifecycle-v07-readonly-smoke"
        );
        assert_eq!(manifest.summary.connectivity_mode, "dry-run");
        assert_eq!(manifest.summary.order_submission_mode, "disabled");
        assert_eq!(manifest.summary.reconciliation_mode, "artifact-only");
        assert!(!manifest.summary.testnet_connection);
        assert!(!manifest.summary.network_attempted);
        assert!(!manifest.summary.production_venue_connection);
        assert!(!manifest.summary.testnet_public_network_connection);
        assert!(!manifest.summary.external_network_attempted);
        assert!(!manifest.summary.real_orders_submitted);

        let events = fs::read_to_string(&result.events_path).unwrap();
        let parsed = events
            .lines()
            .map(serde_json::from_str::<WorkflowEvent>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(parsed.len(), 10);
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
    fn binance_testnet_artifacts_deserialize_through_shared_contract() {
        let root = temp_root("testnet-shared-contract");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceTestnet,
            mode: WorkflowRunMode::DryRun,
            config: Some(config),
            allow_testnet_network: false,
            run_id: Some("shared-contract-run".to_string()),
            output: Some(output),
        })
        .unwrap();
        let testnet_paths = TestnetArtifactPaths::new(&result.output_dir);

        let manifest: WorkflowManifest =
            serde_json::from_str(&fs::read_to_string(&result.manifest_path).unwrap()).unwrap();
        let summary: WorkflowSummary =
            serde_json::from_str(&fs::read_to_string(&result.summary_path).unwrap()).unwrap();
        let boundary: WorkflowBoundary = serde_json::from_str(
            &fs::read_to_string(result.output_dir.join("boundary.json")).unwrap(),
        )
        .unwrap();
        let config_artifact: TestnetConfigArtifact =
            serde_json::from_str(&fs::read_to_string(&testnet_paths.config).unwrap()).unwrap();
        let credential_policy: TestnetCredentialPolicy =
            serde_json::from_str(&fs::read_to_string(&testnet_paths.credential_policy).unwrap())
                .unwrap();
        let connectivity_probe: TestnetConnectivityProbe =
            serde_json::from_str(&fs::read_to_string(&testnet_paths.connectivity_probe).unwrap())
                .unwrap();
        let websocket_probe: TestnetWebSocketConnectivityProbe =
            serde_json::from_str(&fs::read_to_string(&testnet_paths.websocket_probe).unwrap())
                .unwrap();
        let authenticated_probe: TestnetAuthenticatedReadOnlyProbe = serde_json::from_str(
            &fs::read_to_string(&testnet_paths.authenticated_readonly_probe).unwrap(),
        )
        .unwrap();
        let order_lifecycle: TestnetOrderLifecycle =
            serde_json::from_str(&fs::read_to_string(&testnet_paths.order_lifecycle).unwrap())
                .unwrap();
        let reconciliation: TestnetReconciliation =
            serde_json::from_str(&fs::read_to_string(&testnet_paths.reconciliation).unwrap())
                .unwrap();
        let events = fs::read_to_string(&result.events_path).unwrap();
        let events = events
            .lines()
            .map(serde_json::from_str::<WorkflowEvent>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(manifest.schema_version, MANIFEST_SCHEMA_VERSION);
        assert_eq!(manifest.summary, summary);
        assert_eq!(manifest.artifact_count, manifest.artifacts.len());
        assert_eq!(summary.run_id, "shared-contract-run");
        assert_eq!(boundary.schema_version, BOUNDARY_SCHEMA_VERSION);
        assert!(!boundary.production_venue_connection);
        assert!(!boundary.testnet_public_network_connection);
        assert!(!boundary.external_network_attempted);
        assert!(
            boundary
                .notes
                .iter()
                .any(|note| note.contains("No socket is opened"))
        );
        assert_eq!(
            config_artifact.schema_version,
            TESTNET_CONFIG_SCHEMA_VERSION
        );
        assert_eq!(
            credential_policy.schema_version,
            TESTNET_CREDENTIAL_POLICY_SCHEMA_VERSION
        );
        assert_eq!(
            credential_policy.credential_source,
            "environment_variables_only"
        );
        assert!(!credential_policy.values_recorded);
        assert!(!credential_policy.api_key_value_recorded);
        assert!(!credential_policy.api_secret_value_recorded);
        assert!(credential_policy.secrets_redacted);
        assert!(credential_policy.legacy_required_for_network_present);
        assert_eq!(
            credential_policy.credential_config_migration_warning,
            TESTNET_CREDENTIAL_LEGACY_REQUIRED_FOR_NETWORK_WARNING
        );
        assert!(!credential_policy.required_for_public_read_only_probe);
        assert!(credential_policy.required_for_authenticated_read_only_probe);
        assert!(!credential_policy.public_read_only_probe_requires_credentials);
        assert!(credential_policy.authenticated_read_only_probe_requires_credentials);
        assert_eq!(
            credential_policy.authenticated_read_only_probe_gate,
            "manual-online-only"
        );
        assert_eq!(
            credential_policy.authenticated_read_only_probe_status,
            "manual_gate_blocked_missing_credentials"
        );
        assert!(credential_policy.authenticated_read_only_probe_fail_closed);
        assert_eq!(
            connectivity_probe.schema_version,
            TESTNET_CONNECTIVITY_PROBE_SCHEMA_VERSION
        );
        assert_eq!(
            connectivity_probe.public_read_only_probe_status,
            "available_without_credentials"
        );
        assert_eq!(
            connectivity_probe.authenticated_read_only_probe_status,
            "manual_gate_blocked_missing_credentials"
        );
        assert_eq!(
            connectivity_probe.authenticated_read_only_probe_gate,
            "manual-online-only"
        );
        assert!(connectivity_probe.authenticated_read_only_probe_requires_credentials);
        assert_eq!(
            websocket_probe.schema_version,
            TESTNET_WEBSOCKET_PROBE_SCHEMA_VERSION
        );
        assert_eq!(
            authenticated_probe.schema_version,
            TESTNET_AUTHENTICATED_READONLY_PROBE_SCHEMA_VERSION
        );
        assert_eq!(
            authenticated_probe.endpoint_kind,
            "authenticated_http_read_only"
        );
        assert_eq!(
            authenticated_probe.status,
            "authenticated_readonly_probe_deferred"
        );
        assert_eq!(authenticated_probe.error_code, "network_gate_blocked");
        assert!(!authenticated_probe.network_attempted);
        assert!(!authenticated_probe.testnet_connection);
        assert!(!authenticated_probe.response_shape_validated);
        assert!(!authenticated_probe.raw_response_recorded);
        assert!(!authenticated_probe.balances_recorded);
        assert!(!authenticated_probe.uid_recorded);
        assert!(!authenticated_probe.api_key_header_value_recorded);
        assert!(!authenticated_probe.signature_recorded);
        assert!(!authenticated_probe.signed_query_recorded);
        assert!(!authenticated_probe.signed_url_recorded);
        assert!(!authenticated_probe.account_mutation);
        assert!(!authenticated_probe.real_orders_submitted);
        assert!(!authenticated_probe.production_venue_connection);
        assert!(!authenticated_probe.real_funds);
        assert!(!authenticated_probe.production_trading);
        assert_eq!(websocket_probe.endpoint_kind, "websocket_read_only");
        assert_eq!(websocket_probe.websocket_probe_gate, "manual-online-only");
        assert!(!websocket_probe.websocket_attempted);
        assert!(!websocket_probe.network_attempted);
        assert!(!websocket_probe.testnet_connection);
        assert!(!websocket_probe.subscription_attempted);
        assert_eq!(websocket_probe.message_count, 0);
        assert_eq!(websocket_probe.order_submission, "disabled");
        assert!(!websocket_probe.real_orders_submitted);
        assert!(!websocket_probe.values_recorded);
        assert!(websocket_probe.secrets_redacted);
        assert_eq!(
            order_lifecycle.schema_version,
            TESTNET_ORDER_LIFECYCLE_SCHEMA_VERSION
        );
        assert_eq!(
            reconciliation.schema_version,
            TESTNET_RECONCILIATION_SCHEMA_VERSION
        );
        assert_eq!(events.len(), 10);
        assert!(events.iter().all(|event| event.run_id == summary.run_id));
        assert!(!connectivity_probe.network_attempted);
        assert!(!order_lifecycle.real_orders_submitted);
        assert!(!reconciliation.real_orders_submitted);
    }

    #[test]
    fn binance_testnet_websocket_probe_artifact_is_manual_optional_by_default() {
        let root = temp_root("testnet-websocket-default");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceTestnet,
            mode: WorkflowRunMode::DryRun,
            config: Some(config),
            allow_testnet_network: false,
            run_id: Some("ws-default-run".to_string()),
            output: Some(output),
        })
        .unwrap();
        let testnet_paths = TestnetArtifactPaths::new(&result.output_dir);
        let websocket_probe: TestnetWebSocketConnectivityProbe =
            serde_json::from_str(&fs::read_to_string(&testnet_paths.websocket_probe).unwrap())
                .unwrap();

        assert_eq!(
            websocket_probe.schema_version,
            TESTNET_WEBSOCKET_PROBE_SCHEMA_VERSION
        );
        assert_eq!(websocket_probe.run_id, "ws-default-run");
        assert_eq!(websocket_probe.endpoint_kind, "websocket_read_only");
        assert_eq!(
            websocket_probe.endpoint_class,
            "binance-testnet-public-websocket-handshake"
        );
        assert_eq!(websocket_probe.network_gate_status, "blocked");
        assert!(!websocket_probe.network_permission_requested);
        assert!(!websocket_probe.env_network_permission);
        assert_eq!(websocket_probe.websocket_probe_gate, "manual-online-only");
        assert!(!websocket_probe.websocket_attempted);
        assert!(!websocket_probe.network_attempted);
        assert!(!websocket_probe.testnet_connection);
        assert!(!websocket_probe.subscription_attempted);
        assert_eq!(websocket_probe.message_count, 0);
        assert_eq!(websocket_probe.order_submission, "disabled");
        assert!(!websocket_probe.real_orders_submitted);
        assert!(!websocket_probe.values_recorded);
        assert!(websocket_probe.secrets_redacted);
        assert_eq!(websocket_probe.status, "websocket_read_only_probe_deferred");
        assert_eq!(websocket_probe.error_code, "network_gate_blocked");
        assert!(websocket_probe.generated_at.starts_with("unix:"));
        assert!(
            websocket_probe
                .diagnostic
                .contains("skipped before socket creation")
        );
    }

    #[test]
    fn binance_testnet_websocket_probe_records_classified_failure_codes() {
        let config = parsed_testnet_config();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, true);
        let gate = TestnetNetworkGate::evaluate(&config, true, true);

        for error_code in ["timeout", "dns_error", "handshake_error", "protocol_error"] {
            let failure = TestnetWebSocketReadOnlyProbeResult::classified_failure(error_code);
            let probe = TestnetWebSocketConnectivityProbe::from_config(
                "ws-classified-run",
                &config,
                WorkflowRunMode::ConnectivityProbe,
                true,
                &gate,
                &policy,
                Some(&failure),
            );

            assert_eq!(probe.schema_version, TESTNET_WEBSOCKET_PROBE_SCHEMA_VERSION);
            assert_eq!(probe.endpoint_kind, "websocket_read_only");
            assert_eq!(probe.network_gate_status, "allowed");
            assert!(probe.network_permission_requested);
            assert!(probe.env_network_permission);
            assert!(probe.websocket_attempted);
            assert!(probe.network_attempted);
            assert!(!probe.testnet_connection);
            assert!(!probe.subscription_attempted);
            assert_eq!(probe.message_count, 0);
            assert_eq!(probe.order_submission, "disabled");
            assert!(!probe.real_orders_submitted);
            assert!(!probe.values_recorded);
            assert!(probe.secrets_redacted);
            assert_eq!(probe.status, "websocket_read_only_probe_failed");
            assert_eq!(probe.error_code, error_code);
            assert!(probe.generated_at.starts_with("unix:"));
            assert!(probe.diagnostic.contains(error_code));
        }
    }

    #[test]
    fn binance_testnet_authenticated_readonly_probe_records_success_without_sensitive_body() {
        let config = parsed_testnet_config();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, true);
        let gate = TestnetNetworkGate::evaluate(&config, true, true);
        let result = TestnetAuthenticatedReadOnlyProbeResult::success(55, 200);
        let probe = TestnetAuthenticatedReadOnlyProbe::from_config(
            "authenticated-success-run",
            &config,
            WorkflowRunMode::ConnectivityProbe,
            true,
            &gate,
            &policy,
            Some(&result),
        );
        let body = serde_json::to_string(&probe).unwrap();

        assert_eq!(
            probe.schema_version,
            TESTNET_AUTHENTICATED_READONLY_PROBE_SCHEMA_VERSION
        );
        assert_eq!(probe.run_id, "authenticated-success-run");
        assert_eq!(probe.endpoint_kind, "authenticated_http_read_only");
        assert_eq!(
            probe.endpoint_class,
            "binance-testnet-authenticated-readonly-account"
        );
        assert_eq!(
            probe.endpoint_url_redacted,
            "https://testnet.binance.vision/api/v3/account"
        );
        assert_eq!(probe.network_gate_status, "allowed");
        assert!(probe.network_permission_requested);
        assert!(probe.env_network_permission);
        assert!(probe.network_attempted);
        assert!(probe.testnet_connection);
        assert!(probe.api_key_present);
        assert!(probe.api_secret_present);
        assert_eq!(probe.request_method, "GET");
        assert_eq!(probe.request_target, "/api/v3/account");
        assert_eq!(
            probe.query_shape,
            "timestamp=<ms>&recvWindow=<ms>&signature=<redacted>"
        );
        assert_eq!(probe.api_key_header_name, BINANCE_API_KEY_HEADER);
        assert!(!probe.api_key_header_value_recorded);
        assert!(!probe.signature_recorded);
        assert!(!probe.signed_query_recorded);
        assert!(!probe.signed_url_recorded);
        assert!(!probe.raw_response_recorded);
        assert!(!probe.balances_recorded);
        assert!(!probe.uid_recorded);
        assert!(!probe.account_mutation);
        assert_eq!(probe.order_submission, "disabled");
        assert!(!probe.real_orders_submitted);
        assert!(!probe.production_venue_connection);
        assert!(!probe.real_funds);
        assert!(!probe.production_trading);
        assert_eq!(probe.response_status_code, Some(200));
        assert_eq!(probe.response_shape, "binance_account_v1");
        assert!(probe.response_shape_validated);
        assert_eq!(probe.latency_ms, Some(55));
        assert_eq!(probe.error_code, "none");
        assert_eq!(probe.status, "authenticated_readonly_probe_ok");
        assert!(!body.contains("balances\":["));
        assert!(!body.contains("\"uid\""));
        assert!(body.contains("signature=<redacted>"));
        assert!(!body.contains("X-MBX-APIKEY:"));
    }

    #[test]
    fn binance_testnet_authenticated_readonly_probe_records_failure_without_raw_body() {
        let config = parsed_testnet_config();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, true);
        let gate = TestnetNetworkGate::evaluate(&config, true, true);
        let result = TestnetAuthenticatedReadOnlyProbeResult::failure(
            Some(9),
            Some(401),
            "http_status_not_success",
        );
        let probe = TestnetAuthenticatedReadOnlyProbe::from_config(
            "authenticated-failure-run",
            &config,
            WorkflowRunMode::ConnectivityProbe,
            true,
            &gate,
            &policy,
            Some(&result),
        );

        assert_eq!(probe.status, "authenticated_readonly_probe_failed");
        assert_eq!(probe.error_code, "http_status_not_success");
        assert!(probe.network_attempted);
        assert!(!probe.testnet_connection);
        assert_eq!(probe.response_status_code, Some(401));
        assert!(!probe.response_shape_validated);
        assert!(!probe.raw_response_recorded);
        assert!(!probe.balances_recorded);
        assert!(!probe.uid_recorded);
        assert!(!probe.account_mutation);
        assert!(!probe.real_orders_submitted);
    }

    #[test]
    fn binance_testnet_account_response_shape_requires_account_fields_without_recording_details() {
        let valid = serde_json::json!({
            "accountType": "SPOT",
            "canTrade": true,
            "balances": [
                {"asset": "BTC", "free": "0.00000000", "locked": "0.00000000"}
            ],
            "uid": 123456
        });
        let missing_balances = serde_json::json!({
            "accountType": "SPOT",
            "canTrade": true
        });
        let wrong_type = serde_json::json!({
            "accountType": "SPOT",
            "canTrade": "yes",
            "balances": []
        });

        assert!(validates_binance_account_response_shape(&valid));
        assert!(!validates_binance_account_response_shape(&missing_balances));
        assert!(!validates_binance_account_response_shape(&wrong_type));
    }

    #[test]
    fn binance_testnet_connectivity_probe_records_offline_probe_semantics() {
        let root = temp_root("testnet-connectivity-probe");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_binance_testnet_workflow_with_env_permission(
            WorkflowRunOpt {
                workflow: WorkflowKind::BinanceTestnet,
                mode: WorkflowRunMode::ConnectivityProbe,
                config: Some(config),
                allow_testnet_network: true,
                run_id: Some("probe-run".to_string()),
                output: Some(output),
            },
            false,
        )
        .unwrap();

        assert_eq!(result.runtime_status, "offline_probe_validated");
        assert_eq!(result.requested_mode, "connectivity-probe");
        assert!(result.network_permission_requested);
        assert!(!result.network_attempted);
        assert!(!result.testnet_connection);
        assert!(!result.external_venue_connection);
        assert!(!result.production_venue_connection);
        assert!(!result.testnet_public_network_connection);
        assert!(!result.external_network_attempted);
        assert!(!result.real_orders_submitted);

        let manifest: WorkflowManifest =
            serde_json::from_str(&fs::read_to_string(&result.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.runtime_status, "offline_probe_validated");
        assert_eq!(manifest.summary.runtime_status, "offline_probe_validated");
        assert_eq!(manifest.summary.requested_mode, "connectivity-probe");
        assert!(manifest.summary.network_permission_requested);
        assert!(!manifest.summary.network_attempted);
        assert!(!manifest.summary.testnet_connection);
        assert!(!manifest.summary.production_venue_connection);
        assert!(!manifest.summary.testnet_public_network_connection);
        assert!(!manifest.summary.external_network_attempted);
        assert!(!manifest.summary.real_orders_submitted);

        let probe_path = result
            .output_dir
            .join(TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH);
        let probe: TestnetConnectivityProbe =
            serde_json::from_str(&fs::read_to_string(probe_path).unwrap()).unwrap();
        assert_eq!(probe.requested_mode, "connectivity-probe");
        assert!(probe.network_permission_requested);
        assert!(!probe.env_network_permission);
        assert_eq!(probe.network_gate_status, "blocked");
        assert_eq!(
            probe.network_gate_reasons,
            vec![format!("{TESTNET_NETWORK_OPT_IN_ENV}=1 is not set")]
        );
        assert!(!probe.network_attempted);
        assert!(!probe.testnet_connection);
        assert_eq!(probe.status, "offline_probe_validated");
        assert!(probe.diagnostic.contains("blocked before socket creation"));
        assert!(
            probe
                .diagnostic
                .contains("Public read-only probe does not require credentials")
        );
        assert!(
            probe.diagnostic.contains(
                "authenticated read-only probe is manual_gate_blocked_missing_credentials"
            )
        );
        assert!(probe.diagnostic.contains("No socket is opened"));
    }

    #[test]
    fn binance_testnet_credential_policy_records_env_presence_only() {
        let config = parsed_testnet_config();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, false);
        let serialized = serde_json::to_string(&policy).unwrap();

        assert_eq!(policy.policy, "env-var-only-no-secret-persistence");
        assert_eq!(policy.credential_source, "environment_variables_only");
        assert_eq!(policy.api_key_env, "BINANCE_TESTNET_API_KEY");
        assert_eq!(policy.api_secret_env, "BINANCE_TESTNET_API_SECRET");
        assert!(policy.api_key_present);
        assert!(!policy.api_secret_present);
        assert!(!policy.values_in_file);
        assert!(!policy.values_recorded);
        assert!(!policy.api_key_value_recorded);
        assert!(!policy.api_secret_value_recorded);
        assert!(policy.secrets_redacted);
        assert!(policy.legacy_required_for_network_present);
        assert_eq!(
            policy.credential_config_migration_warning,
            TESTNET_CREDENTIAL_LEGACY_REQUIRED_FOR_NETWORK_WARNING
        );
        assert!(!policy.required_for_public_read_only_probe);
        assert!(policy.required_for_authenticated_read_only_probe);
        assert!(!policy.public_read_only_probe_requires_credentials);
        assert!(policy.authenticated_read_only_probe_requires_credentials);
        assert_eq!(
            policy.authenticated_read_only_probe_gate,
            "manual-online-only"
        );
        assert_eq!(
            policy.authenticated_read_only_probe_status,
            "manual_gate_blocked_missing_credentials"
        );
        assert!(policy.authenticated_read_only_probe_fail_closed);
        assert!(!serialized.contains("credential_value"));
    }

    #[test]
    fn binance_testnet_credential_policy_supports_split_probe_fields_without_legacy_field() {
        let body = testnet_config().replace("required_for_network = true\n", "");
        let config: TestnetWorkflowConfig = toml::from_str(&body).unwrap();

        config.validate().unwrap();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, true);

        assert!(!policy.required_for_network);
        assert!(!policy.legacy_required_for_network_present);
        assert!(policy.credential_config_migration_warning.is_empty());
        assert!(!policy.required_for_public_read_only_probe);
        assert!(policy.required_for_authenticated_read_only_probe);
        assert!(!policy.public_read_only_probe_requires_credentials);
        assert!(policy.authenticated_read_only_probe_requires_credentials);
        assert_eq!(
            policy.authenticated_read_only_probe_status,
            "manual_gate_ready"
        );
        assert!(!policy.authenticated_read_only_probe_fail_closed);
    }

    #[test]
    fn binance_testnet_split_probe_fields_take_priority_over_legacy_required_for_network() {
        let body = testnet_config().replace(
            "required_for_network = true",
            "required_for_network = false",
        );
        let config: TestnetWorkflowConfig = toml::from_str(&body).unwrap();

        config.validate().unwrap();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, true);

        assert!(!policy.required_for_network);
        assert!(policy.legacy_required_for_network_present);
        assert!(!policy.required_for_public_read_only_probe);
        assert!(policy.required_for_authenticated_read_only_probe);
        assert!(!policy.public_read_only_probe_requires_credentials);
        assert!(policy.authenticated_read_only_probe_requires_credentials);
        assert_eq!(
            policy.credential_config_migration_warning,
            TESTNET_CREDENTIAL_LEGACY_REQUIRED_FOR_NETWORK_WARNING
        );
        assert!(!policy.authenticated_read_only_probe_fail_closed);
    }

    #[test]
    fn binance_testnet_credential_policy_fails_closed_when_secret_missing() {
        let config = parsed_testnet_config();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, false);

        assert!(policy.api_key_present);
        assert!(!policy.api_secret_present);
        assert_eq!(
            policy.authenticated_read_only_probe_status,
            "manual_gate_blocked_missing_credentials"
        );
        assert!(policy.authenticated_read_only_probe_fail_closed);
        assert!(!policy.values_recorded);
        assert!(!policy.api_key_value_recorded);
        assert!(!policy.api_secret_value_recorded);
        assert!(policy.secrets_redacted);
    }

    #[test]
    fn binance_testnet_secret_redaction_guard_blocks_synthetic_secret_leak() {
        let credentials = EnvOnlyTestnetCredentials::from_values(
            "BINANCE_TESTNET_API_KEY".to_string(),
            Some("ntpro_v080002_synthetic_api_key_value".to_string()),
            "BINANCE_TESTNET_API_SECRET".to_string(),
            Some("ntpro_v080002_synthetic_api_secret_value".to_string()),
        );
        let error = credentials
            .ensure_no_secret_values_absent(
                "synthetic-artifact",
                "credential=ntpro_v080002_synthetic_api_secret_value",
            )
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("testnet secret redaction guard blocked secret value leak")
        );
    }

    #[test]
    fn binance_testnet_credential_policy_does_not_record_synthetic_secret_values() {
        let config = parsed_testnet_config();
        let credentials = EnvOnlyTestnetCredentials::from_values(
            config.credentials.api_key_env.clone(),
            Some("ntpro_v080002_synthetic_api_key_value".to_string()),
            config.credentials.api_secret_env.clone(),
            Some("ntpro_v080002_synthetic_api_secret_value".to_string()),
        );
        let policy = TestnetCredentialPolicy::from_config_and_credentials(&config, &credentials);
        let root = temp_root("testnet-secret-redaction");
        let artifact_path = root.join("credential_policy.json");
        let mut written = Vec::new();

        write_secret_redacted_json_artifact(&artifact_path, &policy, &mut written, &credentials)
            .unwrap();

        let body = fs::read_to_string(&artifact_path).unwrap();
        assert_eq!(written, vec![artifact_path]);
        assert!(policy.api_key_present);
        assert!(policy.api_secret_present);
        assert!(!policy.authenticated_read_only_probe_fail_closed);
        assert!(!body.contains("ntpro_v080002_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v080002_synthetic_api_secret_value"));
        assert!(body.contains("\"api_key_present\": true"));
        assert!(body.contains("\"api_secret_present\": true"));
        assert!(body.contains("\"secrets_redacted\": true"));
        assert!(body.contains("\"api_key_value_recorded\": false"));
        assert!(body.contains("\"api_secret_value_recorded\": false"));
    }

    #[test]
    fn binance_testnet_signed_authenticated_get_builder_constructs_account_request() {
        let config = parsed_testnet_config();
        let credentials = EnvOnlyTestnetCredentials::from_values(
            config.credentials.api_key_env.clone(),
            Some("ntpro_v080003_synthetic_api_key_value".to_string()),
            config.credentials.api_secret_env.clone(),
            Some("ntpro_v080003_synthetic_api_secret_value".to_string()),
        );
        let request = build_testnet_authenticated_read_only_get_request(
            &config,
            &credentials,
            1_718_400_000_000,
        )
        .unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.endpoint_path, "/api/v3/account");
        assert_eq!(request.api_key_header_name, BINANCE_API_KEY_HEADER);
        assert_eq!(
            request.api_key_header_value,
            "ntpro_v080003_synthetic_api_key_value"
        );
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
            "https://testnet.binance.vision/api/v3/account"
        );
        request.ensure_preview_redacted(&credentials).unwrap();
    }

    #[test]
    fn binance_testnet_signed_authenticated_get_builder_rejects_non_get_method() {
        let config = parsed_testnet_config();
        let credentials = EnvOnlyTestnetCredentials::from_values(
            config.credentials.api_key_env.clone(),
            Some("ntpro_v080003_synthetic_api_key_value".to_string()),
            config.credentials.api_secret_env.clone(),
            Some("ntpro_v080003_synthetic_api_secret_value".to_string()),
        );
        let error = build_testnet_authenticated_read_only_request(
            &config,
            &credentials,
            "POST",
            "/api/v3/account",
            1_718_400_000_000,
            TESTNET_AUTHENTICATED_RECV_WINDOW_MS,
        )
        .unwrap_err();

        assert!(error.to_string().contains("only allows GET, got POST"));
    }

    #[test]
    fn binance_testnet_signed_authenticated_get_builder_rejects_order_endpoint() {
        let config = parsed_testnet_config();
        let credentials = EnvOnlyTestnetCredentials::from_values(
            config.credentials.api_key_env.clone(),
            Some("ntpro_v080003_synthetic_api_key_value".to_string()),
            config.credentials.api_secret_env.clone(),
            Some("ntpro_v080003_synthetic_api_secret_value".to_string()),
        );
        let error = build_testnet_authenticated_read_only_request(
            &config,
            &credentials,
            "GET",
            "/api/v3/order",
            1_718_400_000_000,
            TESTNET_AUTHENTICATED_RECV_WINDOW_MS,
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("rejects order mutation endpoint /api/v3/order")
        );
    }

    #[test]
    fn binance_testnet_signed_authenticated_get_builder_rejects_non_allowlist_endpoint() {
        let config = parsed_testnet_config();
        let credentials = EnvOnlyTestnetCredentials::from_values(
            config.credentials.api_key_env.clone(),
            Some("ntpro_v080003_synthetic_api_key_value".to_string()),
            config.credentials.api_secret_env.clone(),
            Some("ntpro_v080003_synthetic_api_secret_value".to_string()),
        );
        let error = build_testnet_authenticated_read_only_request(
            &config,
            &credentials,
            "GET",
            "/api/v3/time",
            1_718_400_000_000,
            TESTNET_AUTHENTICATED_RECV_WINDOW_MS,
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("allowlist only includes /api/v3/account")
        );
    }

    #[test]
    fn binance_testnet_signed_authenticated_get_builder_fails_closed_without_secret() {
        let config = parsed_testnet_config();
        let credentials = EnvOnlyTestnetCredentials::from_values(
            config.credentials.api_key_env.clone(),
            Some("ntpro_v080003_synthetic_api_key_value".to_string()),
            config.credentials.api_secret_env.clone(),
            None,
        );
        let error = build_testnet_authenticated_read_only_get_request(
            &config,
            &credentials,
            1_718_400_000_000,
        )
        .unwrap_err();

        assert!(error.to_string().contains("requires API secret env value"));
    }

    #[test]
    fn binance_testnet_signed_authenticated_get_builder_redacts_all_outputs() {
        let config = parsed_testnet_config();
        let credentials = EnvOnlyTestnetCredentials::from_values(
            config.credentials.api_key_env.clone(),
            Some("ntpro_v080003_synthetic_api_key_value".to_string()),
            config.credentials.api_secret_env.clone(),
            Some("ntpro_v080003_synthetic_api_secret_value".to_string()),
        );
        let request = build_testnet_authenticated_read_only_get_request(
            &config,
            &credentials,
            1_718_400_000_000,
        )
        .unwrap();
        let preview_body = serde_json::to_string(&request.redacted_preview()).unwrap();
        let debug_body = format!("{request:?}");

        for body in [&preview_body, &debug_body] {
            assert!(!body.contains("ntpro_v080003_synthetic_api_key_value"));
            assert!(!body.contains("ntpro_v080003_synthetic_api_secret_value"));
            assert!(!body.contains(&request.signature));
            assert!(!body.contains(&request.signed_query));
        }
        assert!(preview_body.contains("\"signature_recorded\":false"));
        assert!(preview_body.contains("\"signed_query_recorded\":false"));
        assert!(preview_body.contains("\"signed_url_recorded\":false"));
        assert!(preview_body.contains("\"api_key_header_value_recorded\":false"));
    }

    #[test]
    fn binance_testnet_rejects_public_read_only_probe_credentials_requirement() {
        let body = testnet_config().replace(
            "required_for_public_read_only_probe = false",
            "required_for_public_read_only_probe = true",
        );
        let config: TestnetWorkflowConfig = toml::from_str(&body).unwrap();
        let error = config.validate().unwrap_err();

        assert!(
            error
                .to_string()
                .contains("credentials.required_for_public_read_only_probe must be false")
        );
    }

    #[test]
    fn binance_testnet_rejects_disabled_authenticated_read_only_probe_credentials_requirement() {
        let body = testnet_config()
            .replace(
                "required_for_authenticated_read_only_probe = true",
                "required_for_authenticated_read_only_probe = false",
            )
            .replace("required_for_network = true\n", "");
        let config: TestnetWorkflowConfig = toml::from_str(&body).unwrap();
        let error = config.validate().unwrap_err();

        assert!(
            error
                .to_string()
                .contains("credentials.required_for_authenticated_read_only_probe must be true")
        );
    }

    #[test]
    fn binance_testnet_authenticated_probe_is_manual_ready_only_with_both_credentials_present() {
        let config = parsed_testnet_config();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, true);
        let gate = TestnetNetworkGate::evaluate(&config, true, true);
        let probe = TestnetConnectivityProbe::from_config(
            &config,
            WorkflowRunMode::ConnectivityProbe,
            true,
            &gate,
            &policy,
            None,
        );

        assert_eq!(
            policy.authenticated_read_only_probe_status,
            "manual_gate_ready"
        );
        assert!(!policy.authenticated_read_only_probe_fail_closed);
        assert_eq!(
            probe.public_read_only_probe_status,
            "available_without_credentials"
        );
        assert_eq!(
            probe.authenticated_read_only_probe_status,
            "manual_gate_ready"
        );
        assert_eq!(
            probe.authenticated_read_only_probe_gate,
            "manual-online-only"
        );
        assert!(probe.authenticated_read_only_probe_requires_credentials);
        assert_eq!(probe.network_gate_status, "allowed");
        assert!(!probe.network_attempted);
        assert!(probe.diagnostic.contains("V070-003"));
        assert!(probe.diagnostic.contains("No socket is opened"));
    }

    #[test]
    fn binance_testnet_authenticated_probe_requires_manual_online_runtime_permission() {
        let config = parsed_testnet_config();
        let credentials = EnvOnlyTestnetCredentials::from_presence(
            config.credentials.api_key_env.clone(),
            true,
            config.credentials.api_secret_env.clone(),
            true,
        );
        let policy = TestnetCredentialPolicy::from_config_and_credentials(&config, &credentials);
        let gate = TestnetNetworkGate::evaluate(&config, true, true);

        assert!(!should_attempt_testnet_authenticated_read_only_probe(
            WorkflowRunMode::ConnectivityProbe,
            &gate,
            &credentials,
            false,
        ));
        assert!(should_attempt_testnet_authenticated_read_only_probe(
            WorkflowRunMode::ConnectivityProbe,
            &gate,
            &credentials,
            true,
        ));

        let probe = TestnetAuthenticatedReadOnlyProbe::from_config(
            "authenticated-manual-gate-run",
            &config,
            WorkflowRunMode::ConnectivityProbe,
            true,
            &gate,
            &policy,
            None,
        );

        assert_eq!(
            policy.authenticated_read_only_probe_status,
            "manual_gate_ready"
        );
        assert_eq!(probe.network_gate_status, "allowed");
        assert!(!probe.network_attempted);
        assert!(!probe.testnet_connection);
        assert_eq!(probe.status, "authenticated_readonly_probe_manual_not_run");
        assert_eq!(probe.error_code, "manual_authenticated_probe_not_enabled");
        assert!(
            probe
                .diagnostic
                .contains("manual-online-only and was not executed")
        );
    }

    #[test]
    fn binance_testnet_summary_and_manifest_promote_authenticated_proof_status() {
        let config = parsed_testnet_config();
        let policy = TestnetCredentialPolicy::from_config_with_presence(&config, true, true);
        let gate = TestnetNetworkGate::evaluate(&config, true, true);
        let http_result =
            TestnetHttpReadOnlyProbeResult::success(12, 200, "binance_server_time_v1");
        let connectivity = TestnetConnectivityProbe::from_config(
            &config,
            WorkflowRunMode::ConnectivityProbe,
            true,
            &gate,
            &policy,
            Some(&http_result),
        );
        let order_lifecycle = TestnetOrderLifecycle::from_config("auth-status-run", &config);
        let reconciliation =
            TestnetReconciliation::from_order_lifecycle("auth-status-run", &order_lifecycle);
        let boundary = WorkflowBoundary::binance_testnet_dry_run(&policy, &connectivity);
        let paths = WorkflowArtifactPaths::new(Path::new("auth-status-output"));

        for (
            run_id,
            auth_probe,
            expected_attempted,
            expected_status,
            expected_shape,
            expected_proof,
        ) in [
            (
                "auth-success",
                TestnetAuthenticatedReadOnlyProbe::from_config(
                    "auth-success",
                    &config,
                    WorkflowRunMode::ConnectivityProbe,
                    true,
                    &gate,
                    &policy,
                    Some(&TestnetAuthenticatedReadOnlyProbeResult::success(55, 200)),
                ),
                true,
                "authenticated_readonly_probe_ok",
                true,
                true,
            ),
            (
                "auth-failure",
                TestnetAuthenticatedReadOnlyProbe::from_config(
                    "auth-failure",
                    &config,
                    WorkflowRunMode::ConnectivityProbe,
                    true,
                    &gate,
                    &policy,
                    Some(&TestnetAuthenticatedReadOnlyProbeResult::failure(
                        Some(9),
                        Some(401),
                        "http_status_not_success",
                    )),
                ),
                true,
                "authenticated_readonly_probe_failed",
                false,
                false,
            ),
            (
                "auth-deferred",
                TestnetAuthenticatedReadOnlyProbe::from_config(
                    "auth-deferred",
                    &config,
                    WorkflowRunMode::ConnectivityProbe,
                    true,
                    &gate,
                    &policy,
                    None,
                ),
                false,
                "authenticated_readonly_probe_manual_not_run",
                false,
                false,
            ),
        ] {
            let summary = WorkflowSummary::new_binance_testnet(
                run_id,
                &config,
                &connectivity,
                &auth_probe,
                &order_lifecycle,
                &reconciliation,
                &boundary,
            );
            let manifest = WorkflowManifest::new_with_artifacts(run_id, &paths, &summary, vec![]);

            assert_eq!(summary.authenticated_probe_attempted, expected_attempted);
            assert_eq!(summary.authenticated_readonly_probe_status, expected_status);
            assert_eq!(
                summary.authenticated_response_shape_validated,
                expected_shape
            );
            assert_eq!(summary.authenticated_connectivity_proof, expected_proof);
            assert_eq!(
                manifest.authenticated_probe_attempted,
                summary.authenticated_probe_attempted
            );
            assert_eq!(
                manifest.authenticated_readonly_probe_status,
                summary.authenticated_readonly_probe_status
            );
            assert_eq!(
                manifest.authenticated_response_shape_validated,
                summary.authenticated_response_shape_validated
            );
            assert_eq!(
                manifest.authenticated_connectivity_proof,
                summary.authenticated_connectivity_proof
            );
        }
    }

    #[test]
    fn binance_testnet_workflow_rejects_inline_credential_values() {
        let root = temp_root("testnet-inline-credential-values");
        let body = testnet_config().replace("values_in_file = false", "values_in_file = true");
        let config = write_testnet_config_body(&root, &body);
        let output = root.join("artifacts");
        let error = run_binance_testnet_workflow_with_env_permission(
            WorkflowRunOpt {
                workflow: WorkflowKind::BinanceTestnet,
                mode: WorkflowRunMode::DryRun,
                config: Some(config),
                allow_testnet_network: false,
                run_id: Some("inline-credentials-run".to_string()),
                output: Some(output),
            },
            false,
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("credentials.values_in_file must be false")
        );
    }

    #[test]
    fn binance_testnet_network_gate_blocks_without_cli_permission() {
        let config = parsed_testnet_config();
        let gate = TestnetNetworkGate::evaluate(&config, false, true);

        assert!(gate.env_network_permission);
        assert_eq!(gate.status, "blocked");
        assert_eq!(gate.reasons, vec!["missing --allow-testnet-network"]);
    }

    #[test]
    fn binance_testnet_network_gate_allows_only_read_only_testnet_config() {
        let config = parsed_testnet_config();
        let gate = TestnetNetworkGate::evaluate(&config, true, true);

        assert!(gate.env_network_permission);
        assert_eq!(gate.status, "allowed");
        assert!(gate.reasons.is_empty());
    }

    #[test]
    fn binance_testnet_dry_run_records_allowed_gate_without_network_attempt() {
        let root = temp_root("testnet-dry-run-allowed");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_binance_testnet_workflow_with_env_permission(
            WorkflowRunOpt {
                workflow: WorkflowKind::BinanceTestnet,
                mode: WorkflowRunMode::DryRun,
                config: Some(config),
                allow_testnet_network: true,
                run_id: Some("probe-allowed-run".to_string()),
                output: Some(output),
            },
            true,
        )
        .unwrap();

        assert_eq!(result.runtime_status, "dry_run_completed");
        assert!(result.network_permission_requested);
        assert!(!result.network_attempted);
        assert!(!result.testnet_connection);

        let probe_path = result
            .output_dir
            .join(TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH);
        let probe: TestnetConnectivityProbe =
            serde_json::from_str(&fs::read_to_string(probe_path).unwrap()).unwrap();
        assert!(probe.env_network_permission);
        assert_eq!(probe.network_gate_status, "allowed");
        assert!(probe.network_gate_reasons.is_empty());
        assert!(!probe.network_attempted);
        assert!(!probe.testnet_connection);
        assert!(probe.diagnostic.contains("V070 network gate allowed"));
        assert!(probe.diagnostic.contains("dry-run mode stays offline"));
    }

    #[test]
    fn binance_testnet_connectivity_probe_records_http_success_artifact() {
        let root = temp_root("testnet-http-success");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_binance_testnet_workflow_with_env_permission_and_http_probe(
            WorkflowRunOpt {
                workflow: WorkflowKind::BinanceTestnet,
                mode: WorkflowRunMode::ConnectivityProbe,
                config: Some(config),
                allow_testnet_network: true,
                run_id: Some("http-success-run".to_string()),
                output: Some(output),
            },
            true,
            |_| TestnetHttpReadOnlyProbeResult::success(42, 200, "binance_server_time_v1"),
        )
        .unwrap();

        assert_eq!(result.runtime_status, "http_read_only_probe_ok");
        assert!(result.network_permission_requested);
        assert!(result.network_attempted);
        assert!(result.testnet_connection);
        assert!(!result.real_orders_submitted);

        let probe_path = result
            .output_dir
            .join(TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH);
        let probe: TestnetConnectivityProbe =
            serde_json::from_str(&fs::read_to_string(probe_path).unwrap()).unwrap();
        assert_eq!(probe.endpoint_class, "binance-testnet-public-http-time");
        assert_eq!(probe.latency_ms, Some(42));
        assert_eq!(probe.http_status, Some(200));
        assert_eq!(probe.error_code, "none");
        assert_eq!(probe.response_shape, "binance_server_time_v1");
        assert!(probe.response_shape_validated);
        assert_eq!(probe.status, "http_read_only_probe_ok");
        assert!(probe.network_attempted);
        assert!(probe.testnet_connection);
        assert!(probe.diagnostic.contains("read-only probe succeeded"));

        let boundary: WorkflowBoundary = serde_json::from_str(
            &fs::read_to_string(result.output_dir.join("boundary.json")).unwrap(),
        )
        .unwrap();
        assert!(boundary.network_attempted);
        assert!(boundary.testnet_public_network_connection);
        assert!(!boundary.production_venue_connection);
        assert!(boundary.notes.iter().any(|note| {
            note.contains("public HTTP read-only socket was opened after explicit opt-in")
        }));
        assert!(
            boundary
                .notes
                .iter()
                .all(|note| !note.contains("No socket is opened"))
        );
        assert!(
            boundary
                .notes
                .iter()
                .any(|note| note.contains("no Binance credential value is recorded"))
        );
        assert!(
            boundary
                .notes
                .iter()
                .any(|note| note.contains("No real funds, no production trading, no real orders"))
        );
    }

    #[test]
    fn binance_server_time_response_shape_requires_positive_number() {
        let valid: BinanceServerTimeResponse =
            serde_json::from_str(r#"{"serverTime":1718400000000}"#).unwrap();
        let zero: BinanceServerTimeResponse = serde_json::from_str(r#"{"serverTime":0}"#).unwrap();

        assert!(validates_binance_server_time_response_shape(&valid));
        assert!(!validates_binance_server_time_response_shape(&zero));
        assert!(
            serde_json::from_str::<BinanceServerTimeResponse>(r#"{"serverTime":"bad"}"#).is_err()
        );
        assert!(
            serde_json::from_str::<BinanceServerTimeResponse>(r#"{"time":1718400000000}"#).is_err()
        );
    }

    #[test]
    fn binance_testnet_http_probe_shape_failure_is_not_connectivity_proof() {
        let result =
            TestnetHttpReadOnlyProbeResult::failure(Some(3), Some(200), "response_shape_invalid");

        assert_eq!(result.status, "http_read_only_probe_failed");
        assert_eq!(result.error_code, "response_shape_invalid");
        assert_eq!(result.http_status, Some(200));
        assert_eq!(result.response_shape, "binance_server_time_v1");
        assert!(!result.response_shape_validated);
        assert!(result.network_attempted);
        assert!(!result.testnet_connection);
        assert!(result.diagnostic.contains("response_shape_invalid"));
        assert!(!result.diagnostic.contains("serverTime"));
        assert!(!result.diagnostic.contains('{'));
    }

    #[test]
    fn binance_testnet_connectivity_probe_records_http_failure_artifact() {
        let root = temp_root("testnet-http-failure");
        let config = write_testnet_config(&root);
        let output = root.join("artifacts");
        let result = run_binance_testnet_workflow_with_env_permission_and_http_probe(
            WorkflowRunOpt {
                workflow: WorkflowKind::BinanceTestnet,
                mode: WorkflowRunMode::ConnectivityProbe,
                config: Some(config),
                allow_testnet_network: true,
                run_id: Some("http-failure-run".to_string()),
                output: Some(output),
            },
            true,
            |_| {
                TestnetHttpReadOnlyProbeResult::failure(
                    Some(7),
                    Some(503),
                    "http_status_not_success",
                )
            },
        )
        .unwrap();

        assert_eq!(result.runtime_status, "http_read_only_probe_failed");
        assert!(result.network_permission_requested);
        assert!(result.network_attempted);
        assert!(!result.testnet_connection);
        assert!(!result.real_orders_submitted);

        let probe_path = result
            .output_dir
            .join(TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH);
        let probe: TestnetConnectivityProbe =
            serde_json::from_str(&fs::read_to_string(probe_path).unwrap()).unwrap();
        assert_eq!(probe.endpoint_class, "binance-testnet-public-http-time");
        assert_eq!(probe.latency_ms, Some(7));
        assert_eq!(probe.http_status, Some(503));
        assert_eq!(probe.error_code, "http_status_not_success");
        assert_eq!(probe.status, "http_read_only_probe_failed");
        assert!(probe.network_attempted);
        assert!(!probe.testnet_connection);
        assert!(probe.diagnostic.contains("read-only probe attempted"));

        let boundary: WorkflowBoundary = serde_json::from_str(
            &fs::read_to_string(result.output_dir.join("boundary.json")).unwrap(),
        )
        .unwrap();
        assert!(boundary.network_attempted);
        assert!(!boundary.testnet_public_network_connection);
        assert!(!boundary.production_venue_connection);
        assert!(boundary.notes.iter().any(|note| {
            note.contains("public HTTP read-only socket was opened after explicit opt-in")
        }));
        assert!(
            boundary
                .notes
                .iter()
                .all(|note| !note.contains("No socket is opened"))
        );
        assert!(
            boundary
                .notes
                .iter()
                .any(|note| note.contains("no Binance credential value is recorded"))
        );
    }

    #[test]
    fn binance_testnet_workflow_rejects_non_testnet_environment() {
        let root = temp_root("testnet-non-testnet-environment");
        let body = testnet_config().replace("environment = \"testnet\"", "environment = \"prod\"");
        let config = write_testnet_config_body(&root, &body);
        let output = root.join("artifacts");
        let error = run_binance_testnet_workflow_with_env_permission(
            WorkflowRunOpt {
                workflow: WorkflowKind::BinanceTestnet,
                mode: WorkflowRunMode::ConnectivityProbe,
                config: Some(config),
                allow_testnet_network: true,
                run_id: Some("bad-env-run".to_string()),
                output: Some(output),
            },
            true,
        )
        .unwrap_err();

        assert!(error.to_string().contains("venue.environment"));
        assert!(error.to_string().contains("testnet"));
    }

    #[test]
    fn binance_testnet_workflow_rejects_enabled_order_submission() {
        let root = temp_root("testnet-order-submission-enabled");
        let body = testnet_config().replace(
            "order_submission = \"disabled\"",
            "order_submission = \"enabled\"",
        );
        let config = write_testnet_config_body(&root, &body);
        let output = root.join("artifacts");
        let error = run_binance_testnet_workflow_with_env_permission(
            WorkflowRunOpt {
                workflow: WorkflowKind::BinanceTestnet,
                mode: WorkflowRunMode::ConnectivityProbe,
                config: Some(config),
                allow_testnet_network: true,
                run_id: Some("bad-order-run".to_string()),
                output: Some(output),
            },
            true,
        )
        .unwrap_err();

        assert!(error.to_string().contains("execution.order_submission"));
        assert!(error.to_string().contains("disabled"));
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
            "binance-testnet-readonly-no-order-lifecycle-custom-run-id"
        );
        assert_eq!(
            manifest.summary.risk_smoke_id,
            "binance-testnet-artifact-only-reconciliation-custom-run-id"
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
