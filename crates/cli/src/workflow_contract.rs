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

//! Shared local workflow artifact contract for the CLI writer and Dashboard reader.

use serde::{Deserialize, Serialize};

pub(crate) const MANIFEST_SCHEMA_VERSION: &str = "ntpro.workflow_manifest.v1";
pub(crate) const SUMMARY_SCHEMA_VERSION: &str = "ntpro.workflow_summary.v1";
pub(crate) const BOUNDARY_SCHEMA_VERSION: &str = "ntpro.workflow_boundary.v1";
pub(crate) const EVENT_SCHEMA_VERSION: &str = "ntpro.workflow_event.v1";
pub(crate) const TESTNET_CONFIG_SCHEMA_VERSION: &str = "ntpro.v06_binance_testnet_config.v1";
pub(crate) const TESTNET_CREDENTIAL_POLICY_SCHEMA_VERSION: &str =
    "ntpro.v07_binance_testnet_credential_policy.v1";
pub(crate) const TESTNET_CONNECTIVITY_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v07_binance_testnet_connectivity_probe.v1";
pub(crate) const TESTNET_HTTP_CONNECTIVITY_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v07_binance_testnet_http_probe.v1";
pub(crate) const TESTNET_WEBSOCKET_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v07_binance_testnet_ws_probe.v1";
pub(crate) const TESTNET_AUTHENTICATED_READONLY_PROBE_SCHEMA_VERSION: &str =
    "ntpro.v08_binance_testnet_authenticated_readonly_probe.v1";
pub(crate) const TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH: &str = "testnet/connectivity_probe.json";
pub(crate) const TESTNET_HTTP_CONNECTIVITY_PROBE_ARTIFACT_PATH: &str =
    "testnet/http_connectivity_probe.json";
pub(crate) const TESTNET_WEBSOCKET_PROBE_ARTIFACT_PATH: &str = "testnet/ws_connectivity_probe.json";
pub(crate) const TESTNET_AUTHENTICATED_READONLY_PROBE_ARTIFACT_PATH: &str =
    "testnet/authenticated_readonly_probe.json";
pub(crate) const TESTNET_ORDER_LIFECYCLE_SCHEMA_VERSION: &str =
    "ntpro.v06_binance_testnet_order_lifecycle.v1";
