// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
// -------------------------------------------------------------------------------------------------

//! 控制中心静态 shell 与共享状态、运维投影渲染资源。

use std::path::Path;

use serde::Serialize;

use super::*;

pub(super) const CONTROL_CENTER_OPERATIONAL_SCHEMA_VERSION: &str =
    "ntpro.mvp_control_center_snapshot.v2";
pub(super) const CONTROL_CENTER_LIFECYCLE_ACTION_SCHEMA_VERSION: &str =
    "ntpro.mvp_control_center_lifecycle_action.response.v1";
pub(super) const CONTROL_CENTER_LIFECYCLE_ACTION_CONTRACT_VERSION: &str =
    "ntpro.mvp_control_center_lifecycle_action.v1";

#[derive(Clone, Debug, Serialize)]
pub(super) struct ControlCenterOperationalSnapshot {
    schema_version: String,
    generated_at: DashboardValue<String>,
    registry_path: String,
    local_only: bool,
    overview: ControlCenterOverview,
    node: ControlCenterNode,
    data_sources: Vec<ControlCenterDataSource>,
    execution_gateways: Vec<ControlCenterExecutionGateway>,
    runtime_modules: Vec<ControlCenterRuntimeModule>,
    logs: Vec<ControlCenterLog>,
    metrics: Vec<ControlCenterMetric>,
    alerts: Vec<ControlCenterAlert>,
    gaps: Vec<ControlCenterGap>,
    lifecycle_actions: Vec<ControlCenterLifecycleAction>,
    boundaries: ControlCenterOperationalBoundaries,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterOverview {
    node_count: u64,
    running_nodes: u64,
    stopped_nodes: u64,
    error_nodes: u64,
    unknown_nodes: u64,
    health: HealthStatus,
    sandbox_only: bool,
    latest_transition_at: DashboardValue<String>,
    latest_error_present: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterNode {
    node_id: String,
    lifecycle_state: LifecycleStatus,
    process_mode: ProcessMode,
    process_state: SupervisorProcessState,
    pid: SnapshotValue<u32>,
    health: HealthStatus,
    last_transition_at: SnapshotValue<String>,
    error_present: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterDataSource {
    source_id: String,
    source_kind: DashboardValue<String>,
    provider: DashboardValue<String>,
    connection: ConnectionStatus,
    freshness: DashboardValue<String>,
    lag_ms: DashboardValue<u64>,
    health: HealthStatus,
    error_present: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterExecutionGateway {
    gateway_id: String,
    venue: DashboardValue<String>,
    connection: ConnectionStatus,
    started: DashboardValue<bool>,
    last_report_at: DashboardValue<String>,
    error_present: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterRuntimeModule {
    module_name: String,
    status: DashboardValue<String>,
    health: HealthStatus,
    last_seen_at: DashboardValue<String>,
    evidence_source: DashboardValue<String>,
    error_present: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterLog {
    log_id: String,
    availability: DashboardAvailability,
    last_seen_at: DashboardValue<String>,
    error_present: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterMetric {
    metric_id: String,
    value: DashboardValue<String>,
    availability: DashboardAvailability,
    last_seen_at: DashboardValue<String>,
    error_present: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterAlert {
    alert_id: String,
    severity: String,
    source: String,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterGap {
    field_path: String,
    reason: DashboardAvailability,
    owner_task: DashboardValue<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterLifecycleAction {
    action: String,
    target_node_id: String,
    method: String,
    enabled: bool,
    reason_code: String,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterOperationalBoundaries {
    read_only: bool,
    external_venue_connection: bool,
    production_venue_connection: bool,
    testnet_public_network_connection: bool,
    external_network_attempted: bool,
    real_orders_submitted: bool,
    supervisor_actions_exposed: bool,
    unsupported_supervisor_actions_exposed: bool,
    trading_controls_exposed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    raw_errors_exposed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ControlCenterLifecycleActionEnvelope {
    schema_version: String,
    contract_version: String,
    local_only: bool,
    target_node_id: String,
    action_name: String,
    result: ControlActionResponse,
    boundaries: ControlCenterLifecycleActionBoundaries,
}

#[derive(Clone, Debug, Serialize)]
struct ControlCenterLifecycleActionBoundaries {
    supervisor_lifecycle_action: bool,
    external_venue_connection: bool,
    production_venue_connection: bool,
    external_network_attempted: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
}

pub(super) fn project_control_center_lifecycle_action(
    target_node_id: &str,
    action_name: &str,
    result: ControlActionResponse,
) -> ControlCenterLifecycleActionEnvelope {
    ControlCenterLifecycleActionEnvelope {
        schema_version: CONTROL_CENTER_LIFECYCLE_ACTION_SCHEMA_VERSION.to_string(),
        contract_version: CONTROL_CENTER_LIFECYCLE_ACTION_CONTRACT_VERSION.to_string(),
        local_only: true,
        target_node_id: target_node_id.to_string(),
        action_name: action_name.to_string(),
        result,
        boundaries: ControlCenterLifecycleActionBoundaries {
            supervisor_lifecycle_action: true,
            external_venue_connection: false,
            production_venue_connection: false,
            external_network_attempted: false,
            order_submission_allowed: false,
            order_mutation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            real_orders_submitted: false,
        },
    }
}

fn normalize_control_center_value<T>(
    mut value: DashboardValue<T>,
) -> Result<DashboardValue<T>, &'static str> {
    match value.availability {
        DashboardAvailability::Available if value.value.is_some() => Ok(value),
        DashboardAvailability::Available => Err("dashboard_value_missing_available_value"),
        DashboardAvailability::Redacted => {
            value.value = None;
            Ok(value)
        }
        DashboardAvailability::NotConfigured
        | DashboardAvailability::NotSupported
        | DashboardAvailability::Stale
        | DashboardAvailability::Unknown
            if value.value.is_none() =>
        {
            Ok(value)
        }
        DashboardAvailability::NotConfigured
        | DashboardAvailability::NotSupported
        | DashboardAvailability::Stale
        | DashboardAvailability::Unknown => Err("dashboard_value_unavailable_with_value"),
    }
}

pub(super) fn project_control_center_snapshot(
    registry_path: &Path,
    snapshot: DashboardSnapshot,
) -> Result<ControlCenterOperationalSnapshot, &'static str> {
    if snapshot.nodes.len() != 1 || snapshot.overview.node_count != 1 {
        return Err("single_node_contract_violation");
    }
    if !snapshot.overview.sandbox_only
        || snapshot.overview.external_venue_connection
        || snapshot.overview.production_venue_connection
        || snapshot.overview.testnet_public_network_connection
        || snapshot.overview.external_network_attempted
        || snapshot.overview.real_orders_submitted
    {
        return Err("operational_boundary_violation");
    }
    let node = snapshot
        .nodes
        .into_iter()
        .next()
        .ok_or("single_node_contract_violation")?;
    if node.external_venue_connection || node.real_orders_submitted {
        return Err("node_boundary_violation");
    }
    let expected_counts = match node.lifecycle_state {
        LifecycleStatus::Running => (1, 0, 0, 0),
        LifecycleStatus::Stopped => (0, 1, 0, 0),
        LifecycleStatus::Error => (0, 0, 1, 0),
        LifecycleStatus::Unknown => (0, 0, 0, 1),
        LifecycleStatus::Starting
        | LifecycleStatus::Pausing
        | LifecycleStatus::Paused
        | LifecycleStatus::Resuming
        | LifecycleStatus::Stopping => (0, 0, 0, 0),
    };
    if (
        snapshot.overview.running_nodes,
        snapshot.overview.stopped_nodes,
        snapshot.overview.error_nodes,
        snapshot.overview.unknown_nodes,
    ) != expected_counts
    {
        return Err("overview_node_count_mismatch");
    }

    let generated_at = normalize_control_center_value(snapshot.generated_at)?;
    let latest_transition_at =
        normalize_control_center_value(snapshot.overview.latest_transition_at)?;
    let lifecycle_actions = vec![
        ControlCenterLifecycleAction {
            action: "start".to_string(),
            target_node_id: node.node_id.clone(),
            method: "POST".to_string(),
            enabled: node.lifecycle_state == LifecycleStatus::Stopped,
            reason_code: if node.lifecycle_state == LifecycleStatus::Stopped {
                "ready".to_string()
            } else {
                "requires_stopped".to_string()
            },
        },
        ControlCenterLifecycleAction {
            action: "stop".to_string(),
            target_node_id: node.node_id.clone(),
            method: "POST".to_string(),
            enabled: matches!(
                node.lifecycle_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            ),
            reason_code: if matches!(
                node.lifecycle_state,
                LifecycleStatus::Running | LifecycleStatus::Paused
            ) {
                "ready".to_string()
            } else {
                "requires_running_or_paused".to_string()
            },
        },
    ];

    Ok(ControlCenterOperationalSnapshot {
        schema_version: CONTROL_CENTER_OPERATIONAL_SCHEMA_VERSION.to_string(),
        generated_at,
        registry_path: registry_path.display().to_string(),
        local_only: true,
        overview: ControlCenterOverview {
            node_count: snapshot.overview.node_count,
            running_nodes: snapshot.overview.running_nodes,
            stopped_nodes: snapshot.overview.stopped_nodes,
            error_nodes: snapshot.overview.error_nodes,
            unknown_nodes: snapshot.overview.unknown_nodes,
            health: snapshot.overview.health,
            sandbox_only: snapshot.overview.sandbox_only,
            latest_transition_at,
            latest_error_present: snapshot.overview.latest_error.is_some(),
        },
        node: ControlCenterNode {
            node_id: node.node_id,
            lifecycle_state: node.lifecycle_state,
            process_mode: node.process_mode,
            process_state: node.process_state,
            pid: node.pid,
            health: node.health,
            last_transition_at: node.last_transition_at,
            error_present: node.last_error.is_some(),
        },
        data_sources: snapshot
            .data_sources
            .into_iter()
            .map(|item| {
                Ok(ControlCenterDataSource {
                    source_id: item.source_id,
                    source_kind: normalize_control_center_value(item.source_kind)?,
                    provider: normalize_control_center_value(item.provider)?,
                    connection: item.connection,
                    freshness: normalize_control_center_value(item.freshness)?,
                    lag_ms: normalize_control_center_value(item.lag_ms)?,
                    health: item.health,
                    error_present: item.last_error.value.is_some(),
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
        execution_gateways: snapshot
            .execution_gateways
            .into_iter()
            .map(|item| {
                Ok(ControlCenterExecutionGateway {
                    gateway_id: item.gateway_id,
                    venue: normalize_control_center_value(item.venue)?,
                    connection: item.connection,
                    started: normalize_control_center_value(item.started)?,
                    last_report_at: normalize_control_center_value(item.last_report_at)?,
                    error_present: item.last_error.value.is_some(),
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
        runtime_modules: snapshot
            .runtime_modules
            .into_iter()
            .map(|item| {
                Ok(ControlCenterRuntimeModule {
                    module_name: item.module_name,
                    status: normalize_control_center_value(item.status)?,
                    health: item.health,
                    last_seen_at: normalize_control_center_value(item.last_seen_at)?,
                    evidence_source: normalize_control_center_value(item.evidence_source)?,
                    error_present: item.last_error.value.is_some(),
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
        logs: snapshot
            .logs
            .into_iter()
            .map(|item| {
                Ok(ControlCenterLog {
                    log_id: item.log_id,
                    availability: item.availability,
                    last_seen_at: normalize_control_center_value(item.last_seen_at)?,
                    error_present: item.last_error.value.is_some(),
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
        metrics: snapshot
            .metrics
            .into_iter()
            .map(|item| {
                Ok(ControlCenterMetric {
                    metric_id: item.metric_id,
                    value: normalize_control_center_value(item.value)?,
                    availability: item.availability,
                    last_seen_at: normalize_control_center_value(item.last_seen_at)?,
                    error_present: item.last_error.value.is_some(),
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
        alerts: snapshot
            .alerts
            .active
            .into_iter()
            .map(|item| ControlCenterAlert {
                alert_id: item.alert_id,
                severity: item.severity,
                source: item.source,
            })
            .collect(),
        gaps: snapshot
            .gaps
            .into_iter()
            .map(|item| {
                Ok(ControlCenterGap {
                    field_path: item.field_path,
                    reason: item.reason,
                    owner_task: normalize_control_center_value(item.owner_task)?,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
        lifecycle_actions,
        boundaries: ControlCenterOperationalBoundaries {
            read_only: true,
            external_venue_connection: false,
            production_venue_connection: false,
            testnet_public_network_connection: false,
            external_network_attempted: false,
            real_orders_submitted: false,
            supervisor_actions_exposed: true,
            unsupported_supervisor_actions_exposed: false,
            trading_controls_exposed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            raw_errors_exposed: false,
        },
    })
}

pub(super) const CONTROL_CENTER_HTML: &str = r##"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="icon" href="data:,">
  <title>NTPRO 控制中心</title>
  <link rel="stylesheet" href="/assets/control-center.css">
</head>
<body>
  <div class="app-shell">
    <aside class="sidebar" aria-label="控制中心导航">
      <div class="brand"><span class="brand-mark">NT</span><div><strong>NTPRO</strong><span>控制中心</span></div></div>
      <nav>
        <a class="active" href="#overview">运行总览</a>
        <a href="#node">节点</a>
        <a href="#lifecycle-actions">节点操作</a>
        <a href="#components">组件</a>
        <a href="#observability">观测</a>
        <a href="#alerts">告警</a>
        <a href="#evidence">证据</a>
      </nav>
      <div class="sidebar-state"><span id="sidebar-state-dot" class="state-dot"></span><span id="sidebar-state">等待状态对齐</span></div>
    </aside>

    <div class="workspace">
      <header class="context-bar">
        <div class="context-primary">
          <span class="eyebrow">单节点 MVP · 平台运维工作台</span>
          <div><strong id="context-node">节点未加载</strong><span id="context-scope">策略 / 环境未加载</span></div>
        </div>
        <button id="refresh" type="button" title="刷新控制中心状态">刷新</button>
      </header>

      <main>
        <section id="connection-banner" class="connection-banner loading" aria-live="polite">
          <div><strong id="connection-title">正在对齐共享与运维状态</strong><span id="connection-detail">等待版本化合同和本地监督器投影</span></div>
          <span id="connection-badge">读取中</span>
        </section>

        <section id="overview" class="section-block">
          <div class="section-heading"><div><span class="eyebrow">共享控制合同</span><h1>系统状态与业务影响</h1></div><span id="generated-at" class="section-meta">尚未生成</span></div>
          <div id="axis-grid" class="axis-grid"></div>
          <div class="impact-band">
            <div><span class="eyebrow">业务影响</span><strong id="business-impact-title">等待共享状态</strong></div>
            <div id="business-impact-list" class="impact-list"></div>
          </div>
          <div id="event-correlation" class="event-correlation-band">
            <div id="event-correlation-panel" class="event-correlation-copy"><strong>等待事件关联</strong><span>尚未绑定业务影响</span></div>
          </div>
        </section>

        <div class="content-grid">
          <div class="primary-column">
            <section id="node" class="section-block">
              <div class="section-heading"><div><span class="eyebrow">Supervisor</span><h2>单节点运行状态</h2></div><span id="node-health" class="section-meta">等待节点</span></div>
              <div id="node-grid" class="node-grid"></div>
              <div id="lifecycle-actions" class="lifecycle-action-panel">
                <div><span class="eyebrow">Operator only</span><h3>节点生命周期</h3></div>
                <div id="lifecycle-action-buttons" class="lifecycle-action-buttons"></div>
                <div id="lifecycle-action-result" class="lifecycle-action-result" aria-live="polite">等待已验证节点状态</div>
              </div>
            </section>

            <section id="components" class="section-block">
              <div class="section-heading"><div><span class="eyebrow">运行拓扑</span><h2>组件与连接</h2></div></div>
              <div id="component-table" class="table-wrap"></div>
            </section>

            <section id="observability" class="section-block">
              <div class="section-heading"><div><span class="eyebrow">Observability</span><h2>日志与指标</h2></div></div>
              <div id="observability-grid" class="observability-grid"></div>
            </section>

            <section id="alerts" class="section-block">
              <div class="section-heading"><div><span class="eyebrow">异常队列</span><h2>告警与缺口</h2></div><span id="alert-count" class="section-meta">0 项</span></div>
              <div id="alert-list" class="alert-list"></div>
            </section>
          </div>

          <aside id="evidence" class="evidence-panel" aria-label="控制中心状态证据">
            <div class="section-heading"><div><span class="eyebrow">Provenance</span><h2>来源与边界</h2></div></div>
            <div id="source-list" class="source-list"></div>
            <div id="boundary-list" class="boundary-list"></div>
          </aside>
        </div>
      </main>

      <footer class="status-bar">
        <span id="footer-environment">环境：未知</span>
        <span id="footer-node">节点：未知</span>
        <span id="footer-runtime">运行状态：未知</span>
        <span id="footer-health">技术健康：未知</span>
        <span id="footer-readiness">交易准备度：阻断</span>
        <span id="footer-updated">更新时间：未知</span>
      </footer>
    </div>
  </div>
  <script src="/assets/control-center.js"></script>
</body>
</html>
"##;

pub(super) const CONTROL_CENTER_CSS: &str = r#":root {
  color-scheme: light;
  font-family: Inter, "Noto Sans CJK SC", "Noto Sans CJK", ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #edf2f0;
  color: #18221e;
}

* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; min-width: 320px; }
button, a { font: inherit; }

.app-shell { display: grid; grid-template-columns: 196px minmax(0, 1fr); height: 100vh; overflow: hidden; }
.sidebar {
  position: sticky;
  top: 0;
  height: 100vh;
  display: flex;
  flex-direction: column;
  gap: 28px;
  padding: 20px 14px;
  background: #17231e;
  color: #f5f7f6;
  border-right: 1px solid #2d3b35;
}
.brand { display: flex; align-items: center; gap: 10px; padding: 0 6px; }
.brand-mark { display: grid; place-items: center; width: 34px; height: 34px; border: 1px solid #668174; border-radius: 6px; color: #87e0b5; font-weight: 800; }
.brand div { display: grid; gap: 2px; }
.brand span:last-child { color: #aebbb5; font-size: 12px; }
.sidebar nav { display: grid; gap: 4px; }
.sidebar nav a { color: #b9c5bf; text-decoration: none; padding: 10px 12px; border-left: 2px solid transparent; }
.sidebar nav a:hover, .sidebar nav a.active { color: #ffffff; background: #22322b; border-left-color: #62dca4; }
.sidebar-state { margin-top: auto; display: flex; align-items: center; gap: 8px; padding: 0 6px; color: #b9c5bf; font-size: 12px; }
.state-dot { width: 8px; height: 8px; border-radius: 50%; background: #8b9891; }
.state-dot.ready { background: #62dca4; }
.state-dot.blocked { background: #e46f6f; }
.state-dot.loading { background: #d7b96e; }

.workspace { min-width: 0; min-height: 0; display: grid; grid-template-rows: auto minmax(0, 1fr) auto; }
.context-bar { min-height: 72px; display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 14px 24px; background: #ffffff; border-bottom: 1px solid #cfd9d4; }
.context-primary { display: grid; gap: 5px; min-width: 0; }
.context-primary > div { display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap; }
.context-primary strong { font-size: 18px; overflow-wrap: anywhere; }
.context-primary span:last-child { color: #68766f; font-size: 13px; }
.eyebrow { color: #178250; font-size: 11px; font-weight: 800; text-transform: uppercase; }
button { border: 1px solid #8a9991; border-radius: 6px; background: #ffffff; color: #18221e; min-height: 38px; padding: 0 14px; font-weight: 700; cursor: pointer; }
button:disabled { cursor: wait; opacity: .55; }

main { min-height: 0; overflow: auto; padding: 20px 16px 32px; }
.connection-banner { display: flex; align-items: center; justify-content: space-between; gap: 16px; min-height: 68px; padding: 14px 16px; background: #ffffff; border: 1px solid #cad7d0; border-left: 4px solid #d7b96e; border-radius: 6px; }
.connection-banner > div { display: grid; gap: 4px; }
.connection-banner span { color: #68766f; font-size: 12px; }
.connection-banner.ready { border-left-color: #24a66c; }
.connection-banner.blocked { border-left-color: #c74646; }
.connection-banner.blocked strong, .status-blocked, .status-unhealthy, .status-error { color: #a52d2d; }
.connection-banner.ready strong, .status-healthy, .status-running, .status-fresh { color: #087944; }
.status-degraded, .status-stale, .status-unknown, .status-missing { color: #9a5a08; }

.section-block { margin-top: 24px; }
.section-heading { display: flex; align-items: end; justify-content: space-between; gap: 16px; margin-bottom: 10px; }
.section-heading h1, .section-heading h2 { margin: 3px 0 0; letter-spacing: 0; }
.section-heading h1 { font-size: 24px; }
.section-heading h2 { font-size: 18px; }
.section-meta { color: #6b7872; font-size: 12px; text-align: right; }

.axis-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
.axis-card, .node-card, .observability-card { background: #ffffff; border: 1px solid #cbd7d1; border-radius: 6px; padding: 12px; min-width: 0; }
.card-label { color: #68766f; font-size: 11px; font-weight: 700; }
.card-value { margin-top: 7px; font-size: 18px; font-weight: 800; overflow-wrap: anywhere; }
.card-meta { display: grid; grid-template-columns: minmax(0, auto) minmax(0, 1fr); gap: 8px; margin-top: 9px; padding-top: 8px; border-top: 1px solid #e1e7e4; color: #66736d; font-size: 11px; }
.card-meta span:last-child { text-align: right; overflow-wrap: anywhere; }
.impact-band { display: flex; align-items: center; justify-content: space-between; gap: 14px; margin-top: 10px; padding: 11px 12px; background: #ffffff; border: 1px solid #cbd7d1; border-radius: 6px; }
.impact-band > div:first-child { display: grid; gap: 4px; min-width: 128px; }
.impact-band strong { font-size: 14px; }
.impact-list { display: flex; justify-content: flex-end; flex-wrap: wrap; gap: 6px 12px; color: #526159; font-size: 11px; }
.impact-list span { overflow-wrap: anywhere; }
.event-correlation-band { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 16px; align-items: center; margin-top: 10px; padding: 11px 12px; background: #ffffff; border: 1px solid #cbd7d1; border-radius: 6px; }
.event-correlation-copy { display: grid; gap: 4px; min-width: 0; }
.event-correlation-copy strong, .event-correlation-copy span { overflow-wrap: anywhere; }
.event-correlation-copy span { color: #526159; font-size: 11px; }
.portal-link { color: #0c6541; font-size: 12px; font-weight: 800; text-decoration: none; border-bottom: 1px solid currentColor; white-space: nowrap; }
.portal-link:hover { color: #094d32; }

.content-grid { display: grid; grid-template-columns: minmax(0, 1fr) 300px; gap: 18px; align-items: start; }
.primary-column { min-width: 0; }
.node-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }
.lifecycle-action-panel { display: grid; grid-template-columns: minmax(140px, .7fr) minmax(220px, 1fr) minmax(220px, 1.3fr); gap: 14px; align-items: center; margin-top: 10px; padding: 12px; background: #ffffff; border: 1px solid #cbd7d1; border-left: 3px solid #178250; border-radius: 6px; }
.lifecycle-action-panel h3 { margin: 3px 0 0; font-size: 15px; }
.lifecycle-action-buttons { display: flex; gap: 8px; flex-wrap: wrap; }
.lifecycle-action-option { display: grid; gap: 4px; min-width: 100px; }
.lifecycle-action-button { min-width: 92px; }
.lifecycle-action-button.stop { border-color: #b94b4b; color: #9d2f2f; }
.lifecycle-action-button.pending { background: #17231e; border-color: #17231e; color: #ffffff; }
.lifecycle-action-result { min-height: 38px; display: grid; align-content: center; gap: 3px; color: #526159; font-size: 11px; overflow-wrap: anywhere; }
.lifecycle-action-result strong { color: #18221e; font-size: 12px; }
.lifecycle-action-result.error strong { color: #a52d2d; }
.lifecycle-action-reason { color: #68766f; font-size: 10px; }
.table-wrap { overflow-x: auto; background: #ffffff; border: 1px solid #cbd7d1; border-radius: 6px; }
table { width: 100%; border-collapse: collapse; min-width: 760px; }
th, td { padding: 10px 12px; border-bottom: 1px solid #e1e7e4; text-align: left; vertical-align: top; font-size: 12px; }
th { color: #53615a; background: #f3f6f4; font-weight: 800; }
tr:last-child td { border-bottom: 0; }
.path { overflow-wrap: anywhere; max-width: 260px; }

.observability-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
.observability-card { display: grid; gap: 8px; }
.observability-row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 10px; padding-top: 7px; border-top: 1px solid #e1e7e4; font-size: 11px; }
.observability-row span { overflow-wrap: anywhere; }
.observability-row span:last-child { text-align: right; color: #66736d; }
.alert-list { display: grid; gap: 8px; }
.alert-item { display: grid; gap: 4px; padding: 11px 12px; background: #ffffff; border: 1px solid #cbd7d1; border-left: 3px solid #d7b96e; border-radius: 6px; }
.alert-item.error, .alert-item.critical { border-left-color: #c74646; }
.alert-item span { color: #68766f; font-size: 11px; overflow-wrap: anywhere; }
.empty-state { padding: 18px; background: #ffffff; border: 1px solid #cbd7d1; border-radius: 6px; color: #68766f; }

.evidence-panel { position: sticky; top: 90px; margin-top: 24px; background: #ffffff; border: 1px solid #cbd7d1; border-radius: 6px; padding: 14px; }
.source-list, .boundary-list { display: grid; }
.source-item, .boundary-item { display: grid; gap: 4px; padding: 10px 0; border-bottom: 1px solid #e1e7e4; font-size: 11px; overflow-wrap: anywhere; }
.source-item strong, .boundary-item strong { color: #405048; }
.boundary-item span { color: #087944; font-weight: 700; }

.status-bar { display: flex; flex-wrap: wrap; gap: 18px; min-height: 34px; align-items: center; padding: 7px 20px; background: #17231e; color: #d7e0dc; font-size: 11px; }

@media (max-width: 980px) {
  .axis-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .content-grid { grid-template-columns: minmax(0, 1fr); }
  .evidence-panel { position: static; }
}

@media (max-width: 680px) {
  .app-shell { grid-template-columns: minmax(0, 1fr); }
  .app-shell { height: auto; min-height: 100vh; overflow: visible; }
  .sidebar { position: static; height: auto; padding: 10px; gap: 10px; }
  .workspace { min-height: 0; display: block; }
  .brand-mark { width: 26px; height: 26px; font-size: 12px; }
  .sidebar nav { display: flex; overflow-x: auto; }
  .sidebar nav a { white-space: nowrap; padding: 7px 9px; }
  .sidebar-state { display: none; }
  .context-bar { min-height: 64px; padding: 10px 12px; }
  .context-primary strong { font-size: 15px; }
  main { overflow: visible; padding: 12px 8px 24px; }
  .connection-banner { align-items: flex-start; }
  .axis-grid, .node-grid, .observability-grid { grid-template-columns: minmax(0, 1fr); }
  .lifecycle-action-panel { grid-template-columns: minmax(0, 1fr); align-items: start; }
  .impact-band { align-items: flex-start; flex-direction: column; }
  .impact-list { justify-content: flex-start; }
  .event-correlation-band { grid-template-columns: minmax(0, 1fr); }
  .portal-link { justify-self: start; white-space: normal; }
  .section-heading h1 { font-size: 21px; }
  .card-meta { grid-template-columns: 58px minmax(0, 1fr); }
  .table-wrap { overflow-x: visible; }
  table, tbody, tr, td { display: block; min-width: 0; width: 100%; }
  thead { display: none; }
  tr { padding: 7px 0; border-bottom: 1px solid #e1e7e4; }
  td { display: grid; grid-template-columns: 94px minmax(0, 1fr); gap: 8px; border: 0; padding: 5px 10px; overflow-wrap: anywhere; }
  td::before { content: attr(data-label); color: #53615a; font-weight: 800; }
  .status-bar { gap: 10px; padding: 7px 10px; }
}
"#;

pub(super) const CONTROL_CENTER_JS: &str = r#"const SHARED_STATUS_URL = "/api/mvp/v1/status";
const OPS_SNAPSHOT_URL = "/api/mvp/v1/control-center";
const EVENT_CORRELATION_URL = "/api/mvp/v1/event-correlation";
const EXPECTED_SHARED_SCHEMA = "ntpro.mvp_shared_status_api.response.v1";
const EXPECTED_SHARED_CONTRACT = "ntpro.mvp_shared_status_api.v1";
const EXPECTED_EVENT_SCHEMA = "ntpro.mvp_event_correlation_api.response.v1";
const EXPECTED_EVENT_CONTRACT = "ntpro.mvp_event_correlation_api.v1";
const EXPECTED_IDENTITY_SCHEMA = "ntpro.mvp_identity_contract.v1";
const EXPECTED_STATUS_SCHEMA = "ntpro.mvp_status_contract.v1";
const EXPECTED_OPS_SCHEMA = "ntpro.mvp_control_center_snapshot.v2";
const EXPECTED_LIFECYCLE_ACTION_SCHEMA = "ntpro.mvp_control_center_lifecycle_action.response.v1";
const EXPECTED_LIFECYCLE_ACTION_CONTRACT = "ntpro.mvp_control_center_lifecycle_action.v1";
const STATUS_AVAILABILITIES = ["available", "missing", "stale", "error"];
const STATUS_FRESHNESS = ["fresh", "stale", "unknown"];
const BUSINESS_AVAILABILITIES = ["available", "missing", "stale", "error", "identity_mismatch"];
const BUSINESS_HEALTH = ["healthy", "degraded", "unhealthy", "unknown"];
const DASHBOARD_AVAILABILITIES = ["available", "not_configured", "not_supported", "stale", "redacted", "unknown"];
const LIFECYCLE_STATES = ["stopped", "starting", "running", "pausing", "paused", "resuming", "stopping", "error", "unknown"];
const PROCESS_MODES = ["spawned_process", "test_harness", "unknown"];
const PROCESS_STATES = ["not_started", "running", "stopped", "stale", "unknown"];
const HEALTH_STATES = ["healthy", "degraded", "error", "stale", "unknown"];
const CONNECTION_STATES = ["connected", "connecting", "disconnected", "disconnecting", "not_configured", "not_supported", "stale", "unknown"];
const API_FALSE_BOUNDARIES = ["http_success_implies_technical_health", "process_alive_implies_technical_health", "backtest_reference_implies_research_accepted", "backtest_complete_implies_trading_readiness", "raw_event_store_exposed", "raw_venue_payload_exposed", "external_venue_connection", "order_submission_allowed", "order_mutation_allowed", "automatic_retry_allowed", "automatic_remediation_allowed", "real_orders_submitted"];
const CONTRACT_FALSE_BOUNDARIES = ["external_venue_connection", "order_submission_allowed", "order_mutation_allowed", "automatic_retry_allowed", "automatic_remediation_allowed", "real_orders_submitted"];
const STATUS_FALSE_BOUNDARIES = ["http_success_implies_technical_health", "process_alive_implies_technical_health", "backtest_complete_implies_trading_readiness"];
const LIFECYCLE_ACTIONS = ["start", "stop"];
const LIFECYCLE_ACTION_STATUSES = ["accepted", "rejected", "running", "succeeded", "failed", "cancelled", "not_supported", "unknown"];

let lifecycleActionBusy = false;
let pendingLifecycleAction = null;
let currentLifecycleActions = [];

const safe = (value) => value === null || value === undefined ? "unknown" : String(value);
const text = (value) => safe(value).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;");
const DISPLAY = { available: "可用", missing: "缺失", stale: "陈旧", error: "错误", fresh: "新鲜", unknown: "未知", healthy: "健康", degraded: "降级", unhealthy: "不健康", not_running: "未运行", running: "运行中", stopped: "已停止", transitioning: "转换中", blocked: "阻断", reference_bound: "已绑定引用", sandbox: "沙盒", connected: "已连接", disconnected: "已断开", not_configured: "未配置", spawned_process: "托管进程", local: "本地", start: "启动", stop: "停止", ready: "可执行", requires_stopped: "仅已停止节点可启动", requires_running_or_paused: "仅运行中或已暂停节点可停止", succeeded: "已成功", rejected: "已拒绝", failed: "失败" };
const display = (value) => DISPLAY[safe(value)] || safe(value);
const dashboardValue = (value) => value && typeof value === "object" ? value.value ?? value.availability ?? "unknown" : "unknown";

function requireObject(value, field) {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`状态合同缺少 ${field}`);
  return value;
}
function requireString(value, field) {
  if (typeof value !== "string" || value.trim().length === 0) throw new Error(`状态合同缺少 ${field}`);
  return value;
}
function requireOneOf(value, allowed, field) {
  if (!allowed.includes(value)) throw new Error(`状态合同字段异常：${field}`);
  return value;
}
function requirePositiveInteger(value, field) {
  if (!Number.isSafeInteger(value) || value <= 0) throw new Error(`状态合同字段异常：${field}`);
  return value;
}
function requireNonNegativeInteger(value, field) {
  if (!Number.isSafeInteger(value) || value < 0) throw new Error(`状态合同字段异常：${field}`);
  return value;
}
function requireBoolean(value, field) {
  if (typeof value !== "boolean") throw new Error(`状态合同字段异常：${field}`);
  return value;
}
function requireStringArray(value, field, allowEmpty = true) {
  if (!Array.isArray(value) || (!allowEmpty && value.length === 0) || value.some((item) => typeof item !== "string" || item.trim().length === 0)) throw new Error(`状态合同缺少 ${field}`);
  return value;
}
function requireDashboardValue(value, field) {
  const item = requireObject(value, field);
  requireOneOf(item.availability, DASHBOARD_AVAILABILITIES, `${field}.availability`);
  const hasValue = Object.prototype.hasOwnProperty.call(item, "value") && item.value !== null;
  if (item.availability === "available" && !hasValue) throw new Error(`状态合同缺少 ${field}.value`);
  if (item.availability !== "available" && hasValue) throw new Error(`状态合同字段异常：${field}.value`);
  return item;
}
function requireDashboardString(value, field) {
  const item = requireDashboardValue(value, field);
  if (item.value !== undefined && item.value !== null && typeof item.value !== "string") throw new Error(`状态合同字段异常：${field}.value`);
  return item;
}
function requireDashboardBoolean(value, field) {
  const item = requireDashboardValue(value, field);
  if (item.value !== undefined && item.value !== null && typeof item.value !== "boolean") throw new Error(`状态合同字段异常：${field}.value`);
  return item;
}
function requireDashboardInteger(value, field) {
  const item = requireDashboardValue(value, field);
  if (item.value !== undefined && item.value !== null && (!Number.isSafeInteger(item.value) || item.value < 0)) throw new Error(`状态合同字段异常：${field}.value`);
  return item;
}
function requireAxis(value, field, allowedStatuses) {
  const axis = requireObject(value, field);
  requireOneOf(axis.status, allowedStatuses, `${field}.status`);
  requireOneOf(axis.availability, STATUS_AVAILABILITIES, `${field}.availability`);
  requireOneOf(axis.freshness, STATUS_FRESHNESS, `${field}.freshness`);
  requireStringArray(axis.source_refs, `${field}.source_refs`, false);
  requireStringArray(axis.reasons, `${field}.reasons`);
  requirePositiveInteger(axis.observed_at_unix_ms, `${field}.observed_at_unix_ms`);
  const hasError = Object.prototype.hasOwnProperty.call(axis, "error") && axis.error !== null;
  if (hasError) requireString(axis.error, `${field}.error`);
  if ((axis.availability === "error") !== hasError) throw new Error(`状态合同错误信封异常：${field}`);
  return axis;
}
function requireBoundary(boundaries, field, expected, scope) {
  if (!Object.prototype.hasOwnProperty.call(boundaries, field) || boundaries[field] !== expected) throw new Error(`${scope} 只读边界异常：${field}`);
}
function validateSharedStatus(payload) {
  requireObject(payload, "payload");
  if (payload.schema_version !== EXPECTED_SHARED_SCHEMA) throw new Error("共享状态 schema 不匹配");
  if (payload.contract_version !== EXPECTED_SHARED_CONTRACT) throw new Error("共享状态 contract 不匹配");
  if (!Array.isArray(payload.consumers) || !payload.consumers.includes("control_center")) throw new Error("共享状态未授权控制中心消费");
  const identity = requireObject(payload.identity, "identity");
  const identities = requireObject(identity.identities, "identity.identities");
  const identityProvenance = requireObject(identity.provenance, "identity.provenance");
  const status = requireObject(payload.status, "status");
  const statusProvenance = requireObject(status.provenance, "status.provenance");
  const business = requireObject(payload.business, "business");
  const apiBoundaries = requireObject(payload.boundaries, "boundaries");
  const identityBoundaries = requireObject(identity.boundaries, "identity.boundaries");
  const statusBoundaries = requireObject(status.boundaries, "status.boundaries");
  if (identity.schema_version !== EXPECTED_IDENTITY_SCHEMA) throw new Error("身份合同 schema 不匹配");
  if (status.schema_version !== EXPECTED_STATUS_SCHEMA) throw new Error("状态合同 schema 不匹配");
  requireString(identity.contract_id, "identity.contract_id");
  for (const field of ["strategy_id", "strategy_version", "backtest_run_id", "backtest_result_ref", "node_id", "strategy_instance_id", "account_id", "venue_id", "environment"]) requireString(identities[field], `identity.identities.${field}`);
  if (identities.environment !== "sandbox") throw new Error("控制中心 MVP 仅允许 sandbox 环境");
  if (identity.contract_id !== `${identities.node_id}:${identities.strategy_id}:${identities.strategy_instance_id}`) throw new Error("身份合同 ID 与运行身份不一致");
  if (status.identity_contract_id !== identity.contract_id) throw new Error("状态合同与身份合同不一致");
  requireString(identityProvenance.config_path, "identity.provenance.config_path");
  requirePositiveInteger(identityProvenance.generated_at_unix_ms, "identity.provenance.generated_at_unix_ms");
  for (const field of ["identity_contract_path", "supervisor_registry_path", "node_status_path", "node_metrics_path", "unified_read_model_path"]) requireString(statusProvenance[field], `status.provenance.${field}`);
  if (statusProvenance.identity_contract_available !== true) throw new Error("状态合同身份来源不可用");
  requirePositiveInteger(statusProvenance.freshness_max_age_ms, "status.provenance.freshness_max_age_ms");
  requirePositiveInteger(statusProvenance.generated_at_unix_ms, "status.provenance.generated_at_unix_ms");
  requireBoundary(apiBoundaries, "read_only", true, "API");
  API_FALSE_BOUNDARIES.forEach((field) => requireBoundary(apiBoundaries, field, false, "API"));
  requireBoundary(identityBoundaries, "read_only_product_contract", true, "身份合同");
  CONTRACT_FALSE_BOUNDARIES.forEach((field) => requireBoundary(identityBoundaries, field, false, "身份合同"));
  requireBoundary(statusBoundaries, "read_only_product_contract", true, "状态合同");
  STATUS_FALSE_BOUNDARIES.forEach((field) => requireBoundary(statusBoundaries, field, false, "状态合同"));
  requirePositiveInteger(payload.generated_at_unix_ms, "generated_at_unix_ms");
  requireStringArray(payload.source_refs, "source_refs", false);
  requireAxis(status.research, "status.research", ["reference_bound"]);
  requireAxis(status.runtime, "status.runtime", ["running", "stopped", "transitioning", "unknown"]);
  const technicalHealth = requireAxis(status.technical_health, "status.technical_health", ["healthy", "degraded", "unhealthy", "not_running", "unknown"]);
  if (technicalHealth.status === "healthy" && (technicalHealth.availability !== "available" || technicalHealth.freshness !== "fresh" || (technicalHealth.error !== undefined && technicalHealth.error !== null))) throw new Error("健康状态与可用性、时效或错误信封不一致");
  requireAxis(status.trading_readiness, "status.trading_readiness", ["blocked"]);
  requireOneOf(business.availability, BUSINESS_AVAILABILITIES, "business.availability");
  requireOneOf(business.health, BUSINESS_HEALTH, "business.health");
  for (const field of ["readiness_status", "snapshot_id", "schema_version", "freshness_status", "source_type", "source_ref", "redaction_state", "blocking_reasons", "diagnostic"]) requireDashboardValue(business[field], `business.${field}`);
  for (const component of ["account", "positions", "orders", "fills", "risk", "lifecycle"]) {
    const value = requireObject(business[component], `business.${component}`);
    for (const field of ["status", "summary", "freshness_status", "source_ref", "redaction_state"]) requireDashboardValue(value[field], `business.${component}.${field}`);
  }
  return payload;
}
function requestedEventId() {
  const search = typeof location === "object" && typeof location.search === "string" ? location.search : "";
  const values = new URLSearchParams(search).getAll("event_id");
  if (values.length > 1) throw new Error("请求包含重复事件参数");
  return values.length === 1 ? values[0] : null;
}
function validateEventCorrelation(payload, shared) {
  const correlation = requireObject(payload, "event correlation");
  if (correlation.schema_version !== EXPECTED_EVENT_SCHEMA) throw new Error("事件关联 schema 不匹配");
  if (correlation.contract_version !== EXPECTED_EVENT_CONTRACT) throw new Error("事件关联 contract 不匹配");
  const event = requireObject(correlation.event, "event correlation.event");
  const links = requireObject(correlation.links, "event correlation.links");
  const boundaries = requireObject(correlation.boundaries, "event correlation.boundaries");
  for (const field of ["event_id", "event_kind", "event_source", "identity_contract_id", "node_id", "strategy_instance_id"]) requireString(event[field], `event correlation.event.${field}`);
  if (event.event_kind !== "technical_health_observation" || event.event_source !== "projected_status_contract") throw new Error("事件关联不是已投影状态观察");
  const identities = shared.identity.identities;
  const expectedEventId = `mvp-status:v1:${encodeURIComponent(identities.node_id)}:${encodeURIComponent(identities.strategy_id)}:${encodeURIComponent(identities.strategy_instance_id)}:technical-health`;
  if (event.event_id !== expectedEventId || event.identity_contract_id !== shared.identity.contract_id || event.node_id !== identities.node_id || event.strategy_instance_id !== identities.strategy_instance_id) throw new Error("事件关联与共享身份不一致");
  if (links.institution_workbench_path !== "/institution-workbench" || links.control_center_path !== "/control-center") throw new Error("事件关联目标路径异常");
  requireBoundary(boundaries, "read_only", true, "事件关联");
  requireBoundary(boundaries, "projected_status_event", true, "事件关联");
  for (const field of ["raw_event_store_exposed", "raw_event_payload_exposed", "raw_errors_exposed", "supervisor_actions_exposed", "trading_controls_exposed"]) requireBoundary(boundaries, field, false, "事件关联");
  const serialized = JSON.stringify(correlation);
  for (const forbidden of ["source_refs", "config_path", "registry_path", "node_status_path", "node_metrics_path", "unified_read_model_path", "last_error", "message", "credential", "controls"]) if (serialized.includes(`"${forbidden}"`)) throw new Error(`事件关联暴露禁止字段：${forbidden}`);
  const requested = requestedEventId();
  if (requested !== null && requested !== event.event_id) throw new Error("请求的事件与当前运行实例不一致");
  return correlation;
}
function portalEventLink(path, eventId) {
  return `${path}?event_id=${encodeURIComponent(eventId)}#event-correlation`;
}
function validateOperationalProjection(snapshot, shared) {
  const ops = requireObject(snapshot, "operational snapshot");
  if (ops.schema_version !== EXPECTED_OPS_SCHEMA) throw new Error("运维投影 schema 不匹配");
  if (ops.local_only !== true) throw new Error("控制中心只允许本地运维投影");
  requireString(ops.registry_path, "snapshot.registry_path");
  if (!shared.source_refs.includes(ops.registry_path) || shared.status.provenance.supervisor_registry_path !== ops.registry_path) throw new Error("共享状态与运维 registry provenance 不一致");
  requireDashboardString(ops.generated_at, "snapshot.generated_at");
  for (const forbidden of ["controls", "risk", "sandbox_business", "workflow_artifacts", "read_model_runtime", "strategy_runtime", "production_shadow", "production_mutation_evidence", "production_reconciliation_orphan", "production_cancel_recovery", "production_actual_cancel_audit", "production_order_lifecycle_audit"]) {
    if (Object.prototype.hasOwnProperty.call(ops, forbidden)) throw new Error(`运维投影暴露超范围字段：${forbidden}`);
  }
  const overview = requireObject(ops.overview, "snapshot.overview");
  for (const field of ["node_count", "running_nodes", "stopped_nodes", "error_nodes", "unknown_nodes"]) requireNonNegativeInteger(overview[field], `snapshot.overview.${field}`);
  if (overview.node_count !== 1) throw new Error("控制中心 MVP 要求恰好一个运维节点");
  requireOneOf(overview.health, HEALTH_STATES, "snapshot.overview.health");
  requireDashboardString(overview.latest_transition_at, "snapshot.overview.latest_transition_at");
  requireBoolean(overview.latest_error_present, "snapshot.overview.latest_error_present");
  for (const field of ["data_sources", "execution_gateways", "runtime_modules", "logs", "metrics", "alerts", "gaps", "lifecycle_actions"]) if (!Array.isArray(ops[field])) throw new Error(`运维投影缺少 ${field}`);
  const node = requireObject(ops.node, "snapshot.node");
  const identities = shared.identity.identities;
  if (node.node_id !== identities.node_id) throw new Error("共享状态与运维节点身份不一致");
  requireString(node.node_id, "snapshot.node.node_id");
  requireOneOf(node.lifecycle_state, LIFECYCLE_STATES, "snapshot.node.lifecycle_state");
  requireOneOf(node.process_mode, PROCESS_MODES, "snapshot.node.process_mode");
  requireOneOf(node.process_state, PROCESS_STATES, "snapshot.node.process_state");
  requireDashboardInteger(node.pid, "snapshot.node.pid");
  requireOneOf(node.health, HEALTH_STATES, "snapshot.node.health");
  requireDashboardString(node.last_transition_at, "snapshot.node.last_transition_at");
  requireBoolean(node.error_present, "snapshot.node.error_present");
  if (Object.prototype.hasOwnProperty.call(node, "last_error")) throw new Error("运维节点暴露未脱敏错误");
  const runtimeStatus = shared.status.runtime.status;
  const transitionalLifecycle = ["starting", "pausing", "paused", "resuming", "stopping"];
  if (runtimeStatus === "running" && (node.process_state !== "running" || node.lifecycle_state !== "running")) throw new Error("共享运行状态与节点进程或生命周期不一致");
  if (runtimeStatus === "transitioning" && (node.process_state !== "running" || !transitionalLifecycle.includes(node.lifecycle_state))) throw new Error("共享转换状态与节点进程或生命周期不一致");
  if (runtimeStatus === "stopped" && (!["stopped", "not_started"].includes(node.process_state) || node.lifecycle_state !== "stopped")) throw new Error("共享停止状态与节点进程或生命周期不一致");
  const expectedCounts = {
    running_nodes: node.lifecycle_state === "running" ? 1 : 0,
    stopped_nodes: node.lifecycle_state === "stopped" ? 1 : 0,
    error_nodes: node.lifecycle_state === "error" ? 1 : 0,
    unknown_nodes: node.lifecycle_state === "unknown" ? 1 : 0,
  };
  for (const [field, expected] of Object.entries(expectedCounts)) if (overview[field] !== expected) throw new Error(`运维总览与节点生命周期不一致：${field}`);
  if (overview.sandbox_only !== true) throw new Error("运维总览不在 sandbox-only 边界");
  const boundaries = requireObject(ops.boundaries, "snapshot.boundaries");
  requireBoundary(boundaries, "read_only", true, "运维投影");
  requireBoundary(boundaries, "supervisor_actions_exposed", true, "运维投影");
  for (const field of ["external_venue_connection", "production_venue_connection", "testnet_public_network_connection", "external_network_attempted", "real_orders_submitted", "unsupported_supervisor_actions_exposed", "trading_controls_exposed", "automatic_retry_allowed", "automatic_remediation_allowed", "raw_errors_exposed"]) requireBoundary(boundaries, field, false, "运维投影");
  if (ops.lifecycle_actions.length !== 2) throw new Error("控制中心必须且只能提供启动和停止动作");
  const seenActions = new Set();
  for (const [index, value] of ops.lifecycle_actions.entries()) {
    const item = requireObject(value, `snapshot.lifecycle_actions[${index}]`);
    requireOneOf(item.action, LIFECYCLE_ACTIONS, `snapshot.lifecycle_actions[${index}].action`);
    if (seenActions.has(item.action)) throw new Error("控制中心生命周期动作重复");
    seenActions.add(item.action);
    if (item.target_node_id !== node.node_id) throw new Error("生命周期动作目标节点不一致");
    if (item.method !== "POST") throw new Error("生命周期动作方法必须为 POST");
    requireBoolean(item.enabled, `snapshot.lifecycle_actions[${index}].enabled`);
    const expectedReason = item.enabled ? "ready" : item.action === "start" ? "requires_stopped" : "requires_running_or_paused";
    if (item.reason_code !== expectedReason) throw new Error("生命周期动作可用性原因不一致");
    const expectedEnabled = item.action === "start" ? node.lifecycle_state === "stopped" : ["running", "paused"].includes(node.lifecycle_state);
    if (item.enabled !== expectedEnabled) throw new Error("生命周期动作与节点状态不一致");
  }
  for (const action of LIFECYCLE_ACTIONS) if (!seenActions.has(action)) throw new Error(`控制中心缺少生命周期动作：${action}`);
  ops.data_sources.forEach((value, index) => {
    const item = requireObject(value, `snapshot.data_sources[${index}]`);
    requireString(item.source_id, `snapshot.data_sources[${index}].source_id`);
    for (const field of ["source_kind", "provider", "freshness"]) requireDashboardString(item[field], `snapshot.data_sources[${index}].${field}`);
    requireDashboardInteger(item.lag_ms, `snapshot.data_sources[${index}].lag_ms`);
    requireOneOf(item.connection, CONNECTION_STATES, `snapshot.data_sources[${index}].connection`);
    requireOneOf(item.health, HEALTH_STATES, `snapshot.data_sources[${index}].health`);
    requireBoolean(item.error_present, `snapshot.data_sources[${index}].error_present`);
  });
  ops.execution_gateways.forEach((value, index) => {
    const item = requireObject(value, `snapshot.execution_gateways[${index}]`);
    requireString(item.gateway_id, `snapshot.execution_gateways[${index}].gateway_id`);
    requireDashboardString(item.venue, `snapshot.execution_gateways[${index}].venue`);
    requireDashboardBoolean(item.started, `snapshot.execution_gateways[${index}].started`);
    requireDashboardString(item.last_report_at, `snapshot.execution_gateways[${index}].last_report_at`);
    requireOneOf(item.connection, CONNECTION_STATES, `snapshot.execution_gateways[${index}].connection`);
    requireBoolean(item.error_present, `snapshot.execution_gateways[${index}].error_present`);
  });
  ops.runtime_modules.forEach((value, index) => {
    const item = requireObject(value, `snapshot.runtime_modules[${index}]`);
    requireString(item.module_name, `snapshot.runtime_modules[${index}].module_name`);
    for (const field of ["status", "last_seen_at", "evidence_source"]) requireDashboardString(item[field], `snapshot.runtime_modules[${index}].${field}`);
    requireOneOf(item.health, HEALTH_STATES, `snapshot.runtime_modules[${index}].health`);
    requireBoolean(item.error_present, `snapshot.runtime_modules[${index}].error_present`);
  });
  ops.logs.forEach((value, index) => {
    const item = requireObject(value, `snapshot.logs[${index}]`);
    requireString(item.log_id, `snapshot.logs[${index}].log_id`);
    requireOneOf(item.availability, DASHBOARD_AVAILABILITIES, `snapshot.logs[${index}].availability`);
    requireDashboardString(item.last_seen_at, `snapshot.logs[${index}].last_seen_at`);
    requireBoolean(item.error_present, `snapshot.logs[${index}].error_present`);
  });
  ops.metrics.forEach((value, index) => {
    const item = requireObject(value, `snapshot.metrics[${index}]`);
    requireString(item.metric_id, `snapshot.metrics[${index}].metric_id`);
    requireDashboardString(item.value, `snapshot.metrics[${index}].value`);
    requireOneOf(item.availability, DASHBOARD_AVAILABILITIES, `snapshot.metrics[${index}].availability`);
    requireDashboardString(item.last_seen_at, `snapshot.metrics[${index}].last_seen_at`);
    requireBoolean(item.error_present, `snapshot.metrics[${index}].error_present`);
  });
  ops.alerts.forEach((value, index) => {
    const item = requireObject(value, `snapshot.alerts[${index}]`);
    requireString(item.alert_id, `snapshot.alerts[${index}].alert_id`);
    requireOneOf(item.severity, ["info", "warning", "error", "critical"], `snapshot.alerts[${index}].severity`);
    requireString(item.source, `snapshot.alerts[${index}].source`);
    if (Object.prototype.hasOwnProperty.call(item, "message")) throw new Error("运维告警暴露未脱敏错误");
  });
  ops.gaps.forEach((value, index) => {
    const item = requireObject(value, `snapshot.gaps[${index}]`);
    requireString(item.field_path, `snapshot.gaps[${index}].field_path`);
    requireOneOf(item.reason, DASHBOARD_AVAILABILITIES, `snapshot.gaps[${index}].reason`);
    requireDashboardString(item.owner_task, `snapshot.gaps[${index}].owner_task`);
    if (Object.prototype.hasOwnProperty.call(item, "notes")) throw new Error("运维缺口暴露未脱敏说明");
  });
  return { snapshot: ops, node, lifecycleActions: ops.lifecycle_actions };
}

function validateLifecycleActionEnvelope(payload, expectedNodeId, expectedAction) {
  const envelope = requireObject(payload, "lifecycle action response");
  if (envelope.schema_version !== EXPECTED_LIFECYCLE_ACTION_SCHEMA) throw new Error("生命周期动作响应 schema 不匹配");
  if (envelope.contract_version !== EXPECTED_LIFECYCLE_ACTION_CONTRACT) throw new Error("生命周期动作响应 contract 不匹配");
  if (envelope.local_only !== true) throw new Error("生命周期动作响应不是本地边界");
  if (envelope.target_node_id !== expectedNodeId || envelope.action_name !== expectedAction) throw new Error("生命周期动作响应身份不一致");
  const result = requireObject(envelope.result, "lifecycle action response.result");
  requireString(result.action_id, "lifecycle action response.result.action_id");
  if (result.action !== `${expectedAction}:${expectedNodeId}`) throw new Error("生命周期动作响应目标不一致");
  requireOneOf(result.status, LIFECYCLE_ACTION_STATUSES, "lifecycle action response.result.status");
  requireOneOf(result.previous_state, LIFECYCLE_STATES, "lifecycle action response.result.previous_state");
  requireOneOf(result.current_state, LIFECYCLE_STATES, "lifecycle action response.result.current_state");
  for (const field of ["started_at", "finished_at", "error_code", "message", "observability_ref"]) requireDashboardString(result[field], `lifecycle action response.result.${field}`);
  const boundaries = requireObject(envelope.boundaries, "lifecycle action response.boundaries");
  requireBoundary(boundaries, "supervisor_lifecycle_action", true, "生命周期动作响应");
  for (const field of ["external_venue_connection", "production_venue_connection", "external_network_attempted", "order_submission_allowed", "order_mutation_allowed", "automatic_retry_allowed", "automatic_remediation_allowed", "real_orders_submitted"]) requireBoundary(boundaries, field, false, "生命周期动作响应");
  const serialized = JSON.stringify(envelope);
  for (const forbidden of ["credential", "private_key", "auth_header", "raw_error", "raw_payload"]) if (serialized.includes(forbidden)) throw new Error(`生命周期动作响应暴露禁止字段：${forbidden}`);
  return envelope;
}

const emptyCard = (label) => `<div class="node-card"><div class="card-label">${text(label)}</div><div class="card-value">等待状态对齐</div></div>`;
function resetSurface(message) {
  pendingLifecycleAction = null;
  currentLifecycleActions = [];
  document.getElementById("axis-grid").innerHTML = ["研究状态", "运行状态", "技术健康", "交易准备度"].map(emptyCard).join("");
  document.getElementById("node-grid").innerHTML = ["生命周期", "进程", "技术健康", "PID", "最近转换", "错误标记"].map(emptyCard).join("");
  document.getElementById("lifecycle-action-buttons").innerHTML = `<button type="button" disabled>操作不可用</button>`;
  setLifecycleActionResult("等待已验证节点状态", message);
  document.getElementById("business-impact-title").textContent = "等待共享状态";
  document.getElementById("business-impact-list").innerHTML = "";
  document.getElementById("event-correlation-panel").innerHTML = `<strong>等待事件关联</strong><span>${text(message)}</span>`;
  document.getElementById("component-table").innerHTML = `<div class="empty-state">${text(message)}</div>`;
  document.getElementById("observability-grid").innerHTML = ["日志", "指标"].map(emptyCard).join("");
  document.getElementById("alert-list").innerHTML = `<div class="empty-state">${text(message)}</div>`;
  document.getElementById("source-list").innerHTML = `<div class="source-item"><strong>来源</strong>等待已验证合同</div>`;
  document.getElementById("boundary-list").innerHTML = `<div class="boundary-item"><strong>只读边界</strong><span>未验证，保持阻断</span></div>`;
  document.getElementById("context-node").textContent = "节点未加载";
  document.getElementById("context-scope").textContent = "策略 / 环境未加载";
  document.getElementById("generated-at").textContent = "尚未生成";
  document.getElementById("node-health").textContent = "等待节点";
  document.getElementById("alert-count").textContent = "0 项";
  document.getElementById("footer-environment").textContent = "环境：未知";
  document.getElementById("footer-node").textContent = "节点：未知";
  document.getElementById("footer-runtime").textContent = "运行状态：未知";
  document.getElementById("footer-health").textContent = "技术健康：未知";
  document.getElementById("footer-readiness").textContent = "交易准备度：阻断";
  document.getElementById("footer-updated").textContent = "更新时间：未知";
}
function setConnection(state, title, detail, badge) {
  document.getElementById("connection-banner").className = `connection-banner ${state}`;
  document.getElementById("connection-title").textContent = title;
  document.getElementById("connection-detail").textContent = detail;
  document.getElementById("connection-badge").textContent = badge;
  document.getElementById("sidebar-state").textContent = title;
  document.getElementById("sidebar-state-dot").className = `state-dot ${state}`;
}
function axisCard(label, axis) {
  const reasons = Array.isArray(axis.reasons) ? axis.reasons.join(" · ") : "无";
  return `<article class="axis-card"><div class="card-label">${text(label)}</div><div class="card-value status-${text(axis.status)}">${text(display(axis.status))}</div><div class="card-meta"><span>${text(display(axis.availability))} / ${text(display(axis.freshness))}</span><span>${text(reasons)}</span></div></article>`;
}
function nodeCard(label, value, meta = "") {
  return `<article class="node-card"><div class="card-label">${text(label)}</div><div class="card-value status-${text(safe(value))}">${text(display(value))}</div><div class="card-meta"><span>${text(meta || "已验证")}</span></div></article>`;
}
function componentRows(snapshot) {
  const rows = [];
  snapshot.data_sources.forEach((item) => rows.push(["数据源", item.source_id, dashboardValue(item.provider), item.connection, item.health, dashboardValue(item.freshness)]));
  snapshot.execution_gateways.forEach((item) => rows.push(["执行网关", item.gateway_id, dashboardValue(item.venue), item.connection, dashboardValue(item.started), dashboardValue(item.last_report_at)]));
  snapshot.runtime_modules.forEach((item) => rows.push(["运行模块", item.module_name, dashboardValue(item.status), item.health, dashboardValue(item.last_seen_at), dashboardValue(item.evidence_source)]));
  if (rows.length === 0) return `<div class="empty-state">没有组件上报</div>`;
  return `<table><thead><tr><th>类型</th><th>组件</th><th>对象</th><th>状态</th><th>健康 / 启动</th><th>时效 / 来源</th></tr></thead><tbody>${rows.map((row) => `<tr>${row.map((value, index) => `<td data-label="${["类型", "组件", "对象", "状态", "健康 / 启动", "时效 / 来源"][index]}" class="${index === 5 ? "path" : ""}">${text(display(value))}</td>`).join("")}</tr>`).join("")}</tbody></table>`;
}
function observabilityCard(label, items, idField, valueField) {
  const rows = items.length === 0 ? `<div class="observability-row"><span>没有${text(label)}上报</span><span>未知</span></div>` : items.map((item) => `<div class="observability-row"><span>${text(item[idField])}</span><span>${text(display(valueField ? dashboardValue(item[valueField]) : item.availability))}</span></div>`).join("");
  return `<article class="observability-card"><div class="card-label">${text(label)}</div>${rows}</article>`;
}
function setLifecycleActionResult(title, detail, error = false) {
  const result = document.getElementById("lifecycle-action-result");
  result.className = `lifecycle-action-result${error ? " error" : ""}`;
  result.innerHTML = `<strong>${text(title)}</strong><span>${text(detail)}</span>`;
}
function renderLifecycleActions(actions, nodeId) {
  currentLifecycleActions = Array.isArray(actions) ? actions : [];
  const container = document.getElementById("lifecycle-action-buttons");
  if (currentLifecycleActions.length !== 2) {
    container.innerHTML = `<button type="button" disabled>操作不可用</button>`;
    return;
  }
  container.innerHTML = currentLifecycleActions.map((item) => {
    const pending = pendingLifecycleAction === item.action;
    const disabled = lifecycleActionBusy || !item.enabled;
    const label = pending ? `确认${display(item.action)}` : display(item.action);
    return `<div class="lifecycle-action-option"><button type="button" class="lifecycle-action-button ${text(item.action)}${pending ? " pending" : ""}" data-lifecycle-action="${text(item.action)}" data-node-id="${text(nodeId)}" aria-pressed="${pending ? "true" : "false"}" ${disabled ? "disabled" : ""}>${text(label)}</button><span class="lifecycle-action-reason">${text(display(item.reason_code))}</span></div>`;
  }).join("");
}
function renderControlCenter(shared, ops, correlation) {
  const identities = shared.identity.identities;
  const status = shared.status;
  const snapshot = ops.snapshot;
  const node = ops.node;
  document.getElementById("context-node").textContent = node.node_id;
  document.getElementById("context-scope").textContent = `${identities.strategy_id} / ${identities.environment}`;
  document.getElementById("generated-at").textContent = `合同时间 ${shared.generated_at_unix_ms}`;
  document.getElementById("axis-grid").innerHTML = [axisCard("研究状态", status.research), axisCard("运行状态", status.runtime), axisCard("技术健康", status.technical_health), axisCard("交易准备度", status.trading_readiness)].join("");
  document.getElementById("business-impact-title").textContent = `${display(shared.business.health)} / ${display(shared.business.availability)}`;
  document.getElementById("business-impact-list").innerHTML = [["账户", shared.business.account], ["持仓", shared.business.positions], ["订单", shared.business.orders], ["成交", shared.business.fills], ["风险", shared.business.risk], ["生命周期", shared.business.lifecycle]].map(([label, value]) => `<span>${text(label)}：${text(display(dashboardValue(value.status)))}</span>`).join("");
  const event = correlation.event;
  document.getElementById("event-correlation-panel").innerHTML = `<strong>${text(event.event_id)}</strong><span>技术根因：${text(display(status.technical_health.status))} · 业务影响：${text(display(shared.business.health))}</span><a class="portal-link" href="${text(portalEventLink(correlation.links.institution_workbench_path, event.event_id))}">在机构工作台查看业务影响</a>`;
  document.getElementById("node-health").textContent = `${display(node.health)} / ${display(node.lifecycle_state)}`;
  document.getElementById("node-grid").innerHTML = [
    nodeCard("生命周期", node.lifecycle_state, "Supervisor lifecycle"),
    nodeCard("进程", node.process_state, display(node.process_mode)),
    nodeCard("节点健康", node.health, "不能替代共享技术健康"),
    nodeCard("PID", dashboardValue(node.pid), "本地进程标识"),
    nodeCard("最近转换", dashboardValue(node.last_transition_at), "节点状态时间"),
    nodeCard("错误标记", node.error_present ? "present" : "none", "不传输原始错误文本"),
  ].join("");
  renderLifecycleActions(ops.lifecycleActions, node.node_id);
  document.getElementById("component-table").innerHTML = componentRows(snapshot);
  document.getElementById("observability-grid").innerHTML = [observabilityCard("日志", snapshot.logs, "log_id"), observabilityCard("指标", snapshot.metrics, "metric_id", "value")].join("");
  const alerts = [...snapshot.alerts.map((item) => ({ severity: item.severity, title: item.alert_id, detail: `${item.source} · 原始错误已脱敏` })), ...snapshot.gaps.map((item) => ({ severity: "warning", title: item.field_path, detail: `${display(item.reason)} · ${display(dashboardValue(item.owner_task))}` }))];
  document.getElementById("alert-count").textContent = `${alerts.length} 项`;
  document.getElementById("alert-list").innerHTML = alerts.length === 0 ? `<div class="empty-state">没有活动告警或能力缺口</div>` : alerts.map((item) => `<article class="alert-item ${text(item.severity)}"><strong>${text(item.title)}</strong><span>${text(item.detail)}</span></article>`).join("");
  document.getElementById("source-list").innerHTML = [...new Set([...shared.source_refs, snapshot.registry_path])].map((source, index) => `<div class="source-item"><strong>来源 ${index + 1}</strong>${text(source)}</div>`).join("");
  document.getElementById("boundary-list").innerHTML = [["共享合同", shared.contract_version], ["Control Center consumer", "已验证"], ["本地监督器", "启动 / 停止"], ["其他 Supervisor 动作", "关闭"], ["外部 Venue", "关闭"], ["订单提交与变更", "关闭"], ["自动重试与补救", "关闭"], ["真实订单", "关闭"]].map(([label, value]) => `<div class="boundary-item"><strong>${text(label)}</strong><span>${text(value)}</span></div>`).join("");
  document.getElementById("footer-environment").textContent = `环境：${display(identities.environment)}`;
  document.getElementById("footer-node").textContent = `节点：${node.node_id}`;
  document.getElementById("footer-runtime").textContent = `运行状态：${display(status.runtime.status)}`;
  document.getElementById("footer-health").textContent = `技术健康：${display(status.technical_health.status)}`;
  document.getElementById("footer-readiness").textContent = `交易准备度：${display(status.trading_readiness.status)}`;
  document.getElementById("footer-updated").textContent = `更新时间：${shared.generated_at_unix_ms}`;
  setConnection("ready", "共享与运维状态已对齐", "控制中心正在消费版本化共享事实和受控本地生命周期能力", "受控");
}
function renderBlocked(error) {
  resetSurface("状态不可用，旧数据已清空");
  setConnection("blocked", "控制中心已阻断", error.message, "Fail closed");
}
async function refreshControlCenter() {
  const button = document.getElementById("refresh");
  button.disabled = true;
  resetSurface("刷新中，旧数据已清空");
  setConnection("loading", "正在对齐共享与运维状态", "等待版本化合同和本地监督器投影", "读取中");
  try {
    const options = { method: "GET", headers: { "Accept": "application/json" }, cache: "no-store" };
    const [sharedResponse, snapshotResponse, correlationResponse] = await Promise.all([fetch(SHARED_STATUS_URL, options), fetch(OPS_SNAPSHOT_URL, options), fetch(EVENT_CORRELATION_URL, options)]);
    if (!sharedResponse.ok) throw new Error(`共享状态不可用（HTTP ${sharedResponse.status}）`);
    if (!snapshotResponse.ok) throw new Error(`运维投影不可用（HTTP ${snapshotResponse.status}）`);
    if (!correlationResponse.ok) throw new Error(`事件关联不可用（HTTP ${correlationResponse.status}）`);
    const shared = validateSharedStatus(await sharedResponse.json());
    const ops = validateOperationalProjection(await snapshotResponse.json(), shared);
    renderControlCenter(shared, ops, validateEventCorrelation(await correlationResponse.json(), shared));
    return true;
  } catch (error) {
    renderBlocked(error instanceof Error ? error : new Error("控制中心状态读取失败"));
    return false;
  } finally {
    button.disabled = false;
  }
}

async function refreshUntilLifecycleAligned(timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  do {
    if (await refreshControlCenter()) return;
    await new Promise((resolve) => setTimeout(resolve, 200));
  } while (Date.now() < deadline);
  throw new Error("生命周期动作后状态投影未在时限内对齐");
}

async function executeLifecycleAction(action, nodeId) {
  const capability = currentLifecycleActions.find((item) => item.action === action && item.target_node_id === nodeId);
  if (!capability || !capability.enabled || lifecycleActionBusy) throw new Error("生命周期动作当前不可执行");
  lifecycleActionBusy = true;
  renderLifecycleActions(currentLifecycleActions, nodeId);
  setLifecycleActionResult(`正在${display(action)}`, `${nodeId} · 等待 Supervisor 响应`);
  try {
    const endpoint = `/api/mvp/v1/control-center/nodes/${encodeURIComponent(nodeId)}/actions/${encodeURIComponent(action)}`;
    const response = await fetch(endpoint, { method: "POST", headers: { "Accept": "application/json" }, cache: "no-store" });
    const envelope = validateLifecycleActionEnvelope(await response.json(), nodeId, action);
    const result = envelope.result;
    if (response.ok && result.status !== "succeeded") throw new Error("生命周期动作成功响应状态异常");
    if (!response.ok && result.status === "succeeded") throw new Error("生命周期动作错误响应状态异常");
    if (response.ok) {
      const expectedPrevious = action === "start" ? "stopped" : ["running", "paused"];
      const previousMatches = Array.isArray(expectedPrevious) ? expectedPrevious.includes(result.previous_state) : result.previous_state === expectedPrevious;
      const expectedCurrent = action === "start" ? "running" : "stopped";
      if (!previousMatches || result.current_state !== expectedCurrent) throw new Error("生命周期动作状态转换异常");
    }
    pendingLifecycleAction = null;
    await refreshUntilLifecycleAligned();
    const message = dashboardValue(result.message);
    setLifecycleActionResult(`${display(action)}${display(result.status)}`, `${message} · ${result.previous_state} → ${result.current_state}`, !response.ok);
  } catch (error) {
    currentLifecycleActions = [];
    renderLifecycleActions([], nodeId);
    setLifecycleActionResult("生命周期动作已阻断", error instanceof Error ? error.message : "动作响应不可验证", true);
  } finally {
    lifecycleActionBusy = false;
    pendingLifecycleAction = null;
    if (currentLifecycleActions.length === 2) renderLifecycleActions(currentLifecycleActions, nodeId);
  }
}

document.getElementById("refresh").addEventListener("click", refreshControlCenter);
document.addEventListener("click", (event) => {
  const button = event.target.closest("[data-lifecycle-action]");
  if (!button || button.disabled) return;
  const action = button.getAttribute("data-lifecycle-action");
  const nodeId = button.getAttribute("data-node-id");
  if (!LIFECYCLE_ACTIONS.includes(action) || typeof nodeId !== "string" || nodeId.length === 0) {
    setLifecycleActionResult("生命周期动作已阻断", "动作身份不可验证", true);
    return;
  }
  if (pendingLifecycleAction !== action) {
    pendingLifecycleAction = action;
    renderLifecycleActions(currentLifecycleActions, nodeId);
    setLifecycleActionResult(`确认${display(action)}`, `${nodeId} · 再次点击确认执行`);
    return;
  }
  executeLifecycleAction(action, nodeId);
});
refreshControlCenter();
"#;
