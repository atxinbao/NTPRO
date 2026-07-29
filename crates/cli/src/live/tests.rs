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

//! Behavioral regression coverage for the live CLI product surface.

use super::*;
use crate::strategy_session::{
    StrategyOrderPreflightAccount, StrategyOrderPreflightEndpoint, StrategyOrderPreflightLimits,
    StrategyOrderPreflightMarket, StrategyOrderPreflightRisk, StrategyOrderPreflightSession,
};

#[test]
fn live_module_ownership_boundaries_are_explicit() {
    let root = include_str!("../live.rs");
    let runtime = include_str!("node_runtime.rs");

    assert!(root.contains("mod node_runtime;"));
    assert!(root.contains("#[path = \"live/tests.rs\"]"));
    assert!(!root.contains("mod tests {"));
    assert!(!root.contains("async fn run_live_run_with_command("));

    assert!(runtime.contains("//! Live and sandbox node runtime lifecycle."));
    assert!(runtime.contains("pub(super) async fn run_live_run_with_command("));
    assert!(runtime.contains("async fn wait_for_shutdown_signal()"));
}

#[test]
fn retired_production_mutation_http_executors_are_absent_from_current_source() {
    let source = concat!(
        include_str!("../live.rs"),
        include_str!("command.rs"),
        include_str!("node_runtime.rs"),
    );
    let forbidden = [
        ["fn execute_production_mutation_", "guarded_send"].concat(),
        ["fn execute_production_mutation_", "actual_cancel"].concat(),
        [
            "build_production_mutation_guarded_send_artifact_",
            "with_executor",
        ]
        .concat(),
        ["struct ProductionMutationGuardedSend", "HttpResult"].concat(),
        ["struct ProductionMutationActualCancel", "HttpResult"].concat(),
        ["struct ProductionActualCancel", "SignedRequest"].concat(),
        ["PRODUCTION_MUTATION_HTTP_", "SEND_ENV_ALLOW"].concat(),
        ["production-mutation-guarded-send/", "1.0"].concat(),
        ["production-mutation-actual-cancel-single-shot/", "1.0"].concat(),
    ];

    for marker in forbidden {
        assert!(
            !source.contains(&marker),
            "retired production mutation executor marker remains: {marker}"
        );
    }
}

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

[testnet_order]
enabled = false
mode = "disabled"
manual_gate = "owner-approved-manual"
http_base_url = "https://testnet.binance.vision"
symbol = "BTCUSDT"
instrument_id = "BTCUSDT.BINANCE"
side = "BUY"
order_type = "LIMIT"
time_in_force = "GTC"
price = "1.00"
quantity = "0.00001000"
notional = "0.00001000"
cancel_after_submit_ms = 3000
owner_approval_required = true
manual_env_gate_required = true
production_endpoint_allowed = false
dashboard_order_controls = false

[risk]
kill_switch_enabled = true
kill_switch_active = false

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

#[test]
fn validates_strategy_node_testnet_order_contract() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-001-strategy-contract-{}",
        std::process::id()
    ));
    let path = write_config("strategy-contract", &strategy_node_config(&output_dir));

    validate_strategy_node_config_file(&path).unwrap();
}

#[test]
fn rejects_strategy_node_testnet_order_enabled_by_default() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-001-strategy-order-enabled-{}",
        std::process::id()
    ));
    let config = strategy_node_config(&output_dir).replace("enabled = false", "enabled = true");
    let path = write_config("strategy-order-enabled", &config);

    let error = validate_strategy_node_config_file(&path)
        .unwrap_err()
        .to_string();

    assert!(error.contains("testnet_order.enabled must be false"));
}

#[test]
fn rejects_strategy_node_testnet_order_non_decimal_quantity() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-001-strategy-order-bad-quantity-{}",
        std::process::id()
    ));
    let config = strategy_node_config(&output_dir)
        .replace(r#"quantity = "0.00001000""#, r#"quantity = "1e-5""#);
    let path = write_config("strategy-order-bad-quantity", &config);

    let error = validate_strategy_node_config_file(&path)
        .unwrap_err()
        .to_string();

    assert!(error.contains("testnet_order.quantity must be a positive decimal string"));
}

fn testnet_order_gate_opt(config: PathBuf, all_cli_gates: bool) -> LiveTestnetOrderGateOpt {
    LiveTestnetOrderGateOpt {
        config,
        allow_testnet_order: all_cli_gates,
        confirm_owner_approved_testnet_order: all_cli_gates,
        confirm_tiny_notional: all_cli_gates,
        confirm_cancel_after_submit: all_cli_gates,
    }
}

fn testnet_order_preflight_opt(
    config: PathBuf,
    input: PathBuf,
    output: Option<PathBuf>,
    all_cli_gates: bool,
) -> LiveTestnetOrderPreflightOpt {
    LiveTestnetOrderPreflightOpt {
        config,
        input,
        output,
        allow_testnet_order: all_cli_gates,
        confirm_owner_approved_testnet_order: all_cli_gates,
        confirm_tiny_notional: all_cli_gates,
        confirm_cancel_after_submit: all_cli_gates,
    }
}

fn testnet_order_request_preview_opt(
    config: PathBuf,
    output: Option<PathBuf>,
    all_cli_gates: bool,
) -> LiveTestnetOrderRequestPreviewOpt {
    LiveTestnetOrderRequestPreviewOpt {
        config,
        method: TESTNET_ORDER_METHOD_POST.to_string(),
        endpoint_path: TESTNET_ORDER_ENDPOINT_TEST.to_string(),
        timestamp_ms: 1_718_400_000_000,
        recv_window_ms: 5_000,
        api_key_env: "NTPRO_V100004_API_KEY".to_string(),
        api_secret_env: "NTPRO_V100004_API_SECRET".to_string(),
        orig_client_order_id: None,
        output,
        allow_testnet_order: all_cli_gates,
        confirm_owner_approved_testnet_order: all_cli_gates,
        confirm_tiny_notional: all_cli_gates,
        confirm_cancel_after_submit: all_cli_gates,
    }
}

fn testnet_order_test_preflight_opt(
    config: PathBuf,
    output: Option<PathBuf>,
    all_cli_gates: bool,
) -> LiveTestnetOrderTestPreflightOpt {
    LiveTestnetOrderTestPreflightOpt {
        config,
        timestamp_ms: 1_718_400_000_000,
        recv_window_ms: 5_000,
        api_key_env: "NTPRO_V100005_API_KEY".to_string(),
        api_secret_env: "NTPRO_V100005_API_SECRET".to_string(),
        output,
        allow_testnet_order: all_cli_gates,
        confirm_owner_approved_testnet_order: all_cli_gates,
        confirm_tiny_notional: all_cli_gates,
        confirm_cancel_after_submit: all_cli_gates,
    }
}

fn testnet_execution_artifact_contract_opt(
    config: PathBuf,
    output: Option<PathBuf>,
    all_cli_gates: bool,
) -> LiveTestnetExecutionArtifactContractOpt {
    LiveTestnetExecutionArtifactContractOpt {
        config,
        timestamp_ms: 1_718_400_000_000,
        recv_window_ms: 5_000,
        api_key_env: "NTPRO_V100007_API_KEY".to_string(),
        api_secret_env: "NTPRO_V100007_API_SECRET".to_string(),
        orig_client_order_id: "ntpro-v100007-cancel-only".to_string(),
        output,
        allow_testnet_order: all_cli_gates,
        confirm_owner_approved_testnet_order: all_cli_gates,
        confirm_tiny_notional: all_cli_gates,
        confirm_cancel_after_submit: all_cli_gates,
    }
}

fn production_public_read_probe_opt(
    endpoint: ProductionPublicReadEndpoint,
    output: Option<PathBuf>,
    all_cli_gates: bool,
    manual_online: bool,
) -> LiveProductionPublicReadProbeOpt {
    LiveProductionPublicReadProbeOpt {
        endpoint,
        output,
        manual_online,
        allow_production_public_read: all_cli_gates,
        confirm_read_only: all_cli_gates,
        confirm_no_order_mutation: all_cli_gates,
    }
}

fn all_env_enabled(name: &str) -> Option<String> {
    (!name.is_empty()).then(|| "1".to_string())
}

fn production_account_snapshot_contract_opt(
    output: Option<PathBuf>,
    all_cli_gates: bool,
    manual_online: bool,
) -> LiveProductionAccountSnapshotContractOpt {
    LiveProductionAccountSnapshotContractOpt {
        output,
        manual_online,
        api_key_env: "NTPRO_V110003_API_KEY".to_string(),
        api_secret_env: "NTPRO_V110003_API_SECRET".to_string(),
        recv_window_ms: 5_000,
        allow_production_authenticated_read: all_cli_gates,
        confirm_owner_approved_read_only: all_cli_gates,
        confirm_no_order_mutation: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn production_order_state_readonly_proof_opt(
    endpoint: ProductionOrderStateReadEndpoint,
    output: Option<PathBuf>,
    all_cli_gates: bool,
    manual_online: bool,
) -> LiveProductionOrderStateReadOnlyProofOpt {
    LiveProductionOrderStateReadOnlyProofOpt {
        endpoint,
        symbol: "BTCUSDT".to_string(),
        order_id: (endpoint == ProductionOrderStateReadEndpoint::Order).then_some(12_345),
        orig_client_order_id: None,
        output,
        manual_online,
        api_key_env: "NTPRO_V140001_API_KEY".to_string(),
        api_secret_env: "NTPRO_V140001_API_SECRET".to_string(),
        recv_window_ms: 5_000,
        allow_production_order_state_read: all_cli_gates,
        confirm_owner_approved_read_only: all_cli_gates,
        confirm_no_order_mutation: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
    }
}

fn production_live_alpha_dry_run_order_gate_opt(
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionLiveAlphaDryRunOrderGateOpt {
    LiveProductionLiveAlphaDryRunOrderGateOpt {
        run_id: "v140-live-alpha-dry-run".to_string(),
        session_id: Some("session-1".to_string()),
        strategy_id: "ema_cross_btcusdt_v1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        quantity: "0.001".to_string(),
        notional: "10.00".to_string(),
        output,
        allow_production_live_alpha_dry_run: all_cli_gates,
        confirm_owner_approved_dry_run: all_cli_gates,
        confirm_no_production_order_submission: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_no_execution_adapter_call: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_real_funds: all_cli_gates,
    }
}

fn production_live_alpha_limit_dry_run_order_gate_opt(
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionLiveAlphaDryRunOrderGateOpt {
    LiveProductionLiveAlphaDryRunOrderGateOpt {
        run_id: "v150-live-alpha-request-preview".to_string(),
        session_id: Some("session-v150".to_string()),
        strategy_id: "ema_cross_btcusdt_v1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        quantity: "0.001".to_string(),
        notional: "10.00".to_string(),
        output,
        allow_production_live_alpha_dry_run: all_cli_gates,
        confirm_owner_approved_dry_run: all_cli_gates,
        confirm_no_production_order_submission: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_no_execution_adapter_call: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_real_funds: all_cli_gates,
    }
}

fn production_live_alpha_order_request_preview_opt(
    order_gate: PathBuf,
    manual_approval_lifecycle: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionLiveAlphaOrderRequestPreviewOpt {
    LiveProductionLiveAlphaOrderRequestPreviewOpt {
        run_id: "v150-live-alpha-request-preview".to_string(),
        order_gate,
        manual_approval_lifecycle,
        endpoint_path: TESTNET_ORDER_ENDPOINT_ORDER.to_string(),
        price: "10000.00".to_string(),
        time_in_force: TESTNET_ORDER_GTC_TIF.to_string(),
        timestamp_ms: 1_718_400_000_000,
        recv_window_ms: 5_000,
        api_key_env: "NTPRO_V150002_API_KEY".to_string(),
        api_secret_env: "NTPRO_V150002_API_SECRET".to_string(),
        credential_material: "synthetic".to_string(),
        output,
        allow_production_live_alpha_request_preview: all_cli_gates,
        confirm_owner_approved_request_preview: all_cli_gates,
        confirm_memory_only_signature: all_cli_gates,
        confirm_no_production_order_submission: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_no_execution_adapter_call: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_real_funds: all_cli_gates,
    }
}

struct ManualApprovalLifecycleFixture<'a> {
    approval_state: &'a str,
    run_id: &'a str,
    strategy_id: &'a str,
    symbol: &'a str,
    notional: &'a str,
    now_unix_ms: u64,
    expires_at_unix_ms: u64,
}

struct ManualApprovalLifecycleCase<'a> {
    name: &'a str,
    approval_state: &'a str,
    run_id: &'a str,
    symbol: &'a str,
    notional: &'a str,
    now_unix_ms: u64,
    expires_at_unix_ms: u64,
    expected_issue: &'a str,
}

fn production_live_alpha_manual_approval_lifecycle_opt(
    output: PathBuf,
    fixture: &ManualApprovalLifecycleFixture<'_>,
) -> LiveProductionLiveAlphaManualApprovalLifecycleOpt {
    LiveProductionLiveAlphaManualApprovalLifecycleOpt {
        run_id: fixture.run_id.to_string(),
        strategy_id: fixture.strategy_id.to_string(),
        symbol: fixture.symbol.to_string(),
        notional: fixture.notional.to_string(),
        approval_state: fixture.approval_state.to_string(),
        manual_approval_id: (fixture.approval_state != "pending")
            .then(|| "owner-approval-v150-005".to_string()),
        approved_by: (fixture.approval_state != "pending").then(|| "owner".to_string()),
        now_unix_ms: fixture.now_unix_ms,
        expires_at_unix_ms: fixture.expires_at_unix_ms,
        output,
        confirm_dry_run_request_preview_only: true,
        confirm_one_time_approval: true,
        confirm_no_production_mutation: true,
        confirm_dashboard_order_controls_disabled: true,
    }
}

fn production_live_alpha_execution_dry_run_opt(
    order_gate: PathBuf,
    risk_preflight: PathBuf,
    request_preview: PathBuf,
    kill_switch_runtime_gate: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionLiveAlphaExecutionDryRunOpt {
    LiveProductionLiveAlphaExecutionDryRunOpt {
        run_id: "v150-live-alpha-execution-dry-run".to_string(),
        order_gate,
        risk_preflight,
        request_preview,
        kill_switch_runtime_gate,
        output,
        allow_production_live_alpha_execution_dry_run: all_cli_gates,
        confirm_owner_approved_execution_dry_run: all_cli_gates,
        confirm_dry_run_adapter_only: all_cli_gates,
        confirm_no_production_adapter: all_cli_gates,
        confirm_no_production_order_submission: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_real_funds: all_cli_gates,
    }
}

fn production_live_alpha_kill_switch_runtime_gate_opt(
    kill_switch_approval: PathBuf,
    risk_preflight: PathBuf,
    request_preview: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionLiveAlphaKillSwitchRuntimeGateOpt {
    LiveProductionLiveAlphaKillSwitchRuntimeGateOpt {
        run_id: "v150-live-alpha-kill-switch-runtime-gate".to_string(),
        kill_switch_approval,
        risk_preflight,
        request_preview,
        output,
        allow_production_live_alpha_kill_switch_runtime_gate: all_cli_gates,
        confirm_owner_approved_runtime_gate: all_cli_gates,
        confirm_no_production_order_submission: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_real_funds: all_cli_gates,
    }
}

fn production_mutation_runtime_gate_opt(
    order_gate: PathBuf,
    risk_preflight: PathBuf,
    request_preview: PathBuf,
    kill_switch_runtime_gate: PathBuf,
    signing_approval: Option<PathBuf>,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationRuntimeGateOpt {
    LiveProductionMutationRuntimeGateOpt {
        run_id: "v160-production-mutation-runtime-gate".to_string(),
        order_gate,
        risk_preflight,
        request_preview,
        kill_switch_runtime_gate,
        signing_approval,
        output,
        max_notional: "10.00".to_string(),
        allow_production_mutation_runtime_gate: all_cli_gates,
        confirm_owner_approved_production_mutation: all_cli_gates,
        confirm_single_limit_gtc: all_cli_gates,
        confirm_tiny_notional: all_cli_gates,
        confirm_signing_approval_required: all_cli_gates,
        confirm_no_network_before_send: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
        confirm_no_retry: all_cli_gates,
    }
}

fn production_mutation_signing_approval_opt(
    request_preview: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationSigningApprovalOpt {
    LiveProductionMutationSigningApprovalOpt {
        run_id: "v160-production-mutation-signing-approval".to_string(),
        request_preview,
        approval_state: "approved".to_string(),
        manual_approval_id: Some("owner-approval-v160-003".to_string()),
        approved_by: Some("owner".to_string()),
        now_unix_ms: 1_718_400_000_000,
        expires_at_unix_ms: 1_718_400_060_000,
        output,
        allow_production_mutation_signing_approval: all_cli_gates,
        confirm_owner_approved_signing_material: all_cli_gates,
        confirm_env_only_signing_material: all_cli_gates,
        confirm_memory_only_signing: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_production_order_submission: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
    }
}

fn production_mutation_request_builder_opt(
    runtime_gate: PathBuf,
    signing_approval: PathBuf,
    request_preview: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationRequestBuilderOpt {
    LiveProductionMutationRequestBuilderOpt {
        run_id: "v160-production-mutation-request-builder".to_string(),
        runtime_gate,
        signing_approval,
        request_preview,
        api_key_env: "NTPRO_V150002_API_KEY".to_string(),
        api_secret_env: "NTPRO_V150002_API_SECRET".to_string(),
        timestamp_ms: 1_718_400_000_000,
        recv_window_ms: 5_000,
        max_notional: "10.00".to_string(),
        market_reference_source: "fixture_mid_price".to_string(),
        market_reference_price: "10001.00".to_string(),
        max_reference_price_distance_bps: "50".to_string(),
        would_cross_spread: false,
        output,
        allow_production_mutation_request_builder: all_cli_gates,
        confirm_owner_approved_request_builder: all_cli_gates,
        confirm_single_limit_gtc: all_cli_gates,
        confirm_tiny_notional: all_cli_gates,
        confirm_non_marketable_price: all_cli_gates,
        confirm_owner_acknowledged_no_cancel_path: all_cli_gates,
        confirm_signing_approval_ready: all_cli_gates,
        confirm_memory_only_signing: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_production_order_submission: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
        confirm_no_retry: all_cli_gates,
    }
}

fn production_mutation_guarded_send_opt(
    request_builder: PathBuf,
    kill_switch_runtime_gate: PathBuf,
    request_preview: PathBuf,
    output: PathBuf,
    _historical_manual_online: bool,
    all_cli_gates: bool,
) -> LiveProductionMutationGuardedSendOpt {
    LiveProductionMutationGuardedSendOpt {
        run_id: "v160-production-mutation-guarded-send".to_string(),
        request_builder,
        kill_switch_runtime_gate,
        request_preview,
        timestamp_ms: 1_718_400_000_000,
        recv_window_ms: 5_000,
        max_notional: "10.00".to_string(),
        output,
        allow_production_mutation_guarded_send: all_cli_gates,
        confirm_owner_approved_guarded_send: all_cli_gates,
        confirm_single_limit_gtc: all_cli_gates,
        confirm_tiny_notional: all_cli_gates,
        confirm_single_shot: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_response_redacted: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
    }
}

#[test]
fn retired_production_mutation_guarded_send_counters_are_always_zero() {
    let offline = retired_production_mutation_guarded_send_counters();
    assert!(!offline.request_sent);
    assert!(!offline.network_attempted);
    assert!(!offline.production_order_request_attempted);
    assert!(!offline.http_send_attempted);
    assert!(!offline.exchange_ack_observed);
    assert!(!offline.confirmed_production_order_submission);
    assert_eq!(offline.production_order_submissions_attempted, 0);
    assert_eq!(offline.production_orders_submitted, 0);
    assert_eq!(offline.production_order_mutations_attempted, 0);
    assert!(!offline.real_orders_submitted);
    assert!(!offline.platform_production_trading_enabled);
    assert!(!offline.production_trading_enabled);
}

fn production_mutation_response_redaction_opt(
    guarded_send: PathBuf,
    response: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationResponseRedactionOpt {
    LiveProductionMutationResponseRedactionOpt {
        run_id: "v160-production-mutation-response-redaction".to_string(),
        guarded_send,
        response,
        output,
        allow_production_mutation_response_redaction: all_cli_gates,
        confirm_owner_approved_response_redaction: all_cli_gates,
        confirm_no_raw_response_persistence: all_cli_gates,
        confirm_no_headers_persistence: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_order_metadata_only: all_cli_gates,
        confirm_no_account_balances: all_cli_gates,
        confirm_no_unrestricted_payload: all_cli_gates,
        confirm_no_retry: all_cli_gates,
    }
}

fn production_mutation_order_state_readback_opt(
    response_redaction: PathBuf,
    output: PathBuf,
    manual_online: bool,
    all_cli_gates: bool,
) -> LiveProductionMutationOrderStateReadbackOpt {
    LiveProductionMutationOrderStateReadbackOpt {
        run_id: "v160-production-mutation-order-state-readback".to_string(),
        response_redaction,
        output,
        manual_online,
        api_key_env: "NTPRO_V160007_API_KEY".to_string(),
        api_secret_env: "NTPRO_V160007_API_SECRET".to_string(),
        recv_window_ms: 5_000,
        allow_production_mutation_order_state_readback: all_cli_gates,
        confirm_owner_approved_order_state_readback: all_cli_gates,
        confirm_known_order_identifier_only: all_cli_gates,
        confirm_read_only_get_order: all_cli_gates,
        confirm_response_redacted: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
    }
}

fn production_mutation_audit_trail_opt(
    request_builder: PathBuf,
    guarded_send: PathBuf,
    response_redaction: PathBuf,
    order_state_readback: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationAuditTrailOpt {
    LiveProductionMutationAuditTrailOpt {
        run_id: "v160-production-mutation-audit-trail".to_string(),
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        output,
        allow_production_mutation_audit_trail: all_cli_gates,
        confirm_owner_approved_audit_trail: all_cli_gates,
        confirm_redacted_artifacts_only: all_cli_gates,
        confirm_no_secret_or_raw_payload_persistence: all_cli_gates,
        confirm_no_retry_or_followup_mutation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
    }
}

fn production_mutation_failure_semantics_opt(
    audit_trail: PathBuf,
    output: PathBuf,
    failure_mode: ProductionMutationFailureMode,
    all_cli_gates: bool,
) -> LiveProductionMutationFailureSemanticsOpt {
    LiveProductionMutationFailureSemanticsOpt {
        run_id: "v160-production-mutation-failure-semantics".to_string(),
        audit_trail,
        failure_mode,
        output,
        allow_production_mutation_failure_semantics: all_cli_gates,
        confirm_evidence_only_failure_handling: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_automatic_cancel_replace_amend: all_cli_gates,
        confirm_no_correction_or_flatten: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_strategy_continuation: all_cli_gates,
        confirm_no_listen_key_lifecycle: all_cli_gates,
    }
}

fn production_mutation_local_order_ledger_opt(
    sources: (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf, PathBuf),
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationLocalOrderLedgerOpt {
    let (
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        audit_trail,
        failure_semantics,
    ) = sources;
    LiveProductionMutationLocalOrderLedgerOpt {
        run_id: "v170-production-mutation-local-order-ledger".to_string(),
        order_lineage_id: "lineage-v160-single-shot".to_string(),
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        audit_trail,
        failure_semantics,
        output,
        allow_production_mutation_local_order_ledger: all_cli_gates,
        confirm_single_v16_mutation_candidate_lineage: all_cli_gates,
        confirm_read_only_reconciliation_scope: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_duplicate_submit: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn production_mutation_exchange_readback_mapper_opt(
    local_order_ledger: PathBuf,
    order_readback: PathBuf,
    open_orders_readback: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationExchangeReadbackMapperOpt {
    LiveProductionMutationExchangeReadbackMapperOpt {
        run_id: "v170-production-mutation-exchange-readback-mapper".to_string(),
        local_order_ledger,
        order_readback,
        open_orders_readback,
        output,
        allow_production_mutation_exchange_readback_mapper: all_cli_gates,
        confirm_redacted_readback_metadata_only: all_cli_gates,
        confirm_known_order_identifier_only: all_cli_gates,
        confirm_read_only_reconciliation_scope: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
    }
}

fn production_mutation_reconciliation_classifier_opt(
    exchange_readback_mapper: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationReconciliationClassifierOpt {
    LiveProductionMutationReconciliationClassifierOpt {
        run_id: "v170-production-mutation-reconciliation-classifier".to_string(),
        exchange_readback_mapper,
        output,
        allow_production_mutation_reconciliation_classifier: all_cli_gates,
        confirm_single_v16_mutation_candidate_lineage: all_cli_gates,
        confirm_read_only_reconciliation_scope: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn production_mutation_orphan_order_detector_opt(
    reconciliation_classifier: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationOrphanOrderDetectorOpt {
    LiveProductionMutationOrphanOrderDetectorOpt {
        run_id: "v170-production-mutation-orphan-order-detector".to_string(),
        reconciliation_classifier,
        output,
        allow_production_mutation_orphan_order_detector: all_cli_gates,
        confirm_single_v16_mutation_candidate_lineage: all_cli_gates,
        confirm_read_only_reconciliation_scope: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn production_mutation_cancel_request_preview_opt(
    orphan_order_detector: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationCancelRequestPreviewOpt {
    LiveProductionMutationCancelRequestPreviewOpt {
        run_id: "v180-production-mutation-cancel-request-preview".to_string(),
        orphan_order_detector,
        account_label: "prod-account-redacted".to_string(),
        output,
        allow_production_mutation_cancel_request_preview: all_cli_gates,
        confirm_single_v16_mutation_candidate_lineage: all_cli_gates,
        confirm_orphan_risk_halted: all_cli_gates,
        confirm_manual_review_required: all_cli_gates,
        confirm_known_order_identifier_only: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn production_mutation_cancel_risk_gate_opt(
    cancel_request_preview: PathBuf,
    output: PathBuf,
    expected_symbol: &str,
    expected_account_label: &str,
    all_cli_gates: bool,
) -> LiveProductionMutationCancelRiskGateOpt {
    LiveProductionMutationCancelRiskGateOpt {
        run_id: "v180-production-mutation-cancel-risk-gate".to_string(),
        cancel_request_preview,
        expected_symbol: expected_symbol.to_string(),
        expected_account_label: expected_account_label.to_string(),
        output,
        allow_production_mutation_cancel_risk_gate: all_cli_gates,
        confirm_single_v16_mutation_candidate_lineage: all_cli_gates,
        confirm_cancel_request_preview_ready: all_cli_gates,
        confirm_orphan_risk_halted: all_cli_gates,
        confirm_known_order_identifier_only: all_cli_gates,
        confirm_symbol_account_scope: all_cli_gates,
        confirm_owner_approval_required: all_cli_gates,
        confirm_no_cancel_all_or_bulk: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn production_mutation_manual_owner_approval_lifecycle_opt(
    cancel_risk_gate: PathBuf,
    output: PathBuf,
    approval_state: &str,
    all_cli_gates: bool,
) -> LiveProductionMutationManualOwnerApprovalLifecycleOpt {
    LiveProductionMutationManualOwnerApprovalLifecycleOpt {
        run_id: "v180-production-mutation-manual-owner-approval-lifecycle".to_string(),
        cancel_risk_gate,
        approval_state: approval_state.to_string(),
        manual_approval_id: (approval_state != "pending")
            .then(|| "owner-approval-v180-005".to_string()),
        approved_by: (approval_state != "pending").then(|| "owner".to_string()),
        now_unix_ms: 1_718_400_000_000,
        expires_at_unix_ms: 1_718_400_060_000,
        output,
        allow_production_mutation_manual_owner_approval_lifecycle: all_cli_gates,
        confirm_one_order_cancel_candidate: all_cli_gates,
        confirm_one_time_approval: all_cli_gates,
        confirm_non_reusable_approval: all_cli_gates,
        confirm_approval_expiry: all_cli_gates,
        confirm_no_strategy_auto_approval: all_cli_gates,
        confirm_no_background_auto_approval: all_cli_gates,
        confirm_no_dashboard_cancel_approval: all_cli_gates,
        confirm_no_incident_handler_auto_approval: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn write_v190_actual_cancel_owner_approval_source_files(output_dir: &Path) -> (PathBuf, PathBuf) {
    let safety_contract = output_dir.join("v0_19_0_actual_cancel_safety_contract.md");
    fs::write(
        &safety_contract,
        "# v0.19.0 Actual Cancel Safety Contract

schema_version = ntpro.v190_actual_cancel_safety_contract.v1
capability = Owner-Approved Single-Shot Actual Cancel
approval_scope = one_order_one_venue_one_attempt
missing_owner_approval
owner_approval_reused
bulk_cancel_requested
retry_or_repair_requested
dashboard_operation_requested
",
    )
    .unwrap();

    let release_manifest = output_dir.join("v0_18_1_release_manifest.json");
    fs::write(
        &release_manifest,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "schema_version": "ntpro.v181_release_manifest.v1",
                "product_version": "v0.18.1",
                "patch_release": {
                    "planned_tag": "ntpro-rust-only-v0.18.1",
                    "publication_status": "not_published"
                },
                "capability": {
                    "actual_cancel_scope": "not_included"
                },
                "boundary_flags": {
                    "actual_cancel_send_allowed": false,
                    "automatic_cancel_allowed": false,
                    "dashboard_cancel_controls_enabled": false,
                    "network_cancel_endpoint_attempted": false
                }
            }))
            .unwrap()
        ),
    )
    .unwrap();

    (safety_contract, release_manifest)
}

fn production_mutation_actual_cancel_owner_approval_lifecycle_opt(
    actual_cancel_safety_contract: PathBuf,
    release_manifest: PathBuf,
    cancel_risk_gate: PathBuf,
    output: PathBuf,
    approval_state: &str,
    all_cli_gates: bool,
) -> LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt {
    LiveProductionMutationActualCancelOwnerApprovalLifecycleOpt {
        run_id: "v190-actual-cancel-owner-approval-lifecycle".to_string(),
        actual_cancel_safety_contract,
        release_manifest,
        cancel_risk_gate,
        expected_order_lineage_id: "lineage-v160-single-shot".to_string(),
        expected_symbol: "BTCUSDT".to_string(),
        expected_account_label: "prod-account-redacted".to_string(),
        venue: "binance_spot".to_string(),
        expected_release_tag: "ntpro-rust-only-v0.18.1".to_string(),
        approval_state: approval_state.to_string(),
        manual_approval_id: (approval_state != "created")
            .then(|| "owner-approval-v190-003".to_string()),
        approved_by: (approval_state != "created").then(|| "owner".to_string()),
        approval_reason: (approval_state != "created")
            .then(|| "orphan-risk-single-order-cancel".to_string()),
        now_unix_ms: 1_718_400_000_000,
        expires_at_unix_ms: 1_718_400_060_000,
        output,
        allow_production_mutation_actual_cancel_owner_approval_lifecycle: all_cli_gates,
        confirm_actual_cancel_safety_contract: all_cli_gates,
        confirm_one_order_one_venue_one_attempt: all_cli_gates,
        confirm_single_use_approval: all_cli_gates,
        confirm_approval_expiry: all_cli_gates,
        confirm_bind_order_risk_gate_release_provenance: all_cli_gates,
        confirm_audit_evidence: all_cli_gates,
        confirm_no_dashboard_approval: all_cli_gates,
        confirm_no_automatic_cancel: all_cli_gates,
        confirm_no_bulk_cancel: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_submit_lifecycle: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn write_ready_v190_actual_cancel_owner_approval_lifecycle_artifact(output_dir: &Path) -> PathBuf {
    let (safety_contract, release_manifest) =
        write_v190_actual_cancel_owner_approval_source_files(output_dir);
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let output = output_dir.join("actual-cancel-owner-approval-lifecycle.json");
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
        &production_mutation_actual_cancel_owner_approval_lifecycle_opt(
            safety_contract,
            release_manifest,
            risk_gate,
            output.clone(),
            "approved",
            true,
        ),
    )
    .unwrap();
    output
}

fn write_v190_actual_cancel_adapter_capability(
    output_dir: &Path,
    name: &str,
    actual_cancel_supported: bool,
    venues: &[&str],
    order_id_types: &[&str],
) -> PathBuf {
    let adapter_capability = output_dir.join(name);
    fs::write(
        &adapter_capability,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "schema_version": "ntpro.v190_actual_cancel_adapter_capability.v1",
                "artifact_type": "actual_cancel_adapter_capability",
                "adapter_id": "binance_spot_cancel_adapter",
                "actual_cancel_supported": actual_cancel_supported,
                "supported_venues": venues,
                "supported_order_id_types": order_id_types,
                "bulk_cancel_supported": false,
                "cancel_all_supported": false,
                "retry_supported": false,
                "automatic_cancel_supported": false,
                "multi_venue_supported": false
            }))
            .unwrap()
        ),
    )
    .unwrap();
    adapter_capability
}

fn production_mutation_actual_cancel_executor_adapter_boundary_opt(
    owner_approval_lifecycle: PathBuf,
    adapter_capability: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt {
    LiveProductionMutationActualCancelExecutorAdapterBoundaryOpt {
        run_id: "v190-actual-cancel-executor-adapter-boundary".to_string(),
        owner_approval_lifecycle,
        adapter_capability,
        adapter_id: "binance_spot_cancel_adapter".to_string(),
        venue: "binance_spot".to_string(),
        order_id_type: "exchange_order_id".to_string(),
        expected_order_lineage_id: "lineage-v160-single-shot".to_string(),
        expected_symbol: "BTCUSDT".to_string(),
        expected_account_label: "prod-account-redacted".to_string(),
        output,
        allow_production_mutation_actual_cancel_executor_adapter_boundary: all_cli_gates,
        confirm_adapter_capability: all_cli_gates,
        confirm_request_response_readback_audit_contract: all_cli_gates,
        confirm_one_order_one_venue_one_attempt: all_cli_gates,
        confirm_fail_closed_unsupported_capability: all_cli_gates,
        confirm_no_bulk_cancel: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_automatic_cancel: all_cli_gates,
        confirm_no_dashboard_execution: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn write_ready_v190_actual_cancel_adapter_boundary_artifact(
    output_dir: &Path,
    owner_approval_lifecycle: PathBuf,
    adapter_capability: PathBuf,
) -> PathBuf {
    let output = output_dir.join("actual-cancel-executor-adapter-boundary.json");
    run_live_production_mutation_actual_cancel_executor_adapter_boundary(
        &production_mutation_actual_cancel_executor_adapter_boundary_opt(
            owner_approval_lifecycle,
            adapter_capability,
            output.clone(),
            true,
        ),
    )
    .unwrap();
    output
}

#[derive(Clone)]
struct V190ActualCancelSingleShotSourceChain {
    actual_cancel_safety_contract: PathBuf,
    release_manifest: PathBuf,
    cancel_risk_gate: PathBuf,
    owner_approval_lifecycle: PathBuf,
    adapter_boundary: PathBuf,
    adapter_capability: PathBuf,
}

fn production_mutation_actual_cancel_single_shot_opt(
    sources: &V190ActualCancelSingleShotSourceChain,
    output: PathBuf,
    _historical_manual_online: bool,
    all_cli_gates: bool,
) -> LiveProductionMutationActualCancelSingleShotOpt {
    LiveProductionMutationActualCancelSingleShotOpt {
        run_id: "v190-actual-cancel-single-shot".to_string(),
        actual_cancel_safety_contract: sources.actual_cancel_safety_contract.clone(),
        release_manifest: sources.release_manifest.clone(),
        cancel_risk_gate: sources.cancel_risk_gate.clone(),
        owner_approval_lifecycle: sources.owner_approval_lifecycle.clone(),
        adapter_boundary: sources.adapter_boundary.clone(),
        adapter_capability: sources.adapter_capability.clone(),
        expected_order_lineage_id: "lineage-v160-single-shot".to_string(),
        expected_symbol: "BTCUSDT".to_string(),
        expected_account_label: "prod-account-redacted".to_string(),
        venue: "binance_spot".to_string(),
        order_id_type: "exchange_order_id".to_string(),
        expected_release_tag: "ntpro-rust-only-v0.18.1".to_string(),
        cancel_order_id: Some("123456789".to_string()),
        cancel_orig_client_order_id: None,
        timestamp_ms: 1_718_400_000_000,
        recv_window_ms: 5_000,
        output,
        allow_production_mutation_actual_cancel_single_shot: all_cli_gates,
        confirm_owner_approval: all_cli_gates,
        confirm_risk_gate: all_cli_gates,
        confirm_release_provenance: all_cli_gates,
        confirm_adapter_boundary: all_cli_gates,
        confirm_single_shot: all_cli_gates,
        confirm_consume_approval_before_send: all_cli_gates,
        confirm_readback_required: all_cli_gates,
        confirm_no_bulk_cancel: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_automatic_cancel: all_cli_gates,
        confirm_no_dashboard_execution: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn write_ready_v190_actual_cancel_single_shot_attempt_artifact(output_dir: &Path) -> PathBuf {
    let sources = write_ready_v190_actual_cancel_single_shot_source_chain(output_dir);
    let output = output_dir.join("actual-cancel-single-shot-attempt.json");
    let opt =
        production_mutation_actual_cancel_single_shot_opt(&sources, output.clone(), true, true);
    let artifact = build_production_mutation_actual_cancel_single_shot_artifact(&opt).unwrap();
    let mut historical_attempt = serde_json::to_value(artifact).unwrap();
    let object = historical_attempt.as_object_mut().unwrap();
    for field in [
        "actual_cancel_command_ready",
        "single_shot_cancel_allowed",
        "approval_consumed_before_send",
        "approval_consumed_after_send",
        "request_sent",
        "cancel_attempted",
        "network_attempted",
        "network_cancel_endpoint_attempted",
        "http_send_attempted",
        "venue_ack_observed",
        "readback_required",
    ] {
        object.insert(field.to_string(), json!(true));
    }
    object.insert(
        "status".to_string(),
        json!("actual_cancel_attempt_recorded"),
    );
    object.insert("cancel_requests_sent".to_string(), json!(1));
    object.insert("production_order_mutations_attempted".to_string(), json!(1));
    object.insert("approval_state_after_attempt".to_string(), json!("used"));
    object.insert(
        "readback_requirement".to_string(),
        json!("post_cancel_readback_required_before_any_retry_or_followup"),
    );
    object.insert("venue_response_status".to_string(), json!("accepted"));
    object.insert(
        "venue_response_source".to_string(),
        json!("historical_test_fixture"),
    );
    atomic_write_json(&output, &historical_attempt).unwrap();
    output
}

fn write_ready_v190_actual_cancel_single_shot_source_chain(
    output_dir: &Path,
) -> V190ActualCancelSingleShotSourceChain {
    let (safety_contract, release_manifest) =
        write_v190_actual_cancel_owner_approval_source_files(output_dir);
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let owner_approval_lifecycle = output_dir.join("actual-cancel-owner-approval-lifecycle.json");
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
        &production_mutation_actual_cancel_owner_approval_lifecycle_opt(
            safety_contract.clone(),
            release_manifest.clone(),
            risk_gate.clone(),
            owner_approval_lifecycle.clone(),
            "approved",
            true,
        ),
    )
    .unwrap();
    let adapter_capability = write_v190_actual_cancel_adapter_capability(
        output_dir,
        "adapter-capability-ready.json",
        true,
        &["binance_spot"],
        &["exchange_order_id"],
    );
    let adapter_boundary = write_ready_v190_actual_cancel_adapter_boundary_artifact(
        output_dir,
        owner_approval_lifecycle.clone(),
        adapter_capability.clone(),
    );

    V190ActualCancelSingleShotSourceChain {
        actual_cancel_safety_contract: safety_contract,
        release_manifest,
        cancel_risk_gate: risk_gate,
        owner_approval_lifecycle,
        adapter_boundary,
        adapter_capability,
    }
}

fn production_mutation_actual_cancel_readback_reconciliation_opt(
    actual_cancel_attempt: PathBuf,
    readback: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationActualCancelReadbackReconciliationOpt {
    LiveProductionMutationActualCancelReadbackReconciliationOpt {
        run_id: "v190-actual-cancel-readback-reconciliation".to_string(),
        actual_cancel_attempt,
        readback,
        expected_order_lineage_id: "lineage-v160-single-shot".to_string(),
        expected_symbol: "BTCUSDT".to_string(),
        expected_account_label: "prod-account-redacted".to_string(),
        venue: "binance_spot".to_string(),
        output,
        allow_production_mutation_actual_cancel_readback_reconciliation: all_cli_gates,
        confirm_actual_cancel_attempt_recorded: all_cli_gates,
        confirm_readback_required: all_cli_gates,
        confirm_readback_metadata_only: all_cli_gates,
        confirm_order_status_reconciled: all_cli_gates,
        confirm_execution_fill_status_reconciled: all_cli_gates,
        confirm_remaining_quantity_reconciled: all_cli_gates,
        confirm_risk_state_recorded: all_cli_gates,
        confirm_local_audit_state_recorded: all_cli_gates,
        confirm_dashboard_read_only_consumable: all_cli_gates,
        confirm_no_raw_readback_persistence: all_cli_gates,
        confirm_no_headers_persistence: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_no_second_cancel: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
    }
}

fn production_mutation_actual_cancel_failure_evidence_opt(
    readback_reconciliation: PathBuf,
    request_ref: PathBuf,
    response_ref: PathBuf,
    readback_ref: PathBuf,
    audit_ref: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationActualCancelFailureEvidenceOpt {
    LiveProductionMutationActualCancelFailureEvidenceOpt {
        run_id: "v190-actual-cancel-failure-evidence".to_string(),
        readback_reconciliation,
        request_ref,
        response_ref,
        readback_ref,
        audit_ref,
        expected_order_lineage_id: "lineage-v160-single-shot".to_string(),
        expected_symbol: "BTCUSDT".to_string(),
        expected_account_label: "prod-account-redacted".to_string(),
        venue: "binance_spot".to_string(),
        output,
        allow_production_mutation_actual_cancel_failure_evidence: all_cli_gates,
        confirm_request_ref_recorded: all_cli_gates,
        confirm_response_ref_recorded: all_cli_gates,
        confirm_readback_ref_recorded: all_cli_gates,
        confirm_audit_ref_recorded: all_cli_gates,
        confirm_failure_outcomes_classified: all_cli_gates,
        confirm_operator_action_model: all_cli_gates,
        confirm_unknown_not_recovered: all_cli_gates,
        confirm_partial_fill_residual_risk: all_cli_gates,
        confirm_dashboard_release_gate_consumable: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_no_compensation_trade: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn production_mutation_cancel_response_redaction_opt(
    manual_owner_approval_lifecycle: PathBuf,
    response: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationCancelResponseRedactionOpt {
    LiveProductionMutationCancelResponseRedactionOpt {
        run_id: "v180-production-mutation-cancel-response-redaction".to_string(),
        manual_owner_approval_lifecycle,
        response,
        output,
        allow_production_mutation_cancel_response_redaction: all_cli_gates,
        confirm_manual_owner_approval_lifecycle_ready: all_cli_gates,
        confirm_no_raw_response_persistence: all_cli_gates,
        confirm_no_headers_persistence: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_cancel_metadata_only: all_cli_gates,
        confirm_no_account_balances: all_cli_gates,
        confirm_no_unrestricted_payload: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
    }
}

fn production_mutation_post_cancel_readback_opt(
    cancel_response_redaction: PathBuf,
    readback: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationPostCancelReadbackOpt {
    LiveProductionMutationPostCancelReadbackOpt {
        run_id: "v180-production-mutation-post-cancel-readback".to_string(),
        cancel_response_redaction,
        readback,
        output,
        allow_production_mutation_post_cancel_readback: all_cli_gates,
        confirm_cancel_response_redaction_ready: all_cli_gates,
        confirm_readback_metadata_only: all_cli_gates,
        confirm_terminal_and_ambiguous_classification: all_cli_gates,
        confirm_no_raw_readback_persistence: all_cli_gates,
        confirm_no_headers_persistence: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
        confirm_no_mutation: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
    }
}

fn production_mutation_cancel_recovery_incident_audit_closeout_opt(
    cancel_risk_gate: PathBuf,
    manual_owner_approval_lifecycle: PathBuf,
    cancel_response_redaction: PathBuf,
    post_cancel_readback: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionMutationCancelRecoveryIncidentAuditCloseoutOpt {
    LiveProductionMutationCancelRecoveryIncidentAuditCloseoutOpt {
        run_id: "v180-production-mutation-cancel-recovery-incident-audit-closeout".to_string(),
        cancel_risk_gate,
        manual_owner_approval_lifecycle,
        cancel_response_redaction,
        post_cancel_readback,
        output,
        allow_production_mutation_cancel_recovery_incident_audit_closeout: all_cli_gates,
        confirm_cancel_recovery_lineage: all_cli_gates,
        confirm_risk_reason_recorded: all_cli_gates,
        confirm_risk_gate_result_recorded: all_cli_gates,
        confirm_owner_approval_state_recorded: all_cli_gates,
        confirm_redaction_contract_state_recorded: all_cli_gates,
        confirm_readback_state_recorded: all_cli_gates,
        confirm_terminal_action_recommendation: all_cli_gates,
        confirm_remaining_risk_recorded: all_cli_gates,
        confirm_no_mutation: all_cli_gates,
        confirm_no_cancel: all_cli_gates,
        confirm_no_network: all_cli_gates,
        confirm_no_retry: all_cli_gates,
        confirm_no_remediation: all_cli_gates,
        confirm_no_automatic_remediation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
        confirm_no_secret_persistence: all_cli_gates,
    }
}

fn write_kill_switch_approval_artifact(
    output: PathBuf,
    kill_switch_active: bool,
    approval_state: &str,
) {
    run_live_production_kill_switch_approval_artifact(
        &LiveProductionKillSwitchApprovalArtifactOpt {
            run_id: "v150-live-alpha-kill-switch-runtime-gate".to_string(),
            session_id: Some("session-v150".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            output,
            kill_switch_active,
            approval_state: approval_state.to_string(),
            manual_approval_id: (approval_state == "approved")
                .then(|| "owner-approval-v150-004".to_string()),
            approved_by: (approval_state == "approved").then(|| "owner".to_string()),
            confirm_dry_run_only: true,
            confirm_no_production_mutation: true,
            confirm_dashboard_order_controls_disabled: true,
        },
    )
    .unwrap();
}

fn production_live_alpha_risk_preflight_opt(
    order_gate: PathBuf,
    input: PathBuf,
    output: PathBuf,
    all_cli_gates: bool,
) -> LiveProductionLiveAlphaRiskPreflightOpt {
    LiveProductionLiveAlphaRiskPreflightOpt {
        run_id: "v140-live-alpha-risk".to_string(),
        order_gate,
        input,
        output,
        confirm_hypothetical_dry_run_only: all_cli_gates,
        confirm_no_execution_adapter_call: all_cli_gates,
        confirm_no_production_order_submission: all_cli_gates,
        confirm_no_production_order_mutation: all_cli_gates,
        confirm_dashboard_order_controls_disabled: all_cli_gates,
    }
}

fn write_ready_live_alpha_artifact_chain(output_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let risk_input = output_dir.join("live_alpha_risk_input.json");
    let risk_preflight = output_dir.join("live_alpha_risk_preflight.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let request_preview = output_dir.join("live_alpha_order_request_preview.json");

    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    let mut input = passing_live_alpha_risk_input();
    input.order.order_type = "LIMIT".to_string();
    write_live_alpha_risk_input(&risk_input, &input);
    run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
        order_gate.clone(),
        risk_input,
        risk_preflight.clone(),
        true,
    ))
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();
    let request_opt = production_live_alpha_order_request_preview_opt(
        order_gate.clone(),
        manual_approval_lifecycle,
        request_preview.clone(),
        true,
    );
    run_live_production_live_alpha_order_request_preview_with_env(
        &request_opt,
        |name| match name {
            "NTPRO_V150002_API_KEY" => Some("ntpro_v150003_synthetic_api_key_value".to_string()),
            "NTPRO_V150002_API_SECRET" => {
                Some("ntpro_v150003_synthetic_api_secret_value".to_string())
            }
            _ => None,
        },
    )
    .unwrap();

    (order_gate, risk_preflight, request_preview)
}

fn write_ready_live_alpha_production_material_artifact_chain(
    output_dir: &Path,
) -> (PathBuf, PathBuf, PathBuf) {
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let risk_input = output_dir.join("live_alpha_risk_input.json");
    let risk_preflight = output_dir.join("live_alpha_risk_preflight.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let request_preview = output_dir.join("live_alpha_order_request_preview.json");

    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    let mut input = passing_live_alpha_risk_input();
    input.order.order_type = "LIMIT".to_string();
    write_live_alpha_risk_input(&risk_input, &input);
    run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
        order_gate.clone(),
        risk_input,
        risk_preflight.clone(),
        true,
    ))
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();
    let mut request_opt = production_live_alpha_order_request_preview_opt(
        order_gate.clone(),
        manual_approval_lifecycle,
        request_preview.clone(),
        true,
    );
    request_opt.credential_material = "production_live_alpha".to_string();
    let production_api_key = "ntpro_v160003_production_like_api_key_value";
    let production_api_secret = "ntpro_v160003_production_like_api_secret_value";
    run_live_production_live_alpha_order_request_preview_with_env(
        &request_opt,
        |name| match name {
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
            | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
            "NTPRO_V150002_API_KEY" => Some(production_api_key.to_string()),
            "NTPRO_V150002_API_SECRET" => Some(production_api_secret.to_string()),
            _ => None,
        },
    )
    .unwrap();

    let body = fs::read_to_string(&request_preview).unwrap();
    assert!(!body.contains(production_api_key));
    assert!(!body.contains(production_api_secret));
    assert!(!body.contains("signature="));

    (order_gate, risk_preflight, request_preview)
}

fn write_ready_v160_request_builder_sources(output_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let (_, risk_preflight, request_preview) =
        write_ready_live_alpha_production_material_artifact_chain(output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
    let signing_approval = output_dir.join("production_mutation_signing_approval.json");
    let runtime_gate = output_dir.join("production_mutation_runtime_gate.json");

    write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight.clone(),
            request_preview.clone(),
            kill_switch_runtime_gate.clone(),
            true,
        ),
    )
    .unwrap();
    run_live_production_mutation_signing_approval(&production_mutation_signing_approval_opt(
        request_preview.clone(),
        signing_approval.clone(),
        true,
    ))
    .unwrap();
    run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
        output_dir.join("live_alpha_dry_run_order_gate.json"),
        risk_preflight,
        request_preview.clone(),
        kill_switch_runtime_gate,
        Some(signing_approval.clone()),
        runtime_gate.clone(),
        true,
    ))
    .unwrap();

    (runtime_gate, signing_approval, request_preview)
}

fn write_ready_v160_guarded_send_sources(output_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let (runtime_gate, signing_approval, request_preview) =
        write_ready_v160_request_builder_sources(output_dir);
    let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
    let request_builder = output_dir.join("production_mutation_request_builder.json");
    let production_api_key = "ntpro_v160005_production_like_api_key_value";
    let production_api_secret = "ntpro_v160005_production_like_api_secret_value";

    run_live_production_mutation_request_builder_with_env(
        &production_mutation_request_builder_opt(
            runtime_gate,
            signing_approval,
            request_preview.clone(),
            request_builder.clone(),
            true,
        ),
        |name| match name {
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
            | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
            "NTPRO_V150002_API_KEY" => Some(production_api_key.to_string()),
            "NTPRO_V150002_API_SECRET" => Some(production_api_secret.to_string()),
            _ => None,
        },
    )
    .unwrap();

    let body = fs::read_to_string(&request_builder).unwrap();
    assert!(!body.contains(production_api_key));
    assert!(!body.contains(production_api_secret));
    assert!(!body.contains("symbol=BTCUSDT"));

    (request_builder, request_preview, kill_switch_runtime_gate)
}

fn write_ready_v160_guarded_send_artifact(output_dir: &Path) -> PathBuf {
    let (request_builder, request_preview, kill_switch_runtime_gate) =
        write_ready_v160_guarded_send_sources(output_dir);
    let guarded_send = output_dir.join("production_mutation_guarded_send.json");
    run_live_production_mutation_guarded_send(&production_mutation_guarded_send_opt(
        request_builder,
        kill_switch_runtime_gate,
        request_preview,
        guarded_send.clone(),
        false,
        true,
    ))
    .unwrap();
    guarded_send
}

fn write_actual_v161_guarded_send_http_result_artifact(output_dir: &Path) -> PathBuf {
    let guarded_send = write_ready_v160_guarded_send_artifact(output_dir);
    let mut artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&guarded_send).unwrap()).unwrap();
    artifact["status"] =
        serde_json::Value::String("manual_online_send_attempt_recorded".to_string());
    artifact["manual_online_requested"] = serde_json::Value::Bool(true);
    artifact["request_sent"] = serde_json::Value::Bool(true);
    artifact["network_attempted"] = serde_json::Value::Bool(true);
    artifact["production_order_request_attempted"] = serde_json::Value::Bool(true);
    artifact["http_send_attempted"] = serde_json::Value::Bool(true);
    artifact["exchange_ack_observed"] = serde_json::Value::Bool(true);
    artifact["confirmed_production_order_submission"] = serde_json::Value::Bool(true);
    artifact["production_order_submissions_attempted"] = serde_json::Value::Number(1.into());
    artifact["production_orders_submitted"] = serde_json::Value::Number(1.into());
    artifact["production_order_mutations_attempted"] = serde_json::Value::Number(1.into());
    artifact["real_orders_submitted"] = serde_json::Value::Bool(true);
    artifact["production_trading_enabled"] = serde_json::Value::Bool(false);
    fs::write(
        &guarded_send,
        serde_json::to_string_pretty(&artifact).unwrap(),
    )
    .unwrap();
    guarded_send
}

fn write_synthetic_production_mutation_response(path: &Path) {
    fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "BTCUSDT",
            "orderId": 123456789,
            "clientOrderId": "owner-approved-v160-single-shot",
            "transactTime": 1718400000000_u64,
            "workingTime": 1718400000001_u64,
            "status": "NEW",
            "type": "LIMIT",
            "side": "BUY",
            "timeInForce": "GTC"
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_forbidden_production_mutation_response(path: &Path) {
    fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "BTCUSDT",
            "orderId": 123456789,
            "clientOrderId": "owner-approved-v160-single-shot",
            "status": "NEW",
            "type": "LIMIT",
            "side": "BUY",
            "headers": {"X-MBX-APIKEY": "must_not_persist"},
            "signature": "signature=must_not_persist",
            "balances": [{"asset": "USDT", "free": "100.0"}],
            "payload": {"raw": "unrestricted"}
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_synthetic_production_mutation_cancel_response(path: &Path) {
    fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "BTCUSDT",
            "orderId": 123456789,
            "clientOrderId": "owner-approved-v160-single-shot",
            "origClientOrderId": "owner-approved-v160-single-shot",
            "transactTime": 1718400000000_u64,
            "status": "CANCELED"
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_forbidden_production_mutation_cancel_response(path: &Path) {
    fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "BTCUSDT",
            "orderId": 123456789,
            "clientOrderId": "owner-approved-v160-single-shot",
            "origClientOrderId": "owner-approved-v160-single-shot",
            "status": "CANCELED",
            "headers": {"X-MBX-APIKEY": "must_not_persist"},
            "body": {"raw": "raw response must not persist"},
            "signature": "signature=must_not_persist",
            "payload": {"raw": "unrestricted"}
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_synthetic_production_mutation_post_cancel_readback(path: &Path, status: &str) {
    fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "BTCUSDT",
            "orderId": 123456789,
            "clientOrderId": "owner-approved-v160-single-shot",
            "origClientOrderId": "owner-approved-v160-single-shot",
            "updateTime": 1718400000001_u64,
            "status": status
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_synthetic_v190_actual_cancel_readback_reconciliation(
    path: &Path,
    status: &str,
    executed_qty: &str,
    orig_qty: &str,
    readback_result: Option<&str>,
    already_cancelled: bool,
) {
    let mut value = serde_json::json!({
        "symbol": "BTCUSDT",
        "orderId": 123456789,
        "clientOrderId": "owner-approved-v160-single-shot",
        "origClientOrderId": "owner-approved-v160-single-shot",
        "updateTime": 1718400000001_u64,
        "status": status,
        "executedQty": executed_qty,
        "origQty": orig_qty,
        "remainingQty": "0",
        "localAuditState": "actual_cancel_attempt_recorded"
    });
    if let Some(result) = readback_result {
        value["readbackResult"] = serde_json::Value::String(result.to_string());
    }
    if already_cancelled {
        value["alreadyCancelled"] = serde_json::Value::Bool(true);
    }
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn write_synthetic_v190_actual_cancel_failure_evidence_ref(
    path: &Path,
    artifact_type: &str,
    status: &str,
    extra: &serde_json::Value,
) {
    let mut value = serde_json::json!({
        "schema_version": "ntpro.synthetic_actual_cancel_failure_evidence_ref.v1",
        "artifact_type": artifact_type,
        "status": status,
        "ready": true,
        "order_lineage_id": "lineage-v160-single-shot",
        "symbol": "BTCUSDT",
        "account_label": "prod-account-redacted",
        "venue": "binance_spot",
    });
    if let Some(object) = extra.as_object() {
        for (key, extra_value) in object {
            value[key] = extra_value.clone();
        }
    }
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn write_forbidden_production_mutation_post_cancel_readback(path: &Path) {
    fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "BTCUSDT",
            "orderId": 123456789,
            "clientOrderId": "owner-approved-v160-single-shot",
            "origClientOrderId": "owner-approved-v160-single-shot",
            "status": "CANCELED",
            "headers": {"X-MBX-APIKEY": "must_not_persist"},
            "body": {"raw": "raw readback must not persist"},
            "apiSecret": "apiSecret must not persist",
            "payload": {"raw": "unrestricted"},
            "fills": [{"price": "1", "qty": "1"}]
        }))
        .unwrap(),
    )
    .unwrap();
}

fn assert_v190_actual_cancel_readback_reconciliation_false_boundary(artifact: &serde_json::Value) {
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "raw_readback_body_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "unrestricted_payload_recorded",
        "account_balances_recorded",
        "fills_recorded",
        "readback_execution_attempted",
        "order_state_read_attempted",
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "network_attempted",
        "network_readback_endpoint_attempted",
        "network_cancel_endpoint_attempted",
        "retry_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "second_cancel_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "production_order_mutation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
    ] {
        assert_eq!(artifact[field], false, "{field}");
    }
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
}

fn assert_v190_actual_cancel_failure_evidence_false_boundary(artifact: &serde_json::Value) {
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "raw_readback_body_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "unrestricted_payload_recorded",
        "account_balances_recorded",
        "fills_recorded",
        "readback_execution_attempted",
        "order_state_read_attempted",
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "network_attempted",
        "network_readback_endpoint_attempted",
        "network_cancel_endpoint_attempted",
        "retry_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "compensation_trade_attempted",
        "second_cancel_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "production_order_mutation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
    ] {
        assert_eq!(artifact[field], false, "{field}");
    }
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
}

fn assert_v180_post_cancel_readback_false_boundary(artifact: &serde_json::Value) {
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "raw_readback_body_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "unrestricted_payload_recorded",
        "account_balances_recorded",
        "fills_recorded",
        "readback_execution_attempted",
        "order_state_read_attempted",
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "network_attempted",
        "network_readback_endpoint_attempted",
        "network_cancel_endpoint_attempted",
        "retry_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "production_order_mutation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
    ] {
        assert_eq!(artifact[field], false, "{field}");
    }
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
}

fn assert_v180_cancel_recovery_incident_audit_closeout_false_boundary(
    artifact: &serde_json::Value,
) {
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "raw_readback_body_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "unrestricted_payload_recorded",
        "account_balances_recorded",
        "fills_recorded",
        "readback_execution_attempted",
        "order_state_read_attempted",
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "network_attempted",
        "network_readback_endpoint_attempted",
        "network_cancel_endpoint_attempted",
        "retry_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "production_order_mutation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
    ] {
        assert_eq!(artifact[field], false, "{field}");
    }
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
}

fn write_ready_v160_response_redaction_artifact(output_dir: &Path) -> PathBuf {
    let guarded_send = write_ready_v160_guarded_send_artifact(output_dir);
    let response = output_dir.join("synthetic_order_response.json");
    let response_redaction = output_dir.join("production_mutation_response_redaction.json");
    write_synthetic_production_mutation_response(&response);
    run_live_production_mutation_response_redaction(&production_mutation_response_redaction_opt(
        guarded_send,
        response,
        response_redaction.clone(),
        true,
    ))
    .unwrap();
    response_redaction
}

fn write_ready_v160_audit_trail_sources(output_dir: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let (request_builder, request_preview, kill_switch_runtime_gate) =
        write_ready_v160_guarded_send_sources(output_dir);
    let guarded_send = output_dir.join("production_mutation_guarded_send.json");
    run_live_production_mutation_guarded_send(&production_mutation_guarded_send_opt(
        request_builder.clone(),
        kill_switch_runtime_gate,
        request_preview,
        guarded_send.clone(),
        false,
        true,
    ))
    .unwrap();

    let response = output_dir.join("synthetic_order_response.json");
    let response_redaction = output_dir.join("production_mutation_response_redaction.json");
    write_synthetic_production_mutation_response(&response);
    run_live_production_mutation_response_redaction(&production_mutation_response_redaction_opt(
        guarded_send.clone(),
        response,
        response_redaction.clone(),
        true,
    ))
    .unwrap();

    let order_state_readback = output_dir.join("production_mutation_order_state_readback.json");
    run_live_production_mutation_order_state_readback(
        &production_mutation_order_state_readback_opt(
            response_redaction.clone(),
            order_state_readback.clone(),
            false,
            true,
        ),
    )
    .unwrap();

    (
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
    )
}

fn write_ready_v160_audit_trail_artifact(output_dir: &Path) -> PathBuf {
    let (request_builder, guarded_send, response_redaction, order_state_readback) =
        write_ready_v160_audit_trail_sources(output_dir);
    let audit_trail = output_dir.join("production_mutation_audit_trail.json");
    run_live_production_mutation_audit_trail(&production_mutation_audit_trail_opt(
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        audit_trail.clone(),
        true,
    ))
    .unwrap();
    audit_trail
}

fn write_ready_v170_local_order_ledger_sources(
    output_dir: &Path,
) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let (request_builder, guarded_send, response_redaction, order_state_readback) =
        write_ready_v160_audit_trail_sources(output_dir);
    let audit_trail = output_dir.join("production_mutation_audit_trail.json");
    run_live_production_mutation_audit_trail(&production_mutation_audit_trail_opt(
        request_builder.clone(),
        guarded_send.clone(),
        response_redaction.clone(),
        order_state_readback.clone(),
        audit_trail.clone(),
        true,
    ))
    .unwrap();

    let failure_semantics = output_dir.join("production_mutation_failure_semantics.json");
    run_live_production_mutation_failure_semantics(&production_mutation_failure_semantics_opt(
        audit_trail.clone(),
        failure_semantics.clone(),
        ProductionMutationFailureMode::ReadbackMismatch,
        true,
    ))
    .unwrap();

    (
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        audit_trail,
        failure_semantics,
    )
}

fn write_ready_v170_local_order_ledger_artifact(output_dir: &Path) -> PathBuf {
    let (
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        audit_trail,
        failure_semantics,
    ) = write_ready_v170_local_order_ledger_sources(output_dir);
    let ledger = output_dir.join("production_mutation_local_order_ledger.json");
    run_live_production_mutation_local_order_ledger(&production_mutation_local_order_ledger_opt(
        (
            request_builder,
            guarded_send,
            response_redaction,
            order_state_readback,
            audit_trail,
            failure_semantics,
        ),
        ledger.clone(),
        true,
    ))
    .unwrap();
    ledger
}

fn write_redacted_exchange_order_readback(
    output: &Path,
    exchange_status: Option<&str>,
    order_found: bool,
) {
    let body = json!({
        "schema_version": PRODUCTION_MUTATION_EXCHANGE_ORDER_READBACK_SCHEMA_VERSION,
        "artifact_type": "redacted_binance_order_readback",
        "status": "ready_redacted_order_readback_metadata",
        "endpoint": "order",
        "method": "GET",
        "path": "/api/v3/order",
        "order_found": order_found,
        "symbol": "BTCUSDT",
        "order_id": "123456789",
        "client_order_id": "owner-approved-v160-single-shot",
        "exchange_status": exchange_status,
        "response_redacted": true,
        "api_key_value_recorded": false,
        "api_secret_value_recorded": false,
        "api_key_header_value_recorded": false,
        "signature_recorded": false,
        "signed_query_recorded": false,
        "signed_url_recorded": false,
        "raw_exchange_response_recorded": false,
        "response_body_recorded": false,
        "response_headers_recorded": false,
        "request_sent": false,
        "network_attempted": false,
        "retry_attempted": false,
        "cancel_attempted": false,
        "replace_attempted": false,
        "amend_attempted": false,
        "flatten_attempted": false,
        "remediation_attempted": false,
        "dashboard_order_controls_enabled": false
    });
    atomic_write_json(output, &body).unwrap();
}

fn write_redacted_exchange_open_orders_readback(output: &Path, include_order: bool) {
    let open_orders = if include_order {
        json!([
            {
                "symbol": "BTCUSDT",
                "order_id": "123456789",
                "client_order_id": "owner-approved-v160-single-shot",
                "exchange_status": "NEW"
            }
        ])
    } else {
        json!([])
    };
    let body = json!({
        "schema_version": PRODUCTION_MUTATION_EXCHANGE_OPEN_ORDERS_READBACK_SCHEMA_VERSION,
        "artifact_type": "redacted_binance_open_orders_readback",
        "status": "ready_redacted_open_orders_readback_metadata",
        "endpoint": "open_orders",
        "method": "GET",
        "path": "/api/v3/openOrders",
        "symbol": "BTCUSDT",
        "open_orders": open_orders,
        "response_redacted": true,
        "api_key_value_recorded": false,
        "api_secret_value_recorded": false,
        "api_key_header_value_recorded": false,
        "signature_recorded": false,
        "signed_query_recorded": false,
        "signed_url_recorded": false,
        "raw_exchange_response_recorded": false,
        "response_body_recorded": false,
        "response_headers_recorded": false,
        "request_sent": false,
        "network_attempted": false,
        "retry_attempted": false,
        "cancel_attempted": false,
        "replace_attempted": false,
        "amend_attempted": false,
        "flatten_attempted": false,
        "remediation_attempted": false,
        "dashboard_order_controls_enabled": false
    });
    atomic_write_json(output, &body).unwrap();
}

struct V170ExchangeReadbackMapperFixture<'a> {
    source_status: &'a str,
    exchange_readback_mapped: bool,
    request_sent: bool,
    exchange_order_status: &'a str,
    exchange_order_state: &'a str,
    order_found: bool,
    open_order_observed: bool,
    terminal_state_observed: bool,
}

fn write_v170_exchange_readback_mapper_fixture(
    output: &Path,
    fixture: &V170ExchangeReadbackMapperFixture<'_>,
) {
    let mut body = json!({
        "schema_version": PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION,
        "run_id": "v170-production-mutation-exchange-readback-mapper",
        "order_lineage_id": "lineage-v160-single-shot",
        "artifact_type": "production_mutation_exchange_readback_mapper",
        "status": fixture.source_status,
        "mode": "single_mutation_candidate_exchange_readback_mapper",
        "capability": "Production Reconciliation And Orphan Recovery Evidence",
        "capability_expansion_from_v16": "reconciliation_evidence_only",
        "lineage_scope": "single_v16_mutation_candidate",
        "default_fail_closed": true,
        "owner_gated_readback_required": true,
        "local_ledger_ready": true,
        "exchange_readback_mapped": fixture.exchange_readback_mapped,
        "reconciliation_classified": false,
        "orphan_risk_detected": false,
        "known_order_id": "123456789",
        "known_client_order_id": "owner-approved-v160-single-shot",
        "symbol": "BTCUSDT",
        "exchange_order_status": fixture.exchange_order_status,
        "exchange_order_state": fixture.exchange_order_state,
        "open_order_observed": fixture.open_order_observed,
        "terminal_state_observed": fixture.terminal_state_observed,
        "order_found": fixture.order_found,
        "open_orders_count": i32::from(fixture.open_order_observed),
        "source_artifact_issues": [],
        "malformed_readback_issues": [],
        "missing_cli_flags": [],
        "manual_review_required": false,
        "new_orders_blocked": false,
        "request_sent": fixture.request_sent
    });
    let object = body.as_object_mut().unwrap();
    for field in [
        "network_attempted",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "production_order_state_reads_allowed",
        "listen_key_lifecycle_allowed",
        "duplicate_submit_attempted",
        "retry_attempted",
        "cancel_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
    ] {
        object.insert(field.to_string(), json!(false));
    }
    atomic_write_json(output, &body).unwrap();
}

struct V170ReconciliationClassifierFixture<'a> {
    reconciliation_outcome: &'a str,
    local_request_sent: bool,
    exchange_order_status: &'a str,
    exchange_order_state: &'a str,
    order_found: bool,
    open_order_observed: bool,
    terminal_state_observed: bool,
    manual_review_required: bool,
    new_orders_blocked: bool,
    restart_readable: bool,
}

fn write_v170_reconciliation_classifier_fixture(
    output: &Path,
    fixture: &V170ReconciliationClassifierFixture<'_>,
) {
    let mut body = json!({
        "schema_version": PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION,
        "run_id": "v170-production-mutation-reconciliation-classifier",
        "order_lineage_id": "lineage-v160-single-shot",
        "artifact_type": "production_mutation_reconciliation_classifier",
        "status": "ready_reconciliation_classified",
        "mode": "single_mutation_candidate_reconciliation_classifier",
        "capability": "Production Reconciliation And Orphan Recovery Evidence",
        "capability_expansion_from_v16": "reconciliation_evidence_only",
        "lineage_scope": "single_v16_mutation_candidate",
        "default_fail_closed": true,
        "owner_gated_readback_required": true,
        "exchange_readback_mapped": true,
        "reconciliation_classified": true,
        "orphan_risk_detected": false,
        "local_request_sent": fixture.local_request_sent,
        "exchange_order_status": fixture.exchange_order_status,
        "exchange_order_state": fixture.exchange_order_state,
        "open_order_observed": fixture.open_order_observed,
        "terminal_state_observed": fixture.terminal_state_observed,
        "order_found": fixture.order_found,
        "reconciliation_outcome": fixture.reconciliation_outcome,
        "source_artifact_issues": [],
        "missing_cli_flags": [],
        "manual_review_required": fixture.manual_review_required,
        "new_orders_blocked": fixture.new_orders_blocked,
        "restart_readable": fixture.restart_readable
    });
    let object = body.as_object_mut().unwrap();
    for field in [
        "network_attempted",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "production_order_state_reads_allowed",
        "listen_key_lifecycle_allowed",
        "duplicate_submit_attempted",
        "retry_attempted",
        "cancel_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
    ] {
        object.insert(field.to_string(), json!(false));
    }
    atomic_write_json(output, &body).unwrap();
}

fn write_v180_cancel_request_preview_source_chain(
    output_dir: &Path,
    mapper_fixture: &V170ExchangeReadbackMapperFixture<'_>,
) -> PathBuf {
    let mapper = output_dir.join("exchange-readback-mapper.json");
    let classifier = output_dir.join("reconciliation-classifier.json");
    let orphan = output_dir.join("orphan-order-detector.json");

    write_v170_exchange_readback_mapper_fixture(&mapper, mapper_fixture);
    run_live_production_mutation_reconciliation_classifier(
        &production_mutation_reconciliation_classifier_opt(mapper, classifier.clone(), true),
    )
    .unwrap();
    run_live_production_mutation_orphan_order_detector(
        &production_mutation_orphan_order_detector_opt(classifier, orphan.clone(), true),
    )
    .unwrap();

    orphan
}

fn write_v180_manual_owner_approval_lifecycle_source_chain(
    output_dir: &Path,
    mapper_fixture: &V170ExchangeReadbackMapperFixture<'_>,
) -> PathBuf {
    let orphan = write_v180_cancel_request_preview_source_chain(output_dir, mapper_fixture);
    let preview = output_dir.join("cancel-request-preview.json");
    let risk_gate = output_dir.join("cancel-risk-gate.json");

    run_live_production_mutation_cancel_request_preview(
        &production_mutation_cancel_request_preview_opt(orphan, preview.clone(), true),
    )
    .unwrap();
    run_live_production_mutation_cancel_risk_gate(&production_mutation_cancel_risk_gate_opt(
        preview,
        risk_gate.clone(),
        "BTCUSDT",
        "prod-account-redacted",
        true,
    ))
    .unwrap();

    risk_gate
}

fn write_ready_v180_manual_owner_approval_lifecycle_artifact(
    output_dir: &Path,
    mapper_fixture: &V170ExchangeReadbackMapperFixture<'_>,
) -> PathBuf {
    let risk_gate =
        write_v180_manual_owner_approval_lifecycle_source_chain(output_dir, mapper_fixture);
    let approval = output_dir.join("manual-owner-approval-lifecycle.json");
    run_live_production_mutation_manual_owner_approval_lifecycle(
        &production_mutation_manual_owner_approval_lifecycle_opt(
            risk_gate,
            approval.clone(),
            "approved",
            true,
        ),
    )
    .unwrap();
    approval
}

fn write_ready_v180_cancel_response_redaction_artifact(
    output_dir: &Path,
    mapper_fixture: &V170ExchangeReadbackMapperFixture<'_>,
) -> PathBuf {
    let approval =
        write_ready_v180_manual_owner_approval_lifecycle_artifact(output_dir, mapper_fixture);
    let response = output_dir.join("synthetic-cancel-response.json");
    let redaction = output_dir.join("cancel-response-redaction.json");
    write_synthetic_production_mutation_cancel_response(&response);
    run_live_production_mutation_cancel_response_redaction(
        &production_mutation_cancel_response_redaction_opt(
            approval,
            response,
            redaction.clone(),
            true,
        ),
    )
    .unwrap();
    redaction
}

fn write_ready_v180_cancel_recovery_closeout_sources(
    output_dir: &Path,
    mapper_fixture: &V170ExchangeReadbackMapperFixture<'_>,
    readback_state: &str,
) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let risk_gate =
        write_v180_manual_owner_approval_lifecycle_source_chain(output_dir, mapper_fixture);
    let approval = output_dir.join("manual-owner-approval-lifecycle.json");
    run_live_production_mutation_manual_owner_approval_lifecycle(
        &production_mutation_manual_owner_approval_lifecycle_opt(
            risk_gate.clone(),
            approval.clone(),
            "approved",
            true,
        ),
    )
    .unwrap();

    let response = output_dir.join("synthetic-cancel-response.json");
    let redaction = output_dir.join("cancel-response-redaction.json");
    write_synthetic_production_mutation_cancel_response(&response);
    run_live_production_mutation_cancel_response_redaction(
        &production_mutation_cancel_response_redaction_opt(
            approval.clone(),
            response,
            redaction.clone(),
            true,
        ),
    )
    .unwrap();

    let readback = output_dir.join(format!("post-cancel-readback-{readback_state}.json"));
    let post_cancel_readback = output_dir.join("post-cancel-readback-artifact.json");
    write_synthetic_production_mutation_post_cancel_readback(&readback, readback_state);
    run_live_production_mutation_post_cancel_readback(
        &production_mutation_post_cancel_readback_opt(
            redaction.clone(),
            readback,
            post_cancel_readback.clone(),
            true,
        ),
    )
    .unwrap();

    (risk_gate, approval, redaction, post_cancel_readback)
}

fn passing_live_alpha_risk_input() -> ProductionLiveAlphaRiskPreflightInput {
    ProductionLiveAlphaRiskPreflightInput {
        schema_version: PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_INPUT_SCHEMA_VERSION.to_string(),
        session: ProductionLiveAlphaRiskPreflightSession {
            state: "running".to_string(),
        },
        market: ProductionLiveAlphaRiskPreflightMarket {
            symbol: "BTCUSDT".to_string(),
            last_event_at_unix_ms: 1_000,
            now_unix_ms: 1_500,
            max_age_ms: 1_000,
        },
        account: ProductionLiveAlphaRiskPreflightAccount {
            readable: true,
            account_id: "BINANCE-001".to_string(),
        },
        order_state: ProductionLiveAlphaRiskPreflightOrderState {
            readable: true,
            open_order_count: 0,
            last_read_at_unix_ms: None,
            now_unix_ms: None,
            max_age_ms: None,
        },
        risk: ProductionLiveAlphaRiskPreflightRisk {
            kill_switch_active: false,
            allowed_symbols: vec!["BTCUSDT".to_string()],
        },
        order: ProductionLiveAlphaRiskPreflightOrder {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: "0.001".to_string(),
            notional: "10.00".to_string(),
        },
        limits: ProductionLiveAlphaRiskPreflightLimits {
            max_order_notional: "25.00".to_string(),
            current_position_notional: "50.00".to_string(),
            max_position_notional: "100.00".to_string(),
            max_open_orders: 5,
            max_clock_skew_ms: 100,
            observed_clock_skew_ms: 25,
        },
    }
}

fn write_live_alpha_risk_input(path: &Path, input: &ProductionLiveAlphaRiskPreflightInput) {
    fs::write(path, serde_json::to_string_pretty(input).unwrap()).unwrap();
}

fn write_redacted_account_snapshot_report(path: &Path, response_shape_validated: bool) {
    let shape_summary = if response_shape_validated {
        serde_json::json!({
            "status": "accepted",
            "balance_entry_count": 2,
            "shape_validated": true,
            "raw_account_response_recorded": false,
            "raw_balances_recorded": false,
            "raw_permissions_recorded": false
        })
    } else {
        serde_json::json!({
            "status": "not_attempted",
            "balance_entry_count": null,
            "shape_validated": false,
            "raw_account_response_recorded": false,
            "raw_balances_recorded": false,
            "raw_permissions_recorded": false
        })
    };
    fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION,
            "status": if response_shape_validated { "online_account_snapshot_ok" } else { "ready_offline_contract" },
            "response_shape_validated": response_shape_validated,
            "response_shape_summary": shape_summary,
            "network_attempted": response_shape_validated,
            "account_read_attempted": response_shape_validated,
            "api_key_value_recorded": false,
            "api_secret_value_recorded": false,
            "signature_recorded": false,
            "signed_query_recorded": false,
            "signed_url_recorded": false,
            "production_order_submission_attempted": false,
            "production_order_mutation_attempted": false,
            "dashboard_order_controls_enabled": false,
            "secrets_redacted": true
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_shadow_intent(path: &Path, actual_submission: bool) {
    write_shadow_intent_with_notional(path, actual_submission, "10.00");
}

fn write_shadow_intent_with_notional(path: &Path, actual_submission: bool, notional: &str) {
    fs::write(
        path,
        format!(
            r#"{{"schema_version":"ntpro.v110_shadow_execution_intent.v1","run_id":"v120-shadow","intent_id":"intent-1","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","venue":"BINANCE","side":"buy","order_type":"market","quantity":"0.001","notional":"{notional}","mode":"production_shadow","submission_allowed":false,"actual_submission":{actual_submission},"submission_status":"blocked_by_v110_shadow_execution_boundary","execution_adapter_called":false,"order_endpoint_access_attempted":false,"production_order_mutation_attempted":false,"dashboard_order_controls_enabled":false}}
"#
        ),
    )
    .unwrap();
}

fn read_jsonl_values(path: &Path) -> Vec<serde_json::Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn synthetic_order_credentials() -> EnvOnlyTestnetOrderCredentials {
    EnvOnlyTestnetOrderCredentials::from_values(
        "NTPRO_V100004_API_KEY".to_string(),
        Some("ntpro_v100004_synthetic_api_key_value".to_string()),
        "NTPRO_V100004_API_SECRET".to_string(),
        Some("ntpro_v100004_synthetic_api_secret_value".to_string()),
    )
}

fn passing_preflight_input() -> StrategyOrderPreflightInput {
    StrategyOrderPreflightInput {
        schema_version: STRATEGY_ORDER_PREFLIGHT_SCHEMA_VERSION.to_string(),
        session: StrategyOrderPreflightSession {
            state: "running".to_string(),
        },
        market: StrategyOrderPreflightMarket {
            symbol: "BTCUSDT.BINANCE".to_string(),
            last_event_at_unix_ms: 1_000,
            now_unix_ms: 1_500,
            max_age_ms: 1_000,
        },
        account: StrategyOrderPreflightAccount {
            readable: true,
            account_id: "BINANCE_TESTNET-001".to_string(),
        },
        risk: StrategyOrderPreflightRisk {
            kill_switch_active: false,
            allowed_symbols: vec!["BTCUSDT.BINANCE".to_string()],
        },
        limits: StrategyOrderPreflightLimits {
            max_order_notional: "1.00".to_string(),
            max_open_orders: 1,
            open_order_count: 0,
            max_clock_skew_ms: 100,
            observed_clock_skew_ms: 25,
        },
        endpoint: StrategyOrderPreflightEndpoint {
            http_base_url: BINANCE_TESTNET_HTTP_BASE_URL.to_string(),
            production_endpoint_allowed: false,
        },
    }
}

fn write_preflight_input(name: &str, input: &StrategyOrderPreflightInput) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ntpro-{name}-preflight-input-{}.json",
        std::process::id()
    ));
    fs::write(path.clone(), serde_json::to_string_pretty(input).unwrap()).unwrap();
    path
}

#[test]
fn testnet_order_gate_blocks_missing_cli_and_env_gates() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-002-gate-blocked-{}",
        std::process::id()
    ));
    let path = write_config(
        "testnet-order-gate-blocked",
        &strategy_node_config(&output_dir),
    );
    let opt = testnet_order_gate_opt(path, false);

    let error = run_live_testnet_order_gate_with_env(&opt, |_| None)
        .unwrap_err()
        .to_string();

    assert!(error.contains("testnet order gate blocked"));
    assert!(error.contains("--allow-testnet-order"));
    assert!(error.contains("--confirm-owner-approved-testnet-order"));
    assert!(error.contains("NTPRO_ALLOW_BINANCE_TESTNET_ORDER"));
    assert!(error.contains("NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER"));
    assert!(error.contains("order_submission_remains_disabled=true"));
    assert!(error.contains("network_attempted=false"));
    assert!(error.contains("real_orders_submitted=false"));
}

#[test]
fn testnet_order_gate_accepts_all_manual_gates_without_network() {
    let output_dir =
        std::env::temp_dir().join(format!("ntpro-v100-002-gate-ready-{}", std::process::id()));
    let path = write_config(
        "testnet-order-gate-ready",
        &strategy_node_config(&output_dir),
    );
    let opt = testnet_order_gate_opt(path, true);

    run_live_testnet_order_gate_with_env(&opt, |_| Some("1".to_string())).unwrap();
}

#[test]
fn testnet_order_preflight_passes_with_ready_snapshot() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-003-preflight-pass-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-preflight-pass",
        &strategy_node_config(&output_dir),
    );
    let input = write_preflight_input("v100-003-pass", &passing_preflight_input());
    let report = output_dir.join("preflight-report.json");
    let opt = testnet_order_preflight_opt(config, input, Some(report.clone()), true);

    run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string())).unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report).unwrap()).unwrap();
    assert_eq!(report["status"], "pass");
    assert_eq!(report["order_submission_remains_disabled"], true);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["real_orders_submitted"], false);
}

#[test]
fn testnet_order_preflight_blocks_missing_manual_gates() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-003-preflight-missing-gates-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-preflight-missing-gates",
        &strategy_node_config(&output_dir),
    );
    let input = write_preflight_input("v100-003-missing-gates", &passing_preflight_input());
    let opt = testnet_order_preflight_opt(config, input, None, false);

    let error = run_live_testnet_order_preflight_with_env(&opt, |_| None)
        .unwrap_err()
        .to_string();

    assert!(error.contains("testnet order preflight blocked"));
    assert!(error.contains("--allow-testnet-order"));
    assert!(error.contains("NTPRO_ALLOW_BINANCE_TESTNET_ORDER"));
    assert!(error.contains("preflight_evaluated=false"));
    assert!(error.contains("network_attempted=false"));
    assert!(error.contains("real_orders_submitted=false"));
}

#[test]
fn testnet_order_preflight_rejects_stale_market() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-003-preflight-stale-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-preflight-stale",
        &strategy_node_config(&output_dir),
    );
    let mut input = passing_preflight_input();
    input.market.now_unix_ms = 3_000;
    input.market.max_age_ms = 100;
    let input = write_preflight_input("v100-003-stale", &input);
    let opt = testnet_order_preflight_opt(config, input, None, true);

    let error = run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string()))
        .unwrap_err()
        .to_string();

    assert!(error.contains("market_stale"));
    assert!(error.contains("network_attempted=false"));
    assert!(error.contains("real_orders_submitted=false"));
}

#[test]
fn testnet_order_preflight_rejects_kill_switch_active() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-003-preflight-kill-switch-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-preflight-kill-switch",
        &strategy_node_config(&output_dir),
    );
    let mut input = passing_preflight_input();
    input.risk.kill_switch_active = true;
    let input = write_preflight_input("v100-003-kill-switch", &input);
    let opt = testnet_order_preflight_opt(config, input, None, true);

    let error = run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string()))
        .unwrap_err()
        .to_string();

    assert!(error.contains("kill_switch_active"));
    assert!(error.contains("real_orders_submitted=false"));
}

#[test]
fn testnet_order_preflight_rejects_production_endpoint() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-003-preflight-production-endpoint-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-preflight-production-endpoint",
        &strategy_node_config(&output_dir),
    );
    let mut input = passing_preflight_input();
    input.endpoint.http_base_url = "https://api.binance.com".to_string();
    input.endpoint.production_endpoint_allowed = true;
    let input = write_preflight_input("v100-003-production-endpoint", &input);
    let opt = testnet_order_preflight_opt(config, input, None, true);

    let error = run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string()))
        .unwrap_err()
        .to_string();

    assert!(error.contains("endpoint_not_testnet"));
    assert!(error.contains("production_endpoint_allowed"));
    assert!(error.contains("network_attempted=false"));
}

#[test]
fn testnet_order_preflight_rejects_limit_violations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-003-preflight-limit-violations-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-preflight-limit-violations",
        &strategy_node_config(&output_dir),
    );
    let mut input = passing_preflight_input();
    input.limits.max_order_notional = "0.00000001".to_string();
    input.limits.max_open_orders = 1;
    input.limits.open_order_count = 1;
    input.limits.max_clock_skew_ms = 10;
    input.limits.observed_clock_skew_ms = 25;
    let input = write_preflight_input("v100-003-limit-violations", &input);
    let opt = testnet_order_preflight_opt(config, input, None, true);

    let error = run_live_testnet_order_preflight_with_env(&opt, |_| Some("1".to_string()))
        .unwrap_err()
        .to_string();

    assert!(error.contains("notional_limit_exceeded"));
    assert!(error.contains("open_order_limit_exceeded"));
    assert!(error.contains("clock_skew_limit_exceeded"));
    assert!(error.contains("real_orders_submitted=false"));
}

#[test]
fn testnet_signed_order_request_builder_constructs_order_test_preview() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-004-request-preview-{}",
        std::process::id()
    ));
    let path = write_config(
        "testnet-order-request-preview",
        &strategy_node_config(&output_dir),
    );
    let config = load_strategy_node_config(&path).unwrap();
    let testnet_order = config.testnet_order.as_ref().unwrap();
    let credentials = synthetic_order_credentials();

    let request = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_POST,
        TESTNET_ORDER_ENDPOINT_TEST,
        1_718_400_000_000,
        5_000,
        None,
    )
    .unwrap();

    assert_eq!(request.method, TESTNET_ORDER_METHOD_POST);
    assert_eq!(request.endpoint_path, TESTNET_ORDER_ENDPOINT_TEST);
    assert_eq!(request.action, "order_test");
    assert_eq!(request.api_key_header_name, BINANCE_API_KEY_HEADER);
    assert_eq!(
        request.api_key_header_value,
        "ntpro_v100004_synthetic_api_key_value"
    );
    assert!(request.query_without_signature.contains("symbol=BTCUSDT"));
    assert!(request.query_without_signature.contains("side=BUY"));
    assert!(request.query_without_signature.contains("type=LIMIT"));
    assert!(request.query_without_signature.contains("timeInForce=GTC"));
    assert!(
        request
            .signed_query
            .starts_with("symbol=BTCUSDT&side=BUY&type=LIMIT")
    );
    assert_eq!(request.signature.len(), 64);
    assert!(
        request
            .signature
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    );
    assert_eq!(
        request.endpoint_url_redacted,
        "https://testnet.binance.vision/api/v3/order/test"
    );
    request.ensure_preview_redacted(&credentials).unwrap();
}

#[test]
fn testnet_signed_order_request_preview_redacts_all_sensitive_values() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-004-request-redaction-{}",
        std::process::id()
    ));
    let path = write_config(
        "testnet-order-request-redaction",
        &strategy_node_config(&output_dir),
    );
    let config = load_strategy_node_config(&path).unwrap();
    let testnet_order = config.testnet_order.as_ref().unwrap();
    let credentials = synthetic_order_credentials();
    let request = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_POST,
        TESTNET_ORDER_ENDPOINT_ORDER,
        1_718_400_000_000,
        5_000,
        None,
    )
    .unwrap();
    let preview_body = serde_json::to_string(&request.redacted_preview(&credentials)).unwrap();
    let debug_body = format!("{request:?}");

    for body in [&preview_body, &debug_body] {
        assert!(!body.contains("ntpro_v100004_synthetic_api_key_value"));
        assert!(!body.contains("ntpro_v100004_synthetic_api_secret_value"));
        assert!(!body.contains(&request.signature));
        assert!(!body.contains(&request.signed_query));
    }
    assert!(preview_body.contains("\"order_submission_remains_disabled\":true"));
    assert!(preview_body.contains("\"network_attempted\":false"));
    assert!(preview_body.contains("\"real_orders_submitted\":false"));
    assert!(preview_body.contains("\"signature_recorded\":false"));
    assert!(preview_body.contains("\"signed_query_recorded\":false"));
    assert!(preview_body.contains("\"signed_url_recorded\":false"));
    assert!(preview_body.contains("\"api_key_header_value_recorded\":false"));
}

#[test]
fn testnet_signed_order_request_builder_rejects_non_allowlisted_endpoint() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-004-request-bad-endpoint-{}",
        std::process::id()
    ));
    let path = write_config(
        "testnet-order-request-bad-endpoint",
        &strategy_node_config(&output_dir),
    );
    let config = load_strategy_node_config(&path).unwrap();
    let testnet_order = config.testnet_order.as_ref().unwrap();
    let credentials = synthetic_order_credentials();

    let error = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_POST,
        "/api/v3/account",
        1_718_400_000_000,
        5_000,
        None,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("signed order request allowlist only includes"));
    assert!(error.contains("POST /api/v3/account"));
}

#[test]
fn testnet_signed_order_request_builder_rejects_production_base_url() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-004-request-production-base-{}",
        std::process::id()
    ));
    let path = write_config(
        "testnet-order-request-production-base",
        &strategy_node_config(&output_dir),
    );
    let mut config = load_strategy_node_config(&path).unwrap();
    let testnet_order = config.testnet_order.as_mut().unwrap();
    testnet_order.http_base_url = "https://api.binance.com".to_string();
    let credentials = synthetic_order_credentials();

    let error = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_POST,
        TESTNET_ORDER_ENDPOINT_TEST,
        1_718_400_000_000,
        5_000,
        None,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("testnet_order.http_base_url"));
    assert!(error.contains(BINANCE_TESTNET_HTTP_BASE_URL));
}

#[test]
fn testnet_signed_order_request_builder_fails_closed_without_secret() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-004-request-missing-secret-{}",
        std::process::id()
    ));
    let path = write_config(
        "testnet-order-request-missing-secret",
        &strategy_node_config(&output_dir),
    );
    let config = load_strategy_node_config(&path).unwrap();
    let testnet_order = config.testnet_order.as_ref().unwrap();
    let credentials = EnvOnlyTestnetOrderCredentials::from_values(
        "NTPRO_V100004_API_KEY".to_string(),
        Some("ntpro_v100004_synthetic_api_key_value".to_string()),
        "NTPRO_V100004_API_SECRET".to_string(),
        None,
    );

    let error = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_POST,
        TESTNET_ORDER_ENDPOINT_TEST,
        1_718_400_000_000,
        5_000,
        None,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("requires API secret env value"));
}

#[test]
fn testnet_signed_order_request_builder_requires_cancel_client_order_id() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-004-request-cancel-missing-id-{}",
        std::process::id()
    ));
    let path = write_config(
        "testnet-order-request-cancel-missing-id",
        &strategy_node_config(&output_dir),
    );
    let config = load_strategy_node_config(&path).unwrap();
    let testnet_order = config.testnet_order.as_ref().unwrap();
    let credentials = synthetic_order_credentials();

    let error = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_DELETE,
        TESTNET_ORDER_ENDPOINT_ORDER,
        1_718_400_000_000,
        5_000,
        None,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("requires --orig-client-order-id"));
}

#[test]
fn testnet_signed_order_request_builder_constructs_cancel_preview() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-004-request-cancel-{}",
        std::process::id()
    ));
    let path = write_config(
        "testnet-order-request-cancel",
        &strategy_node_config(&output_dir),
    );
    let config = load_strategy_node_config(&path).unwrap();
    let testnet_order = config.testnet_order.as_ref().unwrap();
    let credentials = synthetic_order_credentials();

    let request = build_testnet_signed_order_request(
        testnet_order,
        &credentials,
        TESTNET_ORDER_METHOD_DELETE,
        TESTNET_ORDER_ENDPOINT_ORDER,
        1_718_400_000_000,
        5_000,
        Some("ntpro-cancel-001"),
    )
    .unwrap();

    assert_eq!(request.method, TESTNET_ORDER_METHOD_DELETE);
    assert_eq!(request.endpoint_path, TESTNET_ORDER_ENDPOINT_ORDER);
    assert_eq!(request.action, "cancel");
    assert!(request.query_without_signature.contains("symbol=BTCUSDT"));
    assert!(
        request
            .query_without_signature
            .contains("origClientOrderId=ntpro-cancel-001")
    );
    assert!(
        !request
            .query_without_signature
            .contains("newOrderRespType=ACK")
    );
    request.ensure_preview_redacted(&credentials).unwrap();
}

#[test]
fn testnet_signed_order_request_preview_command_writes_redacted_artifact() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-004-request-command-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-request-command",
        &strategy_node_config(&output_dir),
    );
    let output = output_dir.join("request-preview.json");
    let opt = testnet_order_request_preview_opt(config, Some(output.clone()), true);

    run_live_testnet_order_request_preview_with_env(&opt, |name| match name {
        TESTNET_ORDER_ENV_ALLOW
        | TESTNET_ORDER_ENV_OWNER_APPROVED
        | TESTNET_ORDER_ENV_TINY_NOTIONAL
        | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
        "NTPRO_V100004_API_KEY" => Some("ntpro_v100004_synthetic_api_key_value".to_string()),
        "NTPRO_V100004_API_SECRET" => Some("ntpro_v100004_synthetic_api_secret_value".to_string()),
        _ => None,
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(body.contains(TESTNET_ORDER_PREVIEW_SCHEMA_VERSION));
    assert!(body.contains("\"order_action\": \"order_test\""));
    assert!(body.contains("\"network_attempted\": false"));
    assert!(body.contains("\"real_orders_submitted\": false"));
    assert!(!body.contains("ntpro_v100004_synthetic_api_key_value"));
    assert!(!body.contains("ntpro_v100004_synthetic_api_secret_value"));
}

#[test]
fn testnet_order_test_preflight_command_writes_redacted_report() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-005-order-test-preflight-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-test-preflight",
        &strategy_node_config(&output_dir),
    );
    let output = output_dir.join("order-test-preflight.json");
    let opt = testnet_order_test_preflight_opt(config, Some(output.clone()), true);

    run_live_testnet_order_test_preflight_with_env(&opt, |name| match name {
        TESTNET_ORDER_ENV_ALLOW
        | TESTNET_ORDER_ENV_OWNER_APPROVED
        | TESTNET_ORDER_ENV_TINY_NOTIONAL
        | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
        "NTPRO_V100005_API_KEY" => Some("ntpro_v100005_synthetic_api_key_value".to_string()),
        "NTPRO_V100005_API_SECRET" => Some("ntpro_v100005_synthetic_api_secret_value".to_string()),
        _ => None,
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(body.contains("ntpro.v100_order_test_preflight_report.v1"));
    assert!(body.contains("\"status\": \"ready\""));
    assert!(body.contains("\"request_method\": \"POST\""));
    assert!(body.contains("\"request_target\": \"/api/v3/order/test\""));
    assert!(
        body.contains("\"binance_order_test_acceptance\": \"not_attempted_offline_manual_only\"")
    );
    assert!(body.contains("\"matching_engine_submission\": false"));
    assert!(body.contains("\"network_attempted\": false"));
    assert!(body.contains("\"real_orders_submitted\": false"));
    assert!(!body.contains("ntpro_v100005_synthetic_api_key_value"));
    assert!(!body.contains("ntpro_v100005_synthetic_api_secret_value"));
}

#[test]
fn testnet_order_test_preflight_blocks_missing_manual_gates() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-005-order-test-missing-gates-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-test-missing-gates",
        &strategy_node_config(&output_dir),
    );
    let opt = testnet_order_test_preflight_opt(config, None, false);

    let error = run_live_testnet_order_test_preflight_with_env(&opt, |_| None)
        .unwrap_err()
        .to_string();

    assert!(error.contains("testnet order-test preflight blocked"));
    assert!(error.contains("--allow-testnet-order"));
    assert!(error.contains("matching_engine_submission=false"));
    assert!(error.contains("network_attempted=false"));
    assert!(error.contains("real_orders_submitted=false"));
}

#[test]
fn testnet_order_test_preflight_fails_closed_without_secret() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-005-order-test-missing-secret-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-order-test-missing-secret",
        &strategy_node_config(&output_dir),
    );
    let opt = testnet_order_test_preflight_opt(config, None, true);

    let error = run_live_testnet_order_test_preflight_with_env(&opt, |name| match name {
        TESTNET_ORDER_ENV_ALLOW
        | TESTNET_ORDER_ENV_OWNER_APPROVED
        | TESTNET_ORDER_ENV_TINY_NOTIONAL
        | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
        "NTPRO_V100005_API_KEY" => Some("ntpro_v100005_synthetic_api_key_value".to_string()),
        _ => None,
    })
    .unwrap_err()
    .to_string();

    assert!(error.contains("requires API secret env value"));
}

#[test]
fn testnet_order_test_preflight_rejects_production_base_url() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-005-order-test-production-base-{}",
        std::process::id()
    ));
    let config = strategy_node_config(&output_dir).replace(
        r#"http_base_url = "https://testnet.binance.vision""#,
        r#"http_base_url = "https://api.binance.com""#,
    );
    let config = write_config("testnet-order-test-production-base", &config);
    let opt = testnet_order_test_preflight_opt(config, None, true);

    let error = run_live_testnet_order_test_preflight_with_env(&opt, |name| match name {
        TESTNET_ORDER_ENV_ALLOW
        | TESTNET_ORDER_ENV_OWNER_APPROVED
        | TESTNET_ORDER_ENV_TINY_NOTIONAL
        | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT
        | "NTPRO_V100005_API_KEY"
        | "NTPRO_V100005_API_SECRET" => Some("1".to_string()),
        _ => None,
    })
    .unwrap_err()
    .to_string();

    assert!(error.contains("testnet_order.http_base_url"));
    assert!(error.contains(BINANCE_TESTNET_HTTP_BASE_URL));
}

#[test]
fn testnet_execution_artifact_contract_writes_redacted_report() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-007-artifact-contract-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-execution-artifact-contract",
        &strategy_node_config(&output_dir),
    );
    let output = output_dir.join("execution-artifact-contract.json");
    let opt = testnet_execution_artifact_contract_opt(config, Some(output.clone()), true);

    run_live_testnet_execution_artifact_contract_with_env(&opt, |name| match name {
        TESTNET_ORDER_ENV_ALLOW
        | TESTNET_ORDER_ENV_OWNER_APPROVED
        | TESTNET_ORDER_ENV_TINY_NOTIONAL
        | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
        "NTPRO_V100007_API_KEY" => Some("ntpro_v100007_synthetic_api_key_value".to_string()),
        "NTPRO_V100007_API_SECRET" => Some("ntpro_v100007_synthetic_api_secret_value".to_string()),
        _ => None,
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(body.contains(TESTNET_EXECUTION_ARTIFACT_SCHEMA_VERSION));
    assert!(body.contains("\"artifact_family\": \"binance-testnet-order-lifecycle-proof\""));
    assert!(body.contains("\"name\": \"request.json\""));
    assert!(body.contains("\"name\": \"submit_ack.json\""));
    assert!(body.contains("\"name\": \"cancel_ack.json\""));
    assert!(body.contains("\"name\": \"lifecycle.json\""));
    assert!(body.contains("\"name\": \"reconciliation.json\""));
    assert!(body.contains("\"testnet_orders_submitted\": 0"));
    assert!(body.contains("\"testnet_orders_canceled\": 0"));
    assert!(body.contains("\"production_orders_submitted\": 0"));
    assert!(body.contains("\"production_orders_canceled\": 0"));
    assert!(body.contains("\"manual_submit_cancel_proof_observed\": false"));
    assert!(body.contains("\"network_attempted\": false"));
    assert!(body.contains("\"real_orders_submitted\": false"));
    assert!(!body.contains("ntpro_v100007_synthetic_api_key_value"));
    assert!(!body.contains("ntpro_v100007_synthetic_api_secret_value"));
}

#[test]
fn testnet_execution_artifact_contract_blocks_missing_manual_gates() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-007-artifact-missing-gates-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-execution-artifact-missing-gates",
        &strategy_node_config(&output_dir),
    );
    let opt = testnet_execution_artifact_contract_opt(config, None, false);

    let error = run_live_testnet_execution_artifact_contract_with_env(&opt, |_| None)
        .unwrap_err()
        .to_string();

    assert!(error.contains("testnet execution artifact contract blocked"));
    assert!(error.contains("--allow-testnet-order"));
    assert!(error.contains("artifact_built=false"));
    assert!(error.contains("network_attempted=false"));
    assert!(error.contains("real_orders_submitted=false"));
}

#[test]
fn testnet_execution_artifact_contract_fails_closed_without_secret() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-007-artifact-missing-secret-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-execution-artifact-missing-secret",
        &strategy_node_config(&output_dir),
    );
    let opt = testnet_execution_artifact_contract_opt(config, None, true);

    let error = run_live_testnet_execution_artifact_contract_with_env(&opt, |name| match name {
        TESTNET_ORDER_ENV_ALLOW
        | TESTNET_ORDER_ENV_OWNER_APPROVED
        | TESTNET_ORDER_ENV_TINY_NOTIONAL
        | TESTNET_ORDER_ENV_CANCEL_AFTER_SUBMIT => Some("1".to_string()),
        "NTPRO_V100007_API_KEY" => Some("ntpro_v100007_synthetic_api_key_value".to_string()),
        _ => None,
    })
    .unwrap_err()
    .to_string();

    assert!(error.contains("requires API secret env value"));
}

#[test]
fn testnet_reconciliation_fixture_writes_all_risk_halt_scenarios() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-008-reconciliation-all-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-reconciliation-all",
        &strategy_node_config(&output_dir),
    );
    let output = output_dir.join("reconciliation-fixture.json");

    run_live_testnet_reconciliation_fixture(&LiveTestnetReconciliationFixtureOpt {
        config,
        scenario: TestnetReconciliationScenario::All,
        output: Some(output.clone()),
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(body.contains(TESTNET_RECONCILIATION_FIXTURE_SCHEMA_VERSION));
    assert!(body.contains("\"status\": \"risk_halted\""));
    assert!(body.contains("\"scenario_count\": 4"));
    assert!(body.contains("\"name\": \"submit_without_local_ack\""));
    assert!(body.contains("\"name\": \"cancel_timeout\""));
    assert!(body.contains("\"name\": \"local_open_exchange_filled\""));
    assert!(body.contains("\"name\": \"restart_unfinished_order\""));
    assert!(body.contains("\"risk_halted\": true"));
    assert!(body.contains("\"new_orders_blocked\": true"));
    assert!(body.contains("\"testnet_orders_submitted\": 0"));
    assert!(body.contains("\"production_orders_submitted\": 0"));
    assert!(body.contains("\"network_attempted\": false"));
    assert!(body.contains("\"real_orders_submitted\": false"));
}

#[test]
fn testnet_reconciliation_fixture_filters_single_scenario() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v100-008-reconciliation-one-{}",
        std::process::id()
    ));
    let config = write_config(
        "testnet-reconciliation-one",
        &strategy_node_config(&output_dir),
    );
    let output = output_dir.join("reconciliation-fixture.json");

    run_live_testnet_reconciliation_fixture(&LiveTestnetReconciliationFixtureOpt {
        config,
        scenario: TestnetReconciliationScenario::CancelTimeout,
        output: Some(output.clone()),
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(body.contains("\"scenario\": \"cancel_timeout\""));
    assert!(body.contains("\"scenario_count\": 1"));
    assert!(body.contains("\"name\": \"cancel_timeout\""));
    assert!(!body.contains("\"name\": \"submit_without_local_ack\""));
}

#[test]
fn production_public_read_probe_blocks_missing_gates_without_network() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v110-002-public-read-blocked-{}",
        std::process::id()
    ));
    let output = output_dir.join("public-read-probe.json");
    let opt = production_public_read_probe_opt(
        ProductionPublicReadEndpoint::ServerTime,
        Some(output.clone()),
        false,
        false,
    );

    run_live_production_public_read_probe_with_env(&opt, |_| None).unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        report["schema_version"],
        PRODUCTION_PUBLIC_READ_PROBE_SCHEMA_VERSION
    );
    assert_eq!(report["status"], "blocked_missing_gate");
    assert_eq!(report["endpoint_class"], "production_public_read_only");
    assert_eq!(report["method"], "GET");
    assert_eq!(report["path"], "/api/v3/time");
    assert_eq!(report["requires_api_key"], false);
    assert_eq!(report["requires_signature"], false);
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["mutation_allowed"], false);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["production_public_online_read_attempted"], false);
    assert_eq!(report["response_status_code"], serde_json::Value::Null);
    assert_eq!(report["response_shape"], "binance_server_time_v1");
    assert_eq!(report["response_shape_validated"], false);
    assert_eq!(report["latency_ms"], serde_json::Value::Null);
    assert_eq!(report["error_code"], "not_attempted");
    assert_eq!(report["credentials_used"], false);
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_public_read_probe_writes_ready_offline_contract() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v110-002-public-read-ready-{}",
        std::process::id()
    ));
    let output = output_dir.join("public-read-probe.json");
    let opt = production_public_read_probe_opt(
        ProductionPublicReadEndpoint::ExchangeInfo,
        Some(output.clone()),
        true,
        false,
    );

    run_live_production_public_read_probe_with_env(&opt, |_| Some("1".to_string())).unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(report["status"], "ready_offline_contract");
    assert_eq!(report["endpoint"], "exchange_info");
    assert_eq!(report["path"], "/api/v3/exchangeInfo");
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], true);
    assert_eq!(report["read_allowed"], true);
    assert_eq!(report["contract_ready"], true);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["mutation_allowed"], false);
    assert_eq!(report["online_execution_supported"], false);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["production_public_online_read_attempted"], false);
    assert_eq!(report["response_status_code"], serde_json::Value::Null);
    assert_eq!(report["response_shape"], "binance_exchange_info_v1");
    assert_eq!(report["response_shape_validated"], false);
    assert_eq!(report["latency_ms"], serde_json::Value::Null);
    assert_eq!(report["error_code"], "not_attempted");
    assert_eq!(report["credentials_used"], false);
    assert_eq!(report["account_mutation_attempted"], false);
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_public_read_probe_blocks_manual_online_without_v12_owner_gate() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-001-public-read-online-blocked-{}",
        std::process::id()
    ));
    let output = output_dir.join("public-read-probe.json");
    let opt = production_public_read_probe_opt(
        ProductionPublicReadEndpoint::ServerTime,
        Some(output.clone()),
        true,
        true,
    );
    let mut http_called = false;
    let mut read_env = |name: &str| match name {
        PRODUCTION_PUBLIC_READ_ENV_MANUAL_ONLINE => None,
        _ => Some("1".to_string()),
    };

    run_live_production_public_read_probe_with_env_and_http(
        &opt,
        &mut read_env,
        |endpoint, _url| {
            http_called = true;
            ProductionPublicReadProbeHttpResult::success(endpoint, 1, 200)
        },
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert!(!http_called);
    assert_eq!(
        report["schema_version"],
        PRODUCTION_PUBLIC_ONLINE_READ_PROBE_SCHEMA_VERSION
    );
    assert_eq!(report["status"], "blocked_missing_manual_online_gate");
    assert_eq!(report["manual_online_requested"], true);
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["online_execution_supported"], true);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["production_public_online_read_attempted"], false);
    assert_eq!(report["response_status_code"], serde_json::Value::Null);
    assert_eq!(report["response_shape"], "binance_server_time_v1");
    assert_eq!(report["response_shape_validated"], false);
    assert_eq!(report["error_code"], "not_attempted");
    assert_eq!(report["production_order_mutation_attempted"], false);
}

#[test]
fn production_public_read_probe_records_owner_gated_online_success_without_credentials() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-001-public-read-online-success-{}",
        std::process::id()
    ));
    let output = output_dir.join("public-read-probe.json");
    let opt = production_public_read_probe_opt(
        ProductionPublicReadEndpoint::ServerTime,
        Some(output.clone()),
        true,
        true,
    );
    let mut read_env = all_env_enabled;

    run_live_production_public_read_probe_with_env_and_http(
        &opt,
        &mut read_env,
        |endpoint, url| {
            assert_eq!(endpoint, ProductionPublicReadEndpoint::ServerTime);
            assert_eq!(url, "https://api.binance.com/api/v3/time");
            ProductionPublicReadProbeHttpResult::success(endpoint, 42, 200)
        },
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        report["schema_version"],
        PRODUCTION_PUBLIC_ONLINE_READ_PROBE_SCHEMA_VERSION
    );
    assert_eq!(report["status"], "online_read_probe_ok");
    assert_eq!(report["endpoint"], "server_time");
    assert_eq!(report["method"], "GET");
    assert_eq!(report["path"], "/api/v3/time");
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], true);
    assert_eq!(report["online_execution_supported"], true);
    assert_eq!(report["network_attempted"], true);
    assert_eq!(report["production_public_online_read_attempted"], true);
    assert_eq!(report["response_status_code"], 200);
    assert_eq!(report["response_shape"], "binance_server_time_v1");
    assert_eq!(report["response_shape_validated"], true);
    assert_eq!(report["latency_ms"], 42);
    assert_eq!(report["error_code"], "none");
    assert_eq!(report["credentials_used"], false);
    assert_eq!(report["account_mutation_attempted"], false);
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_public_read_probe_records_owner_gated_online_failure_as_no_proof() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-001-public-read-online-failure-{}",
        std::process::id()
    ));
    let output = output_dir.join("public-read-probe.json");
    let opt = production_public_read_probe_opt(
        ProductionPublicReadEndpoint::ExchangeInfo,
        Some(output.clone()),
        true,
        true,
    );
    let mut read_env = all_env_enabled;

    run_live_production_public_read_probe_with_env_and_http(
        &opt,
        &mut read_env,
        |endpoint, url| {
            assert_eq!(endpoint, ProductionPublicReadEndpoint::ExchangeInfo);
            assert_eq!(url, "https://api.binance.com/api/v3/exchangeInfo");
            ProductionPublicReadProbeHttpResult::failure(
                endpoint,
                Some(7),
                Some(503),
                "http_status_not_success",
            )
        },
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(report["status"], "online_read_probe_failed");
    assert_eq!(report["endpoint"], "exchange_info");
    assert_eq!(report["path"], "/api/v3/exchangeInfo");
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], true);
    assert_eq!(report["network_attempted"], true);
    assert_eq!(report["production_public_online_read_attempted"], true);
    assert_eq!(report["response_status_code"], 503);
    assert_eq!(report["response_shape"], "binance_exchange_info_v1");
    assert_eq!(report["response_shape_validated"], false);
    assert_eq!(report["latency_ms"], 7);
    assert_eq!(report["error_code"], "http_status_not_success");
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_account_snapshot_contract_blocks_missing_gates_without_network() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v110-003-account-blocked-{}",
        std::process::id()
    ));
    let output = output_dir.join("account-snapshot-contract.json");
    let opt = production_account_snapshot_contract_opt(Some(output.clone()), false, false);

    run_live_production_account_snapshot_contract_with_env(&opt, |_| None).unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        report["schema_version"],
        PRODUCTION_ACCOUNT_SNAPSHOT_SCHEMA_VERSION
    );
    assert_eq!(report["status"], "blocked_missing_gate");
    assert_eq!(
        report["endpoint_class"],
        "production_authenticated_read_only"
    );
    assert_eq!(report["method"], "GET");
    assert_eq!(report["path"], "/api/v3/account");
    assert_eq!(report["requires_api_key"], true);
    assert_eq!(report["requires_signature"], true);
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["mutation_allowed"], false);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["env_credentials_only"], true);
    assert_eq!(report["api_key_value_recorded"], false);
    assert_eq!(report["api_secret_value_recorded"], false);
    assert_eq!(report["signature_recorded"], false);
    assert_eq!(report["signed_query_recorded"], false);
    assert_eq!(report["signed_url_recorded"], false);
    assert_eq!(report["account_read_attempted"], false);
    assert_eq!(report["account_mutation_attempted"], false);
    assert_eq!(report["order_endpoint_access_attempted"], false);
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
    assert_eq!(report["secrets_redacted"], true);
}

#[test]
fn production_account_snapshot_contract_blocks_missing_credentials() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v110-003-account-missing-credentials-{}",
        std::process::id()
    ));
    let output = output_dir.join("account-snapshot-contract.json");
    let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, false);

    run_live_production_account_snapshot_contract_with_env(&opt, |name| match name {
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE => Some("1".to_string()),
        _ => None,
    })
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(report["status"], "blocked_missing_credentials");
    assert_eq!(report["api_key_present"], false);
    assert_eq!(report["api_secret_present"], false);
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
}

#[test]
fn production_account_snapshot_contract_writes_ready_offline_redacted_contract() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v110-003-account-ready-{}",
        std::process::id()
    ));
    let output = output_dir.join("account-snapshot-contract.json");
    let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, false);

    run_live_production_account_snapshot_contract_with_env(&opt, |name| match name {
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE => Some("1".to_string()),
        "NTPRO_V110003_API_KEY" => Some("ntpro_v110003_synthetic_api_key_value".to_string()),
        "NTPRO_V110003_API_SECRET" => Some("ntpro_v110003_synthetic_api_secret_value".to_string()),
        _ => None,
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v110003_synthetic_api_key_value"));
    assert!(!body.contains("ntpro_v110003_synthetic_api_secret_value"));
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(report["status"], "ready_offline_contract");
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], true);
    assert_eq!(report["read_allowed"], true);
    assert_eq!(report["contract_ready"], true);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["api_key_present"], true);
    assert_eq!(report["api_secret_present"], true);
    assert_eq!(report["api_key_value_recorded"], false);
    assert_eq!(report["api_secret_value_recorded"], false);
    assert_eq!(report["signature_recorded"], false);
    assert_eq!(report["signed_query_recorded"], false);
    assert_eq!(report["signed_url_recorded"], false);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["account_read_attempted"], false);
    assert_eq!(report["order_endpoint_access_attempted"], false);
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
    assert_eq!(report["secrets_redacted"], true);
}

#[test]
fn production_account_snapshot_contract_blocks_manual_online_without_v12_owner_gate() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-002-account-online-blocked-{}",
        std::process::id()
    ));
    let output = output_dir.join("account-snapshot-contract.json");
    let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, true);
    let mut http_called = false;

    let mut read_env = |name: &str| match name {
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_MANUAL_ONLINE => None,
        PRODUCTION_ACCOUNT_SNAPSHOT_ENV_ALLOW
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_OWNER_APPROVED
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_ORDER_MUTATION
        | PRODUCTION_ACCOUNT_SNAPSHOT_ENV_NO_SECRET_PERSISTENCE => Some("1".to_string()),
        "NTPRO_V110003_API_KEY" => Some("ntpro_v120002_synthetic_api_key_value".to_string()),
        "NTPRO_V110003_API_SECRET" => Some("ntpro_v120002_synthetic_api_secret_value".to_string()),
        _ => None,
    };

    run_live_production_account_snapshot_contract_with_env_and_http(
        &opt,
        &mut read_env,
        |_credentials, _recv_window_ms| {
            http_called = true;
            ProductionAccountSnapshotHttpResult::success(1, 200)
        },
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert!(!http_called);
    assert_eq!(
        report["schema_version"],
        PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION
    );
    assert_eq!(report["status"], "blocked_missing_manual_online_gate");
    assert_eq!(report["manual_online_requested"], true);
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["online_execution_supported"], true);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["account_read_attempted"], false);
    assert_eq!(report["response_shape"], "binance_account_snapshot_v1");
    assert_eq!(report["response_shape_validated"], false);
    assert_eq!(report["error_code"], "not_attempted");
    assert_eq!(report["production_order_mutation_attempted"], false);
}

#[test]
fn production_account_snapshot_contract_records_owner_gated_online_success() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-002-account-online-success-{}",
        std::process::id()
    ));
    let output = output_dir.join("account-snapshot-contract.json");
    let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, true);
    let mut read_env = |name: &str| match name {
        "NTPRO_V110003_API_KEY" => Some("ntpro_v120002_synthetic_api_key_value".to_string()),
        "NTPRO_V110003_API_SECRET" => Some("ntpro_v120002_synthetic_api_secret_value".to_string()),
        _ => all_env_enabled(name),
    };

    run_live_production_account_snapshot_contract_with_env_and_http(
        &opt,
        &mut read_env,
        |credentials, recv_window_ms| {
            assert!(credentials.api_key_present());
            assert!(credentials.api_secret_present());
            assert_eq!(recv_window_ms, 5_000);
            ProductionAccountSnapshotHttpResult::success(53, 200)
        },
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v120002_synthetic_api_key_value"));
    assert!(!body.contains("ntpro_v120002_synthetic_api_secret_value"));
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        report["schema_version"],
        PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION
    );
    assert_eq!(report["status"], "online_account_snapshot_ok");
    assert_eq!(report["method"], "GET");
    assert_eq!(report["path"], "/api/v3/account");
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], true);
    assert_eq!(report["online_execution_supported"], true);
    assert_eq!(report["network_attempted"], true);
    assert_eq!(report["account_read_attempted"], true);
    assert_eq!(report["response_status_code"], 200);
    assert_eq!(report["response_shape"], "binance_account_snapshot_v1");
    assert_eq!(report["response_shape_validated"], true);
    assert_eq!(report["latency_ms"], 53);
    assert_eq!(report["error_code"], "none");
    assert_eq!(report["signature_recorded"], false);
    assert_eq!(report["signed_query_recorded"], false);
    assert_eq!(report["signed_url_recorded"], false);
    assert_eq!(report["account_mutation_attempted"], false);
    assert_eq!(report["order_endpoint_access_attempted"], false);
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_account_snapshot_contract_records_owner_gated_online_failure_as_no_proof() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-002-account-online-failure-{}",
        std::process::id()
    ));
    let output = output_dir.join("account-snapshot-contract.json");
    let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, true);
    let mut read_env = |name: &str| match name {
        "NTPRO_V110003_API_KEY" => Some("ntpro_v120002_synthetic_api_key_value".to_string()),
        "NTPRO_V110003_API_SECRET" => Some("ntpro_v120002_synthetic_api_secret_value".to_string()),
        _ => all_env_enabled(name),
    };

    run_live_production_account_snapshot_contract_with_env_and_http(
        &opt,
        &mut read_env,
        |_credentials, _recv_window_ms| {
            ProductionAccountSnapshotHttpResult::failure(
                Some(9),
                Some(401),
                "http_status_not_success",
            )
        },
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(report["status"], "online_account_snapshot_failed");
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], true);
    assert_eq!(report["network_attempted"], true);
    assert_eq!(report["account_read_attempted"], true);
    assert_eq!(report["response_status_code"], 401);
    assert_eq!(report["response_shape"], "binance_account_snapshot_v1");
    assert_eq!(report["response_shape_validated"], false);
    assert_eq!(report["latency_ms"], 9);
    assert_eq!(report["error_code"], "http_status_not_success");
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_account_snapshot_signed_request_redacts_secret_values() {
    let credentials = EnvOnlyProductionReadCredentials::from_values(
        "NTPRO_V120002_API_KEY".to_string(),
        Some("ntpro_v120002_synthetic_api_key_value".to_string()),
        "NTPRO_V120002_API_SECRET".to_string(),
        Some("ntpro_v120002_synthetic_api_secret_value".to_string()),
    );
    let request =
        build_production_account_snapshot_signed_request(&credentials, 1_718_400_000_000, 5_000)
            .unwrap();

    assert_eq!(request.method, "GET");
    assert_eq!(request.endpoint_path, "/api/v3/account");
    assert_eq!(request.api_key_header_name, BINANCE_API_KEY_HEADER);
    assert_eq!(
        request.query_without_signature,
        "timestamp=1718400000000&recvWindow=5000"
    );
    assert!(
        request
            .signed_query
            .starts_with("timestamp=1718400000000&recvWindow=5000&signature=")
    );
    assert_eq!(request.signature.len(), 64);
    assert!(
        request
            .signature
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    );
    assert_eq!(
        request.endpoint_url_redacted,
        "https://api.binance.com/api/v3/account"
    );

    let debug_body = format!("{request:?}");
    assert!(!debug_body.contains("ntpro_v120002_synthetic_api_key_value"));
    assert!(!debug_body.contains("ntpro_v120002_synthetic_api_secret_value"));
    assert!(!debug_body.contains(&request.signature));
    assert!(!debug_body.contains(&request.signed_query));
}

#[test]
fn production_account_snapshot_shape_summary_accepts_expected_shape() {
    let body = serde_json::json!({
        "accountType": "SPOT",
        "canTrade": true,
        "canWithdraw": false,
        "canDeposit": true,
        "permissions": ["SPOT"],
        "balances": [
            {"asset": "BTC", "free": "0.12345678", "locked": "0.00000000"},
            {"asset": "USDT", "free": "100.00", "locked": "0.00"}
        ]
    });

    let summary = summarize_production_account_snapshot_shape(&body);

    assert!(summary.shape_validated);
    assert_eq!(summary.status, "accepted");
    assert_eq!(summary.balance_entry_count, Some(2));
    assert_eq!(summary.permission_entry_count, Some(1));
    assert!(summary.account_type_is_string);
    assert!(summary.balance_entry_shape_validated);
    assert!(summary.permission_entry_shape_validated);
    assert!(summary.can_trade_is_bool);
    assert!(summary.can_withdraw_is_bool);
    assert!(summary.can_deposit_is_bool);
    assert!(!summary.raw_account_response_recorded);
    assert!(!summary.raw_balances_recorded);
    assert!(!summary.raw_permissions_recorded);

    let summary_body = serde_json::to_string(&summary).unwrap();
    assert!(!summary_body.contains("BTC"));
    assert!(!summary_body.contains("USDT"));
    assert!(!summary_body.contains("0.12345678"));
    assert!(!summary_body.contains("SPOT"));
}

#[test]
fn production_account_snapshot_shape_summary_rejects_missing_required_fields() {
    let body = serde_json::json!({
        "accountType": "SPOT",
        "canTrade": true,
        "balances": [
            {"asset": "BTC", "free": "0.12345678"}
        ]
    });

    let summary = summarize_production_account_snapshot_shape(&body);

    assert!(!summary.shape_validated);
    assert_eq!(summary.status, "rejected");
    assert_eq!(
        summary.rejection_reason,
        "missing_or_invalid_required_fields"
    );
    assert!(summary.account_type_is_string);
    assert!(summary.balances_is_array);
    assert_eq!(summary.balance_entry_count, Some(1));
    assert!(!summary.balance_entry_shape_validated);
    assert!(!summary.permissions_present);
    assert!(!summary.permissions_is_array);
    assert!(summary.can_trade_is_bool);
    assert!(!summary.can_withdraw_present);
    assert!(!summary.can_deposit_present);
}

#[test]
fn production_account_snapshot_online_invalid_shape_records_redacted_summary() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-003-account-shape-invalid-{}",
        std::process::id()
    ));
    let output = output_dir.join("account-snapshot-contract.json");
    let opt = production_account_snapshot_contract_opt(Some(output.clone()), true, true);
    let mut read_env = |name: &str| match name {
        "NTPRO_V110003_API_KEY" => Some("ntpro_v120003_synthetic_api_key_value".to_string()),
        "NTPRO_V110003_API_SECRET" => Some("ntpro_v120003_synthetic_api_secret_value".to_string()),
        _ => all_env_enabled(name),
    };
    let invalid_summary = summarize_production_account_snapshot_shape(&serde_json::json!({
        "accountType": "SPOT",
        "canTrade": true,
        "canWithdraw": false,
        "canDeposit": true,
        "balances": [
            {"asset": "ETH", "free": "1.50000000", "locked": "0.00000000"}
        ]
    }));

    run_live_production_account_snapshot_contract_with_env_and_http(
        &opt,
        &mut read_env,
        |_credentials, _recv_window_ms| {
            ProductionAccountSnapshotHttpResult::failure_with_shape(
                Some(11),
                Some(200),
                "response_shape_invalid",
                invalid_summary.clone(),
            )
        },
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v120003_synthetic_api_key_value"));
    assert!(!body.contains("ntpro_v120003_synthetic_api_secret_value"));
    assert!(!body.contains("ETH"));
    assert!(!body.contains("1.50000000"));
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(report["status"], "online_account_snapshot_failed");
    assert_eq!(report["error_code"], "response_shape_invalid");
    assert_eq!(report["response_shape_validated"], false);
    assert_eq!(report["response_shape_summary"]["status"], "rejected");
    assert_eq!(
        report["response_shape_summary"]["permissions_present"],
        false
    );
    assert_eq!(
        report["response_shape_summary"]["raw_account_response_recorded"],
        false
    );
    assert_eq!(
        report["response_shape_summary"]["raw_balances_recorded"],
        false
    );
    assert_eq!(
        report["response_shape_summary"]["raw_permissions_recorded"],
        false
    );
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_order_state_readonly_proof_blocks_missing_gates_without_network() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-001-order-state-blocked-{}",
        std::process::id()
    ));
    let output = output_dir.join("order-state-proof.json");
    let opt = production_order_state_readonly_proof_opt(
        ProductionOrderStateReadEndpoint::OpenOrders,
        Some(output.clone()),
        false,
        false,
    );

    run_live_production_order_state_readonly_proof_with_env(&opt, |_| None).unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        report["schema_version"],
        PRODUCTION_ORDER_STATE_READONLY_SCHEMA_VERSION
    );
    assert_eq!(report["status"], "blocked_missing_gate");
    assert_eq!(report["endpoint"], "open_orders");
    assert_eq!(report["endpoint_class"], "production_order_state_read_only");
    assert_eq!(report["method"], "GET");
    assert_eq!(report["path"], "/api/v3/openOrders");
    assert_eq!(report["requires_api_key"], true);
    assert_eq!(report["requires_signature"], true);
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["order_state_read_attempted"], false);
    assert_eq!(report["production_order_state_reads_attempted"], 0);
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["listen_key_lifecycle_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
    assert_eq!(report["secrets_redacted"], true);
}

#[test]
fn production_order_state_readonly_proof_writes_ready_offline_redacted_contract() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-001-order-state-ready-{}",
        std::process::id()
    ));
    let output = output_dir.join("order-state-proof.json");
    let opt = production_order_state_readonly_proof_opt(
        ProductionOrderStateReadEndpoint::OpenOrders,
        Some(output.clone()),
        true,
        false,
    );

    run_live_production_order_state_readonly_proof_with_env(&opt, |name| match name {
        PRODUCTION_ORDER_STATE_ENV_ALLOW
        | PRODUCTION_ORDER_STATE_ENV_OWNER_APPROVED
        | PRODUCTION_ORDER_STATE_ENV_NO_ORDER_MUTATION
        | PRODUCTION_ORDER_STATE_ENV_NO_SECRET_PERSISTENCE
        | PRODUCTION_ORDER_STATE_ENV_NO_LISTEN_KEY
        | PRODUCTION_ORDER_STATE_ENV_DASHBOARD_DISABLED => Some("1".to_string()),
        "NTPRO_V140001_API_KEY" => Some("ntpro_v140001_synthetic_api_key_value".to_string()),
        "NTPRO_V140001_API_SECRET" => Some("ntpro_v140001_synthetic_api_secret_value".to_string()),
        _ => None,
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v140001_synthetic_api_key_value"));
    assert!(!body.contains("ntpro_v140001_synthetic_api_secret_value"));
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(report["status"], "ready_offline_contract");
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], true);
    assert_eq!(report["read_allowed"], true);
    assert_eq!(report["contract_ready"], true);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["api_key_present"], true);
    assert_eq!(report["api_secret_present"], true);
    assert_eq!(report["api_key_value_recorded"], false);
    assert_eq!(report["api_secret_value_recorded"], false);
    assert_eq!(report["signature_recorded"], false);
    assert_eq!(report["signed_query_recorded"], false);
    assert_eq!(report["signed_url_recorded"], false);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["order_state_read_attempted"], false);
    assert_eq!(report["production_order_state_reads_attempted"], 0);
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
    assert_eq!(report["secrets_redacted"], true);
}

#[test]
fn production_order_state_readonly_proof_blocks_manual_online_without_v14_owner_gate() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-001-order-state-online-blocked-{}",
        std::process::id()
    ));
    let output = output_dir.join("order-state-proof.json");
    let opt = production_order_state_readonly_proof_opt(
        ProductionOrderStateReadEndpoint::OpenOrders,
        Some(output.clone()),
        true,
        true,
    );
    let mut http_called = false;
    let mut read_env = |name: &str| match name {
        PRODUCTION_ORDER_STATE_ENV_MANUAL_ONLINE => None,
        PRODUCTION_ORDER_STATE_ENV_ALLOW
        | PRODUCTION_ORDER_STATE_ENV_OWNER_APPROVED
        | PRODUCTION_ORDER_STATE_ENV_NO_ORDER_MUTATION
        | PRODUCTION_ORDER_STATE_ENV_NO_SECRET_PERSISTENCE
        | PRODUCTION_ORDER_STATE_ENV_NO_LISTEN_KEY
        | PRODUCTION_ORDER_STATE_ENV_DASHBOARD_DISABLED => Some("1".to_string()),
        "NTPRO_V140001_API_KEY" => Some("ntpro_v140001_synthetic_api_key_value".to_string()),
        "NTPRO_V140001_API_SECRET" => Some("ntpro_v140001_synthetic_api_secret_value".to_string()),
        _ => None,
    };

    run_live_production_order_state_readonly_proof_with_env_and_http(
        &opt,
        &mut read_env,
        |_opt, _credentials, _recv_window_ms| {
            http_called = true;
            ProductionOrderStateHttpResult::success(
                ProductionOrderStateReadEndpoint::OpenOrders,
                1,
                200,
            )
        },
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert!(!http_called);
    assert_eq!(report["status"], "blocked_missing_manual_online_gate");
    assert_eq!(report["manual_online_requested"], true);
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], false);
    assert_eq!(report["online_execution_supported"], true);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["order_state_read_attempted"], false);
    assert_eq!(report["production_order_state_reads_attempted"], 0);
    assert_eq!(report["error_code"], "not_attempted");
    assert_eq!(report["production_order_mutation_attempted"], false);
}

#[test]
fn production_order_state_readonly_proof_records_owner_gated_online_success() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-001-order-state-online-success-{}",
        std::process::id()
    ));
    let output = output_dir.join("order-state-proof.json");
    let opt = production_order_state_readonly_proof_opt(
        ProductionOrderStateReadEndpoint::Order,
        Some(output.clone()),
        true,
        true,
    );
    let mut read_env = |name: &str| match name {
        "NTPRO_V140001_API_KEY" => Some("ntpro_v140001_synthetic_api_key_value".to_string()),
        "NTPRO_V140001_API_SECRET" => Some("ntpro_v140001_synthetic_api_secret_value".to_string()),
        _ => all_env_enabled(name),
    };

    run_live_production_order_state_readonly_proof_with_env_and_http(
        &opt,
        &mut read_env,
        |proof_opt, credentials, recv_window_ms| {
            assert_eq!(proof_opt.endpoint, ProductionOrderStateReadEndpoint::Order);
            assert!(credentials.api_key_present());
            assert!(credentials.api_secret_present());
            assert_eq!(recv_window_ms, 5_000);
            ProductionOrderStateHttpResult::success(
                ProductionOrderStateReadEndpoint::Order,
                17,
                200,
            )
        },
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v140001_synthetic_api_key_value"));
    assert!(!body.contains("ntpro_v140001_synthetic_api_secret_value"));
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(report["status"], "online_order_state_read_ok");
    assert_eq!(report["endpoint"], "order");
    assert_eq!(report["method"], "GET");
    assert_eq!(report["path"], "/api/v3/order");
    assert_eq!(report["endpoint_read_allowed"], true);
    assert_eq!(report["offline_contract_ready"], false);
    assert_eq!(report["read_allowed"], false);
    assert_eq!(report["contract_ready"], false);
    assert_eq!(report["online_read_allowed"], true);
    assert_eq!(report["online_execution_supported"], true);
    assert_eq!(report["network_attempted"], true);
    assert_eq!(report["order_state_read_attempted"], true);
    assert_eq!(report["production_order_state_reads_attempted"], 1);
    assert_eq!(report["response_status_code"], 200);
    assert_eq!(report["response_shape"], "binance_order_state_v1");
    assert_eq!(report["response_shape_validated"], true);
    assert_eq!(report["endpoint_shape_validated"], true);
    assert_eq!(report["order_entries_observed"], 1);
    assert_eq!(report["non_empty_order_state_observed"], true);
    assert_eq!(report["order_lifecycle_readiness"], true);
    assert_eq!(report["latency_ms"], 17);
    assert_eq!(report["error_code"], "none");
    assert_eq!(report["production_order_submission_attempted"], false);
    assert_eq!(report["production_order_mutation_attempted"], false);
    assert_eq!(report["listen_key_lifecycle_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
    assert_eq!(report["real_orders_submitted"], false);
    assert_eq!(report["production_trading_enabled"], false);
    assert_eq!(report["order_state_values_are_exchange_truth"], true);
    assert_eq!(report["shadow_values_are_exchange_truth"], false);
    assert_eq!(report["portfolio_values_are_exchange_truth"], false);
    assert_eq!(report["values_are_exchange_truth"], true);
}

#[test]
fn production_order_state_signed_request_redacts_secret_values() {
    let credentials = EnvOnlyProductionReadCredentials::from_values(
        "NTPRO_V140001_API_KEY".to_string(),
        Some("ntpro_v140001_synthetic_api_key_value".to_string()),
        "NTPRO_V140001_API_SECRET".to_string(),
        Some("ntpro_v140001_synthetic_api_secret_value".to_string()),
    );
    let opt = production_order_state_readonly_proof_opt(
        ProductionOrderStateReadEndpoint::Order,
        None,
        true,
        true,
    );

    let request =
        build_production_order_state_signed_request(&opt, &credentials, 1_718_400_000_000, 5_000)
            .unwrap();

    assert_eq!(request.method, "GET");
    assert_eq!(request.endpoint_path, "/api/v3/order");
    assert_eq!(request.api_key_header_name, BINANCE_API_KEY_HEADER);
    assert!(request.query_without_signature.contains("symbol=BTCUSDT"));
    assert!(request.query_without_signature.contains("orderId=12345"));
    assert!(request.signed_query.contains("timestamp=1718400000000"));
    assert_eq!(request.signature.len(), 64);
    assert!(
        request
            .signature
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    );

    let debug_body = format!("{request:?}");
    assert!(!debug_body.contains("ntpro_v140001_synthetic_api_key_value"));
    assert!(!debug_body.contains("ntpro_v140001_synthetic_api_secret_value"));
    assert!(!debug_body.contains(&request.signature));
    assert!(!debug_body.contains(&request.signed_query));
}

#[test]
fn production_order_state_shape_summary_accepts_expected_shapes() {
    let open_orders = serde_json::json!([
        {"symbol": "BTCUSDT", "orderId": 12345, "status": "NEW"}
    ]);
    let single_order = serde_json::json!({
        "symbol": "BTCUSDT",
        "orderId": 12345,
        "status": "FILLED"
    });

    let open_summary = summarize_production_order_state_shape(
        ProductionOrderStateReadEndpoint::OpenOrders,
        &open_orders,
    );
    let order_summary = summarize_production_order_state_shape(
        ProductionOrderStateReadEndpoint::Order,
        &single_order,
    );

    assert!(open_summary.shape_validated);
    assert!(open_summary.endpoint_shape_validated);
    assert_eq!(open_summary.order_entry_count, Some(1));
    assert_eq!(open_summary.order_entries_observed, 1);
    assert!(open_summary.non_empty_order_state_observed);
    assert!(open_summary.order_lifecycle_readiness);
    assert!(!open_summary.raw_order_list_recorded);
    assert!(order_summary.shape_validated);
    assert!(order_summary.endpoint_shape_validated);
    assert_eq!(order_summary.order_entry_count, Some(1));
    assert_eq!(order_summary.order_entries_observed, 1);
    assert!(order_summary.non_empty_order_state_observed);
    assert!(order_summary.order_lifecycle_readiness);
    assert!(!order_summary.raw_order_response_recorded);

    let summary_body = serde_json::to_string(&open_summary).unwrap();
    assert!(!summary_body.contains("BTCUSDT"));
    assert!(!summary_body.contains("12345"));
    assert!(!summary_body.contains("NEW"));
}

#[test]
fn production_order_state_shape_summary_classifies_empty_open_orders_as_shape_only() {
    let empty_open_orders = serde_json::json!([]);

    let summary = summarize_production_order_state_shape(
        ProductionOrderStateReadEndpoint::OpenOrders,
        &empty_open_orders,
    );

    assert!(summary.shape_validated);
    assert!(summary.endpoint_shape_validated);
    assert_eq!(summary.status, "accepted");
    assert_eq!(summary.order_entry_count, Some(0));
    assert_eq!(summary.order_entries_observed, 0);
    assert!(!summary.non_empty_order_state_observed);
    assert!(!summary.order_lifecycle_readiness);
    assert_eq!(summary.rejection_reason, "none");
    assert!(!summary.raw_order_list_recorded);
}

#[test]
fn production_order_state_shape_summary_rejects_invalid_open_orders_shape() {
    let invalid_open_orders = serde_json::json!({"symbol": "BTCUSDT"});

    let summary = summarize_production_order_state_shape(
        ProductionOrderStateReadEndpoint::OpenOrders,
        &invalid_open_orders,
    );

    assert!(!summary.shape_validated);
    assert!(!summary.endpoint_shape_validated);
    assert_eq!(summary.status, "rejected");
    assert_eq!(summary.order_entry_count, None);
    assert_eq!(summary.order_entries_observed, 0);
    assert!(!summary.non_empty_order_state_observed);
    assert!(!summary.order_lifecycle_readiness);
    assert_eq!(summary.rejection_reason, "root_not_array");
}

#[test]
fn production_shadow_portfolio_runtime_writes_redacted_runtime_and_compat_snapshot() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-004-shadow-portfolio-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let runtime_output = output_dir.join("shadow_portfolio_runtime.json");
    let compat_output = output_dir.join("shadow_portfolio_snapshot.json");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);

    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v120-shadow".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot,
        shadow_intent,
        output: runtime_output.clone(),
        compat_snapshot_output: Some(compat_output.clone()),
    })
    .unwrap();

    let runtime_body = fs::read_to_string(runtime_output).unwrap();
    assert!(!runtime_body.contains("\"asset\": \"BTC\""));
    assert!(!runtime_body.contains("\"free\":"));
    assert!(!runtime_body.contains("\"locked\":"));
    assert!(!runtime_body.contains("api_secret"));
    let runtime: serde_json::Value = serde_json::from_str(&runtime_body).unwrap();
    assert_eq!(
        runtime["schema_version"],
        PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION
    );
    assert_eq!(runtime["status"], "ready_redacted_shadow_portfolio");
    assert_eq!(runtime["balances"]["status"], "observed_shape_only");
    assert_eq!(runtime["balances"]["observed_balance_entry_count"], 2);
    assert_eq!(runtime["balances"]["asset_values_recorded"], false);
    assert_eq!(
        runtime["source_shadow_intent_refs"][0]["intent_id"],
        "intent-1"
    );
    assert_eq!(runtime["exposure"]["status"], "derived_from_shadow_intents");
    assert_eq!(runtime["exposure"]["notional"], "10");
    assert_eq!(
        runtime["notional_preflight"]["status"],
        "shadow_decimal_string_evidence_only"
    );
    assert_eq!(
        runtime["notional_preflight"]["aggregation"],
        "rust_decimal_string_sum"
    );
    assert_eq!(runtime["notional_preflight"]["decimal_string_sum"], "10");
    assert_eq!(runtime["notional_preflight"]["parsed_notional_count"], 1);
    assert_eq!(runtime["notional_preflight"]["f64_aggregation_used"], false);
    assert_eq!(
        runtime["notional_preflight"]["live_alpha_money_math_ready"],
        false
    );
    assert_eq!(
        runtime["notional_preflight"]["risk_or_execution_grade"],
        false
    );
    assert_eq!(runtime["pnl"]["status"], "unavailable");
    assert_eq!(runtime["risk_summary"]["new_orders_blocked"], true);
    assert_eq!(runtime["production_orders_submitted"], 0);
    assert_eq!(runtime["production_order_mutations_attempted"], 0);
    assert_eq!(runtime["dashboard_order_controls_enabled"], false);
    assert_eq!(runtime["full_production_portfolio_parity_claimed"], false);

    let compat: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(compat_output).unwrap()).unwrap();
    assert_eq!(
        compat["schema_version"],
        PRODUCTION_SHADOW_PORTFOLIO_COMPAT_SCHEMA_VERSION
    );
    assert_eq!(compat["snapshot_mode"], "production_readonly_shadow");
    assert_eq!(compat["balances"][0]["asset"], "redacted");
    assert_eq!(compat["exposure"]["status"], "derived_from_shadow_intents");
    assert_eq!(compat["pnl"]["status"], "unavailable");
    assert_eq!(compat["production_orders_submitted"], 0);
    assert_eq!(compat["dashboard_order_controls_enabled"], false);
    assert_eq!(compat["full_production_portfolio_parity_claimed"], false);
}

#[test]
fn production_shadow_portfolio_runtime_preserves_decimal_string_notional_preflight() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v121-008-shadow-portfolio-decimal-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent_with_notional(&shadow_intent, false, "0.100000000000000001");
    fs::write(
        &shadow_intent,
        format!(
            "{}{}",
            fs::read_to_string(&shadow_intent).unwrap(),
            r#"{"schema_version":"ntpro.v110_shadow_execution_intent.v1","run_id":"v120-shadow","intent_id":"intent-2","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","venue":"BINANCE","side":"buy","order_type":"market","quantity":"0.001","notional":"0.200000000000000002","mode":"production_shadow","submission_allowed":false,"actual_submission":false,"submission_status":"blocked_by_v110_shadow_execution_boundary","execution_adapter_called":false,"order_endpoint_access_attempted":false,"production_order_mutation_attempted":false,"dashboard_order_controls_enabled":false}
"#
        ),
    )
    .unwrap();

    let report = build_production_shadow_portfolio_runtime_report(
        "v120-shadow",
        Some("portfolio-1"),
        &account_snapshot,
        &shadow_intent,
    )
    .unwrap();

    assert_eq!(
        report.notional_preflight.status,
        "shadow_decimal_string_evidence_only"
    );
    assert_eq!(
        report.notional_preflight.aggregation,
        "rust_decimal_string_sum"
    );
    assert_eq!(
        report.notional_preflight.decimal_string_sum.as_deref(),
        Some("0.300000000000000003")
    );
    assert_eq!(report.notional_preflight.parsed_notional_count, 2);
    assert!(!report.notional_preflight.f64_aggregation_used);
    assert!(!report.notional_preflight.live_alpha_money_math_ready);
    assert!(!report.notional_preflight.risk_or_execution_grade);
    assert_eq!(
        report.exposure.notional.as_deref(),
        Some("0.300000000000000003")
    );
}

#[test]
fn v13_live_alpha_amount_boundary_uses_decimal_strings_without_f64() {
    let first = parse_non_negative_decimal("0.100000000000000001").unwrap();
    let second = parse_non_negative_decimal("0.200000000000000002").unwrap();
    let sum = first + second;
    let f64_sum = 0.1_f64 + 0.2_f64;

    assert_eq!(format_decimal(&sum), "0.300000000000000003");
    assert_eq!(format!("{sum}"), "0.300000000000000003");
    assert_ne!(format!("{sum}"), format!("{f64_sum}"));

    for invalid in ["", "-0.1", "1e-5", "NaN", "inf"] {
        assert!(
            parse_non_negative_decimal(invalid).is_err(),
            "v0.13 amount boundary must reject non-plain decimal string {invalid}",
        );
    }
}

#[test]
fn production_shadow_portfolio_runtime_rejects_raw_account_balances() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-004-shadow-portfolio-raw-account-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("raw-account.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    fs::write(
        &account_snapshot,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": PRODUCTION_ACCOUNT_SNAPSHOT_ONLINE_SCHEMA_VERSION,
            "balances": [{"asset": "BTC", "free": "1", "locked": "0"}],
            "response_shape_validated": true
        }))
        .unwrap(),
    )
    .unwrap();
    write_shadow_intent(&shadow_intent, false);

    let error = build_production_shadow_portfolio_runtime_report(
        "v120-shadow",
        None,
        &account_snapshot,
        &shadow_intent,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("requires a redacted account summary"));
}

#[test]
fn production_shadow_portfolio_runtime_rejects_actual_submission_intents() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-004-shadow-portfolio-submission-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, true);

    let error = build_production_shadow_portfolio_runtime_report(
        "v120-shadow",
        None,
        &account_snapshot,
        &shadow_intent,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("actual_submission=true"));
}

#[test]
fn production_shadow_strategy_session_writes_heartbeat_gap_and_stop_events() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-005-shadow-strategy-session-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let session_events = output_dir.join("shadow_strategy_session.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);
    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v120-shadow".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot,
        shadow_intent,
        output: portfolio_runtime.clone(),
        compat_snapshot_output: None,
    })
    .unwrap();

    run_live_production_shadow_strategy_session(&LiveProductionShadowStrategySessionOpt {
        run_id: "v120-shadow".to_string(),
        session_id: Some("session-1".to_string()),
        strategy_id: "ema_cross_btcusdt_v1".to_string(),
        shadow_portfolio_runtime: portfolio_runtime,
        strategy_session_status: None,
        output: session_events.clone(),
        heartbeat_count: 2,
        stop_after_heartbeats: true,
        stop_file: None,
    })
    .unwrap();

    let events = read_jsonl_values(&session_events);
    assert_eq!(events.len(), 5);
    assert_eq!(
        events[0]["schema_version"],
        PRODUCTION_SHADOW_STRATEGY_SESSION_EVENT_SCHEMA_VERSION
    );
    assert_eq!(events[0]["event_type"], "shadow_strategy_session_started");
    assert_eq!(events[0]["state"], "degraded_artifact_gap");
    assert_eq!(events[0]["artifact_gap"]["status"], "not_provided");
    assert_eq!(
        events[1]["event_type"],
        "shadow_strategy_session_artifact_gap"
    );
    assert_eq!(events[2]["event_type"], "shadow_strategy_session_heartbeat");
    assert_eq!(events[2]["heartbeat_seq"], 1);
    assert_eq!(events[3]["event_type"], "shadow_strategy_session_heartbeat");
    assert_eq!(events[3]["heartbeat_seq"], 2);
    assert_eq!(events[4]["event_type"], "shadow_strategy_session_stopped");
    assert_eq!(events[4]["state"], "stopped");
    for event in &events {
        assert_eq!(event["production_order_submissions_attempted"], 0);
        assert_eq!(event["production_orders_submitted"], 0);
        assert_eq!(event["production_order_mutations_attempted"], 0);
        assert_eq!(event["production_order_state_reads_attempted"], 0);
        assert_eq!(event["listen_key_lifecycle_attempted"], 0);
        assert_eq!(event["dashboard_order_controls_enabled"], false);
        assert_eq!(event["values_are_exchange_truth"], false);
    }
}

#[test]
fn production_shadow_strategy_session_consumes_existing_session_status() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-005-shadow-strategy-session-status-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let status_path = output_dir.join("strategy_session_status.json");
    let session_events = output_dir.join("shadow_strategy_session.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);
    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v120-shadow".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot,
        shadow_intent,
        output: portfolio_runtime.clone(),
        compat_snapshot_output: None,
    })
    .unwrap();
    fs::write(
        &status_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "ntpro.v09_strategy_session_status.v1",
            "session_id": "session-1",
            "strategy_id": "ema_cross_btcusdt_v1",
            "state": "running",
            "reason": "fixture strategy running"
        }))
        .unwrap(),
    )
    .unwrap();

    run_live_production_shadow_strategy_session(&LiveProductionShadowStrategySessionOpt {
        run_id: "v120-shadow".to_string(),
        session_id: Some("session-1".to_string()),
        strategy_id: "ema_cross_btcusdt_v1".to_string(),
        shadow_portfolio_runtime: portfolio_runtime,
        strategy_session_status: Some(status_path.clone()),
        output: session_events.clone(),
        heartbeat_count: 1,
        stop_after_heartbeats: false,
        stop_file: None,
    })
    .unwrap();

    let events = read_jsonl_values(&session_events);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["state"], "running");
    assert!(events[0].get("artifact_gap").is_none());
    assert_eq!(
        events[0]["strategy_session_status_ref"]["path"],
        status_path.display().to_string()
    );
    assert_eq!(events[0]["strategy_session_status_ref"]["state"], "running");
    assert_eq!(events[1]["event_type"], "shadow_strategy_session_heartbeat");
}

#[tokio::test]
async fn production_shadow_preflight_session_writes_heartbeats_and_stops() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v130-002-shadow-preflight-session-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let preflight_events = output_dir.join("shadow_preflight_session.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);
    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v130-shadow".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot,
        shadow_intent,
        output: portfolio_runtime.clone(),
        compat_snapshot_output: None,
    })
    .unwrap();

    let result =
        run_production_shadow_preflight_session_loop(&LiveProductionShadowPreflightSessionOpt {
            run_id: "v130-shadow".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            shadow_portfolio_runtime: portfolio_runtime,
            strategy_session_status: None,
            output: preflight_events.clone(),
            max_heartbeats: 2,
            heartbeat_interval_ms: 1,
            stale_after_ms: 60_000,
            stop_file: None,
        })
        .await
        .unwrap();

    assert_eq!(result.heartbeats_written, 2);
    assert_eq!(result.final_state, "stopped");
    assert!(!result.stop_file_observed);
    assert!(!result.stale_data_detected);
    let events = read_jsonl_values(&preflight_events);
    assert_eq!(events.len(), 4);
    assert_eq!(
        events[0]["schema_version"],
        PRODUCTION_SHADOW_PREFLIGHT_SESSION_EVENT_SCHEMA_VERSION
    );
    assert_eq!(events[0]["event_type"], "shadow_preflight_session_started");
    assert_eq!(
        events[1]["event_type"],
        "shadow_preflight_session_heartbeat"
    );
    assert_eq!(events[1]["heartbeat_seq"], 1);
    assert_eq!(
        events[2]["event_type"],
        "shadow_preflight_session_heartbeat"
    );
    assert_eq!(events[2]["heartbeat_seq"], 2);
    assert_eq!(events[3]["event_type"], "shadow_preflight_session_stopped");
    assert_eq!(events[3]["shutdown_reason"], "max_heartbeats_reached");
    for event in &events {
        assert_eq!(event["session_network_attempted"], false);
        assert_eq!(event["production_order_submissions_attempted"], 0);
        assert_eq!(event["production_orders_submitted"], 0);
        assert_eq!(event["production_order_mutations_attempted"], 0);
        assert_eq!(event["production_order_state_reads_attempted"], 0);
        assert_eq!(event["listen_key_lifecycle_attempted"], 0);
        assert_eq!(event["cancel_replace_amend_attempted"], false);
        assert_eq!(event["dashboard_order_controls_enabled"], false);
        assert_eq!(event["values_are_exchange_truth"], false);
    }
}

#[tokio::test]
async fn production_shadow_preflight_session_stops_on_owner_stop_file() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v130-002-shadow-preflight-stop-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let preflight_events = output_dir.join("shadow_preflight_session.jsonl");
    let stop_file = output_dir.join("STOP");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);
    fs::write(&stop_file, "stop").unwrap();
    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v130-shadow-stop".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot,
        shadow_intent,
        output: portfolio_runtime.clone(),
        compat_snapshot_output: None,
    })
    .unwrap();

    let result =
        run_production_shadow_preflight_session_loop(&LiveProductionShadowPreflightSessionOpt {
            run_id: "v130-shadow-stop".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            shadow_portfolio_runtime: portfolio_runtime,
            strategy_session_status: None,
            output: preflight_events.clone(),
            max_heartbeats: 5,
            heartbeat_interval_ms: 1,
            stale_after_ms: 60_000,
            stop_file: Some(stop_file.clone()),
        })
        .await
        .unwrap();

    assert_eq!(result.heartbeats_written, 0);
    assert_eq!(result.final_state, "stopped");
    assert!(result.stop_file_observed);
    let events = read_jsonl_values(&preflight_events);
    assert_eq!(events.len(), 2);
    assert_eq!(events[1]["event_type"], "shadow_preflight_session_stopped");
    assert_eq!(events[1]["shutdown_reason"], "owner_stop_file");
    assert_eq!(events[1]["stop_file_observed"], true);
    assert_eq!(events[1]["stop_file_path"], stop_file.display().to_string());
    assert_eq!(events[1]["production_order_mutations_attempted"], 0);
}

#[tokio::test]
async fn production_shadow_preflight_session_detects_stale_portfolio_runtime() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v130-002-shadow-preflight-stale-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let preflight_events = output_dir.join("shadow_preflight_session.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);
    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v130-shadow-stale".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot,
        shadow_intent,
        output: portfolio_runtime.clone(),
        compat_snapshot_output: None,
    })
    .unwrap();
    sleep(Duration::from_millis(5)).await;

    let result =
        run_production_shadow_preflight_session_loop(&LiveProductionShadowPreflightSessionOpt {
            run_id: "v130-shadow-stale".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            shadow_portfolio_runtime: portfolio_runtime,
            strategy_session_status: None,
            output: preflight_events.clone(),
            max_heartbeats: 2,
            heartbeat_interval_ms: 1,
            stale_after_ms: 1,
            stop_file: None,
        })
        .await
        .unwrap();

    assert_eq!(result.heartbeats_written, 0);
    assert_eq!(result.final_state, "stale_data_halted");
    assert!(result.stale_data_detected);
    let events = read_jsonl_values(&preflight_events);
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[1]["event_type"],
        "shadow_preflight_stale_data_detected"
    );
    assert_eq!(events[1]["state"], "stale_data_halted");
    assert_eq!(events[1]["stale_data_detected"], true);
    assert_eq!(
        events[1]["shutdown_reason"],
        "stale_shadow_portfolio_runtime"
    );
    assert_eq!(events[1]["production_orders_submitted"], 0);
}

#[test]
fn production_live_alpha_dry_run_order_gate_blocks_missing_owner_flags() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-003-live-alpha-dry-run-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let output = output_dir.join("live_alpha_dry_run_order_gate.json");

    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_dry_run_order_gate_opt(output.clone(), false),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["dry_run_order_gate_ready"], false);
    assert_eq!(artifact["dry_run_order_intent_recorded"], false);
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 8);
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["order_endpoint_access_attempted"], false);
    assert_eq!(artifact["execution_adapter_called"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["real_funds"], false);
}

#[test]
fn production_live_alpha_dry_run_order_gate_rejects_market_order_type() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v151-004-live-alpha-market-order-gate-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let output = output_dir.join("live_alpha_market_order_gate.json");
    let mut opt = production_live_alpha_dry_run_order_gate_opt(output, true);
    opt.order_type = "MARKET".to_string();

    let err = run_live_production_live_alpha_dry_run_order_gate(&opt).unwrap_err();
    assert!(
        err.to_string().contains("only supports LIMIT order_type"),
        "{err:?}"
    );
}

#[test]
fn production_live_alpha_dry_run_order_gate_records_ready_no_submission_contract() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-003-live-alpha-dry-run-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let output = output_dir.join("live_alpha_dry_run_order_gate.json");

    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_dry_run_order_gate_opt(output.clone(), true),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "ready_dry_run_no_submission");
    assert_eq!(artifact["mode"], "production_live_alpha_dry_run");
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["side"], "BUY");
    assert_eq!(artifact["order_type"], "LIMIT");
    assert_eq!(artifact["quantity"], "0.001");
    assert_eq!(artifact["notional"], "10.00");
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["dry_run_order_gate_ready"], true);
    assert_eq!(artifact["dry_run_order_intent_recorded"], true);
    assert_eq!(artifact["order_submission_mode"], "dry_run_no_submission");
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_state_reads_allowed"], false);
    assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
    assert_eq!(artifact["cancel_replace_amend_attempted"], false);
    assert_eq!(artifact["order_endpoint_access_attempted"], false);
    assert_eq!(artifact["execution_adapter_called"], false);
    assert_eq!(artifact["matching_engine_submission"], false);
    assert_eq!(artifact["actual_submission_count"], 0);
    assert_eq!(artifact["automatic_correction_orders_submitted"], 0);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["external_venue_connection"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["real_funds"], false);
    assert_eq!(artifact["production_trading_enabled"], false);
    assert_eq!(artifact["values_are_exchange_truth"], false);
}

#[test]
fn production_live_alpha_order_request_preview_builds_redacted_metadata_only() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v150-002-request-preview-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let output = output_dir.join("live_alpha_order_request_preview.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();

    let opt = production_live_alpha_order_request_preview_opt(
        order_gate,
        manual_approval_lifecycle.clone(),
        output.clone(),
        true,
    );
    run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| {
        panic!("default synthetic signing material must not read env var {name}")
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains(PRODUCTION_MUTATION_PREVIEW_SYNTHETIC_API_KEY));
    assert!(!body.contains(PRODUCTION_MUTATION_PREVIEW_SYNTHETIC_API_SECRET));
    assert!(!body.contains("signature="));
    assert!(!body.contains("symbol=BTCUSDT"));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "ready_request_preview_only");
    assert_eq!(
        artifact["endpoint_class"],
        "production_mutation_owner_approved_manual_only"
    );
    assert_eq!(artifact["endpoint_decision"], "allow_request_preview_only");
    assert_eq!(artifact["request_method"], "POST");
    assert_eq!(artifact["request_target"], TESTNET_ORDER_ENDPOINT_ORDER);
    assert_eq!(
        artifact["query_shape_without_signature"],
        "symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp"
    );
    assert_eq!(
        artifact["signature_preflight"],
        "created_in_memory_not_recorded"
    );
    assert_eq!(artifact["credential_material"], "synthetic");
    assert_eq!(artifact["production_signing_material_gate_required"], false);
    assert_eq!(artifact["production_signing_material_gate_open"], false);
    assert_eq!(artifact["production_signing_material_env_read"], false);
    assert_eq!(
        artifact["production_signing_material_missing_gate_env_vars"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["api_key_header_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["request_body_recorded"], false);
    assert_eq!(artifact["raw_request_body_recorded"], false);
    assert_eq!(
        artifact["manual_approval_lifecycle_status"],
        "approval_valid_for_dry_run_request_preview"
    );
    assert_eq!(artifact["manual_approval_lifecycle_state"], "approved");
    assert_eq!(artifact["manual_approval_lifecycle_valid"], true);
    assert_eq!(
        artifact["manual_approval_lifecycle_issues"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(artifact["manual_approval_one_time"], true);
    assert_eq!(artifact["manual_approval_used"], true);
    assert_eq!(artifact["manual_approval_consumed"], true);
    assert_eq!(
        artifact["manual_approval_consume_status"],
        "approval_consumed_after_request_preview_created"
    );
    assert_eq!(
        artifact["manual_approval_consume_transition"],
        "approved_to_request_preview_created_to_used"
    );
    assert_eq!(artifact["order_gate_ready"], true);
    assert_eq!(artifact["request_preview_allowed"], true);
    assert_eq!(artifact["request_preview_built"], true);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["order_endpoint_access_attempted"], false);
    assert_eq!(artifact["execution_adapter_called"], false);
    assert_eq!(artifact["production_adapter_called"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["real_funds"], false);
    assert_eq!(artifact["production_trading_enabled"], false);
    assert_eq!(artifact["signed_request_memory_only"], true);
    assert_eq!(artifact["secrets_redacted"], true);

    let consumed_approval: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manual_approval_lifecycle).unwrap()).unwrap();
    assert_eq!(
        consumed_approval["status"],
        "approval_consumed_after_request_preview_created"
    );
    assert_eq!(consumed_approval["approval_state"], "used");
    assert_eq!(consumed_approval["approval_used"], true);
    assert_eq!(consumed_approval["approval_consumed"], true);
    assert_eq!(consumed_approval["request_preview_created"], true);
    assert_eq!(
        consumed_approval["consumed_by_request_preview_run_id"],
        "v150-live-alpha-request-preview"
    );
    assert_eq!(consumed_approval["approval_lifecycle_valid"], false);
}

#[test]
fn production_live_alpha_order_request_preview_rejects_order_test_endpoint() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v151-005-order-test-preview-denied-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let output = output_dir.join("live_alpha_order_request_preview.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();

    let mut opt = production_live_alpha_order_request_preview_opt(
        order_gate,
        manual_approval_lifecycle,
        output,
        true,
    );
    opt.endpoint_path = TESTNET_ORDER_ENDPOINT_TEST.to_string();

    let err = run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| {
        panic!("denied /api/v3/order/test preview must not read env var {name}")
    })
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("allowlist only includes POST /api/v3/order"),
        "{err:?}"
    );
}

#[test]
fn production_live_alpha_order_request_preview_blocks_production_material_without_gates() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v151-003-production-material-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let output = output_dir.join("live_alpha_order_request_preview.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();

    let mut opt = production_live_alpha_order_request_preview_opt(
        order_gate,
        manual_approval_lifecycle,
        output.clone(),
        true,
    );
    opt.credential_material = "production_live_alpha".to_string();

    run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| match name {
        PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
        | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => None,
        "NTPRO_V150002_API_KEY" | "NTPRO_V150002_API_SECRET" => {
            panic!("blocked production signing material must not read {name}")
        }
        _ => None,
    })
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_endpoint_or_owner_scope");
    assert_eq!(artifact["credential_material"], "production_live_alpha");
    assert_eq!(artifact["production_signing_material_gate_required"], true);
    assert_eq!(artifact["production_signing_material_gate_open"], false);
    assert_eq!(artifact["production_signing_material_env_read"], false);
    assert_eq!(
        artifact["production_signing_material_missing_gate_env_vars"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        artifact["missing_env_vars"]
            .as_array()
            .unwrap()
            .iter()
            .any(|env| env.as_str() == Some(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW))
    );
    assert!(
        artifact["missing_env_vars"].as_array().unwrap().iter().any(
            |env| env.as_str() == Some(PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED)
        )
    );
    assert_eq!(artifact["request_preview_allowed"], false);
    assert_eq!(artifact["request_preview_built"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
}

#[test]
fn production_live_alpha_order_request_preview_uses_production_material_only_with_gates() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v151-003-production-material-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let output = output_dir.join("live_alpha_order_request_preview.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();

    let mut opt = production_live_alpha_order_request_preview_opt(
        order_gate,
        manual_approval_lifecycle,
        output.clone(),
        true,
    );
    opt.credential_material = "production_live_alpha".to_string();

    let production_api_key = "ntpro_v151003_production_like_api_key_value";
    let production_api_secret = "ntpro_v151003_production_like_api_secret_value";
    run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| match name {
        PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
        | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
        "NTPRO_V150002_API_KEY" => Some(production_api_key.to_string()),
        "NTPRO_V150002_API_SECRET" => Some(production_api_secret.to_string()),
        _ => None,
    })
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains(production_api_key));
    assert!(!body.contains(production_api_secret));
    assert!(!body.contains("signature="));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(artifact["status"], "ready_request_preview_only");
    assert_eq!(artifact["credential_material"], "production_live_alpha");
    assert_eq!(artifact["production_signing_material_gate_required"], true);
    assert_eq!(artifact["production_signing_material_gate_open"], true);
    assert_eq!(artifact["production_signing_material_env_read"], true);
    assert_eq!(
        artifact["production_signing_material_missing_gate_env_vars"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["request_preview_built"], true);
    assert_eq!(artifact["signed_request_memory_only"], true);
    assert_eq!(artifact["secrets_redacted"], true);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
}

#[test]
fn production_live_alpha_order_request_preview_consumes_one_time_manual_approval() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v151-002-approval-consume-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let first_preview = output_dir.join("live_alpha_order_request_preview_first.json");
    let second_preview = output_dir.join("live_alpha_order_request_preview_second.json");
    let risk_input = output_dir.join("live_alpha_risk_input.json");
    let risk_preflight = output_dir.join("live_alpha_risk_preflight.json");
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let runtime_gate = output_dir.join("live_alpha_kill_switch_runtime_gate.json");
    let execution_output = output_dir.join("live_alpha_execution_dry_run.json");

    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();

    let first_opt = production_live_alpha_order_request_preview_opt(
        order_gate.clone(),
        manual_approval_lifecycle.clone(),
        first_preview.clone(),
        true,
    );
    run_live_production_live_alpha_order_request_preview_with_env(&first_opt, |name| match name {
        "NTPRO_V150002_API_KEY" => Some("ntpro_v151002_synthetic_api_key_value".to_string()),
        "NTPRO_V150002_API_SECRET" => Some("ntpro_v151002_synthetic_api_secret_value".to_string()),
        _ => None,
    })
    .unwrap();

    let first_artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&first_preview).unwrap()).unwrap();
    assert_eq!(first_artifact["status"], "ready_request_preview_only");
    assert_eq!(first_artifact["request_preview_built"], true);
    assert_eq!(first_artifact["manual_approval_consumed"], true);
    assert_eq!(first_artifact["manual_approval_used"], true);

    let consumed_approval: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manual_approval_lifecycle).unwrap()).unwrap();
    assert_eq!(consumed_approval["approval_state"], "used");
    assert_eq!(consumed_approval["approval_used"], true);
    assert_eq!(consumed_approval["request_preview_created"], true);
    assert_eq!(consumed_approval["approval_lifecycle_valid"], false);

    let second_opt = production_live_alpha_order_request_preview_opt(
        order_gate.clone(),
        manual_approval_lifecycle,
        second_preview.clone(),
        true,
    );
    run_live_production_live_alpha_order_request_preview_with_env(&second_opt, |name| match name {
        "NTPRO_V150002_API_KEY" => Some("ntpro_v151002_synthetic_api_key_value".to_string()),
        "NTPRO_V150002_API_SECRET" => Some("ntpro_v151002_synthetic_api_secret_value".to_string()),
        _ => None,
    })
    .unwrap();

    let second_artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&second_preview).unwrap()).unwrap();
    assert_eq!(
        second_artifact["status"],
        "blocked_manual_approval_lifecycle"
    );
    assert_eq!(second_artifact["request_preview_allowed"], false);
    assert_eq!(second_artifact["request_preview_built"], false);
    assert_eq!(second_artifact["manual_approval_lifecycle_valid"], false);
    assert_eq!(second_artifact["manual_approval_used"], true);
    assert_eq!(
        second_artifact["manual_approval_consume_status"],
        "approval_already_used"
    );
    assert!(
        second_artifact["manual_approval_lifecycle_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "manual_approval_used")
    );
    assert_eq!(second_artifact["request_sent"], false);
    assert_eq!(second_artifact["production_orders_submitted"], 0);
    assert_eq!(second_artifact["production_order_mutations_attempted"], 0);
    assert_eq!(second_artifact["network_attempted"], false);

    let mut risk = passing_live_alpha_risk_input();
    risk.order.order_type = "LIMIT".to_string();
    write_live_alpha_risk_input(&risk_input, &risk);
    run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
        order_gate.clone(),
        risk_input,
        risk_preflight.clone(),
        true,
    ))
    .unwrap();
    write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight.clone(),
            second_preview.clone(),
            runtime_gate.clone(),
            true,
        ),
    )
    .unwrap();
    run_live_production_live_alpha_execution_dry_run(&production_live_alpha_execution_dry_run_opt(
        order_gate,
        risk_preflight,
        second_preview,
        runtime_gate,
        execution_output.clone(),
        true,
    ))
    .unwrap();
    let execution: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(execution_output).unwrap()).unwrap();
    assert_eq!(execution["status"], "blocked_source_artifact");
    assert_eq!(execution["dry_run_execution_adapter_called"], false);
    assert_eq!(execution["production_adapter_called"], false);
    assert_eq!(execution["production_orders_submitted"], 0);
    assert_eq!(execution["production_order_mutations_attempted"], 0);
    assert_eq!(execution["network_attempted"], false);
    assert!(
        execution["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "request_preview_not_built")
    );
}

#[test]
fn production_live_alpha_order_request_preview_blocks_without_owner_scope() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v150-002-request-preview-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let output = output_dir.join("live_alpha_order_request_preview.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();

    let opt = production_live_alpha_order_request_preview_opt(
        order_gate,
        manual_approval_lifecycle,
        output.clone(),
        false,
    );
    run_live_production_live_alpha_order_request_preview_with_env(&opt, |_| None).unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_endpoint_or_owner_scope");
    assert_eq!(
        artifact["endpoint_class"],
        "production_mutation_scope_candidate"
    );
    assert_eq!(artifact["endpoint_decision"], "deny");
    assert_eq!(artifact["request_preview_allowed"], false);
    assert_eq!(artifact["request_preview_built"], false);
    assert_eq!(artifact["manual_approval_lifecycle_valid"], true);
    assert_eq!(artifact["signed_request_memory_only"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["execution_adapter_called"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 10);
    assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["credential_material"], "synthetic");
    assert_eq!(artifact["production_signing_material_env_read"], false);
}

#[test]
fn production_live_alpha_order_request_preview_blocks_invalid_manual_approval_lifecycle() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v150-005-approval-lifecycle-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();

    for case in [
        ManualApprovalLifecycleCase {
            name: "pending",
            approval_state: "pending",
            run_id: "v150-live-alpha-request-preview",
            symbol: "BTCUSDT",
            notional: "10.00",
            now_unix_ms: 1_718_400_000_000,
            expires_at_unix_ms: 1_718_400_060_000,
            expected_issue: "manual_approval_not_approved",
        },
        ManualApprovalLifecycleCase {
            name: "expired",
            approval_state: "expired",
            run_id: "v150-live-alpha-request-preview",
            symbol: "BTCUSDT",
            notional: "10.00",
            now_unix_ms: 1_718_400_070_000,
            expires_at_unix_ms: 1_718_400_060_000,
            expected_issue: "manual_approval_expired",
        },
        ManualApprovalLifecycleCase {
            name: "revoked",
            approval_state: "revoked",
            run_id: "v150-live-alpha-request-preview",
            symbol: "BTCUSDT",
            notional: "10.00",
            now_unix_ms: 1_718_400_000_000,
            expires_at_unix_ms: 1_718_400_060_000,
            expected_issue: "manual_approval_revoked",
        },
        ManualApprovalLifecycleCase {
            name: "used",
            approval_state: "used",
            run_id: "v150-live-alpha-request-preview",
            symbol: "BTCUSDT",
            notional: "10.00",
            now_unix_ms: 1_718_400_000_000,
            expires_at_unix_ms: 1_718_400_060_000,
            expected_issue: "manual_approval_used",
        },
        ManualApprovalLifecycleCase {
            name: "run-id-mismatch",
            approval_state: "approved",
            run_id: "wrong-run-id",
            symbol: "BTCUSDT",
            notional: "10.00",
            now_unix_ms: 1_718_400_000_000,
            expires_at_unix_ms: 1_718_400_060_000,
            expected_issue: "manual_approval_run_id_mismatch",
        },
        ManualApprovalLifecycleCase {
            name: "symbol-mismatch",
            approval_state: "approved",
            run_id: "v150-live-alpha-request-preview",
            symbol: "ETHUSDT",
            notional: "10.00",
            now_unix_ms: 1_718_400_000_000,
            expires_at_unix_ms: 1_718_400_060_000,
            expected_issue: "manual_approval_symbol_mismatch",
        },
        ManualApprovalLifecycleCase {
            name: "notional-mismatch",
            approval_state: "approved",
            run_id: "v150-live-alpha-request-preview",
            symbol: "BTCUSDT",
            notional: "11.00",
            now_unix_ms: 1_718_400_000_000,
            expires_at_unix_ms: 1_718_400_060_000,
            expected_issue: "manual_approval_notional_mismatch",
        },
    ] {
        let approval = output_dir.join(format!("manual_approval_{}.json", case.name));
        let output = output_dir.join(format!("request_preview_{}.json", case.name));
        run_live_production_live_alpha_manual_approval_lifecycle(
            &production_live_alpha_manual_approval_lifecycle_opt(
                approval.clone(),
                &ManualApprovalLifecycleFixture {
                    approval_state: case.approval_state,
                    run_id: case.run_id,
                    strategy_id: "ema_cross_btcusdt_v1",
                    symbol: case.symbol,
                    notional: case.notional,
                    now_unix_ms: case.now_unix_ms,
                    expires_at_unix_ms: case.expires_at_unix_ms,
                },
            ),
        )
        .unwrap();
        let opt = production_live_alpha_order_request_preview_opt(
            order_gate.clone(),
            approval,
            output.clone(),
            true,
        );
        run_live_production_live_alpha_order_request_preview_with_env(&opt, |name| match name {
            "NTPRO_V150002_API_KEY" => Some("ntpro_v150005_synthetic_api_key_value".to_string()),
            "NTPRO_V150002_API_SECRET" => {
                Some("ntpro_v150005_synthetic_api_secret_value".to_string())
            }
            _ => None,
        })
        .unwrap();

        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_manual_approval_lifecycle");
        assert_eq!(artifact["manual_approval_lifecycle_valid"], false);
        assert_eq!(artifact["request_preview_allowed"], false);
        assert_eq!(artifact["request_preview_built"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["production_orders_submitted"], 0);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["network_attempted"], false);
        assert!(
            artifact["manual_approval_lifecycle_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == case.expected_issue),
            "missing {} in {:?}",
            case.expected_issue,
            artifact["manual_approval_lifecycle_issues"]
        );
    }
}

#[test]
fn production_live_alpha_execution_dry_run_routes_only_to_local_dry_run_adapter() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v150-003-execution-dry-run-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (order_gate, risk_preflight, request_preview) =
        write_ready_live_alpha_artifact_chain(&output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
    let output = output_dir.join("live_alpha_execution_dry_run.json");

    run_live_production_kill_switch_approval_artifact(
        &LiveProductionKillSwitchApprovalArtifactOpt {
            run_id: "v150-live-alpha-kill-switch-runtime-gate".to_string(),
            session_id: Some("session-v150".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            output: kill_switch_approval.clone(),
            kill_switch_active: false,
            approval_state: "approved".to_string(),
            manual_approval_id: Some("owner-approval-v150-004".to_string()),
            approved_by: Some("owner".to_string()),
            confirm_dry_run_only: true,
            confirm_no_production_mutation: true,
            confirm_dashboard_order_controls_disabled: true,
        },
    )
    .unwrap();
    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight.clone(),
            request_preview.clone(),
            kill_switch_runtime_gate.clone(),
            true,
        ),
    )
    .unwrap();

    run_live_production_live_alpha_execution_dry_run(&production_live_alpha_execution_dry_run_opt(
        order_gate,
        risk_preflight,
        request_preview,
        kill_switch_runtime_gate,
        output.clone(),
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_LIVE_ALPHA_EXECUTION_DRY_RUN_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "ready_dry_run_execution_adapter_only");
    assert_eq!(
        artifact["execution_decision"],
        "dry_run_adapter_artifact_only"
    );
    assert_eq!(
        artifact["execution_boundary_contract_version"],
        "ntpro.v151_execution_dry_run_adapter_boundary.v1"
    );
    assert_eq!(
        artifact["execution_boundary_flow"],
        "StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter"
    );
    assert_eq!(artifact["execution_boundary_contract_ready"], true);
    assert_eq!(artifact["strategy_intent_boundary"], "StrategyIntent");
    assert_eq!(artifact["risk_decision_boundary"], "RiskDecision");
    assert_eq!(artifact["execution_command_boundary"], "ExecutionCommand");
    assert_eq!(artifact["execution_command_created"], true);
    assert_eq!(artifact["execution_command_route"], "dry_run_adapter_only");
    assert_eq!(
        artifact["execution_command_destination"],
        "ntpro_local_artifact_dry_run_execution_adapter"
    );
    assert_eq!(
        artifact["dry_run_adapter_boundary"],
        "DryRunExecutionAdapter"
    );
    assert_eq!(artifact["dry_run_adapter_route_allowed"], true);
    assert_eq!(
        artifact["production_adapter_boundary"],
        "ProductionExecutionAdapter"
    );
    assert_eq!(artifact["production_adapter_route_allowed"], false);
    assert_eq!(artifact["production_adapter_instantiation_allowed"], false);
    assert_eq!(artifact["dry_run_execution_adapter_called"], true);
    assert_eq!(artifact["dry_run_execution_adapter_wrote_artifact"], true);
    assert_eq!(artifact["dry_run_adapter_artifact_only"], true);
    assert_eq!(artifact["real_execution_adapter_called"], false);
    assert_eq!(artifact["production_adapter_instantiated"], false);
    assert_eq!(artifact["production_adapter_called"], false);
    assert_eq!(artifact["strategy_intent_recorded"], true);
    assert_eq!(artifact["strategy_intent_reaches_risk_preflight"], true);
    assert_eq!(artifact["strategy_intent_reaches_dry_run_adapter"], true);
    assert_eq!(
        artifact["strategy_intent_reaches_production_adapter"],
        false
    );
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["order_gate_ready"], true);
    assert_eq!(artifact["risk_preflight_decision"], "dry_run_approved");
    assert_eq!(artifact["request_preview_built"], true);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(
        artifact["kill_switch_runtime_gate_status"],
        "ready_runtime_gate_open_for_dry_run_only"
    );
    assert_eq!(artifact["kill_switch_runtime_gate_open"], true);
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["side"], "BUY");
    assert_eq!(artifact["order_type"], "LIMIT");
    assert_eq!(artifact["quantity"], "0.001");
    assert_eq!(artifact["price"], "10000.00");
    assert_eq!(artifact["time_in_force"], "GTC");
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["order_endpoint_access_attempted"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["real_funds"], false);
    assert_eq!(artifact["production_trading_enabled"], false);
    assert_eq!(artifact["values_are_exchange_truth"], false);
}

#[test]
fn production_live_alpha_execution_dry_run_blocks_without_owner_scope() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v150-003-execution-dry-run-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (order_gate, risk_preflight, request_preview) =
        write_ready_live_alpha_artifact_chain(&output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
    let output = output_dir.join("live_alpha_execution_dry_run.json");

    run_live_production_kill_switch_approval_artifact(
        &LiveProductionKillSwitchApprovalArtifactOpt {
            run_id: "v150-live-alpha-kill-switch-runtime-gate".to_string(),
            session_id: Some("session-v150".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            output: kill_switch_approval.clone(),
            kill_switch_active: false,
            approval_state: "approved".to_string(),
            manual_approval_id: Some("owner-approval-v150-004".to_string()),
            approved_by: Some("owner".to_string()),
            confirm_dry_run_only: true,
            confirm_no_production_mutation: true,
            confirm_dashboard_order_controls_disabled: true,
        },
    )
    .unwrap();
    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight.clone(),
            request_preview.clone(),
            kill_switch_runtime_gate.clone(),
            true,
        ),
    )
    .unwrap();

    run_live_production_live_alpha_execution_dry_run(&production_live_alpha_execution_dry_run_opt(
        order_gate,
        risk_preflight,
        request_preview,
        kill_switch_runtime_gate,
        output.clone(),
        false,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["execution_decision"], "blocked_no_adapter_route");
    assert_eq!(
        artifact["execution_boundary_contract_version"],
        "ntpro.v151_execution_dry_run_adapter_boundary.v1"
    );
    assert_eq!(artifact["execution_boundary_contract_ready"], false);
    assert_eq!(artifact["execution_command_created"], false);
    assert_eq!(
        artifact["execution_command_route"],
        "blocked_before_execution_command"
    );
    assert_eq!(artifact["execution_command_destination"], "none");
    assert_eq!(artifact["dry_run_adapter_route_allowed"], false);
    assert_eq!(artifact["production_adapter_route_allowed"], false);
    assert_eq!(artifact["production_adapter_instantiation_allowed"], false);
    assert_eq!(artifact["dry_run_execution_adapter_called"], false);
    assert_eq!(artifact["dry_run_execution_adapter_wrote_artifact"], false);
    assert_eq!(artifact["dry_run_adapter_artifact_only"], false);
    assert_eq!(artifact["production_adapter_instantiated"], false);
    assert_eq!(artifact["production_adapter_called"], false);
    assert_eq!(
        artifact["strategy_intent_reaches_production_adapter"],
        false
    );
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 10);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
}

#[test]
fn production_live_alpha_kill_switch_runtime_gate_blocks_active_switch() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v150-004-kill-switch-active-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (_, risk_preflight, request_preview) = write_ready_live_alpha_artifact_chain(&output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let output = output_dir.join("kill_switch_runtime_gate.json");
    write_kill_switch_approval_artifact(kill_switch_approval.clone(), true, "approved");

    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight,
            request_preview,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "blocked_kill_switch_active");
    assert_eq!(
        artifact["runtime_gate_decision"],
        "blocked_no_runtime_mutation"
    );
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["kill_switch_active"], true);
    assert_eq!(artifact["manual_approval_recorded"], true);
    assert_eq!(artifact["request_preview_built"], true);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert!(
        artifact["runtime_gate_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "kill_switch_active")
    );
}

#[test]
fn production_live_alpha_kill_switch_runtime_gate_blocks_missing_approval() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v150-004-kill-switch-missing-approval-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (_, risk_preflight, request_preview) = write_ready_live_alpha_artifact_chain(&output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let output = output_dir.join("kill_switch_runtime_gate.json");
    write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "pending");

    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight,
            request_preview,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_manual_approval");
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["kill_switch_active"], false);
    assert_eq!(artifact["approval_state"], "pending");
    assert_eq!(artifact["manual_approval_recorded"], false);
    assert_eq!(artifact["request_preview_built"], true);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert!(
        artifact["runtime_gate_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "manual_approval_missing_or_not_approved")
    );
}

#[test]
fn production_live_alpha_kill_switch_runtime_gate_blocks_request_preview() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v150-004-kill-switch-blocked-preview-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("live_alpha_dry_run_order_gate.json");
    let risk_input = output_dir.join("live_alpha_risk_input.json");
    let risk_preflight = output_dir.join("live_alpha_risk_preflight.json");
    let manual_approval_lifecycle = output_dir.join("manual_approval_lifecycle.json");
    let request_preview = output_dir.join("live_alpha_order_request_preview.json");
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let output = output_dir.join("kill_switch_runtime_gate.json");

    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_limit_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    let mut input = passing_live_alpha_risk_input();
    input.order.order_type = "LIMIT".to_string();
    write_live_alpha_risk_input(&risk_input, &input);
    run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
        order_gate.clone(),
        risk_input,
        risk_preflight.clone(),
        true,
    ))
    .unwrap();
    run_live_production_live_alpha_manual_approval_lifecycle(
        &production_live_alpha_manual_approval_lifecycle_opt(
            manual_approval_lifecycle.clone(),
            &ManualApprovalLifecycleFixture {
                approval_state: "approved",
                run_id: "v150-live-alpha-request-preview",
                strategy_id: "ema_cross_btcusdt_v1",
                symbol: "BTCUSDT",
                notional: "10.00",
                now_unix_ms: 1_718_400_000_000,
                expires_at_unix_ms: 1_718_400_060_000,
            },
        ),
    )
    .unwrap();
    let mut request_opt = production_live_alpha_order_request_preview_opt(
        order_gate,
        manual_approval_lifecycle,
        request_preview.clone(),
        true,
    );
    request_opt.credential_material = "production_live_alpha".to_string();
    run_live_production_live_alpha_order_request_preview_with_env(
        &request_opt,
        |name| match name {
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
            | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => None,
            "NTPRO_V150002_API_KEY" | "NTPRO_V150002_API_SECRET" => {
                panic!("blocked production signing material must not read {name}")
            }
            _ => None,
        },
    )
    .unwrap();
    write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");

    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight,
            request_preview,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_request_preview");
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["kill_switch_active"], false);
    assert_eq!(artifact["manual_approval_recorded"], true);
    assert_eq!(
        artifact["request_preview_status"],
        "blocked_endpoint_or_owner_scope"
    );
    assert_eq!(artifact["request_preview_built"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert!(
        artifact["runtime_gate_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "request_preview_blocked")
    );
}

#[test]
fn production_mutation_runtime_gate_blocks_missing_signing_approval() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-002-runtime-gate-signing-missing-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (order_gate, risk_preflight, request_preview) =
        write_ready_live_alpha_artifact_chain(&output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
    let output = output_dir.join("production_mutation_runtime_gate.json");
    write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight.clone(),
            request_preview.clone(),
            kill_switch_runtime_gate.clone(),
            true,
        ),
    )
    .unwrap();

    run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
        order_gate,
        risk_preflight,
        request_preview,
        kill_switch_runtime_gate,
        None,
        output.clone(),
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_RUNTIME_GATE_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "blocked_signing_approval");
    assert_eq!(
        artifact["capability"],
        "Minimum Owner-Approved Production Order Mutation Candidate"
    );
    assert_eq!(artifact["capability_expansion_from_v15"], true);
    assert_eq!(artifact["default_fail_closed"], true);
    assert_eq!(
        artifact["runtime_gate_decision"],
        "blocked_before_any_send_consideration"
    );
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["send_consideration_allowed"], false);
    assert_eq!(artifact["owner_approval_required"], true);
    assert_eq!(artifact["owner_approval_consumed"], true);
    assert_eq!(artifact["manual_approval_consumed"], true);
    assert_eq!(
        artifact["manual_approval_consume_status"],
        "approval_consumed_after_request_preview_created"
    );
    assert_eq!(artifact["kill_switch_checked_before_send"], true);
    assert_eq!(artifact["kill_switch_runtime_gate_open"], true);
    assert_eq!(artifact["kill_switch_active"], false);
    assert_eq!(artifact["risk_preflight_decision"], "dry_run_approved");
    assert_eq!(artifact["request_preview_built"], true);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["signing_approval_required"], true);
    assert_eq!(artifact["signing_approval_ready"], false);
    assert_eq!(artifact["explicit_send_gate_required"], true);
    assert_eq!(artifact["explicit_send_gate_open"], false);
    assert_eq!(artifact["single_order_candidate"], true);
    assert_eq!(artifact["tiny_notional_gate_ready"], true);
    assert_eq!(artifact["order_type"], "LIMIT");
    assert_eq!(artifact["time_in_force"], "GTC");
    assert_eq!(artifact["notional"], "10.00");
    assert_eq!(
        artifact["production_order_submission_allowed_policy"],
        "owner_approved_single_limit_gtc_only"
    );
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["replace_attempted"], false);
    assert_eq!(artifact["amend_attempted"], false);
    assert_eq!(artifact["flatten_attempted"], false);
    assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert!(
        artifact["runtime_gate_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "signing_approval_missing")
    );
    assert!(
        artifact["runtime_gate_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "explicit_send_gate_closed")
    );
}

#[test]
fn production_mutation_signing_approval_ready_for_production_material_preview() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-003-signing-approval-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (_, _, request_preview) =
        write_ready_live_alpha_production_material_artifact_chain(&output_dir);
    let output = output_dir.join("production_mutation_signing_approval.json");

    run_live_production_mutation_signing_approval(&production_mutation_signing_approval_opt(
        request_preview,
        output.clone(),
        true,
    ))
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v160003_production_like_api_key_value"));
    assert!(!body.contains("ntpro_v160003_production_like_api_secret_value"));
    assert!(!body.contains("signature="));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "ready_signing_material_approval");
    assert_eq!(artifact["credential_material"], "production_live_alpha");
    assert_eq!(artifact["approval_state"], "approved");
    assert_eq!(artifact["manual_approval_recorded"], true);
    assert_eq!(artifact["owner_approval_required"], true);
    assert_eq!(artifact["owner_approved_signing_material"], true);
    assert_eq!(artifact["signing_approval_ready"], true);
    assert_eq!(artifact["production_signing_material_gate_required"], true);
    assert_eq!(artifact["production_signing_material_gate_open"], true);
    assert_eq!(artifact["production_signing_material_env_read"], true);
    assert_eq!(
        artifact["production_signing_material_missing_gate_env_vars"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["request_body_recorded"], false);
    assert_eq!(artifact["raw_request_body_recorded"], false);
    assert_eq!(artifact["request_preview_built"], true);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["real_funds"], false);
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
}

#[test]
fn production_mutation_signing_approval_blocks_synthetic_preview() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-003-signing-approval-synthetic-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (_, _, request_preview) = write_ready_live_alpha_artifact_chain(&output_dir);
    let output = output_dir.join("production_mutation_signing_approval.json");

    run_live_production_mutation_signing_approval(&production_mutation_signing_approval_opt(
        request_preview,
        output.clone(),
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_request_preview");
    assert_eq!(artifact["credential_material"], "synthetic");
    assert_eq!(artifact["signing_approval_ready"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "request_preview_not_production_live_alpha_material")
    );
}

#[test]
fn production_mutation_runtime_gate_accepts_ready_signing_approval_but_blocks_send_gate() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-003-runtime-gate-signing-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (order_gate, risk_preflight, request_preview) =
        write_ready_live_alpha_production_material_artifact_chain(&output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
    let signing_approval = output_dir.join("production_mutation_signing_approval.json");
    let output = output_dir.join("production_mutation_runtime_gate.json");
    write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight.clone(),
            request_preview.clone(),
            kill_switch_runtime_gate.clone(),
            true,
        ),
    )
    .unwrap();
    run_live_production_mutation_signing_approval(&production_mutation_signing_approval_opt(
        request_preview.clone(),
        signing_approval.clone(),
        true,
    ))
    .unwrap();

    run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
        order_gate,
        risk_preflight,
        request_preview,
        kill_switch_runtime_gate,
        Some(signing_approval.clone()),
        output.clone(),
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_explicit_send_gate");
    assert_eq!(artifact["signing_approval_required"], true);
    assert_eq!(artifact["signing_approval_ready"], true);
    assert_eq!(
        artifact["signing_approval_status"],
        "ready_signing_material_approval"
    );
    assert_eq!(
        artifact["source_signing_approval_path"],
        signing_approval.display().to_string()
    );
    assert_eq!(artifact["explicit_send_gate_required"], true);
    assert_eq!(artifact["explicit_send_gate_open"], false);
    assert_eq!(
        artifact["runtime_gate_decision"],
        "blocked_before_any_send_consideration"
    );
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["send_consideration_allowed"], false);
    assert_eq!(artifact["single_order_candidate"], true);
    assert_eq!(artifact["tiny_notional_gate_ready"], true);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["execution_adapter_called"], false);
    assert_eq!(artifact["production_adapter_called"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert!(
        artifact["runtime_gate_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "explicit_send_gate_closed")
    );
}

#[test]
fn production_mutation_request_builder_builds_redacted_limit_gtc_object_without_send() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-004-request-builder-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (runtime_gate, signing_approval, request_preview) =
        write_ready_v160_request_builder_sources(&output_dir);
    let output = output_dir.join("production_mutation_request_builder.json");
    let production_api_key = "ntpro_v160003_production_like_api_key_value";
    let production_api_secret = "ntpro_v160003_production_like_api_secret_value";

    run_live_production_mutation_request_builder_with_env(
        &production_mutation_request_builder_opt(
            runtime_gate,
            signing_approval,
            request_preview,
            output.clone(),
            true,
        ),
        |name| match name {
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
            | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
            "NTPRO_V150002_API_KEY" => Some(production_api_key.to_string()),
            "NTPRO_V150002_API_SECRET" => Some(production_api_secret.to_string()),
            _ => None,
        },
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains(production_api_key));
    assert!(!body.contains(production_api_secret));
    assert!(!body.contains("symbol=BTCUSDT"));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "ready_request_object_built_no_send");
    assert_eq!(artifact["request_builder_ready"], true);
    assert_eq!(artifact["request_object_built"], true);
    assert_eq!(
        artifact["runtime_gate_status"],
        "blocked_explicit_send_gate"
    );
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["send_consideration_allowed"], false);
    assert_eq!(
        artifact["signing_approval_status"],
        "ready_signing_material_approval"
    );
    assert_eq!(artifact["signing_approval_ready"], true);
    assert_eq!(artifact["explicit_send_gate_open"], false);
    assert_eq!(artifact["credential_material"], "production_live_alpha");
    assert_eq!(artifact["production_signing_material_gate_required"], true);
    assert_eq!(artifact["production_signing_material_gate_open"], true);
    assert_eq!(artifact["production_signing_material_env_read"], true);
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["api_key_header_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["request_body_recorded"], false);
    assert_eq!(artifact["raw_request_body_recorded"], false);
    assert_eq!(artifact["request_method"], "POST");
    assert_eq!(artifact["request_target"], TESTNET_ORDER_ENDPOINT_ORDER);
    assert_eq!(
        artifact["query_shape_without_signature"],
        "symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp"
    );
    assert_eq!(
        artifact["signed_query_shape"],
        "symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp&signature=<redacted>"
    );
    assert_eq!(artifact["order_type"], "LIMIT");
    assert_eq!(artifact["time_in_force"], "GTC");
    assert_eq!(artifact["single_order_candidate"], true);
    assert_eq!(artifact["tiny_notional_gate_ready"], true);
    assert_eq!(artifact["market_reference_source"], "fixture_mid_price");
    assert_eq!(artifact["market_reference_price"], "10001.00");
    assert_eq!(artifact["max_reference_price_distance_bps"], "50");
    assert_ne!(artifact["price_distance_from_reference_bps"], "unavailable");
    assert_eq!(artifact["would_cross_spread"], false);
    assert_eq!(artifact["non_marketable_price_preflight_ready"], true);
    assert_eq!(artifact["owner_acknowledged_no_cancel_path"], true);
    assert_eq!(artifact["price_safety_send_consideration_allowed"], true);
    assert_eq!(artifact["manual_review_required"], false);
    assert_eq!(artifact["new_orders_blocked"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
}

#[test]
fn production_mutation_request_builder_blocks_missing_confirmations_and_env_gates() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-004-request-builder-missing-gates-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (runtime_gate, signing_approval, request_preview) =
        write_ready_v160_request_builder_sources(&output_dir);
    let output = output_dir.join("production_mutation_request_builder.json");

    run_live_production_mutation_request_builder_with_env(
        &production_mutation_request_builder_opt(
            runtime_gate,
            signing_approval,
            request_preview,
            output.clone(),
            false,
        ),
        |_| None,
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["request_builder_ready"], false);
    assert_eq!(artifact["request_object_built"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-request-builder")
    );
    assert!(
        artifact["missing_env_vars"]
            .as_array()
            .unwrap()
            .iter()
            .any(|env| env == PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW)
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-owner-acknowledged-no-cancel-path")
    );
}

#[test]
fn production_mutation_request_builder_rejects_non_limit_request_preview() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-004-request-builder-market-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (runtime_gate, signing_approval, request_preview) =
        write_ready_v160_request_builder_sources(&output_dir);
    let mut preview: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&request_preview).unwrap()).unwrap();
    preview["order_type"] = serde_json::Value::String("MARKET".to_string());
    fs::write(
        &request_preview,
        serde_json::to_string_pretty(&preview).unwrap(),
    )
    .unwrap();
    let output = output_dir.join("production_mutation_request_builder.json");

    run_live_production_mutation_request_builder_with_env(
        &production_mutation_request_builder_opt(
            runtime_gate,
            signing_approval,
            request_preview,
            output.clone(),
            true,
        ),
        |name| match name {
            PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
            | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
            "NTPRO_V150002_API_KEY" => Some("ntpro_v160004_api_key".to_string()),
            "NTPRO_V150002_API_SECRET" => Some("ntpro_v160004_api_secret".to_string()),
            _ => None,
        },
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_source_artifact");
    assert_eq!(artifact["request_builder_ready"], false);
    assert_eq!(artifact["request_object_built"], false);
    assert_eq!(artifact["order_type"], "MARKET");
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "request_preview_not_limit")
    );
}

#[test]
fn production_mutation_request_builder_blocks_missing_market_reference() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v161-004-request-builder-missing-reference-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (runtime_gate, signing_approval, request_preview) =
        write_ready_v160_request_builder_sources(&output_dir);
    let output = output_dir.join("production_mutation_request_builder.json");
    let mut opt = production_mutation_request_builder_opt(
        runtime_gate,
        signing_approval,
        request_preview,
        output.clone(),
        true,
    );
    opt.market_reference_source = String::new();

    run_live_production_mutation_request_builder_with_env(&opt, |name| match name {
        PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
        | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
        "NTPRO_V150002_API_KEY" => Some("ntpro_v161004_api_key".to_string()),
        "NTPRO_V150002_API_SECRET" => Some("ntpro_v161004_api_secret".to_string()),
        _ => None,
    })
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_source_artifact");
    assert_eq!(artifact["request_builder_ready"], false);
    assert_eq!(artifact["request_object_built"], false);
    assert_eq!(artifact["non_marketable_price_preflight_ready"], false);
    assert_eq!(artifact["price_safety_send_consideration_allowed"], false);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "market_reference_source_missing")
    );
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
}

#[test]
fn production_mutation_request_builder_blocks_crossing_limit_price() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v161-004-request-builder-crossing-price-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (runtime_gate, signing_approval, request_preview) =
        write_ready_v160_request_builder_sources(&output_dir);
    let output = output_dir.join("production_mutation_request_builder.json");
    let mut opt = production_mutation_request_builder_opt(
        runtime_gate,
        signing_approval,
        request_preview,
        output.clone(),
        true,
    );
    opt.would_cross_spread = true;

    run_live_production_mutation_request_builder_with_env(&opt, |name| match name {
        PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
        | PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED => Some("1".to_string()),
        "NTPRO_V150002_API_KEY" => Some("ntpro_v161004_api_key".to_string()),
        "NTPRO_V150002_API_SECRET" => Some("ntpro_v161004_api_secret".to_string()),
        _ => None,
    })
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_source_artifact");
    assert_eq!(artifact["request_builder_ready"], false);
    assert_eq!(artifact["request_object_built"], false);
    assert_eq!(artifact["would_cross_spread"], true);
    assert_eq!(artifact["non_marketable_price_preflight_ready"], false);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "limit_price_would_cross_spread")
    );
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
}

#[test]
fn production_mutation_guarded_send_offline_ready_without_network() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-005-guarded-send-ready-offline-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (request_builder, request_preview, kill_switch_runtime_gate) =
        write_ready_v160_guarded_send_sources(&output_dir);
    let output = output_dir.join("production_mutation_guarded_send.json");

    run_live_production_mutation_guarded_send(&production_mutation_guarded_send_opt(
        request_builder,
        kill_switch_runtime_gate,
        request_preview,
        output.clone(),
        false,
        true,
    ))
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v160005_production_like_api_key_value"));
    assert!(!body.contains("ntpro_v160005_production_like_api_secret_value"));
    assert!(!body.contains("symbol=BTCUSDT"));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["status"],
        "ready_guarded_send_path_offline_no_network"
    );
    assert_eq!(artifact["manual_online_requested"], false);
    assert_eq!(artifact["guarded_send_ready"], true);
    assert_eq!(artifact["send_path_evaluated"], true);
    assert_eq!(artifact["kill_switch_enforcement_ready"], true);
    assert_eq!(artifact["kill_switch_checked_before_send"], true);
    assert_eq!(artifact["kill_switch_checked_after_send"], true);
    assert_eq!(
        artifact["pre_send_kill_switch_snapshot_source"],
        artifact["post_send_kill_switch_snapshot_source"]
    );
    assert_eq!(
        artifact["pre_send_kill_switch_snapshot_hash"],
        artifact["post_send_kill_switch_snapshot_hash"]
    );
    assert!(
        artifact["pre_send_kill_switch_checked_at"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(
        artifact["post_send_kill_switch_checked_at"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_eq!(artifact["pre_send_kill_switch_runtime_gate_open"], true);
    assert_eq!(artifact["pre_send_kill_switch_active"], false);
    assert_eq!(artifact["post_send_kill_switch_runtime_gate_open"], true);
    assert_eq!(artifact["post_send_kill_switch_active"], false);
    assert_eq!(artifact["post_send_kill_switch_clean"], true);
    assert_eq!(artifact["kill_switch_blocked_send"], false);
    assert_eq!(artifact["post_send_progression_blocked"], false);
    assert_eq!(artifact["manual_review_required"], false);
    assert_eq!(artifact["new_orders_blocked"], false);
    assert_eq!(artifact["single_shot_send_allowed"], false);
    assert_eq!(
        artifact["request_builder_status"],
        "ready_request_object_built_no_send"
    );
    assert_eq!(artifact["request_object_built"], true);
    assert_eq!(artifact["request_method"], "POST");
    assert_eq!(artifact["request_target"], TESTNET_ORDER_ENDPOINT_ORDER);
    assert_eq!(artifact["order_type"], "LIMIT");
    assert_eq!(artifact["time_in_force"], "GTC");
    assert_eq!(artifact["mode"], "retired_guarded_send_offline_evaluation");
    assert_eq!(
        artifact["capability"],
        "Historical Production Mutation Artifact Evaluation"
    );
    assert_eq!(artifact["credential_material"], "retired_not_read");
    assert_eq!(artifact["api_key_env"], "retired");
    assert_eq!(artifact["api_secret_env"], "retired");
    assert_eq!(artifact["production_signing_material_gate_required"], false);
    assert_eq!(artifact["production_signing_material_gate_open"], false);
    assert_eq!(artifact["production_signing_material_env_read"], false);
    assert!(
        artifact["production_signing_material_missing_gate_env_vars"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "production_mutation_executor_retired_after_v0.32.0")
    );
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["api_key_header_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["request_body_recorded"], false);
    assert_eq!(artifact["raw_request_body_recorded"], false);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_body_recorded"], false);
    assert_eq!(artifact["response_redacted"], true);
    assert_eq!(artifact["error_code"], "not_attempted_executor_retired");
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_order_request_attempted"], false);
    assert_eq!(artifact["http_send_attempted"], false);
    assert_eq!(artifact["exchange_ack_observed"], false);
    assert_eq!(artifact["exchange_order_id_observed"], false);
    assert_eq!(artifact["exchange_order_status_observed"], false);
    assert_eq!(artifact["confirmed_production_order_submission"], false);
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_state_reads_allowed"], false);
    assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["replace_attempted"], false);
    assert_eq!(artifact["amend_attempted"], false);
    assert_eq!(artifact["flatten_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["real_funds"], false);
    assert_eq!(artifact["platform_production_trading_enabled"], false);
    assert_eq!(artifact["production_trading_enabled"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
}

#[test]
fn production_mutation_guarded_send_historical_online_selector_cannot_enable_network() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-backend-freeze-guarded-send-retired-online-selector-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (request_builder, request_preview, kill_switch_runtime_gate) =
        write_ready_v160_guarded_send_sources(&output_dir);
    let output = output_dir.join("production_mutation_guarded_send.json");

    run_live_production_mutation_guarded_send(&production_mutation_guarded_send_opt(
        request_builder,
        kill_switch_runtime_gate,
        request_preview,
        output.clone(),
        true,
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["manual_online_requested"], false);
    assert_eq!(artifact["single_shot_send_allowed"], false);
    assert_eq!(artifact["production_signing_material_env_read"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["http_send_attempted"], false);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
}

#[test]
fn production_mutation_guarded_send_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-005-guarded-send-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (request_builder, request_preview, kill_switch_runtime_gate) =
        write_ready_v160_guarded_send_sources(&output_dir);
    let output = output_dir.join("production_mutation_guarded_send.json");

    run_live_production_mutation_guarded_send(&production_mutation_guarded_send_opt(
        request_builder,
        kill_switch_runtime_gate,
        request_preview,
        output.clone(),
        false,
        false,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["guarded_send_ready"], false);
    assert_eq!(artifact["kill_switch_enforcement_ready"], true);
    assert_eq!(artifact["kill_switch_blocked_send"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_order_request_attempted"], false);
    assert_eq!(artifact["http_send_attempted"], false);
    assert_eq!(artifact["exchange_ack_observed"], false);
    assert_eq!(artifact["confirmed_production_order_submission"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_trading_enabled"], false);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-guarded-send")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-owner-approved-guarded-send")
    );
}

#[test]
fn production_mutation_guarded_send_blocks_active_kill_switch() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-008-guarded-send-kill-switch-active-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (request_builder, request_preview, kill_switch_runtime_gate) =
        write_ready_v160_guarded_send_sources(&output_dir);
    let mut gate: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&kill_switch_runtime_gate).unwrap()).unwrap();
    gate["status"] = serde_json::Value::String("blocked_kill_switch_active".to_string());
    gate["runtime_gate_open"] = serde_json::Value::Bool(false);
    gate["kill_switch_active"] = serde_json::Value::Bool(true);
    fs::write(
        &kill_switch_runtime_gate,
        serde_json::to_string_pretty(&gate).unwrap(),
    )
    .unwrap();
    let output = output_dir.join("production_mutation_guarded_send.json");

    run_live_production_mutation_guarded_send(&production_mutation_guarded_send_opt(
        request_builder,
        kill_switch_runtime_gate,
        request_preview,
        output.clone(),
        false,
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_kill_switch_enforcement");
    assert_eq!(artifact["guarded_send_ready"], false);
    assert_eq!(artifact["kill_switch_enforcement_ready"], false);
    assert_eq!(artifact["kill_switch_checked_before_send"], true);
    assert_eq!(artifact["kill_switch_checked_after_send"], true);
    assert_eq!(artifact["pre_send_kill_switch_runtime_gate_open"], false);
    assert_eq!(artifact["pre_send_kill_switch_active"], true);
    assert_eq!(artifact["post_send_kill_switch_runtime_gate_open"], false);
    assert_eq!(artifact["post_send_kill_switch_active"], true);
    assert_eq!(artifact["post_send_kill_switch_clean"], false);
    assert_eq!(artifact["kill_switch_blocked_send"], true);
    assert_eq!(artifact["post_send_progression_blocked"], true);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert_eq!(artifact["single_shot_send_allowed"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_order_request_attempted"], false);
    assert_eq!(artifact["http_send_attempted"], false);
    assert_eq!(artifact["exchange_ack_observed"], false);
    assert_eq!(artifact["confirmed_production_order_submission"], false);
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_trading_enabled"], false);
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "kill_switch_runtime_gate_not_open")
    );
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "kill_switch_active_before_send")
    );
}

#[test]
fn production_mutation_guarded_send_has_no_http_executor_boundary() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-par007-guarded-send-no-http-executor-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (request_builder, request_preview, kill_switch_runtime_gate) =
        write_ready_v160_guarded_send_sources(&output_dir);
    let opt = production_mutation_guarded_send_opt(
        request_builder,
        kill_switch_runtime_gate,
        request_preview,
        output_dir.join("production_mutation_guarded_send.json"),
        true,
        true,
    );
    let artifact = build_production_mutation_guarded_send_artifact(&opt).unwrap();

    assert_eq!(artifact.mode, "retired_guarded_send_offline_evaluation");
    assert!(artifact.guarded_send_ready);
    assert!(artifact.kill_switch_enforcement_ready);
    assert!(!artifact.manual_online_requested);
    assert!(!artifact.single_shot_send_allowed);
    assert!(!artifact.production_signing_material_env_read);
    assert!(!artifact.request_sent);
    assert!(!artifact.network_attempted);
    assert!(!artifact.http_send_attempted);
    assert!(!artifact.exchange_ack_observed);
    assert!(!artifact.confirmed_production_order_submission);
    assert_eq!(artifact.production_orders_submitted, 0);
    assert_eq!(artifact.production_order_mutations_attempted, 0);
    assert!(artifact.pre_send_kill_switch_runtime_gate_open);
    assert!(!artifact.pre_send_kill_switch_active);
    assert!(artifact.post_send_kill_switch_runtime_gate_open);
    assert!(!artifact.post_send_kill_switch_active);
    assert_eq!(
        artifact.pre_send_kill_switch_snapshot_hash,
        artifact.post_send_kill_switch_snapshot_hash
    );
    assert!(artifact.post_send_kill_switch_clean);
    assert!(!artifact.kill_switch_blocked_send);
    assert!(!artifact.post_send_progression_blocked);
    assert!(!artifact.manual_review_required);
    assert!(!artifact.new_orders_blocked);
    assert!(artifact.source_artifact_issues.is_empty());
    assert!(!artifact.retry_attempted);
    assert!(!artifact.cancel_attempted);
    assert!(!artifact.replace_attempted);
    assert!(!artifact.amend_attempted);
    assert!(!artifact.flatten_attempted);
    assert!(!artifact.dashboard_order_controls_enabled);
}

#[test]
fn production_mutation_response_redaction_persists_allowed_order_metadata_only() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-006-response-redaction-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let guarded_send = write_ready_v160_guarded_send_artifact(&output_dir);
    let response = output_dir.join("synthetic_order_response.json");
    let output = output_dir.join("production_mutation_response_redaction.json");
    write_synthetic_production_mutation_response(&response);

    run_live_production_mutation_response_redaction(&production_mutation_response_redaction_opt(
        guarded_send.clone(),
        response,
        output.clone(),
        true,
    ))
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("signature=must_not_persist"));
    assert!(!body.contains("\"headers\""));
    assert!(!body.contains("\"payload\""));
    assert!(!body.contains("\"balances\""));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "ready_response_redacted");
    assert_eq!(artifact["response_redaction_ready"], true);
    assert_eq!(artifact["response_redaction_source"], "synthetic_fixture");
    assert_eq!(
        artifact["source_guarded_send_run_id"],
        "v160-production-mutation-guarded-send"
    );
    assert_eq!(
        artifact["source_guarded_send_hash"],
        file_fnv1a64_hash(&guarded_send.display().to_string())
    );
    assert_eq!(
        artifact["redacted_response_derived_from_actual_http_result"],
        false
    );
    assert_eq!(artifact["synthetic_fixture_redaction_only"], true);
    assert_eq!(artifact["owner_run_mutation_closure_evidence"], false);
    assert_eq!(
        artifact["source_guarded_send_status"],
        "ready_guarded_send_path_offline_no_network"
    );
    assert_eq!(artifact["response_shape_validated"], true);
    assert_eq!(
        artifact["response_type"],
        "binance_order_response_redacted_metadata_v1"
    );
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["side"], "BUY");
    assert_eq!(artifact["order_type"], "LIMIT");
    assert_eq!(artifact["time_in_force"], "GTC");
    assert_eq!(artifact["order_id"], "123456789");
    assert_eq!(
        artifact["client_order_id"],
        "owner-approved-v160-single-shot"
    );
    assert_eq!(artifact["exchange_status"], "NEW");
    assert_eq!(
        artifact["transact_time_shape"],
        "epoch_millis_present_redacted"
    );
    assert_eq!(
        artifact["working_time_shape"],
        "epoch_millis_present_redacted"
    );
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["api_key_header_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["request_body_recorded"], false);
    assert_eq!(artifact["raw_request_body_recorded"], false);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_body_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["unrestricted_payload_recorded"], false);
    assert_eq!(artifact["account_balances_recorded"], false);
    assert_eq!(artifact["fills_recorded"], false);
    assert_eq!(artifact["response_redacted"], true);
    assert_eq!(
        artifact["forbidden_response_markers"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["replace_attempted"], false);
    assert_eq!(artifact["amend_attempted"], false);
    assert_eq!(artifact["flatten_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["real_funds"], false);
}

#[test]
fn production_mutation_response_redaction_marks_actual_http_result_source() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v161-003-response-redaction-actual-source-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let guarded_send = write_actual_v161_guarded_send_http_result_artifact(&output_dir);
    let response = output_dir.join("synthetic_order_response.json");
    let output = output_dir.join("production_mutation_response_redaction.json");
    write_synthetic_production_mutation_response(&response);

    run_live_production_mutation_response_redaction(&production_mutation_response_redaction_opt(
        guarded_send.clone(),
        response,
        output.clone(),
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "ready_response_redacted");
    assert_eq!(
        artifact["response_redaction_source"],
        "actual_guarded_send_http_result"
    );
    assert_eq!(
        artifact["source_guarded_send_hash"],
        file_fnv1a64_hash(&guarded_send.display().to_string())
    );
    assert_eq!(
        artifact["redacted_response_derived_from_actual_http_result"],
        true
    );
    assert_eq!(artifact["synthetic_fixture_redaction_only"], false);
    assert_eq!(artifact["owner_run_mutation_closure_evidence"], true);
    assert_eq!(artifact["request_sent"], true);
    assert_eq!(artifact["network_attempted"], true);
    assert_eq!(artifact["production_orders_submitted"], 1);
    assert_eq!(artifact["production_order_mutations_attempted"], 1);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["unrestricted_payload_recorded"], false);
    assert_eq!(artifact["account_balances_recorded"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_mutation_response_redaction_blocks_forbidden_response_markers() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-006-response-redaction-forbidden-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let guarded_send = write_ready_v160_guarded_send_artifact(&output_dir);
    let response = output_dir.join("forbidden_order_response.json");
    let output = output_dir.join("production_mutation_response_redaction.json");
    write_forbidden_production_mutation_response(&response);

    run_live_production_mutation_response_redaction(&production_mutation_response_redaction_opt(
        guarded_send,
        response,
        output.clone(),
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_forbidden_response_marker");
    assert_eq!(artifact["response_redaction_ready"], false);
    assert_eq!(artifact["response_shape_validated"], false);
    assert!(
        artifact["forbidden_response_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.headers"))
    );
    assert!(
        artifact["forbidden_response_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.signature"))
    );
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["unrestricted_payload_recorded"], false);
    assert_eq!(artifact["account_balances_recorded"], false);
}

#[test]
fn production_mutation_response_redaction_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-006-response-redaction-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let guarded_send = write_ready_v160_guarded_send_artifact(&output_dir);
    let response = output_dir.join("synthetic_order_response.json");
    let output = output_dir.join("production_mutation_response_redaction.json");
    write_synthetic_production_mutation_response(&response);

    run_live_production_mutation_response_redaction(&production_mutation_response_redaction_opt(
        guarded_send,
        response,
        output.clone(),
        false,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["response_redaction_ready"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-response-redaction")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-raw-response-persistence")
    );
}

#[test]
fn production_mutation_order_state_readback_writes_ready_offline_known_order_contract() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-007-order-state-readback-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let response_redaction = write_ready_v160_response_redaction_artifact(&output_dir);
    let output = output_dir.join("production_mutation_order_state_readback.json");

    run_live_production_mutation_order_state_readback_with_env_and_http(
        &production_mutation_order_state_readback_opt(
            response_redaction,
            output.clone(),
            false,
            true,
        ),
        &mut |_| None,
        |_opt, _credentials, _recv_window_ms| {
            panic!("offline order-state readback must not call HTTP")
        },
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("ntpro_v160007_api_key_value"));
    assert!(!body.contains("ntpro_v160007_api_secret_value"));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_ORDER_STATE_READBACK_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["status"],
        "ready_offline_order_state_readback_contract"
    );
    assert_eq!(artifact["readback_contract_ready"], true);
    assert_eq!(
        artifact["source_response_redaction_status"],
        "ready_response_redacted"
    );
    assert_eq!(
        artifact["known_order_identifier_source"],
        "production_mutation_response_redaction"
    );
    assert_eq!(artifact["known_order_id"], "123456789");
    assert_eq!(
        artifact["known_client_order_id"],
        "owner-approved-v160-single-shot"
    );
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["endpoint"], "order");
    assert_eq!(artifact["method"], "GET");
    assert_eq!(artifact["path"], "/api/v3/order");
    assert_eq!(artifact["manual_online_requested"], false);
    assert_eq!(artifact["order_state_read_allowed"], false);
    assert_eq!(artifact["order_state_read_attempted"], false);
    assert_eq!(artifact["response_shape"], "binance_order_state_v1");
    assert_eq!(artifact["response_shape_validated"], false);
    assert_eq!(artifact["strategy_success_inferred"], false);
    assert_eq!(
        artifact["strategy_success_proof"],
        "not_inferred_readback_is_observability_only"
    );
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["api_key_present"], false);
    assert_eq!(artifact["api_secret_present"], false);
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["api_key_header_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_body_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["response_redacted"], true);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_order_state_reads_allowed"], false);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["replace_attempted"], false);
    assert_eq!(artifact["amend_attempted"], false);
    assert_eq!(artifact["flatten_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["production_trading_enabled"], false);
}

#[test]
fn production_mutation_order_state_readback_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-007-order-state-readback-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let response_redaction = write_ready_v160_response_redaction_artifact(&output_dir);
    let output = output_dir.join("production_mutation_order_state_readback.json");

    run_live_production_mutation_order_state_readback_with_env_and_http(
        &production_mutation_order_state_readback_opt(
            response_redaction,
            output.clone(),
            false,
            false,
        ),
        &mut |_| None,
        |_opt, _credentials, _recv_window_ms| {
            panic!("blocked order-state readback must not call HTTP")
        },
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["readback_contract_ready"], false);
    assert_eq!(artifact["order_state_read_attempted"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-order-state-readback")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-known-order-identifier-only")
    );
}

#[test]
fn production_mutation_order_state_readback_blocks_manual_online_without_env_gates() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-007-order-state-readback-manual-missing-env-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let response_redaction = write_ready_v160_response_redaction_artifact(&output_dir);
    let output = output_dir.join("production_mutation_order_state_readback.json");
    let mut http_called = false;

    run_live_production_mutation_order_state_readback_with_env_and_http(
        &production_mutation_order_state_readback_opt(
            response_redaction,
            output.clone(),
            true,
            true,
        ),
        &mut |_| None,
        |_opt, _credentials, _recv_window_ms| {
            http_called = true;
            ProductionOrderStateHttpResult::success(ProductionOrderStateReadEndpoint::Order, 1, 200)
        },
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert!(!http_called);
    assert_eq!(artifact["status"], "blocked_missing_manual_online_gate");
    assert_eq!(artifact["manual_online_requested"], true);
    assert_eq!(artifact["readback_contract_ready"], false);
    assert_eq!(artifact["order_state_read_allowed"], false);
    assert_eq!(artifact["order_state_read_attempted"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert!(
        artifact["missing_env_vars"]
            .as_array()
            .unwrap()
            .iter()
            .any(|env| env == PRODUCTION_ORDER_STATE_ENV_ALLOW)
    );
    assert!(
        artifact["missing_env_vars"]
            .as_array()
            .unwrap()
            .iter()
            .any(|env| env == PRODUCTION_ORDER_STATE_ENV_MANUAL_ONLINE)
    );
}

#[test]
fn production_mutation_order_state_readback_records_owner_gated_online_success() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-007-order-state-readback-online-success-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let response_redaction = write_ready_v160_response_redaction_artifact(&output_dir);
    let output = output_dir.join("production_mutation_order_state_readback.json");
    let mut read_env = |name: &str| match name {
        "NTPRO_V160007_API_KEY" => Some("ntpro_v160007_api_key_value".to_string()),
        "NTPRO_V160007_API_SECRET" => Some("ntpro_v160007_api_secret_value".to_string()),
        _ => all_env_enabled(name),
    };

    run_live_production_mutation_order_state_readback_with_env_and_http(
        &production_mutation_order_state_readback_opt(
            response_redaction,
            output.clone(),
            true,
            true,
        ),
        &mut read_env,
        |proof_opt, credentials, recv_window_ms| {
            assert_eq!(proof_opt.endpoint, ProductionOrderStateReadEndpoint::Order);
            assert_eq!(proof_opt.symbol, "BTCUSDT");
            assert_eq!(proof_opt.order_id, Some(123_456_789));
            assert_eq!(
                proof_opt.orig_client_order_id.as_deref(),
                Some("owner-approved-v160-single-shot")
            );
            assert!(credentials.api_key_present());
            assert!(credentials.api_secret_present());
            assert_eq!(recv_window_ms, 5_000);
            ProductionOrderStateHttpResult::success(
                ProductionOrderStateReadEndpoint::Order,
                17,
                200,
            )
        },
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v160007_api_key_value"));
    assert!(!body.contains("ntpro_v160007_api_secret_value"));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(artifact["status"], "online_order_state_read_ok");
    assert_eq!(artifact["readback_contract_ready"], true);
    assert_eq!(artifact["manual_online_requested"], true);
    assert_eq!(artifact["order_state_read_allowed"], true);
    assert_eq!(artifact["order_state_read_attempted"], true);
    assert_eq!(artifact["network_attempted"], true);
    assert_eq!(artifact["production_order_state_reads_allowed"], true);
    assert_eq!(artifact["production_order_state_reads_attempted"], 1);
    assert_eq!(artifact["response_status_code"], 200);
    assert_eq!(artifact["response_shape"], "binance_order_state_v1");
    assert_eq!(artifact["response_shape_validated"], true);
    assert_eq!(artifact["endpoint_shape_validated"], true);
    assert_eq!(artifact["order_entries_observed"], 1);
    assert_eq!(artifact["non_empty_order_state_observed"], true);
    assert_eq!(artifact["order_lifecycle_readiness"], true);
    assert_eq!(artifact["latency_ms"], 17);
    assert_eq!(artifact["error_code"], "none");
    assert_eq!(artifact["strategy_success_inferred"], false);
    assert_eq!(
        artifact["strategy_success_proof"],
        "not_inferred_readback_is_observability_only"
    );
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["replace_attempted"], false);
    assert_eq!(artifact["amend_attempted"], false);
    assert_eq!(artifact["flatten_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["real_orders_submitted"], false);
    assert_eq!(artifact["production_trading_enabled"], false);
}

#[test]
fn production_mutation_audit_trail_links_redacted_candidate_chain() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-009-audit-trail-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (request_builder, guarded_send, response_redaction, order_state_readback) =
        write_ready_v160_audit_trail_sources(&output_dir);
    let output = output_dir.join("production_mutation_audit_trail.json");

    run_live_production_mutation_audit_trail(&production_mutation_audit_trail_opt(
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        output.clone(),
        true,
    ))
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v160005_production_like_api_key_value"));
    assert!(!body.contains("ntpro_v160005_production_like_api_secret_value"));
    assert!(!body.contains("signature="));
    assert!(!body.contains("X-MBX-APIKEY"));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "ready_redacted_audit_trail");
    assert_eq!(artifact["audit_trail_ready"], true);
    assert!(
        artifact["preview_hash"]
            .as_str()
            .unwrap()
            .starts_with("fnv1a64:")
    );
    assert_eq!(
        artifact["signing_approval_status"],
        "ready_signing_material_approval"
    );
    assert_eq!(artifact["approval_state"], "approved");
    assert_eq!(artifact["manual_approval_recorded"], true);
    assert_eq!(artifact["manual_approval_id"], "owner-approval-v160-003");
    assert_eq!(artifact["approved_by"], "owner");
    assert_eq!(
        artifact["runtime_gate_status"],
        "blocked_explicit_send_gate"
    );
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["send_consideration_allowed"], false);
    assert_eq!(
        artifact["guarded_send_status"],
        "ready_guarded_send_path_offline_no_network"
    );
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(
        artifact["response_redaction_status"],
        "ready_response_redacted"
    );
    assert_eq!(artifact["response_redaction_ready"], true);
    assert_eq!(
        artifact["order_state_readback_status"],
        "ready_offline_order_state_readback_contract"
    );
    assert_eq!(artifact["readback_contract_ready"], true);
    assert_eq!(artifact["order_state_read_attempted"], false);
    assert_eq!(artifact["kill_switch_checked_before_send"], true);
    assert_eq!(artifact["kill_switch_checked_after_send"], true);
    assert_eq!(artifact["pre_send_kill_switch_runtime_gate_open"], true);
    assert_eq!(artifact["pre_send_kill_switch_active"], false);
    assert_eq!(artifact["post_send_kill_switch_runtime_gate_open"], true);
    assert_eq!(artifact["post_send_kill_switch_active"], false);
    assert_eq!(artifact["kill_switch_blocked_send"], false);
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["side"], "BUY");
    assert_eq!(artifact["order_type"], "LIMIT");
    assert_eq!(artifact["time_in_force"], "GTC");
    assert_eq!(artifact["order_id"], "123456789");
    assert_eq!(
        artifact["client_order_id"],
        "owner-approved-v160-single-shot"
    );
    assert_eq!(artifact["exchange_status"], "NEW");
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["failure_state"], "none_recorded");
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_body_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["unrestricted_payload_recorded"], false);
    assert_eq!(artifact["account_balances_recorded"], false);
    assert_eq!(artifact["response_redacted"], true);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_state_reads_allowed"], false);
    assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["replace_attempted"], false);
    assert_eq!(artifact["amend_attempted"], false);
    assert_eq!(artifact["flatten_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["production_trading_enabled"], false);
}

#[test]
fn production_mutation_audit_trail_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-009-audit-trail-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (request_builder, guarded_send, response_redaction, order_state_readback) =
        write_ready_v160_audit_trail_sources(&output_dir);
    let output = output_dir.join("production_mutation_audit_trail.json");

    run_live_production_mutation_audit_trail(&production_mutation_audit_trail_opt(
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        output.clone(),
        false,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["audit_trail_ready"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["failure_state"], "blocked_missing_gate");
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-audit-trail")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-redacted-artifacts-only")
    );
}

#[test]
fn production_mutation_failure_semantics_records_no_retry_for_failure_modes() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-010-failure-semantics-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let audit_trail = write_ready_v160_audit_trail_artifact(&output_dir);

    for (mode, expected_state) in [
        (
            ProductionMutationFailureMode::Timeout,
            "timeout_write_evidence_and_stop",
        ),
        (
            ProductionMutationFailureMode::Http4xx,
            "http_4xx_write_evidence_and_stop",
        ),
        (
            ProductionMutationFailureMode::Http5xx,
            "http_5xx_write_evidence_and_stop",
        ),
        (
            ProductionMutationFailureMode::MalformedResponse,
            "malformed_response_write_evidence_and_stop",
        ),
        (
            ProductionMutationFailureMode::ReadbackMismatch,
            "readback_mismatch_write_evidence_and_stop",
        ),
        (
            ProductionMutationFailureMode::KillSwitchTransition,
            "kill_switch_transition_write_evidence_and_stop",
        ),
    ] {
        let output = output_dir.join(format!("failure_semantics_{}.json", mode.as_str()));
        run_live_production_mutation_failure_semantics(&production_mutation_failure_semantics_opt(
            audit_trail.clone(),
            output.clone(),
            mode,
            true,
        ))
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("ntpro_v160005_production_like_api_key_value"));
        assert!(!body.contains("ntpro_v160005_production_like_api_secret_value"));
        assert!(!body.contains("signature="));
        assert!(!body.contains("X-MBX-APIKEY"));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_failure_semantics_evidence");
        assert_eq!(artifact["failure_semantics_ready"], true);
        assert_eq!(artifact["failure_mode"], mode.as_str());
        assert_eq!(artifact["failure_state"], expected_state);
        assert_eq!(artifact["terminal_action"], "write_evidence_and_stop");
        assert_eq!(artifact["evidence_written"], true);
        assert_eq!(artifact["stop_after_evidence"], true);
        assert_eq!(artifact["strategy_continuation_allowed"], false);
        assert_eq!(
            artifact["source_audit_trail_status"],
            "ready_redacted_audit_trail"
        );
        assert_eq!(artifact["source_audit_trail_ready"], true);
        assert_eq!(artifact["source_failure_state"], "none_recorded");
        assert_eq!(artifact["retry_allowed"], false);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["retry_attempts"], 0);
        assert_eq!(artifact["max_retry_attempts"], 0);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["correction_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["remediation_attempted"], false);
        assert_eq!(artifact["automatic_remediation_allowed"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
        assert_eq!(artifact["production_order_mutations_attempted"], 0);
        assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["evidence_only_failure_handling_confirmed"], true);
        assert_eq!(artifact["no_retry_confirmed"], true);
        assert_eq!(
            artifact["no_automatic_cancel_replace_amend_confirmed"],
            true
        );
        assert_eq!(artifact["no_correction_or_flatten_confirmed"], true);
        assert_eq!(artifact["dashboard_controls_disabled_confirmed"], true);
        assert_eq!(artifact["no_strategy_continuation_confirmed"], true);
        assert_eq!(artifact["no_listen_key_lifecycle_confirmed"], true);
    }
}

#[test]
fn production_mutation_failure_semantics_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-010-failure-semantics-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let audit_trail = write_ready_v160_audit_trail_artifact(&output_dir);
    let output = output_dir.join("failure_semantics_missing_flags.json");

    run_live_production_mutation_failure_semantics(&production_mutation_failure_semantics_opt(
        audit_trail,
        output.clone(),
        ProductionMutationFailureMode::Timeout,
        false,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["failure_semantics_ready"], false);
    assert_eq!(artifact["failure_state"], "blocked_missing_gate");
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
    assert_eq!(artifact["strategy_continuation_allowed"], false);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-failure-semantics")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-retry")
    );
}

#[test]
fn production_mutation_local_order_ledger_links_single_candidate_chain() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-001-local-ledger-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        audit_trail,
        failure_semantics,
    ) = write_ready_v170_local_order_ledger_sources(&output_dir);
    let output = output_dir.join("production_mutation_local_order_ledger.json");

    run_live_production_mutation_local_order_ledger(&production_mutation_local_order_ledger_opt(
        (
            request_builder,
            guarded_send,
            response_redaction,
            order_state_readback,
            audit_trail,
            failure_semantics,
        ),
        output.clone(),
        true,
    ))
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("ntpro_v160005_production_like_api_key_value"));
    assert!(!body.contains("ntpro_v160005_production_like_api_secret_value"));
    assert!(!body.contains("signature="));
    assert!(!body.contains("X-MBX-APIKEY"));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "ready_local_order_ledger");
    assert_eq!(artifact["order_lineage_id"], "lineage-v160-single-shot");
    assert_eq!(artifact["local_ledger_ready"], true);
    assert_eq!(artifact["restart_readable"], true);
    assert_eq!(
        artifact["capability"],
        "Production Reconciliation And Orphan Recovery Evidence"
    );
    assert_eq!(
        artifact["capability_expansion_from_v16"],
        "reconciliation_evidence_only"
    );
    assert_eq!(artifact["lineage_scope"], "single_v16_mutation_candidate");
    assert_eq!(
        artifact["current_local_state"],
        "local_ledger_pending_exchange_reconciliation"
    );
    assert_eq!(artifact["default_fail_closed"], true);
    assert_eq!(artifact["owner_gated_readback_required"], true);
    assert_eq!(
        artifact["request_builder_ref"]["schema_version"],
        PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["guarded_send_ref"]["schema_version"],
        PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["response_redaction_ref"]["schema_version"],
        PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["readback_ref"]["schema_version"],
        PRODUCTION_MUTATION_ORDER_STATE_READBACK_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["audit_ref"]["schema_version"],
        PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["failure_ref"]["schema_version"],
        PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION
    );
    for field in [
        "request_builder_ref",
        "guarded_send_ref",
        "response_redaction_ref",
        "readback_ref",
        "audit_ref",
        "failure_ref",
    ] {
        assert!(
            artifact[field]["hash"]
                .as_str()
                .unwrap()
                .starts_with("fnv1a64:"),
            "{field}"
        );
        assert!(
            artifact[field]["sha256"]
                .as_str()
                .unwrap()
                .starts_with("sha256:"),
            "{field}"
        );
        assert!(artifact[field]["bytes"].as_u64().unwrap() > 0, "{field}");
        assert_ne!(artifact[field]["source_command"], "unknown", "{field}");
        assert_eq!(artifact[field]["source_commit"], "unknown", "{field}");
        assert_eq!(
            artifact[field]["source_release_tag"], "unreleased",
            "{field}"
        );
    }
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["side"], "BUY");
    assert_eq!(artifact["order_type"], "LIMIT");
    assert_eq!(artifact["time_in_force"], "GTC");
    assert_eq!(artifact["order_id"], "123456789");
    assert_eq!(
        artifact["client_order_id"],
        "owner-approved-v160-single-shot"
    );
    assert_eq!(artifact["exchange_status"], "NEW");
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["exchange_readback_mapped"], false);
    assert_eq!(artifact["reconciliation_classified"], false);
    assert_eq!(artifact["orphan_risk_detected"], false);
    assert_eq!(artifact["manual_review_required"], false);
    assert_eq!(artifact["new_orders_blocked"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_state_reads_allowed"], false);
    assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
    assert_eq!(artifact["duplicate_submit_attempted"], false);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["replace_attempted"], false);
    assert_eq!(artifact["amend_attempted"], false);
    assert_eq!(artifact["flatten_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
    assert_eq!(artifact["automatic_cancel_allowed"], false);
    assert_eq!(artifact["automatic_remediation_allowed"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_body_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["no_network_confirmed"], true);
    assert_eq!(artifact["no_duplicate_submit_confirmed"], true);
    assert_eq!(artifact["no_retry_confirmed"], true);
    assert_eq!(artifact["no_cancel_confirmed"], true);
    assert_eq!(artifact["no_remediation_confirmed"], true);
    assert_eq!(artifact["dashboard_controls_disabled_confirmed"], true);
    assert_eq!(artifact["no_secret_persistence_confirmed"], true);
}

#[test]
fn production_mutation_local_order_ledger_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-001-local-ledger-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        audit_trail,
        failure_semantics,
    ) = write_ready_v170_local_order_ledger_sources(&output_dir);
    let output = output_dir.join("production_mutation_local_order_ledger.json");

    run_live_production_mutation_local_order_ledger(&production_mutation_local_order_ledger_opt(
        (
            request_builder,
            guarded_send,
            response_redaction,
            order_state_readback,
            audit_trail,
            failure_semantics,
        ),
        output.clone(),
        false,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["local_ledger_ready"], false);
    assert_eq!(artifact["restart_readable"], false);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert_eq!(artifact["duplicate_submit_attempted"], false);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-local-order-ledger")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-duplicate-submit")
    );
}

#[test]
fn production_mutation_local_order_ledger_is_restart_readable() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-001-local-ledger-restart-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (
        request_builder,
        guarded_send,
        response_redaction,
        order_state_readback,
        audit_trail,
        failure_semantics,
    ) = write_ready_v170_local_order_ledger_sources(&output_dir);
    let output = output_dir.join("production_mutation_local_order_ledger.json");

    run_live_production_mutation_local_order_ledger(&production_mutation_local_order_ledger_opt(
        (
            request_builder.clone(),
            guarded_send,
            response_redaction,
            order_state_readback,
            audit_trail,
            failure_semantics,
        ),
        output.clone(),
        true,
    ))
    .unwrap();

    let first: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
    let reloaded: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
    assert_eq!(reloaded["local_ledger_ready"], true);
    assert_eq!(reloaded["restart_readable"], true);
    assert_eq!(reloaded["order_lineage_id"], first["order_lineage_id"]);
    assert_eq!(
        reloaded["request_builder_ref"]["path"],
        request_builder.display().to_string()
    );
    assert_eq!(
        reloaded["request_builder_ref"]["hash"],
        first["request_builder_ref"]["hash"]
    );
    assert_eq!(reloaded["duplicate_submit_attempted"], false);
    assert_eq!(reloaded["retry_attempted"], false);
    assert_eq!(reloaded["cancel_attempted"], false);
    assert_eq!(reloaded["remediation_attempted"], false);
}

#[test]
fn production_mutation_exchange_readback_mapper_normalizes_statuses() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-002-exchange-mapper-statuses-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let local_order_ledger = write_ready_v170_local_order_ledger_artifact(&output_dir);

    for (status, expected_state, terminal, include_open_order) in [
        ("NEW", "open", false, true),
        ("FILLED", "filled", true, false),
        ("CANCELED", "canceled", true, false),
        ("REJECTED", "rejected", true, false),
    ] {
        let order_readback = output_dir.join(format!("order_readback_{status}.json"));
        let open_orders_readback = output_dir.join(format!("open_orders_{status}.json"));
        let output = output_dir.join(format!("exchange_readback_mapper_{status}.json"));
        write_redacted_exchange_order_readback(&order_readback, Some(status), true);
        write_redacted_exchange_open_orders_readback(&open_orders_readback, include_open_order);

        run_live_production_mutation_exchange_readback_mapper(
            &production_mutation_exchange_readback_mapper_opt(
                local_order_ledger.clone(),
                order_readback,
                open_orders_readback,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("X-MBX-APIKEY"));
        assert!(!body.contains("signature="));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_exchange_readback_mapped");
        assert_eq!(artifact["exchange_readback_mapped"], true);
        assert_eq!(artifact["reconciliation_classified"], false);
        assert_eq!(artifact["orphan_risk_detected"], false);
        assert_eq!(artifact["order_lineage_id"], "lineage-v160-single-shot");
        assert_eq!(artifact["exchange_order_status"], status);
        assert_eq!(artifact["exchange_order_state"], expected_state);
        assert_eq!(artifact["terminal_state_observed"], terminal);
        assert_eq!(artifact["open_order_observed"], include_open_order);
        assert_eq!(artifact["known_order_id"], "123456789");
        assert_eq!(
            artifact["known_client_order_id"],
            "owner-approved-v160-single-shot"
        );
        assert_eq!(artifact["symbol"], "BTCUSDT");
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(
            artifact["malformed_readback_issues"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["manual_review_required"], false);
        assert_eq!(artifact["new_orders_blocked"], false);
        assert_eq!(artifact["request_sent"], false);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["duplicate_submit_attempted"], false);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["remediation_attempted"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["raw_exchange_response_recorded"], false);
        assert_eq!(artifact["response_body_recorded"], false);
        assert_eq!(artifact["response_headers_recorded"], false);
    }
}

#[test]
fn production_mutation_exchange_readback_mapper_handles_missing_and_empty_open_orders() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-002-exchange-mapper-missing-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let local_order_ledger = write_ready_v170_local_order_ledger_artifact(&output_dir);
    let order_readback = output_dir.join("order_readback_missing.json");
    let open_orders_readback = output_dir.join("open_orders_empty.json");
    let output = output_dir.join("exchange_readback_mapper_missing.json");
    write_redacted_exchange_order_readback(&order_readback, None, false);
    write_redacted_exchange_open_orders_readback(&open_orders_readback, false);

    run_live_production_mutation_exchange_readback_mapper(
        &production_mutation_exchange_readback_mapper_opt(
            local_order_ledger,
            order_readback,
            open_orders_readback,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "ready_exchange_readback_mapped");
    assert_eq!(artifact["exchange_readback_mapped"], true);
    assert_eq!(artifact["exchange_order_status"], "MISSING");
    assert_eq!(artifact["exchange_order_state"], "missing");
    assert_eq!(artifact["order_found"], false);
    assert_eq!(artifact["open_orders_count"], 0);
    assert_eq!(artifact["open_order_observed"], false);
    assert_eq!(artifact["terminal_state_observed"], false);
    assert_eq!(artifact["manual_review_required"], false);
    assert_eq!(artifact["new_orders_blocked"], false);
}

#[test]
fn production_mutation_exchange_readback_mapper_blocks_malformed_shape() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-002-exchange-mapper-malformed-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let local_order_ledger = write_ready_v170_local_order_ledger_artifact(&output_dir);
    let order_readback = output_dir.join("order_readback_malformed.json");
    let open_orders_readback = output_dir.join("open_orders_empty.json");
    let output = output_dir.join("exchange_readback_mapper_malformed.json");
    write_redacted_exchange_order_readback(&order_readback, None, true);
    write_redacted_exchange_open_orders_readback(&open_orders_readback, false);

    run_live_production_mutation_exchange_readback_mapper(
        &production_mutation_exchange_readback_mapper_opt(
            local_order_ledger,
            order_readback,
            open_orders_readback,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_malformed_exchange_readback");
    assert_eq!(artifact["exchange_readback_mapped"], false);
    assert_eq!(artifact["exchange_order_state"], "malformed");
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert!(
        artifact["malformed_readback_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "exchange_status_missing")
    );
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
}

#[test]
fn production_mutation_exchange_readback_mapper_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-002-exchange-mapper-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let local_order_ledger = write_ready_v170_local_order_ledger_artifact(&output_dir);
    let order_readback = output_dir.join("order_readback_new.json");
    let open_orders_readback = output_dir.join("open_orders_new.json");
    let output = output_dir.join("exchange_readback_mapper_missing_flags.json");
    write_redacted_exchange_order_readback(&order_readback, Some("NEW"), true);
    write_redacted_exchange_open_orders_readback(&open_orders_readback, true);

    run_live_production_mutation_exchange_readback_mapper(
        &production_mutation_exchange_readback_mapper_opt(
            local_order_ledger,
            order_readback,
            open_orders_readback,
            output.clone(),
            false,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["exchange_readback_mapped"], false);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-exchange-readback-mapper")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-redacted-readback-metadata-only")
    );
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_mutation_reconciliation_classifier_classifies_required_outcomes() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-003-reconciliation-classifier-outcomes-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();

    for (
        name,
        source_status,
        exchange_readback_mapped,
        request_sent,
        exchange_order_status,
        exchange_order_state,
        order_found,
        open_order_observed,
        terminal_state_observed,
        expected_outcome,
        expected_manual_review,
        expected_new_orders_blocked,
    ) in [
        (
            "local-sent-exchange-unknown",
            "ready_exchange_readback_mapped",
            true,
            true,
            "UNKNOWN",
            "unknown",
            true,
            false,
            false,
            "local_sent_exchange_unknown",
            true,
            true,
        ),
        (
            "local-sent-exchange-new",
            "ready_exchange_readback_mapped",
            true,
            true,
            "NEW",
            "open",
            true,
            true,
            false,
            "local_sent_exchange_new",
            true,
            true,
        ),
        (
            "local-sent-exchange-filled",
            "ready_exchange_readback_mapped",
            true,
            true,
            "FILLED",
            "filled",
            true,
            false,
            true,
            "local_sent_exchange_filled",
            false,
            false,
        ),
        (
            "local-sent-exchange-canceled",
            "ready_exchange_readback_mapped",
            true,
            true,
            "CANCELED",
            "canceled",
            true,
            false,
            true,
            "local_sent_exchange_canceled",
            false,
            false,
        ),
        (
            "local-sent-exchange-rejected",
            "ready_exchange_readback_mapped",
            true,
            true,
            "REJECTED",
            "rejected",
            true,
            false,
            true,
            "local_sent_exchange_rejected",
            false,
            false,
        ),
        (
            "local-sent-exchange-missing",
            "ready_exchange_readback_mapped",
            true,
            true,
            "MISSING",
            "missing",
            false,
            false,
            false,
            "local_sent_exchange_missing",
            true,
            true,
        ),
        (
            "local-no-send-exchange-order-seen",
            "ready_exchange_readback_mapped",
            true,
            false,
            "NEW",
            "open",
            true,
            true,
            false,
            "local_no_send_exchange_order_seen",
            true,
            true,
        ),
        (
            "readback-failed",
            "blocked_malformed_exchange_readback",
            false,
            true,
            "MALFORMED",
            "malformed",
            true,
            false,
            false,
            "readback_failed",
            true,
            true,
        ),
    ] {
        let mapper = output_dir.join(format!("mapper-{name}.json"));
        let output = output_dir.join(format!("classifier-{name}.json"));
        write_v170_exchange_readback_mapper_fixture(
            &mapper,
            &V170ExchangeReadbackMapperFixture {
                source_status,
                exchange_readback_mapped,
                request_sent,
                exchange_order_status,
                exchange_order_state,
                order_found,
                open_order_observed,
                terminal_state_observed,
            },
        );

        run_live_production_mutation_reconciliation_classifier(
            &production_mutation_reconciliation_classifier_opt(mapper, output.clone(), true),
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("X-MBX-APIKEY"));
        assert!(!body.contains("signature="));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_reconciliation_classified");
        assert_eq!(artifact["reconciliation_classified"], true);
        assert_eq!(artifact["orphan_risk_detected"], false);
        assert_eq!(artifact["order_lineage_id"], "lineage-v160-single-shot");
        assert_eq!(artifact["local_request_sent"], request_sent);
        assert_eq!(artifact["exchange_order_status"], exchange_order_status);
        assert_eq!(artifact["exchange_order_state"], exchange_order_state);
        assert_eq!(artifact["open_order_observed"], open_order_observed);
        assert_eq!(artifact["terminal_state_observed"], terminal_state_observed);
        assert_eq!(artifact["order_found"], order_found);
        assert_eq!(artifact["reconciliation_outcome"], expected_outcome);
        assert_eq!(artifact["manual_review_required"], expected_manual_review);
        assert_eq!(artifact["new_orders_blocked"], expected_new_orders_blocked);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["duplicate_submit_attempted"], false);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["remediation_attempted"], false);
        assert_eq!(artifact["automatic_cancel_allowed"], false);
        assert_eq!(artifact["automatic_remediation_allowed"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["raw_exchange_response_recorded"], false);
        assert_eq!(artifact["response_body_recorded"], false);
        assert_eq!(artifact["response_headers_recorded"], false);
    }
}

#[test]
fn production_mutation_reconciliation_classifier_integrates_failure_incident_semantics() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-008-failure-incident-semantics-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();

    for (
        mode,
        expected_outcome,
        expected_severity,
        expected_readback_required,
        expected_terminal_evidence_required,
        expected_incident_risk_halted,
        expected_manual_review,
        expected_new_orders_blocked,
        expected_orphan_outcome,
    ) in [
        (
            ProductionMutationFailureMode::Timeout,
            "timeout_readback_required",
            "warning",
            true,
            false,
            false,
            true,
            true,
            "clean_terminal",
        ),
        (
            ProductionMutationFailureMode::Http4xx,
            "http_4xx_terminal_evidence",
            "info",
            false,
            true,
            false,
            false,
            false,
            "clean_terminal",
        ),
        (
            ProductionMutationFailureMode::Http5xx,
            "http_5xx_readback_required",
            "warning",
            true,
            false,
            false,
            true,
            true,
            "clean_terminal",
        ),
        (
            ProductionMutationFailureMode::MalformedResponse,
            "malformed_response_manual_review",
            "warning",
            false,
            false,
            false,
            true,
            true,
            "clean_terminal",
        ),
        (
            ProductionMutationFailureMode::ReadbackMismatch,
            "readback_mismatch_risk_halt",
            "critical",
            false,
            false,
            true,
            true,
            true,
            "failure_incident_risk_halt",
        ),
        (
            ProductionMutationFailureMode::KillSwitchTransition,
            "kill_switch_transition_halt",
            "critical",
            false,
            false,
            true,
            true,
            true,
            "failure_incident_risk_halt",
        ),
    ] {
        let case_dir = output_dir.join(mode.as_str());
        fs::create_dir_all(&case_dir).unwrap();
        let audit_trail = write_ready_v160_audit_trail_artifact(&case_dir);
        let failure_semantics = case_dir.join("production_mutation_failure_semantics.json");
        run_live_production_mutation_failure_semantics(&production_mutation_failure_semantics_opt(
            audit_trail,
            failure_semantics.clone(),
            mode,
            true,
        ))
        .unwrap();

        let failure_artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&failure_semantics).unwrap()).unwrap();
        let local_ledger = case_dir.join("production_mutation_local_order_ledger.json");
        let local_ledger_value = json!({
            "schema_version": PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_SCHEMA_VERSION,
            "run_id": "v170-failure-incident-local-ledger",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "production_mutation_local_order_ledger",
            "status": "ready_local_order_ledger",
            "local_ledger_ready": true,
            "restart_readable": true,
            "failure_ref": production_mutation_local_order_ledger_source_ref(
                &failure_semantics,
                &failure_artifact,
                "failure_semantics_ready",
            ),
        });
        atomic_write_json(&local_ledger, &local_ledger_value).unwrap();

        let mapper = case_dir.join("exchange_readback_mapper.json");
        write_v170_exchange_readback_mapper_fixture(
            &mapper,
            &V170ExchangeReadbackMapperFixture {
                source_status: "ready_exchange_readback_mapped",
                exchange_readback_mapped: true,
                request_sent: true,
                exchange_order_status: "FILLED",
                exchange_order_state: "filled",
                order_found: true,
                open_order_observed: false,
                terminal_state_observed: true,
            },
        );
        let mut mapper_value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&mapper).unwrap()).unwrap();
        mapper_value["local_ledger_ref"] =
            json!(production_mutation_local_order_ledger_source_ref(
                &local_ledger,
                &local_ledger_value,
                "local_ledger_ready",
            ));
        atomic_write_json(&mapper, &mapper_value).unwrap();

        let classifier = case_dir.join("reconciliation_classifier.json");
        run_live_production_mutation_reconciliation_classifier(
            &production_mutation_reconciliation_classifier_opt(mapper, classifier.clone(), true),
        )
        .unwrap();
        let classifier_artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&classifier).unwrap()).unwrap();
        assert_eq!(
            classifier_artifact["schema_version"],
            PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION
        );
        assert_eq!(
            classifier_artifact["status"],
            "ready_reconciliation_classified"
        );
        assert_eq!(
            classifier_artifact["reconciliation_outcome"],
            "local_sent_exchange_filled"
        );
        assert_eq!(classifier_artifact["failure_mode"], mode.as_str());
        assert_eq!(
            classifier_artifact["failure_incident_outcome"],
            expected_outcome
        );
        assert_eq!(
            classifier_artifact["failure_incident_severity"],
            expected_severity
        );
        assert_eq!(
            classifier_artifact["readback_required"],
            expected_readback_required
        );
        assert_eq!(
            classifier_artifact["terminal_evidence_required"],
            expected_terminal_evidence_required
        );
        assert_eq!(
            classifier_artifact["incident_risk_halted"],
            expected_incident_risk_halted
        );
        assert_eq!(
            classifier_artifact["incident_manual_review_required"],
            expected_manual_review
        );
        assert_eq!(
            classifier_artifact["incident_new_orders_blocked"],
            expected_new_orders_blocked
        );
        assert_eq!(
            classifier_artifact["manual_review_required"],
            expected_manual_review
        );
        assert_eq!(
            classifier_artifact["new_orders_blocked"],
            expected_new_orders_blocked
        );
        assert_eq!(classifier_artifact["retry_attempted"], false);
        assert_eq!(classifier_artifact["cancel_attempted"], false);
        assert_eq!(classifier_artifact["remediation_attempted"], false);
        assert_eq!(
            classifier_artifact["dashboard_order_controls_enabled"],
            false
        );

        let detector = case_dir.join("orphan_detector.json");
        run_live_production_mutation_orphan_order_detector(
            &production_mutation_orphan_order_detector_opt(classifier, detector.clone(), true),
        )
        .unwrap();
        let detector_artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(detector).unwrap()).unwrap();
        assert_eq!(
            detector_artifact["schema_version"],
            PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_SCHEMA_VERSION
        );
        assert_eq!(
            detector_artifact["orphan_detection_outcome"],
            expected_orphan_outcome
        );
        assert_eq!(
            detector_artifact["incident_risk_halted"],
            expected_incident_risk_halted
        );
        assert_eq!(
            detector_artifact["risk_halted"],
            expected_incident_risk_halted
        );
        assert_eq!(
            detector_artifact["manual_review_required"],
            expected_manual_review
        );
        assert_eq!(
            detector_artifact["new_orders_blocked"],
            expected_new_orders_blocked
        );
        assert_eq!(detector_artifact["retry_attempted"], false);
        assert_eq!(detector_artifact["cancel_attempted"], false);
        assert_eq!(detector_artifact["remediation_attempted"], false);
        assert_eq!(detector_artifact["dashboard_order_controls_enabled"], false);
    }
}

#[test]
fn production_mutation_reconciliation_classifier_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-003-reconciliation-classifier-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let mapper = output_dir.join("mapper-ready-new.json");
    let output = output_dir.join("classifier-missing-flags.json");
    write_v170_exchange_readback_mapper_fixture(
        &mapper,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );

    run_live_production_mutation_reconciliation_classifier(
        &production_mutation_reconciliation_classifier_opt(mapper, output.clone(), false),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["reconciliation_classified"], false);
    assert_eq!(
        artifact["reconciliation_outcome"],
        "local_sent_exchange_new"
    );
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-reconciliation-classifier")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-remediation")
    );
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_mutation_orphan_order_detector_detects_required_outcomes() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-004-orphan-detector-outcomes-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();

    for (
        name,
        fixture,
        expected_outcome,
        expected_orphan_risk,
        expected_risk_halted,
        expected_manual_review,
        expected_new_orders_blocked,
        expected_stale_restart,
        expected_local_terminal,
    ) in [
        (
            "clean-terminal",
            V170ReconciliationClassifierFixture {
                reconciliation_outcome: "local_sent_exchange_filled",
                local_request_sent: true,
                exchange_order_status: "FILLED",
                exchange_order_state: "filled",
                order_found: true,
                open_order_observed: false,
                terminal_state_observed: true,
                manual_review_required: false,
                new_orders_blocked: false,
                restart_readable: true,
            },
            "clean_terminal",
            false,
            false,
            false,
            false,
            false,
            true,
        ),
        (
            "open-orphan",
            V170ReconciliationClassifierFixture {
                reconciliation_outcome: "local_sent_exchange_new",
                local_request_sent: true,
                exchange_order_status: "NEW",
                exchange_order_state: "open",
                order_found: true,
                open_order_observed: true,
                terminal_state_observed: false,
                manual_review_required: true,
                new_orders_blocked: true,
                restart_readable: true,
            },
            "open_orphan_risk",
            true,
            true,
            true,
            true,
            false,
            false,
        ),
        (
            "local-missing-exchange-seen",
            V170ReconciliationClassifierFixture {
                reconciliation_outcome: "local_no_send_exchange_order_seen",
                local_request_sent: false,
                exchange_order_status: "NEW",
                exchange_order_state: "open",
                order_found: true,
                open_order_observed: false,
                terminal_state_observed: false,
                manual_review_required: true,
                new_orders_blocked: true,
                restart_readable: true,
            },
            "local_missing_exchange_seen",
            true,
            true,
            true,
            true,
            false,
            false,
        ),
        (
            "readback-failure",
            V170ReconciliationClassifierFixture {
                reconciliation_outcome: "readback_failed",
                local_request_sent: true,
                exchange_order_status: "MALFORMED",
                exchange_order_state: "malformed",
                order_found: true,
                open_order_observed: false,
                terminal_state_observed: false,
                manual_review_required: true,
                new_orders_blocked: true,
                restart_readable: true,
            },
            "readback_or_lineage_ambiguous",
            true,
            true,
            true,
            true,
            false,
            false,
        ),
        (
            "stale-ledger-restart",
            V170ReconciliationClassifierFixture {
                reconciliation_outcome: "local_sent_exchange_filled",
                local_request_sent: true,
                exchange_order_status: "FILLED",
                exchange_order_state: "filled",
                order_found: true,
                open_order_observed: false,
                terminal_state_observed: true,
                manual_review_required: false,
                new_orders_blocked: false,
                restart_readable: false,
            },
            "stale_ledger_restart_required",
            true,
            true,
            true,
            true,
            true,
            true,
        ),
    ] {
        let classifier = output_dir.join(format!("classifier-{name}.json"));
        let output = output_dir.join(format!("orphan-detector-{name}.json"));
        write_v170_reconciliation_classifier_fixture(&classifier, &fixture);

        run_live_production_mutation_orphan_order_detector(
            &production_mutation_orphan_order_detector_opt(classifier, output.clone(), true),
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("X-MBX-APIKEY"));
        assert!(!body.contains("signature="));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_SCHEMA_VERSION
        );
        assert_eq!(artifact["status"], "ready_orphan_order_detection_completed");
        assert_eq!(artifact["orphan_detection_completed"], true);
        assert_eq!(artifact["orphan_detection_outcome"], expected_outcome);
        assert_eq!(artifact["orphan_risk_detected"], expected_orphan_risk);
        assert_eq!(artifact["risk_halted"], expected_risk_halted);
        assert_eq!(artifact["manual_review_required"], expected_manual_review);
        assert_eq!(artifact["new_orders_blocked"], expected_new_orders_blocked);
        assert_eq!(
            artifact["stale_ledger_restart_required"],
            expected_stale_restart
        );
        assert_eq!(artifact["local_terminal_state"], expected_local_terminal);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["network_attempted"], false);
        assert_eq!(artifact["production_order_submission_allowed"], false);
        assert_eq!(artifact["production_order_mutation_allowed"], false);
        assert_eq!(artifact["duplicate_submit_attempted"], false);
        assert_eq!(artifact["retry_attempted"], false);
        assert_eq!(artifact["cancel_attempted"], false);
        assert_eq!(artifact["replace_attempted"], false);
        assert_eq!(artifact["amend_attempted"], false);
        assert_eq!(artifact["flatten_attempted"], false);
        assert_eq!(artifact["remediation_attempted"], false);
        assert_eq!(artifact["automatic_cancel_allowed"], false);
        assert_eq!(artifact["automatic_remediation_allowed"], false);
        assert_eq!(artifact["dashboard_order_controls_enabled"], false);
        assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
        assert_eq!(artifact["api_key_value_recorded"], false);
        assert_eq!(artifact["api_secret_value_recorded"], false);
        assert_eq!(artifact["signature_recorded"], false);
        assert_eq!(artifact["signed_query_recorded"], false);
        assert_eq!(artifact["signed_url_recorded"], false);
        assert_eq!(artifact["raw_exchange_response_recorded"], false);
        assert_eq!(artifact["response_body_recorded"], false);
        assert_eq!(artifact["response_headers_recorded"], false);
    }
}

#[test]
fn production_mutation_orphan_order_detector_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v170-004-orphan-detector-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let classifier = output_dir.join("classifier-open-orphan.json");
    let output = output_dir.join("orphan-detector-missing-flags.json");
    write_v170_reconciliation_classifier_fixture(
        &classifier,
        &V170ReconciliationClassifierFixture {
            reconciliation_outcome: "local_sent_exchange_new",
            local_request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
            manual_review_required: true,
            new_orders_blocked: true,
            restart_readable: true,
        },
    );

    run_live_production_mutation_orphan_order_detector(
        &production_mutation_orphan_order_detector_opt(classifier, output.clone(), false),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["orphan_detection_completed"], false);
    assert_eq!(artifact["orphan_risk_detected"], true);
    assert_eq!(artifact["risk_halted"], true);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-orphan-order-detector")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-cancel")
    );
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_mutation_cancel_request_preview_builds_redacted_single_candidate() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-003-cancel-request-preview-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let orphan = write_v180_cancel_request_preview_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let output = output_dir.join("cancel-request-preview.json");

    run_live_production_mutation_cancel_request_preview(
        &production_mutation_cancel_request_preview_opt(orphan, output.clone(), true),
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("123456789"));
    assert!(!body.contains("owner-approved-v160-single-shot"));
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("signature="));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_SCHEMA_VERSION
    );
    assert_eq!(artifact["artifact_type"], "cancel_request_preview");
    assert_eq!(artifact["status"], "ready_cancel_request_preview");
    assert_eq!(
        artifact["capability"],
        "Owner-Approved Cancel Recovery Preview"
    );
    assert_eq!(
        artifact["capability_expansion"],
        "preview_gate_approval_only"
    );
    assert_eq!(artifact["lineage_scope"], "single_v16_mutation_candidate");
    assert_eq!(
        artifact["cancel_candidate_source"],
        "production_mutation_orphan_order_detector"
    );
    assert_eq!(artifact["order_lineage_id"], "lineage-v160-single-shot");
    assert_eq!(artifact["orphan_risk_detected"], true);
    assert_eq!(artifact["risk_halted"], true);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert_eq!(artifact["cancel_request_preview_ready"], true);
    assert_eq!(artifact["order_identifier_known"], true);
    assert!(
        artifact["known_order_id"]
            .as_str()
            .unwrap()
            .starts_with("order_id:sha256:")
    );
    assert!(
        artifact["known_client_order_id"]
            .as_str()
            .unwrap()
            .starts_with("client_order_id:sha256:")
    );
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["account_label"], "prod-account-redacted");
    assert_eq!(artifact["cancel_reason"], "orphan_risk_detected");
    assert_eq!(artifact["candidate_count"], 1);
    assert_eq!(artifact["multi_order_cancel_requested"], false);
    assert_eq!(artifact["cancel_all_requested"], false);
    assert_eq!(artifact["bulk_cancel_requested"], false);
    assert_eq!(artifact["strategy_driven_cancel_requested"], false);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(
        artifact["orphan_order_detector_ref"]["schema_version"],
        PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["reconciliation_classifier_ref"]["schema_version"],
        PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["exchange_readback_mapper_ref"]["schema_version"],
        PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION
    );
}

#[test]
fn production_mutation_cancel_request_preview_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-003-cancel-request-preview-missing-gates-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let orphan = write_v180_cancel_request_preview_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let output = output_dir.join("cancel-request-preview-missing-gates.json");

    run_live_production_mutation_cancel_request_preview(
        &production_mutation_cancel_request_preview_opt(orphan, output.clone(), false),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["cancel_request_preview_ready"], false);
    assert_eq!(artifact["orphan_risk_detected"], true);
    assert_eq!(artifact["risk_halted"], true);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-cancel-request-preview")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-network")
    );
}

#[test]
fn production_mutation_cancel_request_preview_blocks_clean_non_orphan_source() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-003-cancel-request-preview-clean-source-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let orphan = write_v180_cancel_request_preview_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "FILLED",
            exchange_order_state: "filled",
            order_found: true,
            open_order_observed: false,
            terminal_state_observed: true,
        },
    );
    let output = output_dir.join("cancel-request-preview-clean-source.json");

    run_live_production_mutation_cancel_request_preview(
        &production_mutation_cancel_request_preview_opt(orphan, output.clone(), true),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_source_artifact");
    assert_eq!(artifact["cancel_request_preview_ready"], false);
    assert_eq!(artifact["orphan_risk_detected"], false);
    assert_eq!(artifact["risk_halted"], false);
    assert_eq!(artifact["manual_review_required"], false);
    assert_eq!(artifact["new_orders_blocked"], false);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "orphan_order_detector_orphan_risk_detected_not_true")
    );
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "orphan_order_detector_risk_halted_not_true")
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
}

#[test]
fn production_mutation_cancel_risk_gate_builds_scoped_ready_gate() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-004-cancel-risk-gate-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let orphan = write_v180_cancel_request_preview_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let preview = output_dir.join("cancel-request-preview.json");
    let gate = output_dir.join("cancel-risk-gate.json");

    run_live_production_mutation_cancel_request_preview(
        &production_mutation_cancel_request_preview_opt(orphan, preview.clone(), true),
    )
    .unwrap();
    run_live_production_mutation_cancel_risk_gate(&production_mutation_cancel_risk_gate_opt(
        preview,
        gate.clone(),
        "BTCUSDT",
        "prod-account-redacted",
        true,
    ))
    .unwrap();

    let body = fs::read_to_string(gate).unwrap();
    assert!(!body.contains("123456789"));
    assert!(!body.contains("owner-approved-v160-single-shot"));
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("signature="));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION
    );
    assert_eq!(artifact["artifact_type"], "cancel_risk_gate");
    assert_eq!(artifact["status"], "ready_cancel_risk_gate");
    assert_eq!(artifact["cancel_request_preview_ready"], true);
    assert_eq!(artifact["cancel_risk_gate_ready"], true);
    assert_eq!(artifact["orphan_risk_detected"], true);
    assert_eq!(artifact["risk_halted"], true);
    assert_eq!(artifact["manual_review_required"], true);
    assert_eq!(artifact["new_orders_blocked"], true);
    assert_eq!(artifact["lineage_scope"], "single_v16_mutation_candidate");
    assert_eq!(artifact["order_identifier_known"], true);
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["expected_symbol"], "BTCUSDT");
    assert_eq!(artifact["symbol_matches_lineage"], true);
    assert_eq!(artifact["account_label"], "prod-account-redacted");
    assert_eq!(artifact["expected_account_label"], "prod-account-redacted");
    assert_eq!(artifact["account_matches_lineage"], true);
    assert_eq!(artifact["owner_approval_required"], true);
    assert_eq!(artifact["owner_approval_lifecycle_recorded"], false);
    assert_eq!(artifact["candidate_count"], 1);
    assert_eq!(artifact["multi_order_cancel_requested"], false);
    assert_eq!(artifact["cancel_all_requested"], false);
    assert_eq!(artifact["bulk_cancel_requested"], false);
    assert_eq!(artifact["strategy_driven_cancel_requested"], false);
    assert_eq!(artifact["retry_requested"], false);
    assert_eq!(artifact["replace_or_amend_requested"], false);
    assert_eq!(artifact["flatten_requested"], false);
    assert_eq!(artifact["dashboard_cancel_requested"], false);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
    assert_eq!(artifact["automatic_cancel_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(
        artifact["cancel_request_preview_ref"]["schema_version"],
        PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_SCHEMA_VERSION
    );
}

#[test]
fn production_mutation_cancel_risk_gate_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-004-cancel-risk-gate-missing-gates-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let orphan = write_v180_cancel_request_preview_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let preview = output_dir.join("cancel-request-preview.json");
    let gate = output_dir.join("cancel-risk-gate-missing-gates.json");

    run_live_production_mutation_cancel_request_preview(
        &production_mutation_cancel_request_preview_opt(orphan, preview.clone(), true),
    )
    .unwrap();
    run_live_production_mutation_cancel_risk_gate(&production_mutation_cancel_risk_gate_opt(
        preview,
        gate.clone(),
        "BTCUSDT",
        "prod-account-redacted",
        false,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(gate).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["cancel_request_preview_ready"], true);
    assert_eq!(artifact["cancel_risk_gate_ready"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-cancel-risk-gate")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-symbol-account-scope")
    );
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
}

#[test]
fn production_mutation_cancel_risk_gate_blocks_scope_and_forbidden_controls() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-004-cancel-risk-gate-blocked-source-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let orphan = write_v180_cancel_request_preview_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let preview = output_dir.join("cancel-request-preview.json");
    let gate = output_dir.join("cancel-risk-gate-blocked-source.json");

    run_live_production_mutation_cancel_request_preview(
        &production_mutation_cancel_request_preview_opt(orphan, preview.clone(), true),
    )
    .unwrap();
    let mut preview_artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&preview).unwrap()).unwrap();
    preview_artifact["cancel_all_requested"] = json!(true);
    preview_artifact["retry_requested"] = json!(true);
    preview_artifact["replace_or_amend_requested"] = json!(true);
    preview_artifact["flatten_requested"] = json!(true);
    preview_artifact["dashboard_cancel_requested"] = json!(true);
    fs::write(
        &preview,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&preview_artifact).unwrap()
        ),
    )
    .unwrap();

    run_live_production_mutation_cancel_risk_gate(&production_mutation_cancel_risk_gate_opt(
        preview,
        gate.clone(),
        "ETHUSDT",
        "other-account",
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(gate).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_source_artifact");
    assert_eq!(artifact["cancel_risk_gate_ready"], false);
    assert_eq!(artifact["symbol_matches_lineage"], false);
    assert_eq!(artifact["account_matches_lineage"], false);
    assert_eq!(artifact["cancel_all_requested"], true);
    assert_eq!(artifact["retry_requested"], true);
    assert_eq!(artifact["replace_or_amend_requested"], true);
    assert_eq!(artifact["flatten_requested"], true);
    assert_eq!(artifact["dashboard_cancel_requested"], true);
    for expected in [
        "symbol_mismatch",
        "account_label_mismatch",
        "cancel_request_preview_cancel_all_requested_true",
        "cancel_request_preview_retry_requested_true",
        "cancel_request_preview_replace_or_amend_requested_true",
        "cancel_request_preview_flatten_requested_true",
        "cancel_request_preview_dashboard_cancel_requested_true",
    ] {
        assert!(
            artifact["source_artifact_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == expected),
            "missing issue {expected}"
        );
    }
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
}

#[test]
fn production_mutation_manual_owner_approval_lifecycle_records_one_time_approval() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-005-manual-owner-approval-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let output = output_dir.join("manual-owner-approval-lifecycle.json");

    run_live_production_mutation_manual_owner_approval_lifecycle(
        &production_mutation_manual_owner_approval_lifecycle_opt(
            risk_gate,
            output.clone(),
            "approved",
            true,
        ),
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("123456789"));
    assert!(!body.contains("owner-approved-v160-single-shot"));
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("signature="));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION
    );
    assert_eq!(artifact["artifact_type"], "manual_owner_approval_lifecycle");
    assert_eq!(
        artifact["status"],
        "approval_lifecycle_recorded_for_cancel_candidate"
    );
    assert_eq!(artifact["cancel_risk_gate_ready"], true);
    assert_eq!(artifact["approval_scope"], "one_order_cancel_candidate");
    assert_eq!(artifact["approval_source"], "owner_manual_action");
    assert_eq!(artifact["approval_state"], "approved");
    assert_eq!(artifact["manual_approval_recorded"], true);
    assert_eq!(artifact["approval_expires"], true);
    assert_eq!(artifact["approval_expired"], false);
    assert_eq!(artifact["approval_revoked"], false);
    assert_eq!(artifact["approval_used"], false);
    assert_eq!(artifact["approval_reusable"], false);
    assert_eq!(artifact["one_time_approval"], true);
    assert_eq!(artifact["approval_lifecycle_valid"], true);
    assert_eq!(artifact["owner_approval_required"], true);
    assert_eq!(artifact["owner_approval_lifecycle_recorded"], true);
    assert_eq!(artifact["approval_consumed"], false);
    assert_eq!(artifact["approval_consumed_before_send"], false);
    assert_eq!(artifact["approval_consumed_after_send"], false);
    assert_eq!(artifact["candidate_count"], 1);
    assert_eq!(artifact["strategy_auto_approval_allowed"], false);
    assert_eq!(artifact["background_auto_approval_allowed"], false);
    assert_eq!(artifact["dashboard_auto_approval_allowed"], false);
    assert_eq!(artifact["incident_handler_auto_approval_allowed"], false);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["lifecycle_issues"].as_array().unwrap().len(), 0);
    assert_eq!(
        artifact["cancel_risk_gate_ref"]["schema_version"],
        PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION
    );
}

#[test]
fn production_mutation_manual_owner_approval_lifecycle_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-005-manual-owner-approval-missing-gates-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let output = output_dir.join("manual-owner-approval-missing-gates.json");

    run_live_production_mutation_manual_owner_approval_lifecycle(
        &production_mutation_manual_owner_approval_lifecycle_opt(
            risk_gate,
            output.clone(),
            "approved",
            false,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["approval_lifecycle_valid"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["lifecycle_issues"].as_array().unwrap().len(), 0);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| { flag == "--allow-production-mutation-manual-owner-approval-lifecycle" })
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-strategy-auto-approval")
    );
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
}

#[test]
fn production_mutation_manual_owner_approval_lifecycle_blocks_invalid_state_and_source() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-005-manual-owner-approval-blocked-source-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let expired_output = output_dir.join("manual-owner-approval-expired.json");
    let used_output = output_dir.join("manual-owner-approval-used.json");
    let source_output = output_dir.join("manual-owner-approval-source.json");

    let mut expired_opt = production_mutation_manual_owner_approval_lifecycle_opt(
        risk_gate.clone(),
        expired_output.clone(),
        "approved",
        true,
    );
    expired_opt.now_unix_ms = 1_718_400_070_000;
    run_live_production_mutation_manual_owner_approval_lifecycle(&expired_opt).unwrap();
    run_live_production_mutation_manual_owner_approval_lifecycle(
        &production_mutation_manual_owner_approval_lifecycle_opt(
            risk_gate.clone(),
            used_output.clone(),
            "used",
            true,
        ),
    )
    .unwrap();

    let mut gate_artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&risk_gate).unwrap()).unwrap();
    gate_artifact["owner_approval_lifecycle_recorded"] = json!(true);
    fs::write(
        &risk_gate,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&gate_artifact).unwrap()
        ),
    )
    .unwrap();
    run_live_production_mutation_manual_owner_approval_lifecycle(
        &production_mutation_manual_owner_approval_lifecycle_opt(
            risk_gate,
            source_output.clone(),
            "approved",
            true,
        ),
    )
    .unwrap();

    let expired: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(expired_output).unwrap()).unwrap();
    assert_eq!(expired["status"], "approval_expired");
    assert_eq!(expired["approval_lifecycle_valid"], false);
    assert!(
        expired["lifecycle_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "approval_expired")
    );
    assert_eq!(expired["actual_cancel_send_allowed"], false);
    assert_eq!(expired["approval_consumed_after_send"], false);

    let used: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(used_output).unwrap()).unwrap();
    assert_eq!(used["status"], "approval_used");
    assert_eq!(used["approval_lifecycle_valid"], false);
    assert!(
        used["lifecycle_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "approval_used")
    );
    assert_eq!(used["approval_reusable"], false);
    assert_eq!(used["actual_cancel_send_allowed"], false);

    let source: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(source_output).unwrap()).unwrap();
    assert_eq!(source["status"], "blocked_source_artifact");
    assert_eq!(source["approval_lifecycle_valid"], false);
    assert!(
        source["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "cancel_risk_gate_owner_approval_lifecycle_already_recorded")
    );
    assert_eq!(source["actual_cancel_send_allowed"], false);
    assert_eq!(source["network_cancel_endpoint_attempted"], false);
}

#[test]
fn production_mutation_actual_cancel_owner_approval_lifecycle_authorizes_single_use_scope() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-003-actual-cancel-owner-approval-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (safety_contract, release_manifest) =
        write_v190_actual_cancel_owner_approval_source_files(&output_dir);
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let output = output_dir.join("actual-cancel-owner-approval-lifecycle.json");

    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
        &production_mutation_actual_cancel_owner_approval_lifecycle_opt(
            safety_contract,
            release_manifest,
            risk_gate,
            output.clone(),
            "approved",
            true,
        ),
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("123456789"));
    assert!(!body.contains("owner-approved-v160-single-shot"));
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("signature="));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["artifact_type"],
        "actual_cancel_owner_approval_lifecycle"
    );
    assert_eq!(artifact["status"], "approval_execution_lifecycle_ready");
    assert_eq!(
        artifact["capability"],
        "Owner-Approved Single-Shot Actual Cancel"
    );
    assert_eq!(
        artifact["execution_mode"],
        "owner_approved_single_shot_manual_only"
    );
    assert_eq!(
        artifact["approval_scope"],
        "one_order_one_venue_one_attempt"
    );
    assert_eq!(artifact["approval_state"], "approved");
    assert_eq!(artifact["approval_lifecycle_valid"], true);
    assert_eq!(artifact["approval_execution_authorized"], true);
    assert_eq!(artifact["manual_approval_recorded"], true);
    assert_eq!(artifact["approval_reusable"], false);
    assert_eq!(artifact["one_time_approval"], true);
    assert_eq!(artifact["single_order_required"], true);
    assert_eq!(artifact["single_venue_required"], true);
    assert_eq!(artifact["single_execution_attempt_required"], true);
    assert_eq!(artifact["approval_consumed"], false);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert_eq!(artifact["dashboard_auto_approval_allowed"], false);
    assert_eq!(artifact["bulk_cancel_allowed"], false);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(
        artifact["production_order_submit_lifecycle_included"],
        false
    );
    assert_eq!(artifact["venue"], "binance_spot");
    assert_eq!(artifact["release_manifest_product_version"], "v0.18.1");
    assert_eq!(
        artifact["release_manifest_planned_tag"],
        "ntpro-rust-only-v0.18.1"
    );
    assert_eq!(
        artifact["release_manifest_actual_cancel_scope"],
        "not_included"
    );
    assert_eq!(artifact["safety_contract_ready"], true);
    assert_eq!(artifact["release_provenance_ready"], true);
    assert_eq!(artifact["cancel_risk_gate_ready"], true);
    assert_eq!(
        artifact["safety_contract_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(
        artifact["release_manifest_issues"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["lifecycle_issues"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
}

#[test]
fn production_mutation_actual_cancel_owner_approval_lifecycle_blocks_missing_gates_and_owner() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-003-actual-cancel-owner-approval-missing-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (safety_contract, release_manifest) =
        write_v190_actual_cancel_owner_approval_source_files(&output_dir);
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let missing_gates_output = output_dir.join("missing-gates.json");
    let missing_owner_output = output_dir.join("missing-owner.json");

    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
        &production_mutation_actual_cancel_owner_approval_lifecycle_opt(
            safety_contract.clone(),
            release_manifest.clone(),
            risk_gate.clone(),
            missing_gates_output.clone(),
            "approved",
            false,
        ),
    )
    .unwrap();
    let mut missing_owner_opt = production_mutation_actual_cancel_owner_approval_lifecycle_opt(
        safety_contract,
        release_manifest,
        risk_gate,
        missing_owner_output.clone(),
        "approved",
        true,
    );
    missing_owner_opt.manual_approval_id = None;
    missing_owner_opt.approved_by = None;
    missing_owner_opt.approval_reason = None;
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(&missing_owner_opt)
        .unwrap();

    let missing_gates: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(missing_gates_output).unwrap()).unwrap();
    assert_eq!(missing_gates["status"], "blocked_missing_gate");
    assert_eq!(missing_gates["approval_execution_authorized"], false);
    assert!(
        missing_gates["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| {
                flag == "--allow-production-mutation-actual-cancel-owner-approval-lifecycle"
            })
    );
    assert_eq!(missing_gates["actual_cancel_send_allowed"], false);

    let missing_owner: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(missing_owner_output).unwrap()).unwrap();
    assert_eq!(missing_owner["status"], "approval_invalid");
    assert_eq!(missing_owner["approval_execution_authorized"], false);
    assert!(
        missing_owner["lifecycle_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "missing_owner_approval")
    );
    assert_eq!(
        missing_owner["approval_failure_reason"],
        "missing_owner_approval"
    );
    assert_eq!(missing_owner["actual_cancel_send_allowed"], false);
}

#[test]
fn production_mutation_actual_cancel_owner_approval_lifecycle_blocks_reuse_expiry_and_mismatch() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-003-actual-cancel-owner-approval-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (safety_contract, release_manifest) =
        write_v190_actual_cancel_owner_approval_source_files(&output_dir);
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );

    let expired_output = output_dir.join("expired.json");
    let used_output = output_dir.join("used.json");
    let rejected_output = output_dir.join("rejected.json");
    let release_mismatch_output = output_dir.join("release-mismatch.json");
    let symbol_mismatch_output = output_dir.join("symbol-mismatch.json");

    let mut expired_opt = production_mutation_actual_cancel_owner_approval_lifecycle_opt(
        safety_contract.clone(),
        release_manifest.clone(),
        risk_gate.clone(),
        expired_output.clone(),
        "approved",
        true,
    );
    expired_opt.now_unix_ms = 1_718_400_070_000;
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(&expired_opt).unwrap();
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
        &production_mutation_actual_cancel_owner_approval_lifecycle_opt(
            safety_contract.clone(),
            release_manifest.clone(),
            risk_gate.clone(),
            used_output.clone(),
            "used",
            true,
        ),
    )
    .unwrap();
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
        &production_mutation_actual_cancel_owner_approval_lifecycle_opt(
            safety_contract.clone(),
            release_manifest.clone(),
            risk_gate.clone(),
            rejected_output.clone(),
            "rejected",
            true,
        ),
    )
    .unwrap();
    let mut release_mismatch_opt = production_mutation_actual_cancel_owner_approval_lifecycle_opt(
        safety_contract.clone(),
        release_manifest,
        risk_gate.clone(),
        release_mismatch_output.clone(),
        "approved",
        true,
    );
    release_mismatch_opt.expected_release_tag = "ntpro-rust-only-v0.18.0".to_string();
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(&release_mismatch_opt)
        .unwrap();
    let mut symbol_mismatch_opt = production_mutation_actual_cancel_owner_approval_lifecycle_opt(
        safety_contract,
        write_v190_actual_cancel_owner_approval_source_files(&output_dir).1,
        risk_gate,
        symbol_mismatch_output.clone(),
        "approved",
        true,
    );
    symbol_mismatch_opt.expected_symbol = "ETHUSDT".to_string();
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(&symbol_mismatch_opt)
        .unwrap();

    let expired: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(expired_output).unwrap()).unwrap();
    assert_eq!(expired["status"], "approval_expired");
    assert_eq!(expired["approval_execution_authorized"], false);
    assert!(
        expired["lifecycle_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "owner_approval_expired")
    );

    let used: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(used_output).unwrap()).unwrap();
    assert_eq!(used["status"], "approval_used");
    assert_eq!(used["approval_consumed"], true);
    assert_eq!(used["audit_evidence_recorded"], true);
    assert!(
        used["lifecycle_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "owner_approval_reused")
    );

    let rejected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(rejected_output).unwrap()).unwrap();
    assert_eq!(rejected["status"], "approval_rejected");
    assert_eq!(rejected["audit_evidence_recorded"], true);
    assert!(
        rejected["lifecycle_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "owner_approval_rejected")
    );

    let release_mismatch: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(release_mismatch_output).unwrap()).unwrap();
    assert_eq!(release_mismatch["status"], "blocked_release_provenance");
    assert!(
        release_mismatch["release_manifest_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "release_manifest_planned_tag_mismatch")
    );

    let symbol_mismatch: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(symbol_mismatch_output).unwrap()).unwrap();
    assert_eq!(symbol_mismatch["status"], "blocked_source_artifact");
    assert!(
        symbol_mismatch["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "symbol_mismatch")
    );
    assert_eq!(symbol_mismatch["actual_cancel_send_allowed"], false);
}

#[test]
fn production_mutation_actual_cancel_executor_adapter_boundary_records_ready_contract() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-005-actual-cancel-adapter-boundary-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let owner_approval =
        write_ready_v190_actual_cancel_owner_approval_lifecycle_artifact(&output_dir);
    let adapter_capability = write_v190_actual_cancel_adapter_capability(
        &output_dir,
        "adapter-capability.json",
        true,
        &["binance_spot"],
        &["exchange_order_id"],
    );
    let output = output_dir.join("actual-cancel-executor-adapter-boundary.json");

    run_live_production_mutation_actual_cancel_executor_adapter_boundary(
        &production_mutation_actual_cancel_executor_adapter_boundary_opt(
            owner_approval,
            adapter_capability,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("123456789"));
    assert!(!body.contains("owner-approved-v160-single-shot"));
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("signature="));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_ACTUAL_CANCEL_EXECUTOR_ADAPTER_BOUNDARY_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["artifact_type"],
        "actual_cancel_executor_adapter_boundary"
    );
    assert_eq!(artifact["status"], "adapter_boundary_ready");
    assert_eq!(artifact["adapter_boundary_ready"], true);
    assert_eq!(
        artifact["actual_cancel_send_allowed_by_adapter_boundary"],
        true
    );
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["bulk_cancel_allowed"], false);
    assert_eq!(artifact["cancel_all_allowed"], false);
    assert_eq!(artifact["dashboard_execution_allowed"], false);
    assert_eq!(artifact["adapter_id"], "binance_spot_cancel_adapter");
    assert_eq!(artifact["venue"], "binance_spot");
    assert_eq!(artifact["order_id_type"], "exchange_order_id");
    assert_eq!(artifact["max_cancel_requests"], 1);
    assert_eq!(artifact["allowed_attempts"], 1);
    assert_eq!(artifact["allowed_order_count"], 1);
    assert_eq!(artifact["allowed_venue_count"], 1);
    assert_eq!(artifact["request_contract_ready"], true);
    assert_eq!(artifact["response_contract_ready"], true);
    assert_eq!(artifact["readback_contract_ready"], true);
    assert_eq!(artifact["audit_contract_ready"], true);
    assert_eq!(
        artifact["cancel_request_contract"],
        "single_order_cancel_request_v1"
    );
    assert_eq!(
        artifact["cancel_response_contract"],
        "single_order_cancel_response_metadata_v1"
    );
    assert_eq!(
        artifact["post_cancel_readback_contract"],
        "single_order_post_cancel_readback_required_v1"
    );
    assert_eq!(
        artifact["audit_contract"],
        "single_order_cancel_audit_event_required_v1"
    );
    for expected in [
        "rejected",
        "timeout",
        "unknown",
        "already_cancelled",
        "venue_unavailable",
        "transport_failure",
    ] {
        assert!(
            artifact["adapter_failure_taxonomy"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item == expected)
        );
    }
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(
        artifact["adapter_capability_issues"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
}

#[test]
fn production_mutation_actual_cancel_executor_adapter_boundary_blocks_capability_mismatch() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-005-actual-cancel-adapter-boundary-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let owner_approval =
        write_ready_v190_actual_cancel_owner_approval_lifecycle_artifact(&output_dir);
    let unsupported_capability = write_v190_actual_cancel_adapter_capability(
        &output_dir,
        "adapter-capability-unsupported.json",
        false,
        &["binance_spot"],
        &["exchange_order_id"],
    );
    let unsupported_venue = write_v190_actual_cancel_adapter_capability(
        &output_dir,
        "adapter-capability-venue-mismatch.json",
        true,
        &["okx_spot"],
        &["exchange_order_id"],
    );
    let unsupported_order_id_type = write_v190_actual_cancel_adapter_capability(
        &output_dir,
        "adapter-capability-order-id-mismatch.json",
        true,
        &["binance_spot"],
        &["client_order_id"],
    );
    let unsupported_output = output_dir.join("unsupported.json");
    let venue_output = output_dir.join("venue-mismatch.json");
    let order_id_output = output_dir.join("order-id-mismatch.json");

    run_live_production_mutation_actual_cancel_executor_adapter_boundary(
        &production_mutation_actual_cancel_executor_adapter_boundary_opt(
            owner_approval.clone(),
            unsupported_capability,
            unsupported_output.clone(),
            true,
        ),
    )
    .unwrap();
    run_live_production_mutation_actual_cancel_executor_adapter_boundary(
        &production_mutation_actual_cancel_executor_adapter_boundary_opt(
            owner_approval.clone(),
            unsupported_venue,
            venue_output.clone(),
            true,
        ),
    )
    .unwrap();
    run_live_production_mutation_actual_cancel_executor_adapter_boundary(
        &production_mutation_actual_cancel_executor_adapter_boundary_opt(
            owner_approval,
            unsupported_order_id_type,
            order_id_output.clone(),
            true,
        ),
    )
    .unwrap();

    let unsupported: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(unsupported_output).unwrap()).unwrap();
    assert_eq!(unsupported["status"], "blocked_adapter_capability");
    assert_eq!(unsupported["adapter_boundary_ready"], false);
    assert_eq!(
        unsupported["actual_cancel_send_allowed_by_adapter_boundary"],
        false
    );
    assert!(
        unsupported["adapter_capability_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "adapter_actual_cancel_unsupported")
    );
    assert_eq!(unsupported["actual_cancel_send_allowed"], false);
    assert_eq!(unsupported["cancel_attempted"], false);

    let venue: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(venue_output).unwrap()).unwrap();
    assert_eq!(venue["status"], "blocked_adapter_capability");
    assert!(
        venue["adapter_capability_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "adapter_venue_unsupported")
    );
    assert_eq!(venue["network_attempted"], false);

    let order_id: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(order_id_output).unwrap()).unwrap();
    assert_eq!(order_id["status"], "blocked_adapter_capability");
    assert!(
        order_id["adapter_capability_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "adapter_order_id_type_unsupported")
    );
    assert_eq!(order_id["retry_attempted"], false);
}

#[test]
fn production_mutation_actual_cancel_executor_adapter_boundary_blocks_missing_gates_and_owner() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-005-actual-cancel-adapter-boundary-missing-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (safety_contract, release_manifest) =
        write_v190_actual_cancel_owner_approval_source_files(&output_dir);
    let risk_gate = write_v180_manual_owner_approval_lifecycle_source_chain(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let created_owner_approval = output_dir.join("owner-approval-created.json");
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
        &production_mutation_actual_cancel_owner_approval_lifecycle_opt(
            safety_contract,
            release_manifest,
            risk_gate,
            created_owner_approval.clone(),
            "created",
            true,
        ),
    )
    .unwrap();
    let adapter_capability = write_v190_actual_cancel_adapter_capability(
        &output_dir,
        "adapter-capability-ready.json",
        true,
        &["binance_spot"],
        &["exchange_order_id"],
    );
    let missing_gate_output = output_dir.join("missing-gates.json");
    let blocked_owner_output = output_dir.join("blocked-owner.json");

    run_live_production_mutation_actual_cancel_executor_adapter_boundary(
        &production_mutation_actual_cancel_executor_adapter_boundary_opt(
            created_owner_approval.clone(),
            adapter_capability.clone(),
            missing_gate_output.clone(),
            false,
        ),
    )
    .unwrap();
    run_live_production_mutation_actual_cancel_executor_adapter_boundary(
        &production_mutation_actual_cancel_executor_adapter_boundary_opt(
            created_owner_approval,
            adapter_capability,
            blocked_owner_output.clone(),
            true,
        ),
    )
    .unwrap();

    let missing_gate: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(missing_gate_output).unwrap()).unwrap();
    assert_eq!(missing_gate["status"], "blocked_missing_gate");
    assert!(
        missing_gate["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| {
                flag == "--allow-production-mutation-actual-cancel-executor-adapter-boundary"
            })
    );
    assert_eq!(missing_gate["adapter_boundary_ready"], false);

    let blocked_owner: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(blocked_owner_output).unwrap()).unwrap();
    assert_eq!(blocked_owner["status"], "blocked_owner_approval_lifecycle");
    assert!(
        blocked_owner["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "owner_approval_not_authorized")
    );
    assert_eq!(
        blocked_owner["actual_cancel_send_allowed_by_adapter_boundary"],
        false
    );
    assert_eq!(blocked_owner["network_attempted"], false);
}

#[test]
fn production_mutation_actual_cancel_single_shot_records_offline_ready_without_network() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-004-actual-cancel-single-shot-offline-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let sources = write_ready_v190_actual_cancel_single_shot_source_chain(&output_dir);
    let output = output_dir.join("actual-cancel-single-shot-offline.json");

    run_live_production_mutation_actual_cancel_single_shot(
        &production_mutation_actual_cancel_single_shot_opt(&sources, output.clone(), false, true),
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("123456789"));
    assert!(!body.contains("owner-approved-v160-single-shot"));
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("signature="));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_SCHEMA_VERSION
    );
    assert_eq!(artifact["artifact_type"], "actual_cancel_single_shot");
    assert_eq!(
        artifact["status"],
        "ready_actual_cancel_command_offline_no_send"
    );
    assert_eq!(
        artifact["capability"],
        "Historical Actual Cancel Artifact Evaluation"
    );
    assert_eq!(artifact["execution_mode"], "offline_only_executor_retired");
    assert_eq!(artifact["mode"], "retired_actual_cancel_offline_evaluation");
    assert_eq!(artifact["manual_online_requested"], false);
    assert_eq!(artifact["actual_cancel_command_ready"], true);
    assert_eq!(artifact["single_shot_cancel_allowed"], false);
    assert_eq!(artifact["owner_approval_ready"], true);
    assert_eq!(artifact["risk_gate_ready"], true);
    assert_eq!(artifact["release_provenance_ready"], true);
    assert_eq!(artifact["adapter_boundary_ready"], true);
    assert_eq!(artifact["adapter_capability_ready"], true);
    assert_eq!(artifact["request_method"], TESTNET_ORDER_METHOD_DELETE);
    assert_eq!(artifact["request_target"], TESTNET_ORDER_ENDPOINT_ORDER);
    assert_eq!(
        artifact["request_contract"],
        "single_order_cancel_request_v1"
    );
    assert_eq!(artifact["adapter_id"], "binance_spot_cancel_adapter");
    assert_eq!(artifact["venue"], "binance_spot");
    assert_eq!(artifact["order_id_type"], "exchange_order_id");
    assert!(
        artifact["cancel_order_identifier_ref"]
            .as_str()
            .unwrap()
            .starts_with("order_id:sha256:")
    );
    assert_eq!(artifact["approval_consumed_before_send"], false);
    assert_eq!(artifact["approval_consumed_after_send"], false);
    assert_eq!(artifact["approval_state_before_attempt"], "approved");
    assert_eq!(artifact["approval_state_after_attempt"], "approved");
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["http_send_attempted"], false);
    assert_eq!(artifact["venue_ack_observed"], false);
    assert_eq!(artifact["readback_required"], false);
    assert_eq!(
        artifact["readback_requirement"],
        "not_required_without_send_attempt"
    );
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["bulk_cancel_allowed"], false);
    assert_eq!(artifact["cancel_all_allowed"], false);
    assert_eq!(artifact["dashboard_execution_allowed"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert_eq!(artifact["credential_material"], "retired_not_read");
    assert_eq!(artifact["api_key_env"], "retired");
    assert_eq!(artifact["api_secret_env"], "retired");
    assert_eq!(artifact["production_signing_material_gate_required"], false);
    assert_eq!(artifact["production_signing_material_gate_open"], false);
    assert_eq!(artifact["production_signing_material_env_read"], false);
    assert!(
        artifact["production_signing_material_missing_gate_env_vars"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "production_mutation_executor_retired_after_v0.32.0")
    );
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["api_key_header_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_body_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["venue_response_status"], "not_attempted");
    assert_eq!(
        artifact["venue_response_source"],
        "executor_retired_offline"
    );
    assert_eq!(
        artifact["venue_response_error_code"],
        "not_attempted_executor_retired"
    );
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(
        artifact["adapter_capability_issues"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        artifact["safety_contract_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(
        artifact["release_manifest_issues"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
}

#[test]
fn production_mutation_actual_cancel_single_shot_historical_online_selector_stays_offline() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-004-actual-cancel-single-shot-attempt-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let sources = write_ready_v190_actual_cancel_single_shot_source_chain(&output_dir);
    let output = output_dir.join("actual-cancel-single-shot-attempt.json");
    let opt =
        production_mutation_actual_cancel_single_shot_opt(&sources, output.clone(), true, true);
    let owner_approval_before = fs::read_to_string(&sources.owner_approval_lifecycle).unwrap();
    let artifact = build_production_mutation_actual_cancel_single_shot_artifact(&opt).unwrap();
    write_production_mutation_actual_cancel_single_shot_artifact(&output, &artifact).unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("123456789"));
    assert!(!body.contains("owner-approved-v160-single-shot"));
    assert!(!body.contains("ntpro-v190004-api-key"));
    assert!(!body.contains("ntpro-v190004-api-secret"));
    assert!(!body.contains("signature="));
    assert!(!body.contains("X-MBX-APIKEY"));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["status"],
        "ready_actual_cancel_command_offline_no_send"
    );
    assert_eq!(artifact["manual_online_requested"], false);
    assert_eq!(artifact["actual_cancel_command_ready"], true);
    assert_eq!(artifact["single_shot_cancel_allowed"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["http_send_attempted"], false);
    assert_eq!(artifact["venue_ack_observed"], false);
    assert_eq!(artifact["venue_response_status"], "not_attempted");
    assert_eq!(
        artifact["venue_response_source"],
        "executor_retired_offline"
    );
    assert_eq!(artifact["venue_response_code"], serde_json::Value::Null);
    assert_eq!(
        artifact["venue_response_error_code"],
        "not_attempted_executor_retired"
    );
    assert_eq!(artifact["latency_ms"], serde_json::Value::Null);
    assert_eq!(artifact["approval_consumed_before_send"], false);
    assert_eq!(artifact["approval_consumed_after_send"], false);
    assert_eq!(artifact["approval_state_before_attempt"], "approved");
    assert_eq!(artifact["approval_state_after_attempt"], "approved");
    assert_eq!(artifact["readback_required"], false);
    assert_eq!(
        artifact["readback_requirement"],
        "not_required_without_send_attempt"
    );
    assert!(
        artifact["local_audit_reference"]
            .as_str()
            .unwrap()
            .contains("v190-actual-cancel-single-shot")
    );
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["bulk_cancel_allowed"], false);
    assert_eq!(artifact["cancel_all_allowed"], false);
    assert_eq!(artifact["dashboard_execution_allowed"], false);
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["api_key_header_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["response_body_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(
        artifact["adapter_capability_issues"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["missing_env_vars"].as_array().unwrap().len(), 0);

    let owner_approval_after = fs::read_to_string(&sources.owner_approval_lifecycle).unwrap();
    assert_eq!(owner_approval_after, owner_approval_before);
    let unchanged_owner_approval: serde_json::Value =
        serde_json::from_str(&owner_approval_after).unwrap();
    assert_eq!(unchanged_owner_approval["approval_state"], "approved");
    assert_eq!(
        unchanged_owner_approval["approval_execution_authorized"],
        true
    );
    assert_eq!(unchanged_owner_approval["approval_consumed"], false);
    assert_eq!(
        unchanged_owner_approval["approval_consumed_before_send"],
        false
    );
    assert_eq!(
        unchanged_owner_approval["approval_consumed_after_send"],
        false
    );
    assert_eq!(unchanged_owner_approval["approval_used"], false);
    assert_eq!(unchanged_owner_approval["cancel_attempted"], false);
    assert_eq!(unchanged_owner_approval["cancel_requests_sent"], 0);
}

#[test]
fn production_mutation_actual_cancel_single_shot_blocks_gates_mismatch_reuse_and_adapter() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-004-actual-cancel-single-shot-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let sources = write_ready_v190_actual_cancel_single_shot_source_chain(&output_dir);

    let missing_gate_output = output_dir.join("missing-gates.json");
    run_live_production_mutation_actual_cancel_single_shot(
        &production_mutation_actual_cancel_single_shot_opt(
            &sources,
            missing_gate_output.clone(),
            false,
            false,
        ),
    )
    .unwrap();

    let release_mismatch_output = output_dir.join("release-mismatch.json");
    let mut release_mismatch_opt = production_mutation_actual_cancel_single_shot_opt(
        &sources,
        release_mismatch_output.clone(),
        false,
        true,
    );
    release_mismatch_opt.expected_release_tag = "ntpro-rust-only-v0.18.0".to_string();
    run_live_production_mutation_actual_cancel_single_shot(&release_mismatch_opt).unwrap();

    let reused_owner = output_dir.join("owner-approval-used.json");
    run_live_production_mutation_actual_cancel_owner_approval_lifecycle(
        &production_mutation_actual_cancel_owner_approval_lifecycle_opt(
            sources.actual_cancel_safety_contract.clone(),
            sources.release_manifest.clone(),
            sources.cancel_risk_gate.clone(),
            reused_owner.clone(),
            "used",
            true,
        ),
    )
    .unwrap();
    let mut reused_sources = sources.clone();
    reused_sources.owner_approval_lifecycle = reused_owner;
    let reused_output = output_dir.join("reused-owner.json");
    run_live_production_mutation_actual_cancel_single_shot(
        &production_mutation_actual_cancel_single_shot_opt(
            &reused_sources,
            reused_output.clone(),
            false,
            true,
        ),
    )
    .unwrap();

    let unsupported_capability = write_v190_actual_cancel_adapter_capability(
        &output_dir,
        "single-shot-adapter-capability-unsupported.json",
        false,
        &["binance_spot"],
        &["exchange_order_id"],
    );
    let mut unsupported_sources = sources.clone();
    unsupported_sources.adapter_capability = unsupported_capability;
    let unsupported_output = output_dir.join("unsupported-adapter.json");
    run_live_production_mutation_actual_cancel_single_shot(
        &production_mutation_actual_cancel_single_shot_opt(
            &unsupported_sources,
            unsupported_output.clone(),
            false,
            true,
        ),
    )
    .unwrap();

    let order_mismatch_output = output_dir.join("order-mismatch.json");
    let mut order_mismatch_opt = production_mutation_actual_cancel_single_shot_opt(
        &sources,
        order_mismatch_output.clone(),
        false,
        true,
    );
    order_mismatch_opt.cancel_order_id = Some("987654321".to_string());
    run_live_production_mutation_actual_cancel_single_shot(&order_mismatch_opt).unwrap();

    let missing_gate: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(missing_gate_output).unwrap()).unwrap();
    assert_eq!(missing_gate["status"], "blocked_missing_gate");
    assert_eq!(missing_gate["actual_cancel_command_ready"], false);
    assert_eq!(missing_gate["request_sent"], false);
    assert_eq!(missing_gate["cancel_attempted"], false);
    assert!(
        missing_gate["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-actual-cancel-single-shot")
    );

    let release_mismatch: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(release_mismatch_output).unwrap()).unwrap();
    assert_eq!(release_mismatch["status"], "blocked_release_provenance");
    assert_eq!(release_mismatch["request_sent"], false);
    assert!(
        release_mismatch["release_manifest_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "release_manifest_planned_tag_mismatch")
    );

    let reused: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(reused_output).unwrap()).unwrap();
    assert_eq!(reused["status"], "blocked_source_artifact");
    assert_eq!(reused["approval_state_before_attempt"], "used");
    assert_eq!(reused["approval_state_after_attempt"], "used");
    assert!(
        reused["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "owner_approval_reused")
    );
    assert_eq!(reused["request_sent"], false);
    assert_eq!(reused["readback_required"], false);

    let unsupported: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(unsupported_output).unwrap()).unwrap();
    assert_eq!(unsupported["status"], "blocked_adapter_capability");
    assert!(
        unsupported["adapter_capability_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "adapter_capability_missing")
    );
    assert_eq!(unsupported["cancel_attempted"], false);
    assert_eq!(unsupported["network_attempted"], false);

    let order_mismatch: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(order_mismatch_output).unwrap()).unwrap();
    assert_eq!(order_mismatch["status"], "blocked_source_artifact");
    assert!(
        order_mismatch["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "order_identity_mismatch")
    );
    assert_eq!(order_mismatch["single_shot_cancel_allowed"], false);
    assert_eq!(order_mismatch["request_sent"], false);
    assert_eq!(order_mismatch["cancel_attempted"], false);
}

#[test]
fn production_mutation_actual_cancel_readback_reconciliation_classifies_required_paths() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-006-actual-cancel-readback-reconciliation-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let actual_cancel_attempt =
        write_ready_v190_actual_cancel_single_shot_attempt_artifact(&output_dir);

    for (
        name,
        status,
        executed_qty,
        orig_qty,
        readback_result_hint,
        already_cancelled,
        expected_status,
        expected_result,
        expected_reconciliation_status,
        expected_partial_fill,
        expected_filled_before_cancel,
        expected_degraded,
        expected_followup_complete,
    ) in [
        (
            "confirmed",
            "CANCELED",
            "0",
            "1",
            None,
            false,
            "ready_actual_cancel_readback_cancel_confirmed",
            "cancel_confirmed",
            "ready_cancel_confirmed",
            false,
            false,
            false,
            true,
        ),
        (
            "already-cancelled",
            "CANCELED",
            "0",
            "1",
            Some("already_cancelled"),
            true,
            "ready_actual_cancel_readback_already_cancelled",
            "already_cancelled",
            "ready_already_cancelled",
            false,
            false,
            false,
            true,
        ),
        (
            "filled-before-cancel",
            "FILLED",
            "1",
            "1",
            None,
            false,
            "ready_actual_cancel_readback_filled_before_cancel",
            "filled_before_cancel",
            "ready_filled_before_cancel",
            false,
            true,
            false,
            false,
        ),
        (
            "unknown",
            "UNKNOWN",
            "0",
            "1",
            None,
            false,
            "degraded_actual_cancel_readback_unknown",
            "unknown",
            "degraded_unknown",
            false,
            false,
            true,
            false,
        ),
        (
            "timeout",
            "UNKNOWN",
            "0",
            "1",
            Some("timeout"),
            false,
            "degraded_actual_cancel_readback_timeout",
            "timeout",
            "degraded_timeout",
            false,
            false,
            true,
            false,
        ),
        (
            "partial-fill",
            "PARTIALLY_FILLED",
            "0.4",
            "1",
            None,
            false,
            "degraded_actual_cancel_readback_inconsistent",
            "inconsistent",
            "degraded_inconsistent",
            true,
            false,
            true,
            false,
        ),
    ] {
        let readback = output_dir.join(format!("readback-{name}.json"));
        let output = output_dir.join(format!("reconciliation-{name}.json"));
        write_synthetic_v190_actual_cancel_readback_reconciliation(
            &readback,
            status,
            executed_qty,
            orig_qty,
            readback_result_hint,
            already_cancelled,
        );

        run_live_production_mutation_actual_cancel_readback_reconciliation(
            &production_mutation_actual_cancel_readback_reconciliation_opt(
                actual_cancel_attempt.clone(),
                readback,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("123456789"));
        assert!(!body.contains("owner-approved-v160-single-shot"));
        assert!(!body.contains("ntpro-v190004-api-key"));
        assert!(!body.contains("ntpro-v190004-api-secret"));
        assert!(!body.contains("signature="));
        assert!(!body.contains("\"headers\""));
        assert!(!body.contains("\"payload\""));
        assert!(!body.contains("\"body\""));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_SCHEMA_VERSION
        );
        assert_eq!(
            artifact["artifact_type"],
            "actual_cancel_readback_reconciliation"
        );
        assert_eq!(artifact["status"], expected_status);
        assert_eq!(artifact["reconciliation_ready"], true);
        assert_eq!(artifact["readback_evidence_present"], true);
        assert_eq!(artifact["reconciliation_evidence_present"], true);
        assert_eq!(
            artifact["actual_cancel_followup_complete"],
            expected_followup_complete
        );
        assert_eq!(artifact["readback_result"], expected_result);
        assert_eq!(
            artifact["reconciliation_status"],
            expected_reconciliation_status
        );
        assert_eq!(artifact["partial_fill_observed"], expected_partial_fill);
        assert_eq!(
            artifact["filled_before_cancel_observed"],
            expected_filled_before_cancel
        );
        assert_eq!(artifact["already_cancelled_observed"], already_cancelled);
        assert_eq!(artifact["degraded"], expected_degraded);
        assert_eq!(artifact["error_state"], expected_degraded);
        assert_eq!(artifact["dashboard_read_only_consumable"], true);
        assert_eq!(artifact["dashboard_audit_view_ready"], true);
        assert_eq!(artifact["actual_cancel_attempt_recorded"], true);
        assert_eq!(artifact["actual_cancel_request_sent"], true);
        assert_eq!(artifact["readback_required"], true);
        assert_eq!(artifact["symbol"], "BTCUSDT");
        assert_eq!(artifact["account_label"], "prod-account-redacted");
        assert!(
            artifact["readback_order_id"]
                .as_str()
                .unwrap()
                .starts_with("readback_order_id:sha256:")
        );
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(
            artifact["readback_lineage_issues"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            artifact["forbidden_readback_markers"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_v190_actual_cancel_readback_reconciliation_false_boundary(&artifact);
    }
}

#[test]
fn production_mutation_actual_cancel_readback_reconciliation_blocks_bad_inputs() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-006-actual-cancel-readback-reconciliation-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let actual_cancel_attempt =
        write_ready_v190_actual_cancel_single_shot_attempt_artifact(&output_dir);
    let valid_readback = output_dir.join("readback-canceled.json");
    write_synthetic_v190_actual_cancel_readback_reconciliation(
        &valid_readback,
        "CANCELED",
        "0",
        "1",
        None,
        false,
    );

    let missing_flags_output = output_dir.join("reconciliation-missing-flags.json");
    run_live_production_mutation_actual_cancel_readback_reconciliation(
        &production_mutation_actual_cancel_readback_reconciliation_opt(
            actual_cancel_attempt.clone(),
            valid_readback.clone(),
            missing_flags_output.clone(),
            false,
        ),
    )
    .unwrap();

    let sources = write_ready_v190_actual_cancel_single_shot_source_chain(&output_dir);
    let no_attempt_output = output_dir.join("actual-cancel-no-send.json");
    run_live_production_mutation_actual_cancel_single_shot(
        &production_mutation_actual_cancel_single_shot_opt(
            &sources,
            no_attempt_output.clone(),
            false,
            true,
        ),
    )
    .unwrap();
    let invalid_source_output = output_dir.join("reconciliation-invalid-source.json");
    run_live_production_mutation_actual_cancel_readback_reconciliation(
        &production_mutation_actual_cancel_readback_reconciliation_opt(
            no_attempt_output,
            valid_readback,
            invalid_source_output.clone(),
            true,
        ),
    )
    .unwrap();

    let forbidden_readback = output_dir.join("forbidden-readback.json");
    write_forbidden_production_mutation_post_cancel_readback(&forbidden_readback);
    let forbidden_output = output_dir.join("reconciliation-forbidden.json");
    run_live_production_mutation_actual_cancel_readback_reconciliation(
        &production_mutation_actual_cancel_readback_reconciliation_opt(
            actual_cancel_attempt,
            forbidden_readback,
            forbidden_output.clone(),
            true,
        ),
    )
    .unwrap();

    let missing_flags: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(missing_flags_output).unwrap()).unwrap();
    assert_eq!(missing_flags["status"], "blocked_missing_gate");
    assert_eq!(missing_flags["reconciliation_ready"], false);
    assert!(
        missing_flags["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| {
                flag == "--allow-production-mutation-actual-cancel-readback-reconciliation"
            })
    );
    assert_v190_actual_cancel_readback_reconciliation_false_boundary(&missing_flags);

    let invalid_source: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(invalid_source_output).unwrap()).unwrap();
    assert_eq!(invalid_source["status"], "blocked_source_artifact");
    assert_eq!(invalid_source["reconciliation_ready"], false);
    assert!(
        invalid_source["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue
                == "actual_cancel_attempt_status_ready_actual_cancel_command_offline_no_send")
    );
    assert!(
        invalid_source["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "actual_cancel_attempt_request_sent_not_true")
    );
    assert_v190_actual_cancel_readback_reconciliation_false_boundary(&invalid_source);

    let forbidden: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(forbidden_output).unwrap()).unwrap();
    assert_eq!(forbidden["status"], "blocked_forbidden_readback_marker");
    assert_eq!(forbidden["reconciliation_ready"], false);
    assert!(
        forbidden["forbidden_readback_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.headers"))
    );
    assert!(
        forbidden["forbidden_readback_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.fills"))
    );
    assert_v190_actual_cancel_readback_reconciliation_false_boundary(&forbidden);
}

#[test]
fn production_mutation_actual_cancel_failure_evidence_classifies_required_outcomes() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-007-actual-cancel-failure-evidence-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let actual_cancel_attempt =
        write_ready_v190_actual_cancel_single_shot_attempt_artifact(&output_dir);

    for (
        name,
        status,
        executed_qty,
        orig_qty,
        readback_result_hint,
        already_cancelled,
        response_extra,
        audit_extra,
        expected_status,
        expected_outcome,
        expected_category,
        expected_recovered,
        expected_failed,
        expected_partial_success,
        expected_residual_risk_visible,
        expected_operator_action_required,
    ) in [
        (
            "cancel-confirmed",
            "CANCELED",
            "0",
            "1",
            None,
            false,
            serde_json::json!({}),
            serde_json::json!({}),
            "ready_actual_cancel_failure_recovered_cancel_confirmed",
            "cancel_confirmed",
            "recovered",
            true,
            false,
            false,
            false,
            false,
        ),
        (
            "already-cancelled",
            "CANCELED",
            "0",
            "1",
            Some("already_cancelled"),
            true,
            serde_json::json!({}),
            serde_json::json!({}),
            "ready_actual_cancel_failure_recovered_already_cancelled",
            "already_cancelled",
            "recovered",
            true,
            false,
            false,
            false,
            false,
        ),
        (
            "rejected",
            "REJECTED",
            "0",
            "1",
            None,
            false,
            serde_json::json!({}),
            serde_json::json!({}),
            "ready_actual_cancel_failure_rejected",
            "rejected",
            "failed",
            false,
            true,
            false,
            true,
            true,
        ),
        (
            "timeout",
            "UNKNOWN",
            "0",
            "1",
            Some("timeout"),
            false,
            serde_json::json!({}),
            serde_json::json!({}),
            "ready_actual_cancel_failure_timeout",
            "timeout",
            "failed",
            false,
            true,
            false,
            true,
            true,
        ),
        (
            "unknown",
            "UNKNOWN",
            "0",
            "1",
            None,
            false,
            serde_json::json!({}),
            serde_json::json!({}),
            "ready_actual_cancel_failure_unknown",
            "unknown",
            "failed",
            false,
            true,
            false,
            true,
            true,
        ),
        (
            "partial-fill",
            "PARTIALLY_FILLED",
            "0.4",
            "1",
            None,
            false,
            serde_json::json!({}),
            serde_json::json!({}),
            "ready_actual_cancel_partial_success_partial_fill",
            "partial_fill",
            "partial_success",
            false,
            false,
            true,
            true,
            true,
        ),
        (
            "filled-before-cancel",
            "FILLED",
            "1",
            "1",
            None,
            false,
            serde_json::json!({}),
            serde_json::json!({}),
            "ready_actual_cancel_partial_success_filled_before_cancel",
            "filled_before_cancel",
            "partial_success",
            false,
            false,
            true,
            true,
            true,
        ),
        (
            "venue-unavailable",
            "CANCELED",
            "0",
            "1",
            None,
            false,
            serde_json::json!({"venue_unavailable": true}),
            serde_json::json!({}),
            "ready_actual_cancel_failure_venue_unavailable",
            "venue_unavailable",
            "failed",
            false,
            true,
            false,
            true,
            true,
        ),
        (
            "adapter-failure",
            "CANCELED",
            "0",
            "1",
            None,
            false,
            serde_json::json!({}),
            serde_json::json!({"adapter_failure": true}),
            "ready_actual_cancel_failure_adapter_failure",
            "adapter_failure",
            "failed",
            false,
            true,
            false,
            true,
            true,
        ),
    ] {
        let raw_readback = output_dir.join(format!("failure-readback-{name}.json"));
        let reconciliation = output_dir.join(format!("failure-reconciliation-{name}.json"));
        let request_ref = output_dir.join(format!("request-ref-{name}.json"));
        let response_ref = output_dir.join(format!("response-ref-{name}.json"));
        let readback_ref = output_dir.join(format!("readback-ref-{name}.json"));
        let audit_ref = output_dir.join(format!("audit-ref-{name}.json"));
        let output = output_dir.join(format!("failure-evidence-{name}.json"));
        write_synthetic_v190_actual_cancel_readback_reconciliation(
            &raw_readback,
            status,
            executed_qty,
            orig_qty,
            readback_result_hint,
            already_cancelled,
        );
        run_live_production_mutation_actual_cancel_readback_reconciliation(
            &production_mutation_actual_cancel_readback_reconciliation_opt(
                actual_cancel_attempt.clone(),
                raw_readback,
                reconciliation.clone(),
                true,
            ),
        )
        .unwrap();
        write_synthetic_v190_actual_cancel_failure_evidence_ref(
            &request_ref,
            "actual_cancel_request_ref",
            "request_ref_recorded",
            &serde_json::json!({}),
        );
        write_synthetic_v190_actual_cancel_failure_evidence_ref(
            &response_ref,
            "actual_cancel_response_ref",
            "response_ref_recorded",
            &response_extra,
        );
        write_synthetic_v190_actual_cancel_failure_evidence_ref(
            &readback_ref,
            "actual_cancel_readback_ref",
            "readback_ref_recorded",
            &serde_json::json!({}),
        );
        write_synthetic_v190_actual_cancel_failure_evidence_ref(
            &audit_ref,
            "actual_cancel_audit_ref",
            "audit_ref_recorded",
            &audit_extra,
        );

        run_live_production_mutation_actual_cancel_failure_evidence(
            &production_mutation_actual_cancel_failure_evidence_opt(
                reconciliation,
                request_ref,
                response_ref,
                readback_ref,
                audit_ref,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("123456789"));
        assert!(!body.contains("owner-approved-v160-single-shot"));
        assert!(!body.contains("ntpro-v190004-api-key"));
        assert!(!body.contains("ntpro-v190004-api-secret"));
        assert!(!body.contains("signature="));
        assert!(!body.contains("\"headers\""));
        assert!(!body.contains("\"payload\""));
        assert!(!body.contains("\"body\""));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_ACTUAL_CANCEL_FAILURE_EVIDENCE_SCHEMA_VERSION
        );
        assert_eq!(artifact["artifact_type"], "actual_cancel_failure_evidence");
        assert_eq!(artifact["status"], expected_status);
        assert_eq!(artifact["evidence_ready"], true);
        assert_eq!(artifact["failure_evidence_ready"], true);
        assert_eq!(artifact["dashboard_read_only_consumable"], true);
        assert_eq!(artifact["release_gate_consumable"], true);
        assert_eq!(
            artifact["request_response_readback_audit_refs_recorded"],
            true
        );
        assert_eq!(artifact["cancel_outcome"], expected_outcome);
        assert_eq!(artifact["outcome_category"], expected_category);
        assert_eq!(artifact["recovered"], expected_recovered);
        assert_eq!(artifact["failed"], expected_failed);
        assert_eq!(artifact["partial_success"], expected_partial_success);
        assert_eq!(
            artifact["residual_risk_visible"],
            expected_residual_risk_visible
        );
        assert_eq!(
            artifact["operator_action_required"],
            expected_operator_action_required
        );
        assert_eq!(artifact["unknown_not_recovered"], true);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["lineage_issues"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        if expected_outcome == "partial_fill" {
            assert_eq!(artifact["partial_fill_residual_risk_visible"], true);
            assert_eq!(
                artifact["residual_risk_state"],
                "partial_fill_residual_risk_manual_review"
            );
        }
        if expected_outcome == "unknown" {
            assert_eq!(artifact["recovered"], false);
            assert_eq!(artifact["actual_cancel_followup_complete"], false);
        }
        assert_v190_actual_cancel_failure_evidence_false_boundary(&artifact);
    }
}

#[test]
fn production_mutation_actual_cancel_failure_evidence_blocks_missing_gates_and_refs() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v190-007-actual-cancel-failure-evidence-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let actual_cancel_attempt =
        write_ready_v190_actual_cancel_single_shot_attempt_artifact(&output_dir);
    let raw_readback = output_dir.join("failure-readback-valid.json");
    let reconciliation = output_dir.join("failure-reconciliation-valid.json");
    write_synthetic_v190_actual_cancel_readback_reconciliation(
        &raw_readback,
        "CANCELED",
        "0",
        "1",
        None,
        false,
    );
    run_live_production_mutation_actual_cancel_readback_reconciliation(
        &production_mutation_actual_cancel_readback_reconciliation_opt(
            actual_cancel_attempt,
            raw_readback,
            reconciliation.clone(),
            true,
        ),
    )
    .unwrap();
    let request_ref = output_dir.join("request-ref-valid.json");
    let response_ref = output_dir.join("response-ref-valid.json");
    let readback_ref = output_dir.join("readback-ref-valid.json");
    let audit_ref = output_dir.join("audit-ref-valid.json");
    for (path, artifact_type, status) in [
        (
            &request_ref,
            "actual_cancel_request_ref",
            "request_ref_recorded",
        ),
        (
            &response_ref,
            "actual_cancel_response_ref",
            "response_ref_recorded",
        ),
        (
            &readback_ref,
            "actual_cancel_readback_ref",
            "readback_ref_recorded",
        ),
        (&audit_ref, "actual_cancel_audit_ref", "audit_ref_recorded"),
    ] {
        write_synthetic_v190_actual_cancel_failure_evidence_ref(
            path,
            artifact_type,
            status,
            &serde_json::json!({}),
        );
    }

    let missing_flags_output = output_dir.join("failure-evidence-missing-flags.json");
    run_live_production_mutation_actual_cancel_failure_evidence(
        &production_mutation_actual_cancel_failure_evidence_opt(
            reconciliation.clone(),
            request_ref.clone(),
            response_ref.clone(),
            readback_ref.clone(),
            audit_ref.clone(),
            missing_flags_output.clone(),
            false,
        ),
    )
    .unwrap();
    let missing_flags: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(missing_flags_output).unwrap()).unwrap();
    assert_eq!(missing_flags["status"], "blocked_missing_gate");
    assert_eq!(missing_flags["evidence_ready"], false);
    assert!(
        missing_flags["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| { flag == "--allow-production-mutation-actual-cancel-failure-evidence" })
    );
    assert_v190_actual_cancel_failure_evidence_false_boundary(&missing_flags);

    let missing_path = output_dir.join("does-not-exist-request-ref.json");
    let missing_ref_result = run_live_production_mutation_actual_cancel_failure_evidence(
        &production_mutation_actual_cancel_failure_evidence_opt(
            reconciliation.clone(),
            missing_path,
            response_ref.clone(),
            readback_ref.clone(),
            audit_ref.clone(),
            output_dir.join("failure-evidence-missing-request-path.json"),
            true,
        ),
    );
    assert!(missing_ref_result.is_err());

    for (label, invalid_request, invalid_response, invalid_readback, invalid_audit) in [
        ("request", true, false, false, false),
        ("response", false, true, false, false),
        ("readback", false, false, true, false),
        ("audit", false, false, false, true),
    ] {
        let invalid = output_dir.join(format!("{label}-ref-invalid.json"));
        fs::write(&invalid, "{}\n").unwrap();
        let output = output_dir.join(format!("failure-evidence-invalid-{label}.json"));
        run_live_production_mutation_actual_cancel_failure_evidence(
            &production_mutation_actual_cancel_failure_evidence_opt(
                reconciliation.clone(),
                if invalid_request {
                    invalid.clone()
                } else {
                    request_ref.clone()
                },
                if invalid_response {
                    invalid.clone()
                } else {
                    response_ref.clone()
                },
                if invalid_readback {
                    invalid.clone()
                } else {
                    readback_ref.clone()
                },
                if invalid_audit {
                    invalid.clone()
                } else {
                    audit_ref.clone()
                },
                output.clone(),
                true,
            ),
        )
        .unwrap();
        let artifact: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(artifact["status"], "blocked_source_artifact");
        assert_eq!(artifact["evidence_ready"], false);
        let missing_artifact_type = format!("{label}_ref_missing_artifact_type");
        let missing_status = format!("{label}_ref_missing_status");
        assert!(
            artifact["source_artifact_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue.as_str() == Some(missing_artifact_type.as_str()))
        );
        assert!(
            artifact["source_artifact_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue.as_str() == Some(missing_status.as_str()))
        );
        assert_v190_actual_cancel_failure_evidence_false_boundary(&artifact);
    }
}

#[test]
fn production_mutation_cancel_response_redaction_persists_allowed_cancel_metadata_only() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-006-cancel-response-redaction-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let approval = write_ready_v180_manual_owner_approval_lifecycle_artifact(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let response = output_dir.join("synthetic-cancel-response.json");
    let output = output_dir.join("cancel-response-redaction.json");
    write_synthetic_production_mutation_cancel_response(&response);

    run_live_production_mutation_cancel_response_redaction(
        &production_mutation_cancel_response_redaction_opt(
            approval.clone(),
            response,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let body = fs::read_to_string(output).unwrap();
    assert!(!body.contains("123456789"));
    assert!(!body.contains("owner-approved-v160-single-shot"));
    assert!(!body.contains("X-MBX-APIKEY"));
    assert!(!body.contains("signature=must_not_persist"));
    assert!(!body.contains("\"headers\""));
    assert!(!body.contains("\"payload\""));
    assert!(!body.contains("\"body\""));
    let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION
    );
    assert_eq!(artifact["artifact_type"], "cancel_response_redaction");
    assert_eq!(artifact["status"], "ready_cancel_response_redacted");
    assert_eq!(artifact["response_redaction_ready"], true);
    assert_eq!(artifact["cancel_response_redacted"], true);
    assert_eq!(artifact["response_shape_validated"], true);
    assert_eq!(
        artifact["response_type"],
        "binance_cancel_response_redacted_metadata_v1"
    );
    assert_eq!(
        artifact["source_manual_owner_approval_lifecycle_hash"],
        file_fnv1a64_hash(&approval.display().to_string())
    );
    assert_eq!(
        artifact["source_manual_owner_approval_lifecycle_run_id"],
        "v180-production-mutation-manual-owner-approval-lifecycle"
    );
    assert_eq!(artifact["approval_lifecycle_valid"], true);
    assert_eq!(artifact["approval_state"], "approved");
    assert_eq!(artifact["manual_approval_recorded"], true);
    assert_eq!(artifact["approval_consumed"], false);
    assert_eq!(artifact["symbol"], "BTCUSDT");
    assert_eq!(artifact["account_label"], "prod-account-redacted");
    assert!(
        artifact["cancel_order_id"]
            .as_str()
            .unwrap()
            .starts_with("cancel_order_id:sha256:")
    );
    assert!(
        artifact["cancel_client_order_id"]
            .as_str()
            .unwrap()
            .starts_with("cancel_client_order_id:sha256:")
    );
    assert!(
        artifact["orig_client_order_id"]
            .as_str()
            .unwrap()
            .starts_with("orig_client_order_id:sha256:")
    );
    assert_eq!(artifact["exchange_status"], "CANCELED");
    assert_eq!(
        artifact["transact_time_shape"],
        "epoch_millis_present_redacted"
    );
    assert_eq!(
        artifact["forbidden_response_markers"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
    assert_eq!(artifact["api_key_value_recorded"], false);
    assert_eq!(artifact["api_secret_value_recorded"], false);
    assert_eq!(artifact["api_key_header_value_recorded"], false);
    assert_eq!(artifact["signature_recorded"], false);
    assert_eq!(artifact["signed_query_recorded"], false);
    assert_eq!(artifact["signed_url_recorded"], false);
    assert_eq!(artifact["request_body_recorded"], false);
    assert_eq!(artifact["raw_request_body_recorded"], false);
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_body_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["unrestricted_payload_recorded"], false);
    assert_eq!(artifact["account_balances_recorded"], false);
    assert_eq!(artifact["fills_recorded"], false);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["cancel_requests_sent"], 0);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["retry_attempted"], false);
    assert_eq!(artifact["remediation_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
    assert_eq!(
        artifact["manual_owner_approval_lifecycle_ref"]["schema_version"],
        PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION
    );
    assert_eq!(
        artifact["manual_owner_approval_lifecycle_ref"]["ready"],
        true
    );
}

#[test]
fn production_mutation_cancel_response_redaction_blocks_forbidden_response_markers() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-006-cancel-response-redaction-forbidden-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let approval = write_ready_v180_manual_owner_approval_lifecycle_artifact(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let response = output_dir.join("forbidden-cancel-response.json");
    let output = output_dir.join("cancel-response-redaction-forbidden.json");
    write_forbidden_production_mutation_cancel_response(&response);

    run_live_production_mutation_cancel_response_redaction(
        &production_mutation_cancel_response_redaction_opt(
            approval,
            response,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_forbidden_response_marker");
    assert_eq!(artifact["response_redaction_ready"], false);
    assert_eq!(artifact["response_shape_validated"], false);
    assert!(
        artifact["forbidden_response_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.headers"))
    );
    assert!(
        artifact["forbidden_response_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.body"))
    );
    assert!(
        artifact["forbidden_response_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.signature"))
    );
    assert_eq!(artifact["raw_exchange_response_recorded"], false);
    assert_eq!(artifact["response_headers_recorded"], false);
    assert_eq!(artifact["unrestricted_payload_recorded"], false);
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
}

#[test]
fn production_mutation_cancel_response_redaction_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-006-cancel-response-redaction-missing-gates-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let approval = write_ready_v180_manual_owner_approval_lifecycle_artifact(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let response = output_dir.join("synthetic-cancel-response.json");
    let output = output_dir.join("cancel-response-redaction-missing-gates.json");
    write_synthetic_production_mutation_cancel_response(&response);

    run_live_production_mutation_cancel_response_redaction(
        &production_mutation_cancel_response_redaction_opt(
            approval,
            response,
            output.clone(),
            false,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["response_redaction_ready"], false);
    assert_eq!(artifact["response_shape_validated"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-cancel-response-redaction")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-raw-response-persistence")
    );
    assert_eq!(artifact["actual_cancel_send_allowed"], false);
    assert_eq!(artifact["cancel_attempted"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["network_cancel_endpoint_attempted"], false);
    assert_eq!(artifact["dashboard_cancel_controls_enabled"], false);
}

#[test]
fn production_mutation_post_cancel_readback_classifies_terminal_and_ambiguous_states() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-007-post-cancel-readback-states-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let redaction = write_ready_v180_cancel_response_redaction_artifact(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );

    for (
        status,
        expected_class,
        expected_outcome,
        terminal_state_observed,
        ambiguous_state_observed,
        order_found,
    ) in [
        (
            "CANCELED",
            "terminal_canceled",
            "cancel_confirmed",
            true,
            false,
            true,
        ),
        (
            "FILLED",
            "terminal_filled",
            "filled_before_or_during_cancel",
            true,
            false,
            true,
        ),
        (
            "REJECTED",
            "terminal_rejected",
            "cancel_or_order_rejected",
            true,
            false,
            true,
        ),
        (
            "EXPIRED",
            "terminal_expired",
            "order_expired",
            true,
            false,
            true,
        ),
        (
            "MISSING",
            "ambiguous_missing",
            "order_missing_manual_review",
            false,
            true,
            false,
        ),
        (
            "UNKNOWN",
            "ambiguous_unknown",
            "unknown_state_manual_review",
            false,
            true,
            true,
        ),
    ] {
        let readback = output_dir.join(format!("post-cancel-readback-{status}.json"));
        let output = output_dir.join(format!("post-cancel-readback-{status}-artifact.json"));
        write_synthetic_production_mutation_post_cancel_readback(&readback, status);

        run_live_production_mutation_post_cancel_readback(
            &production_mutation_post_cancel_readback_opt(
                redaction.clone(),
                readback,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("123456789"));
        assert!(!body.contains("owner-approved-v160-single-shot"));
        assert!(!body.contains("\"headers\""));
        assert!(!body.contains("\"payload\""));
        assert!(!body.contains("\"body\""));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_POST_CANCEL_READBACK_SCHEMA_VERSION
        );
        assert_eq!(artifact["artifact_type"], "post_cancel_readback");
        assert_eq!(artifact["status"], "ready_post_cancel_readback_classified");
        assert_eq!(artifact["post_cancel_readback_ready"], true);
        assert_eq!(artifact["post_cancel_readback_classified"], true);
        assert_eq!(artifact["redacted_metadata_only"], true);
        assert_eq!(artifact["readback_state"], status);
        assert_eq!(artifact["readback_state_class"], expected_class);
        assert_eq!(artifact["readback_outcome"], expected_outcome);
        assert_eq!(artifact["terminal_state_observed"], terminal_state_observed);
        assert_eq!(
            artifact["ambiguous_state_observed"],
            ambiguous_state_observed
        );
        assert_eq!(artifact["order_found"], order_found);
        assert_eq!(artifact["order_lineage_preserved"], true);
        assert_eq!(artifact["symbol"], "BTCUSDT");
        assert_eq!(artifact["account_label"], "prod-account-redacted");
        assert!(
            artifact["readback_order_id"]
                .as_str()
                .unwrap()
                .starts_with("readback_order_id:sha256:")
        );
        assert!(
            artifact["readback_client_order_id"]
                .as_str()
                .unwrap()
                .starts_with("readback_client_order_id:sha256:")
        );
        assert!(
            artifact["readback_orig_client_order_id"]
                .as_str()
                .unwrap()
                .starts_with("readback_orig_client_order_id:sha256:")
        );
        assert_eq!(
            artifact["readback_update_time_shape"],
            "epoch_millis_present_redacted"
        );
        assert_eq!(
            artifact["cancel_response_redaction_ref"]["schema_version"],
            PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION
        );
        assert_eq!(artifact["cancel_response_redaction_ref"]["ready"], true);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_eq!(
            artifact["forbidden_readback_markers"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            artifact["unsupported_readback_states"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_v180_post_cancel_readback_false_boundary(&artifact);
    }
}

#[test]
fn production_mutation_post_cancel_readback_blocks_forbidden_readback_markers() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-007-post-cancel-readback-forbidden-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let redaction = write_ready_v180_cancel_response_redaction_artifact(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let readback = output_dir.join("forbidden-post-cancel-readback.json");
    let output = output_dir.join("post-cancel-readback-forbidden.json");
    write_forbidden_production_mutation_post_cancel_readback(&readback);

    run_live_production_mutation_post_cancel_readback(
        &production_mutation_post_cancel_readback_opt(redaction, readback, output.clone(), true),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_forbidden_readback_marker");
    assert_eq!(artifact["post_cancel_readback_ready"], false);
    assert_eq!(artifact["post_cancel_readback_classified"], false);
    assert!(
        artifact["forbidden_readback_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.headers"))
    );
    assert!(
        artifact["forbidden_readback_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.body"))
    );
    assert!(
        artifact["forbidden_readback_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|marker| marker.as_str().unwrap().contains("$.fills"))
    );
    assert_v180_post_cancel_readback_false_boundary(&artifact);
}

#[test]
fn production_mutation_post_cancel_readback_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-007-post-cancel-readback-missing-gates-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let redaction = write_ready_v180_cancel_response_redaction_artifact(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let readback = output_dir.join("post-cancel-readback-canceled.json");
    let output = output_dir.join("post-cancel-readback-missing-gates.json");
    write_synthetic_production_mutation_post_cancel_readback(&readback, "CANCELED");

    run_live_production_mutation_post_cancel_readback(
        &production_mutation_post_cancel_readback_opt(redaction, readback, output.clone(), false),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["post_cancel_readback_ready"], false);
    assert_eq!(artifact["post_cancel_readback_classified"], false);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--allow-production-mutation-post-cancel-readback")
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-no-mutation")
    );
    assert_v180_post_cancel_readback_false_boundary(&artifact);
}

#[test]
fn production_mutation_post_cancel_readback_blocks_invalid_source_redaction() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-007-post-cancel-readback-source-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let redaction = write_ready_v180_cancel_response_redaction_artifact(
        &output_dir,
        &V170ExchangeReadbackMapperFixture {
            source_status: "ready_exchange_readback_mapped",
            exchange_readback_mapped: true,
            request_sent: true,
            exchange_order_status: "NEW",
            exchange_order_state: "open",
            order_found: true,
            open_order_observed: true,
            terminal_state_observed: false,
        },
    );
    let mut source: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&redaction).unwrap()).unwrap();
    source["status"] = serde_json::Value::String("blocked_source_artifact".to_string());
    source["response_redaction_ready"] = serde_json::Value::Bool(false);
    let invalid_redaction = output_dir.join("invalid-cancel-response-redaction.json");
    fs::write(
        &invalid_redaction,
        serde_json::to_string_pretty(&source).unwrap(),
    )
    .unwrap();
    let readback = output_dir.join("post-cancel-readback-canceled.json");
    let output = output_dir.join("post-cancel-readback-source-blocked.json");
    write_synthetic_production_mutation_post_cancel_readback(&readback, "CANCELED");

    run_live_production_mutation_post_cancel_readback(
        &production_mutation_post_cancel_readback_opt(
            invalid_redaction,
            readback,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_source_artifact");
    assert_eq!(artifact["post_cancel_readback_ready"], false);
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "cancel_response_redaction_status_blocked_source_artifact")
    );
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "cancel_response_redaction_response_redaction_ready_not_true")
    );
    assert_v180_post_cancel_readback_false_boundary(&artifact);
}

#[test]
fn production_mutation_cancel_recovery_incident_audit_closeout_records_traceability() {
    for (
        readback_state,
        expected_recommendation,
        expected_remaining_risk,
        expected_manual_review,
    ) in [
        (
            "CANCELED",
            "close_incident_cancel_confirmed",
            "none_cancel_confirmed",
            false,
        ),
        (
            "MISSING",
            "manual_exchange_and_local_ledger_review",
            "exchange_state_missing_manual_review_required",
            true,
        ),
    ] {
        let output_dir = std::env::temp_dir().join(format!(
            "ntpro-v180-008-incident-audit-closeout-{readback_state}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let (risk_gate, approval, redaction, post_cancel_readback) =
            write_ready_v180_cancel_recovery_closeout_sources(
                &output_dir,
                &V170ExchangeReadbackMapperFixture {
                    source_status: "ready_exchange_readback_mapped",
                    exchange_readback_mapped: true,
                    request_sent: true,
                    exchange_order_status: "NEW",
                    exchange_order_state: "open",
                    order_found: true,
                    open_order_observed: true,
                    terminal_state_observed: false,
                },
                readback_state,
            );
        let output = output_dir.join("incident-audit-closeout.json");

        run_live_production_mutation_cancel_recovery_incident_audit_closeout(
            &production_mutation_cancel_recovery_incident_audit_closeout_opt(
                risk_gate,
                approval,
                redaction,
                post_cancel_readback,
                output.clone(),
                true,
            ),
        )
        .unwrap();

        let body = fs::read_to_string(output).unwrap();
        assert!(!body.contains("123456789"));
        assert!(!body.contains("owner-approved-v160-single-shot"));
        assert!(!body.contains("X-MBX-APIKEY"));
        assert!(!body.contains("apiSecret"));
        assert!(!body.contains("raw readback must not persist"));
        let artifact: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            PRODUCTION_MUTATION_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_SCHEMA_VERSION
        );
        assert_eq!(
            artifact["artifact_type"],
            "cancel_recovery_incident_audit_closeout"
        );
        assert_eq!(
            artifact["status"],
            "ready_cancel_recovery_incident_audit_closeout"
        );
        assert_eq!(artifact["incident_closeout_ready"], true);
        assert_eq!(artifact["audit_trail_ready"], true);
        assert_eq!(artifact["audit_traceability_ready"], true);
        assert_eq!(artifact["cancel_recovery_lineage_ready"], true);
        assert_eq!(artifact["recovery_needed_reason"], "orphan_risk_detected");
        assert_eq!(
            artifact["risk_gate_result"],
            "ready_owner_approval_required"
        );
        assert_eq!(artifact["risk_gate_ready"], true);
        assert_eq!(artifact["orphan_risk_detected"], true);
        assert_eq!(artifact["risk_halted"], true);
        assert_eq!(artifact["new_orders_blocked"], true);
        assert_eq!(artifact["manual_review_required"], true);
        assert_eq!(artifact["owner_approval_state"], "approved");
        assert_eq!(artifact["manual_approval_recorded"], true);
        assert_eq!(artifact["approval_lifecycle_valid"], true);
        assert_eq!(artifact["approval_consumed"], false);
        assert_eq!(
            artifact["redaction_contract_state"],
            "ready_redacted_metadata_only"
        );
        assert_eq!(artifact["cancel_response_redaction_ready"], true);
        assert_eq!(artifact["cancel_response_redacted"], true);
        assert_eq!(artifact["post_cancel_readback_ready"], true);
        assert_eq!(artifact["readback_state"], readback_state);
        assert_eq!(
            artifact["terminal_action_recommendation"],
            expected_recommendation
        );
        assert_eq!(artifact["remaining_risk"], expected_remaining_risk);
        assert_eq!(
            artifact["remaining_risk_requires_manual_review"],
            expected_manual_review
        );
        assert_eq!(artifact["order_lineage_preserved"], true);
        assert_eq!(artifact["candidate_count"], 1);
        assert_eq!(
            artifact["cancel_risk_gate_ref"]["schema_version"],
            PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION
        );
        assert_eq!(artifact["cancel_risk_gate_ref"]["ready"], true);
        assert_eq!(
            artifact["manual_owner_approval_lifecycle_ref"]["schema_version"],
            PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION
        );
        assert_eq!(
            artifact["manual_owner_approval_lifecycle_ref"]["ready"],
            true
        );
        assert_eq!(
            artifact["cancel_response_redaction_ref"]["schema_version"],
            PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION
        );
        assert_eq!(artifact["cancel_response_redaction_ref"]["ready"], true);
        assert_eq!(
            artifact["post_cancel_readback_ref"]["schema_version"],
            PRODUCTION_MUTATION_POST_CANCEL_READBACK_SCHEMA_VERSION
        );
        assert_eq!(artifact["post_cancel_readback_ref"]["ready"], true);
        assert_eq!(
            artifact["source_artifact_issues"].as_array().unwrap().len(),
            0
        );
        assert_eq!(artifact["lineage_issues"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 0);
        assert_v180_cancel_recovery_incident_audit_closeout_false_boundary(&artifact);
    }
}

#[test]
fn production_mutation_cancel_recovery_incident_audit_closeout_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-008-incident-audit-closeout-missing-gates-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (risk_gate, approval, redaction, post_cancel_readback) =
        write_ready_v180_cancel_recovery_closeout_sources(
            &output_dir,
            &V170ExchangeReadbackMapperFixture {
                source_status: "ready_exchange_readback_mapped",
                exchange_readback_mapped: true,
                request_sent: true,
                exchange_order_status: "NEW",
                exchange_order_state: "open",
                order_found: true,
                open_order_observed: true,
                terminal_state_observed: false,
            },
            "CANCELED",
        );
    let output = output_dir.join("incident-audit-closeout-missing-gates.json");

    run_live_production_mutation_cancel_recovery_incident_audit_closeout(
        &production_mutation_cancel_recovery_incident_audit_closeout_opt(
            risk_gate,
            approval,
            redaction,
            post_cancel_readback,
            output.clone(),
            false,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["incident_closeout_ready"], false);
    assert_eq!(artifact["audit_trail_ready"], false);
    assert_eq!(
        artifact["source_artifact_issues"].as_array().unwrap().len(),
        0
    );
    assert_eq!(artifact["lineage_issues"].as_array().unwrap().len(), 0);
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| {
                flag == "--allow-production-mutation-cancel-recovery-incident-audit-closeout"
            })
    );
    assert!(
        artifact["missing_cli_flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "--confirm-remaining-risk-recorded")
    );
    assert_v180_cancel_recovery_incident_audit_closeout_false_boundary(&artifact);
}

#[test]
fn production_mutation_cancel_recovery_incident_audit_closeout_blocks_invalid_source() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-008-incident-audit-closeout-invalid-source-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (risk_gate, approval, redaction, post_cancel_readback) =
        write_ready_v180_cancel_recovery_closeout_sources(
            &output_dir,
            &V170ExchangeReadbackMapperFixture {
                source_status: "ready_exchange_readback_mapped",
                exchange_readback_mapped: true,
                request_sent: true,
                exchange_order_status: "NEW",
                exchange_order_state: "open",
                order_found: true,
                open_order_observed: true,
                terminal_state_observed: false,
            },
            "CANCELED",
        );
    let mut source: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&post_cancel_readback).unwrap()).unwrap();
    source["status"] = json!("blocked_source_artifact");
    source["post_cancel_readback_ready"] = json!(false);
    let invalid_readback = output_dir.join("invalid-post-cancel-readback.json");
    fs::write(
        &invalid_readback,
        serde_json::to_string_pretty(&source).unwrap(),
    )
    .unwrap();
    let output = output_dir.join("incident-audit-closeout-source-blocked.json");

    run_live_production_mutation_cancel_recovery_incident_audit_closeout(
        &production_mutation_cancel_recovery_incident_audit_closeout_opt(
            risk_gate,
            approval,
            redaction,
            invalid_readback,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_source_artifact");
    assert_eq!(artifact["incident_closeout_ready"], false);
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| {
                issue
                    == "source_post_cancel_readback_post_cancel_readback_status_blocked_source_artifact"
            })
    );
    assert!(
        artifact["source_artifact_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| {
                issue
                    == "source_post_cancel_readback_post_cancel_readback_post_cancel_readback_ready_not_true"
            })
    );
    assert_v180_cancel_recovery_incident_audit_closeout_false_boundary(&artifact);
}

#[test]
fn production_mutation_cancel_recovery_incident_audit_closeout_blocks_lineage_mismatch() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v180-008-incident-audit-closeout-lineage-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (risk_gate, approval, redaction, post_cancel_readback) =
        write_ready_v180_cancel_recovery_closeout_sources(
            &output_dir,
            &V170ExchangeReadbackMapperFixture {
                source_status: "ready_exchange_readback_mapped",
                exchange_readback_mapped: true,
                request_sent: true,
                exchange_order_status: "NEW",
                exchange_order_state: "open",
                order_found: true,
                open_order_observed: true,
                terminal_state_observed: false,
            },
            "CANCELED",
        );
    let mut source: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&post_cancel_readback).unwrap()).unwrap();
    source["order_lineage_id"] = json!("lineage-other-cancel-candidate");
    let mismatched_readback = output_dir.join("mismatched-post-cancel-readback.json");
    fs::write(
        &mismatched_readback,
        serde_json::to_string_pretty(&source).unwrap(),
    )
    .unwrap();
    let output = output_dir.join("incident-audit-closeout-lineage-blocked.json");

    run_live_production_mutation_cancel_recovery_incident_audit_closeout(
        &production_mutation_cancel_recovery_incident_audit_closeout_opt(
            risk_gate,
            approval,
            redaction,
            mismatched_readback,
            output.clone(),
            true,
        ),
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_lineage_mismatch");
    assert_eq!(artifact["incident_closeout_ready"], false);
    assert!(
        artifact["lineage_issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue == "post_cancel_readback_order_lineage_id_mismatch")
    );
    assert_v180_cancel_recovery_incident_audit_closeout_false_boundary(&artifact);
}

#[test]
fn production_mutation_runtime_gate_blocks_missing_owner_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-002-runtime-gate-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (order_gate, risk_preflight, request_preview) =
        write_ready_live_alpha_artifact_chain(&output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
    let output = output_dir.join("production_mutation_runtime_gate.json");
    write_kill_switch_approval_artifact(kill_switch_approval.clone(), false, "approved");
    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight.clone(),
            request_preview.clone(),
            kill_switch_runtime_gate.clone(),
            true,
        ),
    )
    .unwrap();

    run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
        order_gate,
        risk_preflight,
        request_preview,
        kill_switch_runtime_gate,
        None,
        output.clone(),
        false,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_missing_gate");
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["send_consideration_allowed"], false);
    assert_eq!(artifact["missing_cli_flags"].as_array().unwrap().len(), 9);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_mutation_runtime_gate_blocks_active_kill_switch() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v160-002-runtime-gate-active-kill-switch-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let (order_gate, risk_preflight, request_preview) =
        write_ready_live_alpha_artifact_chain(&output_dir);
    let kill_switch_approval = output_dir.join("kill_switch_approval.json");
    let kill_switch_runtime_gate = output_dir.join("kill_switch_runtime_gate.json");
    let output = output_dir.join("production_mutation_runtime_gate.json");
    write_kill_switch_approval_artifact(kill_switch_approval.clone(), true, "approved");
    run_live_production_live_alpha_kill_switch_runtime_gate(
        &production_live_alpha_kill_switch_runtime_gate_opt(
            kill_switch_approval,
            risk_preflight.clone(),
            request_preview.clone(),
            kill_switch_runtime_gate.clone(),
            true,
        ),
    )
    .unwrap();

    run_live_production_mutation_runtime_gate(&production_mutation_runtime_gate_opt(
        order_gate,
        risk_preflight,
        request_preview,
        kill_switch_runtime_gate,
        None,
        output.clone(),
        true,
    ))
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(artifact["status"], "blocked_kill_switch_active");
    assert_eq!(artifact["kill_switch_checked_before_send"], true);
    assert_eq!(artifact["kill_switch_active"], true);
    assert_eq!(artifact["runtime_gate_open"], false);
    assert_eq!(artifact["request_sent"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert!(
        artifact["runtime_gate_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "kill_switch_active")
    );
}

#[test]
fn production_live_alpha_risk_preflight_approves_hypothetical_order_without_submission() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-004-risk-approved-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("order_gate.json");
    let input = output_dir.join("risk_input.json");
    let output = output_dir.join("risk_preflight.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    write_live_alpha_risk_input(&input, &passing_live_alpha_risk_input());

    run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
        order_gate,
        input,
        output.clone(),
        true,
    ))
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        report["schema_version"],
        PRODUCTION_LIVE_ALPHA_RISK_PREFLIGHT_REPORT_SCHEMA_VERSION
    );
    assert_eq!(report["status"], "approved");
    assert_eq!(report["risk_decision"], "dry_run_approved");
    assert_eq!(
        report["execution_decision"],
        "blocked_no_production_mutation"
    );
    assert_eq!(report["reasons"].as_array().unwrap().len(), 0);
    assert_eq!(report["order_gate_ready"], true);
    assert_eq!(report["projected_position_notional"], "60");
    assert_eq!(report["production_order_submission_allowed"], false);
    assert_eq!(report["production_order_mutation_allowed"], false);
    assert_eq!(report["production_order_submissions_attempted"], 0);
    assert_eq!(report["production_orders_submitted"], 0);
    assert_eq!(report["production_order_mutations_attempted"], 0);
    assert_eq!(report["execution_adapter_called"], false);
    assert_eq!(report["order_endpoint_access_attempted"], false);
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["dashboard_order_controls_enabled"], false);
    assert_eq!(report["real_orders_submitted"], false);
}

#[test]
fn production_live_alpha_risk_preflight_blocks_missing_confirmations() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-004-risk-missing-flags-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("order_gate.json");
    let input = output_dir.join("risk_input.json");
    let output = output_dir.join("risk_preflight.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    write_live_alpha_risk_input(&input, &passing_live_alpha_risk_input());

    run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
        order_gate,
        input,
        output.clone(),
        false,
    ))
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(report["status"], "blocked_missing_gate");
    assert_eq!(report["risk_decision"], "dry_run_rejected");
    assert_eq!(
        report["execution_decision"],
        "blocked_no_production_mutation"
    );
    assert_eq!(report["missing_cli_flags"].as_array().unwrap().len(), 5);
    assert!(
        report["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "missing_owner_dry_run_confirmation")
    );
    assert_eq!(report["execution_adapter_called"], false);
    assert_eq!(report["production_orders_submitted"], 0);
}

#[test]
fn production_live_alpha_risk_preflight_rejects_risk_and_state_failures() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-004-risk-rejected-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("order_gate.json");
    let input = output_dir.join("risk_input.json");
    let output = output_dir.join("risk_preflight.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    let mut preflight = passing_live_alpha_risk_input();
    preflight.market.now_unix_ms = 5_000;
    preflight.market.max_age_ms = 100;
    preflight.account.readable = false;
    preflight.order_state.readable = false;
    preflight.order_state.open_order_count = 5;
    preflight.risk.kill_switch_active = true;
    preflight.order.notional = "30.00".to_string();
    preflight.limits.max_order_notional = "25.00".to_string();
    preflight.limits.current_position_notional = "90.00".to_string();
    preflight.limits.max_position_notional = "100.00".to_string();
    write_live_alpha_risk_input(&input, &preflight);

    run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
        order_gate,
        input,
        output.clone(),
        true,
    ))
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(report["status"], "rejected");
    assert_eq!(report["risk_decision"], "dry_run_rejected");
    assert_eq!(
        report["execution_decision"],
        "blocked_no_production_mutation"
    );
    let reasons = report["reasons"].as_array().unwrap();
    for expected in [
        "market_stale",
        "account_read_failed",
        "order_state_read_failed",
        "kill_switch_active",
        "notional_limit_exceeded",
        "position_limit_exceeded",
        "open_order_limit_exceeded",
    ] {
        assert!(
            reasons.iter().any(|reason| reason == expected),
            "missing reason {expected}: {reasons:?}"
        );
    }
    assert_eq!(report["execution_adapter_called"], false);
    assert_eq!(report["order_endpoint_access_attempted"], false);
    assert_eq!(report["production_orders_submitted"], 0);
    assert_eq!(report["network_attempted"], false);
}

#[test]
fn production_live_alpha_risk_preflight_rejects_mutating_order_gate() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v140-004-risk-mutating-gate-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let order_gate = output_dir.join("order_gate.json");
    let input = output_dir.join("risk_input.json");
    let output = output_dir.join("risk_preflight.json");
    run_live_production_live_alpha_dry_run_order_gate(
        &production_live_alpha_dry_run_order_gate_opt(order_gate.clone(), true),
    )
    .unwrap();
    let mut gate_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&order_gate).unwrap()).unwrap();
    gate_json["production_orders_submitted"] = serde_json::json!(1);
    fs::write(
        &order_gate,
        serde_json::to_string_pretty(&gate_json).unwrap(),
    )
    .unwrap();
    write_live_alpha_risk_input(&input, &passing_live_alpha_risk_input());

    run_live_production_live_alpha_risk_preflight(&production_live_alpha_risk_preflight_opt(
        order_gate,
        input,
        output.clone(),
        true,
    ))
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(report["status"], "rejected");
    assert!(
        report["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "order_gate_production_orders_submitted_nonzero")
    );
    assert_eq!(report["execution_adapter_called"], false);
    assert_eq!(report["production_order_mutations_attempted"], 0);
}

#[test]
fn production_kill_switch_approval_artifact_writes_no_mutation_contract() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v130-004-kill-switch-approval-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let output = output_dir.join("kill_switch_approval.json");

    run_live_production_kill_switch_approval_artifact(
        &LiveProductionKillSwitchApprovalArtifactOpt {
            run_id: "v130-live-alpha-preflight".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            output: output.clone(),
            kill_switch_active: true,
            approval_state: "approved".to_string(),
            manual_approval_id: Some("owner-approval-001".to_string()),
            approved_by: Some("owner".to_string()),
            confirm_dry_run_only: true,
            confirm_no_production_mutation: true,
            confirm_dashboard_order_controls_disabled: true,
        },
    )
    .unwrap();

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(
        artifact["schema_version"],
        PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(artifact["status"], "manual_approval_recorded");
    assert_eq!(artifact["kill_switch_enabled"], true);
    assert_eq!(artifact["kill_switch_active"], true);
    assert_eq!(artifact["kill_switch_dry_run"], true);
    assert_eq!(artifact["manual_approval_required"], true);
    assert_eq!(artifact["manual_approval_recorded"], true);
    assert_eq!(artifact["manual_approval_id"], "owner-approval-001");
    assert_eq!(artifact["approved_by"], "owner");
    assert_eq!(artifact["approval_state"], "approved");
    assert_eq!(artifact["approval_artifact_only"], true);
    assert_eq!(
        artifact["owner_approval_required_before_any_mutation"],
        true
    );
    assert_eq!(artifact["production_order_submission_allowed"], false);
    assert_eq!(artifact["production_order_mutation_allowed"], false);
    assert_eq!(artifact["production_order_state_reads_allowed"], false);
    assert_eq!(artifact["listen_key_lifecycle_allowed"], false);
    assert_eq!(artifact["production_order_submissions_attempted"], 0);
    assert_eq!(artifact["production_orders_submitted"], 0);
    assert_eq!(artifact["production_order_mutations_attempted"], 0);
    assert_eq!(artifact["production_order_state_reads_attempted"], 0);
    assert_eq!(artifact["listen_key_lifecycle_attempted"], 0);
    assert_eq!(artifact["cancel_replace_amend_attempted"], false);
    assert_eq!(artifact["dashboard_order_controls_enabled"], false);
    assert_eq!(artifact["network_attempted"], false);
    assert_eq!(artifact["values_are_exchange_truth"], false);
}

#[test]
fn production_kill_switch_approval_artifact_requires_dry_run_confirmations() {
    let output = std::env::temp_dir().join(format!(
        "ntpro-v130-004-kill-switch-approval-missing-confirm-{}.json",
        std::process::id()
    ));
    let err = build_production_kill_switch_approval_artifact(
        &LiveProductionKillSwitchApprovalArtifactOpt {
            run_id: "v130-live-alpha-preflight".to_string(),
            session_id: None,
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            output,
            kill_switch_active: true,
            approval_state: "pending".to_string(),
            manual_approval_id: None,
            approved_by: None,
            confirm_dry_run_only: false,
            confirm_no_production_mutation: true,
            confirm_dashboard_order_controls_disabled: true,
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("--confirm-dry-run-only"));
}

#[test]
fn production_kill_switch_approval_artifact_requires_approved_fields() {
    let output = std::env::temp_dir().join(format!(
        "ntpro-v130-004-kill-switch-approval-missing-fields-{}.json",
        std::process::id()
    ));
    let err = build_production_kill_switch_approval_artifact(
        &LiveProductionKillSwitchApprovalArtifactOpt {
            run_id: "v130-live-alpha-preflight".to_string(),
            session_id: None,
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            output,
            kill_switch_active: true,
            approval_state: "approved".to_string(),
            manual_approval_id: None,
            approved_by: Some("owner".to_string()),
            confirm_dry_run_only: true,
            confirm_no_production_mutation: true,
            confirm_dashboard_order_controls_disabled: true,
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("--manual-approval-id"));
}

#[test]
fn production_shadow_strategy_session_rejects_mutating_portfolio_runtime() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-005-shadow-strategy-session-mutating-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let session_events = output_dir.join("shadow_strategy_session.jsonl");
    fs::write(
        &portfolio_runtime,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION,
            "status": "ready_redacted_shadow_portfolio",
            "production_orders_submitted": 1,
            "production_order_mutations_attempted": 0,
            "automatic_correction_orders_submitted": 0,
            "actual_submission_count": 0,
            "dashboard_order_controls_enabled": false,
            "full_production_portfolio_parity_claimed": false,
            "real_orders_submitted": false,
            "provenance": {
                "values_are_exchange_truth": false
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let error =
        build_production_shadow_strategy_session_events(&LiveProductionShadowStrategySessionOpt {
            run_id: "v120-shadow".to_string(),
            session_id: Some("session-1".to_string()),
            strategy_id: "ema_cross_btcusdt_v1".to_string(),
            shadow_portfolio_runtime: portfolio_runtime,
            strategy_session_status: None,
            output: session_events,
            heartbeat_count: 1,
            stop_after_heartbeats: false,
            stop_file: None,
        })
        .unwrap_err()
        .to_string();

    assert!(error.contains("production_orders_submitted > 0"));
}

#[test]
fn production_readonly_reconciliation_classifies_ok() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-006-reconciliation-ok-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let strategy_status = output_dir.join("strategy_session_status.json");
    let shadow_strategy_session = output_dir.join("shadow_strategy_session.jsonl");
    let reconciliation = output_dir.join("reconciliation_events.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);
    fs::write(
        &strategy_status,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "ntpro.v09_strategy_session_status.v1",
            "session_id": "session-1",
            "strategy_id": "ema_cross_btcusdt_v1",
            "state": "running",
            "reason": "fixture strategy running"
        }))
        .unwrap(),
    )
    .unwrap();
    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v120-shadow".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot: account_snapshot.clone(),
        shadow_intent: shadow_intent.clone(),
        output: portfolio_runtime.clone(),
        compat_snapshot_output: None,
    })
    .unwrap();
    run_live_production_shadow_strategy_session(&LiveProductionShadowStrategySessionOpt {
        run_id: "v120-shadow".to_string(),
        session_id: Some("session-1".to_string()),
        strategy_id: "ema_cross_btcusdt_v1".to_string(),
        shadow_portfolio_runtime: portfolio_runtime.clone(),
        strategy_session_status: Some(strategy_status),
        output: shadow_strategy_session.clone(),
        heartbeat_count: 1,
        stop_after_heartbeats: false,
        stop_file: None,
    })
    .unwrap();

    run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
        run_id: "v120-shadow".to_string(),
        account_snapshot: Some(account_snapshot),
        shadow_portfolio_runtime: Some(portfolio_runtime),
        shadow_strategy_session: Some(shadow_strategy_session),
        shadow_intent: Some(shadow_intent),
        output: reconciliation.clone(),
    })
    .unwrap();

    let events = read_jsonl_values(&reconciliation);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["classification"], "ok");
    assert_eq!(events[0]["event_type"], "observed_account_state");
    assert_eq!(events[0]["severity"], "info");
    assert_eq!(events[0]["recommended_action"], "record_only");
    assert_eq!(events[0]["risk_halted"], false);
    assert_eq!(events[0]["production_order_submissions_attempted"], 0);
    assert_eq!(events[0]["production_order_mutations_attempted"], 0);
    assert_eq!(events[0]["production_order_state_reads_attempted"], 0);
    assert_eq!(events[0]["listen_key_lifecycle_attempted"], 0);
    assert_eq!(events[0]["dashboard_order_controls_enabled"], false);
}

#[test]
fn production_readonly_reconciliation_classifies_missing_account_snapshot() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-006-reconciliation-missing-account-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let reconciliation = output_dir.join("reconciliation_events.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);
    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v120-shadow".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot,
        shadow_intent,
        output: portfolio_runtime.clone(),
        compat_snapshot_output: None,
    })
    .unwrap();

    run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
        run_id: "v120-shadow".to_string(),
        account_snapshot: None,
        shadow_portfolio_runtime: Some(portfolio_runtime),
        shadow_strategy_session: None,
        shadow_intent: None,
        output: reconciliation.clone(),
    })
    .unwrap();

    let events = read_jsonl_values(&reconciliation);
    assert_eq!(events[0]["classification"], "missing_account_snapshot");
    assert_eq!(events[0]["severity"], "degraded");
    assert_eq!(events[0]["recommended_action"], "mark_degraded");
    assert_eq!(events[0]["risk_halted"], true);
}

#[test]
fn production_readonly_reconciliation_classifies_shadow_intent_without_portfolio() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-006-reconciliation-intent-no-portfolio-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let reconciliation = output_dir.join("reconciliation_events.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);

    run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
        run_id: "v120-shadow".to_string(),
        account_snapshot: Some(account_snapshot),
        shadow_portfolio_runtime: None,
        shadow_strategy_session: None,
        shadow_intent: Some(shadow_intent),
        output: reconciliation.clone(),
    })
    .unwrap();

    let events = read_jsonl_values(&reconciliation);
    assert_eq!(
        events[0]["classification"],
        "shadow_intent_without_portfolio"
    );
    assert_eq!(events[0]["event_type"], "shadow_mismatch");
    assert_eq!(events[0]["severity"], "halt");
    assert_eq!(events[0]["recommended_action"], "manual_review_required");
    assert_eq!(events[0]["risk_halted"], true);
}

#[test]
fn production_readonly_reconciliation_classifies_production_mutation_forbidden() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-006-reconciliation-mutation-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let reconciliation = output_dir.join("reconciliation_events.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    fs::write(
        &portfolio_runtime,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": PRODUCTION_SHADOW_PORTFOLIO_RUNTIME_SCHEMA_VERSION,
            "status": "ready_redacted_shadow_portfolio",
            "production_orders_submitted": 1,
            "production_order_mutations_attempted": 0,
            "automatic_correction_orders_submitted": 0,
            "actual_submission_count": 0,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "provenance": {
                "values_are_exchange_truth": false
            }
        }))
        .unwrap(),
    )
    .unwrap();

    run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
        run_id: "v120-shadow".to_string(),
        account_snapshot: Some(account_snapshot),
        shadow_portfolio_runtime: Some(portfolio_runtime),
        shadow_strategy_session: None,
        shadow_intent: None,
        output: reconciliation.clone(),
    })
    .unwrap();

    let events = read_jsonl_values(&reconciliation);
    assert_eq!(events[0]["classification"], "production_mutation_forbidden");
    assert_eq!(events[0]["event_type"], "risk_halt");
    assert_eq!(events[0]["severity"], "halt");
    assert_eq!(events[0]["recommended_action"], "halt_shadow_flow");
    assert_eq!(events[0]["production_orders_submitted"], 0);
}

#[test]
fn production_readonly_reconciliation_classifies_manual_review_required() {
    let output_dir = std::env::temp_dir().join(format!(
        "ntpro-v120-006-reconciliation-manual-review-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let account_snapshot = output_dir.join("production_account_snapshot_redacted.json");
    let shadow_intent = output_dir.join("shadow_execution_intent.jsonl");
    let portfolio_runtime = output_dir.join("shadow_portfolio_runtime.json");
    let shadow_strategy_session = output_dir.join("shadow_strategy_session.jsonl");
    let reconciliation = output_dir.join("reconciliation_events.jsonl");
    write_redacted_account_snapshot_report(&account_snapshot, true);
    write_shadow_intent(&shadow_intent, false);
    run_live_production_shadow_portfolio_runtime(&LiveProductionShadowPortfolioRuntimeOpt {
        run_id: "v120-shadow".to_string(),
        snapshot_id: Some("portfolio-1".to_string()),
        account_snapshot: account_snapshot.clone(),
        shadow_intent: shadow_intent.clone(),
        output: portfolio_runtime.clone(),
        compat_snapshot_output: None,
    })
    .unwrap();
    run_live_production_shadow_strategy_session(&LiveProductionShadowStrategySessionOpt {
        run_id: "v120-shadow".to_string(),
        session_id: Some("session-1".to_string()),
        strategy_id: "ema_cross_btcusdt_v1".to_string(),
        shadow_portfolio_runtime: portfolio_runtime.clone(),
        strategy_session_status: None,
        output: shadow_strategy_session.clone(),
        heartbeat_count: 1,
        stop_after_heartbeats: false,
        stop_file: None,
    })
    .unwrap();

    run_live_production_readonly_reconciliation(&LiveProductionReadonlyReconciliationOpt {
        run_id: "v120-shadow".to_string(),
        account_snapshot: Some(account_snapshot),
        shadow_portfolio_runtime: Some(portfolio_runtime),
        shadow_strategy_session: Some(shadow_strategy_session),
        shadow_intent: Some(shadow_intent),
        output: reconciliation.clone(),
    })
    .unwrap();

    let events = read_jsonl_values(&reconciliation);
    assert_eq!(events[0]["classification"], "manual_review_required");
    assert_eq!(events[0]["event_type"], "manual_remediation_required");
    assert_eq!(events[0]["severity"], "warning");
    assert_eq!(events[0]["manual_review_required"], true);
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
        serde_json::from_str(&fs::read_to_string(output_dir.join("status.json")).unwrap()).unwrap();
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
        serde_json::from_str(&fs::read_to_string(output_dir.join("status.json")).unwrap()).unwrap();
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
        serde_json::from_str(&fs::read_to_string(output_dir.join("status.json")).unwrap()).unwrap();
    assert_eq!(status.node_id, "btc-ema-shadow-001");
    assert_eq!(status.lifecycle_state, LifecycleStatus::Stopped);
    assert_eq!(status.process_mode, ProcessMode::SpawnedProcess);
    assert_eq!(status.data_connection, ConnectionStatus::Disconnected);
    assert_eq!(status.execution_connection, ConnectionStatus::NotConfigured);
    assert!(!status.external_venue_connection);
    assert!(!status.real_orders_submitted);
    assert_eq!(status.risk.command_count.value, Some(2));
    assert_eq!(status.risk.event_count.value, Some(2));
    assert_eq!(status.risk.rejections_total.value, Some(2));

    let session_status: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(output_dir.join("strategy").join("session_status.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(session_status["state"], "stopped");
    assert_eq!(session_status["session_id"], "btc-ema-shadow-001");
    assert_eq!(session_status["strategy_id"], "ema_cross_btcusdt_v1");
    let market_status: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(output_dir.join("strategy").join("market_status.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(market_status["state"], "stopped");
    assert_eq!(market_status["connection"], "stopped");

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
    assert_eq!(metrics.strategy_signal_count.value, Some(2));
    assert_eq!(metrics.strategy_rejection_count.value, Some(2));
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
    let session_status_path = output_dir.join("strategy").join("session_status.json");
    let node_status_path = output_dir.join("status.json");
    let node_metrics_path = output_dir.join("metrics.json");
    let stop_file_writer = stop_file.clone();
    let watcher = tokio::spawn(async move {
        for _ in 0..40 {
            if session_status_path.exists() {
                let status: serde_json::Value =
                    serde_json::from_str(&fs::read_to_string(&session_status_path)?)?;
                if status["state"] == "running"
                    && node_status_path.exists()
                    && node_metrics_path.exists()
                {
                    let node_status: NodeStatus =
                        serde_json::from_str(&fs::read_to_string(&node_status_path)?)?;
                    let node_metrics: NodeMetrics =
                        serde_json::from_str(&fs::read_to_string(&node_metrics_path)?)?;
                    if node_status.lifecycle_state == LifecycleStatus::Running
                        && node_status.risk.command_count.value == Some(2)
                        && node_status.risk.event_count.value == Some(2)
                        && node_status.risk.rejections_total.value == Some(2)
                        && node_metrics.strategy_signal_count.value == Some(2)
                        && node_metrics.strategy_rejection_count.value == Some(2)
                    {
                        fs::write(&stop_file_writer, "stop\n")?;
                        return Ok::<_, anyhow::Error>(());
                    }
                }
            }
            sleep(Duration::from_millis(50)).await;
        }
        anyhow::bail!("strategy session heartbeat counters did not remain non-zero before shutdown")
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
