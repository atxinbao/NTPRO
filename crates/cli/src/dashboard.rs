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

//! Dashboard read-model DTOs for the local v0.3 MVP.

use std::collections::BTreeMap;

use nautilus_live::status::{
    HealthStatus, LifecycleStatus, NodeStatus, ProcessMode, SnapshotValue,
};
use serde::{Deserialize, Serialize};

pub const DASHBOARD_SNAPSHOT_SCHEMA_VERSION: &str = "ntpro.dashboard_snapshot.v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardAvailability {
    Available,
    NotConfigured,
    NotSupported,
    Stale,
    Redacted,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardValue<T> {
    pub availability: DashboardAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,
}

impl<T> DashboardValue<T> {
    #[must_use]
    pub const fn available(value: T) -> Self {
        Self {
            availability: DashboardAvailability::Available,
            value: Some(value),
        }
    }

    #[must_use]
    pub const fn not_configured() -> Self {
        Self {
            availability: DashboardAvailability::NotConfigured,
            value: None,
        }
    }

    #[must_use]
    pub const fn not_supported() -> Self {
        Self {
            availability: DashboardAvailability::NotSupported,
            value: None,
        }
    }

    #[must_use]
    pub const fn stale() -> Self {
        Self {
            availability: DashboardAvailability::Stale,
            value: None,
        }
    }

    #[must_use]
    pub const fn redacted() -> Self {
        Self {
            availability: DashboardAvailability::Redacted,
            value: None,
        }
    }

    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            availability: DashboardAvailability::Unknown,
            value: None,
        }
    }
}

impl<T> Default for DashboardValue<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub schema_version: String,
    pub generated_at: DashboardValue<String>,
    pub overview: DashboardOverview,
    pub nodes: Vec<DashboardNodeSummary>,
    pub data_sources: Vec<DashboardSectionStatus>,
    pub execution_gateways: Vec<DashboardSectionStatus>,
    pub risk: DashboardSectionStatus,
    pub runtime_modules: Vec<DashboardSectionStatus>,
    pub logs: Vec<DashboardArtifactStatus>,
    pub metrics: Vec<DashboardArtifactStatus>,
    pub alerts: AlertSummary,
    pub controls: Vec<ControlStatus>,
    pub gaps: Vec<DashboardGap>,
}

