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

//! Stable local node status DTOs for the V02 runtime foundation.

use serde::{Deserialize, Serialize};

use crate::node::NodeState;

pub const NODE_STATUS_SCHEMA_VERSION: &str = "ntpro.node_status.v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotAvailability {
    Available,
    NotConfigured,
    NotSupported,
    Stale,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotValue<T> {
    pub availability: SnapshotAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,
}

impl<T> SnapshotValue<T> {
    #[must_use]
    pub const fn available(value: T) -> Self {
        Self {
            availability: SnapshotAvailability::Available,
            value: Some(value),
        }
    }

    #[must_use]
    pub const fn not_configured() -> Self {
        Self {
            availability: SnapshotAvailability::NotConfigured,
            value: None,
        }
    }

    #[must_use]
    pub const fn not_supported() -> Self {
        Self {
            availability: SnapshotAvailability::NotSupported,
            value: None,
        }
    }

    #[must_use]
    pub const fn stale() -> Self {
        Self {
            availability: SnapshotAvailability::Stale,
            value: None,
        }
    }

    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            availability: SnapshotAvailability::Unknown,
            value: None,
        }
    }
}

impl<T> Default for SnapshotValue<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    Stopped,
    Starting,
    Running,
    Pausing,
    Paused,
    Resuming,
    Stopping,
    Error,
    #[default]
    Unknown,
}

impl From<NodeState> for LifecycleStatus {
    fn from(state: NodeState) -> Self {
        match state {
            NodeState::Idle | NodeState::Stopped => Self::Stopped,
            NodeState::Starting => Self::Starting,
            NodeState::Running => Self::Running,
            NodeState::ShuttingDown => Self::Stopping,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessMode {
    SpawnedProcess,
    TestHarness,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Disconnecting,
    NotConfigured,
    NotSupported,
    Stale,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Error,
    Stale,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskTradingState {
    Active,
    Reducing,
    Halted,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionStatus {
    pub gateway_id: SnapshotValue<String>,
    pub connection: ConnectionStatus,
    pub started: SnapshotValue<bool>,
    pub account_ref: SnapshotValue<String>,
    pub orders_open: SnapshotValue<u64>,
    pub orders_inflight: SnapshotValue<u64>,
    pub orders_closed: SnapshotValue<u64>,
    pub last_report_at: SnapshotValue<String>,
    pub last_reconciliation_at: SnapshotValue<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskStatus {
    pub trading_state: RiskTradingState,
    pub health: HealthStatus,
    pub command_count: SnapshotValue<u64>,
    pub event_count: SnapshotValue<u64>,
    pub rejections_total: SnapshotValue<u64>,
    pub last_rejection: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeStatus {
    pub schema_version: String,
    pub node_id: String,
    pub process_mode: ProcessMode,
    pub config_path: SnapshotValue<String>,
    pub artifact_root: SnapshotValue<String>,
    pub lifecycle_state: LifecycleStatus,
    pub previous_lifecycle_state: LifecycleStatus,
    pub data_connection: ConnectionStatus,
    pub execution_connection: ConnectionStatus,
    pub execution: ExecutionStatus,
    pub risk: RiskStatus,
    pub generated_at: SnapshotValue<String>,
    pub started_at: SnapshotValue<String>,
    pub stopped_at: SnapshotValue<String>,
    pub last_transition_at: SnapshotValue<String>,
    pub last_error: Option<String>,
    pub external_venue_connection: bool,
    pub real_orders_submitted: bool,
}

impl NodeStatus {
    #[must_use]
    pub fn unknown(node_id: impl Into<String>) -> Self {
        Self {
            schema_version: NODE_STATUS_SCHEMA_VERSION.to_string(),
            node_id: node_id.into(),
            process_mode: ProcessMode::Unknown,
            config_path: SnapshotValue::unknown(),
            artifact_root: SnapshotValue::unknown(),
            lifecycle_state: LifecycleStatus::Unknown,
            previous_lifecycle_state: LifecycleStatus::Unknown,
            data_connection: ConnectionStatus::Unknown,
            execution_connection: ConnectionStatus::Unknown,
            execution: ExecutionStatus::default(),
            risk: RiskStatus::default(),
            generated_at: SnapshotValue::unknown(),
            started_at: SnapshotValue::unknown(),
            stopped_at: SnapshotValue::unknown(),
            last_transition_at: SnapshotValue::unknown(),
            last_error: None,
            external_venue_connection: false,
            real_orders_submitted: false,
        }
    }

    #[must_use]
    pub fn from_node_state(node_id: impl Into<String>, state: NodeState) -> Self {
        let mut status = Self::unknown(node_id);
        status.lifecycle_state = LifecycleStatus::from(state);
        status
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn maps_stopped_node_states_to_stopped_contract_state() {
        assert_eq!(
            LifecycleStatus::from(NodeState::Idle),
            LifecycleStatus::Stopped
        );
        assert_eq!(
            LifecycleStatus::from(NodeState::Stopped),
            LifecycleStatus::Stopped
        );
    }

    #[test]
    fn maps_running_node_state_to_running_contract_state() {
        let status = NodeStatus::from_node_state("sandbox-a", NodeState::Running);

        assert_eq!(status.node_id, "sandbox-a");
        assert_eq!(status.lifecycle_state, LifecycleStatus::Running);
        assert!(!status.external_venue_connection);
        assert!(!status.real_orders_submitted);
    }

    #[test]
    fn unknown_snapshot_keeps_missing_fields_explicit() {
        let status = NodeStatus::unknown("missing-node");

        assert_eq!(status.lifecycle_state, LifecycleStatus::Unknown);
        assert_eq!(
            status.config_path.availability,
            SnapshotAvailability::Unknown
        );
        assert!(status.config_path.value.is_none());
        assert_eq!(status.data_connection, ConnectionStatus::Unknown);
        assert_eq!(status.execution.connection, ConnectionStatus::Unknown);
        assert_eq!(status.risk.trading_state, RiskTradingState::Unknown);
        assert!(status.last_error.is_none());
    }

    #[test]
    fn serializes_missing_values_without_secret_or_raw_payload_fields() {
        let status = NodeStatus::from_node_state("sandbox-a", NodeState::Stopped);
        let value = serde_json::to_value(status).unwrap();

        assert_eq!(value["schema_version"], NODE_STATUS_SCHEMA_VERSION);
        assert_eq!(value["node_id"], "sandbox-a");
        assert_eq!(value["lifecycle_state"], "stopped");
        assert_eq!(value["config_path"], json!({"availability": "unknown"}));
        assert!(value.get("credentials").is_none());
        assert!(value.get("raw_payload").is_none());
        assert!(value.get("orders").is_none());
    }
}
