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
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
use tokio::time::{sleep, timeout};

use crate::{
    artifacts::{atomic_write_json, atomic_write_text},
    opt::{LiveCommand, LiveOpt, LiveRunOpt, LiveValidateOpt},
    process::process_is_alive,
    strategy_session::{StrategySession, ema_cross_demo_fixture_bars},
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
struct StrategyNodeRiskSection {
    kill_switch: bool,
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
    run_live_run_with_command(
        opt,
        "live.run",
        ProcessMode::TestHarness,
        None,
        NtproNodeRunControls::default(),
    )
    .await
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
    let bars = ema_cross_demo_fixture_bars(symbol);
    let runtime = session.run_ema_cross_demo(&bars)?;

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
            signal_count: runtime.signal_count,
            rejection_count: runtime.risk_decision_count,
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
        "command={command_name}\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\nenvironment=sandbox\nprocess_mode={}\nstrategy_id={}\nstrategy_runtime={}\nmarket_source={}\nmarket_symbol={symbol}\nprocessed_events={}\nsignal_count={}\norder_intent_count={}\nrisk_decision_count={}\norder_submission_allowed=false\nstatus_artifact={}\nmetrics_artifact={}\nevents_log={}\nsession_status_artifact={}\nsignal_artifact={}\norder_intent_artifact={}\nrisk_decision_artifact={}\nstrategy_summary_artifact={}\nfinal_state=Stopped\nexternal_venue_connection=false\nreal_orders_submitted=false\nruntime_status=completed\nshutdown_reason={}\n",
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
        runtime.signal_count,
        runtime.order_intent_count,
        runtime.risk_decision_count,
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
         phase=strategy_loop status=ok signal_count={} order_intent_count={} risk_decision_count={}\n\
         phase=shutdown_trigger status=ok reason={}\n\
         phase=strategy_session_stop status=ok state=stopped external_venue_connection=false real_orders_submitted=false\n",
        config.node.mode,
        config.strategy.strategy_id,
        config.strategy.strategy_id,
        runtime.processed_events,
        runtime.signal_count,
        runtime.order_intent_count,
        runtime.risk_decision_count,
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
    if !config.risk.kill_switch {
        anyhow::bail!("risk.kill_switch must be true for v0.9.1 shadow strategy sessions");
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
    signal_count: u64,
    rejection_count: u64,
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
    status.data_connection = ConnectionStatus::Connected;
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
    status.risk.command_count = SnapshotValue::available(context.signal_count);
    status.risk.event_count = SnapshotValue::available(context.signal_count);
    status.risk.rejections_total = SnapshotValue::available(context.rejection_count);
    if context.rejection_count > 0 {
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
                    signal_count: 0,
                    rejection_count: 0,
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

[risk]
kill_switch = true

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
        assert_eq!(status.data_connection, ConnectionStatus::Connected);
        assert_eq!(status.execution_connection, ConnectionStatus::NotConfigured);
        assert!(!status.external_venue_connection);
        assert!(!status.real_orders_submitted);

        let session_status: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(output_dir.join("strategy").join("session_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(session_status["state"], "stopped");
        assert_eq!(session_status["session_id"], "btc-ema-shadow-001");
        assert_eq!(session_status["strategy_id"], "ema_cross_btcusdt_v1");

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
        let status_path = output_dir.join("strategy").join("session_status.json");
        let stop_file_writer = stop_file.clone();
        let watcher = tokio::spawn(async move {
            for _ in 0..40 {
                if status_path.exists() {
                    let status: serde_json::Value =
                        serde_json::from_str(&fs::read_to_string(&status_path)?)?;
                    if status["state"] == "running" {
                        fs::write(&stop_file_writer, "stop\n")?;
                        return Ok::<_, anyhow::Error>(());
                    }
                }
                sleep(Duration::from_millis(50)).await;
            }
            anyhow::bail!("strategy session did not remain running before shutdown")
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
