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

use crate::{
    artifacts::{atomic_write_json, atomic_write_text, remove_file_if_exists},
    opt::{
        SupervisorCommand, SupervisorListOpt, SupervisorNodeOpt, SupervisorOpt,
        SupervisorRegisterOpt, SupervisorStartOpt, SupervisorStopOpt,
    },
    process::{
        SignalDelivery, process_is_alive, send_kill, send_termination, wait_for_process_exit,
    },
};

pub const SUPERVISOR_REGISTRY_SCHEMA_VERSION: &str = "ntpro.supervisor_registry.v1";
pub const NODE_METRICS_SCHEMA_VERSION: &str = "ntpro.node_metrics.v1";
const REGISTRY_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const REGISTRY_LOCK_RETRY: Duration = Duration::from_millis(25);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(100);
const PROCESS_SIGNAL_GRACE: Duration = Duration::from_secs(1);

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
    pub external_venue_connection: bool,
    pub real_orders_submitted: bool,
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
        }
    }
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
        SupervisorCommand::Status(node) => run_supervisor_status(node),
        SupervisorCommand::Connections(node) => run_supervisor_connections(node),
        SupervisorCommand::Execution(node) => run_supervisor_execution(node),
        SupervisorCommand::Risk(node) => run_supervisor_risk(node),
        SupervisorCommand::Logs(node) => run_supervisor_logs(node),
        SupervisorCommand::Metrics(node) => run_supervisor_metrics(node),
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

fn run_supervisor_status(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let status = store.node_status(&opt.node_id)?;
    println!(
        "supervisor.status status=ok registry_node_id={} runtime_node_id={} lifecycle_state={} previous_lifecycle_state={} process_mode={} generated_at={} external_venue_connection={} real_orders_submitted={} last_error={}",
        opt.node_id,
        status.node_id,
        json_label(&status.lifecycle_state),
        json_label(&status.previous_lifecycle_state),
        json_label(&status.process_mode),
        snapshot_display(&status.generated_at),
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
    println!(
        "supervisor.risk status=ok registry_node_id={} runtime_node_id={} trading_state={} health={} command_count={} event_count={} rejections_total={} last_rejection={} last_error={}",
        opt.node_id,
        status.node_id,
        json_label(&status.risk.trading_state),
        json_label(&status.risk.health),
        snapshot_display(&status.risk.command_count),
        snapshot_display(&status.risk.event_count),
        snapshot_display(&status.risk.rejections_total),
        status.risk.last_rejection.as_deref().unwrap_or("none"),
        status.risk.last_error.as_deref().unwrap_or("none"),
    );
    Ok(())
}

fn run_supervisor_logs(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let record = load_node_record(&store, &opt.node_id)?;
    println!(
        "supervisor.logs status=ok node_id={} stdout_log={} stderr_log={} events_log={}",
        record.node_id,
        record.stdout_log_path.display(),
        record.stderr_log_path.display(),
        record.events_log_path.display(),
    );
    Ok(())
}

fn run_supervisor_metrics(opt: SupervisorNodeOpt) -> anyhow::Result<()> {
    let store = SupervisorRegistryStore::new(opt.registry.registry);
    let metrics = store.node_metrics(&opt.node_id)?;
    println!(
        "supervisor.metrics status=ok registry_node_id={} runtime_node_id={} lifecycle_state={} starts_total={} stops_total={} state_transitions_total={} uptime_ms={} external_venue_connection={} real_orders_submitted={} last_error={}",
        opt.node_id,
        metrics.node_id,
        json_label(&metrics.lifecycle_state),
        metrics.starts_total,
        metrics.stops_total,
        metrics.state_transitions_total,
        snapshot_display(&metrics.uptime_ms),
        metrics.external_venue_connection,
        metrics.real_orders_submitted,
        metrics.last_error_summary.as_deref().unwrap_or("none"),
    );
    Ok(())
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

impl Drop for RegistryFileLock {
    fn drop(&mut self) {
        let _ = remove_file_if_exists(&self.path);
    }
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
                    writeln!(file, "pid={} acquired_at={}", process::id(), now_millis())
                        .with_context(|| {
                            format!("failed to write registry lock '{}'", lock_path.display())
                        })?;
                    file.sync_all().with_context(|| {
                        format!("failed to sync registry lock '{}'", lock_path.display())
                    })?;
                    return Ok(lock);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if started
                        .elapsed()
                        .is_ok_and(|elapsed| elapsed >= REGISTRY_LOCK_TIMEOUT)
                    {
                        anyhow::bail!(
                            "timed out waiting for supervisor registry lock '{}'",
                            lock_path.display()
                        );
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
            record.process.state = SupervisorProcessState::Stale;
            record.process.updated_at = SnapshotValue::available(now_millis());
            record.updated_at = SnapshotValue::available(now_millis());
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
            anyhow::bail!(
                "node '{node_id}' published an invalid status artifact: {}",
                record
                    .last_known_status
                    .last_error
                    .as_deref()
                    .unwrap_or("unknown status artifact error")
            );
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
            return Ok(record);
        }
        if started.elapsed().is_ok_and(|elapsed| elapsed >= timeout) {
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
    Ok(artifact.node_id != record.node_id || artifact.pid != expected_pid)
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
    use std::sync::{Arc, Barrier};

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "ntpro-v02-005-supervisor-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_config(root: &Path, name: &str) -> PathBuf {
        let path = root.join(format!("{name}.toml"));
        fs::write(&path, "[run]\nid = \"live-init-smoke\"\n").unwrap();
        path
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
cat > "$output/status.json" <<EOF
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
cat > "$output/metrics.json" <<EOF
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
cat > "$output/status.json" <<EOF
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
cat > "$output/metrics.json" <<EOF
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

    #[cfg(unix)]
    #[test]
    fn start_status_stop_process_roundtrip() {
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
    }

    #[cfg(unix)]
    #[test]
    fn start_rejects_child_that_exits_before_running_status() {
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
