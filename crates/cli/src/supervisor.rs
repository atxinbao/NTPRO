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
    collections::BTreeMap,
    fmt::Display,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{self, Child, Command, Stdio},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, ensure};
use nautilus_live::status::{
    ConnectionStatus, LifecycleStatus, NodeStatus, ProcessMode, SnapshotValue,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    artifacts::{atomic_write_json, atomic_write_text, remove_file_if_exists},
    opt::{
        SupervisorCommand, SupervisorListOpt, SupervisorNodeOpt, SupervisorOpt,
        SupervisorRegisterOpt, SupervisorStartOpt, SupervisorStopOpt,
    },
    process::{
        SignalDelivery, process_is_alive, send_kill, send_termination, wait_for_process_exit,
    },
    strategy_session::audit_strategy_session_artifacts,
};

pub const SUPERVISOR_REGISTRY_SCHEMA_VERSION: &str = "ntpro.supervisor_registry.v1";
pub const SUPERVISOR_REGISTRY_LOCK_SCHEMA_VERSION: &str = "ntpro.supervisor_registry_lock.v1";
pub const NODE_METRICS_SCHEMA_VERSION: &str = "ntpro.node_metrics.v1";
pub const PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION: &str =
    "ntpro.v130_kill_switch_approval_artifact.v1";
const KILL_SWITCH_APPROVAL_ARTIFACT_RELATIVE_PATH: &str =
    "v0_13/kill_switch_approval_artifact.json";
const REGISTRY_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const REGISTRY_LOCK_RETRY: Duration = Duration::from_millis(25);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(100);
const PROCESS_SIGNAL_GRACE: Duration = Duration::from_secs(1);
const DATA_RECONNECT_UNSUPPORTED_MESSAGE: &str =
    "data source reconnect is not supported for local sandbox-only supervisor artifacts";
const EXECUTION_RECONNECT_UNSUPPORTED_MESSAGE: &str =
    "execution gateway reconnect is not supported for local sandbox-only supervisor artifacts";
const SHADOW_PREFLIGHT_SESSION_RELATIVE_PATH: &str = "v0_14/shadow_preflight_session.jsonl";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisorRegistry {
    pub schema_version: String,
    pub nodes: BTreeMap<String, SupervisorNodeRecord>,
    pub updated_at: SnapshotValue<String>,
}

