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

//! Live and sandbox node runtime lifecycle.
//!
//! This module owns node start/stop orchestration, heartbeat, status, metrics,
//! and shutdown handling. CLI parsing and historical artifact evaluation remain
//! outside this boundary.

use super::*;

pub(super) async fn run_live_run_with_command(
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

pub(super) async fn run_strategy_session_node_with_command(
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
