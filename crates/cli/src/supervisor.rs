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
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, ensure};
use nautilus_live::status::{
    ConnectionStatus, LifecycleStatus, NodeStatus, ProcessMode, SnapshotValue,
};
use serde::{Deserialize, Serialize};

pub const SUPERVISOR_REGISTRY_SCHEMA_VERSION: &str = "ntpro.supervisor_registry.v1";

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
pub struct SupervisorRegistryStore {
    registry_path: PathBuf,
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
        let raw = serde_json::to_string_pretty(registry)?;
        fs::write(&self.registry_path, format!("{raw}\n")).with_context(|| {
            format!(
                "failed to write supervisor registry '{}'",
                self.registry_path.display()
            )
        })?;
        Ok(())
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
        let mut registry = self.load()?;
        let record = registry
            .nodes
            .get_mut(node_id)
            .with_context(|| format!("node '{node_id}' is not registered"))?;

        if !record.status_path.exists() {
            record.status_artifact = RegistryArtifactState::Missing;
            record.last_known_status.generated_at = SnapshotValue::stale();
        } else {
            let raw = fs::read_to_string(&record.status_path).with_context(|| {
                format!(
                    "failed to read status artifact '{}'",
                    record.status_path.display()
                )
            })?;
            match serde_json::from_str::<NodeStatus>(&raw) {
                Ok(status) => {
                    record.status_artifact = RegistryArtifactState::Available;
                    record.last_known_status = status;
                }
                Err(error) => {
                    record.status_artifact = RegistryArtifactState::Invalid;
                    record.last_known_status.last_error =
                        Some(format!("invalid status artifact: {error}"));
                    record.last_known_status.generated_at = SnapshotValue::stale();
                }
            }
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
        Ok(self.load()?.nodes.into_values().collect())
    }

    /// # Errors
    ///
    /// Returns an error if the registry cannot be loaded or saved.
    pub fn remove_node(&self, node_id: &str) -> anyhow::Result<Option<SupervisorNodeRecord>> {
        let mut registry = self.load()?;
        let removed = registry.nodes.remove(node_id);
        if removed.is_some() {
            registry.updated_at = SnapshotValue::available(now_millis());
            self.save(&registry)?;
        }
        Ok(removed)
    }

    fn default_node_artifact_root(&self, node_id: &str) -> PathBuf {
        self.registry_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("nodes")
            .join(node_id)
    }
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
            let raw = serde_json::to_string_pretty(&artifact)?;
            fs::write(&record.pid_path, format!("{raw}\n")).with_context(|| {
                format!(
                    "failed to write pid artifact '{}'",
                    record.pid_path.display()
                )
            })?;
        }
        None => {
            if record.pid_path.exists() {
                fs::remove_file(&record.pid_path).with_context(|| {
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
    fn update_process_writes_pid_artifact_and_missing_artifact_marks_stale() {
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

        fs::remove_file(&registered.pid_path).unwrap();
        let refreshed = store.refresh_process_state("sandbox-a").unwrap();
        assert_eq!(refreshed.process.state, SupervisorProcessState::Stale);
    }
}