impl Default for SupervisorRegistry {
    fn default() -> Self {
        Self {
            schema_version: SUPERVISOR_REGISTRY_SCHEMA_VERSION.to_string(),
            nodes: BTreeMap::new(),
            updated_at: SnapshotValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisorNodeRecord {
    pub node_id: String,
    pub config_path: PathBuf,
    pub artifact_root: PathBuf,
    pub pid_path: PathBuf,
    pub status_path: PathBuf,
    pub metrics_path: PathBuf,
    pub stdout_log_path: PathBuf,
    pub stderr_log_path: PathBuf,
    pub events_log_path: PathBuf,
    pub process: SupervisorProcessRecord,
    pub last_known_status: NodeStatus,
    pub status_artifact: RegistryArtifactState,
    #[serde(default)]
    pub metrics_artifact: RegistryArtifactState,
    pub updated_at: SnapshotValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisorProcessRecord {
    pub pid: SnapshotValue<u32>,
    pub state: SupervisorProcessState,
    pub updated_at: SnapshotValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisorPidArtifact {
    pub node_id: String,
    pub pid: u32,
    pub state: SupervisorProcessState,
    pub updated_at: SnapshotValue<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_identity: Option<SupervisorProcessIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisorProcessIdentity {
    pub node_id: String,
    pub artifact_root: String,
    pub status_path: String,
    pub started_at: SnapshotValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub schema_version: String,
    pub node_id: String,
    pub lifecycle_state: LifecycleStatus,
    pub previous_lifecycle_state: LifecycleStatus,
    pub process_mode: ProcessMode,
    pub uptime_ms: SnapshotValue<u64>,
    pub starts_total: u64,
    pub stops_total: u64,
    pub state_transitions_total: u64,
    pub connection_counts: NodeConnectionCounts,
    pub last_error_summary: Option<String>,
    pub generated_at: SnapshotValue<String>,
    pub started_at: SnapshotValue<String>,
    pub stopped_at: SnapshotValue<String>,
    pub status_artifact_path: SnapshotValue<String>,
    pub stdout_log_path: SnapshotValue<String>,
    pub stderr_log_path: SnapshotValue<String>,
    pub events_log_path: SnapshotValue<String>,
    #[serde(default)]
    pub strategy_signal_count: SnapshotValue<u64>,
    #[serde(default)]
    pub strategy_rejection_count: SnapshotValue<u64>,
    #[serde(default)]
    pub kill_switch_dry_run: KillSwitchDryRunMetrics,
    pub external_venue_connection: bool,
    pub real_orders_submitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillSwitchDryRunMetrics {
    pub artifact_path: SnapshotValue<String>,
    pub artifact_status: SnapshotValue<String>,
    pub kill_switch_active: SnapshotValue<bool>,
    pub kill_switch_dry_run: SnapshotValue<bool>,
    pub manual_approval_recorded: SnapshotValue<bool>,
    pub approval_state: SnapshotValue<String>,
    pub production_order_submission_allowed: SnapshotValue<bool>,
    pub production_order_mutation_allowed: SnapshotValue<bool>,
    pub production_order_state_reads_allowed: SnapshotValue<bool>,
    pub listen_key_lifecycle_allowed: SnapshotValue<bool>,
    pub production_order_submissions_attempted: SnapshotValue<u64>,
    pub production_orders_submitted: SnapshotValue<u64>,
    pub production_order_mutations_attempted: SnapshotValue<u64>,
    pub production_order_state_reads_attempted: SnapshotValue<u64>,
    pub dashboard_order_controls_enabled: SnapshotValue<bool>,
    pub real_orders_submitted: SnapshotValue<bool>,
    pub network_attempted: SnapshotValue<bool>,
    pub values_are_exchange_truth: SnapshotValue<bool>,
}

impl Default for KillSwitchDryRunMetrics {
    fn default() -> Self {
        Self {
            artifact_path: SnapshotValue::unknown(),
            artifact_status: SnapshotValue::unknown(),
            kill_switch_active: SnapshotValue::unknown(),
            kill_switch_dry_run: SnapshotValue::unknown(),
            manual_approval_recorded: SnapshotValue::unknown(),
            approval_state: SnapshotValue::unknown(),
            production_order_submission_allowed: SnapshotValue::unknown(),
            production_order_mutation_allowed: SnapshotValue::unknown(),
            production_order_state_reads_allowed: SnapshotValue::unknown(),
            listen_key_lifecycle_allowed: SnapshotValue::unknown(),
            production_order_submissions_attempted: SnapshotValue::unknown(),
            production_orders_submitted: SnapshotValue::unknown(),
            production_order_mutations_attempted: SnapshotValue::unknown(),
            production_order_state_reads_attempted: SnapshotValue::unknown(),
            dashboard_order_controls_enabled: SnapshotValue::unknown(),
            real_orders_submitted: SnapshotValue::unknown(),
            network_attempted: SnapshotValue::unknown(),
            values_are_exchange_truth: SnapshotValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeConnectionCounts {
    pub data_connected: u64,
    pub data_disconnected: u64,
    pub data_not_configured: u64,
    pub execution_connected: u64,
    pub execution_disconnected: u64,
    pub execution_not_configured: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeMetricArtifacts {
    pub status_path: PathBuf,
    pub stdout_log_path: PathBuf,
    pub stderr_log_path: PathBuf,
    pub events_log_path: PathBuf,
    pub kill_switch_approval_artifact_path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeMetricCounts {
    pub uptime_ms: Option<u64>,
    pub starts_total: u64,
    pub stops_total: u64,
    pub state_transitions_total: u64,
}

impl NodeMetrics {
    #[must_use]
    pub fn from_status(
        status: &NodeStatus,
        artifacts: &NodeMetricArtifacts,
        counts: NodeMetricCounts,
    ) -> Self {
        Self {
            schema_version: NODE_METRICS_SCHEMA_VERSION.to_string(),
            node_id: status.node_id.clone(),
            lifecycle_state: status.lifecycle_state,
            previous_lifecycle_state: status.previous_lifecycle_state,
            process_mode: status.process_mode,
            uptime_ms: counts
                .uptime_ms
                .map_or_else(SnapshotValue::unknown, SnapshotValue::available),
            starts_total: counts.starts_total,
            stops_total: counts.stops_total,
            state_transitions_total: counts.state_transitions_total,
            connection_counts: NodeConnectionCounts::from_status(status),
            last_error_summary: status
                .last_error
                .clone()
                .or_else(|| status.execution.last_error.clone())
                .or_else(|| status.risk.last_error.clone()),
            generated_at: status.generated_at.clone(),
            started_at: status.started_at.clone(),
            stopped_at: status.stopped_at.clone(),
            status_artifact_path: SnapshotValue::available(
                artifacts.status_path.display().to_string(),
            ),
            stdout_log_path: SnapshotValue::available(
                artifacts.stdout_log_path.display().to_string(),
            ),
            stderr_log_path: SnapshotValue::available(
                artifacts.stderr_log_path.display().to_string(),
            ),
            events_log_path: SnapshotValue::available(
                artifacts.events_log_path.display().to_string(),
            ),
            strategy_signal_count: status.risk.command_count.clone(),
            strategy_rejection_count: status.risk.rejections_total.clone(),
            kill_switch_dry_run: kill_switch_dry_run_metrics_from_artifact_path(
                &artifacts.kill_switch_approval_artifact_path,
            ),
            external_venue_connection: status.external_venue_connection,
            real_orders_submitted: status.real_orders_submitted,
        }
    }
}

impl NodeConnectionCounts {
    #[must_use]
    pub fn from_status(status: &NodeStatus) -> Self {
        Self {
            data_connected: count_connection(status.data_connection, ConnectionStatus::Connected),
            data_disconnected: count_connection(
                status.data_connection,
                ConnectionStatus::Disconnected,
            ),
            data_not_configured: count_connection(
                status.data_connection,
                ConnectionStatus::NotConfigured,
            ),
            execution_connected: count_connection(
                status.execution_connection,
                ConnectionStatus::Connected,
            ),
            execution_disconnected: count_connection(
                status.execution_connection,
                ConnectionStatus::Disconnected,
            ),
            execution_not_configured: count_connection(
                status.execution_connection,
                ConnectionStatus::NotConfigured,
            ),
        }
    }
}

impl NodeMetricArtifacts {
    #[must_use]
    pub fn from_record(record: &SupervisorNodeRecord) -> Self {
        Self {
            status_path: record.status_path.clone(),
            stdout_log_path: record.stdout_log_path.clone(),
            stderr_log_path: record.stderr_log_path.clone(),
            events_log_path: record.events_log_path.clone(),
            kill_switch_approval_artifact_path: kill_switch_approval_artifact_path(record),
        }
    }
}

fn kill_switch_approval_artifact_path(record: &SupervisorNodeRecord) -> PathBuf {
    record
        .artifact_root
        .join(KILL_SWITCH_APPROVAL_ARTIFACT_RELATIVE_PATH)
}

fn kill_switch_dry_run_metrics_from_artifact_path(path: &Path) -> KillSwitchDryRunMetrics {
    let mut metrics = KillSwitchDryRunMetrics {
        artifact_path: SnapshotValue::available(path.display().to_string()),
        artifact_status: SnapshotValue::not_configured(),
        ..KillSwitchDryRunMetrics::default()
    };
    if !path.exists() {
        return metrics;
    }

    let Ok(raw) = fs::read_to_string(path) else {
        metrics.artifact_status = SnapshotValue::available("read_error".to_string());
        return metrics;
    };
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        metrics.artifact_status = SnapshotValue::available("invalid_json".to_string());
        return metrics;
    };
    if value.get("schema_version").and_then(Value::as_str)
        != Some(PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION)
    {
        metrics.artifact_status = SnapshotValue::available("invalid_schema".to_string());
        return metrics;
    }

    metrics.artifact_status = snapshot_string_field(&value, "status");
    metrics.kill_switch_active = snapshot_bool_field(&value, "kill_switch_active");
    metrics.kill_switch_dry_run = snapshot_bool_field(&value, "kill_switch_dry_run");
    metrics.manual_approval_recorded = snapshot_bool_field(&value, "manual_approval_recorded");
    metrics.approval_state = snapshot_string_field(&value, "approval_state");
    metrics.production_order_submission_allowed =
        snapshot_bool_field(&value, "production_order_submission_allowed");
    metrics.production_order_mutation_allowed =
        snapshot_bool_field(&value, "production_order_mutation_allowed");
    metrics.production_order_state_reads_allowed =
        snapshot_bool_field(&value, "production_order_state_reads_allowed");
    metrics.listen_key_lifecycle_allowed =
        snapshot_bool_field(&value, "listen_key_lifecycle_allowed");
    metrics.production_order_submissions_attempted =
        snapshot_u64_field(&value, "production_order_submissions_attempted");
    metrics.production_orders_submitted = snapshot_u64_field(&value, "production_orders_submitted");
    metrics.production_order_mutations_attempted =
        snapshot_u64_field(&value, "production_order_mutations_attempted");
    metrics.production_order_state_reads_attempted =
        snapshot_u64_field(&value, "production_order_state_reads_attempted");
    metrics.dashboard_order_controls_enabled =
        snapshot_bool_field(&value, "dashboard_order_controls_enabled");
    metrics.real_orders_submitted = snapshot_bool_field(&value, "real_orders_submitted");
    metrics.network_attempted = snapshot_bool_field(&value, "network_attempted");
    metrics.values_are_exchange_truth = snapshot_bool_field(&value, "values_are_exchange_truth");
    metrics
}

fn snapshot_string_field(value: &Value, field: &str) -> SnapshotValue<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .map_or_else(SnapshotValue::unknown, SnapshotValue::available)
}

fn snapshot_bool_field(value: &Value, field: &str) -> SnapshotValue<bool> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .map_or_else(SnapshotValue::unknown, SnapshotValue::available)
}

fn snapshot_u64_field(value: &Value, field: &str) -> SnapshotValue<u64> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .map_or_else(SnapshotValue::unknown, SnapshotValue::available)
}

fn count_connection(actual: ConnectionStatus, expected: ConnectionStatus) -> u64 {
    u64::from(actual == expected)
}

/// Runs local supervisor controls through registry and node artifacts.
///
/// # Errors
///
/// Returns an error if the registry cannot be read/written, the requested node
/// is missing, a node process cannot be started/stopped, or an artifact is
/// missing or invalid.
pub(crate) fn run_supervisor_command(opt: SupervisorOpt) -> anyhow::Result<()> {
    match opt.command {
        SupervisorCommand::Register(register) => run_supervisor_register(register),
        SupervisorCommand::List(list) => run_supervisor_list(list),
        SupervisorCommand::Start(start) => run_supervisor_start(start),
        SupervisorCommand::Stop(stop) => run_supervisor_stop(stop),
        SupervisorCommand::Pause(node) => run_supervisor_pause(node),
        SupervisorCommand::Resume(node) => run_supervisor_resume(node),
        SupervisorCommand::ReconnectData(node) => run_supervisor_reconnect_data(node),
        SupervisorCommand::ReconnectExecution(node) => run_supervisor_reconnect_execution(node),
        SupervisorCommand::Status(node) => run_supervisor_status(node),
        SupervisorCommand::Connections(node) => run_supervisor_connections(node),
        SupervisorCommand::Execution(node) => run_supervisor_execution(node),
        SupervisorCommand::Risk(node) => run_supervisor_risk(node),
        SupervisorCommand::Logs(node) => run_supervisor_logs(node),
        SupervisorCommand::Metrics(node) => run_supervisor_metrics(node),
        SupervisorCommand::ShadowRuntime(node) => run_supervisor_shadow_runtime(node),
    }
}

fn run_supervisor_register(opt: SupervisorRegisterOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let record = store.register_node(RegisterNodeRequest {
        node_id: opt.node_id,
        config_path: opt.config,
        artifact_root: opt.artifact_root,
    })?;
    println!(
        "supervisor.register status=ok node_id={} registry={} artifact_root={} status_artifact={} metrics_artifact={}",
        record.node_id,
        store.registry_path().display(),
        record.artifact_root.display(),
        json_label(&record.status_artifact),
        json_label(&record.metrics_artifact),
    );
    Ok(())
}

fn run_supervisor_list(opt: SupervisorListOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let nodes = store.list_nodes()?;
    let node_ids = nodes
        .iter()
        .map(|record| record.node_id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "supervisor.list status=ok registry={} nodes={} node_ids={}",
        store.registry_path().display(),
        nodes.len(),
        node_ids,
    );
    Ok(())
}

fn run_supervisor_start(opt: SupervisorStartOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let request = StartNodeRequest {
        node_id: opt.node_id,
        ntpro_node_bin: opt.ntpro_node_bin,
        startup_timeout: Duration::from_millis(opt.startup_timeout_ms),
        node_max_runtime: Duration::from_millis(opt.node_max_runtime_ms),
        node_heartbeat_interval: Duration::from_millis(opt.node_heartbeat_interval_ms),
        node_parent_pid: opt.node_parent_pid,
        node_shutdown_timeout: Duration::from_millis(opt.node_shutdown_timeout_ms),
    };
    let record = store.start_node_process(&request)?;
    println!(
        "supervisor.start status=ok node_id={} process_state={} lifecycle_state={} pid={} external_venue_connection=false real_orders_submitted=false",
        record.node_id,
        json_label(&record.process.state),
        json_label(&record.last_known_status.lifecycle_state),
        snapshot_display(&record.process.pid),
    );
    Ok(())
}

fn run_supervisor_stop(opt: SupervisorStopOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let request = StopNodeRequest {
        node_id: opt.node_id,
        stop_timeout: Duration::from_millis(opt.stop_timeout_ms),
    };
    let record = store.stop_node_process(&request)?;
    println!(
        "supervisor.stop status=ok node_id={} process_state={} lifecycle_state={} external_venue_connection=false real_orders_submitted=false",
        record.node_id,
        json_label(&record.process.state),
        json_label(&record.last_known_status.lifecycle_state),
    );
    Ok(())
}

fn run_supervisor_pause(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let record = store.pause_node(&opt.node_id)?;
    println!(
        "supervisor.pause status=ok node_id={} process_state={} lifecycle_state={} previous_lifecycle_state={} external_venue_connection=false real_orders_submitted=false",
        record.node_id,
        json_label(&record.process.state),
        json_label(&record.last_known_status.lifecycle_state),
        json_label(&record.last_known_status.previous_lifecycle_state),
    );
    Ok(())
}

fn run_supervisor_resume(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let record = store.resume_node(&opt.node_id)?;
    println!(
        "supervisor.resume status=ok node_id={} process_state={} lifecycle_state={} previous_lifecycle_state={} external_venue_connection=false real_orders_submitted=false",
        record.node_id,
        json_label(&record.process.state),
        json_label(&record.last_known_status.lifecycle_state),
        json_label(&record.last_known_status.previous_lifecycle_state),
    );
    Ok(())
}

fn run_supervisor_reconnect_data(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let record = store.reconnect_data_source(&opt.node_id)?;
    println!(
        "supervisor.reconnect_data status=not_supported node_id={} process_state={} lifecycle_state={} data_connection={} external_venue_connection=false real_orders_submitted=false reason=data_source_reconnect_not_supported_for_local_sandbox",
        record.node_id,
        json_label(&record.process.state),
        json_label(&record.last_known_status.lifecycle_state),
        json_label(&record.last_known_status.data_connection),
    );
    Ok(())
}

fn run_supervisor_reconnect_execution(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let record = store.reconnect_execution_gateway(&opt.node_id)?;
    println!(
        "supervisor.reconnect_execution status=not_supported node_id={} process_state={} lifecycle_state={} execution_connection={} external_venue_connection=false real_orders_submitted=false reason=execution_gateway_reconnect_not_supported_for_local_sandbox",
        record.node_id,
        json_label(&record.process.state),
        json_label(&record.last_known_status.lifecycle_state),
        json_label(&record.last_known_status.execution_connection),
    );
    Ok(())
}

fn run_supervisor_status(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let status = store.node_status(&opt.node_id)?;
    let strategy = strategy_session_status_from_node_status(&status);
    println!(
        "supervisor.status status=ok registry_node_id={} runtime_node_id={} lifecycle_state={} previous_lifecycle_state={} process_mode={} generated_at={} strategy_session_state={} strategy_id={} market_state={} risk_state={} last_signal_at={} last_rejection_reason={} strategy_health={} strategy_manifest={} strategy_diagnostic={} strategy_session_status={} strategy_events={} strategy_summary={} external_venue_connection={} real_orders_submitted={} last_error={}",
        opt.node_id,
        status.node_id,
        json_label(&status.lifecycle_state),
        json_label(&status.previous_lifecycle_state),
        json_label(&status.process_mode),
        snapshot_display(&status.generated_at),
        strategy.session_state,
        strategy.strategy_id,
        strategy.market_state,
        strategy.risk_state,
        strategy.last_signal_at,
        strategy.last_rejection_reason,
        strategy.artifact_health,
        strategy.manifest_path,
        strategy.artifact_diagnostic.replace(' ', "_"),
        strategy.session_status_path,
        strategy.events_path,
        strategy.summary_path,
        status.external_venue_connection,
        status.real_orders_submitted,
        status.last_error.as_deref().unwrap_or("none"),
    );
    Ok(())
}

fn run_supervisor_connections(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let status = store.node_status(&opt.node_id)?;
    println!(
        "supervisor.connections status=ok registry_node_id={} runtime_node_id={} data_connection={} execution_connection={} external_venue_connection={} real_orders_submitted={}",
        opt.node_id,
        status.node_id,
        json_label(&status.data_connection),
        json_label(&status.execution_connection),
        status.external_venue_connection,
        status.real_orders_submitted,
    );
    Ok(())
}

fn run_supervisor_execution(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let status = store.node_status(&opt.node_id)?;
    println!(
        "supervisor.execution status=ok registry_node_id={} runtime_node_id={} gateway_id={} connection={} started={} account_ref={} orders_open={} orders_inflight={} orders_closed={} last_error={}",
        opt.node_id,
        status.node_id,
        snapshot_display(&status.execution.gateway_id),
        json_label(&status.execution.connection),
        snapshot_display(&status.execution.started),
        snapshot_display(&status.execution.account_ref),
        snapshot_display(&status.execution.orders_open),
        snapshot_display(&status.execution.orders_inflight),
        snapshot_display(&status.execution.orders_closed),
        status.execution.last_error.as_deref().unwrap_or("none"),
    );
    Ok(())
}

fn run_supervisor_risk(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let status = store.node_status(&opt.node_id)?;
    let record = load_node_record(&store, &opt.node_id)?;
    let kill_switch = kill_switch_dry_run_metrics_from_artifact_path(
        &kill_switch_approval_artifact_path(&record),
    );
    println!(
        "supervisor.risk status=ok registry_node_id={} runtime_node_id={} trading_state={} health={} command_count={} event_count={} rejections_total={} kill_switch_artifact_status={} kill_switch_active={} kill_switch_dry_run={} manual_approval_recorded={} production_order_mutation_allowed={} dashboard_order_controls_enabled={} last_rejection={} last_error={}",
        opt.node_id,
        status.node_id,
        json_label(&status.risk.trading_state),
        json_label(&status.risk.health),
        snapshot_display(&status.risk.command_count),
        snapshot_display(&status.risk.event_count),
        snapshot_display(&status.risk.rejections_total),
        snapshot_display(&kill_switch.artifact_status),
        snapshot_display(&kill_switch.kill_switch_active),
        snapshot_display(&kill_switch.kill_switch_dry_run),
        snapshot_display(&kill_switch.manual_approval_recorded),
        snapshot_display(&kill_switch.production_order_mutation_allowed),
        snapshot_display(&kill_switch.dashboard_order_controls_enabled),
        status.risk.last_rejection.as_deref().unwrap_or("none"),
        status.risk.last_error.as_deref().unwrap_or("none"),
    );
    Ok(())
}

fn run_supervisor_logs(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let record = load_node_record(&store, &opt.node_id)?;
    let strategy = strategy_session_status_from_node_status(&record.last_known_status);
    println!(
        "supervisor.logs status=ok node_id={} stdout_log={} stderr_log={} events_log={} strategy_events={} strategy_summary={}",
        record.node_id,
        record.stdout_log_path.display(),
        record.stderr_log_path.display(),
        record.events_log_path.display(),
        strategy.events_path,
        strategy.summary_path,
    );
    Ok(())
}

fn run_supervisor_metrics(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let metrics = store.node_metrics(&opt.node_id)?;
    println!(
        "supervisor.metrics status=ok registry_node_id={} runtime_node_id={} lifecycle_state={} starts_total={} stops_total={} state_transitions_total={} uptime_ms={} strategy_signal_count={} strategy_rejection_count={} kill_switch_artifact_status={} kill_switch_active={} kill_switch_dry_run={} production_order_submissions_attempted={} production_order_mutations_attempted={} dashboard_order_controls_enabled={} external_venue_connection={} real_orders_submitted={} last_error={}",
        opt.node_id,
        metrics.node_id,
        json_label(&metrics.lifecycle_state),
        metrics.starts_total,
        metrics.stops_total,
        metrics.state_transitions_total,
        snapshot_display(&metrics.uptime_ms),
        snapshot_display(&metrics.strategy_signal_count),
        snapshot_display(&metrics.strategy_rejection_count),
        snapshot_display(&metrics.kill_switch_dry_run.artifact_status),
        snapshot_display(&metrics.kill_switch_dry_run.kill_switch_active),
        snapshot_display(&metrics.kill_switch_dry_run.kill_switch_dry_run),
        snapshot_display(
            &metrics
                .kill_switch_dry_run
                .production_order_submissions_attempted
        ),
        snapshot_display(
            &metrics
                .kill_switch_dry_run
                .production_order_mutations_attempted
        ),
        snapshot_display(&metrics.kill_switch_dry_run.dashboard_order_controls_enabled),
        metrics.external_venue_connection,
        metrics.real_orders_submitted,
        metrics.last_error_summary.as_deref().unwrap_or("none"),
    );
    Ok(())
}

fn run_supervisor_shadow_runtime(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let status = store.node_status(&opt.node_id)?;
    let record = load_node_record(&store, &opt.node_id)?;
    let strategy = strategy_session_status_from_node_status(&status);
    let preflight = shadow_preflight_summary_from_record(&record);
    println!(
        "supervisor.shadow_runtime status=ok registry_node_id={} runtime_node_id={} process_state={} lifecycle_state={} strategy_session_state={} strategy_health={} strategy_events={} strategy_summary={} preflight_status={} preflight_events={} preflight_heartbeats={} preflight_final_state={} stale_data_halted={} stop_file_observed={} production_order_submissions_attempted={} production_order_mutations_attempted={} production_order_state_reads_attempted={} listen_key_lifecycle_attempted={} dashboard_order_controls_enabled={} external_venue_connection={} real_orders_submitted={} preflight_events_path={}",
        opt.node_id,
        status.node_id,
        json_label(&record.process.state),
        json_label(&status.lifecycle_state),
        strategy.session_state,
        strategy.artifact_health,
        strategy.events_path,
        strategy.summary_path,
        preflight.status,
        preflight.event_count,
        preflight.heartbeat_count,
        preflight.final_state,
        preflight.stale_data_halted,
        preflight.stop_file_observed,
        preflight.production_order_submissions_attempted,
        preflight.production_order_mutations_attempted,
        preflight.production_order_state_reads_attempted,
        preflight.listen_key_lifecycle_attempted,
        preflight.dashboard_order_controls_enabled,
        status.external_venue_connection,
        status.real_orders_submitted,
        preflight.path.display(),
    );
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StrategySessionSupervisorStatus {
    session_state: String,
    strategy_id: String,
    market_state: String,
    risk_state: String,
    last_signal_at: String,
    last_rejection_reason: String,
    artifact_health: String,
    artifact_diagnostic: String,
    manifest_path: String,
    session_status_path: String,
    events_path: String,
    summary_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShadowPreflightSummary {
    path: PathBuf,
    status: String,
    event_count: u64,
    heartbeat_count: u64,
    final_state: String,
    stale_data_halted: bool,
    stop_file_observed: bool,
    production_order_submissions_attempted: u64,
    production_order_mutations_attempted: u64,
    production_order_state_reads_attempted: u64,
    listen_key_lifecycle_attempted: u64,
    dashboard_order_controls_enabled: bool,
}

fn shadow_preflight_summary_from_record(record: &SupervisorNodeRecord) -> ShadowPreflightSummary {
    let path = record
        .artifact_root
        .join(SHADOW_PREFLIGHT_SESSION_RELATIVE_PATH);
    let mut summary = ShadowPreflightSummary {
        path,
        status: "missing".to_string(),
        event_count: 0,
        heartbeat_count: 0,
        final_state: "none".to_string(),
        stale_data_halted: false,
        stop_file_observed: false,
        production_order_submissions_attempted: 0,
        production_order_mutations_attempted: 0,
        production_order_state_reads_attempted: 0,
        listen_key_lifecycle_attempted: 0,
        dashboard_order_controls_enabled: false,
    };

    if !summary.path.exists() {
        return summary;
    }

    let Ok(raw) = fs::read_to_string(&summary.path) else {
        summary.status = "read_error".to_string();
        return summary;
    };

    summary.status = "available".to_string();
    for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            summary.status = "invalid_jsonl".to_string();
            continue;
        };
        summary.event_count = summary.event_count.saturating_add(1);
        if value.get("event_type").and_then(Value::as_str)
            == Some("shadow_preflight_session_heartbeat")
        {
            summary.heartbeat_count = summary.heartbeat_count.saturating_add(1);
        }
        if let Some(state) = value.get("state").and_then(Value::as_str) {
            summary.final_state = state.to_string();
            summary.stale_data_halted |= state == "stale_data_halted";
        }
        summary.stale_data_halted |= value
            .get("stale_data_detected")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        summary.stop_file_observed |= value
            .get("stop_file_observed")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        summary.production_order_submissions_attempted = summary
            .production_order_submissions_attempted
            .saturating_add(json_u64_or_bool_count(
                &value,
                "production_order_submissions_attempted",
            ));
        summary.production_order_mutations_attempted = summary
            .production_order_mutations_attempted
            .saturating_add(json_u64_or_bool_count(
                &value,
                "production_order_mutations_attempted",
            ));
        summary.production_order_state_reads_attempted = summary
            .production_order_state_reads_attempted
            .saturating_add(json_u64_or_bool_count(
                &value,
                "production_order_state_reads_attempted",
            ));
        summary.listen_key_lifecycle_attempted = summary
            .listen_key_lifecycle_attempted
            .saturating_add(json_u64_or_bool_count(
                &value,
                "listen_key_lifecycle_attempted",
            ));
        summary.dashboard_order_controls_enabled |= value
            .get("dashboard_order_controls_enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    }
    summary
}

fn json_u64_or_bool_count(value: &Value, field: &str) -> u64 {
    if let Some(count) = value.get(field).and_then(Value::as_u64) {
        return count;
    }
    u64::from(value.get(field).and_then(Value::as_bool).unwrap_or(false))
}

impl Default for StrategySessionSupervisorStatus {
    fn default() -> Self {
        Self {
            session_state: "none".to_string(),
            strategy_id: "none".to_string(),
            market_state: "none".to_string(),
            risk_state: "none".to_string(),
            last_signal_at: "none".to_string(),
            last_rejection_reason: "none".to_string(),
            artifact_health: "none".to_string(),
            artifact_diagnostic: "none".to_string(),
            manifest_path: "none".to_string(),
            session_status_path: "none".to_string(),
            events_path: "none".to_string(),
            summary_path: "none".to_string(),
        }
    }
}

fn strategy_session_status_from_node_status(
    status: &NodeStatus,
) -> StrategySessionSupervisorStatus {
    let Some(artifact_root) = status.artifact_root.value.as_deref() else {
        return StrategySessionSupervisorStatus::default();
    };
    let strategy_root = Path::new(artifact_root).join("strategy");
    let lifecycle_state = json_label(&status.lifecycle_state);
    strategy_session_status_from_artifact_root(&strategy_root, Some(&lifecycle_state))
}

fn strategy_session_status_from_artifact_root(
    root: &Path,
    node_lifecycle_state: Option<&str>,
) -> StrategySessionSupervisorStatus {
    let session_status_path = root.join("session_status.json");
    let events_path = root.join("events.jsonl");
    let market_status_path = root.join("market_status.json");
    let signal_path = root.join("signal.jsonl");
    let risk_decision_path = root.join("risk_decision.jsonl");
    let summary_path = root.join("summary.json");
    let manifest_path = root.join("manifest.json");

    let mut status = StrategySessionSupervisorStatus {
        manifest_path: path_display_if_exists(&manifest_path),
        session_status_path: path_display_if_exists(&session_status_path),
        events_path: path_display_if_exists(&events_path),
        summary_path: path_display_if_exists(&summary_path),
        ..StrategySessionSupervisorStatus::default()
    };

    let has_strategy_artifact = [
        &manifest_path,
        &session_status_path,
        &events_path,
        &market_status_path,
        &signal_path,
        &risk_decision_path,
        &summary_path,
    ]
    .iter()
    .any(|path| path.exists());
    if !has_strategy_artifact {
        return status;
    }

    let audit = audit_strategy_session_artifacts(root, node_lifecycle_state);
    status.artifact_health = audit.health.label().to_string();
    status.artifact_diagnostic = audit.diagnostic_label();
    status.manifest_path = path_display_if_exists(&audit.manifest_path);

    if let Some(session) = read_json_value(&session_status_path) {
        status.session_state =
            string_field(&session, "state").unwrap_or_else(|| "unknown".to_string());
        status.strategy_id =
            string_field(&session, "strategy_id").unwrap_or_else(|| "unknown".to_string());
    }
    if let Some(market) = read_json_value(&market_status_path) {
        status.market_state = string_field(&market, "state")
            .or_else(|| string_field(&market, "connection"))
            .unwrap_or_else(|| "unknown".to_string());
    }
    if let Some(signal) = read_latest_jsonl_value(&signal_path) {
        status.last_signal_at =
            string_field(&signal, "generated_at").unwrap_or_else(|| "unknown".to_string());
    }
    if let Some(decision) = read_latest_jsonl_value(&risk_decision_path) {
        status.risk_state =
            string_field(&decision, "decision").unwrap_or_else(|| "unknown".to_string());
        status.last_rejection_reason =
            string_array_field(&decision, "reasons").unwrap_or_else(|| "unknown".to_string());
    }

    status
}

fn path_display_if_exists(path: &Path) -> String {
    if path.exists() {
        path.display().to_string()
    } else {
        "none".to_string()
    }
}

fn read_json_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn read_latest_jsonl_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    raw.lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(|line| serde_json::from_str(line).ok())
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().map(ToString::to_string)
}

fn string_array_field(value: &Value, field: &str) -> Option<String> {
    let values = value.get(field)?.as_array()?;
    let joined = values
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join("+");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

fn load_node_record(
    store: &SupervisorRegistryStore,
    node_id: &str,
) -> anyhow::Result<SupervisorNodeRecord> {
    validate_node_id(node_id)?;
    let registry = store.load()?;
    registry
        .nodes
        .get(node_id)
        .cloned()
        .with_context(|| format!("node '{node_id}' is not registered"))
}

fn json_label<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn snapshot_display<T: Display>(value: &SnapshotValue<T>) -> String {
    value
        .value
        .as_ref()
        .map_or_else(|| json_label(&value.availability), ToString::to_string)
}

impl Default for SupervisorProcessRecord {
    fn default() -> Self {
        Self {
            pid: SnapshotValue::not_configured(),
            state: SupervisorProcessState::NotStarted,
            updated_at: SnapshotValue::unknown(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisorProcessState {
    NotStarted,
    Running,
    Stopped,
    Stale,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryArtifactState {
    Available,
    Missing,
    Invalid,
    Stale,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterNodeRequest {
    pub node_id: String,
    pub config_path: PathBuf,
    pub artifact_root: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartNodeRequest {
    pub node_id: String,
    pub ntpro_node_bin: PathBuf,
    pub startup_timeout: Duration,
    pub node_max_runtime: Duration,
    pub node_heartbeat_interval: Duration,
    pub node_parent_pid: Option<u32>,
    pub node_shutdown_timeout: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StopNodeRequest {
    pub node_id: String,
    pub stop_timeout: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupervisorRegistryStore {
    registry_path: PathBuf,
}

#[derive(Debug)]
struct RegistryFileLock {
    path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct RegistryLockArtifact {
    schema_version: String,
    pid: u32,
    acquired_at: String,
}

#[derive(Debug)]
enum RegistryLockState {
    Active(Option<RegistryLockArtifact>),
    Recoverable(String),
    Vanished,
}

impl Drop for RegistryFileLock {
    fn drop(&mut self) {
        let _ = remove_file_if_exists(&self.path);
    }
}

impl RegistryLockArtifact {
    fn new(pid: u32) -> Self {
        Self {
            schema_version: SUPERVISOR_REGISTRY_LOCK_SCHEMA_VERSION.to_string(),
            pid,
            acquired_at: now_millis(),
        }
    }
}

fn inspect_registry_lock(lock_path: &Path) -> anyhow::Result<RegistryLockState> {
    let raw = match fs::read_to_string(lock_path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RegistryLockState::Vanished);
        }
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "failed to read supervisor registry lock '{}'",
                    lock_path.display()
                )
            });
        }
    };

    let Some(artifact) = parse_registry_lock_artifact(&raw) else {
        return Ok(RegistryLockState::Active(None));
    };

    if process_is_alive(artifact.pid) {
        Ok(RegistryLockState::Active(Some(artifact)))
    } else {
        Ok(RegistryLockState::Recoverable(format!(
            "owner pid {} is not alive",
            artifact.pid
        )))
    }
}

fn parse_registry_lock_artifact(raw: &str) -> Option<RegistryLockArtifact> {
    serde_json::from_str::<RegistryLockArtifact>(raw)
        .ok()
        .or_else(|| parse_legacy_registry_lock_artifact(raw))
}

fn parse_legacy_registry_lock_artifact(raw: &str) -> Option<RegistryLockArtifact> {
    let mut pid = None;
    let mut acquired_at = None;
    for token in raw.split_whitespace() {
        if let Some(value) = token.strip_prefix("pid=") {
            pid = value.parse::<u32>().ok();
        } else if let Some(value) = token.strip_prefix("acquired_at=") {
            acquired_at = Some(value.to_string());
        }
    }

    Some(RegistryLockArtifact {
        schema_version: "legacy.supervisor_registry_lock.v0".to_string(),
        pid: pid?,
        acquired_at: acquired_at.unwrap_or_else(|| "unknown".to_string()),
    })
}

impl SupervisorRegistryStore {
    #[must_use]
    pub fn new(registry_path: impl Into<PathBuf>) -> Self {
        Self {
            registry_path: registry_path.into(),
        }
    }

    #[must_use]
    pub fn registry_path(&self) -> &Path {
        &self.registry_path
    }

    /// # Errors
    ///
    /// Returns an error if the registry file exists but cannot be read or
    /// deserialized.
    pub fn load(&self) -> anyhow::Result<SupervisorRegistry> {
        if !self.registry_path.exists() {
            return Ok(SupervisorRegistry::default());
        }

        let raw = fs::read_to_string(&self.registry_path).with_context(|| {
            format!(
                "failed to read supervisor registry '{}'",
                self.registry_path.display()
            )
        })?;
        let registry = serde_json::from_str(&raw).with_context(|| {
            format!(
                "failed to parse supervisor registry '{}'",
                self.registry_path.display()
            )
        })?;
        Ok(registry)
    }

    /// # Errors
    ///
    /// Returns an error if the registry directory or file cannot be written.
    pub fn save(&self, registry: &SupervisorRegistry) -> anyhow::Result<()> {
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create registry directory '{}'", parent.display())
            })?;
        }
        atomic_write_json(&self.registry_path, registry).with_context(|| {
            format!(
                "failed to write supervisor registry '{}'",
                self.registry_path.display()
            )
        })?;
        Ok(())
    }

    fn acquire_registry_lock(&self) -> anyhow::Result<RegistryFileLock> {
        self.acquire_registry_lock_with_timeout(REGISTRY_LOCK_TIMEOUT)
    }

    fn acquire_registry_lock_with_timeout(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<RegistryFileLock> {
        let lock_path = self.registry_lock_path();
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create registry lock directory '{}'",
                    parent.display()
                )
            })?;
        }

        let started = SystemTime::now();
        loop {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(mut file) => {
                    let lock = RegistryFileLock {
                        path: lock_path.clone(),
                    };
                    let artifact = RegistryLockArtifact::new(process::id());
                    serde_json::to_writer_pretty(&mut file, &artifact).with_context(|| {
                        format!(
                            "failed to serialize registry lock '{}'",
                            lock_path.display()
                        )
                    })?;
                    writeln!(file).with_context(|| {
                        format!("failed to write registry lock '{}'", lock_path.display())
                    })?;
                    file.sync_all().with_context(|| {
                        format!("failed to sync registry lock '{}'", lock_path.display())
                    })?;
                    return Ok(lock);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    match inspect_registry_lock(&lock_path)? {
                        RegistryLockState::Recoverable(reason) => {
                            remove_file_if_exists(&lock_path).with_context(|| {
                                format!(
                                    "failed to recover stale supervisor registry lock '{}' ({reason})",
                                    lock_path.display()
                                )
                            })?;
                            continue;
                        }
                        RegistryLockState::Vanished => continue,
                        RegistryLockState::Active(owner) => {
                            if started.elapsed().is_ok_and(|elapsed| elapsed >= timeout) {
                                let owner = owner.as_ref().map_or_else(
                                    || "unknown owner".to_string(),
                                    |artifact| {
                                        format!(
                                            "active pid={} acquired_at={}",
                                            artifact.pid, artifact.acquired_at
                                        )
                                    },
                                );
                                anyhow::bail!(
                                    "timed out waiting for supervisor registry lock '{}' ({owner})",
                                    lock_path.display()
                                );
                            }
                        }
                    }
                    thread::sleep(REGISTRY_LOCK_RETRY);
                }
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!("failed to create registry lock '{}'", lock_path.display())
                    });
                }
            }
        }
    }

    fn registry_lock_path(&self) -> PathBuf {
        let lock_name = self.registry_path.file_name().map_or_else(
            || "registry.json.lock".to_string(),
            |name| format!("{}.lock", name.to_string_lossy()),
        );
        self.registry_path.with_file_name(lock_name)
    }

    /// # Errors
    ///
    /// Returns an error if `node_id` is invalid, the config path is missing, a
    /// running record would be overwritten, or registry files cannot be saved.
    pub fn register_node(
        &self,
        request: RegisterNodeRequest,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        validate_node_id(&request.node_id)?;
        ensure!(
            request.config_path.exists(),
            "config path '{}' does not exist",
            request.config_path.display()
        );

        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        if let Some(existing) = registry.nodes.get(&request.node_id)
            && existing.process.state == SupervisorProcessState::Running
        {
            anyhow::bail!(
                "node '{}' is running and cannot be replaced",
                request.node_id
            );
        }

        let artifact_root = request
            .artifact_root
            .unwrap_or_else(|| self.default_node_artifact_root(&request.node_id));
        create_node_dirs(&artifact_root)?;
        let record =
            SupervisorNodeRecord::new(request.node_id.clone(), request.config_path, artifact_root);
        registry.nodes.insert(request.node_id, record.clone());
        registry.updated_at = SnapshotValue::available(now_millis());
        self.save(&registry)?;
        Ok(record)
    }

    /// # Errors
    ///
    /// Returns an error if the registry cannot be loaded or saved, or if the
    /// node is missing.
    pub fn update_process(
        &self,
        node_id: &str,
        pid: Option<u32>,
        state: SupervisorProcessState,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;
        record.process = SupervisorProcessRecord {
            pid: pid.map_or_else(SnapshotValue::not_configured, SnapshotValue::available),
            state,
            updated_at: SnapshotValue::available(now_millis()),
        };
        write_or_remove_pid_artifact(record)?;
        record.updated_at = SnapshotValue::available(now_millis());
        let updated = record.clone();
        registry.updated_at = SnapshotValue::available(now_millis());
        self.save(&registry)?;
        Ok(updated)
    }

    /// # Errors
    ///
    /// Returns an error if the registry cannot be loaded or saved, or if the
    /// node is missing.
    pub fn refresh_process_state(&self, node_id: &str) -> anyhow::Result<SupervisorNodeRecord> {
        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;

        if record.process.state == SupervisorProcessState::Running && pid_artifact_is_stale(record)?
        {
            let transition_at = now_millis();
            if let Some(stopped_status) = stopped_status_artifact(record) {
                record.process = SupervisorProcessRecord {
                    pid: SnapshotValue::not_configured(),
                    state: SupervisorProcessState::Stopped,
                    updated_at: SnapshotValue::available(transition_at.clone()),
                };
                record.last_known_status = stopped_status;
                record.status_artifact = RegistryArtifactState::Available;
                record.updated_at = SnapshotValue::available(transition_at);
                write_or_remove_pid_artifact(record)?;
            } else {
                record.process.state = SupervisorProcessState::Stale;
                record.process.updated_at = SnapshotValue::available(transition_at.clone());
                record.updated_at = SnapshotValue::available(transition_at);
            }
        }

        let updated = record.clone();
        registry.updated_at = SnapshotValue::available(now_millis());
        self.save(&registry)?;
        Ok(updated)
    }

    /// # Errors
    ///
    /// Returns an error if the registry cannot be loaded or saved, or if the
    /// node is missing.
    pub fn refresh_status_from_artifact(
        &self,
        node_id: &str,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        self.refresh_process_state(node_id)?;
        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;

        if record.status_path.exists() {
            let raw = fs::read_to_string(&record.status_path).with_context(|| {
                format!(
                    "failed to read status artifact '{}'",
                    record.status_path.display()
                )
            })?;
            match serde_json::from_str::<NodeStatus>(&raw) {
                Ok(status) if status.node_id == record.node_id => {
                    record.status_artifact = RegistryArtifactState::Available;
                    record.last_known_status = status;
                }
                Ok(status) => {
                    record.status_artifact = RegistryArtifactState::Invalid;
                    record.last_known_status.last_error = Some(format!(
                        "status node identity mismatch: registry node '{}' received runtime node '{}'",
                        record.node_id, status.node_id
                    ));
                    record.last_known_status.generated_at = SnapshotValue::stale();
                }
                Err(error) => {
                    record.status_artifact = RegistryArtifactState::Invalid;
                    record.last_known_status.last_error =
                        Some(format!("invalid status artifact: {error}"));
                    record.last_known_status.generated_at = SnapshotValue::stale();
                }
            }
        } else {
            record.status_artifact = RegistryArtifactState::Missing;
            record.last_known_status.generated_at = SnapshotValue::stale();
        }

        record.updated_at = SnapshotValue::available(now_millis());
        let updated = record.clone();
        registry.updated_at = SnapshotValue::available(now_millis());
        self.save(&registry)?;
        Ok(updated)
    }

    /// # Errors
    ///
    /// Returns an error if the registry cannot be loaded.
    pub fn list_nodes(&self) -> anyhow::Result<Vec<SupervisorNodeRecord>> {
        let node_ids = self.load()?.nodes.into_keys().collect::<Vec<_>>();
        for node_id in node_ids {
            self.refresh_process_state(&node_id)?;
        }
        Ok(self.load()?.nodes.into_values().collect())
    }

    /// # Errors
    ///
    /// Returns an error if the registry cannot be loaded or saved.
    pub fn remove_node(&self, node_id: &str) -> anyhow::Result<Option<SupervisorNodeRecord>> {
        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let removed = registry.nodes.remove(node_id);
        if removed.is_some() {
            registry.updated_at = SnapshotValue::available(now_millis());
            self.save(&registry)?;
        }
        Ok(removed)
    }

    /// # Errors
    ///
    /// Returns an error if the node is missing, already running, the
    /// `ntpro-node` process cannot be spawned, or the running status artifact
    /// is not observed before the timeout.
    pub fn start_node_process(
        &self,
        request: &StartNodeRequest,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        validate_node_id(&request.node_id)?;
        ensure!(
            !request.node_max_runtime.is_zero(),
            "node_max_runtime must be greater than zero"
        );
        ensure!(
            !request.node_heartbeat_interval.is_zero(),
            "node_heartbeat_interval must be greater than zero"
        );
        ensure!(
            !request.node_shutdown_timeout.is_zero(),
            "node_shutdown_timeout must be greater than zero"
        );
        ensure!(
            request.ntpro_node_bin.exists(),
            "ntpro-node binary '{}' does not exist",
            request.ntpro_node_bin.display()
        );
        self.refresh_process_state(&request.node_id)?;

        let mut child = {
            let _lock = self.acquire_registry_lock()?;
            let mut registry = self.load()?;
            let record = registry
                .nodes
                .get_mut(&request.node_id)
                .with_context(|| format!("node '{}' is not registered", request.node_id))?;
            ensure!(
                record.process.state != SupervisorProcessState::Running,
                "node '{}' is already running",
                request.node_id
            );
            ensure!(
                record.config_path.exists(),
                "config path '{}' does not exist",
                record.config_path.display()
            );
            create_node_dirs(&record.artifact_root)?;
            let stop_file = stop_file_path(record);
            if stop_file.exists() {
                remove_file_if_exists(&stop_file).with_context(|| {
                    format!("failed to remove stale stop file '{}'", stop_file.display())
                })?;
            }
            for artifact_path in [&record.status_path, &record.metrics_path] {
                if artifact_path.exists() {
                    remove_file_if_exists(artifact_path).with_context(|| {
                        format!(
                            "failed to remove stale node artifact '{}'",
                            artifact_path.display()
                        )
                    })?;
                }
            }
            record.status_artifact = RegistryArtifactState::Missing;
            record.metrics_artifact = RegistryArtifactState::Missing;
            let stdout_log = fs::File::create(&record.stdout_log_path).with_context(|| {
                format!(
                    "failed to create stdout log '{}'",
                    record.stdout_log_path.display()
                )
            })?;
            let stderr_log = fs::File::create(&record.stderr_log_path).with_context(|| {
                format!(
                    "failed to create stderr log '{}'",
                    record.stderr_log_path.display()
                )
            })?;

            let mut command = Command::new(&request.ntpro_node_bin);
            command
                .arg("--config")
                .arg(&record.config_path)
                .arg("--run-id")
                .arg(&record.node_id)
                .arg("--output")
                .arg(&record.artifact_root)
                .arg("--stop-file")
                .arg(&stop_file)
                .arg("--max-runtime-ms")
                .arg(duration_millis_arg(request.node_max_runtime))
                .arg("--heartbeat-interval-ms")
                .arg(duration_millis_arg(request.node_heartbeat_interval))
                .arg("--shutdown-timeout-ms")
                .arg(duration_millis_arg(request.node_shutdown_timeout))
                .stdout(Stdio::from(stdout_log))
                .stderr(Stdio::from(stderr_log));
            if let Some(parent_pid) = request.node_parent_pid {
                command.arg("--parent-pid").arg(parent_pid.to_string());
            }
            let child = command.spawn().with_context(|| {
                format!(
                    "failed to spawn ntpro-node '{}'",
                    request.ntpro_node_bin.display()
                )
            })?;

            record.process = SupervisorProcessRecord {
                pid: SnapshotValue::available(child.id()),
                state: SupervisorProcessState::Running,
                updated_at: SnapshotValue::available(now_millis()),
            };
            record.last_known_status.started_at = SnapshotValue::unknown();
            write_or_remove_pid_artifact(record)?;
            record.updated_at = SnapshotValue::available(now_millis());
            registry.updated_at = SnapshotValue::available(now_millis());
            self.save(&registry)?;
            child
        };

        let startup_result =
            wait_for_startup(self, &request.node_id, &mut child, request.startup_timeout);
        match startup_result {
            Ok(record) => Ok(record),
            Err(error) => {
                let pid = child.id();
                let cleanup_error = stop_child_after_start_failure(&mut child).err();
                let process_exited = cleanup_error.is_none();
                let retained_pid = (!process_exited).then_some(pid);
                let message = cleanup_error.as_ref().map_or_else(
                    || format!("node startup failed: {error}"),
                    |cleanup| format!("node startup failed: {error}; cleanup failed: {cleanup}"),
                );
                self.record_process_failure(&request.node_id, retained_pid, &message)?;
                if let Some(cleanup) = cleanup_error {
                    return Err(
                        error.context(format!("failed to clean up node process {pid}: {cleanup}"))
                    );
                }
                Err(error)
            }
        }
    }

    /// # Errors
    ///
    /// Returns an error if the node is missing, not running, the stop request
    /// cannot be written, or the stopped status artifact is not observed before
    /// the timeout.
    pub fn stop_node_process(
        &self,
        request: &StopNodeRequest,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        validate_node_id(&request.node_id)?;
        self.refresh_process_state(&request.node_id)?;
        let pid = {
            let _lock = self.acquire_registry_lock()?;
            let mut registry = self.load()?;
            let record = registry
                .nodes
                .get_mut(&request.node_id)
                .with_context(|| format!("node '{}' is not registered", request.node_id))?;
            ensure!(
                record.process.state == SupervisorProcessState::Running,
                "node '{}' is not running",
                request.node_id
            );
            let pid = record
                .process
                .pid
                .value
                .with_context(|| format!("node '{}' has no process pid", request.node_id))?;
            let stop_file = stop_file_path(record);
            ensure!(
                !stop_file.exists(),
                "node '{}' stop is already requested",
                request.node_id
            );
            if let Some(parent) = stop_file.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create stop directory '{}'", parent.display())
                })?;
            }
            atomic_write_text(&stop_file, &format!("{}\n", now_millis()))
                .with_context(|| format!("failed to write stop file '{}'", stop_file.display()))?;
            pid
        };

        if let Some(stopped) =
            wait_for_stopped_process(self, &request.node_id, pid, request.stop_timeout)?
        {
            return self.finalize_stopped_process(&request.node_id, stopped);
        }

        match send_termination(pid)? {
            SignalDelivery::Sent => {
                let _ = wait_for_process_exit(pid, PROCESS_SIGNAL_GRACE);
            }
            SignalDelivery::ProcessExited | SignalDelivery::Unsupported => {}
        }
        if process_is_alive(pid) {
            send_kill(pid)?;
            let _ = wait_for_process_exit(pid, PROCESS_SIGNAL_GRACE);
        }

        if process_is_alive(pid) {
            let message = format!(
                "node '{}' process {pid} did not exit after termination escalation",
                request.node_id
            );
            self.record_process_failure(&request.node_id, Some(pid), &message)?;
            anyhow::bail!("{message}");
        }

        let stopped = self.refresh_status_from_artifact(&request.node_id)?;
        if stopped.last_known_status.lifecycle_state != LifecycleStatus::Stopped {
            let message = format!(
                "node '{}' process {pid} exited without a stopped status artifact",
                request.node_id
            );
            self.record_process_failure(&request.node_id, None, &message)?;
            anyhow::bail!("{message}");
        }
        self.finalize_stopped_process(&request.node_id, stopped)
    }

    /// # Errors
    ///
    /// Returns an error if the node is missing, not running, or supervisor
    /// artifacts cannot be written.
    pub fn pause_node(&self, node_id: &str) -> anyhow::Result<SupervisorNodeRecord> {
        self.transition_local_lifecycle(
            node_id,
            LifecycleStatus::Running,
            LifecycleStatus::Paused,
            "pause",
        )
    }

    /// # Errors
    ///
    /// Returns an error if the node is missing, not paused, or supervisor
    /// artifacts cannot be written.
    pub fn resume_node(&self, node_id: &str) -> anyhow::Result<SupervisorNodeRecord> {
        self.transition_local_lifecycle(
            node_id,
            LifecycleStatus::Paused,
            LifecycleStatus::Running,
            "resume",
        )
    }

    /// # Errors
    ///
    /// Returns an error if the node is missing, the process is not running, or
    /// supervisor artifacts cannot be written.
    pub fn reconnect_data_source(&self, node_id: &str) -> anyhow::Result<SupervisorNodeRecord> {
        validate_node_id(node_id)?;
        self.refresh_status_from_artifact(node_id)?;

        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;
        ensure!(
            record.process.state == SupervisorProcessState::Running,
            "node '{node_id}' process is not running"
        );
        ensure!(
            matches!(
                record.last_known_status.lifecycle_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            ),
            "node '{}' lifecycle state is {}, expected running or paused",
            node_id,
            json_label(&record.last_known_status.lifecycle_state),
        );

        let transition_at = now_millis();
        record.last_known_status.data_connection = ConnectionStatus::NotSupported;
        record.last_known_status.generated_at = SnapshotValue::available(transition_at.clone());
        record.last_known_status.last_transition_at =
            SnapshotValue::available(transition_at.clone());
        record.last_known_status.last_error = Some(DATA_RECONNECT_UNSUPPORTED_MESSAGE.to_string());
        record.last_known_status.external_venue_connection = false;
        record.last_known_status.real_orders_submitted = false;
        record.status_artifact = RegistryArtifactState::Available;
        record.metrics_artifact = RegistryArtifactState::Available;
        record.updated_at = SnapshotValue::available(transition_at.clone());

        atomic_write_json(&record.status_path, &record.last_known_status).with_context(|| {
            format!(
                "failed to write status artifact '{}'",
                record.status_path.display()
            )
        })?;
        let counts = control_metric_counts(record, &transition_at);
        let metrics = NodeMetrics::from_status(
            &record.last_known_status,
            &NodeMetricArtifacts::from_record(record),
            counts,
        );
        write_node_metrics_artifact(&record.metrics_path, &metrics)?;
        append_supervisor_event_with_status(record, "reconnect_data", "not_supported")?;

        let updated = record.clone();
        registry.updated_at = SnapshotValue::available(transition_at);
        self.save(&registry)?;
        Ok(updated)
    }

    /// # Errors
    ///
    /// Returns an error if the node is missing, the process is not running, or
    /// supervisor artifacts cannot be written.
    pub fn reconnect_execution_gateway(
        &self,
        node_id: &str,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        validate_node_id(node_id)?;
        self.refresh_status_from_artifact(node_id)?;

        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;
        ensure!(
            record.process.state == SupervisorProcessState::Running,
            "node '{node_id}' process is not running"
        );
        ensure!(
            matches!(
                record.last_known_status.lifecycle_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            ),
            "node '{}' lifecycle state is {}, expected running or paused",
            node_id,
            json_label(&record.last_known_status.lifecycle_state),
        );

        let transition_at = now_millis();
        record.last_known_status.execution_connection = ConnectionStatus::NotSupported;
        record.last_known_status.execution.connection = ConnectionStatus::NotSupported;
        record.last_known_status.execution.last_error =
            Some(EXECUTION_RECONNECT_UNSUPPORTED_MESSAGE.to_string());
        record.last_known_status.generated_at = SnapshotValue::available(transition_at.clone());
        record.last_known_status.last_transition_at =
            SnapshotValue::available(transition_at.clone());
        record.last_known_status.last_error =
            Some(EXECUTION_RECONNECT_UNSUPPORTED_MESSAGE.to_string());
        record.last_known_status.external_venue_connection = false;
        record.last_known_status.real_orders_submitted = false;
        record.status_artifact = RegistryArtifactState::Available;
        record.metrics_artifact = RegistryArtifactState::Available;
        record.updated_at = SnapshotValue::available(transition_at.clone());

        atomic_write_json(&record.status_path, &record.last_known_status).with_context(|| {
            format!(
                "failed to write status artifact '{}'",
                record.status_path.display()
            )
        })?;
        let counts = control_metric_counts(record, &transition_at);
        let metrics = NodeMetrics::from_status(
            &record.last_known_status,
            &NodeMetricArtifacts::from_record(record),
            counts,
        );
        write_node_metrics_artifact(&record.metrics_path, &metrics)?;
        append_supervisor_event_with_status(record, "reconnect_execution", "not_supported")?;

        let updated = record.clone();
        registry.updated_at = SnapshotValue::available(transition_at);
        self.save(&registry)?;
        Ok(updated)
    }

    /// # Errors
    ///
    /// Returns an error if the node is missing or the registry/status artifact
    /// cannot be read.
    pub fn node_status(&self, node_id: &str) -> anyhow::Result<NodeStatus> {
        validate_node_id(node_id)?;
        let record = self.refresh_status_from_artifact(node_id)?;
        if record.status_artifact == RegistryArtifactState::Invalid {
            anyhow::bail!(
                "invalid status artifact for registry node '{node_id}': {}",
                record
                    .last_known_status
                    .last_error
                    .as_deref()
                    .unwrap_or("unknown status artifact error")
            );
        }
        Ok(record.last_known_status)
    }

    /// # Errors
    ///
    /// Returns an error if the node is missing, the metrics artifact is
    /// missing, or the JSON shape is invalid.
    pub fn node_metrics(&self, node_id: &str) -> anyhow::Result<NodeMetrics> {
        validate_node_id(node_id)?;
        self.refresh_process_state(node_id)?;
        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;

        let metrics_path = record.metrics_path.clone();
        if !metrics_path.exists() {
            record.metrics_artifact = RegistryArtifactState::Missing;
            record.updated_at = SnapshotValue::available(now_millis());
            registry.updated_at = SnapshotValue::available(now_millis());
            self.save(&registry)?;
            anyhow::bail!(
                "metrics artifact '{}' does not exist",
                metrics_path.display()
            );
        }

        let raw = fs::read_to_string(&metrics_path).with_context(|| {
            format!(
                "failed to read metrics artifact '{}'",
                metrics_path.display()
            )
        })?;
        match serde_json::from_str::<NodeMetrics>(&raw) {
            Ok(metrics) if metrics.node_id == record.node_id => {
                record.metrics_artifact = RegistryArtifactState::Available;
                record.updated_at = SnapshotValue::available(now_millis());
                registry.updated_at = SnapshotValue::available(now_millis());
                self.save(&registry)?;
                Ok(metrics)
            }
            Ok(metrics) => {
                let message = format!(
                    "metrics node identity mismatch: registry node '{}' received runtime node '{}'",
                    record.node_id, metrics.node_id
                );
                record.metrics_artifact = RegistryArtifactState::Invalid;
                record.updated_at = SnapshotValue::available(now_millis());
                registry.updated_at = SnapshotValue::available(now_millis());
                self.save(&registry)?;
                anyhow::bail!("{message}");
            }
            Err(error) => {
                record.metrics_artifact = RegistryArtifactState::Invalid;
                record.updated_at = SnapshotValue::available(now_millis());
                registry.updated_at = SnapshotValue::available(now_millis());
                self.save(&registry)?;
                anyhow::bail!("invalid metrics artifact: {error}");
            }
        }
    }

    fn default_node_artifact_root(&self, node_id: &str) -> PathBuf {
        self.registry_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("nodes")
            .join(node_id)
    }

    fn finalize_stopped_process(
        &self,
        node_id: &str,
        mut stopped: SupervisorNodeRecord,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        stopped.process = SupervisorProcessRecord {
            pid: SnapshotValue::not_configured(),
            state: SupervisorProcessState::Stopped,
            updated_at: SnapshotValue::available(now_millis()),
        };

        let _lock = self.acquire_registry_lock()?;
        write_or_remove_pid_artifact(&stopped)?;
        let mut registry = self.load()?;
        registry.nodes.insert(node_id.to_string(), stopped.clone());
        registry.updated_at = SnapshotValue::available(now_millis());
        self.save(&registry)?;
        Ok(stopped)
    }

    fn record_process_failure(
        &self,
        node_id: &str,
        pid: Option<u32>,
        message: &str,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;
        record.process = SupervisorProcessRecord {
            pid: pid.map_or_else(SnapshotValue::not_configured, SnapshotValue::available),
            state: SupervisorProcessState::Stale,
            updated_at: SnapshotValue::available(now_millis()),
        };
        record.last_known_status.last_error = Some(message.to_string());
        record.updated_at = SnapshotValue::available(now_millis());
        write_or_remove_pid_artifact(record)?;
        let updated = record.clone();
        registry.updated_at = SnapshotValue::available(now_millis());
        self.save(&registry)?;
        Ok(updated)
    }

    fn transition_local_lifecycle(
        &self,
        node_id: &str,
        expected: LifecycleStatus,
        next: LifecycleStatus,
        event_phase: &str,
    ) -> anyhow::Result<SupervisorNodeRecord> {
        validate_node_id(node_id)?;
        self.refresh_status_from_artifact(node_id)?;

        let _lock = self.acquire_registry_lock()?;
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;
        ensure!(
            record.process.state == SupervisorProcessState::Running,
            "node '{node_id}' process is not running"
        );
        ensure!(
            record.last_known_status.lifecycle_state == expected,
            "node '{}' lifecycle state is {}, expected {}",
            node_id,
            json_label(&record.last_known_status.lifecycle_state),
            json_label(&expected),
        );

        let previous = record.last_known_status.lifecycle_state;
        let transition_at = now_millis();
        record.last_known_status.previous_lifecycle_state = previous;
        record.last_known_status.lifecycle_state = next;
        record.last_known_status.generated_at = SnapshotValue::available(transition_at.clone());
        record.last_known_status.last_transition_at =
            SnapshotValue::available(transition_at.clone());
        record.last_known_status.last_error = None;
        record.last_known_status.external_venue_connection = false;
        record.last_known_status.real_orders_submitted = false;
        record.status_artifact = RegistryArtifactState::Available;
        record.metrics_artifact = RegistryArtifactState::Available;
        record.updated_at = SnapshotValue::available(transition_at.clone());

        atomic_write_json(&record.status_path, &record.last_known_status).with_context(|| {
            format!(
                "failed to write status artifact '{}'",
                record.status_path.display()
            )
        })?;
        let counts = transition_metric_counts(record, &transition_at);
        let metrics = NodeMetrics::from_status(
            &record.last_known_status,
            &NodeMetricArtifacts::from_record(record),
            counts,
        );
        write_node_metrics_artifact(&record.metrics_path, &metrics)?;
        append_supervisor_event(record, event_phase)?;

        let updated = record.clone();
        registry.updated_at = SnapshotValue::available(transition_at);
        self.save(&registry)?;
        Ok(updated)
    }
}

fn stop_file_path(record: &SupervisorNodeRecord) -> PathBuf {
    record.artifact_root.join("stop.request")
}

fn wait_for_startup(
    store: &SupervisorRegistryStore,
    node_id: &str,
    child: &mut Child,
    timeout: Duration,
) -> anyhow::Result<SupervisorNodeRecord> {
    let started = SystemTime::now();
    let mut last_retryable_status_error = None;
    loop {
        if let Some(status) = child
            .try_wait()
            .context("failed to inspect child process state")?
        {
            anyhow::bail!(
                "node '{node_id}' process {} exited before reaching running status: {status}",
                child.id()
            );
        }
        let record = store.refresh_status_from_artifact(node_id)?;
        if record.status_artifact == RegistryArtifactState::Invalid {
            let last_error = record
                .last_known_status
                .last_error
                .as_deref()
                .unwrap_or("unknown status artifact error");
            if last_error.starts_with("invalid status artifact:") {
                last_retryable_status_error = Some(last_error.to_string());
            } else {
                anyhow::bail!(
                    "node '{node_id}' published an invalid status artifact: {last_error}"
                );
            }
        }
        if record.last_known_status.lifecycle_state == LifecycleStatus::Running {
            if let Some(status) = child
                .try_wait()
                .context("failed to inspect child process state")?
            {
                anyhow::bail!(
                    "node '{node_id}' process {} exited while reporting running status: {status}",
                    child.id()
                );
            }
            write_or_remove_pid_artifact(&record)?;
            return Ok(record);
        }
        if started.elapsed().is_ok_and(|elapsed| elapsed >= timeout) {
            if let Some(error) = last_retryable_status_error {
                anyhow::bail!(
                    "node '{node_id}' process {} timed out waiting for running status; last status artifact error: {error}",
                    child.id()
                );
            }
            anyhow::bail!(
                "node '{node_id}' process {} timed out waiting for running status",
                child.id()
            );
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

fn wait_for_stopped_process(
    store: &SupervisorRegistryStore,
    node_id: &str,
    pid: u32,
    timeout: Duration,
) -> anyhow::Result<Option<SupervisorNodeRecord>> {
    let started = SystemTime::now();
    loop {
        let record = store.refresh_status_from_artifact(node_id)?;
        if record.last_known_status.lifecycle_state == LifecycleStatus::Stopped
            && !process_is_alive(pid)
        {
            return Ok(Some(record));
        }
        if started.elapsed().is_ok_and(|elapsed| elapsed >= timeout) {
            return Ok(None);
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

fn stop_child_after_start_failure(child: &mut Child) -> anyhow::Result<bool> {
    if child
        .try_wait()
        .context("failed to inspect failed child process")?
        .is_some()
    {
        return Ok(true);
    }
    child
        .kill()
        .context("failed to kill child after startup failure")?;
    child
        .wait()
        .context("failed to wait for child after startup failure")?;
    Ok(true)
}

impl SupervisorNodeRecord {
    #[must_use]
    pub fn new(node_id: String, config_path: PathBuf, artifact_root: PathBuf) -> Self {
        Self {
            pid_path: artifact_root.join("pid.json"),
            status_path: artifact_root.join("status.json"),
            metrics_path: artifact_root.join("metrics.json"),
            stdout_log_path: artifact_root.join("logs").join("stdout.log"),
            stderr_log_path: artifact_root.join("logs").join("stderr.log"),
            events_log_path: artifact_root.join("logs").join("events.log"),
            last_known_status: initial_status(&node_id, &config_path, &artifact_root),
            status_artifact: RegistryArtifactState::Missing,
            metrics_artifact: RegistryArtifactState::Missing,
            process: SupervisorProcessRecord::default(),
            updated_at: SnapshotValue::available(now_millis()),
            node_id,
            config_path,
            artifact_root,
        }
    }
}

fn initial_status(node_id: &str, config_path: &Path, artifact_root: &Path) -> NodeStatus {
    let mut status = NodeStatus::unknown(node_id);
    status.process_mode = ProcessMode::Unknown;
    status.config_path = SnapshotValue::available(config_path.display().to_string());
    status.artifact_root = SnapshotValue::available(artifact_root.display().to_string());
    status.lifecycle_state = LifecycleStatus::Stopped;
    status.data_connection = ConnectionStatus::NotConfigured;
    status.execution_connection = ConnectionStatus::NotConfigured;
    status
}

fn duration_millis_arg(duration: Duration) -> String {
    u64::try_from(duration.as_millis())
        .unwrap_or(u64::MAX)
        .to_string()
}

fn create_node_dirs(artifact_root: &Path) -> anyhow::Result<()> {
    for path in [
        artifact_root,
        &artifact_root.join("logs"),
        &artifact_root.join("artifacts"),
    ] {
        fs::create_dir_all(path)
            .with_context(|| format!("failed to create node directory '{}'", path.display()))?;
    }
    Ok(())
}

/// # Errors
///
/// Returns an error if the metrics directory cannot be created, metrics cannot
/// be serialized, or the artifact cannot be written.
pub fn write_node_metrics_artifact(path: &Path, metrics: &NodeMetrics) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create metrics directory '{}'", parent.display())
        })?;
    }
    atomic_write_json(path, metrics)
        .with_context(|| format!("failed to write metrics artifact '{}'", path.display()))?;
    Ok(())
}

fn transition_metric_counts(
    record: &SupervisorNodeRecord,
    transition_at: &str,
) -> NodeMetricCounts {
    metric_counts_from_existing(record, transition_at, true)
}

fn control_metric_counts(record: &SupervisorNodeRecord, transition_at: &str) -> NodeMetricCounts {
    metric_counts_from_existing(record, transition_at, false)
}

fn metric_counts_from_existing(
    record: &SupervisorNodeRecord,
    transition_at: &str,
    increment_state_transition: bool,
) -> NodeMetricCounts {
    let current = if record.metrics_path.exists() {
        fs::read_to_string(&record.metrics_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<NodeMetrics>(&raw).ok())
            .filter(|metrics| metrics.node_id == record.node_id)
    } else {
        None
    };
    let uptime_ms = current
        .as_ref()
        .and_then(|metrics| metrics.uptime_ms.value)
        .or_else(|| transition_at.parse::<u64>().ok().map(|_| 0));
    let current_state_transitions = current
        .as_ref()
        .map_or(0, |metrics| metrics.state_transitions_total);
    NodeMetricCounts {
        uptime_ms,
        starts_total: current.as_ref().map_or(0, |metrics| metrics.starts_total),
        stops_total: current.as_ref().map_or(0, |metrics| metrics.stops_total),
        state_transitions_total: if increment_state_transition {
            current_state_transitions.saturating_add(1)
        } else {
            current_state_transitions
        },
    }
}

fn append_supervisor_event(record: &SupervisorNodeRecord, phase: &str) -> anyhow::Result<()> {
    append_supervisor_event_with_status(record, phase, "ok")
}

fn append_supervisor_event_with_status(
    record: &SupervisorNodeRecord,
    phase: &str,
    status: &str,
) -> anyhow::Result<()> {
    if let Some(parent) = record.events_log_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create events log directory '{}'",
                parent.display()
            )
        })?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&record.events_log_path)
        .with_context(|| {
            format!(
                "failed to open events log '{}'",
                record.events_log_path.display()
            )
        })?;
    writeln!(
        file,
        "phase={phase} status={status} node_id={} lifecycle_state={} external_venue_connection=false real_orders_submitted=false",
        record.node_id,
        json_label(&record.last_known_status.lifecycle_state),
    )
    .with_context(|| {
        format!(
            "failed to append supervisor event '{}'",
            record.events_log_path.display()
        )
    })?;
    Ok(())
}