impl DashboardSnapshot {
    #[must_use]
    pub fn empty(generated_at: impl Into<String>) -> Self {
        Self {
            schema_version: DASHBOARD_SNAPSHOT_SCHEMA_VERSION.to_string(),
            generated_at: DashboardValue::available(generated_at.into()),
            overview: DashboardOverview::default(),
            nodes: Vec::new(),
            data_sources: Vec::new(),
            execution_gateways: Vec::new(),
            risk: DashboardSectionStatus::unknown("risk", "Risk"),
            runtime_modules: Vec::new(),
            logs: Vec::new(),
            metrics: Vec::new(),
            alerts: AlertSummary::default(),
            controls: Vec::new(),
            gaps: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_nodes(generated_at: impl Into<String>, nodes: Vec<DashboardNodeSummary>) -> Self {
        let overview = DashboardOverview::from_nodes(&nodes);
        Self {
            overview,
            nodes,
            ..Self::empty(generated_at)
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub node_count: u64,
    pub running_nodes: u64,
    pub stopped_nodes: u64,
    pub error_nodes: u64,
    pub unknown_nodes: u64,
    pub health: HealthStatus,
    pub sandbox_only: bool,
    pub external_venue_connection: bool,
    pub real_orders_submitted: bool,
    pub latest_transition_at: DashboardValue<String>,
    pub latest_error: Option<String>,
}

impl DashboardOverview {
    #[must_use]
    pub fn from_nodes(nodes: &[DashboardNodeSummary]) -> Self {
        let mut overview = Self {
            node_count: nodes.len() as u64,
            sandbox_only: true,
            health: HealthStatus::Unknown,
            ..Self::default()
        };

        for node in nodes {
            match node.lifecycle_state {
                LifecycleStatus::Running => overview.running_nodes += 1,
                LifecycleStatus::Stopped => overview.stopped_nodes += 1,
                LifecycleStatus::Error => overview.error_nodes += 1,
                LifecycleStatus::Unknown => overview.unknown_nodes += 1,
                LifecycleStatus::Starting
                | LifecycleStatus::Pausing
                | LifecycleStatus::Paused
                | LifecycleStatus::Resuming
                | LifecycleStatus::Stopping => {}
            }
            overview.external_venue_connection |= node.external_venue_connection;
            overview.real_orders_submitted |= node.real_orders_submitted;
            if overview.latest_error.is_none() {
                overview.latest_error.clone_from(&node.last_error);
            }
        }

        overview.health = derive_overview_health(&overview);
        overview
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardNodeSummary {
    pub node_id: String,
    pub lifecycle_state: LifecycleStatus,
    pub process_mode: ProcessMode,
    pub health: HealthStatus,
    pub config_path: SnapshotValue<String>,
    pub artifact_root: SnapshotValue<String>,
    pub generated_at: SnapshotValue<String>,
    pub started_at: SnapshotValue<String>,
    pub stopped_at: SnapshotValue<String>,
    pub last_transition_at: SnapshotValue<String>,
    pub last_error: Option<String>,
    pub external_venue_connection: bool,
    pub real_orders_submitted: bool,
    pub gaps: Vec<DashboardGap>,
}

impl DashboardNodeSummary {
    #[must_use]
    pub fn from_status(status: &NodeStatus) -> Self {
        Self {
            node_id: status.node_id.clone(),
            lifecycle_state: status.lifecycle_state,
            process_mode: status.process_mode,
            health: derive_node_health(status),
            config_path: status.config_path.clone(),
            artifact_root: status.artifact_root.clone(),
            generated_at: status.generated_at.clone(),
            started_at: status.started_at.clone(),
            stopped_at: status.stopped_at.clone(),
            last_transition_at: status.last_transition_at.clone(),
            last_error: status.last_error.clone(),
            external_venue_connection: status.external_venue_connection,
            real_orders_submitted: status.real_orders_submitted,
            gaps: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardSectionStatus {
    pub section_id: String,
    pub label: String,
    pub availability: DashboardAvailability,
    pub health: HealthStatus,
    pub status: DashboardValue<String>,
    pub last_error: Option<String>,
}

impl DashboardSectionStatus {
    #[must_use]
    pub fn unknown(section_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            section_id: section_id.into(),
            label: label.into(),
            availability: DashboardAvailability::Unknown,
            health: HealthStatus::Unknown,
            status: DashboardValue::unknown(),
            last_error: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardArtifactStatus {
    pub artifact_id: String,
    pub label: String,
    pub path: DashboardValue<String>,
    pub availability: DashboardAvailability,
    pub last_seen_at: DashboardValue<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertSummary {
    pub active_count: u64,
    pub counts_by_severity: BTreeMap<String, u64>,
    pub active: Vec<DashboardAlert>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardAlert {
    pub alert_id: String,
    pub severity: String,
    pub source: String,
    pub message: String,
    pub first_seen_at: DashboardValue<String>,
    pub last_seen_at: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlStatus {
    pub action: String,
    pub availability: DashboardAvailability,
    pub enabled: bool,
    pub reason: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardGap {
    pub field_path: String,
    pub reason: DashboardAvailability,
    pub owner_task: DashboardValue<String>,
    pub notes: DashboardValue<String>,
}

impl DashboardGap {
    #[must_use]
    pub fn new(
        field_path: impl Into<String>,
        reason: DashboardAvailability,
        owner_task: impl Into<String>,
        notes: impl Into<String>,
    ) -> Self {
        Self {
            field_path: field_path.into(),
            reason,
            owner_task: DashboardValue::available(owner_task.into()),
            notes: DashboardValue::available(notes.into()),
        }
    }
}

fn derive_node_health(status: &NodeStatus) -> HealthStatus {
    if status.last_error.is_some() {
        return HealthStatus::Error;
    }

    match status.lifecycle_state {
        LifecycleStatus::Running | LifecycleStatus::Stopped => HealthStatus::Healthy,
        LifecycleStatus::Error => HealthStatus::Error,
        LifecycleStatus::Unknown => HealthStatus::Unknown,
        LifecycleStatus::Starting
        | LifecycleStatus::Pausing
        | LifecycleStatus::Paused
        | LifecycleStatus::Resuming
        | LifecycleStatus::Stopping => HealthStatus::Degraded,
    }
}

fn derive_overview_health(overview: &DashboardOverview) -> HealthStatus {
    if overview.error_nodes > 0 || overview.latest_error.is_some() {
        HealthStatus::Error
    } else if overview.node_count == 0 || overview.unknown_nodes > 0 {
        HealthStatus::Unknown
    } else if overview.running_nodes > 0 {
        HealthStatus::Healthy
    } else {
        HealthStatus::Degraded
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    #[test]
    fn empty_snapshot_serializes_stable_top_level_sections() {
        let snapshot = DashboardSnapshot::empty("2026-06-07T14:00:00Z");
        let value = serde_json::to_value(snapshot).unwrap();

        assert_eq!(value["schema_version"], DASHBOARD_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(
            value["generated_at"],
            json!({"availability": "available", "value": "2026-06-07T14:00:00Z"})
        );
        for key in [
            "overview",
            "nodes",
            "data_sources",
            "execution_gateways",
            "risk",
            "runtime_modules",
            "logs",
            "metrics",
            "alerts",
            "controls",
            "gaps",
        ] {
            assert!(value.get(key).is_some(), "missing dashboard key {key}");
        }
        assert_eq!(value["overview"]["node_count"], 0);
        assert_eq!(value["overview"]["health"], "unknown");
        assert_eq!(value["risk"]["availability"], "unknown");
    }

    #[test]
    fn one_node_snapshot_counts_running_node() {
        let status = NodeStatus {
            lifecycle_state: LifecycleStatus::Running,
            generated_at: SnapshotValue::available("2026-06-07T14:01:00Z".to_string()),
            ..NodeStatus::unknown("sandbox-a")
        };
        let node = DashboardNodeSummary::from_status(&status);
        let snapshot = DashboardSnapshot::from_nodes("2026-06-07T14:01:01Z", vec![node]);
        let value = serde_json::to_value(snapshot).unwrap();

        assert_eq!(value["overview"]["node_count"], 1);
        assert_eq!(value["overview"]["running_nodes"], 1);
        assert_eq!(value["overview"]["health"], "healthy");
        assert_eq!(value["nodes"][0]["node_id"], "sandbox-a");
        assert_eq!(value["nodes"][0]["lifecycle_state"], "running");
        assert_eq!(value["nodes"][0]["health"], "healthy");
    }

    #[test]
    fn two_node_snapshot_counts_running_and_stopped_nodes() {
        let running = DashboardNodeSummary::from_status(&NodeStatus {
            lifecycle_state: LifecycleStatus::Running,
            ..NodeStatus::unknown("sandbox-a")
        });
        let stopped = DashboardNodeSummary::from_status(&NodeStatus {
            lifecycle_state: LifecycleStatus::Stopped,
            ..NodeStatus::unknown("sandbox-b")
        });
        let snapshot =
            DashboardSnapshot::from_nodes("2026-06-07T14:02:00Z", vec![running, stopped]);
        let value = serde_json::to_value(snapshot).unwrap();

        assert_eq!(value["overview"]["node_count"], 2);
        assert_eq!(value["overview"]["running_nodes"], 1);
        assert_eq!(value["overview"]["stopped_nodes"], 1);
        assert_eq!(value["nodes"][1]["node_id"], "sandbox-b");
    }

    #[test]
    fn explicit_unavailable_states_survive_json_shape() {
        let mut snapshot = DashboardSnapshot::empty("2026-06-07T14:03:00Z");
        snapshot.gaps = vec![
            DashboardGap::new(
                "data_sources[0].last_event_at",
                DashboardAvailability::Unknown,
                "V03-004",
                "aggregator not implemented yet",
            ),
            DashboardGap::new(
                "execution_gateways",
                DashboardAvailability::NotConfigured,
                "V03-003",
                "no execution gateway configured",
            ),
            DashboardGap::new(
                "runtime_modules.cache",
                DashboardAvailability::NotSupported,
                "V03-008",
                "module detail is not supported yet",
            ),
            DashboardGap::new(
                "metrics.generated_at",
                DashboardAvailability::Stale,
                "V03-004",
                "metrics artifact is older than threshold",
            ),
            DashboardGap::new(
                "execution_gateways[0].account_ref",
                DashboardAvailability::Redacted,
                "V03-003",
                "account reference is intentionally hidden",
            ),
        ];
        snapshot.controls.push(ControlStatus {
            action: "pause_trading".to_string(),
            availability: DashboardAvailability::NotSupported,
            enabled: false,
            reason: DashboardValue::not_supported(),
        });

        let value = serde_json::to_value(snapshot).unwrap();
        let reasons: Vec<_> = value["gaps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|gap| gap["reason"].as_str().unwrap())
            .collect();

        assert_eq!(
            reasons,
            [
                "unknown",
                "not_configured",
                "not_supported",
                "stale",
                "redacted"
            ]
        );
        assert_eq!(value["controls"][0]["availability"], "not_supported");
        assert_eq!(
            value["controls"][0]["reason"],
            json!({"availability": "not_supported"})
        );
    }

    #[test]
    fn snapshot_shape_does_not_expose_forbidden_raw_or_secret_fields() {
        let snapshot = DashboardSnapshot::from_nodes(
            "2026-06-07T14:04:00Z",
            vec![DashboardNodeSummary::from_status(&NodeStatus::unknown(
                "sandbox-a",
            ))],
        );
        let value = serde_json::to_value(snapshot).unwrap();

        assert_forbidden_keys_absent(&value);
    }

    fn assert_forbidden_keys_absent(value: &Value) {
        match value {
            Value::Object(map) => {
                for key in map.keys() {
                    assert!(
                        !matches!(
                            key.as_str(),
                            "secret"
                                | "secrets"
                                | "credential"
                                | "credentials"
                                | "api_key"
                                | "token"
                                | "raw_order"
                                | "raw_orders"
                                | "raw_fill"
                                | "raw_fills"
                                | "raw_payload"
                                | "raw_venue_payload"
                                | "account_object"
                        ),
                        "forbidden dashboard key exposed: {key}"
                    );
                }
                for child in map.values() {
                    assert_forbidden_keys_absent(child);
                }
            }
            Value::Array(items) => {
                for child in items {
                    assert_forbidden_keys_absent(child);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }
}
