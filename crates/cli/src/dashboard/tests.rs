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

//! Dashboard read-model、渲染、路由与 fail-closed 行为回归。

use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::supervisor::{
    NodeMetricArtifacts, NodeMetricCounts, NodeMetrics, RegisterNodeRequest, RegistryArtifactState,
    SupervisorNodeRecord, SupervisorPidArtifact, SupervisorProcessIdentity, SupervisorProcessState,
    SupervisorRegistry, SupervisorRegistryStore, write_node_metrics_artifact,
};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::*;

const HTTP_CONTROL_TEST_DEADLINE: Duration = Duration::from_secs(10);

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn dashboard_module_ownership_boundaries_are_explicit() {
    let root = include_str!("../dashboard.rs");
    let control_center = include_str!("control_center.rs");
    let institution_workbench = include_str!("institution_workbench.rs");
    let rendering = include_str!("rendering.rs");
    let server = include_str!("server.rs");

    assert!(root.contains("mod control_center;"));
    assert!(root.contains("mod institution_workbench;"));
    assert!(root.contains("mod rendering;"));
    assert!(root.contains("mod server;"));
    assert!(!root.contains("mod strategy_workbench;"));
    assert!(root.contains("mod trader_terminal_api;"));
    assert!(root.contains("#[path = \"dashboard/tests.rs\"]"));
    assert!(!root.contains("mod tests {"));
    assert!(!root.contains("const DASHBOARD_HTML:"));
    assert!(!root.contains("const DASHBOARD_CSS:"));
    assert!(!root.contains("const DASHBOARD_JS:"));

    assert!(rendering.contains("//! Dashboard 静态 HTML、CSS 与 JavaScript 渲染资源。"));
    assert!(rendering.contains("pub(super) const DASHBOARD_HTML:"));
    assert!(rendering.contains("pub(super) const DASHBOARD_CSS:"));
    assert!(rendering.contains("pub(super) const DASHBOARD_JS:"));
    assert!(control_center.contains("//! 控制中心静态 shell 与共享状态、运维投影渲染资源。"));
    assert!(control_center.contains("pub(super) const CONTROL_CENTER_HTML:"));
    assert!(control_center.contains("pub(super) const CONTROL_CENTER_CSS:"));
    assert!(control_center.contains("pub(super) const CONTROL_CENTER_JS:"));
    assert!(institution_workbench.contains("//! 机构工作台静态 shell 与共享只读状态渲染资源。"));
    assert!(institution_workbench.contains("pub(super) const INSTITUTION_WORKBENCH_HTML:"));
    assert!(institution_workbench.contains("pub(super) const INSTITUTION_WORKBENCH_CSS:"));
    assert!(institution_workbench.contains("pub(super) const INSTITUTION_WORKBENCH_JS:"));
    assert!(server.contains("tower_http::services::{ServeDir, ServeFile}"));
    assert!(server.contains("fn strategy_workbench_routes("));
    assert!(server.contains("pub(crate) fn validate_strategy_workbench_dist("));
}

#[test]
fn strategy_workbench_is_read_only_main_product_shell() {
    let server = include_str!("server.rs");
    let app_shell = include_str!("../../../../apps/strategy-workbench/src/app/AppShell.tsx");
    let status_api = include_str!("../../../../apps/strategy-workbench/src/api/mvpStatus.ts");
    for route in [
        r#""/strategy-workbench""#,
        r#""/assets""#,
        "ServeDir::new(dist_path.join(\"assets\"))",
        "fallback_service(ServeFile::new(index_path))",
        "require_strategy_workbench_access",
    ] {
        assert!(
            server.contains(route),
            "dashboard server missing route {route}"
        );
    }
    for label in [
        "策略工作台",
        "Backtest",
        "Demo",
        "Live",
        "持仓",
        "活动",
        "成交",
        "日志",
        "系统状态",
        "未开放",
    ] {
        assert!(app_shell.contains(label));
    }
    for required in [
        "const STATUS_URL = \"/api/mvp/v1/status\"",
        "parseMvpStatus",
        "credentials: \"same-origin\"",
        "read_only_product_contract",
        "order_submission_allowed",
        "real_orders_submitted",
    ] {
        assert!(
            status_api.contains(required),
            "strategy workbench missing contract marker {required}",
        );
    }
    for forbidden in [
        "method: \"POST\"",
        "data-dashboard-action",
        "submit_order",
        "cancel_order",
        "replace_order",
        "amend_order",
        "flatten_position",
        "retry_order_action",
        "automatic_remediation_action",
    ] {
        assert!(!app_shell.contains(forbidden));
        assert!(!status_api.contains(forbidden));
    }
}

#[test]
fn control_center_correlates_shared_and_operational_status_and_fails_closed() {
    let server = include_str!("server.rs");
    for route in [
        r#""/control-center""#,
        "get(control_center_shell).head(reject_non_get)",
        r#""/assets/control-center.css""#,
        r#""/assets/control-center.js""#,
        r#""/api/mvp/v1/control-center""#,
        "get(control_center_operational_api).head(reject_non_get)",
        r#""/api/mvp/v1/event-correlation""#,
        "get(mvp_event_correlation_api).head(reject_non_get)",
    ] {
        assert!(
            server.contains(route),
            "dashboard server missing route {route}"
        );
    }
    for mount in [
        "axis-grid",
        "business-impact-list",
        "event-correlation-panel",
        "node-grid",
        "lifecycle-action-buttons",
        "lifecycle-action-result",
        "component-table",
        "observability-grid",
        "alert-list",
        "source-list",
        "boundary-list",
    ] {
        assert!(
            CONTROL_CENTER_HTML.contains(mount),
            "control center missing mount {mount}",
        );
    }
    assert!(CONTROL_CENTER_HTML.contains("NTPRO 控制中心"));
    for required in [
        "const SHARED_STATUS_URL = \"/api/mvp/v1/status\"",
        "const OPS_SNAPSHOT_URL = \"/api/mvp/v1/control-center\"",
        "const EVENT_CORRELATION_URL = \"/api/mvp/v1/event-correlation\"",
        "control_center",
        "validateSharedStatus",
        "validateEventCorrelation",
        "requestedEventId",
        "portalEventLink",
        "validateOperationalProjection",
        "validateLifecycleActionEnvelope",
        "/api/mvp/v1/control-center/nodes/",
        "data-lifecycle-action",
        "共享状态与运维节点身份不一致",
        "共享状态与运维 registry provenance 不一致",
        "控制中心 MVP 要求恰好一个运维节点",
        "运维投影暴露超范围字段",
        "运维节点暴露未脱敏错误",
        "resetSurface(\"刷新中，旧数据已清空\")",
        "method: \"GET\"",
        "method: \"POST\"",
        "cache: \"no-store\"",
    ] {
        assert!(
            CONTROL_CENTER_JS.contains(required),
            "control center missing contract marker {required}",
        );
    }
    assert_eq!(
        CONTROL_CENTER_JS.matches("fetch(").count(),
        4,
        "control center must request exactly three read projections and one lifecycle action endpoint",
    );
    for forbidden in [
        "/api/nodes/",
        "/api/server",
        r#""/api/snapshot""#,
        "/api/event-store",
        "data-dashboard-action",
        "/actions/pause",
        "/actions/resume",
        "/actions/reconnect_data",
        "/actions/reconnect_execution",
        "submit_order",
        "cancel_order",
        "replace_order",
        "amend_order",
        "flatten_position",
        "retry_order_action",
        "automatic_remediation_action",
    ] {
        assert!(
            !CONTROL_CENTER_JS.contains(forbidden),
            "control center must not expose {forbidden}",
        );
        assert!(
            !CONTROL_CENTER_HTML.contains(forbidden),
            "control center shell must not expose {forbidden}",
        );
    }
    assert!(CONTROL_CENTER_CSS.contains("@media (max-width: 680px)"));
}

#[test]
fn control_center_operational_projection_is_minimized_and_redacted() {
    let status = NodeStatus {
        lifecycle_state: LifecycleStatus::Running,
        last_error: Some("credential=must-not-cross-product-boundary".to_string()),
        ..NodeStatus::unknown("sandbox-a")
    };
    let mut snapshot = DashboardSnapshot::from_nodes(
        "2026-08-03T12:00:00Z",
        vec![DashboardNodeSummary::from_status(&status)],
    );
    snapshot.controls.push(ControlStatus {
        action: "stop:sandbox-a".to_string(),
        availability: DashboardAvailability::Available,
        enabled: true,
        reason: DashboardValue::available("internal control reason".to_string()),
    });
    snapshot.gaps.push(DashboardGap::new(
        "runtime_modules.sandbox-a.cache",
        DashboardAvailability::NotSupported,
        "MVP-007",
        "raw internal error detail",
    ));
    snapshot.generated_at = DashboardValue {
        availability: DashboardAvailability::Redacted,
        value: Some("credential=must-be-removed".to_string()),
    };

    let projection = project_control_center_snapshot(Path::new("registry.json"), snapshot)
        .expect("valid one-node snapshot should project");
    let value = serde_json::to_value(projection).unwrap();
    let serialized = serde_json::to_string(&value).unwrap();

    assert_eq!(
        value["schema_version"],
        control_center::CONTROL_CENTER_OPERATIONAL_SCHEMA_VERSION
    );
    assert_eq!(value["registry_path"], "registry.json");
    assert_eq!(value["node"]["error_present"], true);
    assert_eq!(value["boundaries"]["read_only"], true);
    assert_eq!(value["boundaries"]["supervisor_actions_exposed"], true);
    assert_eq!(
        value["boundaries"]["unsupported_supervisor_actions_exposed"],
        false
    );
    assert_eq!(value["boundaries"]["trading_controls_exposed"], false);
    assert_eq!(value["lifecycle_actions"].as_array().map(Vec::len), Some(2));
    assert_eq!(value["lifecycle_actions"][0]["action"], "start");
    assert_eq!(value["lifecycle_actions"][0]["enabled"], false);
    assert_eq!(value["lifecycle_actions"][1]["action"], "stop");
    assert_eq!(value["lifecycle_actions"][1]["enabled"], true);
    assert_eq!(value["boundaries"]["raw_errors_exposed"], false);
    for forbidden in [
        "controls",
        "risk",
        "workflow_artifacts",
        "read_model_runtime",
        "production_mutation_evidence",
        "last_error",
        "message",
        "notes",
        "account_ref",
    ] {
        assert!(
            value.get(forbidden).is_none(),
            "unexpected top-level {forbidden}"
        );
        assert!(!serialized.contains(&format!("\"{forbidden}\"")));
    }
    assert!(!serialized.contains("must-not-cross-product-boundary"));
    assert!(!serialized.contains("raw internal error detail"));
    assert!(!serialized.contains("internal control reason"));
    assert!(!serialized.contains("credential=must-be-removed"));
    assert_eq!(value["generated_at"]["availability"], "redacted");
    assert!(value["generated_at"].get("value").is_none());
}

#[test]
fn control_center_operational_projection_rejects_scope_and_boundary_drift() {
    let node = DashboardNodeSummary::from_status(&NodeStatus::unknown("sandbox-a"));
    let two_nodes =
        DashboardSnapshot::from_nodes("2026-08-03T12:00:00Z", vec![node.clone(), node.clone()]);
    assert_eq!(
        validate_control_center_action_scope(&two_nodes, "sandbox-a").err(),
        Some("single_node_contract_violation")
    );
    assert_eq!(
        project_control_center_snapshot(Path::new("registry.json"), two_nodes).unwrap_err(),
        "single_node_contract_violation"
    );

    let mut boundary_drift = DashboardSnapshot::from_nodes("2026-08-03T12:00:00Z", vec![node]);
    boundary_drift.overview.external_venue_connection = true;
    assert_eq!(
        validate_control_center_action_scope(&boundary_drift, "sandbox-a").err(),
        Some("operational_boundary_violation")
    );
    assert_eq!(
        project_control_center_snapshot(Path::new("registry.json"), boundary_drift).unwrap_err(),
        "operational_boundary_violation"
    );

    let mut count_drift = DashboardSnapshot::from_nodes(
        "2026-08-03T12:00:00Z",
        vec![DashboardNodeSummary::from_status(&NodeStatus {
            lifecycle_state: LifecycleStatus::Running,
            ..NodeStatus::unknown("sandbox-a")
        })],
    );
    count_drift.overview.running_nodes = 0;
    assert_eq!(
        project_control_center_snapshot(Path::new("registry.json"), count_drift).unwrap_err(),
        "overview_node_count_mismatch"
    );

    let valid = DashboardSnapshot::from_nodes(
        "2026-08-03T12:00:00Z",
        vec![DashboardNodeSummary::from_status(&NodeStatus::unknown(
            "sandbox-a",
        ))],
    );
    assert_eq!(
        validate_control_center_action_scope(&valid, "sandbox-b").err(),
        Some("action_target_node_mismatch")
    );
}

#[test]
fn institution_workbench_consumes_only_shared_status_and_fails_closed() {
    let server = include_str!("server.rs");
    for route in [
        r#""/institution-workbench""#,
        "get(institution_workbench_shell).head(reject_non_get)",
        r#""/assets/institution-workbench.css"#,
        r#""/assets/institution-workbench.js"#,
    ] {
        assert!(
            server.contains(route),
            "dashboard server missing route {route}"
        );
    }
    for mount in [
        "axis-grid",
        "identity-grid",
        "business-grid",
        "blocking-panel",
        "event-correlation-panel",
        "source-list",
        "boundary-list",
    ] {
        assert!(
            INSTITUTION_WORKBENCH_HTML.contains(mount),
            "institution workbench missing mount {mount}",
        );
    }
    assert!(
        INSTITUTION_WORKBENCH_HTML.contains("NTPRO 机构工作台"),
        "institution workbench must identify its product role",
    );
    assert!(INSTITUTION_WORKBENCH_JS.contains("const SHARED_STATUS_URL = \"/api/mvp/v1/status\""),);
    assert!(
        INSTITUTION_WORKBENCH_JS
            .contains("const EVENT_CORRELATION_URL = \"/api/mvp/v1/event-correlation\""),
    );
    assert_eq!(
        INSTITUTION_WORKBENCH_JS.matches("fetch(").count(),
        2,
        "institution workbench must request exactly shared status and event correlation",
    );
    for required in [
        "ntpro.mvp_shared_status_api.response.v1",
        "ntpro.mvp_shared_status_api.v1",
        "institution_workbench",
        "validateSharedStatus",
        "validateEventCorrelation",
        "requestedEventId",
        "portalEventLink",
        "requireBoundary",
        "requireAxis",
        "requireDashboardValue",
        "EXPECTED_IDENTITY_SCHEMA",
        "EXPECTED_STATUS_SCHEMA",
        "identity.contract_id !==",
        r#"["readiness_status", "snapshot_id", "schema_version""#,
        "read_only_product_contract",
        "requireAxis(status.trading_readiness",
        "resetSurface(\"刷新中，旧数据已清空\")",
        "共享状态不可用，旧数据已清空",
        "method: \"GET\"",
        "cache: \"no-store\"",
    ] {
        assert!(
            INSTITUTION_WORKBENCH_JS.contains(required),
            "institution workbench missing contract marker {required}",
        );
    }
    for forbidden in [
        "/api/server",
        "/api/snapshot",
        "/api/nodes",
        "/api/event-store",
        "method: \"POST\"",
        "data-dashboard-action",
        "submit_order",
        "cancel_order",
        "replace_order",
        "amend_order",
        "flatten_position",
        "retry_order_action",
        "automatic_remediation_action",
    ] {
        assert!(
            !INSTITUTION_WORKBENCH_JS.contains(forbidden),
            "institution workbench must not expose {forbidden}",
        );
        assert!(
            !INSTITUTION_WORKBENCH_HTML.contains(forbidden),
            "institution workbench shell must not expose {forbidden}",
        );
    }
    assert!(
        INSTITUTION_WORKBENCH_CSS.contains("@media (max-width: 680px)"),
        "institution workbench must define a narrow viewport layout",
    );
}

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
        "sandbox_business",
        "workflow_artifacts",
        "read_model_runtime",
        "strategy_runtime",
        "preflight_readiness",
        "live_alpha_dry_run",
        "production_mutation_evidence",
        "production_actual_cancel_audit",
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
    assert_eq!(value["overview"]["production_venue_connection"], false);
    assert_eq!(
        value["overview"]["testnet_public_network_connection"],
        false
    );
    assert_eq!(value["overview"]["external_network_attempted"], false);
    assert_eq!(value["risk"]["availability"], "unknown");
    assert_eq!(value["workflow_artifacts"], json!([]));
    assert_eq!(value["read_model_runtime"], json!([]));
    assert_eq!(value["strategy_runtime"], json!([]));
    assert_eq!(value["preflight_readiness"], json!([]));
    assert_eq!(value["live_alpha_dry_run"], json!([]));
    assert_eq!(value["production_mutation_evidence"], json!([]));
    assert_eq!(value["production_actual_cancel_audit"], json!([]));
    assert_eq!(value["sandbox_business"]["availability"], "available");
    assert_eq!(
        value["sandbox_business"]["exchange"]["venue"],
        json!({"availability": "available", "value": "BINANCE"})
    );
}

#[test]
fn dashboard_shell_includes_system_panel_mounts_and_redaction_helpers() {
    assert!(
        DASHBOARD_HTML.contains(r#"<link rel="icon" href="data:,">"#),
        "dashboard shell must use an inline empty favicon to avoid a failing browser request"
    );

    for mount_id in [
        "data-sources",
        "execution-gateways",
        "sandbox-business",
        "workflow-artifacts",
        "trader-terminal-workbench",
        "read-model-runtime",
        "strategy-runtime",
        "preflight-readiness",
        "live-alpha-dry-run",
        "production-actual-cancel-audit",
        "risk",
        "runtime-modules",
        "logs-metrics",
        "controls",
        "control-result",
    ] {
        assert!(
            DASHBOARD_HTML.contains(mount_id),
            "dashboard shell missing mount id {mount_id}"
        );
    }

    for js_symbol in [
        "renderDataSources",
        "renderExecutionGateways",
        "renderSandboxBusiness",
        "renderWorkflowArtifacts",
        "renderTraderTerminalWorkbench",
        "traderWorkbenchReadiness",
        "renderReadModelRuntime",
        "renderStrategyRuntime",
        "renderPreflightReadiness",
        "renderLiveAlphaDryRun",
        "renderRisk",
        "renderRuntimeModules",
        "renderLogsMetrics",
        "renderControls",
        "redactedDashboardValue",
        "dashboardErrorValue",
        "没有数据源上报",
        "没有执行网关上报",
        "没有 workflow manifest 工件",
        "workbench-artifact-provenance-drawer",
        "没有 Unified Read Model runtime 工件",
        "没有 Strategy Runtime 工件",
        "没有 v0.13 预检就绪工件",
        "没有 v0.14 Live Alpha dry-run 工件",
        "没有运行模块上报",
        "没有日志或指标上报",
        "没有控制项",
        "生产连接",
        "Testnet 只读",
        "外部网络尝试",
        "兼容外部交易场所",
        "订单证明",
        "预检就绪",
        "Live Alpha dry-run",
        "Unified Read Model",
    ] {
        assert!(
            DASHBOARD_JS.contains(js_symbol),
            "dashboard JS missing {js_symbol}"
        );
    }

    for (prefix, suffix) in [
        ("submit", "order"),
        ("cancel", "order"),
        ("enable", "live"),
        ("production", "connect"),
        ("strategy", "hot_reload"),
    ] {
        let forbidden_control = format!("{prefix}_{suffix}");
        assert!(
            !DASHBOARD_JS.contains(&forbidden_control),
            "dashboard JS must not expose trading control {forbidden_control}"
        );
        assert!(
            !DASHBOARD_HTML.contains(&forbidden_control),
            "dashboard shell must not expose trading control {forbidden_control}"
        );
    }
}

#[test]
fn dashboard_trader_ops_boundary_keeps_order_controls_absent() {
    let allowed_local_controls =
        r#"["start", "stop", "pause", "resume", "reconnect_data", "reconnect_execution"]"#;
    assert!(
        DASHBOARD_JS.contains(allowed_local_controls),
        "dashboard JS must keep the local supervisor control allowlist explicit",
    );
    assert!(
        DASHBOARD_HTML.contains("不会连接外部交易场所，也不会提交真实订单"),
        "dashboard shell must describe the local read-model/no-order boundary",
    );

    for forbidden_control in [
        "submit_order",
        "cancel_order",
        "replace_order",
        "amend_order",
        "retry_order_action",
        "data-dashboard-action=\"retry_order\"",
        "/actions/retry_order",
        "correct_order",
        "flatten_position_action",
        "data-dashboard-action=\"flatten_position\"",
        "/actions/flatten_position",
        "credential_entry",
        "listen_key_control",
        "production_order_control",
    ] {
        assert!(
            !DASHBOARD_JS.contains(forbidden_control),
            "dashboard JS must not expose v0.13 forbidden control {forbidden_control}",
        );
        assert!(
            !DASHBOARD_HTML.contains(forbidden_control),
            "dashboard shell must not expose v0.13 forbidden control {forbidden_control}",
        );
    }
}

#[test]
fn trader_terminal_workbench_shell_is_readonly_and_degrades_without_artifact() {
    for required_marker in [
        "trader-terminal-workbench",
        "workbench-tabs",
        "workbench-panel-account",
        "workbench-panel-positions",
        "workbench-panel-orders",
        "workbench-panel-fills",
        "workbench-panel-risk",
        "workbench-panel-alerts",
        "workbench-panel-audit-provenance",
        "workbench-panel-v25-monitoring-surface",
        "foundation-boundary",
        "read-only-boundary",
        "gated-operation-boundary",
        "workbench-artifact-provenance-drawer",
        "canonical_unified_read_model_artifact_missing",
        "missing_artifact",
        "degraded_shell",
        "all_operation_controls_disabled",
        "required_before_any_manual_entry",
        "Funds transfer",
        "Account config mutation",
        "Auto flatten",
        "Position repair",
        "positions_lineage",
        "Client order id",
        "Request digest",
        "Attempt id",
        "Approval id",
        "Ledger present",
        "Duplicate attempt",
        "Fill id",
        "Execution id",
        "Order linkage",
        "Reconciliation",
        "Risk input",
        "Schema-only truth",
        "Priority state",
        "Risk visible",
        "Manual review",
        "Forbidden control",
        "Release provenance",
        "Artifact digest",
        "Provenance repair",
        "v25 Monitoring / Incident / DR",
        "Surface status",
        "Monitoring status",
        "Incident status",
        "Ack status",
        "Runbook evidence",
        "DR preview status",
        "Snapshot lineage",
    ] {
        assert!(
            DASHBOARD_HTML.contains(required_marker) || DASHBOARD_JS.contains(required_marker),
            "Trader Terminal workbench shell missing marker {required_marker}"
        );
    }

    assert!(
        DASHBOARD_JS.contains("renderTraderTerminalWorkbench(snapshot.read_model_runtime || [])"),
        "dashboard render path must load the workbench from read_model_runtime"
    );
    assert!(
        DASHBOARD_JS.contains("v0_21/unified_read_model_snapshot.json"),
        "workbench fallback must name the canonical v0.21.1 read-model artifact"
    );
    assert!(
        !DASHBOARD_HTML.contains("product-grade live trading terminal"),
        "dashboard shell must not claim product-grade live trading terminal readiness"
    );
    assert!(
        !DASHBOARD_JS.contains("product-grade live trading terminal"),
        "dashboard JS must not claim product-grade live trading terminal readiness"
    );

    for forbidden_control in [
        "data-workbench-action",
        "submit_order",
        "cancel_order",
        "retry_order_action",
        "data-workbench-action=\"retry_order\"",
        "/actions/retry_order",
        "replace_order",
        "amend_order",
        "flatten_order",
        "flatten_position_action",
        "data-dashboard-action=\"flatten_position\"",
        "/actions/flatten_position",
        "funds_transfer_action",
        "account_config_action",
        "auto_flatten_action",
        "position_repair_action",
        "order_submit_action",
        "order_retry_action",
        "order_replace_action",
        "order_amend_action",
        "order_cancel_action",
        "fill_repair_action",
        "reconciliation_repair_action",
        "execution_algorithm_action",
        "data-workbench-action=\"risk_action\"",
        "data-workbench-action=\"risk_repair\"",
        "data-workbench-action=\"alert_action\"",
        "data-workbench-action=\"audit_action\"",
        "data-workbench-action=\"provenance_repair\"",
        "/actions/risk",
        "/actions/alert",
        "/actions/audit",
        "/actions/provenance_repair",
    ] {
        assert!(
            !DASHBOARD_HTML.contains(forbidden_control),
            "workbench shell must not expose operation control {forbidden_control}"
        );
        assert!(
            !DASHBOARD_JS.contains(forbidden_control),
            "workbench JS must not expose operation control {forbidden_control}"
        );
    }
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
    let snapshot = DashboardSnapshot::from_nodes("2026-06-07T14:02:00Z", vec![running, stopped]);
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

#[test]
fn missing_supervisor_registry_records_gap() {
    let root = temp_root("missing-registry");
    let snapshot =
        snapshot_from_supervisor_artifacts(root.join("registry.json"), "2026-06-07T15:00:00Z")
            .unwrap();

    assert!(snapshot.nodes.is_empty());
    assert_eq!(snapshot.gaps.len(), 1);
    assert_eq!(snapshot.gaps[0].field_path, "supervisor.registry");
    assert_eq!(snapshot.gaps[0].reason, DashboardAvailability::Unknown);
    assert!(
        snapshot.gaps[0]
            .notes
            .value
            .as_deref()
            .unwrap()
            .contains("missing")
    );
}

#[test]
fn empty_supervisor_registry_records_not_configured_gap() {
    let root = temp_root("empty-registry");
    let registry_path = root.join("registry.json");
    write_registry(&registry_path, []);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:00Z").unwrap();

    assert!(snapshot.nodes.is_empty());
    assert_eq!(snapshot.overview.node_count, 0);
    assert_eq!(snapshot.gaps[0].field_path, "nodes");
    assert_eq!(
        snapshot.gaps[0].reason,
        DashboardAvailability::NotConfigured
    );
}

#[test]
fn workflow_manifest_artifacts_populate_dashboard_snapshot() {
    let root = temp_root("workflow-manifest");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let manifest_path = root.join("runs/workflows/v05-smoke/manifest.json");
    write_workflow_manifest(&manifest_path, "v05-smoke", false);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:30Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    let workflow = &snapshot.workflow_artifacts[0];
    assert_eq!(workflow.run_id, "v05-smoke");
    assert_eq!(workflow.workflow, "binance-sandbox");
    assert_eq!(workflow.schema_version, "ntpro.workflow_manifest.v1");
    assert_eq!(workflow.runtime_status, "completed");
    assert_eq!(workflow.health, HealthStatus::Healthy);
    assert_eq!(workflow.artifact_count, 9);
    assert_eq!(
        workflow.market_fixture_id.value.as_deref(),
        Some("v04-binance-spot-bars")
    );
    assert_eq!(
        workflow.order_lifecycle_id.value.as_deref(),
        Some("v04-binance-mock-order-lifecycle")
    );
    assert_eq!(
        workflow.risk_smoke_id.value.as_deref(),
        Some("v04-binance-risk-rejection-smoke")
    );
    assert!(workflow.sandbox_only);
    assert!(workflow.fixture_replay);
    assert!(workflow.mock_execution);
    assert!(!workflow.external_venue_connection);
    assert!(!workflow.production_venue_connection);
    assert!(!workflow.testnet_public_network_connection);
    assert!(!workflow.external_network_attempted);
    assert!(snapshot.overview.sandbox_only);
    assert!(!snapshot.overview.production_venue_connection);
    assert!(!snapshot.overview.testnet_public_network_connection);
    assert!(!snapshot.overview.external_network_attempted);
    assert!(!workflow.real_funds);
    assert!(!workflow.production_trading);
    assert!(!workflow.real_orders_submitted);
    assert!(!workflow.testnet_connection);
    assert!(!workflow.network_attempted);
    assert_eq!(
        workflow.credential_policy.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        workflow.connectivity_mode.availability,
        DashboardAvailability::Unknown
    );
}

#[test]
fn explicit_workflow_root_populates_snapshot_when_registry_is_missing() {
    let root = temp_root("workflow-root-missing-registry");
    let registry_path = root.join("runs/supervisor/missing-registry.json");
    let workflow_root = root.join("runs/workflows");
    let manifest_path = workflow_root.join("v07-readonly-smoke/manifest.json");
    write_testnet_workflow_manifest(&manifest_path, "v07-readonly-smoke");

    let snapshot = snapshot_from_supervisor_artifacts_with_workflow_root(
        &registry_path,
        Some(workflow_root.as_path()),
        "2026-06-07T15:01:35Z",
    )
    .unwrap();

    assert!(snapshot.nodes.is_empty());
    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    assert_eq!(snapshot.workflow_artifacts[0].run_id, "v07-readonly-smoke");
    assert_eq!(snapshot.workflow_artifacts[0].workflow, "binance-testnet");
    assert_eq!(
        snapshot.workflow_artifacts[0].runtime_status,
        "dry_run_completed"
    );
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "supervisor.registry" && gap.reason == DashboardAvailability::Unknown
    }));
}

#[test]
fn testnet_workflow_manifest_populates_dashboard_runtime_surface() {
    let root = temp_root("testnet-workflow-manifest");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let manifest_path = root.join("runs/workflows/v07-readonly-smoke/manifest.json");
    write_testnet_workflow_manifest(&manifest_path, "v07-readonly-smoke");

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:40Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    let workflow = &snapshot.workflow_artifacts[0];
    assert_eq!(workflow.run_id, "v07-readonly-smoke");
    assert_eq!(workflow.workflow, "binance-testnet");
    assert_eq!(workflow.runtime_status, "dry_run_completed");
    assert_eq!(workflow.health, HealthStatus::Healthy);
    assert_eq!(workflow.artifact_count, 11);
    assert!(workflow.sandbox_only);
    assert!(workflow.mock_execution);
    assert!(!workflow.fixture_replay);
    assert!(!workflow.testnet_connection);
    assert!(!workflow.network_attempted);
    assert!(!workflow.external_venue_connection);
    assert!(!workflow.production_venue_connection);
    assert!(!workflow.testnet_public_network_connection);
    assert!(!workflow.external_network_attempted);
    assert!(!workflow.real_funds);
    assert!(!workflow.production_trading);
    assert!(!workflow.real_orders_submitted);
    assert_eq!(
        workflow.credential_policy.value.as_deref(),
        Some("env-var-only-no-secret-persistence")
    );
    assert_eq!(workflow.connectivity_mode.value.as_deref(), Some("dry-run"));
    assert_eq!(
        workflow.order_submission_mode.value.as_deref(),
        Some("disabled")
    );
    assert_eq!(
        workflow.reconciliation_mode.value.as_deref(),
        Some("artifact-only")
    );
}

#[test]
fn testnet_probe_artifacts_populate_dashboard_read_only_fields() {
    let root = temp_root("testnet-probe-artifacts");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let workflow_dir = root.join("runs/workflows/v07-probe");
    let manifest_path = workflow_dir.join("manifest.json");
    write_testnet_workflow_manifest_with_artifacts(
        &manifest_path,
        "v07-probe",
        &json!([
            {
                "path": "testnet/credential_policy.json",
                "schema_version": "ntpro.v07_binance_testnet_credential_policy.v1"
            },
            {
                "path": TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH,
                "schema_version": "ntpro.v07_binance_testnet_connectivity_probe.v1"
            },
            {
                "path": TESTNET_HTTP_CONNECTIVITY_PROBE_ARTIFACT_PATH,
                "schema_version": "ntpro.v07_binance_testnet_http_probe.v1"
            },
            {
                "path": TESTNET_WEBSOCKET_PROBE_ARTIFACT_PATH,
                "schema_version": "ntpro.v07_binance_testnet_ws_probe.v1"
            }
        ]),
    );
    fs::create_dir_all(workflow_dir.join("testnet")).unwrap();
    fs::write(
        workflow_dir.join("testnet/credential_policy.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v07_binance_testnet_credential_policy.v1",
            "policy": "env-var-only-no-secret-persistence",
            "credential_source": "environment_variables_only",
            "values_recorded": false,
            "secrets_redacted": true
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        workflow_dir.join(TESTNET_CONNECTIVITY_PROBE_ARTIFACT_PATH),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v07_binance_testnet_connectivity_probe.v1",
            "status": "http_read_only_probe_ok",
            "latency_ms": 42,
            "endpoint_class": "binance-testnet-public-http-time",
            "error_code": "none",
            "network_permission_requested": true,
            "network_attempted": true,
            "testnet_connection": true
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        workflow_dir.join(TESTNET_HTTP_CONNECTIVITY_PROBE_ARTIFACT_PATH),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v07_binance_testnet_http_probe.v1",
            "status": "http_read_only_probe_ok",
            "latency_ms": 42,
            "endpoint_kind": "http_read_only",
            "error_code": "none",
            "network_permission_requested": true,
            "network_attempted": true,
            "testnet_connection": true,
            "response_shape": "binance_server_time_v1",
            "response_shape_validated": true
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        workflow_dir.join(TESTNET_WEBSOCKET_PROBE_ARTIFACT_PATH),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v07_binance_testnet_ws_probe.v1",
            "status": "websocket_read_only_probe_failed",
            "error_code": "handshake_error",
            "websocket_attempted": true,
            "network_attempted": true,
            "testnet_connection": false,
            "subscription_attempted": false,
            "message_count": 0,
            "order_submission": "disabled",
            "real_orders_submitted": false,
            "values_recorded": false,
            "secrets_redacted": true
        }))
        .unwrap(),
    )
    .unwrap();

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:43Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    let workflow = &snapshot.workflow_artifacts[0];
    assert_eq!(workflow.workflow, "binance-testnet");
    assert_eq!(workflow.health, HealthStatus::Healthy);
    assert!(workflow.network_permission_requested);
    assert!(workflow.network_attempted);
    assert!(workflow.testnet_connection);
    assert!(!workflow.production_venue_connection);
    assert!(workflow.testnet_public_network_connection);
    assert!(workflow.external_network_attempted);
    assert!(snapshot.overview.sandbox_only);
    assert!(!snapshot.overview.production_venue_connection);
    assert!(snapshot.overview.testnet_public_network_connection);
    assert!(snapshot.overview.external_network_attempted);
    assert_eq!(
        workflow.probe_status.value.as_deref(),
        Some("http_read_only_probe_ok")
    );
    assert_eq!(workflow.probe_latency_ms.value, Some(42));
    assert_eq!(
        workflow.probe_endpoint_class.value.as_deref(),
        Some("http_read_only")
    );
    assert_eq!(workflow.probe_error_code.value.as_deref(), Some("none"));
    assert_eq!(workflow.values_recorded.value, Some(false));
    assert_eq!(workflow.secrets_redacted.value, Some(true));
    assert_eq!(
        workflow.websocket_probe_status.value.as_deref(),
        Some("websocket_read_only_probe_failed")
    );
    assert_eq!(
        workflow.websocket_error_code.value.as_deref(),
        Some("handshake_error")
    );
    assert!(workflow.websocket_attempted);
    assert!(!workflow.websocket_subscription_attempted);
    assert_eq!(workflow.websocket_message_count.value, Some(0));
    assert!(!workflow.real_orders_submitted);
    assert_eq!(
        workflow.order_submission_mode.value.as_deref(),
        Some("disabled")
    );
}

#[test]
fn authenticated_readonly_probe_artifact_populates_dashboard_read_only_fields() {
    let root = temp_root("authenticated-readonly-probe-artifact");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let workflow_dir = root.join("runs/workflows/v08-auth-readonly");
    let manifest_path = workflow_dir.join("manifest.json");
    write_testnet_workflow_manifest_with_artifacts(
        &manifest_path,
        "v08-auth-readonly",
        &json!([
            {
                "path": "testnet/credential_policy.json",
                "schema_version": "ntpro.v08_binance_testnet_credential_policy.v1"
            },
            {
                "path": TESTNET_AUTHENTICATED_READONLY_PROBE_ARTIFACT_PATH,
                "schema_version": "ntpro.v08_binance_testnet_authenticated_readonly_probe.v1"
            }
        ]),
    );
    fs::create_dir_all(workflow_dir.join("testnet")).unwrap();
    fs::write(
        workflow_dir.join("testnet/credential_policy.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v08_binance_testnet_credential_policy.v1",
            "policy": "env-var-only-no-secret-persistence",
            "credential_source": "environment_variables_only",
            "api_key_present": true,
            "api_secret_present": true,
            "values_recorded": false,
            "api_key_value_recorded": false,
            "api_secret_value_recorded": false,
            "secrets_redacted": true
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        workflow_dir.join(TESTNET_AUTHENTICATED_READONLY_PROBE_ARTIFACT_PATH),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v08_binance_testnet_authenticated_readonly_probe.v1",
            "run_id": "v08-auth-readonly",
            "environment": "testnet",
            "product": "spot",
            "endpoint_kind": "authenticated_http_read_only",
            "endpoint_class": "binance-testnet-authenticated-readonly-account",
            "endpoint_url_redacted": "https://testnet.binance.vision/api/v3/account",
            "network_gate_status": "manual-online-disabled",
            "network_gate_reasons": ["manual gate closed"],
            "network_permission_requested": false,
            "env_network_permission": false,
            "network_attempted": false,
            "testnet_connection": false,
            "credential_policy": "env-var-only-no-secret-persistence",
            "api_key_present": true,
            "api_secret_present": true,
            "request_method": "GET",
            "request_target": "/api/v3/account",
            "query_shape": "timestamp=<ms>&recvWindow=<ms>&signature=<redacted>",
            "api_key_header_name": "X-MBX-APIKEY",
            "api_key_header_value_recorded": false,
            "signature_recorded": false,
            "signed_query_recorded": false,
            "signed_url_recorded": false,
            "raw_response_recorded": false,
            "balances_recorded": false,
            "uid_recorded": false,
            "account_mutation": false,
            "order_submission": "disabled",
            "real_orders_submitted": false,
            "production_venue_connection": false,
            "real_funds": false,
            "production_trading": false,
            "response_status_code": null,
            "response_shape": "binance_account_readonly_redacted_v1",
            "response_shape_validated": false,
            "latency_ms": null,
            "error_code": "manual_online_gate_closed",
            "status": "authenticated_readonly_probe_deferred",
            "diagnostic": "manual online gate closed; no authenticated request attempted",
            "generated_at": "2026-06-16T14:30:00Z"
        }))
        .unwrap(),
    )
    .unwrap();

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-16T14:30:01Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    let workflow = &snapshot.workflow_artifacts[0];
    assert_eq!(
        workflow.authenticated_probe_status.value.as_deref(),
        Some("authenticated_readonly_probe_deferred")
    );
    assert_eq!(
        workflow.authenticated_endpoint_kind.value.as_deref(),
        Some("authenticated_http_read_only")
    );
    assert_eq!(
        workflow.authenticated_request_method.value.as_deref(),
        Some("GET")
    );
    assert_eq!(
        workflow.authenticated_response_shape.value.as_deref(),
        Some("binance_account_readonly_redacted_v1")
    );
    assert_eq!(
        workflow.authenticated_response_shape_validated.value,
        Some(false)
    );
    assert_eq!(workflow.authenticated_api_key_present.value, Some(true));
    assert_eq!(workflow.authenticated_api_secret_present.value, Some(true));
    assert_eq!(workflow.authenticated_secrets_redacted.value, Some(true));
    assert_eq!(workflow.authenticated_account_mutation.value, Some(false));
    assert_eq!(
        workflow.authenticated_real_orders_submitted.value,
        Some(false)
    );
    assert_eq!(
        workflow.authenticated_production_venue_connection.value,
        Some(false)
    );
    assert!(!workflow.network_permission_requested);
    assert!(!workflow.network_attempted);
    assert!(!workflow.testnet_connection);
    assert!(!workflow.real_orders_submitted);
    assert!(!workflow.production_venue_connection);

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
    let rendered = serde_json::to_string(&snapshot_value).unwrap();
    assert!(!rendered.contains("ntpro_v080006_synthetic_key"));
    assert!(!rendered.contains("ntpro_v080006_synthetic_secret"));
    assert!(!rendered.contains("raw_response_recorded"));
    assert!(!rendered.contains("signature_recorded"));
    assert!(!rendered.contains("signed_query_recorded"));
    assert!(!rendered.contains("signed_url_recorded"));
    assert!(!rendered.contains("balances_recorded"));
    assert!(!rendered.contains("uid_recorded"));
}

#[test]
fn testnet_order_proof_artifacts_populate_dashboard_read_only_fields() {
    let root = temp_root("testnet-order-proof-dashboard");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let workflow_dir = root.join("runs/workflows/v100-order-proof");
    let manifest_path = workflow_dir.join("manifest.json");
    write_testnet_workflow_manifest_with_artifacts(
        &manifest_path,
        "v100-order-proof",
        &json!([
            {
                "path": "testnet_order_proof/risk_preflight.json",
                "schema_version": "ntpro.v100_order_preflight_report.v1"
            },
            {
                "path": "testnet_order_proof/order_test.json",
                "schema_version": "ntpro.v100_order_test_preflight_report.v1"
            },
            {
                "path": "testnet_order_proof/execution_artifact_contract.json",
                "schema_version": "ntpro.v100_execution_artifact_contract.v1"
            },
            {
                "path": "testnet_order_proof/reconciliation.json",
                "schema_version": "ntpro.v100_reconciliation_fixture_report.v1"
            }
        ]),
    );
    let proof_dir = workflow_dir.join("testnet_order_proof");
    fs::create_dir_all(&proof_dir).unwrap();
    fs::write(
        proof_dir.join("risk_preflight.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v100_order_preflight_report.v1",
            "status": "pass",
            "passed": true,
            "order_submission_remains_disabled": true,
            "network_attempted": false,
            "real_orders_submitted": false,
            "production_endpoint_allowed": false,
            "dashboard_order_controls": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        proof_dir.join("order_test.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v100_order_test_preflight_report.v1",
            "status": "ready",
            "binance_order_test_acceptance": "not_attempted_offline_manual_only",
            "matching_engine_submission": false,
            "order_submission_remains_disabled": true,
            "network_attempted": false,
            "real_orders_submitted": false,
            "production_endpoint_allowed": false,
            "dashboard_order_controls": false,
            "secrets_redacted": true
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        proof_dir.join("execution_artifact_contract.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v100_execution_artifact_contract.v1",
            "status": "ready",
            "artifact_family": "binance-testnet-order-lifecycle-proof",
            "order_test_artifact": {
                "name": "order_test.json",
                "status": "schema_defined_offline_acceptance_not_attempted"
            },
            "submit_ack_artifact": {
                "name": "submit_ack.json",
                "status": "manual_online_artifact_required_not_observed_offline"
            },
            "cancel_ack_artifact": {
                "name": "cancel_ack.json",
                "status": "manual_online_artifact_required_not_observed_offline"
            },
            "lifecycle_artifact": {
                "name": "lifecycle.json",
                "status": "manual_online_artifact_required_not_observed_offline"
            },
            "reconciliation_artifact": {
                "name": "reconciliation.json",
                "status": "schema_defined_manual_or_fixture_input_required"
            },
            "counters": {
                "testnet_orders_submitted": 0,
                "testnet_orders_canceled": 0,
                "production_orders_submitted": 0,
                "production_orders_canceled": 0
            },
            "manual_submit_cancel_proof_observed": false,
            "matching_engine_submission": false,
            "order_submission_remains_disabled": true,
            "network_attempted": false,
            "real_orders_submitted": false,
            "production_endpoint_allowed": false,
            "dashboard_order_controls": false,
            "secrets_redacted": true
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        proof_dir.join("reconciliation.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v100_reconciliation_fixture_report.v1",
            "status": "risk_halted",
            "scenario": "all",
            "scenario_count": 4,
            "risk_halted": true,
            "new_orders_blocked": true,
            "manual_submit_cancel_proof_observed": false,
            "matching_engine_submission": false,
            "order_submission_remains_disabled": true,
            "network_attempted": false,
            "real_orders_submitted": false,
            "production_endpoint_allowed": false,
            "dashboard_order_controls": false,
            "counters": {
                "testnet_orders_submitted": 0,
                "testnet_orders_canceled": 0,
                "production_orders_submitted": 0,
                "production_orders_canceled": 0
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-19T16:45:00Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    let workflow = &snapshot.workflow_artifacts[0];
    assert_eq!(
        workflow.order_proof_risk_preflight_status.value.as_deref(),
        Some("pass")
    );
    assert_eq!(
        workflow.order_proof_order_test_status.value.as_deref(),
        Some("schema_defined_offline_acceptance_not_attempted")
    );
    assert_eq!(
        workflow.order_proof_submit_ack_status.value.as_deref(),
        Some("manual_online_artifact_required_not_observed_offline")
    );
    assert_eq!(
        workflow.order_proof_cancel_ack_status.value.as_deref(),
        Some("manual_online_artifact_required_not_observed_offline")
    );
    assert_eq!(
        workflow.order_proof_terminal_status.value.as_deref(),
        Some("manual_online_artifact_required_not_observed_offline")
    );
    assert_eq!(
        workflow.order_proof_reconciliation_status.value.as_deref(),
        Some("risk_halted")
    );
    assert_eq!(
        workflow.order_proof_manual_submit_cancel_observed.value,
        Some(false)
    );
    assert_eq!(workflow.order_proof_testnet_orders_submitted.value, Some(0));
    assert_eq!(workflow.order_proof_testnet_orders_canceled.value, Some(0));
    assert_eq!(
        workflow.order_proof_production_orders_submitted.value,
        Some(0)
    );
    assert_eq!(
        workflow.order_proof_production_orders_canceled.value,
        Some(0)
    );
    assert_eq!(
        workflow.order_proof_dashboard_order_controls.value,
        Some(false)
    );
    assert!(!workflow.real_orders_submitted);
    assert!(!workflow.production_venue_connection);

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
    let rendered = serde_json::to_string(&snapshot_value).unwrap();
    assert!(rendered.contains("order_proof_risk_preflight_status"));
    assert!(rendered.contains("order_proof_reconciliation_status"));
    assert!(!DASHBOARD_JS.contains("submit_order"));
    assert!(!DASHBOARD_JS.contains("cancel_order"));
}

#[test]
fn missing_workflow_child_artifact_records_gap_and_degrades_health() {
    let root = temp_root("missing-workflow-child-artifact");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let manifest_path = root.join("runs/workflows/v07-readonly-smoke/manifest.json");
    write_testnet_workflow_manifest_with_artifacts(
        &manifest_path,
        "v07-readonly-smoke",
        &json!([
            {
                "path": "testnet/credential_policy.json",
                "schema_version": "ntpro.v07_binance_testnet_credential_policy.v1"
            }
        ]),
    );

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:41Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    assert_eq!(
        snapshot.workflow_artifacts[0].health,
        HealthStatus::Degraded
    );
    assert!(
        snapshot.workflow_artifacts[0]
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|value| value.contains("degraded"))
    );
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.owner_task.value.as_deref() == Some("V061-005")
            && gap.field_path.contains("testnet/credential_policy.json")
            && gap
                .notes
                .value
                .as_deref()
                .is_some_and(|notes| notes.contains("artifact missing"))
    }));
}

#[test]
fn invalid_workflow_child_jsonl_records_gap_and_degrades_health() {
    let root = temp_root("invalid-workflow-child-jsonl");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let workflow_dir = root.join("runs/workflows/v07-readonly-smoke");
    let manifest_path = workflow_dir.join("manifest.json");
    write_testnet_workflow_manifest_with_artifacts(
        &manifest_path,
        "v07-readonly-smoke",
        &json!([
            {
                "path": "events.jsonl",
                "schema_version": "ntpro.workflow_event.v1"
            }
        ]),
    );
    fs::write(workflow_dir.join("events.jsonl"), "not-json\n").unwrap();

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:41Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    assert_eq!(
        snapshot.workflow_artifacts[0].health,
        HealthStatus::Degraded
    );
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.owner_task.value.as_deref() == Some("V061-005")
            && gap.field_path.contains("events.jsonl")
            && gap
                .notes
                .value
                .as_deref()
                .is_some_and(|notes| notes.contains("JSONL"))
    }));
}

#[test]
fn testnet_workflow_dashboard_uses_manifest_run_id_not_directory_name() {
    let root = temp_root("testnet-workflow-effective-run-id");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let manifest_path = root.join("runs/workflows/config-declared-run-id/manifest.json");
    write_testnet_workflow_manifest(&manifest_path, "custom-run-id");

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:42Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    let workflow = &snapshot.workflow_artifacts[0];
    assert_eq!(workflow.run_id, "custom-run-id");
    assert_eq!(workflow.workflow, "binance-testnet");
    assert_eq!(workflow.runtime_status, "dry_run_completed");
    assert_eq!(workflow.health, HealthStatus::Healthy);
}

#[test]
fn invalid_workflow_manifest_records_gap() {
    let root = temp_root("invalid-workflow-manifest");
    let registry_path = root.join("runs/supervisor/registry.json");
    write_registry(&registry_path, []);
    let manifest_path = root.join("runs/workflows/broken/manifest.json");
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(&manifest_path, "not-json").unwrap();

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:45Z").unwrap();

    assert_eq!(snapshot.workflow_artifacts.len(), 1);
    assert_eq!(snapshot.workflow_artifacts[0].run_id, "unknown");
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path.contains("workflow_artifacts")
            && gap.reason == DashboardAvailability::Unknown
            && gap.owner_task.value.as_deref() == Some("V05-006")
    }));
}

#[test]
fn one_node_supervisor_artifacts_populate_dashboard_sections() {
    let root = temp_root("one-node");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    let status = node_status_for_record(&record, LifecycleStatus::Running);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    record.process.state = SupervisorProcessState::Running;
    record.process.pid = SnapshotValue::available(42_001);
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:02:00Z").unwrap();

    assert_eq!(snapshot.overview.node_count, 1);
    assert_eq!(snapshot.overview.running_nodes, 1);
    assert!(!snapshot.overview.external_venue_connection);
    assert!(!snapshot.overview.real_orders_submitted);
    assert_eq!(snapshot.nodes[0].node_id, "sandbox-a");
    assert_eq!(
        snapshot.nodes[0].process_state,
        SupervisorProcessState::Running
    );
    assert_eq!(snapshot.nodes[0].pid.value, Some(42_001));
    assert_eq!(snapshot.data_sources[0].source_id, "sandbox-a:data");
    assert_eq!(
        snapshot.execution_gateways[0].gateway_id,
        "sandbox-a:gateway"
    );
    assert_eq!(snapshot.risk.availability, DashboardAvailability::Available);
    assert_eq!(
        snapshot.sandbox_business.availability,
        DashboardAvailability::Available
    );
    assert_eq!(
        snapshot.sandbox_business.exchange.venue.value.as_deref(),
        Some("BINANCE")
    );
    assert_eq!(snapshot.sandbox_business.strategies.len(), 2);
    assert_eq!(
        snapshot.sandbox_business.order.mock_orders_requested.value,
        Some(7)
    );
    assert_eq!(
        snapshot.sandbox_business.risk.risk_reason.value.as_deref(),
        Some(V04_BINANCE_RISK_REJECTION_REASON)
    );
    assert_eq!(snapshot.logs.len(), 3);
    assert!(
        snapshot
            .logs
            .iter()
            .all(|log| log.availability == DashboardAvailability::Available)
    );
    assert!(snapshot.metrics.iter().any(|metric| {
        metric.metric_id == "sandbox-a:starts_total" && metric.value.value.as_deref() == Some("1")
    }));
    assert!(snapshot.runtime_modules.iter().any(|module| {
        module.module_name == "sandbox-a:NautilusKernel"
            && module.status.availability == DashboardAvailability::NotSupported
    }));
    assert!(snapshot.runtime_modules.iter().any(|module| {
        module.module_name == "sandbox-a:LiveNode"
            && module.status.value.as_deref() == Some("running")
            && module.health == HealthStatus::Healthy
    }));
    assert!(snapshot.runtime_modules.iter().any(|module| {
        module.module_name == "sandbox-a:Metrics writer"
            && module.status.availability == DashboardAvailability::Available
    }));
    assert_eq!(snapshot.runtime_modules.len(), 11);
    assert_eq!(snapshot.controls.len(), 6);
    assert!(snapshot.controls.iter().any(|control| {
        control.action == "start:sandbox-a"
            && !control.enabled
            && control.availability == DashboardAvailability::Available
    }));
    assert!(snapshot.controls.iter().any(|control| {
        control.action == "stop:sandbox-a"
            && control.enabled
            && control.availability == DashboardAvailability::Available
    }));
    assert!(snapshot.controls.iter().any(|control| {
        control.action == "reconnect_execution:sandbox-a"
            && control.enabled
            && control.availability == DashboardAvailability::Available
    }));
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "runtime_modules.sandbox-a.nautiluskernel"
            && gap.reason == DashboardAvailability::NotSupported
    }));
}

#[test]
fn strategy_runtime_artifacts_populate_readonly_dashboard_snapshot() {
    let root = temp_root("strategy-runtime-readonly");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "strategy-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_strategy_runtime_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-18T10:30:00Z").unwrap();

    assert_eq!(snapshot.strategy_runtime.len(), 1);
    let runtime = &snapshot.strategy_runtime[0];
    assert_eq!(runtime.node_id, "strategy-a");
    assert_eq!(runtime.health, HealthStatus::Healthy);
    assert_eq!(
        runtime.diagnostic.value.as_deref(),
        Some("strategy_session_artifacts_ok")
    );
    assert_eq!(
        runtime.session_id.value.as_deref(),
        Some("btc-ema-shadow-001")
    );
    assert_eq!(runtime.session_state.value.as_deref(), Some("stopped"));
    assert_eq!(
        runtime.strategy_id.value.as_deref(),
        Some("ema_cross_btcusdt_v1")
    );
    assert_eq!(runtime.symbol.value.as_deref(), Some("BTCUSDT.BINANCE"));
    assert_eq!(
        runtime.market_stream_status.value.as_deref(),
        Some("exhausted")
    );
    assert_eq!(runtime.signal_count.value, Some(2));
    assert!(
        runtime
            .latest_signal
            .value
            .as_deref()
            .is_some_and(|value| value.contains("flat BTCUSDT.BINANCE"))
    );
    assert!(
        runtime
            .latest_order_intent
            .value
            .as_deref()
            .is_some_and(|value| value.contains("allowed=false"))
    );
    assert!(
        runtime
            .latest_risk_decision
            .value
            .as_deref()
            .is_some_and(|value| value.contains("actual_submission=false"))
    );
    assert_eq!(
        runtime.rejection_reason.value.as_deref(),
        Some("order_submission_disabled,shadow_mode_actual_submission_disabled")
    );
    assert_eq!(
        runtime.order_submission_mode.value.as_deref(),
        Some("disabled")
    );
    assert_eq!(runtime.actual_submission_count.value, Some(0));
    assert!(
        runtime
            .session_status_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("strategy/session_status.json"))
    );
    assert!(
        runtime
            .order_intent_artifact_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("strategy/order_intent.jsonl"))
    );
    assert!(
        runtime
            .manifest_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("strategy/manifest.json"))
    );

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
}

#[test]
fn trader_terminal_read_model_artifact_populates_runtime_bridge() {
    let root = temp_root("trader-terminal-read-model-ready");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |_| {});
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:00:00Z").unwrap();

    assert_eq!(snapshot.read_model_runtime.len(), 1);
    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.node_id, "terminal-a");
    assert_eq!(runtime.health, HealthStatus::Healthy);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("ready_readonly_artifact")
    );
    assert_eq!(
        runtime.contract_version.value.as_deref(),
        Some(UNIFIED_READ_MODEL_CONTRACT_VERSION)
    );
    assert_eq!(
        runtime.schema_version.value.as_deref(),
        Some(UNIFIED_READ_MODEL_SCHEMA_VERSION)
    );
    assert_eq!(
        runtime.snapshot_kind.value.as_deref(),
        Some("unified_snapshot")
    );
    assert_eq!(
        runtime.snapshot_health_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(runtime.freshness_status.value.as_deref(), Some("fresh"));
    assert_eq!(runtime.source_type.value.as_deref(), Some("artifact"));
    assert_eq!(runtime.redaction_state.value.as_deref(), Some("redacted"));
    assert_eq!(runtime.account_status.value.as_deref(), Some("healthy"));
    assert_eq!(runtime.positions_status.value.as_deref(), Some("healthy"));
    assert_eq!(runtime.orders_status.value.as_deref(), Some("healthy"));
    assert_eq!(runtime.fills_status.value.as_deref(), Some("healthy"));
    assert_eq!(runtime.risk_status.value.as_deref(), Some("healthy"));
    assert_eq!(runtime.lifecycle_status.value.as_deref(), Some("healthy"));
    assert_eq!(
        runtime.operation_entry_status.value.as_deref(),
        Some("blocked_missing_owner_approval")
    );
    assert!(
        runtime
            .account_summary
            .value
            .as_deref()
            .is_some_and(|summary| summary.contains("summary_status=ready"))
    );
    assert_eq!(
        runtime.account_freshness_status.value.as_deref(),
        Some("fresh")
    );
    assert_eq!(
        runtime.account_source_type.value.as_deref(),
        Some("artifact")
    );
    assert_eq!(
        runtime.account_source_ref.value.as_deref(),
        Some("artifact://v0_21/unified_read_model_snapshot.json")
    );
    assert_eq!(
        runtime.account_redaction_state.value.as_deref(),
        Some("redacted")
    );
    assert_eq!(runtime.account_risk_state.value.as_deref(), Some("active"));
    assert_eq!(runtime.account_equity.value.as_deref(), Some("1000.00"));
    assert_eq!(
        runtime.account_available_balance.value.as_deref(),
        Some("900.00")
    );
    assert_eq!(
        runtime.account_balance_entry_count.value.as_deref(),
        Some("2")
    );
    assert_eq!(
        runtime.positions_freshness_status.value.as_deref(),
        Some("fresh")
    );
    assert_eq!(
        runtime.positions_source_ref.value.as_deref(),
        Some("artifact://v0_21/unified_read_model_snapshot.json")
    );
    assert_eq!(
        runtime.positions_redaction_state.value.as_deref(),
        Some("redacted")
    );
    assert_eq!(
        runtime.positions_account_id.value.as_deref(),
        Some("acct-redacted-001")
    );
    assert_eq!(
        runtime.positions_net_position_side.value.as_deref(),
        Some("flat")
    );
    assert_eq!(runtime.positions_quantity.value.as_deref(), Some("0"));
    assert_eq!(runtime.positions_notional.value.as_deref(), Some("0"));
    assert_eq!(
        runtime.positions_precision.value.as_deref(),
        Some("standard")
    );
    assert!(
        runtime.positions_lineage.value.as_deref().is_some_and(
            |lineage| lineage.contains("ntpro.v210.trader_terminal_readonly_dashboard.v1")
        )
    );
    assert_eq!(
        runtime.orders_freshness_status.value.as_deref(),
        Some("fresh")
    );
    assert_eq!(
        runtime.orders_source_ref.value.as_deref(),
        Some("artifact://v0_21/unified_read_model_snapshot.json")
    );
    assert_eq!(
        runtime.orders_lifecycle_state.value.as_deref(),
        Some("readback_matched")
    );
    assert_eq!(
        runtime.orders_client_order_id.value.as_deref(),
        Some("client-redacted-001")
    );
    assert_eq!(
        runtime.orders_request_digest.value.as_deref(),
        Some("sha256-redacted-request-001")
    );
    assert_eq!(
        runtime.orders_attempt_id.value.as_deref(),
        Some("attempt-redacted-001")
    );
    assert_eq!(
        runtime.orders_approval_id.value.as_deref(),
        Some("approval-redacted-001")
    );
    assert_eq!(
        runtime.orders_readback_status.value.as_deref(),
        Some("matched")
    );
    assert_eq!(
        runtime.orders_audit_state.value.as_deref(),
        Some("audit_closed")
    );
    assert_eq!(runtime.orders_ledger_present.value.as_deref(), Some("true"));
    assert_eq!(
        runtime.orders_duplicate_attempt_detected.value.as_deref(),
        Some("false")
    );
    assert_eq!(runtime.orders_no_retry.value.as_deref(), Some("true"));
    assert_eq!(
        runtime.orders_exchange_truth.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.orders_adapter_runtime_integrated.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.orders_values_are_exchange_truth.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.fills_freshness_status.value.as_deref(),
        Some("fresh")
    );
    assert_eq!(
        runtime.fills_source_ref.value.as_deref(),
        Some("artifact://v0_21/unified_read_model_snapshot.json")
    );
    assert_eq!(
        runtime.fills_fill_id.value.as_deref(),
        Some("fill-redacted-001")
    );
    assert_eq!(
        runtime.fills_execution_id.value.as_deref(),
        Some("execution-redacted-001")
    );
    assert_eq!(
        runtime.fills_order_id.value.as_deref(),
        Some("order-redacted-001")
    );
    assert_eq!(
        runtime.fills_client_order_id.value.as_deref(),
        Some("client-redacted-001")
    );
    assert_eq!(
        runtime.fills_order_linkage_status.value.as_deref(),
        Some("linked")
    );
    assert_eq!(
        runtime.fills_reconciliation_status.value.as_deref(),
        Some("reconciled")
    );
    assert_eq!(runtime.fills_quantity.value.as_deref(), Some("0.010"));
    assert_eq!(
        runtime.fills_cumulative_quantity.value.as_deref(),
        Some("0.010")
    );
    assert_eq!(runtime.fills_remaining_quantity.value.as_deref(), Some("0"));
    assert_eq!(runtime.fills_quantity_precision.value.as_deref(), Some("3"));
    assert_eq!(runtime.fills_price.value.as_deref(), Some("61000.00"));
    assert_eq!(runtime.fills_price_precision.value.as_deref(), Some("2"));
    assert_eq!(
        runtime.fills_precision_status.value.as_deref(),
        Some("valid")
    );
    assert_eq!(
        runtime.fills_duplicate_fill_detected.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.fills_partial_fill_detected.value.as_deref(),
        Some("false")
    );
    assert!(
        runtime
            .fills_risk_projection_input
            .value
            .as_deref()
            .is_some_and(|risk| risk.contains("fill_reconciliation_status=reconciled"))
    );
    assert_eq!(runtime.fills_exchange_truth.value.as_deref(), Some("false"));
    assert_eq!(
        runtime.fills_adapter_runtime_integrated.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.fills_values_are_exchange_truth.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.risk_priority_state.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(runtime.risk_state.value.as_deref(), Some("active"));
    assert_eq!(
        runtime.risk_freshness_status.value.as_deref(),
        Some("fresh")
    );
    assert_eq!(
        runtime.risk_source_ref.value.as_deref(),
        Some("artifact://v0_21/unified_read_model_snapshot.json")
    );
    assert_eq!(runtime.risk_visible.value.as_deref(), Some("false"));
    assert_eq!(
        runtime.risk_manual_review_required.value.as_deref(),
        Some("false")
    );
    assert_eq!(runtime.risk_halted.value.as_deref(), Some("false"));
    assert_eq!(
        runtime.risk_mismatch_detected.value.as_deref(),
        Some("false")
    );
    assert_eq!(runtime.risk_alert_severity.value.as_deref(), Some("info"));
    assert_eq!(
        runtime.risk_alert_missing_evidence.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime
            .risk_alert_forbidden_control_request
            .value
            .as_deref(),
        Some("false")
    );
    assert!(
        runtime.risk_lineage.value.as_deref().is_some_and(
            |lineage| lineage.contains("ntpro.v210.trader_terminal_readonly_dashboard.v1")
        )
    );
    assert_eq!(runtime.audit_state.value.as_deref(), Some("audit_closed"));
    assert_eq!(runtime.audit_closed.value.as_deref(), Some("true"));
    assert_eq!(
        runtime.audit_required_evidence_complete.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.audit_required_components_complete.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.audit_release_provenance.value.as_deref(),
        Some("ntpro-rust-only-v0.21.1")
    );
    assert_eq!(
        runtime.audit_artifact_digest.value.as_deref(),
        Some("sha256:read-model-dashboard-ready")
    );
    assert_eq!(
        runtime.audit_artifact_sha.value.as_deref(),
        Some("sha256:read-model-dashboard-ready")
    );
    assert_eq!(
        runtime.audit_provenance_mismatch.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.operation_entry_freshness_status.value.as_deref(),
        Some("fresh")
    );
    assert_eq!(
        runtime.operation_entry_source_ref.value.as_deref(),
        Some("artifact://v0_21/unified_read_model_snapshot.json")
    );
    assert_eq!(
        runtime.operation_intent_preview.value.as_deref(),
        Some("manual_operation_preview_only")
    );
    assert_eq!(
        runtime.operation_owner_approval_ref.value.as_deref(),
        Some("missing_owner_approval")
    );
    assert_eq!(
        runtime.operation_risk_decision_ref.value.as_deref(),
        Some("missing_risk_gate")
    );
    assert_eq!(
        runtime.operation_audit_evidence_ref.value.as_deref(),
        Some("missing_audit_gate")
    );
    assert_eq!(
        runtime.operation_entry_disabled.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_entry_blocked_reason.value.as_deref(),
        Some("missing_owner_approval,missing_risk_gate,missing_audit_gate")
    );
    assert_eq!(
        runtime.operation_missing_owner_approval.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_missing_risk_gate.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_missing_audit_gate.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_stale_read_model.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.operation_provenance_mismatch.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.operation_gates_complete.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.operation_ungated_attempted.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.operation_attempt_status.value.as_deref(),
        Some("fail_closed_without_gates")
    );
    assert_eq!(
        runtime
            .operation_ungated_attempt_fail_closed
            .value
            .as_deref(),
        Some("true")
    );
    assert_eq!(runtime.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(runtime.dashboard_fill_controls_enabled.value, Some(false));
    assert_eq!(runtime.dashboard_risk_controls_enabled.value, Some(false));
    assert_eq!(runtime.dashboard_submit_controls_enabled.value, Some(false));
    assert_eq!(
        runtime.dashboard_replace_controls_enabled.value,
        Some(false)
    );
    assert_eq!(runtime.dashboard_amend_controls_enabled.value, Some(false));
    assert_eq!(
        runtime.dashboard_flatten_controls_enabled.value,
        Some(false)
    );
    assert_eq!(
        runtime.trader_terminal_order_ticket_enabled.value,
        Some(false)
    );
    assert_eq!(
        runtime.trader_terminal_live_trading_claim.value,
        Some(false)
    );
    assert_eq!(
        runtime.product_grade_trading_terminal_claim.value,
        Some(false)
    );
    assert_eq!(
        runtime.v25_dashboard_surface_status.value.as_deref(),
        Some("ready_readonly_surface")
    );
    assert_eq!(
        runtime.v25_diagnostics_gate_status.value.as_deref(),
        Some("ready_slo_freshness_gate")
    );
    assert_eq!(
        runtime.v25_slo_status.value.as_deref(),
        Some("slo_evidence_ready")
    );
    assert_eq!(
        runtime.v25_freshness_threshold_status.value.as_deref(),
        Some("freshness_thresholds_ready")
    );
    assert_eq!(
        runtime.v25_source_truth_status.value.as_deref(),
        Some("artifact_truth_only")
    );
    assert_eq!(
        runtime.v25_release_provenance_status.value.as_deref(),
        Some("matched")
    );
    assert_eq!(
        runtime.v25_no_remediation_status.value.as_deref(),
        Some("no_remediation_no_trading_actions")
    );
    assert_eq!(
        runtime.v25_monitoring_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        runtime.v25_monitoring_effective_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        runtime.v25_alert_highest_severity.value.as_deref(),
        Some("warning")
    );
    assert_eq!(
        runtime.v25_incident_ack_status.value.as_deref(),
        Some("acknowledged")
    );
    assert_eq!(
        runtime.v25_runbook_evidence_ref.value.as_deref(),
        Some("audit:v250-runbook:acknowledged")
    );
    assert_eq!(
        runtime.v25_dr_preview_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        runtime.v25_dr_operator_approval_status.value.as_deref(),
        Some("blocked_preview")
    );
    assert_eq!(
        runtime.v25_surface_blocking_reasons.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        runtime.v26_dashboard_admin_surface_status.value.as_deref(),
        Some("ready_readonly_surface")
    );
    assert_eq!(
        runtime.v26_permission_boundary_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        runtime.v26_permission_roles_checked.value.as_deref(),
        Some("viewer,operator,release_gatekeeper,incident_owner,auditor")
    );
    assert_eq!(
        runtime.v26_operation_audit_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        runtime.v26_operation_audit_lineage.value.as_deref(),
        Some("audit:v260:operation-audit:chain")
    );
    assert_eq!(
        runtime.v26_deployment_provenance_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        runtime.v26_deployment_environment.value.as_deref(),
        Some("prod_like_readonly")
    );
    assert_eq!(
        runtime.v26_upgrade_rollback_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        runtime.v26_upgrade_rollback_preview.value.as_deref(),
        Some("preview_only_ready")
    );
    assert_eq!(
        runtime.v26_stability_status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        runtime.v26_stability_degradation_reason.value.as_deref(),
        Some("none")
    );
    assert_eq!(
        runtime.v26_admin_surface_blocking_reasons.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        runtime.production_order_submission_allowed.value,
        Some(false)
    );
    assert_eq!(runtime.production_order_mutation_allowed.value, Some(false));
    assert_eq!(runtime.order_permission_control_allowed.value, Some(false));
    assert_eq!(runtime.retry_order_allowed.value, Some(false));
    assert_eq!(runtime.automatic_cancel_allowed.value, Some(false));
    assert_eq!(
        runtime.automatic_order_remediation_allowed.value,
        Some(false)
    );
    assert_eq!(runtime.funds_transfer_allowed.value, Some(false));
    assert_eq!(
        runtime.account_configuration_mutation_allowed.value,
        Some(false)
    );
    assert_eq!(runtime.auto_flatten_position_allowed.value, Some(false));
    assert_eq!(runtime.automatic_position_repair_allowed.value, Some(false));
    assert_eq!(runtime.execution_algorithm_allowed.value, Some(false));
    assert_eq!(runtime.automatic_fill_repair_allowed.value, Some(false));
    assert_eq!(
        runtime.automatic_reconciliation_repair_allowed.value,
        Some(false)
    );
    assert_eq!(runtime.automatic_risk_action_allowed.value, Some(false));
    assert_eq!(runtime.automatic_risk_repair_allowed.value, Some(false));
    assert_eq!(runtime.automatic_alert_action_allowed.value, Some(false));
    assert_eq!(runtime.automatic_audit_action_allowed.value, Some(false));
    assert_eq!(
        runtime.automatic_provenance_repair_allowed.value,
        Some(false)
    );
    assert_eq!(runtime.manual_operation_entry_enabled.value, Some(false));
    assert_eq!(runtime.manual_operation_submit_allowed.value, Some(false));
    assert_eq!(runtime.manual_operation_cancel_allowed.value, Some(false));
    assert_eq!(runtime.manual_operation_retry_allowed.value, Some(false));
    assert_eq!(runtime.manual_operation_replace_allowed.value, Some(false));
    assert_eq!(runtime.manual_operation_amend_allowed.value, Some(false));
    assert_eq!(runtime.manual_operation_flatten_allowed.value, Some(false));
    assert_eq!(
        runtime.automatic_operation_action_allowed.value,
        Some(false)
    );
    assert!(
        runtime
            .artifact_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with(TRADER_TERMINAL_READ_MODEL_ARTIFACT_RELATIVE_PATH))
    );

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
    assert!(!DASHBOARD_JS.contains("submit_order"));
    assert!(!DASHBOARD_JS.contains("cancel_order"));
    assert!(!DASHBOARD_JS.contains("replace_order"));
    assert!(!DASHBOARD_JS.contains("amend_order"));
    assert!(!DASHBOARD_JS.contains("flatten_order"));
}

#[test]
fn trader_terminal_operation_entry_blocks_without_required_gates() {
    let runtime =
        trader_terminal_read_model_runtime_with_mutation("operation-entry-missing-gates", |_| {});

    assert_eq!(runtime.health, HealthStatus::Healthy);
    assert_eq!(
        runtime.operation_entry_status.value.as_deref(),
        Some("blocked_missing_owner_approval")
    );
    assert_eq!(
        runtime.operation_entry_blocked_reason.value.as_deref(),
        Some("missing_owner_approval,missing_risk_gate,missing_audit_gate")
    );
    assert_eq!(
        runtime.operation_entry_disabled.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_missing_owner_approval.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_missing_risk_gate.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_missing_audit_gate.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime
            .operation_ungated_attempt_fail_closed
            .value
            .as_deref(),
        Some("true")
    );
    assert_eq!(runtime.manual_operation_entry_enabled.value, Some(false));
    assert_eq!(runtime.manual_operation_submit_allowed.value, Some(false));
    assert_eq!(runtime.manual_operation_cancel_allowed.value, Some(false));

    let renderer = dashboard_js_function_body("renderTraderTerminalWorkbench");
    assert!(renderer.contains("workbench-panel-operation-entry"));
    assert!(renderer.contains("Owner approval ref"));
    assert!(renderer.contains("Risk decision ref"));
    assert!(renderer.contains("Audit evidence ref"));
    for forbidden in [
        "<button",
        "data-workbench-action",
        "/actions/submit",
        "/actions/cancel",
        "/actions/retry",
        "/actions/replace",
        "/actions/amend",
        "/actions/flatten",
        "fetch(",
    ] {
        assert!(
            !renderer.contains(forbidden),
            "operation entry renderer must stay display-only and not contain {forbidden}"
        );
    }
}

#[test]
fn trader_terminal_operation_entry_ready_gates_remains_disabled_preview() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "operation-entry-ready-gates-disabled",
        |artifact| {
            let entry = &mut artifact["components"]["operation_entry"]["data"];
            entry["entry_state"] = json!("disabled_gated_preview_ready");
            entry["owner_approval_ref"] = json!("approval:v220:owner:001");
            entry["risk_decision_ref"] = json!("risk:v220:decision:001");
            entry["audit_evidence_ref"] = json!("audit:v220:evidence:001");
            entry["blocked_reason"] = json!("none");
            entry["gates_complete"] = json!(true);
            entry["attempt_status"] = json!("not_attempted_preview_only");
            entry["blocked_states"] = json!({
                "missing_owner_approval": false,
                "missing_risk_gate": false,
                "missing_audit_gate": false,
                "stale_read_model": false,
                "provenance_mismatch": false
            });
        },
    );

    assert_eq!(runtime.health, HealthStatus::Healthy);
    assert_eq!(
        runtime.operation_entry_status.value.as_deref(),
        Some("disabled_gated_preview_ready")
    );
    assert_eq!(
        runtime.operation_entry_blocked_reason.value.as_deref(),
        Some("none")
    );
    assert_eq!(
        runtime.operation_gates_complete.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_entry_disabled.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime.operation_owner_approval_ref.value.as_deref(),
        Some("approval:v220:owner:001")
    );
    assert_eq!(runtime.manual_operation_entry_enabled.value, Some(false));
    assert_eq!(runtime.manual_operation_submit_allowed.value, Some(false));
    assert_eq!(runtime.manual_operation_replace_allowed.value, Some(false));
    assert_eq!(runtime.manual_operation_flatten_allowed.value, Some(false));
}

#[test]
fn trader_terminal_operation_entry_stale_read_model_blocks_entry() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "operation-entry-stale-read-model",
        |artifact| {
            artifact["freshness"]["status"] = json!("stale");
            artifact["components"]["operation_entry"]["data"]["blocked_states"]["stale_read_model"] =
                json!(true);
        },
    );

    assert_eq!(runtime.health, HealthStatus::Stale);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("stale_artifact")
    );
    assert_eq!(
        runtime.operation_entry_status.value.as_deref(),
        Some("blocked_stale_read_model")
    );
    assert_eq!(
        runtime.operation_stale_read_model.value.as_deref(),
        Some("true")
    );
    assert!(
        runtime
            .operation_entry_blocked_reason
            .value
            .as_deref()
            .is_some_and(|reason| reason.contains("stale_read_model"))
    );
    assert_eq!(runtime.manual_operation_submit_allowed.value, Some(false));
}

#[test]
fn trader_terminal_operation_entry_provenance_mismatch_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "operation-entry-provenance-mismatch",
        |artifact| {
            artifact["health_status"] = json!("fail_closed");
            artifact["components"]["lifecycle_status"]["data"]["provenance_mismatch"] = json!(true);
            artifact["components"]["operation_entry"]["data"]["blocked_states"]["provenance_mismatch"] =
                json!(true);
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.operation_entry_status.value.as_deref(),
        Some("fail_closed_provenance_mismatch")
    );
    assert_eq!(
        runtime.operation_provenance_mismatch.value.as_deref(),
        Some("true")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("operation_entry:provenance_mismatch"))
    );
}

#[test]
fn trader_terminal_ungated_operation_attempt_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "operation-entry-ungated-attempt",
        |artifact| {
            artifact["health_status"] = json!("fail_closed");
            let entry = &mut artifact["components"]["operation_entry"]["data"];
            entry["ungated_operation_attempted"] = json!(true);
            entry["ungated_operation_attempt_fail_closed"] = json!(true);
            entry["attempt_status"] = json!("fail_closed_ungated_operation_attempt");
            artifact["capability_boundary"]["manual_operation_submit_allowed"] = json!(true);
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.operation_entry_status.value.as_deref(),
        Some("fail_closed_ungated_operation_attempt")
    );
    assert_eq!(
        runtime.operation_ungated_attempted.value.as_deref(),
        Some("true")
    );
    assert_eq!(
        runtime
            .operation_ungated_attempt_fail_closed
            .value
            .as_deref(),
        Some("true")
    );
    assert_eq!(runtime.manual_operation_submit_allowed.value, Some(true));
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| {
                diagnostic.contains("operation_entry:ungated_operation_attempted")
                    && diagnostic.contains("manual_operation_submit_allowed_true")
            })
    );
}

#[test]
fn trader_terminal_v220_runtime_degradation_cases_disable_operation_controls() {
    let cases = vec![
        (
            "missing-artifact",
            trader_terminal_read_model_runtime_without_artifact("v220-missing-artifact"),
            HealthStatus::Degraded,
            "missing_artifact",
            "canonical_unified_read_model_artifact_missing",
        ),
        (
            "schema-mismatch",
            trader_terminal_read_model_runtime_with_mutation("v220-schema-mismatch", |artifact| {
                artifact["schema_version"] = json!("ntpro.v220.trader_terminal.schema.unexpected");
            }),
            HealthStatus::Error,
            "schema_mismatch",
            "schema_version_mismatch",
        ),
        (
            "component-unavailable",
            trader_terminal_read_model_runtime_with_mutation(
                "v220-component-unavailable",
                |artifact| {
                    artifact["components"]["orders"]["component_status"] = json!("unavailable");
                },
            ),
            HealthStatus::Degraded,
            "component_unavailable",
            "orders:unavailable",
        ),
        (
            "stale-source",
            trader_terminal_read_model_runtime_with_mutation("v220-stale-source", |artifact| {
                artifact["freshness"]["status"] = json!("stale");
                artifact["components"]["operation_entry"]["data"]["blocked_states"]["stale_read_model"] =
                    json!(true);
            }),
            HealthStatus::Stale,
            "stale_artifact",
            "snapshot_freshness_stale",
        ),
        (
            "redaction-breach",
            trader_terminal_read_model_runtime_with_mutation("v220-redaction-breach", |artifact| {
                artifact["components"]["risk"]["data"]["alerts"]["redaction_breach"] = json!(true);
            }),
            HealthStatus::Error,
            "fail_closed",
            "risk:alert:redaction_breach",
        ),
        (
            "provenance-mismatch",
            trader_terminal_read_model_runtime_with_mutation(
                "v220-provenance-mismatch",
                |artifact| {
                    artifact["components"]["lifecycle_status"]["data"]["provenance_mismatch"] =
                        json!(true);
                    artifact["components"]["operation_entry"]["data"]["blocked_states"]["provenance_mismatch"] =
                        json!(true);
                },
            ),
            HealthStatus::Error,
            "fail_closed",
            "lifecycle_status:provenance_mismatch",
        ),
    ];

    for (name, runtime, expected_health, expected_readiness, expected_diagnostic) in cases {
        assert_eq!(runtime.health, expected_health, "{name}");
        assert_ne!(runtime.health, HealthStatus::Healthy, "{name}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some(expected_readiness),
            "{name}"
        );
        assert!(
            runtime
                .diagnostic
                .value
                .as_deref()
                .is_some_and(|diagnostic| diagnostic.contains(expected_diagnostic)),
            "{name}"
        );
        assert_v220_operation_controls_disabled(&runtime, name);
    }
}

#[test]
fn trader_terminal_v221_required_false_boundaries_accept_explicit_false() {
    let runtime =
        trader_terminal_read_model_runtime_with_mutation("v221-required-false-explicit", |_| {});

    assert_eq!(runtime.health, HealthStatus::Healthy);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("ready_readonly_artifact")
    );
    assert_eq!(
        runtime.diagnostic.value.as_deref(),
        Some("canonical_unified_read_model_artifact_ready")
    );

    for field in v220_required_false_operation_boundary_fields() {
        assert_eq!(v220_boundary_value(&runtime, field), Some(false), "{field}");
    }
}

#[test]
fn trader_terminal_v221_workbench_snapshot_populates_render_smoke_fields() {
    let root = temp_root("v221-workbench-render-smoke");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-v221-render-smoke");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |_| {});
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-02T16:00:00Z").unwrap();

    assert_eq!(snapshot.read_model_runtime.len(), 1);
    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.node_id, "terminal-v221-render-smoke");
    assert_eq!(runtime.health, HealthStatus::Healthy);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("ready_readonly_artifact")
    );
    assert_eq!(runtime.account_status.value.as_deref(), Some("healthy"));
    assert_eq!(
        runtime.positions_net_position_side.value.as_deref(),
        Some("flat")
    );
    assert_eq!(
        runtime.orders_lifecycle_state.value.as_deref(),
        Some("readback_matched")
    );
    assert_eq!(
        runtime.fills_fill_status.value.as_deref(),
        Some("reconciled")
    );
    assert_eq!(runtime.risk_state.value.as_deref(), Some("active"));
    assert_eq!(runtime.risk_alert_severity.value.as_deref(), Some("info"));
    assert_eq!(
        runtime.audit_release_provenance.value.as_deref(),
        Some("ntpro-rust-only-v0.21.1")
    );
    assert_eq!(
        runtime.operation_entry_status.value.as_deref(),
        Some("blocked_missing_owner_approval")
    );
    assert_eq!(
        runtime.operation_entry_blocked_reason.value.as_deref(),
        Some("missing_owner_approval,missing_risk_gate,missing_audit_gate")
    );

    for field in v220_required_false_operation_boundary_fields() {
        assert_eq!(v220_boundary_value(runtime, field), Some(false), "{field}");
    }

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    let runtime_value = &snapshot_value["read_model_runtime"][0];
    for field in [
        "account_status",
        "positions_net_position_side",
        "orders_lifecycle_state",
        "fills_fill_status",
        "risk_alert_severity",
        "audit_release_provenance",
        "operation_entry_status",
        "manual_operation_submit_allowed",
        "manual_operation_cancel_allowed",
        "manual_operation_replace_allowed",
        "manual_operation_amend_allowed",
        "manual_operation_flatten_allowed",
    ] {
        assert!(
            runtime_value.get(field).is_some(),
            "render smoke snapshot must expose {field}"
        );
    }
    assert_forbidden_keys_absent(&snapshot_value);
}

#[test]
fn trader_terminal_v221_missing_required_false_boundaries_fail_closed() {
    for field in v220_required_false_operation_boundary_fields() {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("v221-required-false-missing-{field}"),
            |artifact| {
                artifact["capability_boundary"]
                    .as_object_mut()
                    .unwrap()
                    .remove(*field);
            },
        );

        assert_eq!(runtime.health, HealthStatus::Error, "{field}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some("fail_closed"),
            "{field}"
        );
        assert_eq!(v220_boundary_value(&runtime, field), None, "{field}");
        assert!(
            runtime
                .diagnostic
                .value
                .as_deref()
                .is_some_and(|diagnostic| diagnostic.contains(&format!("{field}_missing"))),
            "{field}"
        );
    }
}

#[test]
fn trader_terminal_v220_forbidden_controls_fail_closed_individually() {
    for field in v220_required_false_operation_boundary_fields() {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("v220-forbidden-{field}"),
            |artifact| {
                artifact["capability_boundary"][*field] = json!(true);
            },
        );

        assert_eq!(runtime.health, HealthStatus::Error, "{field}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some("fail_closed"),
            "{field}"
        );
        assert_eq!(v220_boundary_value(&runtime, field), Some(true), "{field}");
        assert!(
            runtime
                .diagnostic
                .value
                .as_deref()
                .is_some_and(|diagnostic| diagnostic.contains(&format!("{field}_true"))),
            "{field}"
        );
    }
}

#[test]
fn trader_terminal_v220_display_claim_boundary_stays_read_only_first() {
    let scope_doc = include_str!(
        "../../../../docs/rust-cutover/scope/v0_22_0_trader_terminal_workbench_scope.md"
    );
    assert!(scope_doc.contains("read_only_first = required"));
    assert!(scope_doc.contains("product_grade_trading_terminal_claim = forbidden"));
    assert!(scope_doc.contains("read_only_first_boundary"));

    for marker in [
        "workbench-boundary",
        "foundation-boundary",
        "read-only-boundary",
        "gated-operation-boundary",
        "product_grade_trading_terminal_claim",
        "all_operation_controls_disabled",
        "disabled_gated_preview_only",
    ] {
        assert!(
            DASHBOARD_JS.contains(marker) || DASHBOARD_HTML.contains(marker),
            "V220 workbench display boundary marker missing: {marker}"
        );
    }

    for forbidden_claim in [
        "product-grade live trading terminal",
        "production trading terminal",
        "enable live trading",
    ] {
        assert!(
            !DASHBOARD_JS.contains(forbidden_claim),
            "workbench JS must not claim {forbidden_claim}"
        );
        assert!(
            !DASHBOARD_HTML.contains(forbidden_claim),
            "workbench shell must not claim {forbidden_claim}"
        );
    }

    for field in [
        "product_grade_trading_terminal_claim",
        "trader_terminal_live_trading_claim",
    ] {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("v220-display-{field}"),
            |artifact| {
                artifact["capability_boundary"][field] = json!(true);
            },
        );

        assert_eq!(runtime.health, HealthStatus::Error, "{field}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some("fail_closed"),
            "{field}"
        );
        assert!(
            runtime
                .diagnostic
                .value
                .as_deref()
                .is_some_and(|diagnostic| diagnostic.contains(&format!("{field}_true"))),
            "{field}"
        );
    }
}

#[test]
fn trader_terminal_account_position_component_stale_degrades_panel() {
    let root = temp_root("trader-terminal-account-position-stale");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-account-position-stale");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |artifact| {
        artifact["components"]["account"]["freshness"]["status"] = json!("stale");
    });
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:02:00Z").unwrap();

    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Stale);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("stale_artifact")
    );
    assert_eq!(
        runtime.account_freshness_status.value.as_deref(),
        Some("stale")
    );
    assert!(
        runtime
            .component_diagnostics
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("account:freshness_stale"))
    );
}

#[test]
fn trader_terminal_account_position_missing_provenance_fails_closed() {
    let root = temp_root("trader-terminal-account-position-provenance");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-account-position-provenance");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |artifact| {
        artifact["components"]["positions"]
            .as_object_mut()
            .unwrap()
            .remove("source_provenance");
    });
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:03:00Z").unwrap();

    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.positions_source_ref.availability,
        DashboardAvailability::Unknown
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("positions:source_provenance_missing"))
    );
}

#[test]
fn trader_terminal_account_position_mismatch_fails_closed() {
    let root = temp_root("trader-terminal-account-position-mismatch");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-account-position-mismatch");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |artifact| {
        artifact["components"]["account"]["data"]["account_id"] = json!("acct-a");
        artifact["components"]["positions"]["data"]["account_id"] = json!("acct-b");
    });
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:04:00Z").unwrap();

    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.positions_account_id.value.as_deref(),
        Some("acct-b")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("account_position_mismatch"))
    );
}

#[test]
fn trader_terminal_order_lifecycle_panel_covers_fail_closed_cases() {
    for (name, lifecycle, readback, diagnostic, ledger_present, duplicate_attempt) in [
        (
            "unknown-response",
            "unknown_response",
            "unavailable",
            "unknown_order_response_no_retry",
            true,
            false,
        ),
        (
            "readback-mismatch",
            "readback_mismatch",
            "mismatch",
            "order_readback_mismatch",
            true,
            false,
        ),
        (
            "duplicate-attempt",
            "duplicate_attempt",
            "not_attempted",
            "duplicate_submit_attempt",
            true,
            true,
        ),
        (
            "missing-ledger",
            "missing_ledger",
            "not_attempted",
            "missing_attempt_ledger",
            false,
            false,
        ),
    ] {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("order-{name}"),
            |artifact| {
                artifact["health_status"] = json!("fail_closed");
                artifact["blocking_reasons"] = json!([diagnostic]);
                let orders = &mut artifact["components"]["orders"];
                orders["component_status"] = json!("fail_closed");
                orders["data"]["lifecycle_status"] = json!(lifecycle);
                orders["data"]["readback_status"] = json!(readback);
                orders["data"]["ledger_present"] = json!(ledger_present);
                orders["data"]["duplicate_attempt_detected"] = json!(duplicate_attempt);
                orders["data"]["audit_state"] = json!("audit_risk_visible");
                orders["diagnostics"] = json!([diagnostic]);
            },
        );

        assert_eq!(runtime.health, HealthStatus::Error, "{name}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some("fail_closed"),
            "{name}"
        );
        assert_eq!(
            runtime.orders_lifecycle_state.value.as_deref(),
            Some(lifecycle),
            "{name}"
        );
        assert_eq!(
            runtime.orders_readback_status.value.as_deref(),
            Some(readback),
            "{name}"
        );
        assert_eq!(
            runtime.orders_ledger_present.value.as_deref(),
            Some(if ledger_present { "true" } else { "false" }),
            "{name}"
        );
        assert_eq!(
            runtime.orders_duplicate_attempt_detected.value.as_deref(),
            Some(if duplicate_attempt { "true" } else { "false" }),
            "{name}"
        );
        assert!(
            runtime
                .orders_diagnostics
                .value
                .as_deref()
                .is_some_and(|value| value.contains(diagnostic)),
            "{name}"
        );
        assert!(
            runtime
                .diagnostic
                .value
                .as_deref()
                .is_some_and(|value| value.contains(diagnostic)),
            "{name}"
        );
        assert_eq!(
            runtime.production_order_submission_allowed.value,
            Some(false)
        );
        assert_eq!(runtime.dashboard_order_controls_enabled.value, Some(false));
        assert_eq!(runtime.retry_order_allowed.value, Some(false));
        assert_eq!(runtime.automatic_cancel_allowed.value, Some(false));
        assert_eq!(
            runtime.automatic_order_remediation_allowed.value,
            Some(false)
        );
    }
}

#[test]
fn trader_terminal_fill_panel_covers_partial_duplicate_and_missing_linkage() {
    for (
        name,
        component_status,
        expected_health,
        readiness,
        fill_status,
        reconciliation_status,
        linkage_status,
        diagnostic,
        partial_fill,
        duplicate_fill,
    ) in [
        (
            "partial-fill",
            "degraded",
            HealthStatus::Degraded,
            "degraded_artifact",
            "partial",
            "partial_fill_visible",
            "linked",
            "partial_fill_visible_readonly",
            true,
            false,
        ),
        (
            "duplicate-fill",
            "fail_closed",
            HealthStatus::Error,
            "fail_closed",
            "duplicate",
            "duplicate_rejected",
            "linked",
            "duplicate_fill",
            false,
            true,
        ),
        (
            "missing-linkage",
            "fail_closed",
            HealthStatus::Error,
            "fail_closed",
            "missing_linkage",
            "missing_order_linkage",
            "missing",
            "missing_order_linkage",
            false,
            false,
        ),
    ] {
        let runtime =
            trader_terminal_read_model_runtime_with_mutation(&format!("fill-{name}"), |artifact| {
                artifact["health_status"] = json!(if expected_health == HealthStatus::Error {
                    "fail_closed"
                } else {
                    "degraded"
                });
                artifact["blocking_reasons"] = if expected_health == HealthStatus::Error {
                    json!([diagnostic])
                } else {
                    json!([])
                };
                let fills = &mut artifact["components"]["fills"];
                fills["component_status"] = json!(component_status);
                fills["data"]["fill_status"] = json!(fill_status);
                fills["data"]["reconciliation_status"] = json!(reconciliation_status);
                fills["data"]["order_linkage_status"] = json!(linkage_status);
                fills["data"]["partial_fill_detected"] = json!(partial_fill);
                fills["data"]["duplicate_fill_detected"] = json!(duplicate_fill);
                fills["data"]["remaining_quantity"] = if partial_fill {
                    json!("0.005")
                } else {
                    json!("0")
                };
                fills["data"]["risk_projection_input"]["fill_reconciliation_status"] =
                    json!(reconciliation_status);
                fills["data"]["risk_projection_input"]["remaining_order_quantity"] =
                    fills["data"]["remaining_quantity"].clone();
                fills["data"]["risk_projection_input"]["risk_state"] =
                    if expected_health == HealthStatus::Error {
                        json!("risk_blocked")
                    } else {
                        json!("active")
                    };
                fills["data"]["risk_projection_input"]["blocking_reasons"] =
                    if expected_health == HealthStatus::Error {
                        json!([diagnostic])
                    } else {
                        json!([])
                    };
                fills["diagnostics"] = json!([diagnostic]);
            });

        assert_eq!(runtime.health, expected_health, "{name}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some(readiness),
            "{name}"
        );
        assert_eq!(
            runtime.fills_reconciliation_status.value.as_deref(),
            Some(reconciliation_status),
            "{name}"
        );
        assert_eq!(
            runtime.fills_order_linkage_status.value.as_deref(),
            Some(linkage_status),
            "{name}"
        );
        assert_eq!(
            runtime.fills_partial_fill_detected.value.as_deref(),
            Some(if partial_fill { "true" } else { "false" }),
            "{name}"
        );
        assert_eq!(
            runtime.fills_duplicate_fill_detected.value.as_deref(),
            Some(if duplicate_fill { "true" } else { "false" }),
            "{name}"
        );
        assert!(
            runtime
                .fills_risk_projection_input
                .value
                .as_deref()
                .is_some_and(|value| value.contains(reconciliation_status)),
            "{name}"
        );
        assert!(
            runtime
                .fills_diagnostics
                .value
                .as_deref()
                .is_some_and(|value| value.contains(diagnostic)),
            "{name}"
        );
        assert_eq!(runtime.dashboard_fill_controls_enabled.value, Some(false));
        assert_eq!(runtime.execution_algorithm_allowed.value, Some(false));
        assert_eq!(runtime.automatic_fill_repair_allowed.value, Some(false));
        assert_eq!(
            runtime.automatic_reconciliation_repair_allowed.value,
            Some(false)
        );
    }
}

#[test]
fn trader_terminal_order_fill_schema_only_truth_claim_fails_closed() {
    for component in ["orders", "fills"] {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("{component}-exchange-truth-claim"),
            |artifact| {
                artifact["components"][component]["source_provenance"]["exchange_truth"] =
                    json!(true);
                artifact["components"][component]["source_provenance"]["adapter_runtime_integrated"] =
                    json!(true);
                artifact["components"][component]["data"]["values_are_exchange_truth"] =
                    json!(true);
            },
        );

        assert_eq!(runtime.health, HealthStatus::Error, "{component}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some("fail_closed"),
            "{component}"
        );
        assert!(
            runtime.diagnostic.value.as_deref().is_some_and(
                |value| value.contains(&format!("{component}:runtime_exchange_truth_claimed"))
            ),
            "{component}"
        );
    }
}

#[test]
fn trader_terminal_risk_panel_prioritizes_state() {
    for (
        name,
        risk_visible,
        manual_review,
        halted,
        stale,
        mismatch,
        component_status,
        snapshot_health,
        expected_priority,
        expected_health,
        expected_readiness,
    ) in [
        (
            "healthy",
            false,
            false,
            false,
            false,
            false,
            "healthy",
            "healthy",
            "healthy",
            HealthStatus::Healthy,
            "ready_readonly_artifact",
        ),
        (
            "risk-visible",
            true,
            false,
            false,
            false,
            false,
            "healthy",
            "healthy",
            "risk_visible",
            HealthStatus::Healthy,
            "ready_readonly_artifact",
        ),
        (
            "manual-review",
            true,
            true,
            false,
            false,
            false,
            "degraded",
            "degraded",
            "manual_review",
            HealthStatus::Degraded,
            "degraded_artifact",
        ),
        (
            "stale",
            true,
            true,
            false,
            true,
            false,
            "healthy",
            "healthy",
            "stale",
            HealthStatus::Stale,
            "stale_artifact",
        ),
        (
            "mismatch",
            true,
            true,
            false,
            false,
            true,
            "healthy",
            "healthy",
            "mismatch",
            HealthStatus::Error,
            "fail_closed",
        ),
        (
            "halted",
            true,
            true,
            true,
            false,
            false,
            "degraded",
            "degraded",
            "halted",
            HealthStatus::Degraded,
            "degraded_artifact",
        ),
    ] {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("risk-priority-{name}"),
            |artifact| {
                artifact["health_status"] = json!(snapshot_health);
                let risk = &mut artifact["components"]["risk"];
                risk["component_status"] = json!(component_status);
                risk["freshness"]["status"] = if stale {
                    json!("stale")
                } else {
                    json!("fresh")
                };
                risk["data"]["risk_visible"] = json!(risk_visible);
                risk["data"]["manual_review_required"] = json!(manual_review);
                risk["data"]["halted"] = json!(halted);
                risk["data"]["mismatch_detected"] = json!(mismatch);
                risk["data"]["freshness_rollup"] = if stale {
                    json!("stale")
                } else {
                    json!("fresh")
                };
            },
        );

        assert_eq!(runtime.health, expected_health, "{name}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some(expected_readiness),
            "{name}"
        );
        assert_eq!(
            runtime.risk_priority_state.value.as_deref(),
            Some(expected_priority),
            "{name}"
        );
        assert_eq!(
            runtime.risk_manual_review_required.value.as_deref(),
            Some(if manual_review { "true" } else { "false" }),
            "{name}"
        );
        assert_eq!(
            runtime.risk_halted.value.as_deref(),
            Some(if halted { "true" } else { "false" }),
            "{name}"
        );
        assert_eq!(runtime.dashboard_risk_controls_enabled.value, Some(false));
        assert_eq!(runtime.automatic_risk_action_allowed.value, Some(false));
        assert_eq!(runtime.automatic_risk_repair_allowed.value, Some(false));
    }
}

#[test]
fn trader_terminal_alert_panel_prioritizes_severity() {
    for (name, field, expected_severity, expected_health, expected_readiness) in [
        (
            "stale-source",
            "stale_source",
            "warning",
            HealthStatus::Stale,
            "stale_artifact",
        ),
        (
            "missing-evidence",
            "missing_evidence",
            "critical",
            HealthStatus::Error,
            "fail_closed",
        ),
        (
            "schema-mismatch",
            "schema_mismatch",
            "critical",
            HealthStatus::Error,
            "fail_closed",
        ),
        (
            "redaction-breach",
            "redaction_breach",
            "critical",
            HealthStatus::Error,
            "fail_closed",
        ),
        (
            "forbidden-control",
            "forbidden_control_request",
            "critical",
            HealthStatus::Error,
            "fail_closed",
        ),
    ] {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("risk-alert-{name}"),
            |artifact| {
                let risk = &mut artifact["components"]["risk"];
                risk["data"]["alerts"][field] = json!(true);
                if field == "stale_source" {
                    risk["freshness"]["status"] = json!("stale");
                    risk["data"]["freshness_rollup"] = json!("stale");
                }
            },
        );

        assert_eq!(runtime.health, expected_health, "{name}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some(expected_readiness),
            "{name}"
        );
        assert_eq!(
            runtime.risk_alert_severity.value.as_deref(),
            Some(expected_severity),
            "{name}"
        );
        assert!(
            runtime
                .risk_alert_summary
                .value
                .as_deref()
                .is_some_and(|summary| summary.contains(field)),
            "{name}"
        );
        assert!(
            runtime
                .diagnostic
                .value
                .as_deref()
                .is_some_and(|diagnostic| diagnostic.contains(field)),
            "{name}"
        );
        assert_eq!(runtime.automatic_alert_action_allowed.value, Some(false));
    }
}

#[test]
fn trader_terminal_audit_closed_requires_complete_evidence() {
    let runtime =
        trader_terminal_read_model_runtime_with_mutation("audit-evidence-missing", |artifact| {
            let lifecycle = &mut artifact["components"]["lifecycle_status"];
            lifecycle["data"]["audit_closed"] = json!(true);
            lifecycle["data"]["required_evidence_complete"] = json!(false);
            lifecycle["data"]["missing_evidence"] =
                json!(["risk_required_evidence", "release_provenance_digest"]);
        });

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(runtime.audit_closed.value.as_deref(), Some("true"));
    assert_eq!(
        runtime.audit_required_evidence_complete.value.as_deref(),
        Some("false")
    );
    assert!(
        runtime
            .audit_missing_evidence
            .value
            .as_deref()
            .is_some_and(|missing| missing.contains("release_provenance_digest"))
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic
                .contains("lifecycle_status:audit_closed_without_complete_evidence"))
    );
    assert_eq!(runtime.automatic_audit_action_allowed.value, Some(false));
}

#[test]
fn trader_terminal_provenance_mismatch_fails_closed() {
    let runtime =
        trader_terminal_read_model_runtime_with_mutation("audit-provenance-mismatch", |artifact| {
            let lifecycle = &mut artifact["components"]["lifecycle_status"];
            lifecycle["data"]["release_provenance"] = json!("unexpected-release-tag");
            lifecycle["data"]["artifact_digest"] = json!("sha256:mismatched");
            lifecycle["data"]["provenance_mismatch"] = json!(true);
        });

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.audit_release_provenance.value.as_deref(),
        Some("unexpected-release-tag")
    );
    assert_eq!(
        runtime.audit_artifact_digest.value.as_deref(),
        Some("sha256:mismatched")
    );
    assert_eq!(
        runtime.audit_provenance_mismatch.value.as_deref(),
        Some("true")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("lifecycle_status:provenance_mismatch"))
    );
    assert_eq!(
        runtime.automatic_provenance_repair_allowed.value,
        Some(false)
    );
}

#[test]
fn trader_terminal_risk_alert_audit_action_flags_fail_closed_when_true() {
    for field in [
        "dashboard_risk_controls_enabled",
        "automatic_risk_action_allowed",
        "automatic_risk_repair_allowed",
        "automatic_alert_action_allowed",
        "automatic_audit_action_allowed",
        "automatic_provenance_repair_allowed",
    ] {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("automatic-action-{field}"),
            |artifact| {
                artifact["capability_boundary"][field] = json!(true);
            },
        );

        assert_eq!(runtime.health, HealthStatus::Error, "{field}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some("fail_closed"),
            "{field}"
        );
        assert!(
            runtime
                .diagnostic
                .value
                .as_deref()
                .is_some_and(|diagnostic| diagnostic.contains(&format!("{field}_true"))),
            "{field}"
        );
    }
}

#[test]
fn dashboard_v24_order_control_preview_renderer_stays_readonly() {
    let workbench_renderer = dashboard_js_function_body("renderTraderTerminalWorkbench");
    let runtime_renderer = dashboard_js_function_body("renderReadModelRuntime");
    let renderer = format!("{workbench_renderer}\n{runtime_renderer}");

    for required in [
        "v24 Order-control preview",
        "v24_order_control_preview_status",
        "v24_missing_preview_evidence",
        "v24_forbidden_control_detected",
        "Submit control",
        "Cancel control",
        "Replace control",
        "Amend control",
        "Flatten control",
        "Order ticket",
    ] {
        assert!(
            renderer.contains(required),
            "v24 dashboard renderer must contain {required}"
        );
    }

    for forbidden in [
        "<button",
        "<form",
        "<input",
        "fetch(",
        "data-dashboard-action",
        "data-workbench-action",
        "/api/control",
        "/api/order",
        "/api/orders",
        "/actions/submit",
        "/actions/cancel",
        "/actions/replace",
        "/actions/amend",
        "/actions/flatten",
        "submit_order",
        "cancel_order",
        "replace_order",
        "amend_order",
        "flatten_position_action",
    ] {
        assert!(
            !renderer.contains(forbidden),
            "v24 dashboard renderer must stay read-only and not contain {forbidden}"
        );
    }
}

#[test]
fn dashboard_v25_monitoring_surface_renderer_stays_readonly() {
    let workbench_renderer = dashboard_js_function_body("renderTraderTerminalWorkbench");
    let runtime_renderer = dashboard_js_function_body("renderReadModelRuntime");
    let renderer = format!("{workbench_renderer}\n{runtime_renderer}");

    for required in [
        "v25 Monitoring / Incident / DR",
        "v25_dashboard_surface_status",
        "v25_diagnostics_gate_status",
        "v25_freshness_threshold_status",
        "v25_source_truth_status",
        "v25_release_provenance_status",
        "v25_no_remediation_status",
        "Diagnostics gate",
        "Freshness threshold",
        "Source truth",
        "No-action boundary",
        "v25_monitoring_effective_status",
        "v25_incident_ack_status",
        "v25_runbook_evidence_ref",
        "v25_dr_operator_approval_status",
        "v25_surface_blocking_reasons",
        "Submit control",
        "Cancel control",
        "Retry control",
        "Replace control",
        "Amend control",
        "Flatten control",
        "Order ticket",
    ] {
        assert!(
            renderer.contains(required),
            "v25 dashboard renderer must contain {required}"
        );
    }

    for forbidden in [
        "<button",
        "<form",
        "<input",
        "fetch(",
        "data-workbench-action",
        "/api/order",
        "/api/orders",
        "/actions/submit",
        "/actions/cancel",
        "/actions/retry",
        "/actions/replace",
        "/actions/amend",
        "/actions/flatten",
        "submit_order",
        "cancel_order",
        "replace_order",
        "amend_order",
        "flatten_position_action",
    ] {
        assert!(
            !renderer.contains(forbidden),
            "v25 dashboard renderer must stay read-only and not contain {forbidden}"
        );
    }
}

#[test]
fn dashboard_v26_admin_surface_renderer_stays_readonly() {
    let workbench_renderer = dashboard_js_function_body("renderTraderTerminalWorkbench");
    let runtime_renderer = dashboard_js_function_body("renderReadModelRuntime");
    let renderer = format!("{workbench_renderer}\n{runtime_renderer}");

    for required in [
        "v26 Product hardening admin",
        "v26_dashboard_admin_surface_status",
        "v26_permission_boundary_status",
        "v26_permission_roles_checked",
        "v26_operation_audit_status",
        "v26_operation_audit_lineage",
        "v26_deployment_provenance_status",
        "v26_deployment_environment",
        "v26_upgrade_rollback_status",
        "v26_upgrade_rollback_preview",
        "v26_stability_status",
        "v26_stability_degradation_reason",
        "v26_admin_surface_blocking_reasons",
        "Admin surface status",
        "Permission boundary",
        "Deployment provenance",
        "Stability / SLO",
        "Submit control",
        "Cancel control",
        "Retry control",
        "Replace control",
        "Amend control",
        "Flatten control",
        "Order ticket",
    ] {
        assert!(
            renderer.contains(required),
            "v26 dashboard renderer must contain {required}"
        );
    }

    for forbidden in [
        "<button",
        "<form",
        "<input",
        "fetch(",
        "data-workbench-action",
        "/api/order",
        "/api/orders",
        "/actions/submit",
        "/actions/cancel",
        "/actions/retry",
        "/actions/replace",
        "/actions/amend",
        "/actions/flatten",
        "submit_order",
        "cancel_order",
        "replace_order",
        "amend_order",
        "flatten_position_action",
    ] {
        assert!(
            !renderer.contains(forbidden),
            "v26 dashboard renderer must stay read-only and not contain {forbidden}"
        );
    }
}

#[test]
fn dashboard_v25_monitoring_surface_missing_component_degrades() {
    let runtime =
        trader_terminal_read_model_runtime_with_mutation("v25-surface-missing-dr", |artifact| {
            artifact["components"]
                .as_object_mut()
                .unwrap()
                .remove(V25_DR_PREVIEW_COMPONENT);
        });

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("component_missing")
    );
    assert_eq!(
        runtime.v25_dashboard_surface_status.value.as_deref(),
        Some("degraded_missing_surface_artifact")
    );
    assert!(
        runtime
            .missing_components
            .value
            .as_deref()
            .is_some_and(|missing| missing.contains(V25_DR_PREVIEW_COMPONENT))
    );
    assert_v220_operation_controls_disabled(&runtime, "v25-surface-missing-dr");
}

#[test]
fn dashboard_v25_monitoring_surface_missing_provenance_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v25-surface-missing-provenance",
        |artifact| {
            artifact["components"][V25_MONITORING_OBSERVABILITY_COMPONENT]["source_provenance"] =
                json!({});
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.v25_dashboard_surface_status.value.as_deref(),
        Some("fail_closed_surface_artifact")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic
                .contains("v25_monitoring_observability:source_provenance_missing"))
    );
    assert_v220_operation_controls_disabled(&runtime, "v25-surface-missing-provenance");
}

#[test]
fn dashboard_v25_monitoring_surface_forbidden_control_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v25-surface-forbidden-control",
        |artifact| {
            artifact["components"][V25_INCIDENT_LIFECYCLE_COMPONENT]["data"]["dashboard_trading_control_allowed"] =
                json!(true);
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.v25_dashboard_surface_status.value.as_deref(),
        Some("fail_closed_surface_artifact")
    );
    assert!(
        runtime
            .v25_surface_blocking_reasons
            .value
            .as_deref()
            .is_some_and(|reasons| reasons.contains(
                "v25_incident_lifecycle:boundary_true:dashboard_trading_control_allowed"
            ))
    );
    assert_eq!(runtime.dashboard_submit_controls_enabled.value, Some(false));
    assert_eq!(runtime.dashboard_cancel_controls_enabled.value, Some(false));
    assert_eq!(
        runtime.trader_terminal_order_ticket_enabled.value,
        Some(false)
    );
}

#[test]
fn dashboard_v26_admin_surface_missing_provenance_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v26-surface-missing-provenance",
        |artifact| {
            artifact["components"][V26_PERMISSION_BOUNDARY_COMPONENT]["source_provenance"] =
                json!({});
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.v26_dashboard_admin_surface_status.value.as_deref(),
        Some("fail_closed_surface_artifact")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic
                .contains("v26_permission_boundary:source_provenance_missing"))
    );
    assert_v220_operation_controls_disabled(&runtime, "v26-surface-missing-provenance");
}

#[test]
fn dashboard_v26_admin_surface_forbidden_control_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v26-surface-forbidden-control",
        |artifact| {
            artifact["components"][V26_STABILITY_SLO_COMPONENT]["data"]["automatic_remediation_allowed"] =
                json!(true);
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.v26_dashboard_admin_surface_status.value.as_deref(),
        Some("fail_closed_surface_artifact")
    );
    assert!(
        runtime
            .v26_admin_surface_blocking_reasons
            .value
            .as_deref()
            .is_some_and(|reasons| {
                reasons.contains("v26_stability_slo:boundary_true:automatic_remediation_allowed")
            })
    );
    assert_eq!(runtime.dashboard_submit_controls_enabled.value, Some(false));
    assert_eq!(runtime.dashboard_cancel_controls_enabled.value, Some(false));
    assert_eq!(
        runtime.trader_terminal_order_ticket_enabled.value,
        Some(false)
    );
}

#[test]
fn dashboard_v25_slo_freshness_threshold_stale_source_degrades() {
    let runtime =
        trader_terminal_read_model_runtime_with_mutation("v25-slo-stale-source", |artifact| {
            artifact["components"][V25_ALERT_TAXONOMY_COMPONENT]["freshness"] = json!({
                "status": "stale",
                "observed_age_ms": 120000,
                "max_age_ms": 60000,
                "as_of_unix_ns": "1782917999000000000",
                "checked_at_unix_ns": "1782918120000000000",
                "staleness_reason": "source_lag_exceeded"
            });
            artifact["components"][V25_ALERT_TAXONOMY_COMPONENT]["data"]["diagnostic_severity"] =
                json!("warning");
        });

    assert_eq!(runtime.health, HealthStatus::Stale);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("stale_artifact")
    );
    assert_eq!(
        runtime.v25_diagnostics_gate_status.value.as_deref(),
        Some("degraded_stale_source")
    );
    assert_eq!(
        runtime.v25_freshness_threshold_status.value.as_deref(),
        Some("degraded_threshold_exceeded")
    );
    assert!(
        runtime
            .v25_staleness_reasons
            .value
            .as_deref()
            .is_some_and(|reasons| reasons.contains("source_lag_exceeded"))
    );
    assert_eq!(
        runtime.v25_diagnostic_severity.value.as_deref(),
        Some("warning")
    );
    assert_v220_operation_controls_disabled(&runtime, "v25-slo-stale-source");
}

#[test]
fn dashboard_v25_partial_projection_degrades_without_healthy_claim() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v25-partial-projection",
        |artifact| {
            artifact["components"][V25_MONITORING_OBSERVABILITY_COMPONENT]["data"]["partial_projection"] =
                json!(true);
            artifact["components"][V25_MONITORING_OBSERVABILITY_COMPONENT]["data"]["diagnostic_severity"] =
                json!("warning");
        },
    );

    assert_eq!(runtime.health, HealthStatus::Degraded);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("degraded_artifact")
    );
    assert_eq!(
        runtime.v25_diagnostics_gate_status.value.as_deref(),
        Some("degraded_partial_projection")
    );
    assert_eq!(
        runtime.v25_dashboard_surface_status.value.as_deref(),
        Some("degraded_surface_artifact")
    );
    assert_v220_operation_controls_disabled(&runtime, "v25-partial-projection");
}

#[test]
fn dashboard_v25_unknown_adapter_truth_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v25-unknown-adapter-truth",
        |artifact| {
            artifact["components"][V25_MONITORING_OBSERVABILITY_COMPONENT]["data"]["adapter_truth_status"] =
                json!("unknown");
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        runtime.v25_diagnostics_gate_status.value.as_deref(),
        Some("fail_closed_unknown_source_truth")
    );
    assert_eq!(
        runtime.v25_source_truth_status.value.as_deref(),
        Some("fail_closed_unknown_source_truth")
    );
    assert_v220_operation_controls_disabled(&runtime, "v25-unknown-adapter-truth");
}

#[test]
fn dashboard_v25_release_provenance_drift_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v25-release-provenance-drift",
        |artifact| {
            artifact["components"][V25_RUNBOOK_AUDIT_COMPONENT]["data"]["release_provenance_status"] =
                json!("drift");
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.v25_diagnostics_gate_status.value.as_deref(),
        Some("fail_closed_release_provenance_drift")
    );
    assert_eq!(
        runtime.v25_release_provenance_status.value.as_deref(),
        Some("fail_closed_release_provenance_drift")
    );
    assert_v220_operation_controls_disabled(&runtime, "v25-release-provenance-drift");
}

#[test]
fn dashboard_v25_diagnostics_gate_forbidden_actions_fail_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v25-diagnostics-forbidden-action",
        |artifact| {
            artifact["components"][V25_DR_PREVIEW_COMPONENT]["data"]["remediation_action_allowed"] =
                json!(true);
            artifact["components"][V25_DR_PREVIEW_COMPONENT]["data"]["trading_action_allowed"] =
                json!(true);
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.v25_diagnostics_gate_status.value.as_deref(),
        Some("fail_closed_forbidden_action")
    );
    assert_eq!(
        runtime.v25_no_remediation_status.value.as_deref(),
        Some("fail_closed_forbidden_action")
    );
    assert_eq!(runtime.dashboard_submit_controls_enabled.value, Some(false));
    assert_eq!(runtime.dashboard_retry_controls_enabled.value, Some(false));
    assert_eq!(
        runtime.trader_terminal_order_ticket_enabled.value,
        Some(false)
    );
}

#[test]
fn dashboard_v24_order_control_preview_missing_evidence_degrades() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v24-preview-missing-evidence",
        |artifact| {
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT] = read_model_component(
                "degraded",
                &json!({
                    "preview_status": "degraded_unavailable",
                    "order_intent_status": "ready_preview",
                    "execution_policy_status": "ready_preview",
                    "rate_limit_status": "ready_preview",
                    "slicing_status": "ready_preview",
                    "cancel_replace_amend_status": "ready_preview",
                    "retry_policy_status": "ready_preview",
                    "readback_audit_status": "degraded_unavailable",
                    "blocked_reasons": ["missing_provenance"],
                    "scope_key": "acct-redacted-001|strategy:strategy-redacted-alpha|venue:venue-node-binance-a",
                    "source_provenance": "missing_provenance",
                    "redaction_state": "redacted",
                    "order_intent_ref": "tests/golden/v240_order_intent_execution_policy.jsonl#ready",
                    "policy_ref": "docs/rust-cutover/release/v0_24_0_order_intent_execution_policy.md",
                    "rate_limit_ref": "tests/golden/v240_rate_limit_throttle_gate.jsonl#accepted",
                    "slicing_ref": "tests/golden/v240_order_slicing_preview.jsonl#single_slice",
                    "cancel_replace_amend_ref": "tests/golden/v240_cancel_replace_amend_preview.jsonl#replace_preview",
                    "retry_policy_ref": "tests/golden/v240_retry_policy_ledger.jsonl#transport_retry_allowed",
                    "readback_ref": "tests/golden/v240_readback_audit_evidence.jsonl#degraded_unavailable",
                    "audit_ref": "tests/golden/v240_readback_audit_evidence.jsonl#audit_degraded",
                    "provenance_ref": "",
                    "dashboard_redacted_ref": "docs/rust-cutover/evidence/V240-008.md#dashboard-redacted-degraded",
                    "preview_evidence_present": false,
                    "missing_preview_evidence": ["provenance_ref"],
                    "forbidden_control_detected": false,
                    "render_smoke_case": "v240-dashboard-case-missing-provenance"
                }),
            );
        },
    );

    assert_eq!(runtime.health, HealthStatus::Degraded);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("degraded_artifact")
    );
    assert_eq!(
        runtime.v24_order_control_preview_status.value.as_deref(),
        Some("degraded_unavailable")
    );
    assert_eq!(
        runtime.v24_preview_evidence_present.value.as_deref(),
        Some("false")
    );
    assert_eq!(
        runtime.v24_missing_preview_evidence.value.as_deref(),
        Some("provenance_ref")
    );
    assert_eq!(
        runtime.v24_forbidden_control_detected.value.as_deref(),
        Some("false")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| {
                diagnostic.contains("v24_order_control_preview:degraded_unavailable")
                    && diagnostic.contains("v24_order_control_preview:preview_evidence_missing")
            })
    );
    assert_v220_operation_controls_disabled(&runtime, "v24-preview-missing-evidence");
}

#[test]
fn dashboard_v24_order_control_preview_forbidden_true_controls_fail_closed_from_artifact() {
    for field in [
        "dashboard_submit_controls_enabled",
        "trader_terminal_order_ticket_enabled",
        "manual_operation_submit_allowed",
    ] {
        let runtime = trader_terminal_read_model_runtime_with_mutation(
            &format!("v24-preview-forbidden-{field}"),
            |artifact| {
                artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT] =
                    read_model_component("healthy", &v24_order_control_preview_ready_data());
                artifact["capability_boundary"][field] = json!(true);
            },
        );

        assert_eq!(runtime.health, HealthStatus::Error, "{field}");
        assert_eq!(
            runtime.readiness_status.value.as_deref(),
            Some("fail_closed"),
            "{field}"
        );
        assert!(
            runtime
                .diagnostic
                .value
                .as_deref()
                .is_some_and(|diagnostic| diagnostic.contains(&format!("{field}_true"))),
            "{field}"
        );
    }
}

#[test]
fn dashboard_v24_order_control_preview_stale_component_is_not_ready() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v24-preview-stale-component",
        |artifact| {
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT] =
                read_model_component("healthy", &v24_order_control_preview_ready_data());
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT]["freshness"] =
                read_model_freshness("stale");
        },
    );

    assert_eq!(runtime.health, HealthStatus::Stale);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("stale_artifact")
    );
    assert!(
        runtime.diagnostic.value.as_deref().is_some_and(
            |diagnostic| diagnostic.contains("v24_order_control_preview:freshness_stale")
        )
    );
    assert_v220_operation_controls_disabled(&runtime, "v24-preview-stale-component");
}

#[test]
fn dashboard_v24_order_control_preview_malformed_provenance_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v24-preview-malformed-provenance",
        |artifact| {
            let mut data = v24_order_control_preview_ready_data();
            data["provenance_ref"] = json!("");
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT] =
                read_model_component("healthy", &data);
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic
                .contains("v24_order_control_preview:preview_provenance_missing"))
    );
    assert_v220_operation_controls_disabled(&runtime, "v24-preview-malformed-provenance");
}

#[test]
fn dashboard_v24_order_control_preview_source_provenance_fails_closed_when_missing() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v24-preview-missing-source-provenance",
        |artifact| {
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT] =
                read_model_component("healthy", &v24_order_control_preview_ready_data());
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT]["source_provenance"] =
                json!({});
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic
                .contains("v24_order_control_preview:source_provenance_missing"))
    );
    assert_v220_operation_controls_disabled(&runtime, "v24-preview-missing-source-provenance");
}

#[test]
fn dashboard_v24_order_control_preview_scope_mismatch_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v24-preview-scope-mismatch",
        |artifact| {
            let mut data = v24_order_control_preview_ready_data();
            data["scope_key"] = json!(
                "acct-redacted-other|strategy:strategy-redacted-alpha|venue:venue-node-binance-a"
            );
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT] =
                read_model_component("healthy", &data);
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert!(
        runtime.diagnostic.value.as_deref().is_some_and(
            |diagnostic| diagnostic.contains("v24_order_control_preview:scope_mismatch")
        )
    );
    assert_v220_operation_controls_disabled(&runtime, "v24-preview-scope-mismatch");
}

#[test]
fn dashboard_v24_order_control_preview_missing_redaction_fails_closed() {
    let runtime = trader_terminal_read_model_runtime_with_mutation(
        "v24-preview-missing-redaction",
        |artifact| {
            let mut data = v24_order_control_preview_ready_data();
            data["redaction_state"] = json!("unredacted");
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT] =
                read_model_component("healthy", &data);
            artifact["components"][V24_ORDER_CONTROL_PREVIEW_COMPONENT]["redaction"] = json!({});
        },
    );

    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| {
                diagnostic.contains("v24_order_control_preview:redaction_state_not_ready")
                    && diagnostic
                        .contains("v24_order_control_preview:data_redaction_state_not_ready")
            })
    );
    assert_v220_operation_controls_disabled(&runtime, "v24-preview-missing-redaction");
}

fn v24_order_control_preview_ready_data() -> Value {
    json!({
        "preview_status": "ready_preview",
        "order_intent_status": "ready_preview",
        "execution_policy_status": "ready_preview",
        "rate_limit_status": "ready_preview",
        "slicing_status": "ready_preview",
        "cancel_replace_amend_status": "ready_preview",
        "retry_policy_status": "ready_preview",
        "readback_audit_status": "ready_preview",
        "blocked_reasons": [],
        "scope_key": "acct-redacted-001|strategy:strategy-redacted-alpha|venue:venue-node-binance-a",
        "source_provenance": "tests/golden/v240_readback_audit_evidence.jsonl#ready_preview",
        "redaction_state": "redacted",
        "order_intent_ref": "tests/golden/v240_order_intent_execution_policy.jsonl#ready",
        "policy_ref": "docs/rust-cutover/release/v0_24_0_order_intent_execution_policy.md",
        "rate_limit_ref": "tests/golden/v240_rate_limit_throttle_gate.jsonl#accepted",
        "slicing_ref": "tests/golden/v240_order_slicing_preview.jsonl#single_slice",
        "cancel_replace_amend_ref": "tests/golden/v240_cancel_replace_amend_preview.jsonl#cancel_preview",
        "retry_policy_ref": "tests/golden/v240_retry_policy_ledger.jsonl#no_retry_terminal",
        "readback_ref": "tests/golden/v240_readback_audit_evidence.jsonl#readback",
        "audit_ref": "tests/golden/v240_readback_audit_evidence.jsonl#audit",
        "provenance_ref": "tests/golden/v240_readback_audit_evidence.jsonl#provenance",
        "dashboard_redacted_ref": "docs/rust-cutover/evidence/V241-005.md#dashboard-artifact-ingestion",
        "preview_evidence_present": true,
        "missing_preview_evidence": [],
        "forbidden_control_detected": false,
        "render_smoke_case": "v241-dashboard-artifact-ingestion-ready"
    })
}

#[test]
fn trader_terminal_read_model_missing_artifact_degrades_runtime_bridge() {
    let root = temp_root("trader-terminal-read-model-missing");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-missing");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:05:00Z").unwrap();

    assert_eq!(snapshot.read_model_runtime.len(), 1);
    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Degraded);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("missing_artifact")
    );
    assert_eq!(
        runtime.diagnostic.value.as_deref(),
        Some("canonical_unified_read_model_artifact_missing")
    );
    assert_eq!(runtime.dashboard_order_controls_enabled.value, Some(false));
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "read_model_runtime.terminal-missing"
            && gap.reason == DashboardAvailability::Unknown
    }));
}

#[test]
fn trader_terminal_read_model_stale_artifact_is_not_healthy() {
    let root = temp_root("trader-terminal-read-model-stale");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-stale");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |artifact| {
        artifact["freshness"]["status"] = json!("stale");
    });
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:10:00Z").unwrap();

    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Stale);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("stale_artifact")
    );
    assert_eq!(runtime.freshness_status.value.as_deref(), Some("stale"));
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("snapshot_freshness_stale"))
    );
}

#[test]
fn trader_terminal_read_model_schema_mismatch_fails_closed() {
    let root = temp_root("trader-terminal-read-model-schema-mismatch");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-schema");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |artifact| {
        artifact["schema_version"] = json!("ntpro.v210.unified_read_model.schema.wrong");
    });
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:15:00Z").unwrap();

    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("schema_mismatch")
    );
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("schema_version_mismatch"))
    );
}

#[test]
fn trader_terminal_read_model_component_unavailable_degrades_runtime_bridge() {
    let root = temp_root("trader-terminal-read-model-component-unavailable");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-unavailable");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |artifact| {
        artifact["components"]["fills"]["component_status"] = json!("unavailable");
        artifact["components"]["fills"]["freshness"]["status"] = json!("missing");
    });
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:20:00Z").unwrap();

    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Degraded);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("component_unavailable")
    );
    assert_eq!(runtime.fills_status.value.as_deref(), Some("unavailable"));
    assert!(
        runtime
            .component_diagnostics
            .value
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("fills:unavailable"))
    );
}

#[test]
fn trader_terminal_read_model_component_missing_fails_closed() {
    let root = temp_root("trader-terminal-read-model-component-missing");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "terminal-component-missing");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, |artifact| {
        artifact["components"]
            .as_object_mut()
            .unwrap()
            .remove("risk");
    });
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T18:25:00Z").unwrap();

    let runtime = &snapshot.read_model_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Error);
    assert_eq!(
        runtime.readiness_status.value.as_deref(),
        Some("component_missing")
    );
    assert!(
        runtime
            .missing_components
            .value
            .as_deref()
            .is_some_and(|missing| missing.contains("risk"))
    );
    assert_eq!(
        runtime.risk_status.availability,
        DashboardAvailability::Unknown
    );
}

#[test]
fn production_shadow_artifacts_populate_readonly_dashboard_snapshot() {
    let root = temp_root("production-shadow-readonly");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "prod-shadow-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_shadow_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-20T10:00:00Z").unwrap();

    assert_eq!(snapshot.production_shadow.len(), 1);
    let shadow = &snapshot.production_shadow[0];
    assert_eq!(shadow.node_id, "prod-shadow-a");
    assert_eq!(shadow.health, HealthStatus::Healthy);
    assert_eq!(
        shadow.diagnostic.value.as_deref(),
        Some("production_shadow_readonly_artifacts_ok")
    );
    assert_eq!(
        shadow.account_snapshot_status.value.as_deref(),
        Some("ready_offline_contract")
    );
    assert_eq!(
        shadow.account_snapshot_endpoint_class.value.as_deref(),
        Some("production_authenticated_read_only")
    );
    assert_eq!(
        shadow.shadow_intent_status.value.as_deref(),
        Some("blocked_by_v110_shadow_execution_boundary")
    );
    assert_eq!(shadow.shadow_intents_created.value, Some(1));
    assert_eq!(
        shadow.portfolio_snapshot_status.value.as_deref(),
        Some("production_readonly_shadow")
    );
    assert_eq!(
        shadow.portfolio_exposure_status.value.as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        shadow.portfolio_pnl_status.value.as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        shadow.lifecycle_status.value.as_deref(),
        Some("ShadowSubmitted")
    );
    assert_eq!(shadow.lifecycle_events_created.value, Some(1));
    assert_eq!(
        shadow.reconciliation_status.value.as_deref(),
        Some("warning")
    );
    assert_eq!(shadow.reconciliation_events_created.value, Some(1));
    assert_eq!(
        shadow.manifest_status.value.as_deref(),
        Some("production_shadow_manifest_ok")
    );
    assert_eq!(shadow.manifest_artifact_count.value, Some(5));
    assert_eq!(shadow.actual_submission_count.value, Some(0));
    assert_eq!(shadow.production_orders_submitted.value, Some(0));
    assert_eq!(shadow.production_order_mutations_attempted.value, Some(0));
    assert_eq!(shadow.dashboard_order_controls_enabled.value, Some(false));
    assert!(
        shadow
            .portfolio_snapshot_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_11/shadow_portfolio_snapshot.json"))
    );
    assert!(
        shadow
            .manifest_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_11/manifest.json"))
    );

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
    assert!(!DASHBOARD_JS.contains("submit_order"));
    assert!(!DASHBOARD_JS.contains("cancel_order"));
    assert!(!DASHBOARD_JS.contains("replace_order"));
    assert!(!DASHBOARD_JS.contains("amend_order"));
}

#[test]
fn production_shadow_v12_artifacts_populate_dashboard_readonly_panel() {
    let root = temp_root("production-shadow-v12-readonly");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "prod-shadow-v12");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_shadow_v12_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-21T05:40:00Z").unwrap();

    assert_eq!(snapshot.production_shadow.len(), 1);
    let shadow = &snapshot.production_shadow[0];
    assert_eq!(shadow.node_id, "prod-shadow-v12");
    assert_eq!(shadow.health, HealthStatus::Healthy);
    assert_eq!(shadow.artifact_version.value.as_deref(), Some("v0_12"));
    assert_eq!(
        shadow.public_read_status.value.as_deref(),
        Some("online_read_probe_ok")
    );
    assert_eq!(
        shadow.account_snapshot_status.value.as_deref(),
        Some("online_account_snapshot_ok")
    );
    assert_eq!(
        shadow.response_shape_status.value.as_deref(),
        Some("accepted")
    );
    assert_eq!(shadow.response_shape_validated.value, Some(true));
    assert_eq!(
        shadow.portfolio_snapshot_status.value.as_deref(),
        Some("ready_redacted_shadow_portfolio")
    );
    assert_eq!(
        shadow.portfolio_exposure_status.value.as_deref(),
        Some("derived_from_shadow_intents")
    );
    assert_eq!(
        shadow.shadow_strategy_session_status.value.as_deref(),
        Some("stopped")
    );
    assert_eq!(shadow.shadow_strategy_session_heartbeats.value, Some(2));
    assert_eq!(
        shadow.reconciliation_classification.value.as_deref(),
        Some("ok")
    );
    assert_eq!(
        shadow.reconciliation_recommended_action.value.as_deref(),
        Some("record_only")
    );
    assert_eq!(shadow.risk_halted.value, Some(true));
    assert_eq!(shadow.manual_review_required.value, Some(false));
    assert_eq!(shadow.new_orders_blocked.value, Some(true));
    assert_eq!(shadow.actual_submission_count.value, Some(0));
    assert_eq!(shadow.production_order_submissions_attempted.value, Some(0));
    assert_eq!(shadow.production_orders_submitted.value, Some(0));
    assert_eq!(shadow.production_order_mutations_attempted.value, Some(0));
    assert_eq!(shadow.production_order_state_reads_attempted.value, Some(0));
    assert_eq!(shadow.listen_key_lifecycle_attempted.value, Some(0));
    assert_eq!(shadow.automatic_correction_orders_submitted.value, Some(0));
    assert_eq!(shadow.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(shadow.real_orders_submitted.value, Some(false));
    assert_eq!(
        shadow.order_state_values_are_exchange_truth.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(shadow.shadow_values_are_exchange_truth.value, Some(false));
    assert_eq!(
        shadow.portfolio_values_are_exchange_truth.value,
        Some(false)
    );
    assert_eq!(shadow.values_are_exchange_truth.value, Some(false));
    assert!(
        shadow
            .portfolio_snapshot_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_12/shadow_portfolio_runtime.json"))
    );
    assert!(
        shadow
            .shadow_strategy_session_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_12/shadow_strategy_session.jsonl"))
    );
    assert!(
        shadow
            .reconciliation_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_12/reconciliation_events.jsonl"))
    );

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
    assert!(!DASHBOARD_JS.contains("submit_order"));
    assert!(!DASHBOARD_JS.contains("cancel_order"));
    assert!(!DASHBOARD_JS.contains("replace_order"));
    assert!(!DASHBOARD_JS.contains("amend_order"));
    assert!(!DASHBOARD_JS.contains("retry_order_action"));
    assert!(!DASHBOARD_JS.contains("data-workbench-action=\"retry_order\""));
    assert!(!DASHBOARD_JS.contains("/actions/retry_order"));
    assert!(!DASHBOARD_HTML.contains("credential"));
}

#[test]
fn production_shadow_v13_kill_switch_artifact_populates_dashboard_boundary_panel() {
    let root = temp_root("production-shadow-v13-kill-switch");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "prod-shadow-v13");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_shadow_v13_kill_switch_artifact(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-22T00:30:00Z").unwrap();

    assert_eq!(snapshot.production_shadow.len(), 1);
    let shadow = &snapshot.production_shadow[0];
    assert_eq!(shadow.node_id, "prod-shadow-v13");
    assert_eq!(shadow.health, HealthStatus::Healthy);
    assert_eq!(shadow.artifact_version.value.as_deref(), Some("v0_13"));
    assert_eq!(
        shadow.kill_switch_status.value.as_deref(),
        Some("manual_approval_recorded")
    );
    assert_eq!(shadow.kill_switch_active.value, Some(true));
    assert_eq!(shadow.kill_switch_dry_run.value, Some(true));
    assert_eq!(
        shadow.kill_switch_manual_approval_recorded.value,
        Some(true)
    );
    assert_eq!(
        shadow.kill_switch_approval_state.value.as_deref(),
        Some("approved")
    );
    assert_eq!(
        shadow.kill_switch_production_order_submission_allowed.value,
        Some(false)
    );
    assert_eq!(
        shadow.kill_switch_production_order_mutation_allowed.value,
        Some(false)
    );
    assert_eq!(
        shadow
            .kill_switch_production_order_state_reads_allowed
            .value,
        Some(false)
    );
    assert_eq!(
        shadow.kill_switch_listen_key_lifecycle_allowed.value,
        Some(false)
    );
    assert_eq!(shadow.risk_halted.value, Some(true));
    assert_eq!(shadow.manual_review_required.value, Some(true));
    assert_eq!(shadow.new_orders_blocked.value, Some(true));
    assert_eq!(shadow.actual_submission_count.value, Some(0));
    assert_eq!(shadow.production_order_submissions_attempted.value, Some(0));
    assert_eq!(shadow.production_orders_submitted.value, Some(0));
    assert_eq!(shadow.production_order_mutations_attempted.value, Some(0));
    assert_eq!(shadow.production_order_state_reads_attempted.value, Some(0));
    assert_eq!(shadow.listen_key_lifecycle_attempted.value, Some(0));
    assert_eq!(shadow.automatic_correction_orders_submitted.value, Some(0));
    assert_eq!(shadow.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(shadow.real_orders_submitted.value, Some(false));
    assert_eq!(
        shadow.order_state_values_are_exchange_truth.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(shadow.shadow_values_are_exchange_truth.value, Some(false));
    assert_eq!(
        shadow.portfolio_values_are_exchange_truth.value,
        Some(false)
    );
    assert_eq!(shadow.values_are_exchange_truth.value, Some(false));
    assert!(
        shadow
            .kill_switch_approval_artifact_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_13/kill_switch_approval_artifact.json"))
    );
    assert_eq!(snapshot.preflight_readiness.len(), 1);
    let readiness = &snapshot.preflight_readiness[0];
    assert_eq!(readiness.node_id, "prod-shadow-v13");
    assert_eq!(readiness.health, HealthStatus::Healthy);
    assert_eq!(
        readiness.readiness_status.value.as_deref(),
        Some("v13_preflight_readiness_ok")
    );
    assert_eq!(
        readiness.owner_proof_pack_status.value.as_deref(),
        Some("not_included_default_offline_preflight")
    );
    assert_eq!(
        readiness.kill_switch_artifact_status.value.as_deref(),
        Some("manual_approval_recorded")
    );
    assert_eq!(
        readiness.bounded_shadow_preflight_status.value.as_deref(),
        Some("bounded_shadow_preflight_contract_only")
    );
    assert_eq!(
        readiness.decimal_boundary_status.value.as_deref(),
        Some("decimal_boundary_contract_present")
    );
    assert_eq!(
        readiness
            .no_production_mutation_gate_status
            .value
            .as_deref(),
        Some("no_production_mutation_boundary_ok")
    );
    assert_eq!(
        readiness.production_order_submission_allowed.value,
        Some(false)
    );
    assert_eq!(
        readiness.production_order_mutation_allowed.value,
        Some(false)
    );
    assert_eq!(
        readiness.production_order_state_reads_allowed.value,
        Some(false)
    );
    assert_eq!(readiness.listen_key_lifecycle_allowed.value, Some(false));
    assert_eq!(
        readiness.dashboard_order_controls_enabled.value,
        Some(false)
    );
    assert_eq!(readiness.real_orders_submitted.value, Some(false));
    assert_eq!(
        readiness.order_state_values_are_exchange_truth.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        readiness.shadow_values_are_exchange_truth.value,
        Some(false)
    );
    assert_eq!(
        readiness.portfolio_values_are_exchange_truth.value,
        Some(false)
    );
    assert_eq!(readiness.values_are_exchange_truth.value, Some(false));
    assert!(
        readiness
            .evidence_source
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_13/kill_switch_approval_artifact.json"))
    );

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
    assert!(!DASHBOARD_JS.contains("submit_order"));
    assert!(!DASHBOARD_JS.contains("cancel_order"));
    assert!(!DASHBOARD_JS.contains("replace_order"));
    assert!(!DASHBOARD_JS.contains("amend_order"));
    assert!(!DASHBOARD_JS.contains("retry_order_action"));
    assert!(!DASHBOARD_JS.contains("data-workbench-action=\"retry_order\""));
    assert!(!DASHBOARD_JS.contains("/actions/retry_order"));
    assert!(!DASHBOARD_HTML.contains("credential"));
}

#[test]
fn dashboard_keeps_order_state_truth_separate_from_shadow_truth() {
    let root = temp_root("production-shadow-v13-order-state-truth");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "prod-shadow-v13-order-state-truth");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_shadow_v13_kill_switch_artifact(&record);

    let artifact_path = record
        .artifact_root
        .join(PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_RELATIVE_PATH);
    let mut artifact: Value =
        serde_json::from_str(&fs::read_to_string(&artifact_path).unwrap()).unwrap();
    artifact["order_state_values_are_exchange_truth"] = json!(true);
    artifact["shadow_values_are_exchange_truth"] = json!(false);
    artifact["portfolio_values_are_exchange_truth"] = json!(false);
    fs::write(
        &artifact_path,
        serde_json::to_string_pretty(&artifact).unwrap(),
    )
    .unwrap();

    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-22T01:40:00Z").unwrap();

    let shadow = &snapshot.production_shadow[0];
    assert_eq!(shadow.health, HealthStatus::Healthy);
    assert_eq!(
        shadow.order_state_values_are_exchange_truth.value,
        Some(true)
    );
    assert_eq!(shadow.shadow_values_are_exchange_truth.value, Some(false));
    assert_eq!(
        shadow.portfolio_values_are_exchange_truth.value,
        Some(false)
    );
    assert_eq!(shadow.values_are_exchange_truth.value, Some(true));

    let readiness = &snapshot.preflight_readiness[0];
    assert_eq!(readiness.health, HealthStatus::Healthy);
    assert_eq!(
        readiness
            .no_production_mutation_gate_status
            .value
            .as_deref(),
        Some("no_production_mutation_boundary_ok")
    );
    assert_eq!(
        readiness.order_state_values_are_exchange_truth.value,
        Some(true)
    );
    assert_eq!(
        readiness.shadow_values_are_exchange_truth.value,
        Some(false)
    );
    assert_eq!(
        readiness.portfolio_values_are_exchange_truth.value,
        Some(false)
    );
    assert_eq!(readiness.values_are_exchange_truth.value, Some(true));
}

#[test]
fn live_alpha_dry_run_artifacts_populate_readonly_dashboard_panel() {
    let root = temp_root("live-alpha-dry-run");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "live-alpha-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_live_alpha_dry_run_artifacts(&record);
    write_live_alpha_order_state_readonly_proof_artifact(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-22T01:10:00Z").unwrap();

    assert_eq!(snapshot.live_alpha_dry_run.len(), 1);
    let item = &snapshot.live_alpha_dry_run[0];
    assert_eq!(item.node_id, "live-alpha-a");
    assert_eq!(item.health, HealthStatus::Healthy);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("live_alpha_dry_run_ready")
    );
    assert_eq!(
        item.gate_status.value.as_deref(),
        Some("ready_dry_run_no_submission")
    );
    assert_eq!(item.gate_ready.value, Some(true));
    assert_eq!(item.missing_gate_flags.value.as_deref(), Some("none"));
    assert_eq!(item.dry_run_order_intent_recorded.value, Some(true));
    assert_eq!(
        item.order_submission_mode.value.as_deref(),
        Some("dry_run_no_submission")
    );
    assert_eq!(
        item.risk_preflight_status.value.as_deref(),
        Some("approved")
    );
    assert_eq!(
        item.risk_decision.value.as_deref(),
        Some("dry_run_approved")
    );
    assert_eq!(
        item.execution_decision.value.as_deref(),
        Some("blocked_no_production_mutation")
    );
    assert_eq!(
        item.risk_reasons.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(item.kill_switch_active.value, Some(false));
    assert_eq!(item.order_state_readable.value, Some(true));
    assert_eq!(
        item.order_state_read_status.value.as_deref(),
        Some("online_order_state_read_ok")
    );
    assert_eq!(
        item.order_state_endpoint.value.as_deref(),
        Some("open_orders")
    );
    assert_eq!(item.order_state_network_attempted.value, Some(true));
    assert_eq!(item.order_state_read_attempted.value, Some(true));
    assert_eq!(item.order_state_shape_validated.value, Some(true));
    assert_eq!(item.order_state_age_ms.value, Some(100));
    assert_eq!(item.max_order_state_age_ms.value, Some(1_000));
    assert_eq!(item.open_order_count.value, Some(0));
    assert_eq!(item.max_open_orders.value, Some(5));
    assert_eq!(item.non_empty_order_state_observed.value, Some(false));
    assert_eq!(item.order_lifecycle_readiness.value, Some(false));
    assert_eq!(
        item.order_state_truth_source.value.as_deref(),
        Some("exchange_order_state_readonly_proof")
    );
    assert_eq!(
        item.reconciliation_status.value.as_deref(),
        Some("approved")
    );
    assert_eq!(item.production_order_submission_allowed.value, Some(false));
    assert_eq!(item.production_order_mutation_allowed.value, Some(false));
    assert_eq!(item.production_order_state_reads_allowed.value, Some(false));
    assert_eq!(item.listen_key_lifecycle_allowed.value, Some(false));
    assert_eq!(item.production_order_submissions_attempted.value, Some(0));
    assert_eq!(item.production_orders_submitted.value, Some(0));
    assert_eq!(item.production_order_mutations_attempted.value, Some(0));
    assert_eq!(item.production_order_state_reads_attempted.value, Some(0));
    assert_eq!(item.listen_key_lifecycle_attempted.value, Some(0));
    assert_eq!(item.cancel_replace_amend_attempted.value, Some(false));
    assert_eq!(item.order_endpoint_access_attempted.value, Some(false));
    assert_eq!(item.execution_adapter_called.value, Some(false));
    assert_eq!(item.matching_engine_submission.value, Some(false));
    assert_eq!(item.actual_submission_count.value, Some(0));
    assert_eq!(item.automatic_correction_orders_submitted.value, Some(0));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.network_attempted.value, Some(false));
    assert_eq!(item.real_orders_submitted.value, Some(false));
    assert_eq!(item.real_funds.value, Some(false));
    assert_eq!(item.production_trading_enabled.value, Some(false));
    assert_eq!(item.order_state_values_are_exchange_truth.value, Some(true));
    assert_eq!(
        item.shadow_values_are_exchange_truth.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        item.portfolio_values_are_exchange_truth.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(item.values_are_exchange_truth.value, Some(true));
    assert!(
        item.order_gate_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_14/live_alpha_dry_run_order_gate.json"))
    );
    assert!(
        item.risk_preflight_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_14/live_alpha_risk_preflight.json"))
    );
    assert!(
        item.order_state_proof_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_14/production_order_state_readonly_proof.json"))
    );

    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
    assert!(DASHBOARD_JS.contains("读取状态"));
    assert!(DASHBOARD_JS.contains("生命周期"));
    assert!(DASHBOARD_JS.contains("真值来源"));
    assert!(!DASHBOARD_JS.contains("submit_order"));
    assert!(!DASHBOARD_JS.contains("cancel_order"));
    assert!(!DASHBOARD_JS.contains("replace_order"));
    assert!(!DASHBOARD_JS.contains("amend_order"));
    assert!(!DASHBOARD_JS.contains("retry_order_action"));
    assert!(!DASHBOARD_JS.contains("data-workbench-action=\"retry_order\""));
    assert!(!DASHBOARD_JS.contains("/actions/retry_order"));
}

#[test]
fn live_alpha_v15_dashboard_mutation_preflight_panel_is_readonly() {
    let root = temp_root("live-alpha-v15-mutation-preflight");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "live-alpha-v15-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_live_alpha_dry_run_artifacts(&record);
    write_live_alpha_order_state_readonly_proof_artifact(&record);
    write_live_alpha_v15_mutation_preflight_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-22T02:10:00Z").unwrap();

    assert_eq!(snapshot.live_alpha_dry_run.len(), 1);
    let item = &snapshot.live_alpha_dry_run[0];
    assert_eq!(item.node_id, "live-alpha-v15-a");
    assert_eq!(item.health, HealthStatus::Healthy);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("live_alpha_mutation_preflight_ready_for_owner_review")
    );
    assert_eq!(
        item.manual_approval_state.value.as_deref(),
        Some("approved")
    );
    assert_eq!(item.manual_approval_valid.value, Some(true));
    assert_eq!(item.manual_approval_recorded.value, Some(true));
    assert_eq!(item.manual_approval_one_time.value, Some(true));
    assert_eq!(item.manual_approval_used.value, Some(false));
    assert_eq!(
        item.manual_approval_issues.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        item.request_preview_status.value.as_deref(),
        Some("ready_request_preview_only")
    );
    assert_eq!(item.request_preview_allowed.value, Some(true));
    assert_eq!(item.request_preview_built.value, Some(true));
    assert_eq!(item.request_sent.value, Some(false));
    assert_eq!(item.request_method.value.as_deref(), Some("POST"));
    assert_eq!(item.request_target.value.as_deref(), Some("/api/v3/order"));
    assert_eq!(
        item.endpoint_class.value.as_deref(),
        Some("production_mutation_owner_approved_manual_only")
    );
    assert_eq!(
        item.endpoint_decision.value.as_deref(),
        Some("allow_request_preview_only")
    );
    assert_eq!(
        item.query_shape_without_signature.value.as_deref(),
        Some("symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp")
    );
    assert_eq!(
        item.signature_preflight.value.as_deref(),
        Some("created_in_memory_not_recorded")
    );
    assert_eq!(item.secrets_redacted.value, Some(true));
    assert_eq!(item.signed_request_memory_only.value, Some(true));
    assert_eq!(
        item.execution_dry_run_status.value.as_deref(),
        Some("ready_dry_run_execution_adapter_only")
    );
    assert_eq!(item.dry_run_execution_adapter_called.value, Some(true));
    assert_eq!(
        item.dry_run_execution_adapter_wrote_artifact.value,
        Some(true)
    );
    assert_eq!(item.dry_run_adapter_artifact_only.value, Some(true));
    assert_eq!(item.production_adapter_called.value, Some(false));
    assert_eq!(item.production_adapter_instantiated.value, Some(false));
    assert_eq!(item.strategy_intent_recorded.value, Some(true));
    assert_eq!(
        item.strategy_intent_reaches_risk_preflight.value,
        Some(true)
    );
    assert_eq!(
        item.strategy_intent_reaches_dry_run_adapter.value,
        Some(true)
    );
    assert_eq!(
        item.strategy_intent_reaches_production_adapter.value,
        Some(false)
    );
    assert_eq!(
        item.kill_switch_runtime_gate_status.value.as_deref(),
        Some("ready_runtime_gate_open_for_dry_run_only")
    );
    assert_eq!(
        item.runtime_gate_decision.value.as_deref(),
        Some("dry_run_runtime_gate_open")
    );
    assert_eq!(item.runtime_gate_open.value, Some(true));
    assert_eq!(
        item.runtime_gate_reasons.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(item.kill_switch_active.value, Some(false));
    assert_eq!(item.production_order_submission_allowed.value, Some(false));
    assert_eq!(item.production_order_mutation_allowed.value, Some(false));
    assert_eq!(item.production_order_submissions_attempted.value, Some(0));
    assert_eq!(item.production_orders_submitted.value, Some(0));
    assert_eq!(item.production_order_mutations_attempted.value, Some(0));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.network_attempted.value, Some(false));
    assert_eq!(item.real_orders_submitted.value, Some(false));
    assert_eq!(item.real_funds.value, Some(false));
    assert_eq!(item.production_trading_enabled.value, Some(false));
    assert!(
        item.manual_approval_lifecycle_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_15/manual_approval_lifecycle.json"))
    );
    assert!(
        item.request_preview_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_15/live_alpha_order_request_preview.json"))
    );
    assert!(
        item.execution_dry_run_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_15/live_alpha_execution_dry_run.json"))
    );
    assert!(
        item.kill_switch_runtime_gate_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_15/kill_switch_runtime_gate.json"))
    );

    let live_alpha_renderer = dashboard_js_function_body("renderLiveAlphaDryRun");
    for forbidden in [
        "<button",
        "data-dashboard-action",
        "撤单",
        "改单",
        "重试",
        "重连",
    ] {
        assert!(
            !live_alpha_renderer.contains(forbidden),
            "live-alpha renderer must stay read-only and not contain {forbidden}"
        );
    }
    assert!(live_alpha_renderer.contains("请求预览"));
    assert!(live_alpha_renderer.contains("人工审批"));
    assert!(live_alpha_renderer.contains("Runtime Gate"));
    assert!(live_alpha_renderer.contains("执行 Dry-run"));
}

#[test]
fn live_alpha_v15_dashboard_control_attempt_trace_is_blocked() {
    let root = temp_root("live-alpha-v15-control-trace-blocked");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "live-alpha-v15-b");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_live_alpha_dry_run_artifacts(&record);
    write_live_alpha_v15_mutation_preflight_artifacts(&record);
    let request_preview_path = record
        .artifact_root
        .join(LIVE_ALPHA_ORDER_REQUEST_PREVIEW_ARTIFACT_RELATIVE_PATH);
    let mut request_preview: Value =
        serde_json::from_str(&fs::read_to_string(&request_preview_path).unwrap()).unwrap();
    request_preview["request_sent"] = json!(true);
    request_preview["dashboard_order_controls_enabled"] = json!(true);
    fs::write(
        &request_preview_path,
        serde_json::to_string_pretty(&request_preview).unwrap(),
    )
    .unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-22T02:15:00Z").unwrap();

    assert_eq!(snapshot.live_alpha_dry_run.len(), 1);
    let item = &snapshot.live_alpha_dry_run[0];
    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("live_alpha_mutation_preflight_boundary_violation")
    );
    assert_eq!(
        item.diagnostic.value.as_deref(),
        Some("live_alpha_dry_run_readonly_boundary_violation")
    );
    assert_eq!(item.request_sent.value, Some(true));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(true));
    assert_eq!(item.production_orders_submitted.value, Some(0));
    assert_eq!(item.production_order_mutations_attempted.value, Some(0));
    assert_eq!(item.production_adapter_called.value, Some(false));
}

#[test]
fn production_mutation_v16_evidence_populates_readonly_dashboard_panel() {
    let root = temp_root("production-mutation-v16-evidence");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-mutation-v16-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_mutation_v16_evidence_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-23T05:20:00Z").unwrap();

    assert_eq!(snapshot.production_mutation_evidence.len(), 1);
    let item = &snapshot.production_mutation_evidence[0];
    assert_eq!(item.node_id, "production-mutation-v16-a");
    assert_eq!(item.health, HealthStatus::Healthy);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_mutation_evidence_ready_for_owner_review")
    );
    assert_eq!(
        item.runtime_gate_status.value.as_deref(),
        Some("blocked_explicit_send_gate")
    );
    assert_eq!(item.runtime_gate_open.value, Some(false));
    assert_eq!(
        item.signing_approval_status.value.as_deref(),
        Some("ready_signing_material_approval")
    );
    assert_eq!(item.approval_state.value.as_deref(), Some("approved"));
    assert_eq!(item.manual_approval_recorded.value, Some(true));
    assert_eq!(item.approved_by.value.as_deref(), Some("owner"));
    assert_eq!(
        item.request_builder_status.value.as_deref(),
        Some("ready_request_object_built_no_send")
    );
    assert_eq!(item.request_builder_ready.value, Some(true));
    assert_eq!(
        item.guarded_send_status.value.as_deref(),
        Some("ready_guarded_send_path_offline_no_network")
    );
    assert_eq!(item.guarded_send_ready.value, Some(true));
    assert_eq!(item.request_sent.value, Some(false));
    assert_eq!(item.network_attempted.value, Some(false));
    assert_eq!(item.kill_switch_checked_before_send.value, Some(true));
    assert_eq!(item.kill_switch_checked_after_send.value, Some(true));
    assert_eq!(
        item.response_redaction_status.value.as_deref(),
        Some("ready_response_redacted")
    );
    assert_eq!(item.response_redaction_ready.value, Some(true));
    assert_eq!(
        item.order_state_readback_status.value.as_deref(),
        Some("ready_offline_order_state_readback_contract")
    );
    assert_eq!(item.readback_contract_ready.value, Some(true));
    assert_eq!(item.order_state_read_attempted.value, Some(false));
    assert_eq!(item.response_shape_validated.value, Some(true));
    assert_eq!(
        item.audit_trail_status.value.as_deref(),
        Some("ready_redacted_audit_trail")
    );
    assert_eq!(item.audit_trail_ready.value, Some(true));
    assert_eq!(
        item.failure_semantics_status.value.as_deref(),
        Some("ready_failure_semantics_evidence")
    );
    assert_eq!(item.failure_semantics_ready.value, Some(true));
    assert_eq!(item.failure_mode.value.as_deref(), Some("timeout"));
    assert_eq!(
        item.terminal_action.value.as_deref(),
        Some("write_evidence_and_stop")
    );
    assert_eq!(item.strategy_continuation_allowed.value, Some(false));
    assert_eq!(item.symbol.value.as_deref(), Some("BTCUSDT"));
    assert_eq!(item.side.value.as_deref(), Some("BUY"));
    assert_eq!(item.order_type.value.as_deref(), Some("LIMIT"));
    assert_eq!(item.time_in_force.value.as_deref(), Some("GTC"));
    assert_eq!(item.quantity.value.as_deref(), Some("0.001"));
    assert_eq!(item.price.value.as_deref(), Some("10000.00"));
    assert_eq!(item.order_id.value.as_deref(), Some("123456789"));
    assert_eq!(item.production_order_submissions_attempted.value, Some(0));
    assert_eq!(item.production_orders_submitted.value, Some(0));
    assert_eq!(item.production_order_mutations_attempted.value, Some(0));
    assert_eq!(item.production_order_state_reads_attempted.value, Some(0));
    assert_eq!(item.listen_key_lifecycle_attempted.value, Some(0));
    assert_eq!(item.retry_attempted.value, Some(false));
    assert_eq!(item.cancel_attempted.value, Some(false));
    assert_eq!(item.replace_attempted.value, Some(false));
    assert_eq!(item.amend_attempted.value, Some(false));
    assert_eq!(item.correction_attempted.value, Some(false));
    assert_eq!(item.flatten_attempted.value, Some(false));
    assert_eq!(item.remediation_attempted.value, Some(false));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.real_orders_submitted.value, Some(false));
    assert_eq!(item.production_trading_enabled.value, Some(false));
    assert!(
        item.audit_trail_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_16/production_mutation_audit_trail.json"))
    );
    assert!(
        item.failure_semantics_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_16/production_mutation_failure_semantics.json"))
    );

    let renderer = dashboard_js_function_body("renderProductionMutationEvidence");
    for forbidden in ["<button", "data-dashboard-action", "fetch(", "credential"] {
        assert!(
            !renderer.contains(forbidden),
            "production mutation evidence renderer must stay read-only and not contain {forbidden}"
        );
    }
    assert!(renderer.contains("审批 / Runtime"));
    assert!(renderer.contains("请求 / Send"));
    assert!(renderer.contains("审计 / 失败"));
    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
}

#[test]
fn production_reconciliation_orphan_artifacts_populate_readonly_dashboard_panel() {
    let root = temp_root("production-reconciliation-orphan");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-reconciliation-orphan-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_mutation_v17_reconciliation_orphan_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-24T06:10:00Z").unwrap();

    assert_eq!(snapshot.production_reconciliation_orphan.len(), 1);
    let item = &snapshot.production_reconciliation_orphan[0];
    assert_eq!(item.node_id, "production-reconciliation-orphan-a");
    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_reconciliation_orphan_manual_review_required")
    );
    assert_eq!(
        item.missing_artifacts.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        item.schema_diagnostics.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        item.provenance_diagnostics.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        item.stale_artifacts.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        item.order_lineage_id.value.as_deref(),
        Some("lineage-v160-single-shot")
    );
    assert_eq!(
        item.local_ledger_status.value.as_deref(),
        Some("ready_local_order_ledger")
    );
    assert_eq!(
        item.local_order_state.value.as_deref(),
        Some("local_ledger_pending_exchange_reconciliation")
    );
    assert_eq!(item.local_ledger_ready.value, Some(true));
    assert_eq!(item.restart_readable.value, Some(true));
    assert_eq!(
        item.exchange_readback_status.value.as_deref(),
        Some("ready_exchange_readback_mapped")
    );
    assert_eq!(item.exchange_readback_mapped.value, Some(true));
    assert_eq!(item.exchange_order_state.value.as_deref(), Some("open"));
    assert_eq!(item.exchange_order_status.value.as_deref(), Some("NEW"));
    assert_eq!(item.open_order_observed.value, Some(true));
    assert_eq!(item.terminal_state_observed.value, Some(false));
    assert_eq!(
        item.reconciliation_status.value.as_deref(),
        Some("ready_reconciliation_classified")
    );
    assert_eq!(item.reconciliation_classified.value, Some(true));
    assert_eq!(
        item.reconciliation_outcome.value.as_deref(),
        Some("local_sent_exchange_new")
    );
    assert_eq!(
        item.orphan_status.value.as_deref(),
        Some("ready_orphan_order_detection_completed")
    );
    assert_eq!(item.orphan_detection_completed.value, Some(true));
    assert_eq!(
        item.orphan_detection_outcome.value.as_deref(),
        Some("open_orphan_risk")
    );
    assert_eq!(item.orphan_risk_detected.value, Some(true));
    assert_eq!(item.risk_halted.value, Some(true));
    assert_eq!(item.manual_review_required.value, Some(true));
    assert_eq!(item.new_orders_blocked.value, Some(true));
    assert_eq!(item.stale_ledger_restart_required.value, Some(false));
    assert_eq!(item.duplicate_submit_attempted.value, Some(false));
    assert_eq!(item.retry_attempted.value, Some(false));
    assert_eq!(item.cancel_attempted.value, Some(false));
    assert_eq!(item.remediation_attempted.value, Some(false));
    assert_eq!(item.automatic_cancel_allowed.value, Some(false));
    assert_eq!(item.automatic_remediation_allowed.value, Some(false));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(false));
    assert_eq!(item.network_attempted.value, Some(false));
    assert_eq!(item.production_order_submission_allowed.value, Some(false));
    assert_eq!(item.production_order_mutation_allowed.value, Some(false));
    assert!(
        item.local_order_ledger_path.value.as_deref().is_some_and(
            |path| path.ends_with("v0_17/production_mutation_local_order_ledger.json")
        )
    );
    assert!(
        item.orphan_order_detector_path
            .value
            .as_deref()
            .is_some_and(
                |path| path.ends_with("v0_17/production_mutation_orphan_order_detector.json")
            )
    );

    let renderer = dashboard_js_function_body("renderProductionReconciliationOrphan");
    for forbidden in [
        "<button",
        "data-dashboard-action",
        "fetch(",
        "credential",
        "api_key",
        "api_secret",
    ] {
        assert!(
            !renderer.contains(forbidden),
            "reconciliation/orphan renderer must stay read-only and not contain {forbidden}"
        );
    }
    assert!(renderer.contains("Lineage / 本地"));
    assert!(renderer.contains("交易所 Readback"));
    assert!(renderer.contains("孤儿单风险"));
    assert!(renderer.contains("缺失证据"));
    assert!(renderer.contains("Schema 诊断"));
    assert!(renderer.contains("Provenance 诊断"));
    assert!(renderer.contains("Stale 证据"));
    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
}

#[test]
fn production_reconciliation_orphan_missing_artifact_names_are_visible() {
    let root = temp_root("production-reconciliation-orphan-missing");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-reconciliation-orphan-missing");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_mutation_v17_reconciliation_orphan_artifacts(&record);
    fs::remove_file(
        record
            .artifact_root
            .join("v0_17")
            .join("production_mutation_exchange_readback_mapper.json"),
    )
    .unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-26T11:20:00Z").unwrap();

    assert_eq!(snapshot.production_reconciliation_orphan.len(), 1);
    let item = &snapshot.production_reconciliation_orphan[0];
    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_reconciliation_orphan_manual_review_required")
    );
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_reconciliation_orphan_missing_artifacts")
            && value.contains("exchange_readback_mapper")
    }));
    assert_eq!(
        item.missing_artifacts.value.as_deref(),
        Some("exchange_readback_mapper")
    );
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(false));
}

#[test]
fn production_reconciliation_orphan_schema_mismatch_is_explicit() {
    let root = temp_root("production-reconciliation-orphan-schema");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-reconciliation-orphan-schema");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_mutation_v17_reconciliation_orphan_artifacts(&record);
    let classifier_path = record
        .artifact_root
        .join("v0_17")
        .join("production_mutation_reconciliation_classifier.json");
    let mut classifier: Value =
        serde_json::from_str(&fs::read_to_string(&classifier_path).unwrap()).unwrap();
    classifier["schema_version"] = json!("ntpro.v170_wrong_schema.v1");
    fs::write(
        &classifier_path,
        serde_json::to_string_pretty(&classifier).unwrap(),
    )
    .unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-26T11:21:00Z").unwrap();

    assert_eq!(snapshot.production_reconciliation_orphan.len(), 1);
    let item = &snapshot.production_reconciliation_orphan[0];
    assert_eq!(item.health, HealthStatus::Degraded);
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_reconciliation_orphan_schema_mismatch")
            && value.contains("reconciliation_classifier")
    }));
    assert!(
        item.schema_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| {
                value.contains(PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION)
                    && value.contains("ntpro.v170_wrong_schema.v1")
            })
    );
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(false));
}

#[test]
fn production_reconciliation_orphan_provenance_and_stale_diagnostics_are_visible() {
    let root = temp_root("production-reconciliation-orphan-provenance");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-reconciliation-orphan-provenance");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_mutation_v17_reconciliation_orphan_artifacts(&record);
    let source_path = record.artifact_root.join("v0_17").join("source.json");
    fs::write(&source_path, r#"{"status":"source"}"#).unwrap();
    let orphan_path = record
        .artifact_root
        .join("v0_17")
        .join("production_mutation_orphan_order_detector.json");
    let mut orphan: Value =
        serde_json::from_str(&fs::read_to_string(&orphan_path).unwrap()).unwrap();
    orphan["reconciliation_classifier_ref"] = json!({
        "path": source_path.display().to_string(),
        "hash": "fnv1a64:legacy",
        "sha256": "sha256:0000",
        "bytes": 1,
        "source_command": "nautilus live production-mutation-reconciliation-classifier",
        "source_commit": "commit-a",
        "source_release_tag": "ntpro-rust-only-v0.17.1"
    });
    orphan["stale_ledger_restart_required"] = json!(true);
    fs::write(&orphan_path, serde_json::to_string_pretty(&orphan).unwrap()).unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-26T11:22:00Z").unwrap();

    assert_eq!(snapshot.production_reconciliation_orphan.len(), 1);
    let item = &snapshot.production_reconciliation_orphan[0];
    assert_eq!(item.health, HealthStatus::Degraded);
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_reconciliation_orphan_provenance_mismatch")
    }));
    assert!(
        item.provenance_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| {
                value.contains("orphan_order_detector.reconciliation_classifier_ref")
                    && value.contains("sha256_mismatch")
                    && value.contains("bytes_mismatch")
            })
    );
    assert_eq!(
        item.stale_artifacts.value.as_deref(),
        Some("orphan_order_detector")
    );
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(false));
}

#[test]
fn production_actual_cancel_audit_artifacts_populate_readonly_dashboard_view() {
    let root = temp_root("production-actual-cancel-audit");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-actual-cancel-audit-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_actual_cancel_audit_v19_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-28T09:10:00Z").unwrap();

    assert_eq!(snapshot.production_actual_cancel_audit.len(), 1);
    let item = &snapshot.production_actual_cancel_audit[0];
    assert_eq!(item.node_id, "production-actual-cancel-audit-a");
    assert_eq!(item.health, HealthStatus::Healthy);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_actual_cancel_audit_recovered")
    );
    assert_eq!(item.audit_state.value.as_deref(), Some("recovered"));
    assert_eq!(
        item.order_lineage_id.value.as_deref(),
        Some("lineage-v190-actual-cancel")
    );
    assert_eq!(item.risk_gate_ready.value, Some(true));
    assert_eq!(item.owner_approval_state.value.as_deref(), Some("approved"));
    assert_eq!(item.approval_execution_authorized.value, Some(true));
    assert_eq!(item.actual_cancel_command_ready.value, Some(true));
    assert_eq!(item.single_shot_cancel_allowed.value, Some(true));
    assert_eq!(item.request_sent.value, Some(true));
    assert_eq!(item.cancel_attempted.value, Some(true));
    assert_eq!(item.cancel_requests_sent.value, Some(1));
    assert_eq!(
        item.venue_response_status.value.as_deref(),
        Some("accepted")
    );
    assert_eq!(
        item.readback_result.value.as_deref(),
        Some("cancel_confirmed")
    );
    assert_eq!(
        item.cancel_outcome.value.as_deref(),
        Some("cancel_confirmed")
    );
    assert_eq!(item.outcome_category.value.as_deref(), Some("recovered"));
    assert_eq!(item.recovered.value, Some(true));
    assert_eq!(item.dashboard_read_only_consumable.value, Some(true));
    assert_eq!(item.dashboard_audit_view_ready.value, Some(true));
    assert_eq!(
        item.request_response_readback_audit_refs_recorded.value,
        Some(true)
    );
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(false));
    assert_eq!(item.retry_attempted.value, Some(false));
    assert_eq!(item.bulk_cancel_allowed.value, Some(false));
    assert!(
        item.failure_evidence_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_19/actual_cancel_failure_evidence.json"))
    );

    assert!(DASHBOARD_HTML.contains("production-actual-cancel-audit"));
    assert!(DASHBOARD_HTML.contains("v0.19 真实撤单审计只读视图"));
    let renderer = dashboard_js_function_body("renderProductionActualCancelAudit");
    for forbidden in [
        "<button",
        "data-dashboard-action",
        "fetch(",
        "/api/control/cancel",
        "/api/control/order",
        "cancel button",
        "approve button",
        "retry button",
        "bulk action",
    ] {
        assert!(
            !renderer.contains(forbidden),
            "actual cancel audit renderer must stay read-only and not contain {forbidden}"
        );
    }
    assert!(renderer.contains("Cancel Attempt / Venue"));
    assert!(renderer.contains("Outcome / Audit"));
    assert!(renderer.contains("只读边界"));
    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
}

fn production_actual_cancel_audit_item_with(
    name: &str,
    mutate: impl FnOnce(&SupervisorNodeRecord),
) -> ProductionActualCancelAuditStatus {
    let root = temp_root(name);
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, name);
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_actual_cancel_audit_v19_artifacts(&record);
    mutate(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-28T09:30:00Z").unwrap();
    assert_eq!(snapshot.production_actual_cancel_audit.len(), 1);
    snapshot.production_actual_cancel_audit[0].clone()
}

fn mutate_v19_actual_cancel_audit_artifact(
    record: &SupervisorNodeRecord,
    artifact_name: &str,
    mutate: impl FnOnce(&mut Value),
) {
    let path = record.artifact_root.join("v0_19").join(artifact_name);
    let mut artifact: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap())
        .expect("v0.19 actual cancel audit test artifact must be valid JSON");
    mutate(&mut artifact);
    fs::write(&path, serde_json::to_string_pretty(&artifact).unwrap()).unwrap();
}

#[test]
fn production_actual_cancel_audit_distinguishes_ready_recovered_degraded_failed_unknown() {
    for (category, outcome, recovered, failed, partial, expected_state, expected_health) in [
        (
            "ready",
            "ready_for_owner_review",
            false,
            false,
            false,
            "ready",
            HealthStatus::Healthy,
        ),
        (
            "recovered",
            "cancel_confirmed",
            true,
            false,
            false,
            "recovered",
            HealthStatus::Healthy,
        ),
        (
            "partial_success",
            "partial_fill",
            false,
            false,
            true,
            "degraded",
            HealthStatus::Degraded,
        ),
        (
            "failed",
            "rejected",
            false,
            true,
            false,
            "failed",
            HealthStatus::Error,
        ),
    ] {
        let item = production_actual_cancel_audit_item_with(
            &format!("production-actual-cancel-audit-state-{expected_state}"),
            |record| {
                mutate_v19_actual_cancel_audit_artifact(
                    record,
                    "actual_cancel_failure_evidence.json",
                    |artifact| {
                        artifact["outcome_category"] = json!(category);
                        artifact["cancel_outcome"] = json!(outcome);
                        artifact["recovered"] = json!(recovered);
                        artifact["failed"] = json!(failed);
                        artifact["partial_success"] = json!(partial);
                        artifact["degraded"] = json!(partial);
                        artifact["operator_action_required"] = json!(partial || failed);
                        artifact["residual_risk_visible"] = json!(partial || failed);
                    },
                );
            },
        );

        assert_eq!(item.audit_state.value.as_deref(), Some(expected_state));
        assert_eq!(item.health, expected_health);
        let expected_status = format!("production_actual_cancel_audit_{expected_state}");
        assert_eq!(
            item.readiness_status.value.as_deref(),
            Some(expected_status.as_str())
        );
    }

    let unknown = production_actual_cancel_audit_item_with(
        "production-actual-cancel-audit-state-unknown",
        |record| {
            mutate_v19_actual_cancel_audit_artifact(
                record,
                "actual_cancel_readback_reconciliation.json",
                |artifact| {
                    artifact["readback_result"] = json!("unknown");
                    artifact["unknown_observed"] = json!(true);
                },
            );
            mutate_v19_actual_cancel_audit_artifact(
                record,
                "actual_cancel_failure_evidence.json",
                |artifact| {
                    artifact["readback_result"] = json!("unknown");
                    artifact["cancel_outcome"] = json!("unknown");
                    artifact["outcome_category"] = json!("failed");
                    artifact["recovered"] = json!(false);
                    artifact["failed"] = json!(true);
                    artifact["unknown_not_recovered"] = json!(true);
                },
            );
        },
    );
    assert_eq!(unknown.audit_state.value.as_deref(), Some("unknown"));
    assert_eq!(unknown.health, HealthStatus::Degraded);
    assert!(unknown.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_actual_cancel_audit_unknown_readback")
    }));
}

#[test]
fn production_actual_cancel_audit_missing_evidence_never_shows_recovered() {
    let item = production_actual_cancel_audit_item_with(
        "production-actual-cancel-audit-missing",
        |record| {
            fs::remove_file(
                record
                    .artifact_root
                    .join("v0_19")
                    .join("actual_cancel_failure_evidence.json"),
            )
            .unwrap();
        },
    );

    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_actual_cancel_audit_incomplete")
    );
    assert_ne!(item.audit_state.value.as_deref(), Some("recovered"));
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_actual_cancel_audit_missing_evidence")
            && value.contains("actual_cancel_failure_evidence")
    }));
}

#[test]
fn production_actual_cancel_audit_schema_mismatch_degrades_view() {
    let item = production_actual_cancel_audit_item_with(
        "production-actual-cancel-audit-schema",
        |record| {
            mutate_v19_actual_cancel_audit_artifact(
                record,
                "actual_cancel_single_shot.json",
                |artifact| {
                    artifact["schema_version"] =
                        json!("ntpro.v190_wrong_actual_cancel_single_shot.v1");
                },
            );
        },
    );

    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_actual_cancel_audit_boundary_violation")
    );
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_actual_cancel_audit_schema_mismatch")
            && value.contains("actual_cancel_single_shot")
    }));
    assert!(
        item.schema_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| {
                value.contains(PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_SCHEMA_VERSION)
                    && value.contains("ntpro.v190_wrong_actual_cancel_single_shot.v1")
            })
    );
}

#[test]
fn production_actual_cancel_audit_provenance_mismatch_degrades_view() {
    let item = production_actual_cancel_audit_item_with(
        "production-actual-cancel-audit-provenance",
        |record| {
            let source_path = record
                .artifact_root
                .join("v0_19")
                .join("source-readback-reconciliation.json");
            let source = json!({
                "schema_version": PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_SCHEMA_VERSION,
                "status": "ready_actual_cancel_readback_cancel_confirmed",
                "source_commit": "commit-from-source-artifact",
                "source_release_tag": "ntpro-rust-only-v0.19.0"
            });
            let source_bytes = serde_json::to_vec_pretty(&source).unwrap();
            fs::write(&source_path, &source_bytes).unwrap();
            mutate_v19_actual_cancel_audit_artifact(
                record,
                "actual_cancel_failure_evidence.json",
                |artifact| {
                    artifact["readback_reconciliation_ref"] = json!({
                        "path": source_path.display().to_string(),
                        "sha256": sha256_bytes(&source_bytes),
                        "bytes": source_bytes.len() as u64,
                        "source_command": "nautilus live production-mutation-actual-cancel-readback-reconciliation",
                        "source_commit": "different-commit-in-ref",
                        "source_release_tag": "ntpro-rust-only-v0.19.bad"
                    });
                },
            );
        },
    );

    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_actual_cancel_audit_boundary_violation")
    );
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_actual_cancel_audit_provenance_mismatch")
    }));
    assert!(
        item.provenance_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| {
                value.contains("actual_cancel_failure_evidence.readback_reconciliation_ref")
                    && value.contains("source_commit_mismatch")
                    && value.contains("source_release_tag_mismatch")
            })
    );
}

#[test]
fn production_actual_cancel_audit_forbidden_controls_degrade_without_routes() {
    for (artifact_name, field, value) in [
        (
            "actual_cancel_failure_evidence.json",
            "dashboard_cancel_controls_enabled",
            json!(true),
        ),
        (
            "actual_cancel_failure_evidence.json",
            "retry_attempted",
            json!(true),
        ),
        (
            "actual_cancel_single_shot.json",
            "bulk_cancel_allowed",
            json!(true),
        ),
        (
            "actual_cancel_readback_reconciliation.json",
            "second_cancel_attempted",
            json!(true),
        ),
        (
            "actual_cancel_failure_evidence.json",
            "compensation_trade_attempted",
            json!(true),
        ),
    ] {
        let item = production_actual_cancel_audit_item_with(
            &format!("production-actual-cancel-audit-forbidden-{field}"),
            |record| {
                mutate_v19_actual_cancel_audit_artifact(record, artifact_name, |artifact| {
                    artifact[field] = value;
                });
            },
        );

        assert_eq!(item.health, HealthStatus::Degraded);
        assert_eq!(
            item.readiness_status.value.as_deref(),
            Some("production_actual_cancel_audit_boundary_violation")
        );
        assert_ne!(item.audit_state.value.as_deref(), Some("recovered"));
    }

    for forbidden_route in [
        "/api/control/cancel",
        "/api/control/order",
        "production_actual_cancel_control",
        "production_order_control",
    ] {
        assert!(
            !DASHBOARD_JS.contains(forbidden_route),
            "dashboard JS must not expose actual cancel/order control route {forbidden_route}"
        );
        assert!(
            !DASHBOARD_HTML.contains(forbidden_route),
            "dashboard shell must not expose actual cancel/order control route {forbidden_route}"
        );
    }
}

#[test]
fn production_order_lifecycle_audit_artifacts_populate_readonly_dashboard_view() {
    let root = temp_root("production-order-lifecycle-audit");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-order-lifecycle-audit-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_order_lifecycle_audit_v20_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-29T17:10:00Z").unwrap();

    assert_eq!(snapshot.production_order_lifecycle_audit.len(), 1);
    let item = &snapshot.production_order_lifecycle_audit[0];
    assert_eq!(item.node_id, "production-order-lifecycle-audit-a");
    assert_eq!(item.health, HealthStatus::Healthy);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_order_lifecycle_audit_audit_closed")
    );
    assert_eq!(item.audit_state.value.as_deref(), Some("audit_closed"));
    assert_eq!(
        item.risk_visibility.value.as_deref(),
        Some("no_risk_visible")
    );
    assert_eq!(
        item.lifecycle_id.value.as_deref(),
        Some("lineage-v200-single-shot-submit")
    );
    assert_eq!(item.attempt_id.value.as_deref(), Some("attempt-v200-001"));
    assert_eq!(
        item.submit_attempt_state.value.as_deref(),
        Some("submit_attempt_recorded")
    );
    assert_eq!(item.production_submit_attempted.value, Some(true));
    assert_eq!(item.readback_required.value, Some(true));
    assert_eq!(item.response_state.value.as_deref(), Some("accepted"));
    assert_eq!(item.venue_status.value.as_deref(), Some("NEW"));
    assert_eq!(item.venue_order_id.value.as_deref(), Some("123456789"));
    assert_eq!(
        item.client_order_id.value.as_deref(),
        Some("owner-approved-v200-submit")
    );
    assert_eq!(item.readback_state.value.as_deref(), Some("matched"));
    assert_eq!(item.readback_consistent.value, Some(true));
    assert_eq!(item.readback_missing.value, Some(false));
    assert_eq!(item.readback_failed.value, Some(false));
    assert_eq!(
        item.evidence_source_class.value.as_deref(),
        Some("foundation_only_manual_structured")
    );
    assert_eq!(item.adapter_runtime_integrated.value, Some(false));
    assert_eq!(item.foundation_only.value, Some(true));
    assert_eq!(item.exchange_truth_claimed.value, Some(false));
    assert_eq!(
        item.source_diagnostics.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(
        item.foundation_boundary_status.value.as_deref(),
        Some("foundation_only_no_adapter_runtime")
    );
    assert_eq!(
        item.foundation_boundary_diagnostics.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(item.failure_category.value.as_deref(), Some("none"));
    assert_eq!(
        item.next_allowed_action.value.as_deref(),
        Some("audit_closeout_only")
    );
    assert_eq!(item.no_implicit_retry.value, Some(true));
    assert_eq!(item.unknown_state_visible.value, Some(false));
    assert_eq!(item.audit_closed.value, Some(true));
    assert_eq!(item.dashboard_audit_consumable.value, Some(true));
    assert_eq!(item.release_gate_consumable.value, Some(true));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_approval_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(false));
    assert_eq!(item.retry_attempted.value, Some(false));
    assert_eq!(item.replace_attempted.value, Some(false));
    assert_eq!(item.amend_attempted.value, Some(false));
    assert_eq!(item.flatten_attempted.value, Some(false));
    assert_eq!(item.automatic_cancel_attempted.value, Some(false));
    assert_eq!(item.automatic_remediation_allowed.value, Some(false));
    assert_eq!(item.strategy_continuation_allowed.value, Some(false));
    assert!(
        item.submit_candidate_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_20/guarded_submit_candidate.json"))
    );
    assert!(
        item.audit_closeout_path
            .value
            .as_deref()
            .is_some_and(|path| path.ends_with("v0_20/order_lifecycle_audit_closeout.json"))
    );

    assert!(DASHBOARD_HTML.contains("production-order-lifecycle-audit"));
    assert!(DASHBOARD_HTML.contains("v0.20 订单生命周期审计只读视图"));
    let renderer = dashboard_js_function_body("renderProductionOrderLifecycleAudit");
    for forbidden in [
        "<button",
        "data-dashboard-action",
        "fetch(",
        "/api/control/cancel",
        "/api/control/order",
        "approve button",
        "retry button",
        "cancel button",
    ] {
        assert!(
            !renderer.contains(forbidden),
            "order lifecycle audit renderer must stay read-only and not contain {forbidden}"
        );
    }
    assert!(renderer.contains("Submit / Approval"));
    assert!(renderer.contains("Response / Readback"));
    assert!(renderer.contains("Source class"));
    assert!(renderer.contains("Adapter runtime"));
    assert!(renderer.contains("Foundation only"));
    assert!(renderer.contains("Exchange truth"));
    assert!(renderer.contains("Foundation boundary"));
    assert!(renderer.contains("Boundary diagnostics"));
    assert!(renderer.contains("Failure / Audit"));
    assert!(renderer.contains("只读边界"));
    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
}

fn production_order_lifecycle_audit_item_with(
    name: &str,
    mutate: impl FnOnce(&SupervisorNodeRecord),
) -> ProductionOrderLifecycleAuditStatus {
    let root = temp_root(name);
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, name);
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_order_lifecycle_audit_v20_artifacts(&record);
    mutate(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-29T17:20:00Z").unwrap();
    assert_eq!(snapshot.production_order_lifecycle_audit.len(), 1);
    snapshot.production_order_lifecycle_audit[0].clone()
}

fn mutate_v20_order_lifecycle_audit_artifact(
    record: &SupervisorNodeRecord,
    artifact_name: &str,
    mutate: impl FnOnce(&mut Value),
) {
    let path = record.artifact_root.join("v0_20").join(artifact_name);
    let mut artifact: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap())
        .expect("v0.20 order lifecycle audit test artifact must be valid JSON");
    mutate(&mut artifact);
    fs::write(&path, serde_json::to_string_pretty(&artifact).unwrap()).unwrap();
}

#[test]
fn production_order_lifecycle_audit_readback_mismatch_is_risk_visible_not_success() {
    let item = production_order_lifecycle_audit_item_with(
        "production-order-lifecycle-audit-mismatch",
        |record| {
            mutate_v20_order_lifecycle_audit_artifact(
                record,
                "submit_readback_reconciliation.json",
                |artifact| {
                    artifact["state"] = json!("mismatched");
                    artifact["code"] = json!("venue_status_mismatch");
                    artifact["mismatch_fields"] = json!(["venue_status"]);
                    artifact["readback_consistent"] = json!(false);
                },
            );
            mutate_v20_order_lifecycle_audit_artifact(
                record,
                "failure_no_retry_evidence.json",
                |artifact| {
                    artifact["category"] = json!("readback_mismatch");
                    artifact["code"] = json!("manual_audit_required");
                    artifact["next_allowed_action"] = json!("operator_manual_audit");
                },
            );
        },
    );

    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_order_lifecycle_audit_risk_visible")
    );
    assert_eq!(item.audit_state.value.as_deref(), Some("risk_visible"));
    assert_eq!(item.risk_visibility.value.as_deref(), Some("risk_visible"));
    assert_eq!(item.readback_state.value.as_deref(), Some("mismatched"));
    assert_eq!(
        item.failure_category.value.as_deref(),
        Some("readback_mismatch")
    );
    assert!(
        item.diagnostic.value.as_deref().is_some_and(|value| {
            value.contains("production_order_lifecycle_audit_risk_visible")
        })
    );
}

#[test]
fn production_order_lifecycle_audit_unknown_response_is_risk_visible_not_success() {
    let item = production_order_lifecycle_audit_item_with(
        "production-order-lifecycle-audit-unknown-response",
        |record| {
            mutate_v20_order_lifecycle_audit_artifact(
                record,
                "submit_response_redaction.json",
                |artifact| {
                    artifact["state"] = json!("unknown");
                    artifact["code"] = json!("exchange_response_timeout");
                },
            );
            mutate_v20_order_lifecycle_audit_artifact(
                record,
                "failure_no_retry_evidence.json",
                |artifact| {
                    artifact["category"] = json!("response_unknown");
                    artifact["code"] = json!("unknown_response_no_retry");
                    artifact["unknown_state_visible"] = json!(true);
                    artifact["next_allowed_action"] = json!("operator_manual_audit");
                },
            );
        },
    );

    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_order_lifecycle_audit_risk_visible")
    );
    assert_eq!(item.audit_state.value.as_deref(), Some("risk_visible"));
    assert_eq!(item.response_state.value.as_deref(), Some("unknown"));
    assert_eq!(
        item.failure_category.value.as_deref(),
        Some("response_unknown")
    );
    assert_eq!(item.unknown_state_visible.value, Some(true));
}

#[test]
fn production_order_lifecycle_audit_manual_source_claiming_exchange_truth_is_boundary_violation() {
    let item = production_order_lifecycle_audit_item_with(
        "production-order-lifecycle-audit-manual-source-claim",
        |record| {
            mutate_v20_order_lifecycle_audit_artifact(
                record,
                "submit_response_redaction.json",
                |artifact| {
                    artifact["exchange_truth_claimed"] = json!(true);
                    artifact["source_claim_consistent"] = json!(false);
                },
            );
        },
    );

    assert_eq!(item.health, HealthStatus::Error);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_order_lifecycle_audit_boundary_violation")
    );
    assert_eq!(
        item.audit_state.value.as_deref(),
        Some("boundary_violation")
    );
    assert_eq!(item.exchange_truth_claimed.value, Some(true));
    assert!(
        item.source_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| {
                value.contains("submit_response_redaction_source_claim_inconsistent")
                    && value.contains(
                        "submit_response_redaction_manual_structured_claims_exchange_truth",
                    )
            })
    );
}

#[test]
fn production_order_lifecycle_audit_adapter_source_missing_provenance_is_boundary_violation() {
    let item = production_order_lifecycle_audit_item_with(
        "production-order-lifecycle-audit-adapter-source-provenance",
        |record| {
            mutate_v20_order_lifecycle_audit_artifact(
                record,
                "submit_readback_reconciliation.json",
                |artifact| {
                    artifact["evidence_source"] = json!("exchange_readback");
                    artifact["adapter_runtime_integrated"] = json!(true);
                    artifact["foundation_only"] = json!(false);
                    artifact["source_provenance_id"] = Value::Null;
                    artifact["source_provenance_valid"] = json!(false);
                },
            );
        },
    );

    assert_eq!(item.health, HealthStatus::Error);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_order_lifecycle_audit_boundary_violation")
    );
    assert_eq!(
        item.evidence_source_class.value.as_deref(),
        Some("adapter_integrated_runtime")
    );
    assert_eq!(item.adapter_runtime_integrated.value, Some(true));
    assert_eq!(item.foundation_only.value, Some(false));
    assert!(
        item.source_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| {
                value.contains("submit_readback_reconciliation_source_provenance_missing")
            })
    );
}

#[test]
fn production_order_lifecycle_audit_adapter_runtime_claim_mismatch_is_boundary_violation() {
    let item = production_order_lifecycle_audit_item_with(
        "production-order-lifecycle-audit-adapter-runtime-claim",
        |record| {
            mutate_v20_order_lifecycle_audit_artifact(
                record,
                "submit_response_redaction.json",
                |artifact| {
                    artifact["adapter_runtime_integrated"] = json!(true);
                    artifact["foundation_only"] = json!(false);
                },
            );
        },
    );

    assert_eq!(item.health, HealthStatus::Error);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_order_lifecycle_audit_boundary_violation")
    );
    assert_eq!(
        item.foundation_boundary_status.value.as_deref(),
        Some("adapter_runtime_claim_mismatch")
    );
    assert!(
        item.source_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| value
                .contains("submit_response_redaction_manual_structured_claims_adapter_runtime"))
    );
    assert!(
        item.foundation_boundary_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| value.contains("adapter_runtime_claim_mismatch"))
    );
}

#[test]
fn production_order_lifecycle_audit_stale_evidence_blocks_audit_closed() {
    let item = production_order_lifecycle_audit_item_with(
        "production-order-lifecycle-audit-stale-evidence",
        |record| {
            mutate_v20_order_lifecycle_audit_artifact(
                record,
                "submit_response_redaction.json",
                |artifact| {
                    artifact["stale_evidence"] = json!(true);
                },
            );
        },
    );

    assert_eq!(item.health, HealthStatus::Error);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_order_lifecycle_audit_boundary_violation")
    );
    assert_eq!(
        item.audit_state.value.as_deref(),
        Some("boundary_violation")
    );
    assert_eq!(
        item.stale_artifacts.value.as_deref(),
        Some("submit_response_redaction")
    );
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_order_lifecycle_audit_stale_evidence")
            && value.contains("submit_response_redaction")
    }));
}

#[test]
fn production_order_lifecycle_audit_forbidden_controls_degrade_without_routes() {
    for (artifact_name, field, value) in [
        (
            "guarded_submit_candidate.json",
            "dashboard_order_controls_enabled",
            json!(true),
        ),
        (
            "guarded_submit_candidate.json",
            "dashboard_approval_controls_enabled",
            json!(true),
        ),
        (
            "failure_no_retry_evidence.json",
            "dashboard_cancel_controls_enabled",
            json!(true),
        ),
        (
            "failure_no_retry_evidence.json",
            "retry_attempted",
            json!(true),
        ),
        (
            "failure_no_retry_evidence.json",
            "replace_attempted",
            json!(true),
        ),
        (
            "failure_no_retry_evidence.json",
            "amend_attempted",
            json!(true),
        ),
        (
            "failure_no_retry_evidence.json",
            "flatten_attempted",
            json!(true),
        ),
        (
            "order_lifecycle_audit_closeout.json",
            "automatic_cancel_attempted",
            json!(true),
        ),
        (
            "order_lifecycle_audit_closeout.json",
            "automatic_remediation_allowed",
            json!(true),
        ),
        (
            "order_lifecycle_audit_closeout.json",
            "strategy_continuation_allowed",
            json!(true),
        ),
    ] {
        let item = production_order_lifecycle_audit_item_with(
            &format!("production-order-lifecycle-audit-forbidden-{field}"),
            |record| {
                mutate_v20_order_lifecycle_audit_artifact(record, artifact_name, |artifact| {
                    artifact[field] = value;
                });
            },
        );

        assert_eq!(item.health, HealthStatus::Error);
        assert_eq!(
            item.readiness_status.value.as_deref(),
            Some("production_order_lifecycle_audit_boundary_violation")
        );
        assert_ne!(item.audit_state.value.as_deref(), Some("audit_closed"));
    }

    for forbidden_route in [
        "/api/control/cancel",
        "/api/control/order",
        "/api/control/approval",
        "production_order_lifecycle_control",
        "production_order_approval_control",
        "production_order_cancel_control",
        "data-dashboard-action=\"submit\"",
        "data-dashboard-action=\"cancel\"",
        "data-dashboard-action=\"approval\"",
    ] {
        assert!(
            !DASHBOARD_JS.contains(forbidden_route),
            "dashboard JS must not expose order lifecycle execution route {forbidden_route}"
        );
        assert!(
            !DASHBOARD_HTML.contains(forbidden_route),
            "dashboard shell must not expose order lifecycle execution route {forbidden_route}"
        );
    }
}

#[test]
fn production_order_lifecycle_audit_missing_evidence_never_healthy() {
    let item = production_order_lifecycle_audit_item_with(
        "production-order-lifecycle-audit-missing",
        |record| {
            fs::remove_file(
                record
                    .artifact_root
                    .join("v0_20")
                    .join("failure_no_retry_evidence.json"),
            )
            .unwrap();
        },
    );

    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_order_lifecycle_audit_incomplete")
    );
    assert_ne!(item.audit_state.value.as_deref(), Some("audit_closed"));
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_order_lifecycle_audit_missing_evidence")
            && value.contains("failure_no_retry_evidence")
    }));
    assert_eq!(
        item.missing_artifacts.value.as_deref(),
        Some("failure_no_retry_evidence")
    );
}

#[test]
fn production_cancel_recovery_artifacts_populate_readonly_dashboard_panel() {
    let root = temp_root("production-cancel-recovery");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-cancel-recovery-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_cancel_recovery_v18_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-26T18:10:00Z").unwrap();

    assert_eq!(snapshot.production_cancel_recovery.len(), 1);
    let item = &snapshot.production_cancel_recovery[0];
    assert_eq!(item.node_id, "production-cancel-recovery-a");
    assert_eq!(item.health, HealthStatus::Healthy);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_cancel_recovery_ready")
    );
    assert_eq!(
        item.order_lineage_id.value.as_deref(),
        Some("lineage-v160-single-shot")
    );
    assert_eq!(
        item.cancel_preview_status.value.as_deref(),
        Some("ready_cancel_request_preview")
    );
    assert_eq!(item.cancel_request_preview_ready.value, Some(true));
    assert_eq!(
        item.risk_gate_result.value.as_deref(),
        Some("ready_owner_approval_required")
    );
    assert_eq!(item.risk_gate_ready.value, Some(true));
    assert_eq!(item.orphan_risk_detected.value, Some(true));
    assert_eq!(item.risk_halted.value, Some(true));
    assert_eq!(item.manual_review_required.value, Some(true));
    assert_eq!(item.new_orders_blocked.value, Some(true));
    assert_eq!(item.owner_approval_state.value.as_deref(), Some("approved"));
    assert_eq!(item.manual_approval_recorded.value, Some(true));
    assert_eq!(item.approval_lifecycle_valid.value, Some(true));
    assert_eq!(
        item.redaction_contract_state.value.as_deref(),
        Some("ready_redacted_metadata_only")
    );
    assert_eq!(item.cancel_response_redaction_ready.value, Some(true));
    assert_eq!(item.cancel_response_redacted.value, Some(true));
    assert_eq!(item.post_cancel_readback_ready.value, Some(true));
    assert_eq!(item.readback_state.value.as_deref(), Some("CANCELED"));
    assert_eq!(
        item.terminal_action_recommendation.value.as_deref(),
        Some("close_incident_cancel_confirmed")
    );
    assert_eq!(
        item.remaining_risk.value.as_deref(),
        Some("none_cancel_confirmed")
    );
    assert_eq!(
        item.remaining_risk_requires_manual_review.value,
        Some(false)
    );
    assert_eq!(item.actual_cancel_send_allowed.value, Some(false));
    assert_eq!(item.cancel_attempted.value, Some(false));
    assert_eq!(item.cancel_requests_sent.value, Some(0));
    assert_eq!(item.production_order_mutations_attempted.value, Some(0));
    assert_eq!(item.readback_execution_attempted.value, Some(false));
    assert_eq!(item.production_order_state_reads_attempted.value, Some(0));
    assert_eq!(item.network_attempted.value, Some(false));
    assert_eq!(item.network_readback_endpoint_attempted.value, Some(false));
    assert_eq!(item.network_cancel_endpoint_attempted.value, Some(false));
    assert_eq!(item.retry_attempted.value, Some(false));
    assert_eq!(item.remediation_attempted.value, Some(false));
    assert_eq!(item.automatic_cancel_allowed.value, Some(false));
    assert_eq!(item.automatic_remediation_allowed.value, Some(false));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_auto_approval_allowed.value, Some(false));
    assert_eq!(item.dashboard_auto_approval_attempted.value, Some(false));
    assert!(
        item.incident_audit_closeout_path
            .value
            .as_deref()
            .is_some_and(|path| {
                path.ends_with("v0_18/cancel_recovery_incident_audit_closeout.json")
            })
    );

    assert!(DASHBOARD_HTML.contains("production-cancel-recovery"));
    assert!(DASHBOARD_HTML.contains("v0.18 撤单恢复只读面板"));
    let renderer = dashboard_js_function_body("renderProductionCancelRecovery");
    for forbidden in [
        "<button",
        "data-dashboard-action",
        "fetch(",
        "credential",
        "api_key",
        "api_secret",
        "/api/control/cancel",
        "/api/control/order",
    ] {
        assert!(
            !renderer.contains(forbidden),
            "cancel recovery renderer must stay read-only and not contain {forbidden}"
        );
    }
    assert!(renderer.contains("撤单预览"));
    assert!(renderer.contains("风险门禁"));
    assert!(renderer.contains("Owner 审批"));
    assert!(renderer.contains("Incident / Audit"));
    let snapshot_value = serde_json::to_value(&snapshot).unwrap();
    assert_forbidden_keys_absent(&snapshot_value);
}

fn production_cancel_recovery_item_with(
    name: &str,
    mutate: impl FnOnce(&SupervisorNodeRecord),
) -> ProductionCancelRecoveryStatus {
    let root = temp_root(name);
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, name);
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_cancel_recovery_v18_artifacts(&record);
    mutate(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-27T18:58:00Z").unwrap();
    assert_eq!(snapshot.production_cancel_recovery.len(), 1);
    snapshot.production_cancel_recovery[0].clone()
}

fn mutate_v18_cancel_recovery_artifact(
    record: &SupervisorNodeRecord,
    artifact_name: &str,
    mutate: impl FnOnce(&mut Value),
) {
    let path = record.artifact_root.join("v0_18").join(artifact_name);
    let mut artifact: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap())
        .expect("v0.18 cancel recovery test artifact must be valid JSON");
    mutate(&mut artifact);
    fs::write(&path, serde_json::to_string_pretty(&artifact).unwrap()).unwrap();
}

fn assert_production_cancel_recovery_boundary_violation(item: &ProductionCancelRecoveryStatus) {
    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_cancel_recovery_boundary_violation")
    );
    assert_ne!(
        item.readiness_status.value.as_deref(),
        Some("production_cancel_recovery_ready")
    );
}

#[test]
fn production_cancel_recovery_missing_artifact_names_are_visible() {
    let root = temp_root("production-cancel-recovery-missing");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-cancel-recovery-missing");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_cancel_recovery_v18_artifacts(&record);
    fs::remove_file(
        record
            .artifact_root
            .join("v0_18")
            .join("post_cancel_readback.json"),
    )
    .unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-26T18:11:00Z").unwrap();

    assert_eq!(snapshot.production_cancel_recovery.len(), 1);
    let item = &snapshot.production_cancel_recovery[0];
    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_cancel_recovery_incomplete")
    );
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_cancel_recovery_missing_artifacts")
            && value.contains("post_cancel_readback")
    }));
    assert_eq!(
        item.missing_artifacts.value.as_deref(),
        Some("post_cancel_readback")
    );
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(false));
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(false));
}

#[test]
fn production_cancel_recovery_schema_mismatch_degrades_panel() {
    let item =
        production_cancel_recovery_item_with("production-cancel-recovery-schema", |record| {
            mutate_v18_cancel_recovery_artifact(record, "cancel_risk_gate.json", |artifact| {
                artifact["schema_version"] = json!("ntpro.v180_wrong_cancel_risk_gate.v1");
            });
        });

    assert_production_cancel_recovery_boundary_violation(&item);
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_cancel_recovery_schema_mismatch")
            && value.contains("cancel_risk_gate")
    }));
    assert!(
        item.schema_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| {
                value.contains(PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION)
                    && value.contains("ntpro.v180_wrong_cancel_risk_gate.v1")
            })
    );
}

#[test]
fn production_cancel_recovery_source_commit_and_release_tag_mismatch_degrade_panel() {
    let item =
        production_cancel_recovery_item_with("production-cancel-recovery-provenance", |record| {
            let source_path = record
                .artifact_root
                .join("v0_18")
                .join("source-preview.json");
            let source = json!({
                "schema_version": PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_SCHEMA_VERSION,
                "status": "ready_cancel_request_preview",
                "source_commit": "commit-from-source-artifact",
                "source_release_tag": "ntpro-rust-only-v0.18.0"
            });
            let source_bytes = serde_json::to_vec_pretty(&source).unwrap();
            fs::write(&source_path, &source_bytes).unwrap();
            mutate_v18_cancel_recovery_artifact(record, "cancel_risk_gate.json", |artifact| {
                artifact["cancel_request_preview_ref"] = json!({
                    "path": source_path.display().to_string(),
                    "sha256": sha256_bytes(&source_bytes),
                    "bytes": source_bytes.len() as u64,
                    "source_command": "nautilus live production-mutation-cancel-request-preview",
                    "source_commit": "different-commit-in-ref",
                    "source_release_tag": "ntpro-rust-only-v0.18.bad"
                });
            });
        });

    assert_production_cancel_recovery_boundary_violation(&item);
    assert!(
        item.diagnostic.value.as_deref().is_some_and(|value| {
            value.contains("production_cancel_recovery_provenance_mismatch")
        })
    );
    assert!(
        item.provenance_diagnostics
            .value
            .as_deref()
            .is_some_and(|value| {
                value.contains("cancel_risk_gate.cancel_request_preview_ref")
                    && value.contains("source_commit_mismatch")
                    && value.contains("source_release_tag_mismatch")
            })
    );
}

#[test]
fn production_cancel_recovery_stale_artifact_degrades_panel() {
    let item = production_cancel_recovery_item_with("production-cancel-recovery-stale", |record| {
        mutate_v18_cancel_recovery_artifact(record, "post_cancel_readback.json", |artifact| {
            artifact["stale_evidence"] = json!(true);
            artifact["status"] = json!("stale_post_cancel_readback");
        });
    });

    assert_production_cancel_recovery_boundary_violation(&item);
    assert!(item.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("production_cancel_recovery_stale_evidence")
            && value.contains("post_cancel_readback")
    }));
    assert_eq!(
        item.stale_artifacts.value.as_deref(),
        Some("post_cancel_readback")
    );
}

#[test]
fn production_cancel_recovery_forbidden_flags_each_degrade_panel() {
    for (field, value) in [
        ("actual_cancel_send_allowed", json!(true)),
        ("cancel_attempted", json!(true)),
        ("network_cancel_endpoint_attempted", json!(true)),
        ("dashboard_order_controls_enabled", json!(true)),
        ("dashboard_cancel_controls_enabled", json!(true)),
        ("dashboard_auto_approval_allowed", json!(true)),
        ("dashboard_auto_approval_attempted", json!(true)),
        ("cancel_requests_sent", json!(1)),
    ] {
        let item = production_cancel_recovery_item_with(
            &format!("production-cancel-recovery-forbidden-{field}"),
            |record| {
                mutate_v18_cancel_recovery_artifact(
                    record,
                    "cancel_recovery_incident_audit_closeout.json",
                    |artifact| {
                        artifact[field] = value;
                    },
                );
            },
        );

        assert_production_cancel_recovery_boundary_violation(&item);
        match field {
            "actual_cancel_send_allowed" => {
                assert_eq!(item.actual_cancel_send_allowed.value, Some(true));
            }
            "cancel_attempted" => {
                assert_eq!(item.cancel_attempted.value, Some(true));
            }
            "network_cancel_endpoint_attempted" => {
                assert_eq!(item.network_cancel_endpoint_attempted.value, Some(true));
            }
            "dashboard_order_controls_enabled" => {
                assert_eq!(item.dashboard_order_controls_enabled.value, Some(true));
            }
            "dashboard_cancel_controls_enabled" => {
                assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(true));
            }
            "dashboard_auto_approval_allowed" => {
                assert_eq!(item.dashboard_auto_approval_allowed.value, Some(true));
            }
            "dashboard_auto_approval_attempted" => {
                assert_eq!(item.dashboard_auto_approval_attempted.value, Some(true));
            }
            "cancel_requests_sent" => {
                assert_eq!(item.cancel_requests_sent.value, Some(1));
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn production_cancel_recovery_boundary_violation_degrades_without_control_routes() {
    let root = temp_root("production-cancel-recovery-boundary");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-cancel-recovery-boundary");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_cancel_recovery_v18_artifacts(&record);
    let closeout_path = record
        .artifact_root
        .join("v0_18")
        .join("cancel_recovery_incident_audit_closeout.json");
    let mut closeout: Value =
        serde_json::from_str(&fs::read_to_string(&closeout_path).unwrap()).unwrap();
    closeout["dashboard_cancel_controls_enabled"] = json!(true);
    closeout["network_cancel_endpoint_attempted"] = json!(true);
    fs::write(
        &closeout_path,
        serde_json::to_string_pretty(&closeout).unwrap(),
    )
    .unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-26T18:12:00Z").unwrap();

    assert_eq!(snapshot.production_cancel_recovery.len(), 1);
    let item = &snapshot.production_cancel_recovery[0];
    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_cancel_recovery_boundary_violation")
    );
    assert_eq!(item.dashboard_cancel_controls_enabled.value, Some(true));
    assert_eq!(item.network_cancel_endpoint_attempted.value, Some(true));
    for forbidden_route in [
        "/api/control/cancel",
        "/api/control/order",
        "production_cancel_control",
        "production_order_control",
    ] {
        assert!(
            !DASHBOARD_JS.contains(forbidden_route),
            "dashboard JS must not expose cancel/order control route {forbidden_route}"
        );
        assert!(
            !DASHBOARD_HTML.contains(forbidden_route),
            "dashboard shell must not expose cancel/order control route {forbidden_route}"
        );
    }
}

#[test]
fn production_mutation_v16_evidence_boundary_violation_degrades_panel() {
    let root = temp_root("production-mutation-v16-boundary");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "production-mutation-v16-b");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_mutation_v16_evidence_artifacts(&record);
    let failure_semantics_path = record
        .artifact_root
        .join(PRODUCTION_MUTATION_FAILURE_SEMANTICS_ARTIFACT_RELATIVE_PATH);
    let mut failure_semantics: Value =
        serde_json::from_str(&fs::read_to_string(&failure_semantics_path).unwrap()).unwrap();
    failure_semantics["production_order_mutations_attempted"] = json!(2);
    failure_semantics["dashboard_order_controls_enabled"] = json!(true);
    fs::write(
        &failure_semantics_path,
        serde_json::to_string_pretty(&failure_semantics).unwrap(),
    )
    .unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-23T05:21:00Z").unwrap();

    assert_eq!(snapshot.production_mutation_evidence.len(), 1);
    let item = &snapshot.production_mutation_evidence[0];
    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("production_mutation_evidence_boundary_violation")
    );
    assert_eq!(
        item.diagnostic.value.as_deref(),
        Some("production_mutation_evidence_readonly_boundary_violation")
    );
    assert_eq!(item.production_order_mutations_attempted.value, Some(2));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(true));
}

#[test]
fn live_alpha_dry_run_boundary_violation_degrades_dashboard_panel() {
    let root = temp_root("live-alpha-dry-run-boundary");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "live-alpha-b");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_live_alpha_dry_run_artifacts(&record);
    let risk_path = record
        .artifact_root
        .join(LIVE_ALPHA_RISK_PREFLIGHT_ARTIFACT_RELATIVE_PATH);
    let mut risk: Value = serde_json::from_str(&fs::read_to_string(&risk_path).unwrap()).unwrap();
    risk["production_orders_submitted"] = json!(1);
    risk["dashboard_order_controls_enabled"] = json!(true);
    fs::write(&risk_path, serde_json::to_string_pretty(&risk).unwrap()).unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-22T01:15:00Z").unwrap();

    assert_eq!(snapshot.live_alpha_dry_run.len(), 1);
    let item = &snapshot.live_alpha_dry_run[0];
    assert_eq!(item.health, HealthStatus::Degraded);
    assert_eq!(
        item.readiness_status.value.as_deref(),
        Some("live_alpha_dry_run_boundary_violation")
    );
    assert_eq!(
        item.diagnostic.value.as_deref(),
        Some("live_alpha_dry_run_readonly_boundary_violation")
    );
    assert_eq!(item.production_orders_submitted.value, Some(1));
    assert_eq!(item.dashboard_order_controls_enabled.value, Some(true));
    assert_eq!(item.execution_adapter_called.value, Some(false));
    assert_eq!(item.order_endpoint_access_attempted.value, Some(false));
    assert_eq!(item.network_attempted.value, Some(false));
}

#[test]
fn production_shadow_manifest_checksum_mismatch_degrades_dashboard_snapshot() {
    let root = temp_root("production-shadow-manifest-degraded");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "prod-shadow-b");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_shadow_artifacts(&record);
    fs::write(
        record
            .artifact_root
            .join("v0_11")
            .join("shadow_portfolio_snapshot.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v110_shadow_portfolio_snapshot.v1",
            "snapshot_mode": "production_readonly_shadow",
            "actual_submission_count": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "automatic_correction_orders_submitted": 0,
            "dashboard_order_controls_enabled": false,
            "full_production_portfolio_parity_claimed": false,
            "note": "changed after manifest checksum was recorded"
        }))
        .unwrap(),
    )
    .unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-20T10:00:00Z").unwrap();

    assert_eq!(snapshot.production_shadow.len(), 1);
    let shadow = &snapshot.production_shadow[0];
    assert_eq!(shadow.health, HealthStatus::Degraded);
    assert_eq!(
        shadow.manifest_status.value.as_deref(),
        Some("production_shadow_manifest_degraded")
    );
    assert!(
        shadow
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|value| value.contains("artifact_checksum_mismatch"))
    );
    assert_eq!(shadow.production_orders_submitted.value, Some(0));
    assert_eq!(shadow.production_order_mutations_attempted.value, Some(0));
    assert_eq!(shadow.dashboard_order_controls_enabled.value, Some(false));
}

#[test]
fn production_shadow_missing_required_artifact_degrades_dashboard_snapshot() {
    let root = temp_root("production-shadow-missing-required");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "prod-shadow-c");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_shadow_artifacts(&record);
    fs::remove_file(
        record
            .artifact_root
            .join("v0_11")
            .join("account_snapshot_redacted.json"),
    )
    .unwrap();
    fs::remove_file(record.artifact_root.join("v0_11").join("manifest.json")).unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-20T10:00:00Z").unwrap();

    assert_eq!(snapshot.production_shadow.len(), 1);
    let shadow = &snapshot.production_shadow[0];
    assert_eq!(shadow.health, HealthStatus::Degraded);
    assert!(
        shadow
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|value| value.contains("account_snapshot:missing_required_artifact"))
    );
    assert_eq!(
        shadow.manifest_status.availability,
        DashboardAvailability::Unknown
    );
    assert_eq!(shadow.production_orders_submitted.value, Some(0));
    assert_eq!(shadow.production_order_mutations_attempted.value, Some(0));
    assert_eq!(shadow.dashboard_order_controls_enabled.value, Some(false));
}

#[test]
fn production_shadow_health_scans_all_jsonl_records_for_boundary_violations() {
    let root = temp_root("production-shadow-jsonl-all-records");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "prod-shadow-d");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_production_shadow_artifacts(&record);
    fs::write(
        record
            .artifact_root
            .join("v0_11")
            .join("order_lifecycle_state.jsonl"),
        r#"{"schema_version":"ntpro.v110_order_lifecycle_state.v1","run_id":"v110-shadow","lifecycle_event_id":"life-1","intent_id":"intent-1","previous_state":"PreflightPassed","next_state":"ShadowSubmitted","reason":"local shadow ledger only","actual_submission":false,"actual_submission_count":0,"production_orders_submitted":1,"production_order_mutations_attempted":0,"exchange_order_id_recorded":false,"venue_order_id_recorded":false,"dashboard_order_controls_enabled":false}
{"schema_version":"ntpro.v110_order_lifecycle_state.v1","run_id":"v110-shadow","lifecycle_event_id":"life-2","intent_id":"intent-1","previous_state":"PreflightPassed","next_state":"ShadowSubmitted","reason":"local shadow ledger only","actual_submission":false,"actual_submission_count":0,"production_orders_submitted":0,"production_order_mutations_attempted":0,"exchange_order_id_recorded":false,"venue_order_id_recorded":false,"dashboard_order_controls_enabled":false}
"#,
    )
    .unwrap();
    fs::remove_file(record.artifact_root.join("v0_11").join("manifest.json")).unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-20T10:00:00Z").unwrap();

    assert_eq!(snapshot.production_shadow.len(), 1);
    let shadow = &snapshot.production_shadow[0];
    assert_eq!(shadow.health, HealthStatus::Degraded);
    assert_eq!(shadow.production_orders_submitted.value, Some(0));
    assert!(shadow.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("order_lifecycle_state:production_orders_submitted_nonzero")
    }));
}

#[test]
fn strategy_runtime_manifest_errors_degrade_dashboard_snapshot() {
    let root = temp_root("strategy-runtime-degraded");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "strategy-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_strategy_runtime_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;

    let strategy_root = record.artifact_root.join("strategy");
    fs::remove_file(strategy_root.join("manifest.json")).unwrap();
    write_registry(&registry_path, [record.clone()]);

    let missing_manifest =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-18T10:31:00Z").unwrap();
    let runtime = &missing_manifest.strategy_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Degraded);
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|value| value.contains("strategy manifest missing"))
    );

    write_strategy_manifest(&strategy_root);
    fs::write(strategy_root.join("manifest.json"), "{not-json").unwrap();
    let corrupt_manifest =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-18T10:32:00Z").unwrap();
    let runtime = &corrupt_manifest.strategy_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Degraded);
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|value| value.contains("strategy manifest JSON invalid"))
    );

    write_strategy_manifest(&strategy_root);
    fs::write(strategy_root.join("signal.jsonl"), "corrupted\n").unwrap();
    let corrupt_child =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-18T10:33:00Z").unwrap();
    let runtime = &corrupt_child.strategy_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Degraded);
    assert!(
        runtime
            .diagnostic
            .value
            .as_deref()
            .is_some_and(|value| value.contains("strategy artifact signal checksum mismatch"))
    );

    write_strategy_runtime_artifacts(&record);
    let manifest_path = strategy_root.join("manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["state"] = json!("running");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let conflicting_state =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-18T10:34:00Z").unwrap();
    let runtime = &conflicting_state.strategy_runtime[0];
    assert_eq!(runtime.health, HealthStatus::Degraded);
    assert!(runtime.diagnostic.value.as_deref().is_some_and(|value| {
        value.contains("node lifecycle stopped but strategy session state is running")
    }));
}

#[tokio::test]
async fn dashboard_http_server_serves_shell_snapshot_and_rejects_invalid_action_state() {
    let root = temp_root("http-server");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    let status = node_status_for_record(&record, LifecycleStatus::Running);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    record.last_known_status = status.clone();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    record.process.state = SupervisorProcessState::Running;
    record.process.pid = SnapshotValue::available(std::process::id());
    write_pid_artifact(&record);
    write_registry(&registry_path, [record]);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ntpro_node_bin = root.join("ntpro-node-missing");
    let server = tokio::spawn(async move {
        axum::serve(listener, dashboard_router(registry_path, ntpro_node_bin))
            .await
            .unwrap();
    });

    let shell = http_request(addr, "GET", "/dashboard").await;
    assert!(shell.contains("HTTP/1.1 200 OK"));
    assert!(shell.contains("NTPRO 监督器控制台"));

    let metadata = http_request(addr, "GET", "/api/server").await;
    assert!(metadata.contains("HTTP/1.1 200 OK"));
    let metadata_body = response_body(&metadata);
    let metadata_value: Value = serde_json::from_str(metadata_body).unwrap();
    assert_eq!(metadata_value["local_only"], true);
    assert!(
        metadata_value["registry_path"]
            .as_str()
            .unwrap()
            .ends_with("registry.json")
    );

    let snapshot = http_request(addr, "GET", "/api/snapshot").await;
    assert!(snapshot.contains("HTTP/1.1 200 OK"));
    let snapshot_body = response_body(&snapshot);
    let snapshot_value: Value = serde_json::from_str(snapshot_body).unwrap();
    assert_eq!(snapshot_value["nodes"][0]["node_id"], "sandbox-a");
    assert_eq!(snapshot_value["overview"]["running_nodes"], 1);
    assert_eq!(
        snapshot_value["data_sources"][0]["source_id"],
        "sandbox-a:data"
    );
    assert_eq!(
        snapshot_value["data_sources"][0]["source_kind"],
        json!({"availability": "available", "value": "supervisor_artifact"})
    );
    assert_eq!(
        snapshot_value["data_sources"][0]["provider"],
        json!({"availability": "available", "value": "local"})
    );
    assert_eq!(
        snapshot_value["execution_gateways"][0]["gateway_id"],
        "sandbox-a:gateway"
    );
    assert_eq!(
        snapshot_value["execution_gateways"][0]["account_ref"],
        json!({"availability": "redacted"})
    );
    assert_eq!(snapshot_value["risk"]["trading_state"], "active");
    assert_eq!(snapshot_value["risk"]["health"], "healthy");
    assert_eq!(
        snapshot_value["sandbox_business"]["availability"],
        "available"
    );
    assert_eq!(
        snapshot_value["sandbox_business"]["exchange"]["venue"],
        json!({"availability": "available", "value": "BINANCE"})
    );
    assert_eq!(
        snapshot_value["sandbox_business"]["exchange"]["instrument_id"],
        json!({"availability": "available", "value": "BTCUSDT.BINANCE"})
    );
    assert_eq!(
        snapshot_value["sandbox_business"]["strategies"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        snapshot_value["sandbox_business"]["order"]["mock_orders_requested"],
        json!({"availability": "available", "value": 7})
    );
    assert_eq!(
        snapshot_value["sandbox_business"]["risk"]["risk_reason"],
        json!({"availability": "available", "value": "TradingState::HALTED"})
    );
    assert!(
        snapshot_value["runtime_modules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|module| module["module_name"] == "sandbox-a:LiveNode")
    );
    assert!(
        snapshot_value["runtime_modules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|module| module["module_name"] == "sandbox-a:MessageBus"
                && module["status"]["availability"] == "not_supported")
    );
    assert_forbidden_keys_absent(&snapshot_value);

    let metrics = http_request(addr, "GET", "/api/nodes/sandbox-a/metrics").await;
    assert!(metrics.contains("HTTP/1.1 200 OK"));
    let metrics_body = response_body(&metrics);
    let metrics_value: Value = serde_json::from_str(metrics_body).unwrap();
    assert!(
        metrics_value
            .as_array()
            .unwrap()
            .iter()
            .any(|metric| metric["metric_id"] == "sandbox-a:starts_total")
    );

    let reconnected_data =
        http_request(addr, "POST", "/api/nodes/sandbox-a/actions/reconnect_data").await;
    assert_http_ok(&reconnected_data, "reconnect_data");
    let reconnected_data_value: Value =
        serde_json::from_str(response_body(&reconnected_data)).unwrap();
    assert_eq!(reconnected_data_value["status"], "not_supported");
    assert_eq!(
        reconnected_data_value["error_code"],
        json!({"availability": "available", "value": "sandbox_reconnect_not_supported"})
    );
    assert_eq!(
        reconnected_data_value["message"],
        json!({"availability": "available", "value": DASHBOARD_DATA_RECONNECT_UNSUPPORTED_MESSAGE})
    );

    let reconnected_execution = http_request(
        addr,
        "POST",
        "/api/nodes/sandbox-a/actions/reconnect_execution",
    )
    .await;
    assert_http_ok(&reconnected_execution, "reconnect_execution");
    let reconnected_execution_value: Value =
        serde_json::from_str(response_body(&reconnected_execution)).unwrap();
    assert_eq!(reconnected_execution_value["status"], "not_supported");
    assert_eq!(
        reconnected_execution_value["error_code"],
        json!({"availability": "available", "value": "sandbox_reconnect_not_supported"})
    );
    assert_eq!(
        reconnected_execution_value["message"],
        json!({"availability": "available", "value": DASHBOARD_EXECUTION_RECONNECT_UNSUPPORTED_MESSAGE})
    );

    let action = http_request(addr, "POST", "/api/nodes/sandbox-a/actions/start").await;
    assert!(action.contains("HTTP/1.1 409 Conflict"));
    let action_body = response_body(&action);
    let action_value: Value = serde_json::from_str(action_body).unwrap();
    assert_eq!(action_value["status"], "rejected");
    assert_eq!(
        action_value["error_code"],
        json!({"availability": "available", "value": "invalid_lifecycle_state"})
    );

    server.abort();
}

#[tokio::test]
async fn dashboard_http_server_rejects_v13_production_order_control_routes() {
    let root = temp_root("v13-dashboard-boundary");
    let registry_path = root.join("registry.json");
    let record = node_record(&root, "sandbox-a");
    write_registry(&registry_path, [record]);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ntpro_node_bin = root.join("ntpro-node-missing");
    let server = tokio::spawn({
        let registry_path = registry_path.clone();
        async move {
            axum::serve(listener, dashboard_router(registry_path, ntpro_node_bin))
                .await
                .unwrap();
        }
    });

    for action in [
        "submit",
        "submit_order",
        "cancel",
        "cancel_order",
        "replace",
        "replace_order",
        "amend",
        "amend_order",
        "retry",
        "retry_order",
        "correct",
        "correct_order",
        "flatten",
        "flatten_position",
        "credential_entry",
        "listen_key",
    ] {
        let path = format!("/api/nodes/sandbox-a/actions/{action}");
        let response = http_request(addr, "POST", &path).await;
        assert!(
            response.contains("HTTP/1.1 404 Not Found"),
            "{action} must remain outside the v0.13 Dashboard control router, got:\n{response}",
        );
    }

    server.abort();
}

#[tokio::test]
async fn dashboard_http_server_rejects_missing_and_stopped_control_actions() {
    let root = temp_root("http-negative-control");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    record.last_known_status = status;
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ntpro_node_bin = root.join("ntpro-node-missing");
    let server = tokio::spawn({
        let registry_path = registry_path.clone();
        async move {
            axum::serve(listener, dashboard_router(registry_path, ntpro_node_bin))
                .await
                .unwrap();
        }
    });

    let missing = http_request(addr, "POST", "/api/nodes/missing/actions/pause").await;
    assert!(missing.contains("HTTP/1.1 404 Not Found"));
    let missing_value: Value = serde_json::from_str(response_body(&missing)).unwrap();
    assert_eq!(missing_value["status"], "rejected");
    assert_eq!(missing_value["previous_state"], "unknown");
    assert_eq!(missing_value["current_state"], "unknown");
    assert_eq!(
        missing_value["error_code"],
        json!({"availability": "available", "value": "node_not_found"})
    );

    for action in [
        "stop",
        "pause",
        "resume",
        "reconnect_data",
        "reconnect_execution",
    ] {
        let path = format!("/api/nodes/sandbox-a/actions/{action}");
        let response = http_request(addr, "POST", &path).await;
        assert!(
            response.contains("HTTP/1.1 409 Conflict"),
            "{action} expected HTTP 409 Conflict, got:\n{response}"
        );
        let value: Value = serde_json::from_str(response_body(&response)).unwrap();
        assert_eq!(value["status"], "rejected", "{action}");
        assert_eq!(value["previous_state"], "stopped", "{action}");
        assert_eq!(value["current_state"], "stopped", "{action}");
        assert_eq!(
            value["error_code"],
            json!({"availability": "available", "value": "invalid_lifecycle_state"}),
            "{action}",
        );
    }

    let state = DashboardServerState {
        registry_path,
        workflow_root: None,
        ntpro_node_bin: root.join("ntpro-node-missing"),
        lifecycle_action_lock: Arc::new(Mutex::new(())),
    };
    let (status, Json(unknown_action)) =
        control_action_response(&state, "sandbox-a", "reboot").unwrap();
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(unknown_action.status, ControlActionStatus::Rejected);
    assert_eq!(
        unknown_action.error_code,
        DashboardValue::available("unsupported_control_action".to_string())
    );
    assert_eq!(unknown_action.previous_state, LifecycleStatus::Stopped);
    assert_eq!(unknown_action.current_state, LifecycleStatus::Stopped);

    server.abort();
}

#[cfg(unix)]
#[tokio::test]
async fn dashboard_http_server_starts_and_stops_fixture_node() {
    let root = temp_root("http-control");
    let registry_path = root.join("registry.json");
    let config = write_config(&root, "sandbox-a");
    let fixture = write_fixture_node(&root);
    let store = SupervisorRegistryStore::new(registry_path.clone());
    store
        .register_node(RegisterNodeRequest {
            node_id: "sandbox-a".to_string(),
            config_path: config,
            artifact_root: Some(root.join("nodes").join("sandbox-a")),
        })
        .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, dashboard_router(registry_path, fixture))
            .await
            .unwrap();
    });

    let before = http_request(addr, "GET", "/api/snapshot").await;
    let before_value: Value = serde_json::from_str(response_body(&before)).unwrap();
    assert!(
        before_value["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["action"] == "start:sandbox-a" && control["enabled"] == true)
    );

    let started = http_request(addr, "POST", "/api/nodes/sandbox-a/actions/start").await;
    assert_http_ok(&started, "start fixture node");
    let started_value: Value = serde_json::from_str(response_body(&started)).unwrap();
    assert_eq!(started_value["status"], "succeeded");
    assert_eq!(started_value["previous_state"], "stopped");
    assert_eq!(started_value["current_state"], "running");
    assert_eq!(
        started_value["error_code"],
        json!({"availability": "unknown"})
    );

    let running_value = wait_for_http_node_state(
        addr,
        "sandbox-a",
        LifecycleStatus::Running,
        SupervisorProcessState::Running,
        "fixture node running before pause",
    )
    .await;
    assert_eq!(running_value["overview"]["running_nodes"], 1);
    assert!(
        running_value["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["action"] == "stop:sandbox-a" && control["enabled"] == true)
    );
    assert!(
        running_value["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["action"] == "pause:sandbox-a" && control["enabled"] == true)
    );
    assert!(
        running_value["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["action"] == "reconnect_data:sandbox-a"
                && control["enabled"] == true)
    );

    let paused = post_action_until_ok(
        addr,
        "/api/nodes/sandbox-a/actions/pause",
        "pause fixture node",
    )
    .await;
    let paused_value: Value = serde_json::from_str(response_body(&paused)).unwrap();
    assert_eq!(paused_value["status"], "succeeded");
    assert_eq!(paused_value["previous_state"], "running");
    assert_eq!(paused_value["current_state"], "paused");

    let paused_snapshot_value = wait_for_http_node_state(
        addr,
        "sandbox-a",
        LifecycleStatus::Paused,
        SupervisorProcessState::Running,
        "fixture node paused before resume",
    )
    .await;
    assert!(
        paused_snapshot_value["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["action"] == "resume:sandbox-a" && control["enabled"] == true)
    );
    assert!(
        paused_snapshot_value["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["action"] == "stop:sandbox-a" && control["enabled"] == true)
    );

    let resumed = post_action_until_ok(
        addr,
        "/api/nodes/sandbox-a/actions/resume",
        "resume fixture node",
    )
    .await;
    let resumed_value: Value = serde_json::from_str(response_body(&resumed)).unwrap();
    assert_eq!(resumed_value["status"], "succeeded");
    assert_eq!(resumed_value["previous_state"], "paused");
    assert_eq!(resumed_value["current_state"], "running");

    wait_for_http_node_state(
        addr,
        "sandbox-a",
        LifecycleStatus::Running,
        SupervisorProcessState::Running,
        "fixture node resumed before stop",
    )
    .await;

    let stop_path = "/api/mvp/v1/control-center/nodes/sandbox-a/actions/stop";
    let (first_stop, second_stop) = tokio::join!(
        http_request(addr, "POST", stop_path),
        http_request(addr, "POST", stop_path),
    );
    let stop_responses = [&first_stop, &second_stop];
    assert_eq!(
        stop_responses
            .iter()
            .filter(|response| response.contains("HTTP/1.1 200 OK"))
            .count(),
        1,
        "concurrent stop must produce exactly one success",
    );
    assert_eq!(
        stop_responses
            .iter()
            .filter(|response| response.contains("HTTP/1.1 409 Conflict"))
            .count(),
        1,
        "concurrent stop must reject the duplicate transition",
    );
    let stopped = if first_stop.contains("HTTP/1.1 200 OK") {
        &first_stop
    } else {
        &second_stop
    };
    let rejected = if first_stop.contains("HTTP/1.1 409 Conflict") {
        &first_stop
    } else {
        &second_stop
    };
    let stopped_value: Value = serde_json::from_str(response_body(stopped)).unwrap();
    assert_eq!(stopped_value["result"]["status"], "succeeded");
    assert_eq!(stopped_value["result"]["previous_state"], "running");
    assert_eq!(stopped_value["result"]["current_state"], "stopped");
    assert_eq!(
        stopped_value["result"]["error_code"],
        json!({"availability": "unknown"})
    );
    assert!(response_body(rejected).contains("\"status\":\"rejected\""));
    assert!(response_body(rejected).contains("\"value\":\"invalid_lifecycle_state\""));

    let after = http_request(addr, "GET", "/api/snapshot").await;
    let after_value: Value = serde_json::from_str(response_body(&after)).unwrap();
    assert_eq!(after_value["overview"]["stopped_nodes"], 1);
    assert!(
        after_value["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["action"] == "start:sandbox-a" && control["enabled"] == true)
    );

    server.abort();
}

#[tokio::test]
async fn control_center_lifecycle_api_rejects_multi_node_and_wrong_target_scope() {
    let root = temp_root("mvp-control-scope");
    let registry_path = root.join("registry.json");
    let first = node_record(&root, "sandbox-a");
    let second = node_record(&root, "sandbox-b");
    write_registry(&registry_path, [first.clone(), second]);

    let listener_result = tokio::net::TcpListener::bind("127.0.0.1:0").await;
    assert!(listener_result.is_ok());
    let Ok(listener) = listener_result else {
        return;
    };
    let addr_result = listener.local_addr();
    assert!(addr_result.is_ok());
    let Ok(addr) = addr_result else {
        return;
    };
    let server_registry_path = registry_path.clone();
    let missing_bin = root.join("ntpro-node-missing");
    let server = tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            dashboard_router(server_registry_path, missing_bin),
        )
        .await;
    });

    let multi_node = http_request(
        addr,
        "POST",
        "/api/mvp/v1/control-center/nodes/sandbox-a/actions/start",
    )
    .await;
    assert!(multi_node.contains("HTTP/1.1 503 Service Unavailable"));
    assert!(response_body(&multi_node).contains("control_center_scope_violation"));

    write_registry(&registry_path, [first]);
    let wrong_target = http_request(
        addr,
        "POST",
        "/api/mvp/v1/control-center/nodes/sandbox-b/actions/start",
    )
    .await;
    assert!(wrong_target.contains("HTTP/1.1 404 Not Found"));
    assert!(response_body(&wrong_target).contains("node_not_found"));

    server.abort();
}

#[test]
fn two_node_supervisor_artifacts_aggregate_overview() {
    let root = temp_root("two-node");
    let registry_path = root.join("registry.json");
    let mut first = node_record(&root, "sandbox-a");
    let mut second = node_record(&root, "sandbox-b");
    let first_status = node_status_for_record(&first, LifecycleStatus::Running);
    let second_status = node_status_for_record(&second, LifecycleStatus::Stopped);

    for (record, status) in [(&mut first, &first_status), (&mut second, &second_status)] {
        write_status_artifact(record, status);
        write_metrics_artifact(record, status);
        write_log_artifacts(record);
        record.status_artifact = RegistryArtifactState::Available;
        record.metrics_artifact = RegistryArtifactState::Available;
    }
    write_registry(&registry_path, [first, second]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:03:00Z").unwrap();

    assert_eq!(snapshot.overview.node_count, 2);
    assert_eq!(snapshot.overview.running_nodes, 1);
    assert_eq!(snapshot.overview.stopped_nodes, 1);
    assert_eq!(snapshot.nodes.len(), 2);
    assert_eq!(snapshot.data_sources.len(), 2);
    assert_eq!(snapshot.execution_gateways.len(), 2);
    assert_eq!(snapshot.runtime_modules.len(), 22);
    assert!(!snapshot.overview.external_venue_connection);
    assert!(!snapshot.overview.real_orders_submitted);
}

#[test]
fn missing_status_artifact_is_marked_explicitly() {
    let root = temp_root("missing-status");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    record.status_artifact = RegistryArtifactState::Missing;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:04:00Z").unwrap();

    assert_eq!(
        snapshot.nodes[0].generated_at.availability,
        SnapshotAvailability::Stale
    );
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "nodes.sandbox-a.status"
            && gap.notes.value.as_deref().unwrap().contains("missing")
    }));
}

#[test]
fn invalid_status_artifact_is_marked_explicitly() {
    let root = temp_root("invalid-status");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    create_node_dirs(&record);
    fs::write(&record.status_path, "not-json").unwrap();
    record.status_artifact = RegistryArtifactState::Invalid;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:05:00Z").unwrap();

    assert!(
        snapshot.nodes[0]
            .last_error
            .as_deref()
            .unwrap()
            .contains("状态工件无效")
    );
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "nodes.sandbox-a.status"
            && gap.notes.value.as_deref().unwrap().contains("无效")
    }));
}

#[test]
fn mismatched_status_identity_is_marked_explicitly() {
    let root = temp_root("mismatched-status-identity");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    let mut status = node_status_for_record(&record, LifecycleStatus::Running);
    status.node_id = "sandbox-b".to_string();
    write_status_artifact(&record, &status);
    record.status_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:05:30Z").unwrap();

    assert_eq!(snapshot.overview.running_nodes, 0);
    assert!(
        snapshot.nodes[0]
            .last_error
            .as_deref()
            .unwrap()
            .contains("状态节点身份不匹配")
    );
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "nodes.sandbox-a.status.node_id"
            && gap
                .notes
                .value
                .as_deref()
                .unwrap()
                .contains("注册表节点 'sandbox-a' 收到运行时节点 'sandbox-b'")
    }));
}

#[test]
fn missing_metrics_artifact_is_marked_explicitly() {
    let root = temp_root("missing-metrics");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    let status = node_status_for_record(&record, LifecycleStatus::Running);
    write_status_artifact(&record, &status);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Missing;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:06:00Z").unwrap();

    assert_eq!(snapshot.metrics.len(), 1);
    assert_eq!(
        snapshot.metrics[0].availability,
        DashboardAvailability::Unknown
    );
    assert!(snapshot.runtime_modules.iter().any(|module| {
        module.module_name == "sandbox-a:Metrics writer"
            && module.status.availability == DashboardAvailability::Unknown
            && module.health == HealthStatus::Unknown
    }));
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "runtime_modules.sandbox-a.metrics_writer"
            && gap.reason == DashboardAvailability::Unknown
    }));
    assert!(
        snapshot.metrics[0]
            .last_error
            .value
            .as_deref()
            .unwrap()
            .contains("missing")
    );
}

#[test]
fn mismatched_metrics_identity_is_marked_explicitly() {
    let root = temp_root("mismatched-metrics-identity");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    let status = node_status_for_record(&record, LifecycleStatus::Running);
    write_status_artifact(&record, &status);
    let mut metrics = NodeMetrics::from_status(
        &status,
        &NodeMetricArtifacts::from_record(&record),
        NodeMetricCounts {
            uptime_ms: Some(100),
            starts_total: 1,
            stops_total: 0,
            state_transitions_total: 1,
        },
    );
    metrics.node_id = "sandbox-b".to_string();
    write_node_metrics_artifact(&record.metrics_path, &metrics).unwrap();
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:06:30Z").unwrap();

    assert_eq!(snapshot.metrics.len(), 1);
    assert_eq!(
        snapshot.metrics[0].availability,
        DashboardAvailability::Unknown
    );
    assert!(
        snapshot.metrics[0]
            .last_error
            .value
            .as_deref()
            .unwrap()
            .contains("指标节点身份不匹配：注册表节点 'sandbox-a' 收到运行时节点 'sandbox-b'")
    );
}

#[test]
fn stale_process_and_artifact_states_are_marked_explicitly() {
    let root = temp_root("stale");
    let registry_path = root.join("registry.json");
    let mut record = node_record(&root, "sandbox-a");
    let mut status = node_status_for_record(&record, LifecycleStatus::Running);
    status.generated_at = SnapshotValue::stale();
    write_status_artifact(&record, &status);
    let mut metrics = NodeMetrics::from_status(
        &status,
        &NodeMetricArtifacts::from_record(&record),
        NodeMetricCounts {
            uptime_ms: Some(100),
            starts_total: 1,
            stops_total: 0,
            state_transitions_total: 1,
        },
    );
    metrics.generated_at = SnapshotValue::stale();
    write_node_metrics_artifact(&record.metrics_path, &metrics).unwrap();
    record.process.state = SupervisorProcessState::Stale;
    record.status_artifact = RegistryArtifactState::Stale;
    record.metrics_artifact = RegistryArtifactState::Stale;
    write_registry(&registry_path, [record]);

    let snapshot =
        snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:07:00Z").unwrap();

    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "nodes.sandbox-a.process" && gap.reason == DashboardAvailability::Stale
    }));
    assert!(snapshot.gaps.iter().any(|gap| {
        gap.field_path == "nodes.sandbox-a.status.generated_at"
            && gap.reason == DashboardAvailability::Stale
    }));
    assert!(
        snapshot
            .metrics
            .iter()
            .all(|metric| metric.availability == DashboardAvailability::Stale)
    );
    assert!(snapshot.alerts.active_count >= 3);
}

fn temp_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "ntpro-v03-004-{name}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_config(root: &Path, name: &str) -> PathBuf {
    let path = root.join(format!("{name}.toml"));
    fs::write(&path, "[run]\nid = \"dashboard-control-smoke\"\n").unwrap();
    path
}

fn node_record(root: &std::path::Path, node_id: &str) -> SupervisorNodeRecord {
    let config_path = root.join(format!("{node_id}.toml"));
    fs::write(&config_path, "environment = \"sandbox\"\n").unwrap();
    SupervisorNodeRecord::new(
        node_id.to_string(),
        config_path,
        root.join("nodes").join(node_id),
    )
}

fn node_status_for_record(
    record: &SupervisorNodeRecord,
    lifecycle_state: LifecycleStatus,
) -> NodeStatus {
    let mut status = NodeStatus::unknown(record.node_id.clone());
    status.process_mode = ProcessMode::TestHarness;
    status.config_path = SnapshotValue::available(record.config_path.display().to_string());
    status.artifact_root = SnapshotValue::available(record.artifact_root.display().to_string());
    status.lifecycle_state = lifecycle_state;
    status.previous_lifecycle_state = LifecycleStatus::Stopped;
    status.data_connection = ConnectionStatus::NotConfigured;
    status.execution_connection = ConnectionStatus::NotConfigured;
    status.execution.gateway_id = SnapshotValue::available(format!("{}:gateway", record.node_id));
    status.execution.connection = ConnectionStatus::NotConfigured;
    status.execution.started =
        SnapshotValue::available(lifecycle_state == LifecycleStatus::Running);
    status.execution.orders_open = SnapshotValue::available(0);
    status.execution.orders_inflight = SnapshotValue::available(0);
    status.execution.orders_closed = SnapshotValue::available(0);
    status.execution.last_report_at = SnapshotValue::available("2026-06-07T15:00:00Z".to_string());
    status.risk.trading_state = RiskTradingState::Active;
    status.risk.health = HealthStatus::Healthy;
    status.risk.command_count = SnapshotValue::available(0);
    status.risk.event_count = SnapshotValue::available(0);
    status.risk.rejections_total = SnapshotValue::available(0);
    status.generated_at = SnapshotValue::available("2026-06-07T15:00:00Z".to_string());
    status.last_transition_at = SnapshotValue::available("2026-06-07T15:00:00Z".to_string());
    status
}

fn write_registry(
    registry_path: &std::path::Path,
    records: impl IntoIterator<Item = SupervisorNodeRecord>,
) {
    let mut registry = SupervisorRegistry::default();
    for record in records {
        registry.nodes.insert(record.node_id.clone(), record);
    }
    registry.updated_at = SnapshotValue::available("2026-06-07T15:00:00Z".to_string());
    if let Some(parent) = registry_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let raw = serde_json::to_string_pretty(&registry).unwrap();
    fs::write(registry_path, format!("{raw}\n")).unwrap();
}

fn write_workflow_manifest(path: &Path, run_id: &str, real_orders_submitted: bool) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let manifest = json!({
        "schema_version": "ntpro.workflow_manifest.v1",
        "workflow_id": "v05-binance-sandbox-local-workflow",
        "workflow": "binance-sandbox",
        "run_id": run_id,
        "runtime_status": "completed",
        "artifact_count": 9,
        "artifacts": [],
        "summary": {
            "schema_version": "ntpro.workflow_summary.v1",
            "workflow_id": "v05-binance-sandbox-local-workflow",
            "workflow": "binance-sandbox",
            "run_id": run_id,
            "runtime_status": "completed",
            "market_fixture_id": "v04-binance-spot-bars",
            "market_bar_count": 12,
            "market_checksum": "be481da0f80f7ca2",
            "ema_smoke_id": "v04-binance-ema-smoke",
            "ema_signals_emitted": 3,
            "ema_checksum": "ema-checksum",
            "rsi_smoke_id": "v04-binance-rsi-smoke",
            "rsi_signals_emitted": 4,
            "rsi_checksum": "rsi-checksum",
            "order_lifecycle_id": "v04-binance-mock-order-lifecycle",
            "order_event_count": 5,
            "order_checksum": "order-checksum",
            "risk_smoke_id": "v04-binance-risk-rejection-smoke",
            "risk_checksum": "60b0dc50f47caea8",
            "sandbox_only": true,
            "fixture_replay": true,
            "mock_execution": true,
            "external_venue_connection": false,
            "real_funds": false,
            "production_trading": false,
            "real_orders_submitted": real_orders_submitted
        }
    });
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
    .unwrap();
}

fn write_testnet_workflow_manifest(path: &Path, run_id: &str) {
    write_testnet_workflow_manifest_with_artifacts(path, run_id, &json!([]));
}

fn write_testnet_workflow_manifest_with_artifacts(path: &Path, run_id: &str, artifacts: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let order_lifecycle_id = format!("binance-testnet-readonly-no-order-lifecycle-{run_id}");
    let reconciliation_id = format!("binance-testnet-artifact-only-reconciliation-{run_id}");
    let summary = json!({
        "schema_version": "ntpro.workflow_summary.v1",
        "workflow_id": "binance-testnet-readonly-connectivity-foundation",
        "workflow": "binance-testnet",
        "run_id": run_id,
        "runtime_status": "dry_run_completed",
        "market_fixture_id": "not-applicable-testnet-dry-run",
        "market_bar_count": 0,
        "market_checksum": "dry_run_validated",
        "ema_smoke_id": "not-applicable-testnet-dry-run",
        "ema_signals_emitted": 0,
        "ema_checksum": "not-applicable",
        "rsi_smoke_id": "not-applicable-testnet-dry-run",
        "rsi_signals_emitted": 0,
        "rsi_checksum": "not-applicable",
        "order_lifecycle_id": order_lifecycle_id,
        "order_event_count": 1,
        "order_checksum": "binance-testnet-readonly-no-real-orders",
        "risk_smoke_id": reconciliation_id,
        "risk_checksum": "ok",
        "sandbox_only": true,
        "fixture_replay": false,
        "mock_execution": true,
        "external_venue_connection": false,
        "production_venue_connection": false,
        "testnet_public_network_connection": false,
        "external_network_attempted": false,
        "real_funds": false,
        "production_trading": false,
        "real_orders_submitted": false,
        "testnet_connection": false,
        "network_attempted": false,
        "credential_policy": "env-var-only-no-secret-persistence",
        "connectivity_mode": "dry-run",
        "order_submission_mode": "disabled",
        "reconciliation_mode": "artifact-only"
    });
    let manifest = json!({
        "schema_version": "ntpro.workflow_manifest.v1",
        "workflow_id": "binance-testnet-readonly-connectivity-foundation",
        "workflow": "binance-testnet",
        "run_id": run_id,
        "runtime_status": "dry_run_completed",
        "artifact_count": 11,
        "artifacts": artifacts,
        "summary": summary
    });
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
    .unwrap();
}

fn write_status_artifact(record: &SupervisorNodeRecord, status: &NodeStatus) {
    create_node_dirs(record);
    fs::write(
        &record.status_path,
        serde_json::to_string_pretty(status).unwrap(),
    )
    .unwrap();
}

fn write_metrics_artifact(record: &SupervisorNodeRecord, status: &NodeStatus) {
    let metrics = NodeMetrics::from_status(
        status,
        &NodeMetricArtifacts::from_record(record),
        NodeMetricCounts {
            uptime_ms: Some(100),
            starts_total: 1,
            stops_total: 0,
            state_transitions_total: 1,
        },
    );
    write_node_metrics_artifact(&record.metrics_path, &metrics).unwrap();
}

fn write_pid_artifact(record: &SupervisorNodeRecord) {
    create_node_dirs(record);
    let pid = record.process.pid.value.expect("test process pid");
    let artifact = SupervisorPidArtifact {
        node_id: record.node_id.clone(),
        pid,
        state: record.process.state,
        updated_at: record.process.updated_at.clone(),
        process_identity: Some(SupervisorProcessIdentity::from_record(record)),
    };
    fs::write(
        &record.pid_path,
        serde_json::to_string_pretty(&artifact).unwrap(),
    )
    .unwrap();
}

fn write_log_artifacts(record: &SupervisorNodeRecord) {
    create_node_dirs(record);
    fs::write(&record.stdout_log_path, "stdout\n").unwrap();
    fs::write(&record.stderr_log_path, "stderr\n").unwrap();
    fs::write(&record.events_log_path, "event=start status=ok\n").unwrap();
}

fn write_strategy_runtime_artifacts(record: &SupervisorNodeRecord) {
    let strategy_root = record.artifact_root.join("strategy");
    fs::create_dir_all(&strategy_root).unwrap();
    fs::write(
        strategy_root.join("session_status.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v09_strategy_session_status.v1",
            "session_id": "btc-ema-shadow-001",
            "strategy_id": "ema_cross_btcusdt_v1",
            "state": "stopped",
            "reason": "demo strategy stopped",
            "updated_at_unix_ms": 1000,
            "artifacts": {
                "session_status": strategy_root.join("session_status.json").display().to_string(),
                "events": strategy_root.join("events.jsonl").display().to_string(),
                "market_status": strategy_root.join("market_status.json").display().to_string(),
                "market_events": strategy_root.join("market_events.jsonl").display().to_string(),
                "signal": strategy_root.join("signal.jsonl").display().to_string(),
                "order_intent": strategy_root.join("order_intent.jsonl").display().to_string(),
                "risk_decision": strategy_root.join("risk_decision.jsonl").display().to_string(),
                "summary": strategy_root.join("summary.json").display().to_string(),
                "manifest": strategy_root.join("manifest.json").display().to_string()
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        strategy_root.join("market_status.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v09_market_status.v1",
            "session_id": "btc-ema-shadow-001",
            "strategy_id": "ema_cross_btcusdt_v1",
            "connection": "exhausted",
            "state": "exhausted",
            "source": "fixture_bar_stream",
            "event_count": 8,
            "last_event_at_unix_ms": 2000,
            "updated_at_unix_ms": 2001
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        strategy_root.join("events.jsonl"),
        r#"{"schema_version":"ntpro.v09_strategy_session_event.v1","event_type":"strategy_session_state_changed","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","state":"stopped","reason":"demo strategy stopped","occurred_at_unix_ms":3}
"#,
    )
    .unwrap();
    fs::write(
        strategy_root.join("market_events.jsonl"),
        r#"{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","event_type":"fixture_bar","source":"fixture_bar_stream","seq":1,"symbol":"BTCUSDT.BINANCE","price":100.0,"event_at_unix_ms":1,"recorded_at_unix_ms":2}
{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","event_type":"fixture_bar","source":"fixture_bar_stream","seq":2,"symbol":"BTCUSDT.BINANCE","price":101.0,"event_at_unix_ms":3,"recorded_at_unix_ms":4}
{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","event_type":"fixture_bar","source":"fixture_bar_stream","seq":3,"symbol":"BTCUSDT.BINANCE","price":102.0,"event_at_unix_ms":5,"recorded_at_unix_ms":6}
{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","event_type":"fixture_bar","source":"fixture_bar_stream","seq":4,"symbol":"BTCUSDT.BINANCE","price":103.0,"event_at_unix_ms":7,"recorded_at_unix_ms":8}
{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","event_type":"fixture_bar","source":"fixture_bar_stream","seq":5,"symbol":"BTCUSDT.BINANCE","price":104.0,"event_at_unix_ms":9,"recorded_at_unix_ms":10}
{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","event_type":"fixture_bar","source":"fixture_bar_stream","seq":6,"symbol":"BTCUSDT.BINANCE","price":105.0,"event_at_unix_ms":11,"recorded_at_unix_ms":12}
{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","event_type":"fixture_bar","source":"fixture_bar_stream","seq":7,"symbol":"BTCUSDT.BINANCE","price":106.0,"event_at_unix_ms":13,"recorded_at_unix_ms":14}
{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","event_type":"fixture_bar","source":"fixture_bar_stream","seq":8,"symbol":"BTCUSDT.BINANCE","price":107.0,"event_at_unix_ms":15,"recorded_at_unix_ms":16}
"#,
    )
    .unwrap();
    fs::write(
        strategy_root.join("signal.jsonl"),
        r#"{"schema_version":"ntpro.v09_signal.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","signal":"long","confidence":0.71,"market_event_seq":3,"generated_at":"unix:100","generated_at_unix_ms":100}
{"schema_version":"ntpro.v09_signal.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","signal":"flat","confidence":0.62,"market_event_seq":7,"generated_at":"unix:200","generated_at_unix_ms":200}
"#,
    )
    .unwrap();
    fs::write(
        strategy_root.join("order_intent.jsonl"),
        r#"{"schema_version":"ntpro.v09_order_intent.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","intent_id":"intent-1","symbol":"BTCUSDT.BINANCE","side":"buy","order_type":"market","quantity":1.0,"source_signal":"long","confidence":0.71,"market_event_seq":3,"signal_generated_at":"unix:100","created_at":"unix:101","created_at_unix_ms":101,"submission_allowed":false,"submission_status":"blocked_by_v09_strategy_runtime_boundary"}
{"schema_version":"ntpro.v09_order_intent.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","intent_id":"intent-2","symbol":"BTCUSDT.BINANCE","side":"sell","order_type":"market","quantity":1.0,"source_signal":"flat","confidence":0.62,"market_event_seq":7,"signal_generated_at":"unix:200","created_at":"unix:201","created_at_unix_ms":201,"submission_allowed":false,"submission_status":"blocked_by_v09_strategy_runtime_boundary"}
"#,
    )
    .unwrap();
    fs::write(
        strategy_root.join("risk_decision.jsonl"),
        r#"{"schema_version":"ntpro.v09_risk_decision.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","decision_id":"risk:intent-1","intent_id":"intent-1","symbol":"BTCUSDT.BINANCE","decision":"rejected","reasons":["order_submission_disabled"],"mode":"shadow","order_submission":"disabled","kill_switch_enabled":true,"kill_switch_active":false,"account_state":"missing","market_state":"available","actual_submission":false,"evaluated_at":"unix:102","evaluated_at_unix_ms":102}
{"schema_version":"ntpro.v09_risk_decision.v1","session_id":"btc-ema-shadow-001","strategy_id":"ema_cross_btcusdt_v1","decision_id":"risk:intent-2","intent_id":"intent-2","symbol":"BTCUSDT.BINANCE","decision":"rejected","reasons":["order_submission_disabled","shadow_mode_actual_submission_disabled"],"mode":"shadow","order_submission":"disabled","kill_switch_enabled":true,"kill_switch_active":false,"account_state":"missing","market_state":"available","actual_submission":false,"evaluated_at":"unix:202","evaluated_at_unix_ms":202}
"#,
    )
    .unwrap();
    fs::write(
        strategy_root.join("summary.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v09_strategy_session_summary.v1",
            "session_id": "btc-ema-shadow-001",
            "strategy_id": "ema_cross_btcusdt_v1",
            "state": "stopped",
            "event_count": 8,
            "market_event_count": 8,
            "signal_count": 2,
            "intent_count": 2,
            "risk_decision_count": 2,
            "rejection_count": 2,
            "actual_submission_count": 0,
            "updated_at_unix_ms": 3000
        }))
        .unwrap(),
    )
    .unwrap();
    write_strategy_manifest(&strategy_root);
}

fn write_trader_terminal_read_model_artifact(
    record: &SupervisorNodeRecord,
    mutate: impl FnOnce(&mut Value),
) {
    let artifact_path = record
        .artifact_root
        .join(TRADER_TERMINAL_READ_MODEL_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = artifact_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut artifact = healthy_trader_terminal_read_model_artifact();
    mutate(&mut artifact);
    fs::write(
        artifact_path,
        format!("{}\n", serde_json::to_string_pretty(&artifact).unwrap()),
    )
    .unwrap();
}

fn trader_terminal_read_model_runtime_with_mutation(
    name: &str,
    mutate: impl FnOnce(&mut Value),
) -> TraderTerminalReadModelStatus {
    let root = temp_root(name);
    let registry_path = root.join("registry.json");
    let node_id = format!("terminal-{name}");
    let mut record = node_record(&root, &node_id);
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    write_trader_terminal_read_model_artifact(&record, mutate);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T19:00:00Z")
        .unwrap()
        .read_model_runtime
        .into_iter()
        .next()
        .unwrap()
}

fn trader_terminal_read_model_runtime_without_artifact(
    name: &str,
) -> TraderTerminalReadModelStatus {
    let root = temp_root(name);
    let registry_path = root.join("registry.json");
    let node_id = format!("terminal-{name}");
    let mut record = node_record(&root, &node_id);
    let status = node_status_for_record(&record, LifecycleStatus::Stopped);
    write_status_artifact(&record, &status);
    write_metrics_artifact(&record, &status);
    write_log_artifacts(&record);
    record.status_artifact = RegistryArtifactState::Available;
    record.metrics_artifact = RegistryArtifactState::Available;
    write_registry(&registry_path, [record]);

    snapshot_from_supervisor_artifacts(&registry_path, "2026-07-01T19:30:00Z")
        .unwrap()
        .read_model_runtime
        .into_iter()
        .next()
        .unwrap()
}

fn assert_v220_operation_controls_disabled(
    runtime: &TraderTerminalReadModelStatus,
    case_name: &str,
) {
    for field in v220_required_false_operation_boundary_fields() {
        assert_eq!(
            v220_boundary_value(runtime, field),
            Some(false),
            "{case_name}: {field}"
        );
    }
}

fn v220_required_false_operation_boundary_fields() -> &'static [&'static str] {
    &[
        "new_submit_capability",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_approval_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "dashboard_retry_controls_enabled",
        "dashboard_fill_controls_enabled",
        "dashboard_submit_controls_enabled",
        "dashboard_replace_controls_enabled",
        "dashboard_amend_controls_enabled",
        "dashboard_flatten_controls_enabled",
        "dashboard_risk_controls_enabled",
        "retry_replace_amend_flatten_allowed",
        "trader_terminal_order_ticket_enabled",
        "trader_terminal_live_trading_claim",
        "product_grade_trading_terminal_claim",
        "funds_transfer_allowed",
        "account_configuration_mutation_allowed",
        "order_permission_control_allowed",
        "auto_flatten_position_allowed",
        "automatic_position_repair_allowed",
        "retry_order_allowed",
        "automatic_cancel_allowed",
        "automatic_order_remediation_allowed",
        "execution_algorithm_allowed",
        "automatic_fill_repair_allowed",
        "automatic_reconciliation_repair_allowed",
        "automatic_risk_action_allowed",
        "automatic_risk_repair_allowed",
        "automatic_alert_action_allowed",
        "automatic_audit_action_allowed",
        "automatic_provenance_repair_allowed",
        "manual_operation_entry_enabled",
        "manual_operation_submit_allowed",
        "manual_operation_cancel_allowed",
        "manual_operation_retry_allowed",
        "manual_operation_replace_allowed",
        "manual_operation_amend_allowed",
        "manual_operation_flatten_allowed",
        "automatic_operation_action_allowed",
    ]
}

fn v220_boundary_value(runtime: &TraderTerminalReadModelStatus, field: &str) -> Option<bool> {
    match field {
        "new_submit_capability" => runtime.new_submit_capability.value,
        "production_order_submission_allowed" => runtime.production_order_submission_allowed.value,
        "production_order_mutation_allowed" => runtime.production_order_mutation_allowed.value,
        "dashboard_order_controls_enabled" => runtime.dashboard_order_controls_enabled.value,
        "dashboard_approval_controls_enabled" => runtime.dashboard_approval_controls_enabled.value,
        "dashboard_submit_controls_enabled" => runtime.dashboard_submit_controls_enabled.value,
        "dashboard_cancel_controls_enabled" => runtime.dashboard_cancel_controls_enabled.value,
        "dashboard_retry_controls_enabled" => runtime.dashboard_retry_controls_enabled.value,
        "dashboard_fill_controls_enabled" => runtime.dashboard_fill_controls_enabled.value,
        "dashboard_replace_controls_enabled" => runtime.dashboard_replace_controls_enabled.value,
        "dashboard_amend_controls_enabled" => runtime.dashboard_amend_controls_enabled.value,
        "dashboard_flatten_controls_enabled" => runtime.dashboard_flatten_controls_enabled.value,
        "trader_terminal_order_ticket_enabled" => {
            runtime.trader_terminal_order_ticket_enabled.value
        }
        "trader_terminal_live_trading_claim" => runtime.trader_terminal_live_trading_claim.value,
        "product_grade_trading_terminal_claim" => {
            runtime.product_grade_trading_terminal_claim.value
        }
        "funds_transfer_allowed" => runtime.funds_transfer_allowed.value,
        "account_configuration_mutation_allowed" => {
            runtime.account_configuration_mutation_allowed.value
        }
        "order_permission_control_allowed" => runtime.order_permission_control_allowed.value,
        "auto_flatten_position_allowed" => runtime.auto_flatten_position_allowed.value,
        "automatic_position_repair_allowed" => runtime.automatic_position_repair_allowed.value,
        "retry_order_allowed" => runtime.retry_order_allowed.value,
        "automatic_cancel_allowed" => runtime.automatic_cancel_allowed.value,
        "automatic_order_remediation_allowed" => runtime.automatic_order_remediation_allowed.value,
        "execution_algorithm_allowed" => runtime.execution_algorithm_allowed.value,
        "automatic_fill_repair_allowed" => runtime.automatic_fill_repair_allowed.value,
        "automatic_reconciliation_repair_allowed" => {
            runtime.automatic_reconciliation_repair_allowed.value
        }
        "dashboard_risk_controls_enabled" => runtime.dashboard_risk_controls_enabled.value,
        "retry_replace_amend_flatten_allowed" => runtime.retry_replace_amend_flatten_allowed.value,
        "automatic_risk_action_allowed" => runtime.automatic_risk_action_allowed.value,
        "automatic_risk_repair_allowed" => runtime.automatic_risk_repair_allowed.value,
        "automatic_alert_action_allowed" => runtime.automatic_alert_action_allowed.value,
        "automatic_audit_action_allowed" => runtime.automatic_audit_action_allowed.value,
        "automatic_provenance_repair_allowed" => runtime.automatic_provenance_repair_allowed.value,
        "manual_operation_entry_enabled" => runtime.manual_operation_entry_enabled.value,
        "manual_operation_submit_allowed" => runtime.manual_operation_submit_allowed.value,
        "manual_operation_cancel_allowed" => runtime.manual_operation_cancel_allowed.value,
        "manual_operation_retry_allowed" => runtime.manual_operation_retry_allowed.value,
        "manual_operation_replace_allowed" => runtime.manual_operation_replace_allowed.value,
        "manual_operation_amend_allowed" => runtime.manual_operation_amend_allowed.value,
        "manual_operation_flatten_allowed" => runtime.manual_operation_flatten_allowed.value,
        "automatic_operation_action_allowed" => runtime.automatic_operation_action_allowed.value,
        _ => None,
    }
}

pub(super) fn healthy_trader_terminal_read_model_artifact() -> Value {
    let components = json!({
        "account": read_model_component("healthy", &json!({
            "summary_status": "ready",
            "account_id": "acct-redacted-001",
            "account_status": "redacted_ready",
            "risk_state": "active",
            "equity": "1000.00",
            "available_balance": "900.00",
            "balance_entry_count": 2,
            "dashboard_visible": true,
            "values_are_exchange_truth": false
        })),
        "positions": read_model_component("healthy", &json!({
            "summary_status": "ready",
            "account_id": "acct-redacted-001",
            "position_count": 1,
            "net_position_side": "flat",
            "quantity": "0",
            "net_exposure": "0",
            "notional": "0",
            "precision": "standard",
            "values_are_exchange_truth": false
        })),
        "orders": read_model_component("healthy", &json!({
            "order_id": "order-redacted-001",
            "client_order_id": "client-redacted-001",
            "request_digest": "sha256-redacted-request-001",
            "attempt_id": "attempt-redacted-001",
            "approval_id": "approval-redacted-001",
            "ledger_present": true,
            "duplicate_attempt_detected": false,
            "lifecycle_status": "readback_matched",
            "open_order_count": 0,
            "terminal_order_count": 1,
            "submitted": true,
            "accepted": true,
            "rejected": false,
            "readback_status": "matched",
            "cancel_evidence_state": "not_requested",
            "audit_state": "audit_closed",
            "refs": {
                "candidate_ref": "candidate:v210:matched",
                "attempt_ref": "attempt:v201:001",
                "approval_ref": "approval:v200:001",
                "audit_ref": "audit:v200:matched",
                "provenance_ref": "provenance:v200:matched"
            },
            "redaction_state": "redacted_refs_only",
            "no_retry": true,
            "automatic_remediation_allowed": false,
            "dashboard_readonly_visible": true,
            "values_are_exchange_truth": false
        })),
        "fills": read_model_component("healthy", &json!({
            "fill_id": "fill-redacted-001",
            "execution_id": "execution-redacted-001",
            "order_id": "order-redacted-001",
            "client_order_id": "client-redacted-001",
            "order_linkage_status": "linked",
            "fill_status": "reconciled",
            "fill_count": 1,
            "reconciliation_status": "reconciled",
            "duplicate_fill_detected": false,
            "partial_fill_detected": false,
            "quantity": "0.010",
            "cumulative_quantity": "0.010",
            "remaining_quantity": "0",
            "quantity_precision": 3,
            "price": "61000.00",
            "price_precision": 2,
            "precision_status": "valid",
            "source_provenance_ref": "execution:fixture:v210:fill:001",
            "risk_projection_input": {
                "fill_reconciliation_status": "reconciled",
                "realized_fill_quantity": "0.010",
                "remaining_order_quantity": "0",
                "risk_state": "active",
                "blocking_reasons": [],
                "automatic_reconciliation_repair_allowed": false,
                "execution_algorithm_allowed": false
            },
            "values_are_exchange_truth": false,
            "redaction_state": "redacted_refs_only",
            "no_execution_algorithm": true,
            "automatic_reconciliation_repair_allowed": false,
            "dashboard_readonly_visible": true
        })),
        "risk": read_model_component("healthy", &json!({
            "risk_state": "active",
            "risk_visible": false,
            "critical_evidence_complete": true,
            "manual_review_required": false,
            "halted": false,
            "mismatch_detected": false,
            "freshness_rollup": "fresh",
            "alerts": {
                "highest_severity": "info",
                "missing_evidence": false,
                "stale_source": false,
                "schema_mismatch": false,
                "redaction_breach": false,
                "forbidden_control_request": false
            },
            "blocking_reasons": [],
            "production_mutation_allowed": false,
            "automatic_trading_action_allowed": false,
            "audit_closed_allowed": true
        })),
        "lifecycle_status": read_model_component("healthy", &json!({
            "lifecycle_status": "read_only_foundation",
            "audit_state": "audit_closed",
            "audit_closed": true,
            "required_evidence_complete": true,
            "required_components_complete": true,
            "missing_evidence": [],
            "release_provenance": "ntpro-rust-only-v0.21.1",
            "artifact_digest": "sha256:read-model-dashboard-ready",
            "artifact_sha": "sha256:read-model-dashboard-ready",
            "provenance_mismatch": false,
            "readback_status": "not_applicable_readonly",
            "no_retry": true,
            "ledger_present": true,
            "automatic_remediation_allowed": false
        })),
        "operation_entry": read_model_component("healthy", &json!({
            "entry_contract_version": "ntpro.v220.manual_operation_entry.v1",
            "entry_state": "blocked_missing_gates",
            "intent_preview": "manual_operation_preview_only",
            "owner_approval_ref": "missing_owner_approval",
            "risk_decision_ref": "missing_risk_gate",
            "audit_evidence_ref": "missing_audit_gate",
            "disabled": true,
            "blocked_reason": "missing_owner_approval,missing_risk_gate,missing_audit_gate",
            "gates_complete": false,
            "ungated_operation_attempted": false,
            "ungated_operation_attempt_fail_closed": true,
            "attempt_status": "fail_closed_without_gates",
            "blocked_states": {
                "missing_owner_approval": true,
                "missing_risk_gate": true,
                "missing_audit_gate": true,
                "stale_read_model": false,
                "provenance_mismatch": false
            },
            "operation_controls": {
                "submit_allowed": false,
                "cancel_allowed": false,
                "retry_allowed": false,
                "replace_allowed": false,
                "amend_allowed": false,
                "flatten_allowed": false
            },
            "future_execution_version": "v0.24",
            "execution_algorithm_allowed": false,
            "production_order_submission_allowed": false,
            "production_order_mutation_allowed": false,
            "automatic_operation_action_allowed": false
        })),
        "v25_monitoring_observability": read_model_component("healthy", &json!({
            "runtime_health_status": "healthy",
            "effective_monitoring_status": "healthy",
            "monitoring_truth_scope": "runtime_monitoring_evidence_only",
            "component_count": 5,
            "slo_evidence_ref": "slo:v250:monitoring:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "live_exchange_request_allowed": false,
            "adapter_send_allowed": false,
            "automatic_remediation_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v25_alert_taxonomy_routing": read_model_component("healthy", &json!({
            "alert_status": "routed_readonly",
            "highest_severity": "warning",
            "route_status": "manual_observation_only",
            "dedupe_key": "v250:risk_fail_closed:acct-redacted-001",
            "slo_evidence_ref": "slo:v250:alert:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "automatic_actions_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v25_incident_lifecycle": read_model_component("healthy", &json!({
            "incident_lifecycle_status": "acknowledged_readonly",
            "current_state": "acknowledged",
            "ack_status": "acknowledged",
            "owner": "ops-owner-redacted",
            "incident_count": 1,
            "audit_trace": "audit:v250-incident:acknowledged",
            "slo_evidence_ref": "slo:v250:incident:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "automatic_actions_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v25_runbook_audit": read_model_component("healthy", &json!({
            "runbook_status": "manual_evidence_ready",
            "decision_type": "manual_acknowledgement",
            "decision_status": "owner_approved",
            "evidence_ref": "audit:v250-runbook:acknowledged",
            "audit_trace": "audit:v250-runbook:trace",
            "slo_evidence_ref": "slo:v250:runbook:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "automatic_actions_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v25_dr_preview_drill": read_model_component("healthy", &json!({
            "dr_preview_status": "preview_ready",
            "scenario": "read_model_rebuild_preview",
            "recovery_point": "rpo:v250:read-model-rebuild",
            "operator_approval_status": "blocked_preview",
            "snapshot_lineage": "snapshot:v250:read-model-rebuild",
            "slo_evidence_ref": "slo:v250:dr-preview:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "live_exchange_request_allowed": false,
            "adapter_send_allowed": false,
            "automatic_remediation_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v26_permission_boundary": read_model_component("healthy", &json!({
            "permission_status": "permission_evidence_ready",
            "roles_checked": "viewer,operator,release_gatekeeper,incident_owner,auditor",
            "slo_evidence_ref": "slo:v260:permission-boundary:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "submit_order_allowed": false,
            "cancel_order_allowed": false,
            "replace_order_allowed": false,
            "amend_order_allowed": false,
            "flatten_position_allowed": false,
            "order_ticket_enabled": false,
            "live_exchange_request_allowed": false,
            "adapter_send_allowed": false,
            "automatic_remediation_allowed": false,
            "automatic_actions_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v26_operation_audit": read_model_component("healthy", &json!({
            "audit_status": "immutable_chain_ready",
            "audit_lineage": "audit:v260:operation-audit:chain",
            "slo_evidence_ref": "slo:v260:operation-audit:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "automatic_actions_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v26_deployment_provenance": read_model_component("healthy", &json!({
            "deployment_status": "provenance_ready",
            "environment": "prod_like_readonly",
            "slo_evidence_ref": "slo:v260:deployment-provenance:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "live_exchange_request_allowed": false,
            "adapter_send_allowed": false,
            "automatic_remediation_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v26_upgrade_rollback": read_model_component("healthy", &json!({
            "runbook_status": "runbook_preview_ready",
            "preview_status": "preview_only_ready",
            "slo_evidence_ref": "slo:v260:upgrade-rollback:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "automatic_remediation_allowed": false,
            "automatic_actions_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        })),
        "v26_stability_slo": read_model_component("healthy", &json!({
            "stability_status": "stability_healthy",
            "degradation_reason": "none",
            "slo_evidence_ref": "slo:v260:stability:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": false,
            "operation_boundary_readonly": true,
            "forbidden_control_detected": false,
            "dashboard_trading_control_allowed": false,
            "automatic_remediation_allowed": false,
            "automatic_actions_allowed": false,
            "remediation_action_allowed": false,
            "trading_action_allowed": false
        }))
    });
    let capability_boundary = json!({
        "new_submit_capability": false,
        "production_order_submission_allowed": false,
        "production_order_mutation_allowed": false,
        "dashboard_order_controls_enabled": false,
        "dashboard_approval_controls_enabled": false,
        "dashboard_cancel_controls_enabled": false,
        "dashboard_retry_controls_enabled": false,
        "dashboard_submit_controls_enabled": false,
        "dashboard_replace_controls_enabled": false,
        "dashboard_amend_controls_enabled": false,
        "dashboard_flatten_controls_enabled": false,
        "dashboard_fill_controls_enabled": false,
        "dashboard_risk_controls_enabled": false,
        "retry_replace_amend_flatten_allowed": false,
        "trader_terminal_order_ticket_enabled": false,
        "trader_terminal_live_trading_claim": false,
        "product_grade_trading_terminal_claim": false,
        "funds_transfer_allowed": false,
        "account_configuration_mutation_allowed": false,
        "order_permission_control_allowed": false,
        "auto_flatten_position_allowed": false,
        "automatic_position_repair_allowed": false,
        "retry_order_allowed": false,
        "automatic_cancel_allowed": false,
        "automatic_order_remediation_allowed": false,
        "execution_algorithm_allowed": false,
        "automatic_fill_repair_allowed": false,
        "automatic_reconciliation_repair_allowed": false,
        "automatic_risk_action_allowed": false,
        "automatic_risk_repair_allowed": false,
        "automatic_alert_action_allowed": false,
        "automatic_audit_action_allowed": false,
        "automatic_provenance_repair_allowed": false,
        "manual_operation_entry_enabled": false,
        "manual_operation_submit_allowed": false,
        "manual_operation_cancel_allowed": false,
        "manual_operation_retry_allowed": false,
        "manual_operation_replace_allowed": false,
        "manual_operation_amend_allowed": false,
        "manual_operation_flatten_allowed": false,
        "automatic_operation_action_allowed": false
    });
    json!({
        "contract_version": UNIFIED_READ_MODEL_CONTRACT_VERSION,
        "schema_version": UNIFIED_READ_MODEL_SCHEMA_VERSION,
        "snapshot_id": "read_model.dashboard.ready.001",
        "snapshot_kind": "unified_snapshot",
        "snapshot_identity": {
            "account_id": "acct-redacted-001",
            "venue": "BINANCE",
            "instrument_ids": ["BTCUSDT.BINANCE"],
            "created_at_unix_ns": "1782918000000000000"
        },
        "as_of_unix_ns": "1782917999000000000",
        "health_status": "healthy",
        "freshness": read_model_freshness("fresh"),
        "source_provenance": read_model_source_provenance(),
        "lineage": read_model_lineage(),
        "components": components,
        "blocking_reasons": [],
        "redaction": {
            "status": "redacted",
            "raw_secret_persisted": false,
            "raw_exchange_response_persisted": false,
            "raw_account_payload_persisted": false
        },
        "capability_boundary": capability_boundary
    })
}

fn read_model_component(status: &str, data: &Value) -> Value {
    json!({
        "component_status": status,
        "source_provenance": read_model_source_provenance(),
        "lineage": read_model_lineage(),
        "freshness": read_model_freshness("fresh"),
        "redaction": {
            "status": "redacted",
            "raw_account_payload_persisted": false,
            "credential_material_persisted": false
        },
        "data": data,
        "diagnostics": []
    })
}

fn read_model_source_provenance() -> Value {
    json!({
        "source_type": "artifact",
        "source_ref": "artifact://v0_21/unified_read_model_snapshot.json",
        "captured_at_unix_ns": "1782917999000000000",
        "redaction_state": "redacted",
        "exchange_truth": false,
        "adapter_runtime_integrated": false
    })
}

fn read_model_lineage() -> Value {
    json!({
        "input_refs": ["tests/golden/read_model_dashboard_schema.jsonl"],
        "transform": "ntpro.v210.trader_terminal_readonly_dashboard.v1",
        "parent_snapshot_ids": [],
        "lossy_fields": ["raw_exchange_response", "account_object"]
    })
}

fn read_model_freshness(status: &str) -> Value {
    json!({
        "status": status,
        "observed_age_ms": 100,
        "max_age_ms": 60000,
        "as_of_unix_ns": "1782917999000000000",
        "checked_at_unix_ns": "1782918000000000000"
    })
}

fn write_production_shadow_artifacts(record: &SupervisorNodeRecord) {
    let shadow_root = record.artifact_root.join("v0_11");
    fs::create_dir_all(&shadow_root).unwrap();
    fs::write(
        shadow_root.join("account_snapshot_redacted.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v110_authenticated_account_snapshot_contract.v1",
            "status": "ready_offline_contract",
            "endpoint_class": "production_authenticated_read_only",
            "network_attempted": false,
            "account_read_attempted": false,
            "account_mutation_attempted": false,
            "order_endpoint_access_attempted": false,
            "production_order_submission_attempted": false,
            "production_order_mutation_attempted": false,
            "dashboard_order_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        shadow_root.join("shadow_execution_intent.jsonl"),
        r#"{"schema_version":"ntpro.v110_shadow_execution_intent.v1","run_id":"v110-shadow","intent_id":"intent-1","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","venue":"BINANCE","side":"buy","order_type":"market","quantity":"0.001","notional":"10.00","mode":"production_shadow","submission_allowed":false,"actual_submission":false,"submission_status":"blocked_by_v110_shadow_execution_boundary","execution_adapter_called":false,"order_endpoint_access_attempted":false,"production_order_mutation_attempted":false,"dashboard_order_controls_enabled":false}
"#,
    )
    .unwrap();
    fs::write(
        shadow_root.join("shadow_portfolio_snapshot.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v110_shadow_portfolio_snapshot.v1",
            "snapshot_mode": "production_readonly_shadow",
            "balances": [
                {
                    "asset": "USDT",
                    "free": "100.00",
                    "locked": "0.00",
                    "source": "production_readonly_account_snapshot",
                    "confidence": "observed"
                }
            ],
            "positions": [
                {
                    "instrument_id": "BTCUSDT.BINANCE",
                    "quantity": "0",
                    "average_price": null,
                    "source": "unavailable",
                    "status": "unavailable",
                    "reason": "no live fills in v0.11"
                }
            ],
            "exposure": {
                "asset": "BTC",
                "gross": null,
                "net": null,
                "notional": null,
                "quote_currency": "USDT",
                "status": "unavailable",
                "reason": "no production fills in v0.11"
            },
            "pnl": {
                "realized": null,
                "unrealized": null,
                "quote_currency": "USDT",
                "status": "unavailable",
                "reason": "no cost basis in v0.11"
            },
            "actual_submission_count": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "automatic_correction_orders_submitted": 0,
            "dashboard_order_controls_enabled": false,
            "full_production_portfolio_parity_claimed": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        shadow_root.join("order_lifecycle_state.jsonl"),
        r#"{"schema_version":"ntpro.v110_order_lifecycle_state.v1","run_id":"v110-shadow","lifecycle_event_id":"life-1","intent_id":"intent-1","previous_state":"PreflightPassed","next_state":"ShadowSubmitted","reason":"local shadow ledger only","actual_submission":false,"actual_submission_count":0,"production_orders_submitted":0,"production_order_mutations_attempted":0,"exchange_order_id_recorded":false,"venue_order_id_recorded":false,"dashboard_order_controls_enabled":false}
"#,
    )
    .unwrap();
    fs::write(
        shadow_root.join("reconciliation_events.jsonl"),
        r#"{"schema_version":"ntpro.v110_reconciliation_event.v1","run_id":"v110-shadow","event_id":"recon-1","event_type":"shadow_mismatch","severity":"warning","recommended_action":"manual_review_required","automatic_correction_orders_submitted":0,"production_orders_submitted":0,"production_order_mutations_attempted":0,"cancel_replace_amend_attempted":false,"dashboard_order_controls_enabled":false}
"#,
    )
    .unwrap();
    write_production_shadow_manifest(&shadow_root);
}

fn write_production_shadow_v12_artifacts(record: &SupervisorNodeRecord) {
    let shadow_root = record.artifact_root.join("v0_12");
    fs::create_dir_all(&shadow_root).unwrap();
    fs::write(
        shadow_root.join("production_public_online_read_probe.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v120_production_public_online_read_probe.v1",
            "status": "online_read_probe_ok",
            "endpoint": "server_time",
            "endpoint_class": "production_public_read_only",
            "method": "GET",
            "endpoint_read_allowed": true,
            "offline_contract_ready": false,
            "read_allowed": false,
            "contract_ready": false,
            "online_read_allowed": true,
            "network_attempted": true,
            "response_shape": "binance_server_time_v1",
            "response_shape_validated": true,
            "account_mutation_attempted": false,
            "production_order_submission_attempted": false,
            "production_order_mutation_attempted": false,
            "dashboard_order_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        shadow_root.join("production_account_snapshot_redacted.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v120_authenticated_account_snapshot_online_read.v1",
            "status": "online_account_snapshot_ok",
            "endpoint_class": "production_authenticated_read_only",
            "method": "GET",
            "endpoint_read_allowed": true,
            "offline_contract_ready": false,
            "read_allowed": false,
            "contract_ready": false,
            "online_read_allowed": true,
            "network_attempted": true,
            "account_read_attempted": true,
            "account_mutation_attempted": false,
            "order_endpoint_access_attempted": false,
            "production_order_submission_attempted": false,
            "production_order_mutation_attempted": false,
            "dashboard_order_controls_enabled": false,
            "response_shape": "binance_account_snapshot_v1",
            "response_shape_validated": true,
            "response_shape_summary": {
                "status": "accepted",
                "shape_validated": true,
                "balance_entry_count": 1,
                "raw_account_response_recorded": false,
                "raw_balances_recorded": false,
                "raw_permissions_recorded": false
            },
            "secrets_redacted": true
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        shadow_root.join("shadow_execution_intent.jsonl"),
        r#"{"schema_version":"ntpro.v110_shadow_execution_intent.v1","run_id":"v120-shadow","intent_id":"intent-1","strategy_id":"ema_cross_btcusdt_v1","symbol":"BTCUSDT.BINANCE","venue":"BINANCE","side":"buy","order_type":"market","quantity":"0.001","notional":"10.00","mode":"production_shadow","submission_allowed":false,"actual_submission":false,"submission_status":"blocked_by_v110_shadow_execution_boundary","execution_adapter_called":false,"order_endpoint_access_attempted":false,"production_order_mutation_attempted":false,"dashboard_order_controls_enabled":false}
"#,
    )
    .unwrap();
    fs::write(
        shadow_root.join("shadow_portfolio_runtime.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v120_shadow_portfolio_runtime.v1",
            "status": "ready_redacted_shadow_portfolio",
            "run_id": "v120-shadow",
            "snapshot_id": "v120-shadow-portfolio",
            "snapshot_mode": "production_readonly_shadow",
            "source_account_snapshot_ref": {
                "path": shadow_root.join("production_account_snapshot_redacted.json").display().to_string(),
                "schema_version": "ntpro.v120_authenticated_account_snapshot_online_read.v1",
                "status": "online_account_snapshot_ok",
                "response_shape_validated": true,
                "raw_payload_recorded": false
            },
            "source_shadow_intent_refs": [
                {
                    "intent_id": "intent-1",
                    "symbol": "BTCUSDT.BINANCE",
                    "venue": "BINANCE",
                    "side": "buy",
                    "quantity": "0.001",
                    "notional": "10.00",
                    "submission_status": "blocked_by_v110_shadow_execution_boundary",
                    "actual_submission": false
                }
            ],
            "balances": {
                "status": "observed_shape_only",
                "source": "redacted_production_account_snapshot_shape",
                "confidence": "observed_shape_only",
                "observed_balance_entry_count": 1,
                "asset_values_recorded": false,
                "free_values_recorded": false,
                "locked_values_recorded": false,
                "reason": "production account response shape was validated; values remain redacted"
            },
            "positions": [],
            "exposure": {
                "asset": null,
                "gross": null,
                "net": null,
                "notional": "10",
                "quote_currency": "USDT",
                "status": "derived_from_shadow_intents",
                "reason": "derived from local shadow intent notional only; this is not exchange truth"
            },
            "pnl": {
                "realized": null,
                "unrealized": null,
                "quote_currency": "USDT",
                "status": "unavailable",
                "reason": "production fills, cost basis, and mark prices are not available"
            },
            "risk_summary": {
                "status": "risk_halted",
                "new_orders_blocked": true,
                "risk_halted": true,
                "reason": "read-only shadow evidence cannot unlock production orders"
            },
            "provenance": {
                "values_are_exchange_truth": false
            },
            "shadow_intents_created": 1,
            "actual_submission_count": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "automatic_correction_orders_submitted": 0,
            "dashboard_order_controls_enabled": false,
            "full_production_portfolio_parity_claimed": false,
            "network_attempted": true,
            "real_orders_submitted": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        shadow_root.join("shadow_strategy_session.jsonl"),
        r#"{"schema_version":"ntpro.v120_shadow_strategy_session_event.v1","run_id":"v120-shadow","session_id":"v120-shadow-session","strategy_id":"ema_cross_btcusdt_v1","event_type":"shadow_strategy_session_started","state":"running","occurred_at":"unix:1","shadow_portfolio_runtime_ref":{"path":"shadow_portfolio_runtime.json","schema_version":"ntpro.v120_shadow_portfolio_runtime.v1","status":"ready_redacted_shadow_portfolio","snapshot_id":"v120-shadow-portfolio","exposure_status":"derived_from_shadow_intents","pnl_status":"unavailable","risk_status":"risk_halted","shadow_intents_created":1,"network_attempted":true,"values_are_exchange_truth":false},"production_order_submissions_attempted":0,"production_orders_submitted":0,"production_order_mutations_attempted":0,"production_order_state_reads_attempted":0,"listen_key_lifecycle_attempted":0,"actual_submission_count":0,"automatic_correction_orders_submitted":0,"dashboard_order_controls_enabled":false,"real_orders_submitted":false,"values_are_exchange_truth":false}
{"schema_version":"ntpro.v120_shadow_strategy_session_event.v1","run_id":"v120-shadow","session_id":"v120-shadow-session","strategy_id":"ema_cross_btcusdt_v1","event_type":"shadow_strategy_session_heartbeat","state":"running","heartbeat_seq":1,"occurred_at":"unix:2","shadow_portfolio_runtime_ref":{"path":"shadow_portfolio_runtime.json","schema_version":"ntpro.v120_shadow_portfolio_runtime.v1","status":"ready_redacted_shadow_portfolio","snapshot_id":"v120-shadow-portfolio","exposure_status":"derived_from_shadow_intents","pnl_status":"unavailable","risk_status":"risk_halted","shadow_intents_created":1,"network_attempted":true,"values_are_exchange_truth":false},"production_order_submissions_attempted":0,"production_orders_submitted":0,"production_order_mutations_attempted":0,"production_order_state_reads_attempted":0,"listen_key_lifecycle_attempted":0,"actual_submission_count":0,"automatic_correction_orders_submitted":0,"dashboard_order_controls_enabled":false,"real_orders_submitted":false,"values_are_exchange_truth":false}
{"schema_version":"ntpro.v120_shadow_strategy_session_event.v1","run_id":"v120-shadow","session_id":"v120-shadow-session","strategy_id":"ema_cross_btcusdt_v1","event_type":"shadow_strategy_session_heartbeat","state":"running","heartbeat_seq":2,"occurred_at":"unix:3","shadow_portfolio_runtime_ref":{"path":"shadow_portfolio_runtime.json","schema_version":"ntpro.v120_shadow_portfolio_runtime.v1","status":"ready_redacted_shadow_portfolio","snapshot_id":"v120-shadow-portfolio","exposure_status":"derived_from_shadow_intents","pnl_status":"unavailable","risk_status":"risk_halted","shadow_intents_created":1,"network_attempted":true,"values_are_exchange_truth":false},"production_order_submissions_attempted":0,"production_orders_submitted":0,"production_order_mutations_attempted":0,"production_order_state_reads_attempted":0,"listen_key_lifecycle_attempted":0,"actual_submission_count":0,"automatic_correction_orders_submitted":0,"dashboard_order_controls_enabled":false,"real_orders_submitted":false,"values_are_exchange_truth":false}
{"schema_version":"ntpro.v120_shadow_strategy_session_event.v1","run_id":"v120-shadow","session_id":"v120-shadow-session","strategy_id":"ema_cross_btcusdt_v1","event_type":"shadow_strategy_session_stopped","state":"stopped","occurred_at":"unix:4","shadow_portfolio_runtime_ref":{"path":"shadow_portfolio_runtime.json","schema_version":"ntpro.v120_shadow_portfolio_runtime.v1","status":"ready_redacted_shadow_portfolio","snapshot_id":"v120-shadow-portfolio","exposure_status":"derived_from_shadow_intents","pnl_status":"unavailable","risk_status":"risk_halted","shadow_intents_created":1,"network_attempted":true,"values_are_exchange_truth":false},"production_order_submissions_attempted":0,"production_orders_submitted":0,"production_order_mutations_attempted":0,"production_order_state_reads_attempted":0,"listen_key_lifecycle_attempted":0,"actual_submission_count":0,"automatic_correction_orders_submitted":0,"dashboard_order_controls_enabled":false,"real_orders_submitted":false,"values_are_exchange_truth":false}
"#,
    )
    .unwrap();
    fs::write(
        shadow_root.join("reconciliation_events.jsonl"),
        r#"{"schema_version":"ntpro.v120_readonly_reconciliation_event.v1","run_id":"v120-shadow","event_id":"v120-shadow:ok","event_type":"observed_account_state","classification":"ok","severity":"info","observed_at":"unix:5","source_ref":{"engine":"production_readonly_reconciliation","mode":"local_shadow_artifact_classification","network_attempted":false},"recommended_action":"record_only","risk_halted":false,"new_orders_blocked":true,"manual_review_required":false,"automatic_correction_orders_submitted":0,"production_order_submissions_attempted":0,"production_orders_submitted":0,"production_order_mutations_attempted":0,"production_order_state_reads_attempted":0,"listen_key_lifecycle_attempted":0,"cancel_replace_amend_attempted":false,"dashboard_order_controls_enabled":false,"real_orders_submitted":false,"values_are_exchange_truth":false,"diagnostic":"read-only reconciliation classified local shadow evidence as ok; record only"}
"#,
    )
    .unwrap();
}

fn write_production_shadow_v13_kill_switch_artifact(record: &SupervisorNodeRecord) {
    let shadow_root = record.artifact_root.join("v0_13");
    fs::create_dir_all(&shadow_root).unwrap();
    fs::write(
        shadow_root.join("kill_switch_approval_artifact.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_KILL_SWITCH_APPROVAL_ARTIFACT_SCHEMA_VERSION,
            "status": "manual_approval_recorded",
            "kill_switch_active": true,
            "kill_switch_dry_run": true,
            "manual_approval_recorded": true,
            "manual_approval_required": true,
            "approval_state": "approved",
            "owner_approval_required_before_any_mutation": true,
            "new_submit_capability": false,
            "production_order_submission_allowed": false,
            "production_order_mutation_allowed": false,
            "production_order_state_reads_allowed": false,
            "listen_key_lifecycle_allowed": false,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "production_order_state_reads_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "actual_submission_count": 0,
            "automatic_correction_orders_submitted": 0,
            "cancel_replace_amend_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false,
            "network_attempted": false,
            "values_are_exchange_truth": false
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_live_alpha_dry_run_artifacts(record: &SupervisorNodeRecord) {
    let root = record.artifact_root.join("v0_14");
    fs::create_dir_all(&root).unwrap();
    let order_gate_path = root.join("live_alpha_dry_run_order_gate.json");
    let order_gate_json = r#"{
  "schema_version": "__ORDER_GATE_SCHEMA__",
  "run_id": "v140-live-alpha-dry-run",
  "session_id": "v140-live-alpha-dry-run",
  "strategy_id": "ema_cross_btcusdt_v1",
  "artifact_type": "live_alpha_dry_run_order_gate",
  "status": "ready_dry_run_no_submission",
  "created_at": "unix_ms:1000",
  "mode": "production_live_alpha_dry_run",
  "symbol": "BTCUSDT",
  "side": "BUY",
  "order_type": "MARKET",
  "quantity": "0.001",
  "notional": "10.00",
  "owner_gate_required": true,
  "manual_gate_required": true,
  "missing_cli_flags": [],
  "dry_run_order_intent_recorded": true,
  "dry_run_order_gate_ready": true,
  "order_submission_mode": "dry_run_no_submission",
  "production_order_submission_allowed": false,
  "production_order_mutation_allowed": false,
  "production_order_state_reads_allowed": false,
  "listen_key_lifecycle_allowed": false,
  "production_order_submissions_attempted": 0,
  "production_orders_submitted": 0,
  "production_order_mutations_attempted": 0,
  "production_order_state_reads_attempted": 0,
  "listen_key_lifecycle_attempted": 0,
  "cancel_replace_amend_attempted": false,
  "order_endpoint_access_attempted": false,
  "execution_adapter_called": false,
  "matching_engine_submission": false,
  "actual_submission_count": 0,
  "automatic_correction_orders_submitted": 0,
  "dashboard_order_controls_enabled": false,
  "external_venue_connection": false,
  "network_attempted": false,
  "real_orders_submitted": false,
  "real_funds": false,
  "production_trading_enabled": false,
  "values_are_exchange_truth": false,
  "no_production_order_submission_confirmed": true,
  "no_production_order_mutation_confirmed": true,
  "no_execution_adapter_call_confirmed": true,
  "no_listen_key_lifecycle_confirmed": true,
  "dashboard_controls_disabled_confirmed": true,
  "no_real_funds_confirmed": true,
  "diagnostic": "local live-alpha dry-run order gate is ready; no production order was submitted or mutated"
}
"#
    .replace(
        "__ORDER_GATE_SCHEMA__",
        LIVE_ALPHA_DRY_RUN_ORDER_GATE_SCHEMA_VERSION,
    );
    fs::write(&order_gate_path, order_gate_json).unwrap();

    let order_gate_path_json = serde_json::to_string(&order_gate_path.display().to_string())
        .expect("serialize order gate path");
    let risk_preflight_json = r#"{
  "schema_version": "__RISK_PREFLIGHT_SCHEMA__",
  "status": "approved",
  "run_id": "v140-live-alpha-dry-run",
  "evaluated_at": "unix_ms:1100",
  "risk_decision": "dry_run_approved",
  "execution_decision": "blocked_no_production_mutation",
  "reasons": [],
  "missing_cli_flags": [],
  "order_gate_status": "ready_dry_run_no_submission",
  "order_gate_ready": true,
  "order_gate_path": __ORDER_GATE_PATH__,
  "session_state": "running",
  "symbol": "BTCUSDT",
  "side": "BUY",
  "order_type": "MARKET",
  "quantity": "0.001",
  "notional": "10.00",
  "max_order_notional": "25.00",
  "current_position_notional": "50.00",
  "projected_position_notional": "60",
  "max_position_notional": "100.00",
  "market_age_ms": 500,
  "max_market_age_ms": 1000,
  "account_readable": true,
  "order_state_readable": true,
  "order_state_age_ms": 100,
  "max_order_state_age_ms": 1000,
  "open_order_count": 0,
  "max_open_orders": 5,
  "observed_clock_skew_ms": 25,
  "max_clock_skew_ms": 100,
  "kill_switch_active": false,
  "production_order_submission_allowed": false,
  "production_order_mutation_allowed": false,
  "production_order_state_reads_allowed": false,
  "listen_key_lifecycle_allowed": false,
  "production_order_submissions_attempted": 0,
  "production_orders_submitted": 0,
  "production_order_mutations_attempted": 0,
  "production_order_state_reads_attempted": 0,
  "listen_key_lifecycle_attempted": 0,
  "cancel_replace_amend_attempted": false,
  "order_endpoint_access_attempted": false,
  "execution_adapter_called": false,
  "matching_engine_submission": false,
  "actual_submission_count": 0,
  "automatic_correction_orders_submitted": 0,
  "dashboard_order_controls_enabled": false,
  "external_venue_connection": false,
  "network_attempted": false,
  "real_orders_submitted": false,
  "real_funds": false,
  "production_trading_enabled": false,
  "values_are_exchange_truth": false,
  "diagnostic": "hypothetical live-alpha order passed local risk preflight; execution remains disabled"
}
"#
    .replace(
        "__RISK_PREFLIGHT_SCHEMA__",
        LIVE_ALPHA_RISK_PREFLIGHT_SCHEMA_VERSION,
    )
    .replace("__ORDER_GATE_PATH__", &order_gate_path_json);
    fs::write(
        root.join("live_alpha_risk_preflight.json"),
        risk_preflight_json,
    )
    .unwrap();
}

fn write_live_alpha_v15_mutation_preflight_artifacts(record: &SupervisorNodeRecord) {
    let root = record.artifact_root.join("v0_15");
    fs::create_dir_all(&root).unwrap();
    let manual_approval_json = r#"{
  "schema_version": "__MANUAL_APPROVAL_SCHEMA__",
  "status": "approval_valid_for_dry_run_request_preview",
  "run_id": "v150-live-alpha-request-preview",
  "strategy_id": "ema_cross_btcusdt_v1",
  "symbol": "BTCUSDT",
  "notional": "10.00",
  "approval_state": "approved",
  "manual_approval_recorded": true,
  "manual_approval_id": "owner-approval-v150-008",
  "approved_by": "owner",
  "approval_lifecycle_valid": true,
  "approval_lifecycle_issues": [],
  "one_time_approval": true,
  "approval_used": false,
  "now_unix_ms": 1718400000000,
  "expires_at_unix_ms": 1718400060000,
  "dry_run_request_preview_only": true,
  "production_order_submission_allowed": false,
  "production_order_mutation_allowed": false,
  "production_order_submissions_attempted": 0,
  "production_orders_submitted": 0,
  "production_order_mutations_attempted": 0,
  "listen_key_lifecycle_attempted": 0,
  "dashboard_order_controls_enabled": false,
  "network_attempted": false,
  "real_orders_submitted": false,
  "real_funds": false,
  "production_trading_enabled": false,
  "values_are_exchange_truth": false
}
"#
    .replace(
        "__MANUAL_APPROVAL_SCHEMA__",
        LIVE_ALPHA_MANUAL_APPROVAL_LIFECYCLE_SCHEMA_VERSION,
    );
    fs::write(
        root.join("manual_approval_lifecycle.json"),
        manual_approval_json,
    )
    .unwrap();

    let request_preview_json = r#"{
  "schema_version": "__REQUEST_PREVIEW_SCHEMA__",
  "status": "ready_request_preview_only",
  "run_id": "v150-live-alpha-request-preview",
  "artifact_type": "live_alpha_order_request_preview",
  "endpoint_class": "production_mutation_owner_approved_manual_only",
  "endpoint_decision": "allow_request_preview_only",
  "request_method": "POST",
  "request_target": "/api/v3/order",
  "query_shape_without_signature": "symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp",
  "signature_preflight": "created_in_memory_not_recorded",
  "api_key_header_value_recorded": false,
  "api_secret_value_recorded": false,
  "signature_recorded": false,
  "signed_query_recorded": false,
  "signed_url_recorded": false,
  "request_body_recorded": false,
  "raw_request_body_recorded": false,
  "manual_approval_lifecycle_status": "approval_valid_for_dry_run_request_preview",
  "manual_approval_lifecycle_state": "approved",
  "manual_approval_lifecycle_valid": true,
  "manual_approval_lifecycle_issues": [],
  "manual_approval_one_time": true,
  "manual_approval_used": false,
  "manual_approval_expires_at_unix_ms": 1718400060000,
  "order_gate_ready": true,
  "request_preview_allowed": true,
  "request_preview_built": true,
  "request_sent": false,
  "production_order_submission_allowed": false,
  "production_order_mutation_allowed": false,
  "production_order_submissions_attempted": 0,
  "production_orders_submitted": 0,
  "production_order_mutations_attempted": 0,
  "order_endpoint_access_attempted": false,
  "execution_adapter_called": false,
  "production_adapter_called": false,
  "network_attempted": false,
  "dashboard_order_controls_enabled": false,
  "real_orders_submitted": false,
  "real_funds": false,
  "production_trading_enabled": false,
  "signed_request_memory_only": true,
  "secrets_redacted": true,
  "values_are_exchange_truth": false
}
"#
    .replace(
        "__REQUEST_PREVIEW_SCHEMA__",
        LIVE_ALPHA_ORDER_REQUEST_PREVIEW_SCHEMA_VERSION,
    );
    fs::write(
        root.join("live_alpha_order_request_preview.json"),
        request_preview_json,
    )
    .unwrap();

    let runtime_gate_json = r#"{
  "schema_version": "__RUNTIME_GATE_SCHEMA__",
  "status": "ready_runtime_gate_open_for_dry_run_only",
  "runtime_gate_decision": "dry_run_runtime_gate_open",
  "runtime_gate_open": true,
  "runtime_gate_reasons": [],
  "kill_switch_active": false,
  "manual_approval_recorded": true,
  "approval_state": "approved",
  "manual_approval_id": "owner-approval-v150-008",
  "request_preview_status": "ready_request_preview_only",
  "request_preview_built": true,
  "request_sent": false,
  "production_order_submission_allowed": false,
  "production_order_mutation_allowed": false,
  "production_order_submissions_attempted": 0,
  "production_orders_submitted": 0,
  "production_order_mutations_attempted": 0,
  "dashboard_order_controls_enabled": false,
  "network_attempted": false,
  "real_orders_submitted": false,
  "real_funds": false,
  "production_trading_enabled": false,
  "values_are_exchange_truth": false
}
"#
    .replace(
        "__RUNTIME_GATE_SCHEMA__",
        LIVE_ALPHA_KILL_SWITCH_RUNTIME_GATE_SCHEMA_VERSION,
    );
    fs::write(
        root.join("kill_switch_runtime_gate.json"),
        runtime_gate_json,
    )
    .unwrap();

    let execution_dry_run_json = r#"{
  "schema_version": "__EXECUTION_DRY_RUN_SCHEMA__",
  "status": "ready_dry_run_execution_adapter_only",
  "execution_decision": "dry_run_adapter_artifact_only",
  "dry_run_execution_adapter_called": true,
  "dry_run_execution_adapter_wrote_artifact": true,
  "dry_run_adapter_artifact_only": true,
  "real_execution_adapter_called": false,
  "production_adapter_instantiated": false,
  "production_adapter_called": false,
  "strategy_intent_recorded": true,
  "strategy_intent_reaches_risk_preflight": true,
  "strategy_intent_reaches_dry_run_adapter": true,
  "strategy_intent_reaches_production_adapter": false,
  "source_artifact_issues": [],
  "missing_cli_flags": [],
  "order_gate_ready": true,
  "risk_preflight_decision": "dry_run_approved",
  "request_preview_built": true,
  "request_sent": false,
  "kill_switch_runtime_gate_status": "ready_runtime_gate_open_for_dry_run_only",
  "kill_switch_runtime_gate_open": true,
  "production_order_submission_allowed": false,
  "production_order_mutation_allowed": false,
  "production_order_submissions_attempted": 0,
  "production_orders_submitted": 0,
  "production_order_mutations_attempted": 0,
  "order_endpoint_access_attempted": false,
  "network_attempted": false,
  "dashboard_order_controls_enabled": false,
  "real_orders_submitted": false,
  "real_funds": false,
  "production_trading_enabled": false,
  "values_are_exchange_truth": false
}
"#
    .replace(
        "__EXECUTION_DRY_RUN_SCHEMA__",
        LIVE_ALPHA_EXECUTION_DRY_RUN_SCHEMA_VERSION,
    );
    fs::write(
        root.join("live_alpha_execution_dry_run.json"),
        execution_dry_run_json,
    )
    .unwrap();
}

fn write_production_mutation_v16_evidence_artifacts(record: &SupervisorNodeRecord) {
    let root = record.artifact_root.join("v0_16");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("production_mutation_runtime_gate.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_RUNTIME_GATE_SCHEMA_VERSION,
            "status": "blocked_explicit_send_gate",
            "runtime_gate_open": false,
            "send_consideration_allowed": false,
            "request_sent": false,
            "network_attempted": false,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "production_order_state_reads_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_signing_approval.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_SIGNING_APPROVAL_SCHEMA_VERSION,
            "status": "ready_signing_material_approval",
            "signing_approval_ready": true,
            "approval_state": "approved",
            "manual_approval_recorded": true,
            "manual_approval_id": "owner-approval-v160-003",
            "approved_by": "owner",
            "request_sent": false,
            "network_attempted": false,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "retry_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_request_builder.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_REQUEST_BUILDER_SCHEMA_VERSION,
            "status": "ready_request_object_built_no_send",
            "request_builder_ready": true,
            "request_object_built": true,
            "runtime_gate_status": "blocked_explicit_send_gate",
            "runtime_gate_open": false,
            "signing_approval_status": "ready_signing_material_approval",
            "signing_approval_ready": true,
            "symbol": "BTCUSDT",
            "side": "BUY",
            "order_type": "LIMIT",
            "quantity": "0.001",
            "price": "10000.00",
            "time_in_force": "GTC",
            "notional": "10.00",
            "request_sent": false,
            "network_attempted": false,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "production_order_state_reads_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_guarded_send.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_GUARDED_SEND_SCHEMA_VERSION,
            "status": "ready_guarded_send_path_offline_no_network",
            "guarded_send_ready": true,
            "request_sent": false,
            "network_attempted": false,
            "kill_switch_checked_before_send": true,
            "kill_switch_checked_after_send": true,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "production_order_state_reads_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_response_redaction.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_RESPONSE_REDACTION_SCHEMA_VERSION,
            "status": "ready_response_redacted",
            "response_redaction_ready": true,
            "symbol": "BTCUSDT",
            "side": "BUY",
            "order_id": "123456789",
            "client_order_id": "owner-approved-v160-single-shot",
            "exchange_status": "NEW",
            "request_sent": false,
            "network_attempted": false,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "production_order_state_reads_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_order_state_readback.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_ORDER_STATE_READBACK_SCHEMA_VERSION,
            "status": "ready_offline_order_state_readback_contract",
            "readback_contract_ready": true,
            "order_state_read_attempted": false,
            "response_shape_validated": true,
            "endpoint_shape_validated": true,
            "request_sent": false,
            "network_attempted": false,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "production_order_state_reads_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_audit_trail.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_AUDIT_TRAIL_SCHEMA_VERSION,
            "status": "ready_redacted_audit_trail",
            "audit_trail_ready": true,
            "signing_approval_status": "ready_signing_material_approval",
            "approval_state": "approved",
            "manual_approval_recorded": true,
            "approved_by": "owner",
            "runtime_gate_status": "blocked_explicit_send_gate",
            "runtime_gate_open": false,
            "guarded_send_status": "ready_guarded_send_path_offline_no_network",
            "response_redaction_status": "ready_response_redacted",
            "response_redaction_ready": true,
            "order_state_readback_status": "ready_offline_order_state_readback_contract",
            "readback_contract_ready": true,
            "order_state_read_attempted": false,
            "kill_switch_checked_before_send": true,
            "kill_switch_checked_after_send": true,
            "failure_state": "none_recorded",
            "symbol": "BTCUSDT",
            "side": "BUY",
            "order_type": "LIMIT",
            "time_in_force": "GTC",
            "quantity": "0.001",
            "price": "10000.00",
            "order_id": "123456789",
            "request_sent": false,
            "network_attempted": false,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "production_order_state_reads_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_failure_semantics.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_FAILURE_SEMANTICS_SCHEMA_VERSION,
            "status": "ready_failure_semantics_evidence",
            "failure_semantics_ready": true,
            "failure_mode": "timeout",
            "terminal_action": "write_evidence_and_stop",
            "strategy_continuation_allowed": false,
            "request_sent": false,
            "network_attempted": false,
            "production_order_submissions_attempted": 0,
            "production_orders_submitted": 0,
            "production_order_mutations_attempted": 0,
            "production_order_state_reads_attempted": 0,
            "listen_key_lifecycle_attempted": 0,
            "retry_attempted": false,
            "cancel_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "correction_attempted": false,
            "flatten_attempted": false,
            "remediation_attempted": false,
            "dashboard_order_controls_enabled": false,
            "real_orders_submitted": false,
            "production_trading_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_production_mutation_v17_reconciliation_orphan_artifacts(record: &SupervisorNodeRecord) {
    let root = record.artifact_root.join("v0_17");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("production_mutation_local_order_ledger.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_LOCAL_ORDER_LEDGER_SCHEMA_VERSION,
            "run_id": "v170-production-reconciliation-dashboard",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "production_mutation_local_order_ledger",
            "status": "ready_local_order_ledger",
            "local_ledger_ready": true,
            "restart_readable": true,
            "current_local_state": "local_ledger_pending_exchange_reconciliation",
            "request_sent": true,
            "network_attempted": false,
            "production_order_submission_allowed": false,
            "production_order_mutation_allowed": false,
            "duplicate_submit_attempted": false,
            "retry_attempted": false,
            "cancel_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_exchange_readback_mapper.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_EXCHANGE_READBACK_MAPPER_SCHEMA_VERSION,
            "run_id": "v170-production-reconciliation-dashboard",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "production_mutation_exchange_readback_mapper",
            "status": "ready_exchange_readback_mapped",
            "local_ledger_ready": true,
            "exchange_readback_mapped": true,
            "exchange_order_status": "NEW",
            "exchange_order_state": "open",
            "open_order_observed": true,
            "terminal_state_observed": false,
            "order_found": true,
            "request_sent": true,
            "network_attempted": false,
            "production_order_submission_allowed": false,
            "production_order_mutation_allowed": false,
            "duplicate_submit_attempted": false,
            "retry_attempted": false,
            "cancel_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false,
            "dashboard_auto_approval_allowed": false,
            "dashboard_auto_approval_attempted": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_reconciliation_classifier.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_RECONCILIATION_CLASSIFIER_SCHEMA_VERSION,
            "run_id": "v170-production-reconciliation-dashboard",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "production_mutation_reconciliation_classifier",
            "status": "ready_reconciliation_classified",
            "exchange_readback_mapped": true,
            "reconciliation_classified": true,
            "reconciliation_outcome": "local_sent_exchange_new",
            "orphan_risk_detected": false,
            "manual_review_required": true,
            "new_orders_blocked": true,
            "request_sent": true,
            "network_attempted": false,
            "production_order_submission_allowed": false,
            "production_order_mutation_allowed": false,
            "duplicate_submit_attempted": false,
            "retry_attempted": false,
            "cancel_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("production_mutation_orphan_order_detector.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_ORPHAN_ORDER_DETECTOR_SCHEMA_VERSION,
            "run_id": "v170-production-reconciliation-dashboard",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "production_mutation_orphan_order_detector",
            "status": "ready_orphan_order_detection_completed",
            "orphan_detection_completed": true,
            "orphan_detection_outcome": "open_orphan_risk",
            "orphan_risk_detected": true,
            "risk_halted": true,
            "manual_review_required": true,
            "new_orders_blocked": true,
            "stale_ledger_restart_required": false,
            "local_terminal_state": false,
            "request_sent": true,
            "network_attempted": false,
            "production_order_submission_allowed": false,
            "production_order_mutation_allowed": false,
            "duplicate_submit_attempted": false,
            "retry_attempted": false,
            "cancel_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_production_actual_cancel_audit_v19_artifacts(record: &SupervisorNodeRecord) {
    let v18_root = record.artifact_root.join("v0_18");
    let v19_root = record.artifact_root.join("v0_19");
    fs::create_dir_all(&v18_root).unwrap();
    fs::create_dir_all(&v19_root).unwrap();
    fs::write(
        v18_root.join("cancel_risk_gate.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION,
            "run_id": "v190-dashboard-actual-cancel-audit",
            "order_lineage_id": "lineage-v190-actual-cancel",
            "artifact_type": "cancel_risk_gate",
            "status": "ready_owner_approval_required",
            "cancel_risk_gate_ready": true,
            "risk_gate_ready": true,
            "risk_gate_result": "ready_owner_approval_required",
            "owner_approval_required": true,
            "actual_cancel_send_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        v19_root.join("actual_cancel_owner_approval_lifecycle.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_ACTUAL_CANCEL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION,
            "run_id": "v190-dashboard-actual-cancel-audit",
            "order_lineage_id": "lineage-v190-actual-cancel",
            "artifact_type": "actual_cancel_owner_approval_lifecycle",
            "status": "ready_actual_cancel_owner_approval_lifecycle",
            "approval_state": "approved",
            "approval_lifecycle_valid": true,
            "approval_execution_authorized": true,
            "approval_reusable": false,
            "one_order_one_venue_one_attempt": true,
            "actual_cancel_send_allowed": false,
            "cancel_attempted": false,
            "cancel_requests_sent": 0,
            "network_attempted": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    let single_shot_json = r#"{
  "schema_version": "__SINGLE_SHOT_SCHEMA__",
  "run_id": "v190-dashboard-actual-cancel-audit",
  "order_lineage_id": "lineage-v190-actual-cancel",
  "artifact_type": "actual_cancel_single_shot",
  "status": "actual_cancel_attempt_recorded",
  "mode": "online",
  "execution_mode": "single_shot",
  "manual_online_requested": true,
  "actual_cancel_command_ready": true,
  "single_shot_cancel_allowed": true,
  "owner_approval_ready": true,
  "risk_gate_ready": true,
  "release_provenance_ready": true,
  "adapter_boundary_ready": true,
  "adapter_capability_ready": true,
  "approval_state_before_attempt": "approved",
  "approval_state_after_attempt": "consumed",
  "request_id": "actual-cancel-request-001",
  "request_method": "DELETE",
  "request_target": "/api/v3/order",
  "venue": "binance_spot",
  "order_id_type": "order_id",
  "known_order_id": "123456789",
  "known_client_order_id": "owner-approved-v190-single-shot",
  "symbol": "BTCUSDT",
  "account_label": "prod-account-redacted",
  "venue_response_status": "accepted",
  "venue_response_source": "redacted_exchange_metadata",
  "venue_response_code": 200,
  "venue_response_error_code": "none",
  "local_audit_reference": "audit://v190/actual-cancel/request-001",
  "readback_required": true,
  "readback_requirement": "post_cancel_readback_required",
  "source_artifact_issues": [],
  "adapter_capability_issues": [],
  "safety_contract_issues": [],
  "release_manifest_issues": [],
  "missing_cli_flags": [],
  "missing_env_vars": [],
  "request_sent": true,
  "cancel_attempted": true,
  "cancel_requests_sent": 1,
  "production_order_mutations_attempted": 1,
  "network_attempted": true,
  "network_cancel_endpoint_attempted": true,
  "http_send_attempted": true,
  "venue_ack_observed": true,
  "retry_attempted": false,
  "replace_attempted": false,
  "amend_attempted": false,
  "flatten_attempted": false,
  "remediation_attempted": false,
  "automatic_cancel_allowed": false,
  "automatic_remediation_allowed": false,
  "bulk_cancel_allowed": false,
  "cancel_all_allowed": false,
  "multi_account_cancel_allowed": false,
  "multi_strategy_cancel_allowed": false,
  "multi_venue_cancel_allowed": false,
  "dashboard_order_controls_enabled": false,
  "dashboard_cancel_controls_enabled": false,
  "dashboard_execution_allowed": false,
  "raw_exchange_response_recorded": false,
  "response_body_recorded": false,
  "response_headers_recorded": false,
  "signature_recorded": false,
  "signed_query_recorded": false,
  "signed_url_recorded": false,
  "api_key_value_recorded": false,
  "api_secret_value_recorded": false
}
"#
    .replace(
        "__SINGLE_SHOT_SCHEMA__",
        PRODUCTION_MUTATION_ACTUAL_CANCEL_SINGLE_SHOT_SCHEMA_VERSION,
    );
    fs::write(
        v19_root.join("actual_cancel_single_shot.json"),
        single_shot_json,
    )
    .unwrap();
    let readback_json = r#"{
  "schema_version": "__READBACK_SCHEMA__",
  "run_id": "v190-dashboard-actual-cancel-audit",
  "order_lineage_id": "lineage-v190-actual-cancel",
  "artifact_type": "actual_cancel_readback_reconciliation",
  "status": "ready_actual_cancel_readback_cancel_confirmed",
  "actual_cancel_attempt_ready": true,
  "actual_cancel_attempt_recorded": true,
  "actual_cancel_request_sent": true,
  "actual_cancel_request_id": "actual-cancel-request-001",
  "readback_required": true,
  "readback_evidence_present": true,
  "reconciliation_evidence_present": true,
  "reconciliation_ready": true,
  "readback_reconciliation_complete": true,
  "actual_cancel_followup_complete": true,
  "redacted_metadata_only": true,
  "venue": "binance_spot",
  "symbol": "BTCUSDT",
  "account_label": "prod-account-redacted",
  "known_order_id": "123456789",
  "known_client_order_id": "owner-approved-v190-single-shot",
  "readback_type": "metadata_only",
  "readback_state": "CANCELED",
  "readback_result": "cancel_confirmed",
  "reconciliation_status": "ready_cancel_confirmed",
  "venue_state": "CANCELED",
  "order_status": "CANCELED",
  "execution_fill_status": "not_filled",
  "remaining_quantity_state": "zero",
  "residual_risk_state": "none",
  "local_audit_state": "actual_cancel_attempt_recorded",
  "partial_fill_observed": false,
  "already_cancelled_observed": false,
  "filled_before_cancel_observed": false,
  "timeout_observed": false,
  "unknown_observed": false,
  "inconsistent_observed": false,
  "degraded": false,
  "error_state": false,
  "terminal_state_observed": true,
  "manual_review_required": false,
  "new_orders_blocked": false,
  "risk_halted": false,
  "dashboard_read_only_consumable": true,
  "dashboard_audit_view_ready": true,
  "source_artifact_issues": [],
  "readback_lineage_issues": [],
  "unsupported_readback_states": [],
  "missing_cli_flags": [],
  "actual_cancel_send_allowed": false,
  "cancel_attempted": false,
  "cancel_requests_sent": 0,
  "production_order_mutations_attempted": 0,
  "readback_execution_attempted": false,
  "order_state_read_attempted": false,
  "production_order_state_reads_attempted": 0,
  "network_attempted": false,
  "network_readback_endpoint_attempted": false,
  "network_cancel_endpoint_attempted": false,
  "retry_attempted": false,
  "remediation_attempted": false,
  "second_cancel_attempted": false,
  "automatic_cancel_allowed": false,
  "automatic_remediation_allowed": false,
  "production_order_mutation_allowed": false,
  "dashboard_order_controls_enabled": false,
  "dashboard_cancel_controls_enabled": false
}
"#
    .replace(
        "__READBACK_SCHEMA__",
        PRODUCTION_MUTATION_ACTUAL_CANCEL_READBACK_RECONCILIATION_SCHEMA_VERSION,
    );
    fs::write(
        v19_root.join("actual_cancel_readback_reconciliation.json"),
        readback_json,
    )
    .unwrap();
    let failure_json = r#"{
  "schema_version": "__FAILURE_SCHEMA__",
  "run_id": "v190-dashboard-actual-cancel-audit",
  "order_lineage_id": "lineage-v190-actual-cancel",
  "artifact_type": "actual_cancel_failure_evidence",
  "status": "ready_actual_cancel_failure_recovered_cancel_confirmed",
  "references_ready": true,
  "evidence_ready": true,
  "failure_evidence_ready": true,
  "dashboard_read_only_consumable": true,
  "release_gate_consumable": true,
  "venue": "binance_spot",
  "symbol": "BTCUSDT",
  "account_label": "prod-account-redacted",
  "readback_result": "cancel_confirmed",
  "reconciliation_status": "ready_cancel_confirmed",
  "source_readback_state": "CANCELED",
  "source_venue_state": "CANCELED",
  "source_order_status": "CANCELED",
  "source_execution_fill_status": "not_filled",
  "source_remaining_quantity_state": "zero",
  "source_residual_risk_state": "none",
  "source_local_audit_state": "actual_cancel_attempt_recorded",
  "cancel_outcome": "cancel_confirmed",
  "outcome_category": "recovered",
  "failure_mode": "none",
  "partial_success_mode": "none",
  "operator_action": "none",
  "operator_action_required": false,
  "recovered": true,
  "degraded": false,
  "failed": false,
  "partial_success": false,
  "residual_risk_visible": false,
  "residual_risk_state": "none",
  "manual_review_required": false,
  "new_orders_blocked": false,
  "risk_halted": false,
  "outcome_cancel_confirmed": true,
  "outcome_already_cancelled": false,
  "outcome_rejected": false,
  "outcome_timeout": false,
  "outcome_unknown": false,
  "outcome_partial_fill": false,
  "outcome_filled_before_cancel": false,
  "outcome_venue_unavailable": false,
  "outcome_adapter_failure": false,
  "outcome_inconsistent": false,
  "outcome_failed": false,
  "actual_cancel_followup_complete": true,
  "unknown_not_recovered": true,
  "partial_fill_residual_risk_visible": true,
  "request_response_readback_audit_refs_recorded": true,
  "source_artifact_issues": [],
  "lineage_issues": [],
  "missing_cli_flags": [],
  "actual_cancel_send_allowed": false,
  "cancel_attempted": false,
  "cancel_requests_sent": 0,
  "production_order_mutations_attempted": 0,
  "readback_execution_attempted": false,
  "order_state_read_attempted": false,
  "production_order_state_reads_attempted": 0,
  "network_attempted": false,
  "network_readback_endpoint_attempted": false,
  "network_cancel_endpoint_attempted": false,
  "retry_attempted": false,
  "replace_attempted": false,
  "amend_attempted": false,
  "flatten_attempted": false,
  "remediation_attempted": false,
  "compensation_trade_attempted": false,
  "second_cancel_attempted": false,
  "automatic_cancel_allowed": false,
  "automatic_remediation_allowed": false,
  "production_order_mutation_allowed": false,
  "dashboard_order_controls_enabled": false,
  "dashboard_cancel_controls_enabled": false
}
"#
    .replace(
        "__FAILURE_SCHEMA__",
        PRODUCTION_MUTATION_ACTUAL_CANCEL_FAILURE_EVIDENCE_SCHEMA_VERSION,
    );
    fs::write(
        v19_root.join("actual_cancel_failure_evidence.json"),
        failure_json,
    )
    .unwrap();
}

fn write_production_order_lifecycle_audit_v20_artifacts(record: &SupervisorNodeRecord) {
    let root = record.artifact_root.join("v0_20");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("guarded_submit_candidate.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_ORDER_LIFECYCLE_SUBMIT_CANDIDATE_SCHEMA_VERSION,
            "run_id": "v200-dashboard-order-lifecycle-audit",
            "lifecycle_id": "lineage-v200-single-shot-submit",
            "attempt_id": "attempt-v200-001",
            "artifact_type": "guarded_submit_candidate",
            "state": "submit_attempt_recorded",
            "code": "production_submit_single_shot_recorded",
            "owner_approval_state_before_attempt": "approved",
            "owner_approval_state_after_attempt": "consumed",
            "owner_approval_consumed": true,
            "production_submit_attempted": true,
            "readback_required": true,
            "dashboard_order_controls_enabled": false,
            "dashboard_approval_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false,
            "retry_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "automatic_cancel_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "strategy_continuation_allowed": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("submit_response_redaction.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_ORDER_LIFECYCLE_RESPONSE_REDACTION_SCHEMA_VERSION,
            "run_id": "v200-dashboard-order-lifecycle-audit",
            "lifecycle_id": "lineage-v200-single-shot-submit",
            "attempt_id": "attempt-v200-001",
            "artifact_type": "submit_response_redaction",
            "state": "accepted",
            "code": "venue_metadata_redacted",
            "venue_status": "NEW",
            "order_id": "123456789",
            "client_order_id": "owner-approved-v200-submit",
            "response_redacted": true,
            "evidence_source": "manual_structured",
            "source_provenance_id": "dashboard-fixture-manual-response-v200",
            "source_provenance_required": true,
            "source_provenance_valid": true,
            "source_claim_consistent": true,
            "exchange_truth_claimed": false,
            "adapter_runtime_integrated": false,
            "foundation_only": true,
            "raw_response_recorded": false,
            "api_key_value_recorded": false,
            "api_secret_value_recorded": false,
            "production_submit_attempted": true,
            "readback_required": true,
            "dashboard_order_controls_enabled": false,
            "dashboard_approval_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false,
            "retry_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "automatic_cancel_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "strategy_continuation_allowed": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("submit_readback_reconciliation.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_ORDER_LIFECYCLE_READBACK_RECONCILIATION_SCHEMA_VERSION,
            "run_id": "v200-dashboard-order-lifecycle-audit",
            "lifecycle_id": "lineage-v200-single-shot-submit",
            "attempt_id": "attempt-v200-001",
            "artifact_type": "submit_readback_reconciliation",
            "state": "matched",
            "code": "readback_matched_redacted_metadata",
            "mismatch_fields": [],
            "readback_consistent": true,
            "readback_missing": false,
            "readback_failed": false,
            "evidence_source": "manual_structured",
            "source_provenance_id": "dashboard-fixture-manual-readback-v200",
            "source_provenance_required": true,
            "source_provenance_valid": true,
            "source_claim_consistent": true,
            "exchange_truth_claimed": false,
            "adapter_runtime_integrated": false,
            "foundation_only": true,
            "production_submit_attempted": true,
            "readback_required": true,
            "dashboard_order_controls_enabled": false,
            "dashboard_approval_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false,
            "retry_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "automatic_cancel_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "strategy_continuation_allowed": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("failure_no_retry_evidence.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_ORDER_LIFECYCLE_FAILURE_NO_RETRY_SCHEMA_VERSION,
            "run_id": "v200-dashboard-order-lifecycle-audit",
            "lifecycle_id": "lineage-v200-single-shot-submit",
            "attempt_id": "attempt-v200-001",
            "artifact_type": "failure_no_retry_evidence",
            "category": "none",
            "code": "no_failure",
            "next_allowed_action": "audit_closeout_only",
            "no_implicit_retry": true,
            "unknown_state_visible": false,
            "production_submit_attempted": true,
            "readback_required": true,
            "dashboard_order_controls_enabled": false,
            "dashboard_approval_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false,
            "retry_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "automatic_cancel_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "strategy_continuation_allowed": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("order_lifecycle_audit_closeout.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_ORDER_LIFECYCLE_AUDIT_CLOSEOUT_SCHEMA_VERSION,
            "run_id": "v200-dashboard-order-lifecycle-audit",
            "lifecycle_id": "lineage-v200-single-shot-submit",
            "attempt_id": "attempt-v200-001",
            "artifact_type": "order_lifecycle_audit_closeout",
            "status": "closed",
            "audit_closed": true,
            "dashboard_audit_consumable": true,
            "release_gate_consumable": true,
            "production_submit_attempted": true,
            "readback_required": true,
            "no_implicit_retry": true,
            "dashboard_order_controls_enabled": false,
            "dashboard_approval_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false,
            "retry_attempted": false,
            "replace_attempted": false,
            "amend_attempted": false,
            "flatten_attempted": false,
            "automatic_cancel_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "strategy_continuation_allowed": false
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_production_cancel_recovery_v18_artifacts(record: &SupervisorNodeRecord) {
    let root = record.artifact_root.join("v0_18");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("cancel_request_preview.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_CANCEL_REQUEST_PREVIEW_SCHEMA_VERSION,
            "run_id": "v180-dashboard-cancel-recovery",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "cancel_request_preview",
            "status": "ready_cancel_request_preview",
            "cancel_request_preview_ready": true,
            "cancel_reason": "orphan_risk_detected",
            "lineage_scope": "single_mutation_candidate",
            "candidate_count": 1,
            "known_order_id": "123456789",
            "known_client_order_id": "owner-approved-v160-single-shot",
            "symbol": "BTCUSDT",
            "account_label": "BINANCE-001",
            "orphan_risk_detected": true,
            "risk_halted": true,
            "manual_review_required": true,
            "new_orders_blocked": true,
            "actual_cancel_send_allowed": false,
            "cancel_attempted": false,
            "cancel_requests_sent": 0,
            "network_attempted": false,
            "network_cancel_endpoint_attempted": false,
            "retry_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("cancel_risk_gate.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_CANCEL_RISK_GATE_SCHEMA_VERSION,
            "run_id": "v180-dashboard-cancel-recovery",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "cancel_risk_gate",
            "status": "ready_owner_approval_required",
            "cancel_risk_gate_ready": true,
            "risk_gate_ready": true,
            "cancel_request_preview_ready": true,
            "risk_gate_result": "ready_owner_approval_required",
            "orphan_risk_detected": true,
            "risk_halted": true,
            "manual_review_required": true,
            "new_orders_blocked": true,
            "owner_approval_required": true,
            "candidate_count": 1,
            "known_order_id": "123456789",
            "known_client_order_id": "owner-approved-v160-single-shot",
            "symbol": "BTCUSDT",
            "account_label": "BINANCE-001",
            "actual_cancel_send_allowed": false,
            "cancel_attempted": false,
            "cancel_requests_sent": 0,
            "network_attempted": false,
            "network_cancel_endpoint_attempted": false,
            "retry_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("manual_owner_approval_lifecycle.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_MANUAL_OWNER_APPROVAL_LIFECYCLE_SCHEMA_VERSION,
            "run_id": "v180-dashboard-cancel-recovery",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "manual_owner_approval_lifecycle",
            "status": "ready_manual_owner_approval_lifecycle",
            "approval_state": "approved",
            "manual_approval_recorded": true,
            "approval_lifecycle_valid": true,
            "approval_consumed": false,
            "actual_cancel_send_allowed": false,
            "cancel_attempted": false,
            "cancel_requests_sent": 0,
            "network_attempted": false,
            "network_cancel_endpoint_attempted": false,
            "retry_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("cancel_response_redaction.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_CANCEL_RESPONSE_REDACTION_SCHEMA_VERSION,
            "run_id": "v180-dashboard-cancel-recovery",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "cancel_response_redaction",
            "status": "ready_cancel_response_redaction",
            "response_redaction_ready": true,
            "cancel_response_redacted": true,
            "known_order_id": "123456789",
            "known_client_order_id": "owner-approved-v160-single-shot",
            "symbol": "BTCUSDT",
            "account_label": "BINANCE-001",
            "actual_cancel_send_allowed": false,
            "cancel_attempted": false,
            "cancel_requests_sent": 0,
            "network_attempted": false,
            "network_cancel_endpoint_attempted": false,
            "retry_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("post_cancel_readback.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_MUTATION_POST_CANCEL_READBACK_SCHEMA_VERSION,
            "run_id": "v180-dashboard-cancel-recovery",
            "order_lineage_id": "lineage-v160-single-shot",
            "artifact_type": "post_cancel_readback",
            "status": "ready_post_cancel_readback",
            "post_cancel_readback_ready": true,
            "readback_state": "CANCELED",
            "readback_state_class": "terminal_canceled",
            "readback_outcome": "cancel_confirmed",
            "terminal_state_observed": true,
            "ambiguous_state_observed": false,
            "order_lineage_preserved": true,
            "actual_cancel_send_allowed": false,
            "cancel_attempted": false,
            "cancel_requests_sent": 0,
            "readback_execution_attempted": false,
            "order_state_read_attempted": false,
            "production_order_state_reads_attempted": 0,
            "network_attempted": false,
            "network_readback_endpoint_attempted": false,
            "network_cancel_endpoint_attempted": false,
            "retry_attempted": false,
            "remediation_attempted": false,
            "automatic_cancel_allowed": false,
            "automatic_remediation_allowed": false,
            "dashboard_order_controls_enabled": false,
            "dashboard_cancel_controls_enabled": false
        }))
        .unwrap(),
    )
    .unwrap();
    let closeout_json = r#"{
  "schema_version": "__CLOSEOUT_SCHEMA__",
  "run_id": "v180-dashboard-cancel-recovery",
  "order_lineage_id": "lineage-v160-single-shot",
  "artifact_type": "cancel_recovery_incident_audit_closeout",
  "status": "ready_cancel_recovery_incident_audit_closeout",
  "incident_closeout_ready": true,
  "audit_trail_ready": true,
  "audit_traceability_ready": true,
  "cancel_recovery_lineage_ready": true,
  "recovery_needed_reason": "orphan_risk_detected",
  "risk_gate_result": "ready_owner_approval_required",
  "risk_gate_ready": true,
  "orphan_risk_detected": true,
  "risk_halted": true,
  "manual_review_required": true,
  "new_orders_blocked": true,
  "owner_approval_state": "approved",
  "manual_approval_recorded": true,
  "approval_lifecycle_valid": true,
  "approval_consumed": false,
  "redaction_contract_state": "ready_redacted_metadata_only",
  "cancel_response_redaction_ready": true,
  "cancel_response_redacted": true,
  "post_cancel_readback_ready": true,
  "readback_state": "CANCELED",
  "readback_state_class": "terminal_canceled",
  "readback_outcome": "cancel_confirmed",
  "terminal_state_observed": true,
  "ambiguous_state_observed": false,
  "terminal_action_recommendation": "close_incident_cancel_confirmed",
  "remaining_risk": "none_cancel_confirmed",
  "remaining_risk_requires_manual_review": false,
  "source_artifact_issues": [],
  "lineage_issues": [],
  "missing_cli_flags": [],
  "actual_cancel_send_allowed": false,
  "cancel_attempted": false,
  "cancel_requests_sent": 0,
  "production_order_mutations_attempted": 0,
  "readback_execution_attempted": false,
  "order_state_read_attempted": false,
  "production_order_state_reads_attempted": 0,
  "network_attempted": false,
  "network_readback_endpoint_attempted": false,
  "network_cancel_endpoint_attempted": false,
  "retry_attempted": false,
  "remediation_attempted": false,
  "automatic_cancel_allowed": false,
  "automatic_remediation_allowed": false,
  "dashboard_order_controls_enabled": false,
  "dashboard_cancel_controls_enabled": false,
  "dashboard_auto_approval_allowed": false,
  "dashboard_auto_approval_attempted": false
}
"#
    .replace(
        "__CLOSEOUT_SCHEMA__",
        PRODUCTION_MUTATION_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_SCHEMA_VERSION,
    );
    fs::write(
        root.join("cancel_recovery_incident_audit_closeout.json"),
        closeout_json,
    )
    .unwrap();
}

fn dashboard_js_function_body(function_name: &str) -> &str {
    let needle = format!("function {function_name}");
    let start = DASHBOARD_JS
        .find(&needle)
        .expect("dashboard function must exist");
    let after_start = start + needle.len();
    let end = DASHBOARD_JS[after_start..]
        .find("\nfunction ")
        .map_or(DASHBOARD_JS.len(), |offset| after_start + offset);
    &DASHBOARD_JS[start..end]
}

fn write_live_alpha_order_state_readonly_proof_artifact(record: &SupervisorNodeRecord) {
    let root = record.artifact_root.join("v0_14");
    fs::create_dir_all(&root).unwrap();
    let proof_json = r#"{
  "schema_version": "__ORDER_STATE_PROOF_SCHEMA__",
  "status": "online_order_state_read_ok",
  "endpoint": "open_orders",
  "endpoint_class": "production_order_state_read_only",
  "http_base_url": "https://api.binance.com",
  "method": "GET",
  "path": "/api/v3/openOrders",
  "request_url_redacted": "https://api.binance.com/api/v3/openOrders?symbol=BTCUSDT&timestamp=<redacted>&signature=<redacted>",
  "query_shape": "symbol,timestamp,signature",
  "symbol": "BTCUSDT",
  "order_id_provided": false,
  "orig_client_order_id_provided": false,
  "requires_api_key": true,
  "requires_signature": true,
  "endpoint_read_allowed": true,
  "offline_contract_ready": true,
  "read_allowed": true,
  "contract_ready": true,
  "online_read_allowed": true,
  "mutation_allowed": false,
  "owner_gate_required": true,
  "manual_gate_required": true,
  "missing_cli_flags": [],
  "missing_env_vars": [],
  "manual_online_requested": true,
  "online_execution_supported": true,
  "network_attempted": true,
  "response_status_code": 200,
  "response_shape": "open_orders_empty_array",
  "response_shape_validated": true,
  "response_shape_summary": {
"endpoint": "open_orders",
"response_shape": "open_orders_empty_array",
"shape_validated": true,
"endpoint_shape_validated": true,
"order_entries_observed": 0,
"non_empty_order_state_observed": false,
"order_lifecycle_readiness": false,
"diagnostic": "openOrders endpoint returned an empty but valid array"
  },
  "endpoint_shape_validated": true,
  "order_entries_observed": 0,
  "non_empty_order_state_observed": false,
  "order_lifecycle_readiness": false,
  "latency_ms": 42,
  "error_code": "none",
  "env_credentials_only": true,
  "api_key_env": "BINANCE_API_KEY",
  "api_secret_env": "BINANCE_API_SECRET",
  "api_key_present": true,
  "api_secret_present": true,
  "api_key_value_recorded": false,
  "api_secret_value_recorded": false,
  "signature_recorded": false,
  "signed_query_recorded": false,
  "signed_url_recorded": false,
  "order_state_read_attempted": true,
  "production_order_state_reads_attempted": 1,
  "production_order_submission_attempted": false,
  "production_order_mutation_attempted": false,
  "cancel_replace_amend_attempted": false,
  "listen_key_lifecycle_attempted": false,
  "dashboard_order_controls_enabled": false,
  "automatic_remediation_attempted": false,
  "real_orders_submitted": false,
  "real_funds": false,
  "production_trading_enabled": false,
  "order_state_values_are_exchange_truth": true,
  "shadow_values_are_exchange_truth": false,
  "portfolio_values_are_exchange_truth": false,
  "values_are_exchange_truth": true,
  "secrets_redacted": true,
  "diagnostic": "production order-state read-only proof returned empty openOrders; endpoint is readable but lifecycle readiness is false"
}
"#
    .replace(
        "__ORDER_STATE_PROOF_SCHEMA__",
        PRODUCTION_ORDER_STATE_READONLY_PROOF_SCHEMA_VERSION,
    );
    fs::write(
        root.join("production_order_state_readonly_proof.json"),
        proof_json,
    )
    .unwrap();
}

fn write_production_shadow_manifest(shadow_root: &FsPath) {
    let artifact_specs = [
        (
            "account_snapshot",
            "json",
            "account_snapshot_redacted.json",
            Some(1_u64),
        ),
        (
            "shadow_execution_intent",
            "jsonl",
            "shadow_execution_intent.jsonl",
            Some(1_u64),
        ),
        (
            "shadow_portfolio_snapshot",
            "json",
            "shadow_portfolio_snapshot.json",
            Some(1_u64),
        ),
        (
            "order_lifecycle_state",
            "jsonl",
            "order_lifecycle_state.jsonl",
            Some(1_u64),
        ),
        (
            "reconciliation_events",
            "jsonl",
            "reconciliation_events.jsonl",
            Some(1_u64),
        ),
    ];
    let artifacts = artifact_specs
        .into_iter()
        .map(|(name, format, file, record_count)| {
            let path = shadow_root.join(file);
            let bytes = fs::read(&path).unwrap();
            json!({
                "name": name,
                "path": file,
                "format": format,
                "required": true,
                "present": true,
                "record_count": record_count,
                "byte_len": u64::try_from(bytes.len()).unwrap(),
                "checksum": checksum_bytes(&bytes),
                "raw_secret_recorded": false,
                "raw_payload_recorded": false,
                "signed_query_recorded": false,
                "signed_url_recorded": false
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        shadow_root.join("manifest.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": PRODUCTION_SHADOW_MANIFEST_SCHEMA_VERSION,
            "run_id": "v110-shadow",
            "generated_at": "2026-06-20T10:00:00Z",
            "artifact_root": "v0_11",
            "artifact_count": artifacts.len(),
            "artifacts": artifacts,
            "summary": {
                "account_snapshots": 1,
                "shadow_intents_created": 1,
                "shadow_portfolio_snapshots_created": 1,
                "lifecycle_events_created": 1,
                "reconciliation_events_created": 1,
                "actual_submission_count": 0,
                "production_orders_submitted": 0,
                "production_order_mutations_attempted": 0,
                "dashboard_order_controls_enabled": false,
                "raw_secret_recorded": false,
                "raw_payload_recorded": false
            }
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_strategy_manifest(strategy_root: &FsPath) {
    let artifact_specs = [
        ("session_status", "json", "session_status.json", Some(1)),
        ("events", "jsonl", "events.jsonl", Some(1)),
        ("market_status", "json", "market_status.json", Some(1)),
        ("market_events", "jsonl", "market_events.jsonl", Some(8)),
        ("signal", "jsonl", "signal.jsonl", Some(2)),
        ("order_intent", "jsonl", "order_intent.jsonl", Some(2)),
        ("risk_decision", "jsonl", "risk_decision.jsonl", Some(2)),
        ("summary", "json", "summary.json", Some(1)),
    ];
    let artifacts = artifact_specs
        .into_iter()
        .map(|(name, format, file, record_count)| {
            let path = strategy_root.join(file);
            let bytes = fs::read(&path).unwrap();
            json!({
                "name": name,
                "path": path.display().to_string(),
                "format": format,
                "present": true,
                "record_count": record_count,
                "byte_len": u64::try_from(bytes.len()).unwrap(),
                "checksum": format!("blake3:{}", blake3::hash(&bytes).to_hex())
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        strategy_root.join("manifest.json"),
        serde_json::to_string_pretty(&json!({
            "schema_version": "ntpro.v091_strategy_session_manifest.v1",
            "session_id": "btc-ema-shadow-001",
            "strategy_id": "ema_cross_btcusdt_v1",
            "state": "stopped",
            "created_at_unix_ms": 1,
            "updated_at_unix_ms": 3000,
            "artifacts": artifacts
        }))
        .unwrap(),
    )
    .unwrap();
}

fn create_node_dirs(record: &SupervisorNodeRecord) {
    fs::create_dir_all(record.artifact_root.join("logs")).unwrap();
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

async fn http_request(addr: SocketAddr, method: &str, path: &str) -> String {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let request = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

async fn post_action_until_ok(addr: SocketAddr, path: &str, context: &str) -> String {
    let deadline = tokio::time::Instant::now() + HTTP_CONTROL_TEST_DEADLINE;
    loop {
        let response = http_request(addr, "POST", path).await;
        if response.contains("HTTP/1.1 200 OK") {
            return response;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "{context} expected HTTP 200 OK, got:\n{response}",
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_http_node_state(
    addr: SocketAddr,
    node_id: &str,
    lifecycle_state: LifecycleStatus,
    process_state: SupervisorProcessState,
    context: &str,
) -> Value {
    let expected_lifecycle = json_label(&lifecycle_state);
    let expected_process = json_label(&process_state);
    let deadline = tokio::time::Instant::now() + HTTP_CONTROL_TEST_DEADLINE;
    loop {
        let response = http_request(addr, "GET", "/api/snapshot").await;
        if response.contains("HTTP/1.1 200 OK") {
            let value: Value = serde_json::from_str(response_body(&response)).unwrap();
            let node_matches = value["nodes"].as_array().is_some_and(|nodes| {
                nodes.iter().any(|node| {
                    node["node_id"] == node_id
                        && node["lifecycle_state"] == expected_lifecycle
                        && node["process_state"] == expected_process
                })
            });
            if node_matches {
                return value;
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "{context} expected node_id={node_id} lifecycle_state={expected_lifecycle} process_state={expected_process}, got:\n{response}",
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn assert_http_ok(response: &str, context: &str) {
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "{context} expected HTTP 200 OK, got:\n{response}"
    );
}

fn response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map_or("", |(_, body)| body.trim())
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