fn write_or_remove_pid_artifact(record: &SupervisorNodeRecord) -> anyhow::Result<()> {
    if let Some(parent) = record.pid_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create pid directory '{}'", parent.display()))?;
    }

    match record.process.pid.value {
        Some(pid) => {
            let artifact = SupervisorPidArtifact {
                node_id: record.node_id.clone(),
                pid,
                state: record.process.state,
                updated_at: record.process.updated_at.clone(),
                process_identity: Some(SupervisorProcessIdentity::from_record(record)),
            };
            atomic_write_json(&record.pid_path, &artifact).with_context(|| {
                format!(
                    "failed to write pid artifact '{}'",
                    record.pid_path.display()
                )
            })?;
        }
        None => {
            if record.pid_path.exists() {
                remove_file_if_exists(&record.pid_path).with_context(|| {
                    format!(
                        "failed to remove pid artifact '{}'",
                        record.pid_path.display()
                    )
                })?;
            }
        }
    }
    Ok(())
}

fn pid_artifact_is_stale(record: &SupervisorNodeRecord) -> anyhow::Result<bool> {
    let Some(expected_pid) = record.process.pid.value else {
        return Ok(true);
    };
    if !process_is_alive(expected_pid) {
        return Ok(true);
    }
    if !record.pid_path.exists() {
        return Ok(true);
    }

    let raw = fs::read_to_string(&record.pid_path).with_context(|| {
        format!(
            "failed to read pid artifact '{}'",
            record.pid_path.display()
        )
    })?;
    let artifact = match serde_json::from_str::<SupervisorPidArtifact>(&raw) {
        Ok(artifact) => artifact,
        Err(_) => return Ok(true),
    };
    if artifact.node_id != record.node_id || artifact.pid != expected_pid {
        return Ok(true);
    }

    let Some(identity) = artifact.process_identity.as_ref() else {
        return Ok(false);
    };

    if identity.node_id != record.node_id
        || identity.artifact_root != record.artifact_root.display().to_string()
        || identity.status_path != record.status_path.display().to_string()
    {
        return Ok(true);
    }

    if !record.status_path.exists() {
        return Ok(false);
    }

    let raw = fs::read_to_string(&record.status_path).with_context(|| {
        format!(
            "failed to read status artifact '{}'",
            record.status_path.display()
        )
    })?;
    let status = match serde_json::from_str::<NodeStatus>(&raw) {
        Ok(status) => status,
        Err(_) => return Ok(true),
    };
    if status.node_id != identity.node_id {
        return Ok(true);
    }
    if status
        .artifact_root
        .value
        .as_deref()
        .is_some_and(|artifact_root| artifact_root != identity.artifact_root)
    {
        return Ok(true);
    }
    if let (Some(expected), Some(actual)) = (
        identity.started_at.value.as_deref(),
        status.started_at.value.as_deref(),
    ) {
        return Ok(expected != actual);
    }

    Ok(false)
}

