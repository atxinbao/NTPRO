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
    ConnectionStatus, HealthStatus, LifecycleStatus, NodeStatus, ProcessMode, RiskTradingState,
    SnapshotValue,
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
    pub data_sources: Vec<DataSourceStatus>,
    pub execution_gateways: Vec<ExecutionGatewayStatus>,
    pub risk: RiskStatus,
    pub runtime_modules: Vec<RuntimeModuleStatus>,
    pub logs: Vec<LogStatus>,
    pub metrics: Vec<MetricStatus>,
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
            risk: RiskStatus::unknown(),
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
pub struct DataSourceStatus {
    pub source_id: String,
    pub source_kind: DashboardValue<String>,
    pub provider: DashboardValue<String>,
    pub connection: ConnectionStatus,
    pub freshness: DashboardValue<String>,
    pub lag_ms: DashboardValue<u64>,
    pub health: HealthStatus,
    pub last_error: DashboardValue<String>,
}

impl DataSourceStatus {
    #[must_use]
    pub fn unknown(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            source_kind: DashboardValue::unknown(),
            provider: DashboardValue::unknown(),
            connection: ConnectionStatus::Unknown,
            freshness: DashboardValue::unknown(),
            lag_ms: DashboardValue::unknown(),
            health: HealthStatus::Unknown,
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderCountSummary {
    pub open: DashboardValue<u64>,
    pub inflight: DashboardValue<u64>,
    pub closed: DashboardValue<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionGatewayStatus {
    pub gateway_id: String,
    pub venue: DashboardValue<String>,
    pub connection: ConnectionStatus,
    pub started: DashboardValue<bool>,
    pub account_ref: DashboardValue<String>,
    pub order_counts: OrderCountSummary,
    pub last_report_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl ExecutionGatewayStatus {
    #[must_use]
    pub fn unknown(gateway_id: impl Into<String>) -> Self {
        Self {
            gateway_id: gateway_id.into(),
            venue: DashboardValue::unknown(),
            connection: ConnectionStatus::Unknown,
            started: DashboardValue::unknown(),
            account_ref: DashboardValue::redacted(),
            order_counts: OrderCountSummary::default(),
            last_report_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectionSummary {
    pub reason: DashboardValue<String>,
    pub last_rejected_at: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskStatus {
    pub availability: DashboardAvailability,
    pub trading_state: RiskTradingState,
    pub health: HealthStatus,
    pub command_count: DashboardValue<u64>,
    pub event_count: DashboardValue<u64>,
    pub rejections_total: DashboardValue<u64>,
    pub last_rejection: DashboardValue<RejectionSummary>,
    pub last_error: DashboardValue<String>,
}

impl RiskStatus {
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            availability: DashboardAvailability::Unknown,
            trading_state: RiskTradingState::Unknown,
            health: HealthStatus::Unknown,
            command_count: DashboardValue::unknown(),
            event_count: DashboardValue::unknown(),
            rejections_total: DashboardValue::unknown(),
            last_rejection: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeModuleStatus {
    pub module_name: String,
    pub status: DashboardValue<String>,
    pub health: HealthStatus,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
    pub evidence_source: DashboardValue<String>,
}

impl RuntimeModuleStatus {
    #[must_use]
    pub fn unknown(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
            status: DashboardValue::unknown(),
            health: HealthStatus::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
            evidence_source: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogStatus {
    pub log_id: String,
    pub node_id: DashboardValue<String>,
    pub path: DashboardValue<String>,
    pub availability: DashboardAvailability,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl LogStatus {
    #[must_use]
    pub fn unknown(log_id: impl Into<String>) -> Self {
        Self {
            log_id: log_id.into(),
            node_id: DashboardValue::unknown(),
            path: DashboardValue::unknown(),
            availability: DashboardAvailability::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricStatus {
    pub metric_id: String,
    pub node_id: DashboardValue<String>,
    pub value: DashboardValue<String>,
    pub availability: DashboardAvailability,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl MetricStatus {
    #[must_use]
    pub fn unknown(metric_id: impl Into<String>) -> Self {
        Self {
            metric_id: metric_id.into(),
            node_id: DashboardValue::unknown(),
            value: DashboardValue::unknown(),
            availability: DashboardAvailability::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlActionStatus {
    Accepted,
    Rejected,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlActionResponse {
    pub action_id: String,
    pub action: String,
    pub status: ControlActionStatus,
    pub previous_state: LifecycleStatus,
    pub current_state: LifecycleStatus,
    pub started_at: DashboardValue<String>,
    pub finished_at: DashboardValue<String>,
    pub error_code: DashboardValue<String>,
    pub message: DashboardValue<String>,
    pub observability_ref: DashboardValue<String>,
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
        snapshot
            .data_sources
            .push(DataSourceStatus::unknown("sandbox-data"));
        snapshot
            .execution_gateways
            .push(ExecutionGatewayStatus::unknown("sandbox-exec"));
        snapshot
            .runtime_modules
            .push(RuntimeModuleStatus::unknown("MessageBus"));
        snapshot.logs.push(LogStatus::unknown("events"));
        snapshot.metrics.push(MetricStatus::unknown("node-metrics"));
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
        assert_eq!(value["data_sources"][0]["connection"], "unknown");
        assert_eq!(
            value["execution_gateways"][0]["account_ref"],
            json!({"availability": "redacted"})
        );
        assert_eq!(value["runtime_modules"][0]["module_name"], "MessageBus");
        assert_eq!(value["logs"][0]["availability"], "unknown");
        assert_eq!(value["metrics"][0]["availability"], "unknown");
        assert_eq!(value["risk"]["availability"], "unknown");
        assert_eq!(value["risk"]["trading_state"], "unknown");
    }

    #[test]
    fn detail_dtos_serialize_without_raw_or_secret_fields() {
        let mut snapshot = DashboardSnapshot::empty("2026-06-07T14:05:00Z");
        snapshot.data_sources.push(DataSourceStatus {
            source_id: "sandbox-data".to_string(),
            source_kind: DashboardValue::available("sandbox".to_string()),
            provider: DashboardValue::available("sandbox".to_string()),
            connection: ConnectionStatus::NotConfigured,
            freshness: DashboardValue::not_configured(),
            lag_ms: DashboardValue::not_configured(),
            health: HealthStatus::Unknown,
            last_error: DashboardValue::unknown(),
        });
        snapshot.execution_gateways.push(ExecutionGatewayStatus {
            gateway_id: "sandbox-exec".to_string(),
            venue: DashboardValue::available("SIM".to_string()),
            connection: ConnectionStatus::NotConfigured,
            started: DashboardValue::not_configured(),
            account_ref: DashboardValue::redacted(),
            order_counts: OrderCountSummary {
                open: DashboardValue::available(0),
                inflight: DashboardValue::available(0),
                closed: DashboardValue::available(0),
            },
            last_report_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        });
        snapshot.risk = RiskStatus {
            availability: DashboardAvailability::Available,
            trading_state: RiskTradingState::Active,
            health: HealthStatus::Healthy,
            command_count: DashboardValue::available(0),
            event_count: DashboardValue::available(0),
            rejections_total: DashboardValue::available(0),
            last_rejection: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        };
        snapshot
            .runtime_modules
            .push(RuntimeModuleStatus::unknown("RiskEngine"));
        snapshot.controls.push(ControlStatus {
            action: "start".to_string(),
            availability: DashboardAvailability::Available,
            enabled: true,
            reason: DashboardValue::available("node is stopped".to_string()),
        });

        let response = ControlActionResponse {
            action_id: "action-001".to_string(),
            action: "start".to_string(),
            status: ControlActionStatus::Accepted,
            previous_state: LifecycleStatus::Stopped,
            current_state: LifecycleStatus::Starting,
            started_at: DashboardValue::available("2026-06-07T14:05:01Z".to_string()),
            finished_at: DashboardValue::unknown(),
            error_code: DashboardValue::unknown(),
            message: DashboardValue::available("start accepted".to_string()),
            observability_ref: DashboardValue::unknown(),
        };

        let snapshot_value = serde_json::to_value(snapshot).unwrap();
        let response_value = serde_json::to_value(response).unwrap();

        assert_eq!(
            snapshot_value["execution_gateways"][0]["account_ref"],
            json!({"availability": "redacted"})
        );
        assert_eq!(snapshot_value["risk"]["trading_state"], "active");
        assert_eq!(snapshot_value["controls"][0]["enabled"], true);
        assert_eq!(response_value["status"], "accepted");
        assert_eq!(response_value["previous_state"], "stopped");
        assert_eq!(response_value["current_state"], "starting");
        assert_forbidden_keys_absent(&snapshot_value);
        assert_forbidden_keys_absent(&response_value);
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
