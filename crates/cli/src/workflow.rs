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

use std::path::{Path, PathBuf};

use nautilus_binance::{
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
    opt::{WorkflowCommand, WorkflowKind, WorkflowOpt, WorkflowRunOpt},
};

const WORKFLOW_ID: &str = "v05-binance-sandbox-local-workflow";
const MANIFEST_SCHEMA_VERSION: &str = "ntpro.workflow_manifest.v1";
const SUMMARY_SCHEMA_VERSION: &str = "ntpro.workflow_summary.v1";
const BOUNDARY_SCHEMA_VERSION: &str = "ntpro.workflow_boundary.v1";
const EVENT_SCHEMA_VERSION: &str = "ntpro.workflow_event.v1";
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
                "workflow.run status=ok workflow={} run_id={} output={} manifest={} summary={} events={} artifact_count={} external_venue_connection=false real_funds=false production_trading=false real_orders_submitted=false runtime_status=completed",
                result.workflow,
                result.run_id,
                result.output_dir.display(),
                result.manifest_path.display(),
                result.summary_path.display(),
                result.events_path.display(),
                result.artifact_paths.len(),
            );
        }
    }
    Ok(())
}

fn run_workflow(opt: WorkflowRunOpt) -> anyhow::Result<WorkflowRunResult> {
    match opt.workflow {
        WorkflowKind::BinanceSandbox => run_binance_sandbox_workflow(opt),
    }
}

fn run_binance_sandbox_workflow(opt: WorkflowRunOpt) -> anyhow::Result<WorkflowRunResult> {
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
        output_dir,
        manifest_path: artifact_paths.manifest,
        summary_path: artifact_paths.summary,
        events_path: artifact_paths.events,
        artifact_paths: written,
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
            notes: vec![
                "Binance sandbox-only local workflow; no real venue connection.".to_string(),
                "Uses checked-in fixture replay and mock order lifecycle evidence.".to_string(),
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
        let artifacts = [
            ("workflow.market_replay.ready", "market/replay.json"),
            ("workflow.strategy_ema.ready", "strategies/ema.json"),
            ("workflow.strategy_rsi.ready", "strategies/rsi.json"),
            ("workflow.orders.ready", "orders/mock_lifecycle.json"),
            ("workflow.risk.ready", "risk/rejection.json"),
            ("workflow.summary.ready", "summary.json"),
            ("workflow.events.ready", "events.jsonl"),
        ];
        let events = artifacts
            .into_iter()
            .enumerate()
            .map(|(index, (event_type, artifact))| WorkflowEvent {
                schema_version: EVENT_SCHEMA_VERSION.to_string(),
                workflow_id: WORKFLOW_ID.to_string(),
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
        events: &WorkflowEvents,
    ) -> Self {
        let artifacts = vec![
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.market_replay, &paths.output_dir),
                schema_version: "nautilus.binance_replay_summary.v1".to_string(),
            },
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.ema_strategy, &paths.output_dir),
                schema_version: "nautilus.v04_ema_smoke_summary.v1".to_string(),
            },
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.rsi_strategy, &paths.output_dir),
                schema_version: "nautilus.v04_rsi_smoke_summary.v1".to_string(),
            },
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.order_lifecycle, &paths.output_dir),
                schema_version: "nautilus.binance_mock_order_lifecycle_summary.v1".to_string(),
            },
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.risk_rejection, &paths.output_dir),
                schema_version: "nautilus.v04_risk_rejection_summary.v1".to_string(),
            },
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.boundary, &paths.output_dir),
                schema_version: BOUNDARY_SCHEMA_VERSION.to_string(),
            },
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.summary, &paths.output_dir),
                schema_version: SUMMARY_SCHEMA_VERSION.to_string(),
            },
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.events, &paths.output_dir),
                schema_version: EVENT_SCHEMA_VERSION.to_string(),
            },
            WorkflowManifestArtifact {
                path: relative_artifact_path(&paths.manifest, &paths.output_dir),
                schema_version: MANIFEST_SCHEMA_VERSION.to_string(),
            },
        ];

        Self {
            schema_version: MANIFEST_SCHEMA_VERSION.to_string(),
            workflow_id: WORKFLOW_ID.to_string(),
            workflow: "binance-sandbox".to_string(),
            run_id: run_id.to_string(),
            runtime_status: "completed".to_string(),
            artifact_count: events.events.len() + 2,
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
    output_dir: PathBuf,
    manifest_path: PathBuf,
    summary_path: PathBuf,
    events_path: PathBuf,
    artifact_paths: Vec<PathBuf>,
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

    #[test]
    fn workflow_run_writes_manifest_last_and_all_artifacts() {
        let output = temp_root("manifest-last");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceSandbox,
            run_id: Some("v05-test".to_string()),
            output: Some(output),
        })
        .unwrap();

        assert_eq!(result.artifact_paths.len(), 9);
        assert_eq!(result.artifact_paths.last(), Some(&result.manifest_path));
        for artifact in &result.artifact_paths {
            assert!(artifact.exists(), "{} should exist", artifact.display());
        }
    }

    #[test]
    fn workflow_summary_keeps_v04_sandbox_evidence_boundaries() {
        let output = temp_root("summary-boundary");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceSandbox,
            run_id: Some("v05-summary".to_string()),
            output: Some(output),
        })
        .unwrap();
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
    }

    #[test]
    fn workflow_events_are_valid_jsonl_and_reference_event_completion() {
        let output = temp_root("events");
        let result = run_workflow(WorkflowRunOpt {
            workflow: WorkflowKind::BinanceSandbox,
            run_id: Some("v05-events".to_string()),
            output: Some(output),
        })
        .unwrap();
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
            run_id: Some(" ".to_string()),
            output: Some(output),
        })
        .unwrap_err();

        assert!(error.to_string().contains("run_id must not be empty"));
    }
}