fn stopped_status_artifact(record: &SupervisorNodeRecord) -> Option<NodeStatus> {
    let raw = fs::read_to_string(&record.status_path).ok()?;
    let status = serde_json::from_str::<NodeStatus>(&raw).ok()?;
    (status.node_id == record.node_id && status.lifecycle_state == LifecycleStatus::Stopped)
        .then_some(status)
}

impl SupervisorProcessIdentity {
    #[must_use]
    pub fn from_record(record: &SupervisorNodeRecord) -> Self {
        Self {
            node_id: record.node_id.clone(),
            artifact_root: record.artifact_root.display().to_string(),
            status_path: record.status_path.display().to_string(),
            started_at: record.last_known_status.started_at.clone(),
        }
    }
}

fn validate_node_id(node_id: &str) -> anyhow::Result<()> {
    ensure!(!node_id.is_empty(), "node_id must not be empty");
    ensure!(
        node_id.len() <= 64,
        "node_id must be 64 characters or fewer"
    );
    let mut chars = node_id.chars();
    let first = chars.next().unwrap();
    ensure!(
        first.is_ascii_lowercase() || first.is_ascii_digit(),
        "node_id must start with [a-z0-9]"
    );
    ensure!(
        chars.all(|ch| ch.is_ascii_lowercase()
            || ch.is_ascii_digit()
            || matches!(ch, '.' | '_' | '-')),
        "node_id contains unsupported characters"
    );
    Ok(())
}

