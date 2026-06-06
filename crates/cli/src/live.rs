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
use nautilus_common::enums::Environment;
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
use serde::Deserialize;

use crate::opt::{LiveCommand, LiveOpt, LiveRunOpt, LiveValidateOpt};

const LIVE_INIT_SMOKE_MODE: &str = "live-init-smoke";
const SANDBOX_ENVIRONMENT: &str = "sandbox";
const SANDBOX_SIMULATED_EXECUTION: &str = "sandbox-simulated-execution";
const DISABLED_ORDER_SUBMISSION: &str = "disabled";
const START_STOP_SHUTDOWN: &str = "start-stop";

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

pub(crate) async fn run_live_command(opt: LiveOpt) -> anyhow::Result<()> {
    match opt.command {
        LiveCommand::Validate(validate) => run_live_validate(&validate),
        LiveCommand::Run(run) => run_live_run(&run).await,
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
    run_live_run_with_command(opt, "live.run", ProcessMode::TestHarness).await
}

pub(crate) async fn run_ntpro_node(
    config: PathBuf,
    run_id: Option<String>,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    run_live_run_with_command(
        &LiveRunOpt {
            config,
            run_id,
            output,
        },
        "ntpro-node.run",
        ProcessMode::SpawnedProcess,
    )
    .await
}

async fn run_live_run_with_command(
    opt: &LiveRunOpt,
    command_name: &str,
    process_mode: ProcessMode,
) -> anyhow::Result<()> {
    let config = load_minimal_live_config(&opt.config)?;
    let run_id = opt.run_id.as_deref().unwrap_or(config.run.id.as_str());
    validate_non_empty("run_id", run_id)?;

    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let summary_path = output_dir.join("summary.txt");
    let events_path = output_dir.join("events.log");
    let status_path = output_dir.join("status.json");

    let smoke = run_live_init_smoke(&config).await?;
    let status = build_node_status(
        &config,
        &opt.config,
        run_id,
        &output_dir,
        process_mode,
        &smoke,
    );

    let summary = format!(
        "command={command_name}\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\nenvironment={}\nnode_name={}\nprocess_mode={}\nadapter={}\naccount_id={}\nvenue={}\npre_start_state={}\nrunning_state={}\nfinal_state={}\naccount_cached={}\nstatus_artifact={}\nexternal_venue_connection=false\nreal_orders_submitted=false\nruntime_status=completed\nshutdown_reason=start-stop\n",
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
    );
    fs::write(&summary_path, summary)
        .with_context(|| format!("failed to write summary '{}'", summary_path.display()))?;

    let status_json = serde_json::to_string_pretty(&status)?;
    fs::write(&status_path, format!("{status_json}\n"))
        .with_context(|| format!("failed to write status '{}'", status_path.display()))?;

    let event_log = format!(
        "phase=validate_config status=ok\n\
         phase=build_node status=ok node_name={}\n\
         phase=register_adapter status=ok adapter={} venue={}\n\
         phase=start status=ok state={} account_cached={}\n\
         phase=stop status=ok state={} external_venue_connection=false real_orders_submitted=false\n",
        node_name(&config),
        config.adapter.kind,
        config.adapter.venue,
        smoke.running_state,
        smoke.account_cached,
        smoke.final_state,
    );
    fs::write(&events_path, event_log)
        .with_context(|| format!("failed to write events '{}'", events_path.display()))?;

    println!(
        "{command_name} status=ok mode={} run_id={} config={} output={} summary={} events={} status_artifact={} node_name={} adapter={} final_state={} external_venue_connection=false real_orders_submitted=false runtime_status=completed",
        config.run.mode,
        run_id,
        opt.config.display(),
        output_dir.display(),
        summary_path.display(),
        events_path.display(),
        status_path.display(),
        node_name(&config),
        config.adapter.kind,
        smoke.final_state,
    );

    Ok(())
}

pub(crate) fn validate_minimal_live_config_file(path: &Path) -> anyhow::Result<()> {
    load_minimal_live_config(path)?;
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

#[derive(Debug)]
struct LiveSmokeResult {
    pre_start_state: String,
    running_state: String,
    final_state: String,
    final_node_state: NodeState,
    account_cached: bool,
}

async fn run_live_init_smoke(config: &MinimalLiveConfig) -> anyhow::Result<LiveSmokeResult> {
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

    node.stop().await?;
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
    })
}