pub(crate) const TESTNET_RECONCILIATION_SCHEMA_VERSION: &str =
    "ntpro.v06_binance_testnet_reconciliation.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestnetConfigArtifact {
    pub(crate) schema_version: String,
    pub(crate) source_path: String,
    pub(crate) run_id: String,
    pub(crate) config_declared_run_id: String,
    pub(crate) mode: String,
    pub(crate) venue: String,
    pub(crate) product: String,
    pub(crate) environment: String,
    pub(crate) http_base_url: String,
    pub(crate) ws_base_url: String,
    pub(crate) order_submission: String,
    pub(crate) reconciliation: String,
    pub(crate) real_orders_submitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestnetCredentialPolicy {
    pub(crate) schema_version: String,
    pub(crate) policy: String,
    #[serde(default)]
    pub(crate) credential_source: String,
    pub(crate) api_key_env: String,
    pub(crate) api_secret_env: String,
    pub(crate) values_in_file: bool,
    pub(crate) values_recorded: bool,
    #[serde(default)]
    pub(crate) api_key_value_recorded: bool,
    #[serde(default)]
    pub(crate) api_secret_value_recorded: bool,
    pub(crate) secrets_redacted: bool,
    #[serde(default)]
    pub(crate) required_for_network: bool,
    #[serde(default)]
    pub(crate) required_for_public_read_only_probe: bool,
    #[serde(default)]
    pub(crate) required_for_authenticated_read_only_probe: bool,
    #[serde(default)]
    pub(crate) legacy_required_for_network_present: bool,
    #[serde(default)]
    pub(crate) credential_config_migration_warning: String,
    #[serde(default)]
    pub(crate) public_read_only_probe_requires_credentials: bool,
    #[serde(default)]
    pub(crate) authenticated_read_only_probe_requires_credentials: bool,
    #[serde(default)]
    pub(crate) authenticated_read_only_probe_gate: String,
    #[serde(default)]
    pub(crate) authenticated_read_only_probe_status: String,
    #[serde(default)]
    pub(crate) authenticated_read_only_probe_fail_closed: bool,
    pub(crate) api_key_present: bool,
    pub(crate) api_secret_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestnetConnectivityProbe {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) requested_mode: String,
    #[serde(default)]
    pub(crate) public_read_only_probe_status: String,
    #[serde(default)]
    pub(crate) authenticated_read_only_probe_status: String,
    #[serde(default)]
    pub(crate) authenticated_read_only_probe_gate: String,
    #[serde(default)]
    pub(crate) authenticated_read_only_probe_requires_credentials: bool,
    pub(crate) http_base_url: String,
    pub(crate) ws_base_url: String,
    #[serde(default)]
    pub(crate) endpoint_class: String,
    #[serde(default)]
    pub(crate) latency_ms: Option<u64>,
    #[serde(default)]
    pub(crate) http_status: Option<u16>,
    #[serde(default)]
    pub(crate) response_shape: String,
    #[serde(default)]
    pub(crate) response_shape_validated: bool,
    #[serde(default)]
    pub(crate) error_code: String,
    pub(crate) network_permission_requested: bool,
    #[serde(default)]
    pub(crate) env_network_permission: bool,
    #[serde(default)]
    pub(crate) network_gate_status: String,
    #[serde(default)]
    pub(crate) network_gate_reasons: Vec<String>,
    pub(crate) network_attempted: bool,
    pub(crate) testnet_connection: bool,
    pub(crate) status: String,
    pub(crate) diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestnetHttpConnectivityProbe {
    pub(crate) schema_version: String,
    pub(crate) run_id: String,
    pub(crate) environment: String,
    pub(crate) product: String,
    pub(crate) endpoint_kind: String,
    pub(crate) endpoint_url_redacted: String,
    pub(crate) network_gate_status: String,
    pub(crate) network_gate_reasons: Vec<String>,
    pub(crate) network_permission_requested: bool,
    pub(crate) env_network_permission: bool,
    pub(crate) network_attempted: bool,
    pub(crate) testnet_connection: bool,
    pub(crate) order_submission: String,
    pub(crate) real_orders_submitted: bool,
    pub(crate) credential_policy: String,
    pub(crate) api_key_present: bool,
    pub(crate) api_secret_present: bool,
    pub(crate) request_method: String,
    pub(crate) request_target: String,
    pub(crate) response_status_code: Option<u16>,
    pub(crate) response_shape: String,
    pub(crate) response_shape_validated: bool,
    pub(crate) latency_ms: Option<u64>,
    pub(crate) error_code: String,
    pub(crate) status: String,
    pub(crate) diagnostic: String,
    pub(crate) generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestnetWebSocketConnectivityProbe {
    pub(crate) schema_version: String,
    pub(crate) run_id: String,
    pub(crate) mode: String,
    pub(crate) requested_mode: String,
    pub(crate) endpoint_kind: String,
    pub(crate) endpoint_class: String,
    pub(crate) ws_base_url: String,
    pub(crate) network_gate_status: String,
    pub(crate) network_gate_reasons: Vec<String>,
    pub(crate) network_permission_requested: bool,
    pub(crate) env_network_permission: bool,
    pub(crate) websocket_probe_gate: String,
    pub(crate) websocket_attempted: bool,
    pub(crate) network_attempted: bool,
    pub(crate) testnet_connection: bool,
    pub(crate) subscription_attempted: bool,
    pub(crate) message_count: u64,
    pub(crate) order_submission: String,
    pub(crate) real_orders_submitted: bool,
    pub(crate) values_recorded: bool,
    pub(crate) secrets_redacted: bool,
    pub(crate) status: String,
    pub(crate) error_code: String,
    pub(crate) diagnostic: String,
    pub(crate) generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestnetAuthenticatedReadOnlyProbe {
    pub(crate) schema_version: String,
    pub(crate) run_id: String,
    pub(crate) environment: String,
    pub(crate) product: String,
    pub(crate) endpoint_kind: String,
    pub(crate) endpoint_class: String,
    pub(crate) endpoint_url_redacted: String,
    pub(crate) network_gate_status: String,
    pub(crate) network_gate_reasons: Vec<String>,
    pub(crate) network_permission_requested: bool,
    pub(crate) env_network_permission: bool,
    pub(crate) network_attempted: bool,
    pub(crate) testnet_connection: bool,
    pub(crate) credential_policy: String,
    pub(crate) api_key_present: bool,
    pub(crate) api_secret_present: bool,
    pub(crate) request_method: String,
    pub(crate) request_target: String,
    pub(crate) query_shape: String,
    pub(crate) api_key_header_name: String,
    pub(crate) api_key_header_value_recorded: bool,
    pub(crate) signature_recorded: bool,
    pub(crate) signed_query_recorded: bool,
    pub(crate) signed_url_recorded: bool,
    pub(crate) raw_response_recorded: bool,
    pub(crate) balances_recorded: bool,
    pub(crate) uid_recorded: bool,
    pub(crate) account_mutation: bool,
    pub(crate) order_submission: String,
    pub(crate) real_orders_submitted: bool,
    pub(crate) production_venue_connection: bool,
    pub(crate) real_funds: bool,
    pub(crate) production_trading: bool,
    pub(crate) response_status_code: Option<u16>,
    pub(crate) response_shape: String,
    pub(crate) response_shape_validated: bool,
    pub(crate) latency_ms: Option<u64>,
    pub(crate) error_code: String,
    pub(crate) status: String,
    pub(crate) diagnostic: String,
    pub(crate) generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestnetOrderLifecycle {
    pub(crate) schema_version: String,
    pub(crate) lifecycle_id: String,
    pub(crate) mode: String,
    pub(crate) order_submission: String,
    pub(crate) submitted_count: u64,
    pub(crate) accepted_count: u64,
    pub(crate) filled_count: u64,
    pub(crate) canceled_count: u64,
    pub(crate) rejected_count: u64,
    pub(crate) real_orders_submitted: bool,
    pub(crate) external_venue_connection: bool,
    pub(crate) checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestnetReconciliation {
    pub(crate) schema_version: String,
    pub(crate) reconciliation_id: String,
    pub(crate) mode: String,
    pub(crate) matched_orders: u64,
    pub(crate) unmatched_orders: u64,
    pub(crate) external_account_state_loaded: bool,
    pub(crate) real_orders_submitted: bool,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorkflowBoundary {
    pub(crate) schema_version: String,
    pub(crate) sandbox_only: bool,
    pub(crate) fixture_replay: bool,
    pub(crate) mock_execution: bool,
    pub(crate) external_venue_connection: bool,
    #[serde(default)]
    pub(crate) production_venue_connection: bool,
    #[serde(default)]
    pub(crate) testnet_public_network_connection: bool,
    #[serde(default)]
    pub(crate) external_network_attempted: bool,
    pub(crate) real_funds: bool,
    pub(crate) production_trading: bool,
    pub(crate) real_orders_submitted: bool,
    pub(crate) testnet_connection: bool,
    pub(crate) network_attempted: bool,
    pub(crate) credential_policy: String,
    pub(crate) connectivity_mode: String,
    pub(crate) order_submission_mode: String,
    pub(crate) reconciliation_mode: String,
    pub(crate) notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorkflowSummary {
    pub(crate) schema_version: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow: String,
    pub(crate) run_id: String,
    pub(crate) runtime_status: String,
    pub(crate) market_fixture_id: String,
    pub(crate) market_bar_count: usize,
    pub(crate) market_checksum: String,
    pub(crate) ema_smoke_id: String,
    pub(crate) ema_signals_emitted: usize,
    pub(crate) ema_checksum: String,
    pub(crate) rsi_smoke_id: String,
    pub(crate) rsi_signals_emitted: usize,
    pub(crate) rsi_checksum: String,
    pub(crate) order_lifecycle_id: String,
    pub(crate) order_event_count: usize,
    pub(crate) order_checksum: String,
    pub(crate) risk_smoke_id: String,
    pub(crate) risk_checksum: String,
    pub(crate) sandbox_only: bool,
    pub(crate) fixture_replay: bool,
    pub(crate) mock_execution: bool,
    pub(crate) external_venue_connection: bool,
    #[serde(default)]
    pub(crate) production_venue_connection: bool,
    #[serde(default)]
    pub(crate) testnet_public_network_connection: bool,
    #[serde(default)]
    pub(crate) external_network_attempted: bool,
    pub(crate) real_funds: bool,
    pub(crate) production_trading: bool,
    pub(crate) real_orders_submitted: bool,
    #[serde(default)]
    pub(crate) testnet_connection: bool,
    #[serde(default)]
    pub(crate) network_attempted: bool,
    #[serde(default)]
    pub(crate) requested_mode: String,
    #[serde(default)]
    pub(crate) network_permission_requested: bool,
    #[serde(default)]
    pub(crate) credential_policy: String,
    #[serde(default)]
    pub(crate) connectivity_mode: String,
    #[serde(default)]
    pub(crate) order_submission_mode: String,
    #[serde(default)]
    pub(crate) reconciliation_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorkflowEvent {
    pub(crate) schema_version: String,
    pub(crate) workflow_id: String,
    pub(crate) run_id: String,
    pub(crate) sequence: u64,
    pub(crate) event_type: String,
    pub(crate) status: String,
    pub(crate) artifact: String,
    pub(crate) sandbox_only: bool,
    pub(crate) real_orders_submitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorkflowManifestArtifact {
    pub(crate) path: String,
    pub(crate) schema_version: String,
}

impl WorkflowManifestArtifact {
    pub(crate) fn new(path: String, schema_version: &str) -> Self {
        Self {
            path,
            schema_version: schema_version.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorkflowManifest {
    pub(crate) schema_version: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow: String,
    pub(crate) run_id: String,
    pub(crate) runtime_status: String,
    pub(crate) artifact_count: usize,
    #[serde(default)]
    pub(crate) artifacts: Vec<WorkflowManifestArtifact>,
    pub(crate) summary: WorkflowSummary,
}