fn now_millis() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    format!("{millis}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opt::SupervisorRegistryOpt;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Arc, Barrier, Mutex, MutexGuard};

    #[cfg(unix)]
    static SUPERVISOR_PROCESS_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[cfg(unix)]
    fn supervisor_process_test_guard() -> MutexGuard<'static, ()> {
        SUPERVISOR_PROCESS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn temp_root(name: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "ntpro-v02-005-supervisor-{name}-{}-{unique}",
            std::process::id(),
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_config(root: &Path, name: &str) -> PathBuf {
        let path = root.join(format!("{name}.toml"));
        fs::write(&path, "[run]\nid = \"live-init-smoke\"\n").unwrap();
        path
    }

    fn write_strategy_artifacts(root: &Path) -> PathBuf {
        let strategy_root = root.join("strategy");
        fs::create_dir_all(&strategy_root).unwrap();
        fs::write(
            strategy_root.join("session_status.json"),
            r#"{
  "schema_version": "ntpro.v09_strategy_session_status.v1",
  "session_id": "btc-ema-shadow-001",
  "strategy_id": "ema_cross_btcusdt_v1",
  "state": "stopped",
  "reason": "demo strategy stopped",
  "updated_at_unix_ms": 1,
  "artifacts": {}
}
"#,
        )
        .unwrap();
        fs::write(
            strategy_root.join("market_status.json"),
            r#"{
  "schema_version": "ntpro.v09_market_stream_status.v1",
  "session_id": "btc-ema-shadow-001",
  "strategy_id": "ema_cross_btcusdt_v1",
  "connection": "exhausted",
  "state": "exhausted",
  "source": "fixture_bar_stream",
  "event_count": 8,
  "last_event_at_unix_ms": 2,
  "updated_at_unix_ms": 3
}
"#,
        )
        .unwrap();
        fs::write(
            strategy_root.join("events.jsonl"),
            r#"{"schema_version":"ntpro.v09_strategy_session_event.v1","event_type":"strategy_session_state_changed","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","state":"stopped","reason":"demo strategy stopped","occurred_at_unix_ms":3}
"#,
        )
        .unwrap();
        fs::write(
            strategy_root.join("signal.jsonl"),
            r#"{"schema_version":"ntpro.v09_strategy_signal.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","signal":"long","confidence":0.6,"market_event_seq":5,"generated_at":"unix:100","generated_at_unix_ms":100}
{"schema_version":"ntpro.v09_strategy_signal.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","signal":"flat","confidence":0.7,"market_event_seq":8,"generated_at":"unix:200","generated_at_unix_ms":200}
"#,
        )
        .unwrap();
        fs::write(
            strategy_root.join("risk_decision.jsonl"),
            r#"{"schema_version":"ntpro.v09_risk_decision.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","decision_id":"risk:1","intent_id":"intent-1","symbol":"BTCUSDT.BINANCE","decision":"rejected","reasons":["order_submission_disabled"],"mode":"shadow","order_submission":"disabled","kill_switch_enabled":true,"kill_switch_active":false,"account_state":"missing","market_state":"available","actual_submission":false,"evaluated_at":"unix:100","evaluated_at_unix_ms":100}
{"schema_version":"ntpro.v09_risk_decision.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","decision_id":"risk:2","intent_id":"intent-2","symbol":"BTCUSDT.BINANCE","decision":"rejected","reasons":["order_submission_disabled","shadow_mode_actual_submission_disabled"],"mode":"shadow","order_submission":"disabled","kill_switch_enabled":true,"kill_switch_active":false,"account_state":"missing","market_state":"available","actual_submission":false,"evaluated_at":"unix:200","evaluated_at_unix_ms":200}
"#,
        )
        .unwrap();
        fs::write(
            strategy_root.join("summary.json"),
            r#"{
  "schema_version": "ntpro.v09_strategy_session_summary.v1",
  "session_id": "btc-ema-shadow-001",
  "strategy_id": "ema_cross_btcusdt_v1",
  "state": "stopped",
  "event_count": 10,
  "market_event_count": 8,
  "signal_count": 2,
  "intent_count": 2,
  "risk_decision_count": 2,
  "rejection_count": 2,
  "actual_submission_count": 0,
  "updated_at_unix_ms": 4
}
"#,
        )
        .unwrap();
        strategy_root
    }

    #[test]
    fn supervisor_reads_strategy_session_artifact_status() {
        let root = temp_root("strategy-artifacts");
        let strategy_root = write_strategy_artifacts(&root);

        let status = strategy_session_status_from_artifact_root(&strategy_root, Some("stopped"));

        assert_eq!(status.session_state, "stopped");
        assert_eq!(status.strategy_id, "ema_cross_btcusdt_v1");
        assert_eq!(status.market_state, "exhausted");
        assert_eq!(status.risk_state, "rejected");
        assert_eq!(status.artifact_health, "degraded");
        assert!(
            status
                .artifact_diagnostic
                .contains("strategy manifest missing")
        );
        assert_eq!(status.last_signal_at, "unix:200");
        assert_eq!(
            status.last_rejection_reason,
            "order_submission_disabled+shadow_mode_actual_submission_disabled"
        );
        assert!(status.session_status_path.ends_with("session_status.json"));
        assert!(status.events_path.ends_with("events.jsonl"));
        assert!(status.summary_path.ends_with("summary.json"));
    }

    #[test]
    fn node_metrics_expose_strategy_signal_and_rejection_counts() {
        let mut status = NodeStatus::unknown("btc-ema-shadow-001");
        status.risk.command_count = SnapshotValue::available(2);
        status.risk.rejections_total = SnapshotValue::available(2);
        let artifacts = NodeMetricArtifacts {
            status_path: PathBuf::from("status.json"),
            stdout_log_path: PathBuf::from("stdout.log"),
            stderr_log_path: PathBuf::from("stderr.log"),
            events_log_path: PathBuf::from("events.log"),
            kill_switch_approval_artifact_path: PathBuf::from(
                "v0_13/kill_switch_approval_artifact.json",
            ),
        };

        let metrics = NodeMetrics::from_status(
            &status,
            &artifacts,
            NodeMetricCounts {
                uptime_ms: Some(10),
                starts_total: 1,
                stops_total: 1,
                state_transitions_total: 2,
            },
        );

        assert_eq!(metrics.strategy_signal_count.value, Some(2));
        assert_eq!(metrics.strategy_rejection_count.value, Some(2));
        assert_eq!(
            metrics.kill_switch_dry_run.artifact_status.availability,
            nautilus_live::status::SnapshotAvailability::NotConfigured
        );
        assert!(!metrics.external_venue_connection);
        assert!(!metrics.real_orders_submitted);
    }

    #[test]
    fn node_metrics_expose_kill_switch_dry_run_artifact_status() {
        let root = temp_root("kill-switch-metrics");
        let record = SupervisorNodeRecord::new(
            "live-alpha-a".to_string(),
            root.join("config.toml"),
            root.join("node-artifacts"),
        );
        fs::create_dir_all(record.artifact_root.join("v0_13")).unwrap();
        fs::write(
            record
                .artifact_root
                .join("v0_13/kill_switch_approval_artifact.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION,
                "status": "manual_approval_recorded",
                "kill_switch_active": true,
                "kill_switch_dry_run": true,
                "manual_approval_recorded": true,
                "approval_state": "approved",
                "production_order_submission_allowed": false,
                "production_order_mutation_allowed": false,
                "production_order_state_reads_allowed": false,
                "listen_key_lifecycle_allowed": false,
                "production_order_submissions_attempted": 0,
                "production_orders_submitted": 0,
                "production_order_mutations_attempted": 0,
                "production_order_state_reads_attempted": 0,
                "dashboard_order_controls_enabled": false,
                "real_orders_submitted": false,
                "network_attempted": false,
                "values_are_exchange_truth": false
            }))
            .unwrap(),
        )
        .unwrap();
        let status = NodeStatus::unknown("live-alpha-a");

        let metrics = NodeMetrics::from_status(
            &status,
            &NodeMetricArtifacts::from_record(&record),
            NodeMetricCounts {
                uptime_ms: Some(0),
                starts_total: 0,
                stops_total: 0,
                state_transitions_total: 0,
            },
        );

        assert_eq!(
            metrics.kill_switch_dry_run.artifact_status.value.as_deref(),
            Some("manual_approval_recorded")
        );
        assert_eq!(
            metrics.kill_switch_dry_run.kill_switch_active.value,
            Some(true)
        );
        assert_eq!(
            metrics.kill_switch_dry_run.kill_switch_dry_run.value,
            Some(true)
        );
        assert_eq!(
            metrics
                .kill_switch_dry_run
                .production_order_mutation_allowed
                .value,
            Some(false)
        );
        assert_eq!(
            metrics
                .kill_switch_dry_run
                .production_order_mutations_attempted
                .value,
            Some(0)
        );
        assert_eq!(
            metrics
                .kill_switch_dry_run
                .dashboard_order_controls_enabled
                .value,
            Some(false)
        );
    }

    fn start_request(
        node_id: &str,
        ntpro_node_bin: PathBuf,
        startup_timeout: Duration,
    ) -> StartNodeRequest {
        StartNodeRequest {
            node_id: node_id.to_string(),
            ntpro_node_bin,
            startup_timeout,
            node_max_runtime: Duration::from_secs(60),
            node_heartbeat_interval: Duration::from_millis(100),
            node_parent_pid: None,
            node_shutdown_timeout: Duration::from_secs(3),
        }
    }

    fn temp_artifacts(root: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    paths.extend(temp_artifacts(&path));
                } else if path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().contains(".tmp."))
                {
                    paths.push(path);
                }
            }
        }
        paths
    }

    fn assert_no_temp_artifacts(root: &Path) {
        let temp_paths = temp_artifacts(root);
        assert!(
            temp_paths.is_empty(),
            "unexpected temp files: {temp_paths:?}"
        );
    }

    #[cfg(unix)]
    fn write_fixture_node(root: &Path) -> PathBuf {
        let path = root.join("fixture-ntpro-node.sh");
        fs::write(
            &path,
            r#"#!/bin/sh
set -eu
node_id=""
output=""
stop_file=""
write_atomic() {
  target="$1"
  tmp="$target.tmp.$$"
  cat > "$tmp"
  mv "$tmp" "$target"
}
while [ "$#" -gt 0 ]; do
  case "$1" in
    --run-id) node_id="$2"; shift 2 ;;
    --output) output="$2"; shift 2 ;;
    --stop-file) stop_file="$2"; shift 2 ;;
    --max-runtime-ms) shift 2 ;;
    --heartbeat-interval-ms) shift 2 ;;
    --parent-pid) shift 2 ;;
    --shutdown-timeout-ms) shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$output/logs"