fn build_node_status(
    config: &MinimalLiveConfig,
    config_path: &Path,
    run_id: &str,
    output_dir: &Path,
    process_mode: ProcessMode,
    smoke: &LiveSmokeResult,
) -> NodeStatus {
    let mut status = NodeStatus::from_node_state(node_id(config, run_id), smoke.final_node_state);
    status.process_mode = process_mode;
    status.config_path = SnapshotValue::available(config_path.display().to_string());
    status.artifact_root = SnapshotValue::available(output_dir.display().to_string());
    status.previous_lifecycle_state = LifecycleStatus::Running;
    status.data_connection = ConnectionStatus::NotConfigured;
    status.execution_connection = ConnectionStatus::Disconnected;
    status.execution = ExecutionStatus {
        gateway_id: SnapshotValue::available(config.adapter.name.clone()),
        connection: ConnectionStatus::Disconnected,
        started: SnapshotValue::available(false),
        account_ref: SnapshotValue::available("configured".to_string()),
        orders_open: SnapshotValue::unknown(),
        orders_inflight: SnapshotValue::unknown(),
        orders_closed: SnapshotValue::unknown(),
        last_report_at: SnapshotValue::unknown(),
        last_reconciliation_at: SnapshotValue::unknown(),
        last_error: None,
    };
    status
}

fn node_id<'a>(config: &'a MinimalLiveConfig, run_id: &'a str) -> &'a str {
    config
        .system
        .instance_id
        .as_deref()
        .or(config.system.node_name.as_deref())
        .unwrap_or(run_id)
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

    #[test]
    fn validates_minimal_live_config() {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-drg-005-live-validate-{}",
            std::process::id()
        ));
        let path = write_config("validate", &minimal_config(&output_dir));

        validate_minimal_live_config_file(&path).unwrap();
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
        assert!(summary.contains("external_venue_connection=false"));
        assert!(summary.contains("real_orders_submitted=false"));

        let events = fs::read_to_string(output_dir.join("events.log")).unwrap();
        assert!(events.contains("phase=start status=ok"));
        assert!(events.contains("phase=stop status=ok"));

        let status: NodeStatus =
            serde_json::from_str(&fs::read_to_string(output_dir.join("status.json")).unwrap())
                .unwrap();
        assert_eq!(status.node_id, "LiveInitSmoke");
        assert_eq!(status.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(status.process_mode, ProcessMode::TestHarness);
        assert_eq!(status.execution_connection, ConnectionStatus::Disconnected);
        assert!(!status.external_venue_connection);
        assert!(!status.real_orders_submitted);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_ntpro_node_writes_spawned_process_status() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-v02-004-node-run-{}", std::process::id()));
        let path = write_config("ntpro-node", &minimal_config(&output_dir));

        run_ntpro_node(path, Some("sandbox-a".to_string()), None)
            .await
            .unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("command=ntpro-node.run"));
        assert!(summary.contains("process_mode=spawned_process"));
        assert!(summary.contains("final_state=Stopped"));

        let status: NodeStatus =
            serde_json::from_str(&fs::read_to_string(output_dir.join("status.json")).unwrap())
                .unwrap();
        assert_eq!(status.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(status.process_mode, ProcessMode::SpawnedProcess);
        assert_eq!(
            status.config_path.availability,
            nautilus_live::status::SnapshotAvailability::Available
        );
        assert!(!status.external_venue_connection);
        assert!(!status.real_orders_submitted);
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