echo "fixture stdout started node_id=$node_id"
echo "fixture stderr initialized node_id=$node_id" >&2
cat > "$output/logs/events.log" <<EOF
phase=start status=ok node_id=$node_id
EOF
write_atomic "$output/status.json" <<EOF
{
  "schema_version": "ntpro.node_status.v1",
  "node_id": "$node_id",
  "process_mode": "spawned_process",
  "config_path": {"availability": "available", "value": "fixture.toml"},
  "artifact_root": {"availability": "available", "value": "$output"},
  "lifecycle_state": "running",
  "previous_lifecycle_state": "starting",
  "data_connection": "not_configured",
  "execution_connection": "disconnected",
  "execution": {
    "gateway_id": {"availability": "available", "value": "SANDBOX"},
    "connection": "disconnected",
    "started": {"availability": "available", "value": true},
    "account_ref": {"availability": "available", "value": "configured"},
    "orders_open": {"availability": "unknown"},
    "orders_inflight": {"availability": "unknown"},
    "orders_closed": {"availability": "unknown"},
    "last_report_at": {"availability": "unknown"},
    "last_reconciliation_at": {"availability": "unknown"},
    "last_error": null
  },
  "risk": {
    "trading_state": "unknown",
    "health": "unknown",
    "command_count": {"availability": "unknown"},
    "event_count": {"availability": "unknown"},
    "rejections_total": {"availability": "unknown"},
    "last_rejection": null,
    "last_error": null
  },
  "generated_at": {"availability": "unknown"},
  "started_at": {"availability": "unknown"},
  "stopped_at": {"availability": "unknown"},
  "last_transition_at": {"availability": "unknown"},
  "last_error": null,
  "external_venue_connection": false,
  "real_orders_submitted": false
}
EOF
write_atomic "$output/metrics.json" <<EOF
{
  "schema_version": "ntpro.node_metrics.v1",
  "node_id": "$node_id",
  "lifecycle_state": "running",
  "previous_lifecycle_state": "starting",
  "process_mode": "spawned_process",
  "uptime_ms": {"availability": "available", "value": 0},
  "starts_total": 1,
  "stops_total": 0,
  "state_transitions_total": 1,
  "connection_counts": {
    "data_connected": 0,
    "data_disconnected": 0,
    "data_not_configured": 1,
    "execution_connected": 0,
    "execution_disconnected": 1,
    "execution_not_configured": 0
  },
  "last_error_summary": null,
  "generated_at": {"availability": "available", "value": "1"},
  "started_at": {"availability": "available", "value": "1"},
  "stopped_at": {"availability": "unknown"},
  "status_artifact_path": {"availability": "available", "value": "$output/status.json"},
  "stdout_log_path": {"availability": "available", "value": "$output/logs/stdout.log"},
  "stderr_log_path": {"availability": "available", "value": "$output/logs/stderr.log"},
  "events_log_path": {"availability": "available", "value": "$output/logs/events.log"},
  "external_venue_connection": false,
  "real_orders_submitted": false
}
EOF
if [ -f "$output/ignore-stop" ]; then
  trap 'echo term > "$output/term.signal"; exit 0' TERM
  while :; do
    sleep 0.05
  done
fi
while [ ! -f "$stop_file" ]; do
  sleep 0.05
done
cat >> "$output/logs/events.log" <<EOF
phase=stop status=ok node_id=$node_id
EOF
write_atomic "$output/status.json" <<EOF
{
  "schema_version": "ntpro.node_status.v1",
  "node_id": "$node_id",
  "process_mode": "spawned_process",
  "config_path": {"availability": "available", "value": "fixture.toml"},
  "artifact_root": {"availability": "available", "value": "$output"},
  "lifecycle_state": "stopped",
  "previous_lifecycle_state": "running",
  "data_connection": "not_configured",
  "execution_connection": "disconnected",
  "execution": {
    "gateway_id": {"availability": "available", "value": "SANDBOX"},
    "connection": "disconnected",
    "started": {"availability": "available", "value": false},
    "account_ref": {"availability": "available", "value": "configured"},
    "orders_open": {"availability": "unknown"},
    "orders_inflight": {"availability": "unknown"},
    "orders_closed": {"availability": "unknown"},
    "last_report_at": {"availability": "unknown"},
    "last_reconciliation_at": {"availability": "unknown"},
    "last_error": null
  },
  "risk": {
    "trading_state": "unknown",
    "health": "unknown",
    "command_count": {"availability": "unknown"},
    "event_count": {"availability": "unknown"},
    "rejections_total": {"availability": "unknown"},
    "last_rejection": null,
    "last_error": null
  },
  "generated_at": {"availability": "unknown"},
  "started_at": {"availability": "unknown"},
  "stopped_at": {"availability": "unknown"},
  "last_transition_at": {"availability": "unknown"},
  "last_error": null,
  "external_venue_connection": false,
  "real_orders_submitted": false
}
EOF
write_atomic "$output/metrics.json" <<EOF
{
  "schema_version": "ntpro.node_metrics.v1",
  "node_id": "$node_id",
  "lifecycle_state": "stopped",
  "previous_lifecycle_state": "running",
  "process_mode": "spawned_process",
  "uptime_ms": {"availability": "available", "value": 1},
  "starts_total": 1,
  "stops_total": 1,
  "state_transitions_total": 2,
  "connection_counts": {
    "data_connected": 0,
    "data_disconnected": 0,
    "data_not_configured": 1,
    "execution_connected": 0,
    "execution_disconnected": 1,
    "execution_not_configured": 0
  },
  "last_error_summary": null,
  "generated_at": {"availability": "available", "value": "2"},
  "started_at": {"availability": "available", "value": "1"},
  "stopped_at": {"availability": "available", "value": "2"},
  "status_artifact_path": {"availability": "available", "value": "$output/status.json"},
  "stdout_log_path": {"availability": "available", "value": "$output/logs/stdout.log"},
  "stderr_log_path": {"availability": "available", "value": "$output/logs/stderr.log"},
  "events_log_path": {"availability": "available", "value": "$output/logs/events.log"},
  "external_venue_connection": false,
  "real_orders_submitted": false
}
EOF
"#,
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[cfg(unix)]
    fn write_early_exit_node(root: &Path) -> PathBuf {
        let path = root.join("early-exit-ntpro-node.sh");
        fs::write(&path, "#!/bin/sh\nexit 23\n").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[cfg(unix)]
    fn write_hanging_node(root: &Path) -> PathBuf {
        let path = root.join("hanging-ntpro-node.sh");
        fs::write(
            &path,
            r#"#!/bin/sh
set -eu
output=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output) output="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$output"
while :; do
  sleep 1
done
"#,
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[test]
    fn register_update_list_and_remove_node_records() {
        let root = temp_root("crud");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config_a = write_config(&root, "sandbox-a");
        let config_b = write_config(&root, "sandbox-b");

        let first = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config_a.clone(),
                artifact_root: None,
            })
            .unwrap();
        assert_eq!(first.node_id, "sandbox-a");
        assert_eq!(first.config_path, config_a);
        assert_eq!(first.process.state, SupervisorProcessState::NotStarted);
        assert_eq!(first.status_artifact, RegistryArtifactState::Missing);
        assert_eq!(first.metrics_artifact, RegistryArtifactState::Missing);
        assert!(first.stdout_log_path.ends_with("logs/stdout.log"));

        let second_root = root.join("custom-b");
        let second = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-b".to_string(),
                config_path: config_b,
                artifact_root: Some(second_root.clone()),
            })
            .unwrap();
        assert_eq!(second.artifact_root, second_root);

        let nodes = store.list_nodes().unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].node_id, "sandbox-a");
        assert_eq!(nodes[1].node_id, "sandbox-b");

        let removed = store.remove_node("sandbox-a").unwrap().unwrap();
        assert_eq!(removed.node_id, "sandbox-a");
        assert_eq!(store.list_nodes().unwrap().len(), 1);
        assert!(!store.registry_lock_path().exists());
        assert_no_temp_artifacts(&root);
    }

    #[test]
    fn concurrent_registers_are_serialized_by_registry_lock() {
        let root = temp_root("concurrent-register");
        let store = Arc::new(SupervisorRegistryStore::new(root.join("registry.json")));
        let node_ids = (0..8)
            .map(|idx| format!("sandbox-{idx}"))
            .collect::<Vec<_>>();
        let configs = node_ids
            .iter()
            .map(|node_id| write_config(&root, node_id))
            .collect::<Vec<_>>();
        let barrier = Arc::new(Barrier::new(node_ids.len()));
        let handles = node_ids
            .iter()
            .cloned()
            .zip(configs)
            .map(|(node_id, config_path)| {
                let store = Arc::clone(&store);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    store
                        .register_node(RegisterNodeRequest {
                            node_id,
                            config_path,
                            artifact_root: None,
                        })
                        .unwrap();
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }

        let registry = store.load().unwrap();
        assert_eq!(registry.nodes.len(), 8);
        for node_id in node_ids {
            assert!(registry.nodes.contains_key(&node_id));
        }
        assert!(!store.registry_lock_path().exists());
        assert_no_temp_artifacts(&root);
    }

    #[test]
    fn stale_registry_lock_owned_by_dead_process_is_recovered() {
        let root = temp_root("stale-registry-lock");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let lock_path = store.registry_lock_path();
        let artifact = RegistryLockArtifact {
            schema_version: SUPERVISOR_REGISTRY_LOCK_SCHEMA_VERSION.to_string(),
            pid: u32::MAX,
            acquired_at: "1".to_string(),
        };
        fs::write(
            &lock_path,
            format!("{}\n", serde_json::to_string_pretty(&artifact).unwrap()),
        )
        .unwrap();

        {
            let _lock = store
                .acquire_registry_lock_with_timeout(Duration::from_millis(100))
                .unwrap();
            let recovered: RegistryLockArtifact =
                serde_json::from_str(&fs::read_to_string(&lock_path).unwrap()).unwrap();
            assert_eq!(
                recovered.schema_version,
                SUPERVISOR_REGISTRY_LOCK_SCHEMA_VERSION
            );
            assert_eq!(recovered.pid, process::id());
            assert!(process_is_alive(recovered.pid));
        }

        assert!(!lock_path.exists());
        assert_no_temp_artifacts(&root);
    }

    #[test]
    fn active_registry_lock_is_refused_after_timeout() {
        let root = temp_root("active-registry-lock");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let lock_path = store.registry_lock_path();
        let artifact = RegistryLockArtifact::new(process::id());
        fs::write(
            &lock_path,
            format!("{}\n", serde_json::to_string_pretty(&artifact).unwrap()),
        )
        .unwrap();

        let error = store
            .acquire_registry_lock_with_timeout(Duration::from_millis(50))
            .unwrap_err()
            .to_string();

        assert!(error.contains("timed out waiting for supervisor registry lock"));
        assert!(error.contains(&format!("active pid={}", process::id())));
        assert!(lock_path.exists());
        fs::remove_file(&lock_path).unwrap();
        assert_no_temp_artifacts(&root);
    }

    #[test]
    fn rejects_invalid_node_id_and_missing_config() {
        let root = temp_root("reject");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));

        let invalid = store
            .register_node(RegisterNodeRequest {
                node_id: "Bad Node".to_string(),
                config_path: root.join("missing.toml"),
                artifact_root: None,
            })
            .unwrap_err()
            .to_string();
        assert!(invalid.contains("node_id must start with"));

        let missing = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: root.join("missing.toml"),
                artifact_root: None,
            })
            .unwrap_err()
            .to_string();
        assert!(missing.contains("does not exist"));
    }

    #[test]
    fn control_actions_reject_missing_nodes_and_invalid_lifecycle() {
        let root = temp_root("negative-control");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let missing_pause = store.pause_node("missing").unwrap_err().to_string();
        assert!(missing_pause.contains("node 'missing' is not registered"));

        let missing_resume = store.resume_node("missing").unwrap_err().to_string();
        assert!(missing_resume.contains("node 'missing' is not registered"));

        let missing_reconnect = store
            .reconnect_data_source("missing")
            .unwrap_err()
            .to_string();
        assert!(missing_reconnect.contains("node 'missing' is not registered"));

        let missing_stop = store
            .stop_node_process(&StopNodeRequest {
                node_id: "missing".to_string(),
                stop_timeout: Duration::from_millis(1),
            })
            .unwrap_err()
            .to_string();
        assert!(missing_stop.contains("node 'missing' is not registered"));

        let stopped_stop = store
            .stop_node_process(&StopNodeRequest {
                node_id: "sandbox-a".to_string(),
                stop_timeout: Duration::from_millis(1),
            })
            .unwrap_err()
            .to_string();
        assert!(stopped_stop.contains("node 'sandbox-a' is not running"));

        let stopped_pause = store.pause_node("sandbox-a").unwrap_err().to_string();
        assert!(stopped_pause.contains("node 'sandbox-a' process is not running"));

        let stopped_resume = store.resume_node("sandbox-a").unwrap_err().to_string();
        assert!(stopped_resume.contains("node 'sandbox-a' process is not running"));

        let mut stopped_status = record.last_known_status.clone();
        stopped_status.lifecycle_state = LifecycleStatus::Stopped;
        if let Some(parent) = record.status_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(
            &record.status_path,
            serde_json::to_string_pretty(&stopped_status).unwrap(),
        )
        .unwrap();
        store
            .update_process(
                "sandbox-a",
                Some(std::process::id()),
                SupervisorProcessState::Running,
            )
            .unwrap();

        let running_process_pause = store.pause_node("sandbox-a").unwrap_err().to_string();
        assert!(running_process_pause.contains("lifecycle state is stopped, expected running"));

        let running_process_resume = store.resume_node("sandbox-a").unwrap_err().to_string();
        assert!(running_process_resume.contains("lifecycle state is stopped, expected paused"));

        let running_process_reconnect_data = store
            .reconnect_data_source("sandbox-a")
            .unwrap_err()
            .to_string();
        assert!(
            running_process_reconnect_data
                .contains("lifecycle state is stopped, expected running or paused")
        );

        let running_process_reconnect_execution = store
            .reconnect_execution_gateway("sandbox-a")
            .unwrap_err()
            .to_string();
        assert!(
            running_process_reconnect_execution
                .contains("lifecycle state is stopped, expected running or paused")
        );
    }

    #[test]
    fn refresh_status_reads_available_artifact() {
        let root = temp_root("status");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let mut status = NodeStatus::unknown("sandbox-a");
        status.lifecycle_state = LifecycleStatus::Running;
        status.process_mode = ProcessMode::SpawnedProcess;
        fs::write(
            &record.status_path,
            serde_json::to_string_pretty(&status).unwrap(),
        )
        .unwrap();

        let refreshed = store.refresh_status_from_artifact("sandbox-a").unwrap();
        assert_eq!(refreshed.status_artifact, RegistryArtifactState::Available);
        assert_eq!(
            refreshed.last_known_status.lifecycle_state,
            LifecycleStatus::Running
        );
        assert_eq!(
            refreshed.last_known_status.process_mode,
            ProcessMode::SpawnedProcess
        );
    }

    #[test]
    fn refresh_status_marks_missing_or_invalid_artifact() {
        let root = temp_root("missing-status");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let missing = store.refresh_status_from_artifact("sandbox-a").unwrap();
        assert_eq!(missing.status_artifact, RegistryArtifactState::Missing);
        assert_eq!(
            missing.last_known_status.generated_at.availability,
            nautilus_live::status::SnapshotAvailability::Stale
        );

        fs::write(&record.status_path, "not-json").unwrap();
        let invalid = store.refresh_status_from_artifact("sandbox-a").unwrap();
        assert_eq!(invalid.status_artifact, RegistryArtifactState::Invalid);
        assert!(
            invalid
                .last_known_status
                .last_error
                .as_deref()
                .unwrap()
                .contains("invalid status artifact")
        );
    }

    #[test]
    fn refresh_status_rejects_runtime_identity_mismatch() {
        let root = temp_root("status-identity-mismatch");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let mut status = NodeStatus::unknown("sandbox-b");
        status.lifecycle_state = LifecycleStatus::Running;
        fs::write(
            &record.status_path,
            serde_json::to_string_pretty(&status).unwrap(),
        )
        .unwrap();

        let refreshed = store.refresh_status_from_artifact("sandbox-a").unwrap();
        assert_eq!(refreshed.status_artifact, RegistryArtifactState::Invalid);
        assert_eq!(refreshed.last_known_status.node_id, "sandbox-a");
        assert_ne!(
            refreshed.last_known_status.lifecycle_state,
            LifecycleStatus::Running
        );
        assert!(
            refreshed
                .last_known_status
                .last_error
                .as_deref()
                .unwrap()
                .contains(
                    "status node identity mismatch: registry node 'sandbox-a' received runtime node 'sandbox-b'"
                )
        );
        let error = store.node_status("sandbox-a").unwrap_err().to_string();
        assert!(error.contains(
            "invalid status artifact for registry node 'sandbox-a': status node identity mismatch"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn start_replaces_stale_identity_artifacts() {
        let _process_test_guard = supervisor_process_test_guard();
        let root = temp_root("replace-stale-identity");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let fixture = write_fixture_node(&root);
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let stale_status = NodeStatus::unknown("legacy-display-name");
        fs::write(
            &record.status_path,
            serde_json::to_string_pretty(&stale_status).unwrap(),
        )
        .unwrap();
        let stale_metrics = NodeMetrics::from_status(
            &stale_status,
            &NodeMetricArtifacts::from_record(&record),
            NodeMetricCounts {
                uptime_ms: Some(0),
                starts_total: 1,
                stops_total: 1,
                state_transitions_total: 2,
            },
        );
        write_node_metrics_artifact(&record.metrics_path, &stale_metrics).unwrap();

        let started = store
            .start_node_process(&start_request("sandbox-a", fixture, Duration::from_secs(3)))
            .unwrap();
        assert_eq!(started.last_known_status.node_id, "sandbox-a");
        assert_eq!(started.status_artifact, RegistryArtifactState::Available);
        assert_eq!(
            store.node_metrics("sandbox-a").unwrap().node_id,
            "sandbox-a"
        );

        store
            .stop_node_process(&StopNodeRequest {
                node_id: "sandbox-a".to_string(),
                stop_timeout: Duration::from_secs(3),
            })
            .unwrap();
    }

    #[test]
    fn node_metrics_rejects_runtime_identity_mismatch() {
        let root = temp_root("metrics-identity-mismatch");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let status = NodeStatus::unknown("sandbox-b");
        let metrics = NodeMetrics::from_status(
            &status,
            &NodeMetricArtifacts::from_record(&record),
            NodeMetricCounts {
                uptime_ms: Some(0),
                starts_total: 1,
                stops_total: 0,
                state_transitions_total: 1,
            },
        );
        write_node_metrics_artifact(&record.metrics_path, &metrics).unwrap();

        let error = store.node_metrics("sandbox-a").unwrap_err().to_string();
        assert!(error.contains(
            "metrics node identity mismatch: registry node 'sandbox-a' received runtime node 'sandbox-b'"
        ));
        let registry = store.load().unwrap();
        assert_eq!(
            registry.nodes["sandbox-a"].metrics_artifact,
            RegistryArtifactState::Invalid
        );
    }

    #[test]
    fn update_process_writes_pid_artifact_and_dead_or_missing_process_marks_stale() {
        let root = temp_root("stale-pid");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let registered = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();
        store
            .update_process("sandbox-a", Some(123_456), SupervisorProcessState::Running)
            .unwrap();

        let pid_artifact: SupervisorPidArtifact =
            serde_json::from_str(&fs::read_to_string(&registered.pid_path).unwrap()).unwrap();
        assert_eq!(pid_artifact.node_id, "sandbox-a");
        assert_eq!(pid_artifact.pid, 123_456);
        assert!(!store.registry_lock_path().exists());
        assert_no_temp_artifacts(&root);

        let dead = store.refresh_process_state("sandbox-a").unwrap();
        assert_eq!(dead.process.state, SupervisorProcessState::Stale);

        store
            .update_process("sandbox-a", Some(123_456), SupervisorProcessState::Running)
            .unwrap();
        fs::remove_file(&registered.pid_path).unwrap();
        let refreshed = store.refresh_process_state("sandbox-a").unwrap();
        assert_eq!(refreshed.process.state, SupervisorProcessState::Stale);
        assert!(!store.registry_lock_path().exists());
    }

    #[test]
    fn refresh_process_state_accepts_graceful_external_stopped_status() {
        let root = temp_root("external-stopped-status");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();
        store
            .update_process("sandbox-a", Some(123_456), SupervisorProcessState::Running)
            .unwrap();

        let mut stopped_status = NodeStatus::unknown("sandbox-a");
        stopped_status.lifecycle_state = LifecycleStatus::Stopped;
        stopped_status.process_mode = ProcessMode::SpawnedProcess;
        stopped_status.artifact_root =
            SnapshotValue::available(record.artifact_root.display().to_string());
        fs::write(
            &record.status_path,
            serde_json::to_string_pretty(&stopped_status).unwrap(),
        )
        .unwrap();

        let refreshed = store.refresh_process_state("sandbox-a").unwrap();
        assert_eq!(refreshed.process.state, SupervisorProcessState::Stopped);
        assert!(refreshed.process.pid.value.is_none());
        assert!(!refreshed.pid_path.exists());
        assert_eq!(
            refreshed.last_known_status.lifecycle_state,
            LifecycleStatus::Stopped
        );
        assert_eq!(refreshed.status_artifact, RegistryArtifactState::Available);
    }

    #[test]
    fn supervisor_shadow_preflight_summary_reports_lifecycle_and_no_mutation_boundary() {
        let root = temp_root("shadow-preflight-summary");
        let record = SupervisorNodeRecord::new(
            "shadow-a".to_string(),
            root.join("config.toml"),
            root.join("shadow-a-artifacts"),
        );
        let preflight_path = record
            .artifact_root
            .join(SHADOW_PREFLIGHT_SESSION_RELATIVE_PATH);
        fs::create_dir_all(preflight_path.parent().unwrap()).unwrap();
        fs::write(
            &preflight_path,
            r#"{"event_type":"shadow_preflight_session_started","state":"running","stale_data_detected":false,"stop_file_observed":false,"production_order_submissions_attempted":0,"production_order_mutations_attempted":0,"production_order_state_reads_attempted":0,"listen_key_lifecycle_attempted":0,"dashboard_order_controls_enabled":false}
{"event_type":"shadow_preflight_session_heartbeat","state":"running","heartbeat_seq":1,"stale_data_detected":false,"stop_file_observed":false,"production_order_submissions_attempted":0,"production_order_mutations_attempted":0,"production_order_state_reads_attempted":0,"listen_key_lifecycle_attempted":0,"dashboard_order_controls_enabled":false}
{"event_type":"shadow_preflight_stale_data_detected","state":"stale_data_halted","stale_data_detected":true,"stop_file_observed":false,"production_order_submissions_attempted":0,"production_order_mutations_attempted":0,"production_order_state_reads_attempted":0,"listen_key_lifecycle_attempted":0,"dashboard_order_controls_enabled":false}
"#,
        )
        .unwrap();

        let summary = shadow_preflight_summary_from_record(&record);

        assert_eq!(summary.status, "available");
        assert_eq!(summary.event_count, 3);
        assert_eq!(summary.heartbeat_count, 1);
        assert_eq!(summary.final_state, "stale_data_halted");
        assert!(summary.stale_data_halted);
        assert!(!summary.stop_file_observed);
        assert_eq!(summary.production_order_submissions_attempted, 0);
        assert_eq!(summary.production_order_mutations_attempted, 0);
        assert_eq!(summary.production_order_state_reads_attempted, 0);
        assert_eq!(summary.listen_key_lifecycle_attempted, 0);
        assert!(!summary.dashboard_order_controls_enabled);
    }

    #[test]
    fn refresh_process_state_accepts_matching_process_identity() {
        let root = temp_root("process-identity-match");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();
        store
            .update_process(
                "sandbox-a",
                Some(process::id()),
                SupervisorProcessState::Running,
            )
            .unwrap();

        let mut pid_artifact: SupervisorPidArtifact =
            serde_json::from_str(&fs::read_to_string(&record.pid_path).unwrap()).unwrap();
        pid_artifact.process_identity.as_mut().unwrap().started_at =
            SnapshotValue::available("identity-1".to_string());
        fs::write(
            &record.pid_path,
            serde_json::to_string_pretty(&pid_artifact).unwrap(),
        )
        .unwrap();
        let mut status = NodeStatus::unknown("sandbox-a");
        status.lifecycle_state = LifecycleStatus::Running;
        status.artifact_root = SnapshotValue::available(record.artifact_root.display().to_string());
        status.started_at = SnapshotValue::available("identity-1".to_string());
        fs::write(
            &record.status_path,
            serde_json::to_string_pretty(&status).unwrap(),
        )
        .unwrap();

        let refreshed = store.refresh_process_state("sandbox-a").unwrap();
        assert_eq!(refreshed.process.state, SupervisorProcessState::Running);
    }

    #[test]
    fn refresh_process_state_marks_identity_mismatch_stale() {
        let root = temp_root("process-identity-mismatch");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();
        store
            .update_process(
                "sandbox-a",
                Some(process::id()),
                SupervisorProcessState::Running,
            )
            .unwrap();

        let mut pid_artifact: SupervisorPidArtifact =
            serde_json::from_str(&fs::read_to_string(&record.pid_path).unwrap()).unwrap();
        pid_artifact.process_identity.as_mut().unwrap().started_at =
            SnapshotValue::available("identity-1".to_string());
        fs::write(
            &record.pid_path,
            serde_json::to_string_pretty(&pid_artifact).unwrap(),
        )
        .unwrap();
        let mut status = NodeStatus::unknown("sandbox-a");
        status.lifecycle_state = LifecycleStatus::Running;
        status.artifact_root = SnapshotValue::available(record.artifact_root.display().to_string());
        status.started_at = SnapshotValue::available("identity-2".to_string());
        fs::write(
            &record.status_path,
            serde_json::to_string_pretty(&status).unwrap(),
        )
        .unwrap();

        let refreshed = store.refresh_process_state("sandbox-a").unwrap();
        assert_eq!(refreshed.process.state, SupervisorProcessState::Stale);
    }

    #[cfg(unix)]
    #[test]
    fn start_status_stop_process_roundtrip() {
        let _process_test_guard = supervisor_process_test_guard();
        let root = temp_root("process");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let fixture = write_fixture_node(&root);
        store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let initial_start_request = start_request("sandbox-a", fixture, Duration::from_secs(3));
        let started = store.start_node_process(&initial_start_request).unwrap();
        let started_pid = started.process.pid.value.unwrap();
        assert_eq!(started.process.state, SupervisorProcessState::Running);
        assert!(process_is_alive(started_pid));
        assert_eq!(
            started.last_known_status.lifecycle_state,
            LifecycleStatus::Running
        );

        let duplicate_start_request = start_request(
            "sandbox-a",
            root.join("fixture-ntpro-node.sh"),
            Duration::from_secs(1),
        );
        let duplicate_start = store
            .start_node_process(&duplicate_start_request)
            .unwrap_err()
            .to_string();
        assert!(duplicate_start.contains("already running"));

        let status = store.node_status("sandbox-a").unwrap();
        assert_eq!(status.lifecycle_state, LifecycleStatus::Running);
        assert!(!status.external_venue_connection);
        assert!(!status.real_orders_submitted);
        wait_for_metrics_state(&store, "sandbox-a", LifecycleStatus::Running);
        let running_metrics = store.node_metrics("sandbox-a").unwrap();
        assert_eq!(running_metrics.lifecycle_state, LifecycleStatus::Running);
        assert_eq!(running_metrics.starts_total, 1);
        assert_eq!(running_metrics.stops_total, 0);
        assert!(!running_metrics.external_venue_connection);
        assert!(!running_metrics.real_orders_submitted);

        let paused = store.pause_node("sandbox-a").unwrap();
        assert_eq!(paused.process.state, SupervisorProcessState::Running);
        assert!(process_is_alive(started_pid));
        assert_eq!(
            paused.last_known_status.lifecycle_state,
            LifecycleStatus::Paused
        );
        assert_eq!(
            paused.last_known_status.previous_lifecycle_state,
            LifecycleStatus::Running
        );
        let paused_status = store.node_status("sandbox-a").unwrap();
        assert_eq!(paused_status.lifecycle_state, LifecycleStatus::Paused);
        assert!(!paused_status.external_venue_connection);
        assert!(!paused_status.real_orders_submitted);
        let paused_metrics = store.node_metrics("sandbox-a").unwrap();
        assert_eq!(paused_metrics.lifecycle_state, LifecycleStatus::Paused);
        assert_eq!(paused_metrics.starts_total, 1);
        assert_eq!(paused_metrics.stops_total, 0);
        assert_eq!(paused_metrics.state_transitions_total, 2);

        let resumed = store.resume_node("sandbox-a").unwrap();
        assert_eq!(resumed.process.state, SupervisorProcessState::Running);
        assert_eq!(
            resumed.last_known_status.lifecycle_state,
            LifecycleStatus::Running
        );
        assert_eq!(
            resumed.last_known_status.previous_lifecycle_state,
            LifecycleStatus::Paused
        );
        let resumed_status = store.node_status("sandbox-a").unwrap();
        assert_eq!(resumed_status.lifecycle_state, LifecycleStatus::Running);
        let resumed_metrics = store.node_metrics("sandbox-a").unwrap();
        assert_eq!(resumed_metrics.lifecycle_state, LifecycleStatus::Running);
        assert_eq!(resumed_metrics.state_transitions_total, 3);

        let reconnected_data = store.reconnect_data_source("sandbox-a").unwrap();
        assert_eq!(
            reconnected_data.process.state,
            SupervisorProcessState::Running
        );
        assert_eq!(
            reconnected_data.last_known_status.lifecycle_state,
            LifecycleStatus::Running
        );
        assert_eq!(
            reconnected_data.last_known_status.data_connection,
            ConnectionStatus::NotSupported
        );
        assert!(!reconnected_data.last_known_status.external_venue_connection);
        assert!(!reconnected_data.last_known_status.real_orders_submitted);
        assert_eq!(
            reconnected_data.last_known_status.last_error.as_deref(),
            Some(DATA_RECONNECT_UNSUPPORTED_MESSAGE)
        );
        let reconnected_metrics = store.node_metrics("sandbox-a").unwrap();
        assert_eq!(
            reconnected_metrics.last_error_summary.as_deref(),
            Some(DATA_RECONNECT_UNSUPPORTED_MESSAGE)
        );
        assert_eq!(reconnected_metrics.state_transitions_total, 3);
        assert!(!reconnected_metrics.external_venue_connection);
        assert!(!reconnected_metrics.real_orders_submitted);

        let reconnected_execution = store.reconnect_execution_gateway("sandbox-a").unwrap();
        assert_eq!(
            reconnected_execution.process.state,
            SupervisorProcessState::Running
        );
        assert_eq!(
            reconnected_execution.last_known_status.lifecycle_state,
            LifecycleStatus::Running
        );
        assert_eq!(
            reconnected_execution.last_known_status.execution_connection,
            ConnectionStatus::NotSupported
        );
        assert_eq!(
            reconnected_execution.last_known_status.execution.connection,
            ConnectionStatus::NotSupported
        );
        assert!(
            !reconnected_execution
                .last_known_status
                .external_venue_connection
        );
        assert!(
            !reconnected_execution
                .last_known_status
                .real_orders_submitted
        );
        assert_eq!(
            reconnected_execution
                .last_known_status
                .execution
                .last_error
                .as_deref(),
            Some(EXECUTION_RECONNECT_UNSUPPORTED_MESSAGE)
        );
        let reconnected_execution_metrics = store.node_metrics("sandbox-a").unwrap();
        assert_eq!(
            reconnected_execution_metrics.last_error_summary.as_deref(),
            Some(EXECUTION_RECONNECT_UNSUPPORTED_MESSAGE)
        );
        assert_eq!(reconnected_execution_metrics.state_transitions_total, 3);
        assert!(!reconnected_execution_metrics.external_venue_connection);
        assert!(!reconnected_execution_metrics.real_orders_submitted);

        let mut registry = store.load().unwrap();
        let refreshed = registry.nodes.remove("sandbox-a").unwrap();
        assert_eq!(refreshed.metrics_artifact, RegistryArtifactState::Available);
        assert!(
            fs::read_to_string(&started.stdout_log_path)
                .unwrap()
                .contains("fixture stdout started")
        );
        assert!(
            fs::read_to_string(&started.stderr_log_path)
                .unwrap()
                .contains("fixture stderr initialized")
        );
        assert!(
            fs::read_to_string(&started.events_log_path)
                .unwrap()
                .contains("phase=start status=ok")
        );
        let events = fs::read_to_string(&started.events_log_path).unwrap();
        assert!(events.contains("phase=pause status=ok"));
        assert!(events.contains("phase=resume status=ok"));
        assert!(events.contains("phase=reconnect_data status=not_supported"));
        assert!(events.contains("phase=reconnect_execution status=not_supported"));

        let stopped = store
            .stop_node_process(&StopNodeRequest {
                node_id: "sandbox-a".to_string(),
                stop_timeout: Duration::from_secs(3),
            })
            .unwrap();
        assert_eq!(stopped.process.state, SupervisorProcessState::Stopped);
        assert!(!stopped.pid_path.exists());
        assert!(!process_is_alive(started_pid));
        assert_eq!(
            stopped.last_known_status.lifecycle_state,
            LifecycleStatus::Stopped
        );
        wait_for_metrics_state(&store, "sandbox-a", LifecycleStatus::Stopped);
        let stopped_metrics = store.node_metrics("sandbox-a").unwrap();
        assert_eq!(stopped_metrics.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(stopped_metrics.starts_total, 1);
        assert_eq!(stopped_metrics.stops_total, 1);

        let duplicate_stop = store
            .stop_node_process(&StopNodeRequest {
                node_id: "sandbox-a".to_string(),
                stop_timeout: Duration::from_secs(1),
            })
            .unwrap_err()
            .to_string();
        assert!(duplicate_stop.contains("not running"));

        let restarted = store
            .start_node_process(&start_request(
                "sandbox-a",
                root.join("fixture-ntpro-node.sh"),
                Duration::from_secs(3),
            ))
            .unwrap();
        let restarted_pid = restarted.process.pid.value.unwrap();
        assert_eq!(restarted.process.state, SupervisorProcessState::Running);
        assert!(process_is_alive(restarted_pid));
        assert_eq!(
            restarted.last_known_status.lifecycle_state,
            LifecycleStatus::Running
        );
        let restarted_status = store.node_status("sandbox-a").unwrap();
        assert_eq!(restarted_status.lifecycle_state, LifecycleStatus::Running);
        let restarted_stop = store
            .stop_node_process(&StopNodeRequest {
                node_id: "sandbox-a".to_string(),
                stop_timeout: Duration::from_secs(3),
            })
            .unwrap();
        assert_eq!(
            restarted_stop.last_known_status.lifecycle_state,
            LifecycleStatus::Stopped
        );
        assert!(!process_is_alive(restarted_pid));
    }

    #[cfg(unix)]
    #[test]
    fn stop_accepts_paused_running_process() {
        let _process_test_guard = supervisor_process_test_guard();
        let root = temp_root("stop-paused");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let fixture = write_fixture_node(&root);
        store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let started = store
            .start_node_process(&start_request("sandbox-a", fixture, Duration::from_secs(3)))
            .unwrap();
        let started_pid = started.process.pid.value.unwrap();
        store.pause_node("sandbox-a").unwrap();
        let paused_status = store.node_status("sandbox-a").unwrap();
        assert_eq!(paused_status.lifecycle_state, LifecycleStatus::Paused);

        let stopped = store
            .stop_node_process(&StopNodeRequest {
                node_id: "sandbox-a".to_string(),
                stop_timeout: Duration::from_secs(3),
            })
            .unwrap();
        assert_eq!(stopped.process.state, SupervisorProcessState::Stopped);
        assert_eq!(
            stopped.last_known_status.lifecycle_state,
            LifecycleStatus::Stopped
        );
        assert!(!process_is_alive(started_pid));
        let stopped_metrics = store.node_metrics("sandbox-a").unwrap();
        assert_eq!(stopped_metrics.lifecycle_state, LifecycleStatus::Stopped);
        assert_eq!(stopped_metrics.stops_total, 1);
        assert!(!stopped_metrics.real_orders_submitted);
    }

    #[cfg(unix)]
    #[test]
    fn start_rejects_child_that_exits_before_running_status() {
        let _process_test_guard = supervisor_process_test_guard();
        let root = temp_root("early-exit");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let fixture = write_early_exit_node(&root);
        store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let error = store
            .start_node_process(&start_request("sandbox-a", fixture, Duration::from_secs(3)))
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("exited before reaching running status"),
            "unexpected startup error: {error}"
        );
        let record = store.load().unwrap().nodes.remove("sandbox-a").unwrap();
        assert_eq!(record.process.state, SupervisorProcessState::Stale);
        assert!(record.process.pid.value.is_none());
        assert!(!record.pid_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn start_timeout_kills_child_and_marks_registry_stale() {
        let _process_test_guard = supervisor_process_test_guard();
        let root = temp_root("startup-timeout");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let fixture = write_hanging_node(&root);
        store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();

        let error = store
            .start_node_process(&start_request("sandbox-a", fixture, Duration::from_secs(1)))
            .unwrap_err()
            .to_string();

        assert!(error.contains("timed out waiting"));
        let child_pid = error
            .split(" process ")
            .nth(1)
            .and_then(|suffix| suffix.split_whitespace().next())
            .unwrap()
            .parse::<u32>()
            .unwrap();
        assert!(!process_is_alive(child_pid));
        let record = store.load().unwrap().nodes.remove("sandbox-a").unwrap();
        assert_eq!(record.process.state, SupervisorProcessState::Stale);
        assert!(record.process.pid.value.is_none());
        assert!(!record.pid_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn stop_escalates_but_rejects_missing_stopped_status() {
        let _process_test_guard = supervisor_process_test_guard();
        let root = temp_root("stop-escalation");
        let store = SupervisorRegistryStore::new(root.join("registry.json"));
        let config = write_config(&root, "sandbox-a");
        let fixture = write_fixture_node(&root);
        let registered = store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: None,
            })
            .unwrap();
        fs::write(registered.artifact_root.join("ignore-stop"), "").unwrap();

        let started = store
            .start_node_process(&start_request("sandbox-a", fixture, Duration::from_secs(3)))
            .unwrap();
        let pid = started.process.pid.value.unwrap();

        let error = store
            .stop_node_process(&StopNodeRequest {
                node_id: "sandbox-a".to_string(),
                stop_timeout: Duration::from_millis(200),
            })
            .unwrap_err()
            .to_string();

        assert!(error.contains("exited without a stopped status artifact"));
        assert!(registered.artifact_root.join("term.signal").exists());
        assert!(!process_is_alive(pid));
        let record = store.load().unwrap().nodes.remove("sandbox-a").unwrap();
        assert_eq!(record.process.state, SupervisorProcessState::Stale);
        assert!(record.process.pid.value.is_none());
        assert!(!record.pid_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn supervisor_command_handlers_control_fixture_node() {
        let _process_test_guard = supervisor_process_test_guard();
        let root = temp_root("commands");
        let registry = root.join("registry.json");
        let config = write_config(&root, "sandbox-a");
        let fixture = write_fixture_node(&root);
        let artifact_root = root.join("sandbox-a-artifacts");
        let registry_opt = SupervisorRegistryOpt {
            registry: registry.clone(),
        };

        run_supervisor_command(SupervisorOpt {
            command: SupervisorCommand::Register(SupervisorRegisterOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
                config,
                artifact_root: Some(artifact_root.clone()),
            }),
        })
        .unwrap();
        run_supervisor_command(SupervisorOpt {
            command: SupervisorCommand::List(SupervisorListOpt {
                registry: registry_opt.clone(),
            }),
        })
        .unwrap();
        run_supervisor_command(SupervisorOpt {
            command: SupervisorCommand::Start(SupervisorStartOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
                ntpro_node_bin: fixture,
                startup_timeout_ms: 3_000,
                node_max_runtime_ms: 60_000,
                node_heartbeat_interval_ms: 100,
                node_parent_pid: None,
                node_shutdown_timeout_ms: 3_000,
            }),
        })
        .unwrap();

        let store = SupervisorRegistryStore::new(registry);
        wait_for_metrics_state(&store, "sandbox-a", LifecycleStatus::Running);

        run_supervisor_command(SupervisorOpt {
            command: SupervisorCommand::Pause(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
        })
        .unwrap();
        assert_eq!(
            store.node_status("sandbox-a").unwrap().lifecycle_state,
            LifecycleStatus::Paused
        );
        run_supervisor_command(SupervisorOpt {
            command: SupervisorCommand::Resume(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
        })
        .unwrap();
        assert_eq!(
            store.node_status("sandbox-a").unwrap().lifecycle_state,
            LifecycleStatus::Running
        );
        run_supervisor_command(SupervisorOpt {
            command: SupervisorCommand::ReconnectData(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
        })
        .unwrap();
        assert_eq!(
            store.node_status("sandbox-a").unwrap().data_connection,
            ConnectionStatus::NotSupported
        );
        run_supervisor_command(SupervisorOpt {
            command: SupervisorCommand::ReconnectExecution(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
        })
        .unwrap();
        assert_eq!(
            store.node_status("sandbox-a").unwrap().execution_connection,
            ConnectionStatus::NotSupported
        );

        for command in [
            SupervisorCommand::Status(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
            SupervisorCommand::Connections(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
            SupervisorCommand::Execution(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
            SupervisorCommand::Risk(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
            SupervisorCommand::Logs(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
            SupervisorCommand::Metrics(SupervisorNodeOpt {
                registry: registry_opt.clone(),
                node_id: "sandbox-a".to_string(),
            }),
        ] {
            run_supervisor_command(SupervisorOpt { command }).unwrap();
        }

        assert_eq!(
            store.node_status("sandbox-a").unwrap().lifecycle_state,
            LifecycleStatus::Running
        );
        assert_eq!(
            store.node_metrics("sandbox-a").unwrap().lifecycle_state,
            LifecycleStatus::Running
        );

        run_supervisor_command(SupervisorOpt {
            command: SupervisorCommand::Stop(SupervisorStopOpt {
                registry: registry_opt,
                node_id: "sandbox-a".to_string(),
                stop_timeout_ms: 3_000,
            }),
        })
        .unwrap();

        wait_for_metrics_state(&store, "sandbox-a", LifecycleStatus::Stopped);
        assert_eq!(
            store.node_status("sandbox-a").unwrap().lifecycle_state,
            LifecycleStatus::Stopped
        );
        assert_eq!(
            store.node_metrics("sandbox-a").unwrap().lifecycle_state,
            LifecycleStatus::Stopped
        );
        assert!(artifact_root.join("logs").join("stdout.log").exists());
        assert!(artifact_root.join("logs").join("stderr.log").exists());
        assert!(artifact_root.join("logs").join("events.log").exists());
    }

    fn wait_for_metrics_state(
        store: &SupervisorRegistryStore,
        node_id: &str,
        expected: LifecycleStatus,
    ) {
        let started = SystemTime::now();
        loop {
            if let Ok(metrics) = store.node_metrics(node_id)
                && metrics.lifecycle_state == expected
            {
                return;
            }
            if started
                .elapsed()
                .is_ok_and(|elapsed| elapsed >= Duration::from_secs(3))
            {
                panic!("timed out waiting for metrics artifact state {expected:?}");
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
}
