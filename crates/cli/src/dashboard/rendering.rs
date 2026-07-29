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

//! Dashboard 静态 HTML、CSS 与 JavaScript 渲染资源。

pub(super) const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>NTPRO 控制台</title>
  <link rel="stylesheet" href="/assets/dashboard.css">
</head>
<body>
  <header class="topbar">
    <div>
      <h1>NTPRO 监督器控制台</h1>
      <p>本页面只查看本地监督器工件，不会连接外部交易场所，也不会提交真实订单。</p>
    </div>
    <button id="refresh" type="button">刷新</button>
  </header>
  <main>
    <section class="band">
      <h2>概览</h2>
      <div id="overview" class="grid"></div>
    </section>
    <section class="band">
      <h2>Binance 沙盒业务状态</h2>
      <div id="sandbox-business" class="grid"></div>
    </section>
    <section class="band">
      <h2>Workflow 工件</h2>
      <div id="workflow-artifacts" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Trader Terminal Workbench</h2>
      <div id="trader-terminal-workbench" class="workbench-shell"></div>
    </section>
    <section class="band">
      <h2>Unified Read Model</h2>
      <div id="read-model-runtime" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Strategy Runtime</h2>
      <div id="strategy-runtime" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Production Shadow</h2>
      <div id="production-shadow" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>预检就绪</h2>
      <div id="preflight-readiness" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Live Alpha Dry-run</h2>
      <div id="live-alpha-dry-run" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>v0.16 Production Mutation Evidence</h2>
      <div id="production-mutation-evidence" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>v0.17 对账与孤儿单风险</h2>
      <div id="production-reconciliation-orphan" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>v0.18 撤单恢复只读面板</h2>
      <div id="production-cancel-recovery" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>v0.19 真实撤单审计只读视图</h2>
      <div id="production-actual-cancel-audit" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>v0.20 订单生命周期审计只读视图</h2>
      <div id="production-order-lifecycle-audit" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>节点</h2>
      <div id="nodes" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>控制</h2>
      <div id="controls" class="table-wrap"></div>
      <div id="control-result" class="list"></div>
    </section>
    <section class="band">
      <h2>数据源</h2>
      <div id="data-sources" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>执行网关</h2>
      <div id="execution-gateways" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>风控引擎</h2>
      <div id="risk" class="grid"></div>
    </section>
    <section class="band">
      <h2>运行模块</h2>
      <div id="runtime-modules" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>日志 / 指标</h2>
      <div id="logs-metrics" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>告警</h2>
      <div id="alerts" class="list"></div>
    </section>
    <section class="band">
      <h2>待补能力</h2>
      <div id="gaps" class="list"></div>
    </section>
  </main>
  <script src="/assets/dashboard.js"></script>
</body>
</html>
"#;

pub(super) const DASHBOARD_CSS: &str = r#":root {
  color-scheme: light;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #f6f7f9;
  color: #1f2933;
}

body {
  margin: 0;
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

.topbar {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  align-items: center;
  padding: 24px 32px;
  background: #111827;
  color: #ffffff;
}

.topbar h1 {
  margin: 0 0 6px;
  font-size: 28px;
  letter-spacing: 0;
}

.topbar p {
  margin: 0;
  color: #cbd5e1;
}

button {
  border: 1px solid #94a3b8;
  background: #ffffff;
  color: #111827;
  border-radius: 6px;
  padding: 8px 12px;
  font-weight: 600;
  cursor: pointer;
}

main {
  width: min(1180px, calc(100vw - 32px));
  margin: 24px auto 48px;
}

.band {
  margin: 0 0 28px;
}

.band h2 {
  font-size: 18px;
  margin: 0 0 12px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
  gap: 12px;
}

.tile,
.row {
  background: #ffffff;
  border: 1px solid #d8dee6;
  border-radius: 6px;
  padding: 12px;
}

.label {
  color: #64748b;
  font-size: 12px;
  text-transform: uppercase;
}

.value {
  margin-top: 6px;
  font-size: 18px;
  font-weight: 700;
  overflow-wrap: anywhere;
}

.panel-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  border-top: 1px solid #edf1f6;
  padding-top: 8px;
  margin-top: 8px;
}

.panel-row span:last-child {
  text-align: right;
  overflow-wrap: anywhere;
}

.table-wrap {
  overflow-x: auto;
  border: 1px solid #d8dee6;
  border-radius: 6px;
  background: #ffffff;
}

table {
  width: 100%;
  border-collapse: collapse;
  min-width: 980px;
}

th,
td {
  padding: 10px 12px;
  border-bottom: 1px solid #e5eaf0;
  text-align: left;
  vertical-align: top;
}

th {
  background: #eef2f6;
  color: #334155;
  font-size: 12px;
  text-transform: uppercase;
}

td {
  font-size: 13px;
}

.path {
  max-width: 260px;
  overflow-wrap: anywhere;
}

.muted {
  color: #64748b;
}

.list {
  display: grid;
  gap: 8px;
}

.row {
  display: grid;
  gap: 4px;
}

.workbench-shell {
  display: grid;
  gap: 12px;
}

.workbench-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.workbench-tab {
  background: #ffffff;
  border: 1px solid #d8dee6;
  border-radius: 6px;
  color: #334155;
  font-weight: 700;
  padding: 8px 10px;
}

.workbench-tab[aria-selected="true"] {
  background: #111827;
  color: #ffffff;
}

.workbench-panels,
.workbench-boundary {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.workbench-panel,
.workbench-drawer {
  background: #ffffff;
  border: 1px solid #d8dee6;
  border-radius: 6px;
  padding: 12px;
}

.workbench-panel h3,
.workbench-drawer summary {
  font-size: 14px;
  font-weight: 700;
  margin: 0 0 8px;
}

.status-healthy {
  color: #166534;
}

.status-error,
.status-stale {
  color: #991b1b;
}

.status-degraded,
.status-unknown {
  color: #92400e;
}

@media (max-width: 720px) {
  main {
    width: min(100%, calc(100vw - 24px));
    margin-top: 18px;
  }

  .topbar {
    align-items: flex-start;
    flex-direction: column;
    padding: 20px 16px;
  }

  .topbar h1 {
    font-size: 24px;
  }

  .topbar button,
  td button {
    max-width: 100%;
    white-space: normal;
  }

  .table-wrap {
    overflow-x: visible;
  }

  table,
  tbody,
  tr,
  td {
    display: block;
    min-width: 0;
    width: 100%;
  }

  table {
    min-width: 0;
  }

  thead {
    display: none;
  }

  tr {
    border-bottom: 1px solid #e5eaf0;
    padding: 8px 0;
  }

  tr:last-child {
    border-bottom: 0;
  }

  td {
    display: grid;
    grid-template-columns: 116px minmax(0, 1fr);
    gap: 10px;
    border-bottom: 0;
    padding: 6px 12px;
  }

  td::before {
    content: attr(data-label);
    color: #475569;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .path {
    max-width: none;
  }
}
"#;

pub(super) const DASHBOARD_JS: &str = r#"const renderTile = (label, value, extraClass = "") =>
  `<div class="tile ${extraClass}"><div class="label">${text(label)}</div><div class="value">${text(value)}</div></div>`;

const safe = (value) => value === null || value === undefined ? "unknown" : String(value);

const DISPLAY_TEXT = {
  "true": "是",
  "false": "否",
  unknown: "未知",
  available: "可用",
  not_supported: "不支持",
  redacted: "已脱敏",
  "present (redacted)": "存在（已脱敏）",
  none: "无",
  present: "存在",
  ok: "正常",
  record_only: "仅记录",
  mark_degraded: "标记降级",
  halt_shadow_flow: "停止 shadow 流",
  manual_review_required: "需要人工复核",
  missing_account_snapshot: "缺少账户快照",
  portfolio_unavailable: "组合不可用",
  shadow_intent_without_portfolio: "有 shadow 意图但无组合快照",
  production_mutation_forbidden: "检测到禁止的生产变更",
  running: "运行中",
  stopped: "已停止",
  starting: "启动中",
  stopping: "停止中",
  paused: "已暂停",
  pausing: "暂停中",
  resuming: "恢复中",
  healthy: "健康",
  stale: "已失联",
  degraded: "降级",
  error: "错误",
  not_started: "未启动",
  connected: "已连接",
  disconnected: "已断开",
  connecting: "连接中",
  disconnecting: "断开中",
  not_configured: "未配置",
  owner_proof_pack_artifacts_observed: "已观察到 owner proof pack 工件",
  not_included_default_offline_preflight: "默认离线预检，未包含 owner 在线成功证据",
  bounded_shadow_preflight_artifacts_observed: "已观察到 bounded shadow preflight 工件",
  bounded_shadow_preflight_contract_only: "仅合同边界，未观察到运行工件",
  decimal_boundary_contract_present: "Decimal 边界合同存在",
  no_production_mutation_boundary_ok: "无生产变更边界正常",
  no_production_mutation_boundary_violation: "检测到生产变更边界异常",
  v13_preflight_readiness_ok: "v0.13 预检只读边界正常",
  v13_preflight_readiness_degraded: "v0.13 预检边界降级",
  live_alpha_dry_run_ready: "Live Alpha dry-run 就绪",
  live_alpha_dry_run_blocked: "Live Alpha dry-run 阻断",
  live_alpha_dry_run_boundary_violation: "Live Alpha dry-run 边界异常",
  live_alpha_mutation_preflight_ready_for_owner_review: "Live Alpha mutation preflight 可人工复核",
  live_alpha_mutation_preflight_blocked: "Live Alpha mutation preflight 阻断",
  live_alpha_mutation_preflight_boundary_violation: "Live Alpha mutation preflight 边界异常",
  production_mutation_evidence_ready_for_owner_review: "v0.16 production mutation evidence 可人工复核",
  production_mutation_evidence_blocked: "v0.16 production mutation evidence 阻断",
  production_mutation_evidence_boundary_violation: "v0.16 production mutation evidence 边界异常",
  ready_signing_material_approval: "签名材料审批就绪",
  blocked_explicit_send_gate: "显式发送 gate 阻断",
  ready_request_builder_redacted: "脱敏请求构造就绪",
  ready_guarded_send_path_offline_no_network: "Guarded send 路径就绪，默认离线无网络",
  ready_response_redacted: "响应脱敏就绪",
  ready_offline_order_state_readback_contract: "离线订单状态 readback 合同就绪",
  ready_redacted_audit_trail: "脱敏审计链路就绪",
  ready_failure_semantics_evidence: "失败语义证据就绪",
  write_evidence_and_stop: "写入证据并停止",
  none_recorded: "未记录失败",
  ready_request_preview_only: "仅请求预览就绪",
  blocked_endpoint_or_owner_scope: "端点或 owner 范围阻断",
  blocked_manual_approval_lifecycle: "人工审批生命周期阻断",
  approval_valid_for_dry_run_request_preview: "审批可用于 dry-run 请求预览",
  ready_dry_run_execution_adapter_only: "仅本地 dry-run adapter 就绪",
  dry_run_adapter_artifact_only: "仅生成 dry-run adapter 工件",
  ready_runtime_gate_open_for_dry_run_only: "runtime gate 仅对 dry-run 打开",
  blocked_no_runtime_mutation: "阻断 runtime mutation",
  blocked_kill_switch_active: "kill switch 已激活",
  blocked_missing_manual_approval: "缺少人工审批",
  blocked_request_preview: "请求预览未就绪",
  allow_request_preview_only: "仅允许请求预览",
  production_mutation_owner_approved_manual_only: "生产 mutation 候选，需 owner 人工审批",
  created_in_memory_not_recorded: "仅内存创建，未记录",
  ready_dry_run_no_submission: "Dry-run 就绪，无真实提交",
  blocked_missing_gate: "缺少 owner gate",
  production_live_alpha_dry_run: "生产 Live Alpha dry-run",
  dry_run_no_submission: "Dry-run 不提交订单",
  approved: "通过",
  accepted: "已接收",
  succeeded: "成功",
  completed: "已完成",
  failed: "失败",
  rejected: "已拒绝",
  log: "日志",
  metric: "指标",
  local: "本地",
  supervisor_artifact: "监督器工件",
  node_not_found: "节点不存在",
  invalid_lifecycle_state: "生命周期状态不允许",
  unsupported_control_action: "控制动作不支持",
  sandbox_reconnect_not_supported: "本地沙盒重连不支持",
  process_state_conflict: "进程状态冲突",
  ntpro_node_binary_unavailable: "ntpro-node 二进制不可用",
  supervisor_action_failed: "监督器动作失败",
  snapshot_load_failed: "快照加载失败",
  logs_available: "日志可用",
  metrics_available: "指标可用",
  lifecycle_timeout: "生命周期操作超时",
  sandbox: "沙盒",
  "binance-sandbox": "Binance 沙盒",
  fixture_replay: "Fixture replay",
  mock_execution: "Mock execution",
  spawned_process: "托管进程",
  test_harness: "测试夹具",
  active: "活跃",
  reducing: "减仓中",
  halted: "已暂停交易",
  starting: "启动中",
  running: "运行中",
  exhausted: "已耗尽",
  paused: "已暂停",
  stopped: "已停止",
  failed: "失败",
  fixture_stream_running: "Fixture 流运行中",
  mock_stream_running: "Mock 流运行中",
  shadow: "影子模式",
  disabled: "已禁用",
  order_submission_disabled: "订单提交已禁用",
  pass: "通过",
  fail: "失败",
  ready: "就绪",
  risk_halted: "风险停止",
  schema_defined_redacted_preview_only: "只定义脱敏请求格式",
  schema_defined_offline_acceptance_not_attempted: "离线定义，未请求 order-test",
  manual_online_artifact_required_not_observed_offline: "需要人工在线证据，离线未观察",
  schema_defined_manual_or_fixture_input_required: "已定义格式，等待人工或夹具输入",
  shadow_mode_actual_submission_disabled: "影子模式未实际提交",
  blocked_by_v09_strategy_runtime_boundary: "被 v0.9 策略运行边界阻止",
  warning: "警告",
  info: "信息",
  missing: "缺失",
  invalid: "无效",
  configured: "已配置",
  missing_artifact: "缺失工件",
  canonical_unified_read_model_artifact_missing: "canonical read model 工件缺失",
  locked_readonly: "只读锁定",
  degraded_boundary: "边界降级",
  read_model_runtime_degraded: "read model runtime 降级",
  all_operation_controls_disabled: "所有操作控件禁用",
  operation_boundary_degraded: "操作边界降级",
  degraded_shell: "降级 shell",
  canonical_read_model: "canonical read model",
  required_before_any_manual_entry: "任何人工录入前必须审批",
  display_only: "仅展示",
};

const displayValue = (value) => {
  const normalized = safe(value);
  if (normalized.startsWith("unix_seconds:")) {
    return `时间戳秒：${normalized.slice("unix_seconds:".length)}`;
  }
  return DISPLAY_TEXT[normalized] || normalized;
};
const displayText = (value) => text(displayValue(value));

const text = (value) => safe(value)
  .replaceAll("&", "&amp;")
  .replaceAll("<", "&lt;")
  .replaceAll(">", "&gt;")
  .replaceAll("\"", "&quot;")
  .replaceAll("'", "&#39;");

const snapshotValue = (value) => {
  if (!value || typeof value !== "object") return "unknown";
  return value.value ?? value.availability ?? "unknown";
};

const availability = (value) => value && typeof value === "object" ? value.availability : "unknown";
const dashboardValueText = (value) => displayValue(snapshotValue(value));
const panelRow = (label, value) =>
  `<div class="panel-row"><span class="muted">${text(label)}</span><span>${displayText(value)}</span></div>`;

const redactedError = (value) => {
  const present = value && typeof value === "string" && value.trim().length > 0;
  return present ? "存在（已脱敏）" : "无";
};

const redactedDashboardValue = (value) => {
  if (!value || typeof value !== "object") return "未知";
  if (value.availability === "redacted") return "已脱敏";
  if (value.value !== null && value.value !== undefined) return "存在（已脱敏）";
  return displayValue(value.availability ?? "unknown");
};

const dashboardErrorValue = (value) => {
  if (!value || typeof value !== "object") return "未知";
  if (value.value !== null && value.value !== undefined) return "存在（已脱敏）";
  return displayValue(value.availability ?? "unknown");
};

const emptyTable = (message) => `<div class="tile"><div class="value">${text(message)}</div></div>`;

async function loadSnapshot() {
  const [metaResponse, snapshotResponse] = await Promise.all([
    fetch("/api/server"),
    fetch("/api/snapshot"),
  ]);
  if (!metaResponse.ok) {
    throw new Error(`服务元数据请求失败：${metaResponse.status}`);
  }
  if (!snapshotResponse.ok) {
    throw new Error(`快照请求失败：${snapshotResponse.status}`);
  }
  return {
    metadata: await metaResponse.json(),
    snapshot: await snapshotResponse.json(),
  };
}

function render(payload) {
  const metadata = payload.metadata || {};
  const snapshot = payload.snapshot || {};
  const overview = snapshot.overview || {};
  const nodes = snapshot.nodes || [];
  const staleNodes = nodes.filter((node) => node.health === "stale").length;
  document.getElementById("overview").innerHTML = [
    renderTile("注册表路径", safe(metadata.registry_path)),
    renderTile("节点总数", safe(overview.node_count)),
    renderTile("运行中", safe(overview.running_nodes), "status-healthy"),
    renderTile("已停止", safe(overview.stopped_nodes)),
    renderTile("错误", safe(overview.error_nodes), "status-error"),
    renderTile("已失联", safe(staleNodes), "status-stale"),
    renderTile("未知", safe(overview.unknown_nodes), "status-unknown"),
    renderTile("健康状态", displayValue(overview.health), `status-${safe(overview.health)}`),
    renderTile("仅沙盒", displayValue(overview.sandbox_only)),
    renderTile("生产连接", displayValue(overview.production_venue_connection ?? overview.external_venue_connection)),
    renderTile("Testnet 只读", displayValue(overview.testnet_public_network_connection)),
    renderTile("外部网络尝试", displayValue(overview.external_network_attempted)),
    renderTile("兼容外部交易场所", displayValue(overview.external_venue_connection)),
    renderTile("真实订单", displayValue(overview.real_orders_submitted)),
    renderTile("最近状态变化", displayValue(snapshotValue(overview.latest_transition_at))),
    renderTile("最近错误", redactedError(overview.latest_error), overview.latest_error ? "status-error" : ""),
    renderTile("生成时间", displayValue(snapshotValue(snapshot.generated_at))),
  ].join("");

  document.getElementById("nodes").innerHTML = nodes.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>生命周期</th>
          <th>进程</th>
          <th>PID</th>
          <th>配置</th>
          <th>工件</th>
          <th>启动时间</th>
          <th>停止时间</th>
          <th>最近状态变化</th>
          <th>最近错误</th>
        </tr>
      </thead>
      <tbody>
        ${nodes.map((node) => `
          <tr>
            <td data-label="节点"><strong>${text(node.node_id)}</strong><div class="muted">${displayText(node.process_mode)}</div></td>
            <td data-label="生命周期"><span class="status-${safe(node.health)}">${displayText(node.lifecycle_state)}</span></td>
            <td data-label="进程">${displayText(node.process_state)}</td>
            <td data-label="PID">${displayText(snapshotValue(node.pid))}<div class="muted">${displayText(availability(node.pid))}</div></td>
            <td data-label="配置" class="path">${text(snapshotValue(node.config_path))}</td>
            <td data-label="工件" class="path">${text(snapshotValue(node.artifact_root))}</td>
            <td data-label="启动时间">${displayText(snapshotValue(node.started_at))}</td>
            <td data-label="停止时间">${displayText(snapshotValue(node.stopped_at))}</td>
            <td data-label="最近状态变化">${displayText(snapshotValue(node.last_transition_at))}</td>
            <td data-label="最近错误">${text(redactedError(node.last_error))}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : `<div class="tile"><div class="value">没有已注册节点</div></div>`;

  renderSandboxBusiness(snapshot.sandbox_business || {});
  renderWorkflowArtifacts(snapshot.workflow_artifacts || []);
  renderTraderTerminalWorkbench(snapshot.read_model_runtime || []);
  renderReadModelRuntime(snapshot.read_model_runtime || []);
  renderStrategyRuntime(snapshot.strategy_runtime || []);
  renderProductionShadow(snapshot.production_shadow || []);
  renderPreflightReadiness(snapshot.preflight_readiness || []);
  renderLiveAlphaDryRun(snapshot.live_alpha_dry_run || []);
  renderProductionMutationEvidence(snapshot.production_mutation_evidence || []);
  renderProductionReconciliationOrphan(snapshot.production_reconciliation_orphan || []);
  renderProductionCancelRecovery(snapshot.production_cancel_recovery || []);
  renderProductionActualCancelAudit(snapshot.production_actual_cancel_audit || []);
  renderProductionOrderLifecycleAudit(snapshot.production_order_lifecycle_audit || []);
  renderDataSources(snapshot.data_sources || []);
  renderExecutionGateways(snapshot.execution_gateways || []);
  renderRisk(snapshot.risk || {});
  renderRuntimeModules(snapshot.runtime_modules || []);
  renderLogsMetrics(snapshot.logs || [], snapshot.metrics || []);
  renderControls(snapshot.controls || []);

  document.getElementById("alerts").innerHTML = ((snapshot.alerts || {}).active || []).map((alert) =>
    `<div class="row"><strong>${displayText(alert.severity)}: ${text(alert.source)}</strong><span>${text(alert.message)}</span></div>`
  ).join("") || `<div class="row">没有活动告警</div>`;

  document.getElementById("gaps").innerHTML = (snapshot.gaps || []).map((gap) =>
    `<div class="row"><strong>${text(gap.field_path)}</strong><span>${displayText(gap.reason)} - ${displayText(snapshotValue(gap.notes))}</span></div>`
  ).join("") || `<div class="row">没有待补能力</div>`;
}

const controlLabel = (action) => {
  const name = safe(action).split(":")[0];
  return {
    start: "启动",
    stop: "停止",
    pause: "暂停",
    resume: "恢复",
    reconnect_data: "记录数据源重连不支持",
    reconnect_execution: "记录执行网关重连不支持",
  }[name] || name;
};

const controlNodeId = (action) => safe(action).split(":").slice(1).join(":");
const controlActionName = (action) => safe(action).split(":")[0];

function renderSandboxBusiness(business) {
  const exchange = business.exchange || {};
  const strategies = business.strategies || [];
  const order = business.order || {};
  const risk = business.risk || {};
  const strategySignals = strategies.map((strategy) =>
    `${snapshotValue(strategy.strategy_name)} ${snapshotValue(strategy.signals_emitted)}`
  ).join(" / ") || "unknown";
  const strategyFinalSignals = strategies.map((strategy) =>
    `${snapshotValue(strategy.strategy_name)} ${snapshotValue(strategy.final_signal)}`
  ).join(" / ") || "unknown";
  const orderCounts = [
    `提交 ${dashboardValueText(order.submitted_count)}`,
    `接收 ${dashboardValueText(order.accepted_count)}`,
    `成交 ${dashboardValueText(order.filled_count)}`,
    `撤单 ${dashboardValueText(order.canceled_count)}`,
    `拒绝 ${dashboardValueText(order.rejected_count)}`,
  ].join(" / ");
  document.getElementById("sandbox-business").innerHTML = [
    `<div class="tile">
      <div class="label">交易场所</div>
      <div class="value">${dashboardValueText(exchange.venue)}</div>
      ${panelRow("合约", snapshotValue(exchange.instrument_id))}
      ${panelRow("Bar 类型", snapshotValue(exchange.bar_type))}
      ${panelRow("Fixture", snapshotValue(exchange.fixture_id))}
      ${panelRow("Bar 数量", dashboardValueText(exchange.bars_processed))}
      ${panelRow("外部连接", displayValue(exchange.external_venue_connection))}
    </div>`,
    `<div class="tile">
      <div class="label">策略</div>
      <div class="value">${text(strategies.length)} 条 smoke</div>
      ${panelRow("信号数量", strategySignals)}
      ${panelRow("最终信号", strategyFinalSignals)}
      ${panelRow("运行状态", strategies.map((strategy) => snapshotValue(strategy.runtime_status)).join(" / ") || "unknown")}
      ${panelRow("真实订单", displayValue(strategies.some((strategy) => strategy.real_orders_submitted)))}
    </div>`,
    `<div class="tile">
      <div class="label">订单</div>
      <div class="value">${dashboardValueText(order.lifecycle_id)}</div>
      ${panelRow("事件数", dashboardValueText(order.event_count))}
      ${panelRow("状态覆盖", orderCounts)}
      ${panelRow("策略请求", dashboardValueText(order.mock_orders_requested))}
      ${panelRow("真实订单", displayValue(order.real_orders_submitted))}
    </div>`,
    `<div class="tile status-${safe(risk.health)}">
      <div class="label">风控</div>
      <div class="value">${dashboardValueText(risk.smoke_id)}</div>
      ${panelRow("拒绝原因", snapshotValue(risk.risk_reason))}
      ${panelRow("拒绝订单", snapshotValue(risk.client_order_id))}
      ${panelRow("拒绝数", dashboardValueText(risk.rejection_count))}
      ${panelRow("转发执行", displayValue(risk.forwarded_to_execution))}
      ${panelRow("真实订单", displayValue(risk.real_orders_submitted))}
    </div>`,
  ].join("");
}

function renderControls(controls) {
  document.getElementById("controls").innerHTML = controls.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>操作</th>
          <th>可用性</th>
          <th>是否启用</th>
          <th>原因</th>
          <th>执行</th>
        </tr>
      </thead>
      <tbody>
        ${controls.map((control) => {
          const action = controlActionName(control.action);
          const nodeId = controlNodeId(control.action);
          const runnable = control.enabled && ["start", "stop", "pause", "resume", "reconnect_data", "reconnect_execution"].includes(action);
          return `
            <tr>
              <td data-label="节点"><strong>${text(nodeId)}</strong></td>
              <td data-label="操作">${text(controlLabel(control.action))}</td>
              <td data-label="可用性">${displayText(control.availability)}</td>
              <td data-label="是否启用">${displayText(control.enabled)}</td>
              <td data-label="原因">${displayText(snapshotValue(control.reason))}</td>
              <td data-label="执行"><button type="button" data-dashboard-action="${text(action)}" data-node-id="${text(nodeId)}" ${runnable ? "" : "disabled"}>${text(controlLabel(control.action))}</button></td>
            </tr>`;
        }).join("")}
      </tbody>
    </table>` : emptyTable("没有控制项");
}

function renderWorkflowArtifacts(workflows) {
  document.getElementById("workflow-artifacts").innerHTML = workflows.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>Run ID</th>
          <th>Workflow</th>
          <th>状态</th>
          <th>Manifest</th>
          <th>Artifact</th>
          <th>边界</th>
          <th>订单证明</th>
          <th>探测</th>
          <th>证据</th>
        </tr>
      </thead>
      <tbody>
        ${workflows.map((workflow) => `
          <tr>
            <td data-label="Run ID"><strong>${text(workflow.run_id)}</strong></td>
            <td data-label="Workflow">${displayText(workflow.workflow)}</td>
            <td data-label="状态"><span class="status-${safe(workflow.health)}">${displayText(workflow.runtime_status)}</span></td>
            <td data-label="Manifest" class="path">${text(workflow.manifest_path)}</td>
            <td data-label="Artifact">${displayText(workflow.artifact_count)} 个<div class="muted">${displayText(workflow.schema_version)}</div></td>
            <td data-label="边界">${panelRow("沙盒", workflow.sandbox_only)}${panelRow("Testnet 只读", workflow.testnet_public_network_connection ?? workflow.testnet_connection)}${panelRow("网络许可", workflow.network_permission_requested)}${panelRow("网络尝试", workflow.external_network_attempted ?? workflow.network_attempted)}${panelRow("生产连接", workflow.production_venue_connection ?? workflow.external_venue_connection)}${panelRow("账户变更", snapshotValue(workflow.authenticated_account_mutation))}${panelRow("真实资金", workflow.real_funds)}${panelRow("生产交易", workflow.production_trading)}${panelRow("真实订单", workflow.real_orders_submitted)}</td>
            <td data-label="订单证明">${panelRow("风险预检", snapshotValue(workflow.order_proof_risk_preflight_status))}${panelRow("Order test", snapshotValue(workflow.order_proof_order_test_status))}${panelRow("Submit ack", snapshotValue(workflow.order_proof_submit_ack_status))}${panelRow("Cancel ack", snapshotValue(workflow.order_proof_cancel_ack_status))}${panelRow("Terminal", snapshotValue(workflow.order_proof_terminal_status))}${panelRow("Reconciliation", snapshotValue(workflow.order_proof_reconciliation_status))}${panelRow("人工生命周期", snapshotValue(workflow.order_proof_manual_submit_cancel_observed))}${panelRow("Testnet 提交", snapshotValue(workflow.order_proof_testnet_orders_submitted))}${panelRow("Testnet 撤单", snapshotValue(workflow.order_proof_testnet_orders_canceled))}${panelRow("生产提交", snapshotValue(workflow.order_proof_production_orders_submitted))}${panelRow("生产撤单", snapshotValue(workflow.order_proof_production_orders_canceled))}${panelRow("Dashboard 下单控件", snapshotValue(workflow.order_proof_dashboard_order_controls))}</td>
            <td data-label="探测">${text(snapshotValue(workflow.probe_status))}<div class="muted">${text(snapshotValue(workflow.probe_endpoint_class))}</div><div class="muted">latency=${text(snapshotValue(workflow.probe_latency_ms))} error=${text(snapshotValue(workflow.probe_error_code))}</div><div class="muted">auth=${text(snapshotValue(workflow.authenticated_probe_status))} ${text(snapshotValue(workflow.authenticated_request_method))} ${text(snapshotValue(workflow.authenticated_endpoint_kind))}</div><div class="muted">shape=${text(snapshotValue(workflow.authenticated_response_shape))} validated=${text(snapshotValue(workflow.authenticated_response_shape_validated))}</div><div class="muted">ws=${text(snapshotValue(workflow.websocket_probe_status))} / ${text(snapshotValue(workflow.websocket_error_code))}</div><div class="muted">${panelRow("WS尝试", workflow.websocket_attempted)}${panelRow("WS订阅", workflow.websocket_subscription_attempted)}${panelRow("Key存在", snapshotValue(workflow.authenticated_api_key_present))}${panelRow("Secret存在", snapshotValue(workflow.authenticated_api_secret_present))}${panelRow("密钥记录", snapshotValue(workflow.values_recorded))}${panelRow("密钥脱敏", snapshotValue(workflow.authenticated_secrets_redacted))}</div></td>
            <td data-label="证据">${text(snapshotValue(workflow.market_fixture_id))}<div class="muted">${text(snapshotValue(workflow.risk_smoke_id))}</div><div class="muted">${text(snapshotValue(workflow.credential_policy))}</div><div class="muted">${text(snapshotValue(workflow.connectivity_mode))} / ${text(snapshotValue(workflow.order_submission_mode))}</div></td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 workflow manifest 工件");
}

function traderWorkbenchReadiness(readModels) {
  if (readModels.length === 0) {
    return {
      health: "degraded",
      readiness: "missing_artifact",
      diagnostic: "canonical_unified_read_model_artifact_missing",
      artifact: "v0_21/unified_read_model_snapshot.json",
    };
  }

  const orderedHealth = ["error", "degraded", "stale", "unknown", "healthy"];
  const strongest = readModels
    .map((item) => safe(item.health))
    .sort((a, b) => orderedHealth.indexOf(a) - orderedHealth.indexOf(b))[0] || "unknown";
  const primary = readModels.find((item) => safe(item.health) !== "healthy") || readModels[0];
  const shellHealth = strongest === "healthy" ? "healthy" : ["error", "stale"].includes(strongest) ? strongest : "degraded";
  return {
    health: shellHealth,
    readiness: snapshotValue(primary.readiness_status) || "read_model_runtime_degraded",
    diagnostic: snapshotValue(primary.diagnostic) || "read_model_runtime_degraded",
    artifact: snapshotValue(primary.artifact_path) || "v0_21/unified_read_model_snapshot.json",
  };
}

function renderTraderTerminalWorkbench(readModels) {
  const readiness = traderWorkbenchReadiness(readModels);
  const primary = readModels.find((item) => safe(item.health) !== "healthy") || readModels[0] || {};
  const panelStatus = (field) => snapshotValue(primary[field]) || "missing_artifact";
  const boundaryValue = (field) => readModels.length > 0 ? snapshotValue(primary[field]) : false;
  const panels = [
    ["workbench-tab-account", "workbench-panel-account", "账户", panelStatus("account_status"), "V220-002"],
    ["workbench-tab-positions", "workbench-panel-positions", "持仓", panelStatus("positions_status"), "V220-002"],
    ["workbench-tab-orders", "workbench-panel-orders", "订单", panelStatus("orders_status"), "V220-003"],
    ["workbench-tab-fills", "workbench-panel-fills", "成交", panelStatus("fills_status"), "V220-003"],
    ["workbench-tab-risk", "workbench-panel-risk", "风险", panelStatus("risk_status"), "V220-004"],
    ["workbench-tab-alerts", "workbench-panel-alerts", "告警", snapshotValue(primary.risk_alert_severity) || panelStatus("risk_status"), "V220-004"],
    ["workbench-tab-audit-provenance", "workbench-panel-audit-provenance", "审计 / Provenance", panelStatus("lifecycle_status"), "V220-004"],
    ["workbench-tab-operation-entry", "workbench-panel-operation-entry", "操作入口", snapshotValue(primary.operation_entry_status) || panelStatus("operation_entry_status"), "V220-005"],
    ["workbench-tab-v24-order-control-preview", "workbench-panel-v24-order-control-preview", "v24 Order-control preview", panelStatus("v24_order_control_preview_status"), "V240-008"],
    ["workbench-tab-v25-monitoring-surface", "workbench-panel-v25-monitoring-surface", "v25 Monitoring / Incident / DR", panelStatus("v25_dashboard_surface_status"), "V250-006"],
    ["workbench-tab-v26-admin-surface", "workbench-panel-v26-admin-surface", "v26 Product hardening admin", panelStatus("v26_dashboard_admin_surface_status"), "V260-007"],
  ];
  const controlsDisabled = [
    "new_submit_capability",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "dashboard_submit_controls_enabled",
    "dashboard_replace_controls_enabled",
    "dashboard_amend_controls_enabled",
    "dashboard_flatten_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_entry_enabled",
    "manual_operation_submit_allowed",
    "manual_operation_cancel_allowed",
    "manual_operation_retry_allowed",
    "manual_operation_replace_allowed",
    "manual_operation_amend_allowed",
    "manual_operation_flatten_allowed",
    "automatic_operation_action_allowed",
  ].every((field) => boundaryValue(field) === false);
  const accountRows = [
    panelRow("Account status", snapshotValue(primary.account_status)),
    panelRow("Freshness", snapshotValue(primary.account_freshness_status)),
    panelRow("Risk state", snapshotValue(primary.account_risk_state)),
    panelRow("Equity", snapshotValue(primary.account_equity)),
    panelRow("Available balance", snapshotValue(primary.account_available_balance)),
    panelRow("Balance entries", snapshotValue(primary.account_balance_entry_count)),
    panelRow("Source", `${snapshotValue(primary.account_source_type)} ${snapshotValue(primary.account_source_ref)}`),
    panelRow("Redaction", snapshotValue(primary.account_redaction_state)),
    panelRow("Funds transfer", boundaryValue("funds_transfer_allowed")),
    panelRow("Account config mutation", boundaryValue("account_configuration_mutation_allowed")),
  ];
  const positionRows = [
    panelRow("Position status", snapshotValue(primary.positions_status)),
    panelRow("Freshness", snapshotValue(primary.positions_freshness_status)),
    panelRow("Account", snapshotValue(primary.positions_account_id)),
    panelRow("Side", snapshotValue(primary.positions_net_position_side)),
    panelRow("Quantity", snapshotValue(primary.positions_quantity)),
    panelRow("Notional", snapshotValue(primary.positions_notional)),
    panelRow("Precision", snapshotValue(primary.positions_precision)),
    panelRow("Lineage", snapshotValue(primary.positions_lineage)),
    panelRow("Source", `${snapshotValue(primary.positions_source_type)} ${snapshotValue(primary.positions_source_ref)}`),
    panelRow("Redaction", snapshotValue(primary.positions_redaction_state)),
    panelRow("Auto flatten", boundaryValue("auto_flatten_position_allowed")),
    panelRow("Position repair", boundaryValue("automatic_position_repair_allowed")),
  ];
  const orderRows = [
    panelRow("Order status", snapshotValue(primary.orders_status)),
    panelRow("Freshness", snapshotValue(primary.orders_freshness_status)),
    panelRow("Lifecycle", snapshotValue(primary.orders_lifecycle_state)),
    panelRow("Client order id", snapshotValue(primary.orders_client_order_id)),
    panelRow("Request digest", snapshotValue(primary.orders_request_digest)),
    panelRow("Attempt id", snapshotValue(primary.orders_attempt_id)),
    panelRow("Approval id", snapshotValue(primary.orders_approval_id)),
    panelRow("Readback", snapshotValue(primary.orders_readback_status)),
    panelRow("Audit", snapshotValue(primary.orders_audit_state)),
    panelRow("Ledger present", snapshotValue(primary.orders_ledger_present)),
    panelRow("Duplicate attempt", snapshotValue(primary.orders_duplicate_attempt_detected)),
    panelRow("No retry", snapshotValue(primary.orders_no_retry)),
    panelRow("Diagnostics", snapshotValue(primary.orders_diagnostics)),
    panelRow("Lineage", snapshotValue(primary.orders_lineage)),
    panelRow("Source", `${snapshotValue(primary.orders_source_type)} ${snapshotValue(primary.orders_source_ref)}`),
    panelRow("Exchange truth", snapshotValue(primary.orders_exchange_truth)),
    panelRow("Adapter runtime", snapshotValue(primary.orders_adapter_runtime_integrated)),
    panelRow("Schema-only truth", snapshotValue(primary.orders_values_are_exchange_truth)),
    panelRow("Redaction", snapshotValue(primary.orders_redaction_state)),
    panelRow("Submit allowed", boundaryValue("production_order_submission_allowed")),
    panelRow("Mutation allowed", boundaryValue("production_order_mutation_allowed")),
    panelRow("Order controls", boundaryValue("dashboard_order_controls_enabled")),
    panelRow("Permission controls", boundaryValue("order_permission_control_allowed")),
    panelRow("Retry allowed", boundaryValue("retry_order_allowed")),
    panelRow("Auto cancel", boundaryValue("automatic_cancel_allowed")),
    panelRow("Auto remediation", boundaryValue("automatic_order_remediation_allowed")),
  ];
  const fillRows = [
    panelRow("Fill status", snapshotValue(primary.fills_fill_status)),
    panelRow("Freshness", snapshotValue(primary.fills_freshness_status)),
    panelRow("Fill id", snapshotValue(primary.fills_fill_id)),
    panelRow("Execution id", snapshotValue(primary.fills_execution_id)),
    panelRow("Order id", snapshotValue(primary.fills_order_id)),
    panelRow("Client order id", snapshotValue(primary.fills_client_order_id)),
    panelRow("Order linkage", snapshotValue(primary.fills_order_linkage_status)),
    panelRow("Reconciliation", snapshotValue(primary.fills_reconciliation_status)),
    panelRow("Quantity", snapshotValue(primary.fills_quantity)),
    panelRow("Cumulative", snapshotValue(primary.fills_cumulative_quantity)),
    panelRow("Remaining", snapshotValue(primary.fills_remaining_quantity)),
    panelRow("Price", snapshotValue(primary.fills_price)),
    panelRow("Precision", snapshotValue(primary.fills_precision_status)),
    panelRow("Partial fill", snapshotValue(primary.fills_partial_fill_detected)),
    panelRow("Duplicate fill", snapshotValue(primary.fills_duplicate_fill_detected)),
    panelRow("Risk input", snapshotValue(primary.fills_risk_projection_input)),
    panelRow("Diagnostics", snapshotValue(primary.fills_diagnostics)),
    panelRow("Lineage", snapshotValue(primary.fills_lineage)),
    panelRow("Source", `${snapshotValue(primary.fills_source_type)} ${snapshotValue(primary.fills_source_ref)}`),
    panelRow("Exchange truth", snapshotValue(primary.fills_exchange_truth)),
    panelRow("Adapter runtime", snapshotValue(primary.fills_adapter_runtime_integrated)),
    panelRow("Schema-only truth", snapshotValue(primary.fills_values_are_exchange_truth)),
    panelRow("Redaction", snapshotValue(primary.fills_redaction_state)),
    panelRow("Fill controls", boundaryValue("dashboard_fill_controls_enabled")),
    panelRow("Execution algorithm", boundaryValue("execution_algorithm_allowed")),
    panelRow("Fill repair", boundaryValue("automatic_fill_repair_allowed")),
    panelRow("Reconciliation repair", boundaryValue("automatic_reconciliation_repair_allowed")),
  ];
  const riskRows = [
    panelRow("Risk status", snapshotValue(primary.risk_status)),
    panelRow("Priority state", snapshotValue(primary.risk_priority_state)),
    panelRow("Risk state", snapshotValue(primary.risk_state)),
    panelRow("Freshness", snapshotValue(primary.risk_freshness_status)),
    panelRow("Freshness rollup", snapshotValue(primary.risk_freshness_rollup)),
    panelRow("Critical evidence", snapshotValue(primary.risk_critical_evidence_complete)),
    panelRow("Risk visible", snapshotValue(primary.risk_visible)),
    panelRow("Manual review", snapshotValue(primary.risk_manual_review_required)),
    panelRow("Halted", snapshotValue(primary.risk_halted)),
    panelRow("Mismatch", snapshotValue(primary.risk_mismatch_detected)),
    panelRow("Diagnostics", snapshotValue(primary.risk_diagnostics)),
    panelRow("Lineage", snapshotValue(primary.risk_lineage)),
    panelRow("Source", `${snapshotValue(primary.risk_source_type)} ${snapshotValue(primary.risk_source_ref)}`),
    panelRow("Redaction", snapshotValue(primary.risk_redaction_state)),
    panelRow("Risk controls", boundaryValue("dashboard_risk_controls_enabled")),
    panelRow("Auto risk action", boundaryValue("automatic_risk_action_allowed")),
    panelRow("Risk repair", boundaryValue("automatic_risk_repair_allowed")),
  ];
  const alertRows = [
    panelRow("Severity", snapshotValue(primary.risk_alert_severity)),
    panelRow("Missing evidence", snapshotValue(primary.risk_alert_missing_evidence)),
    panelRow("Stale source", snapshotValue(primary.risk_alert_stale_source)),
    panelRow("Schema mismatch", snapshotValue(primary.risk_alert_schema_mismatch)),
    panelRow("Redaction breach", snapshotValue(primary.risk_alert_redaction_breach)),
    panelRow("Forbidden control", snapshotValue(primary.risk_alert_forbidden_control_request)),
    panelRow("Summary", snapshotValue(primary.risk_alert_summary)),
    panelRow("Alert action", boundaryValue("automatic_alert_action_allowed")),
  ];
  const auditProvenanceRows = [
    panelRow("Lifecycle", snapshotValue(primary.lifecycle_summary)),
    panelRow("Audit freshness", snapshotValue(primary.audit_freshness_status)),
    panelRow("Audit state", snapshotValue(primary.audit_state)),
    panelRow("Audit closed", snapshotValue(primary.audit_closed)),
    panelRow("Evidence complete", snapshotValue(primary.audit_required_evidence_complete)),
    panelRow("Components complete", snapshotValue(primary.audit_required_components_complete)),
    panelRow("Missing evidence", snapshotValue(primary.audit_missing_evidence)),
    panelRow("Provenance mismatch", snapshotValue(primary.audit_provenance_mismatch)),
    panelRow("Release provenance", snapshotValue(primary.audit_release_provenance)),
    panelRow("Artifact digest", snapshotValue(primary.audit_artifact_digest)),
    panelRow("Artifact sha", snapshotValue(primary.audit_artifact_sha)),
    panelRow("Diagnostics", snapshotValue(primary.audit_diagnostics)),
    panelRow("Lineage", snapshotValue(primary.audit_lineage)),
    panelRow("Source", `${snapshotValue(primary.audit_source_type)} ${snapshotValue(primary.audit_source_ref)}`),
    panelRow("Redaction", snapshotValue(primary.audit_redaction_state)),
    panelRow("Audit action", boundaryValue("automatic_audit_action_allowed")),
    panelRow("Provenance repair", boundaryValue("automatic_provenance_repair_allowed")),
  ];
  const operationEntryRows = [
    panelRow("Entry status", snapshotValue(primary.operation_entry_status)),
    panelRow("Freshness", snapshotValue(primary.operation_entry_freshness_status)),
    panelRow("Intent preview", snapshotValue(primary.operation_intent_preview)),
    panelRow("Owner approval ref", snapshotValue(primary.operation_owner_approval_ref)),
    panelRow("Risk decision ref", snapshotValue(primary.operation_risk_decision_ref)),
    panelRow("Audit evidence ref", snapshotValue(primary.operation_audit_evidence_ref)),
    panelRow("Disabled", snapshotValue(primary.operation_entry_disabled)),
    panelRow("Blocked reason", snapshotValue(primary.operation_entry_blocked_reason)),
    panelRow("Missing approval", snapshotValue(primary.operation_missing_owner_approval)),
    panelRow("Missing risk gate", snapshotValue(primary.operation_missing_risk_gate)),
    panelRow("Missing audit gate", snapshotValue(primary.operation_missing_audit_gate)),
    panelRow("Stale read model", snapshotValue(primary.operation_stale_read_model)),
    panelRow("Provenance mismatch", snapshotValue(primary.operation_provenance_mismatch)),
    panelRow("Gates complete", snapshotValue(primary.operation_gates_complete)),
    panelRow("Ungated attempt", snapshotValue(primary.operation_ungated_attempted)),
    panelRow("Attempt status", snapshotValue(primary.operation_attempt_status)),
    panelRow("Fail-closed attempt", snapshotValue(primary.operation_ungated_attempt_fail_closed)),
    panelRow("Source", `${snapshotValue(primary.operation_entry_source_type)} ${snapshotValue(primary.operation_entry_source_ref)}`),
    panelRow("Redaction", snapshotValue(primary.operation_entry_redaction_state)),
    panelRow("Entry enabled", boundaryValue("manual_operation_entry_enabled")),
    panelRow("Submit", boundaryValue("manual_operation_submit_allowed")),
    panelRow("Cancel", boundaryValue("manual_operation_cancel_allowed")),
    panelRow("Retry", boundaryValue("manual_operation_retry_allowed")),
    panelRow("Replace", boundaryValue("manual_operation_replace_allowed")),
    panelRow("Amend", boundaryValue("manual_operation_amend_allowed")),
    panelRow("Flatten", boundaryValue("manual_operation_flatten_allowed")),
    panelRow("Automatic action", boundaryValue("automatic_operation_action_allowed")),
  ];
  const v24OrderControlPreviewRows = [
    panelRow("Preview status", snapshotValue(primary.v24_order_control_preview_status)),
    panelRow("Order intent", snapshotValue(primary.v24_order_intent_status)),
    panelRow("Execution policy", snapshotValue(primary.v24_execution_policy_status)),
    panelRow("Rate limit", snapshotValue(primary.v24_rate_limit_status)),
    panelRow("Slicing", snapshotValue(primary.v24_slicing_status)),
    panelRow("Cancel / replace / amend", snapshotValue(primary.v24_cancel_replace_amend_status)),
    panelRow("Retry policy", snapshotValue(primary.v24_retry_policy_status)),
    panelRow("Readback / audit", snapshotValue(primary.v24_readback_audit_status)),
    panelRow("Blocked reasons", snapshotValue(primary.v24_blocked_reasons)),
    panelRow("Scope key", snapshotValue(primary.v24_scope_key)),
    panelRow("Source provenance", snapshotValue(primary.v24_source_provenance)),
    panelRow("Redaction", snapshotValue(primary.v24_redaction_state)),
    panelRow("Order intent ref", snapshotValue(primary.v24_order_intent_ref)),
    panelRow("Policy ref", snapshotValue(primary.v24_policy_ref)),
    panelRow("Rate-limit ref", snapshotValue(primary.v24_rate_limit_ref)),
    panelRow("Slicing ref", snapshotValue(primary.v24_slicing_ref)),
    panelRow("Cancel/replace/amend ref", snapshotValue(primary.v24_cancel_replace_amend_ref)),
    panelRow("Retry policy ref", snapshotValue(primary.v24_retry_policy_ref)),
    panelRow("Readback ref", snapshotValue(primary.v24_readback_ref)),
    panelRow("Audit ref", snapshotValue(primary.v24_audit_ref)),
    panelRow("Provenance ref", snapshotValue(primary.v24_provenance_ref)),
    panelRow("Dashboard redacted ref", snapshotValue(primary.v24_dashboard_redacted_ref)),
    panelRow("Preview evidence present", snapshotValue(primary.v24_preview_evidence_present)),
    panelRow("Missing preview evidence", snapshotValue(primary.v24_missing_preview_evidence)),
    panelRow("Forbidden control detected", snapshotValue(primary.v24_forbidden_control_detected)),
    panelRow("Render smoke case", snapshotValue(primary.v24_render_smoke_case)),
    panelRow("Submit control", boundaryValue("dashboard_submit_controls_enabled")),
    panelRow("Cancel control", boundaryValue("dashboard_cancel_controls_enabled")),
    panelRow("Replace control", boundaryValue("dashboard_replace_controls_enabled")),
    panelRow("Amend control", boundaryValue("dashboard_amend_controls_enabled")),
    panelRow("Flatten control", boundaryValue("dashboard_flatten_controls_enabled")),
    panelRow("Order ticket", boundaryValue("trader_terminal_order_ticket_enabled")),
    panelRow("Manual entry", boundaryValue("manual_operation_entry_enabled")),
    panelRow("Automatic action", boundaryValue("automatic_operation_action_allowed")),
  ];
  const v25MonitoringSurfaceRows = [
    panelRow("Surface status", snapshotValue(primary.v25_dashboard_surface_status)),
    panelRow("Diagnostics gate", snapshotValue(primary.v25_diagnostics_gate_status)),
    panelRow("SLO status", snapshotValue(primary.v25_slo_status)),
    panelRow("Freshness threshold", snapshotValue(primary.v25_freshness_threshold_status)),
    panelRow("Staleness reasons", snapshotValue(primary.v25_staleness_reasons)),
    panelRow("Diagnostic severity", snapshotValue(primary.v25_diagnostic_severity)),
    panelRow("Source truth", snapshotValue(primary.v25_source_truth_status)),
    panelRow("Release provenance", snapshotValue(primary.v25_release_provenance_status)),
    panelRow("No-action boundary", snapshotValue(primary.v25_no_remediation_status)),
    panelRow("Monitoring status", snapshotValue(primary.v25_monitoring_status)),
    panelRow("Runtime health", snapshotValue(primary.v25_monitoring_runtime_health)),
    panelRow("Effective status", snapshotValue(primary.v25_monitoring_effective_status)),
    panelRow("Monitoring freshness", snapshotValue(primary.v25_monitoring_freshness_status)),
    panelRow("Monitoring source", snapshotValue(primary.v25_monitoring_source_ref)),
    panelRow("Monitoring redaction", snapshotValue(primary.v25_monitoring_redaction_state)),
    panelRow("Alert status", snapshotValue(primary.v25_alert_status)),
    panelRow("Alert severity", snapshotValue(primary.v25_alert_highest_severity)),
    panelRow("Alert route", snapshotValue(primary.v25_alert_route_status)),
    panelRow("Alert dedupe", snapshotValue(primary.v25_alert_dedupe_key)),
    panelRow("Incident status", snapshotValue(primary.v25_incident_status)),
    panelRow("Incident state", snapshotValue(primary.v25_incident_current_state)),
    panelRow("Ack status", snapshotValue(primary.v25_incident_ack_status)),
    panelRow("Incident owner", snapshotValue(primary.v25_incident_owner)),
    panelRow("Runbook status", snapshotValue(primary.v25_runbook_status)),
    panelRow("Runbook decision", snapshotValue(primary.v25_runbook_decision_type)),
    panelRow("Runbook evidence", snapshotValue(primary.v25_runbook_evidence_ref)),
    panelRow("DR preview status", snapshotValue(primary.v25_dr_preview_status)),
    panelRow("DR scenario", snapshotValue(primary.v25_dr_scenario)),
    panelRow("Recovery point", snapshotValue(primary.v25_dr_recovery_point)),
    panelRow("Operator approval", snapshotValue(primary.v25_dr_operator_approval_status)),
    panelRow("Snapshot lineage", snapshotValue(primary.v25_dr_snapshot_lineage)),
    panelRow("Blocking reasons", snapshotValue(primary.v25_surface_blocking_reasons)),
    panelRow("Submit control", boundaryValue("dashboard_submit_controls_enabled")),
    panelRow("Cancel control", boundaryValue("dashboard_cancel_controls_enabled")),
    panelRow("Retry control", boundaryValue("dashboard_retry_controls_enabled")),
    panelRow("Replace control", boundaryValue("dashboard_replace_controls_enabled")),
    panelRow("Amend control", boundaryValue("dashboard_amend_controls_enabled")),
    panelRow("Flatten control", boundaryValue("dashboard_flatten_controls_enabled")),
    panelRow("Order ticket", boundaryValue("trader_terminal_order_ticket_enabled")),
  ];
  const v26AdminSurfaceRows = [
    panelRow("Admin surface status", snapshotValue(primary.v26_dashboard_admin_surface_status)),
    panelRow("Permission boundary", snapshotValue(primary.v26_permission_boundary_status)),
    panelRow("Roles checked", snapshotValue(primary.v26_permission_roles_checked)),
    panelRow("Operation audit", snapshotValue(primary.v26_operation_audit_status)),
    panelRow("Audit lineage", snapshotValue(primary.v26_operation_audit_lineage)),
    panelRow("Deployment provenance", snapshotValue(primary.v26_deployment_provenance_status)),
    panelRow("Environment", snapshotValue(primary.v26_deployment_environment)),
    panelRow("Upgrade rollback", snapshotValue(primary.v26_upgrade_rollback_status)),
    panelRow("Runbook preview", snapshotValue(primary.v26_upgrade_rollback_preview)),
    panelRow("Stability / SLO", snapshotValue(primary.v26_stability_status)),
    panelRow("Degradation reason", snapshotValue(primary.v26_stability_degradation_reason)),
    panelRow("Blocking reasons", snapshotValue(primary.v26_admin_surface_blocking_reasons)),
    panelRow("Submit control", boundaryValue("dashboard_submit_controls_enabled")),
    panelRow("Cancel control", boundaryValue("dashboard_cancel_controls_enabled")),
    panelRow("Retry control", boundaryValue("dashboard_retry_controls_enabled")),
    panelRow("Replace control", boundaryValue("dashboard_replace_controls_enabled")),
    panelRow("Amend control", boundaryValue("dashboard_amend_controls_enabled")),
    panelRow("Flatten control", boundaryValue("dashboard_flatten_controls_enabled")),
    panelRow("Order ticket", boundaryValue("trader_terminal_order_ticket_enabled")),
    panelRow("Manual submit", boundaryValue("manual_operation_submit_allowed")),
    panelRow("Automatic action", boundaryValue("automatic_operation_action_allowed")),
  ];
  const panelRows = {
    "workbench-panel-account": accountRows,
    "workbench-panel-positions": positionRows,
    "workbench-panel-orders": orderRows,
    "workbench-panel-fills": fillRows,
    "workbench-panel-risk": riskRows,
    "workbench-panel-alerts": alertRows,
    "workbench-panel-audit-provenance": auditProvenanceRows,
    "workbench-panel-operation-entry": operationEntryRows,
    "workbench-panel-v24-order-control-preview": v24OrderControlPreviewRows,
    "workbench-panel-v25-monitoring-surface": v25MonitoringSurfaceRows,
    "workbench-panel-v26-admin-surface": v26AdminSurfaceRows,
  };

  document.getElementById("trader-terminal-workbench").innerHTML = `
    <div class="grid">
      ${renderTile("Workbench 状态", readiness.readiness, `status-${safe(readiness.health)}`)}
      ${renderTile("Read Model", readModels.length > 0 ? `${readModels.length} 个节点` : "缺失")}
      ${renderTile("只读边界", controlsDisabled ? "locked_readonly" : "degraded_boundary", controlsDisabled ? "status-healthy" : "status-degraded")}
      ${renderTile("诊断", readiness.diagnostic, `status-${safe(readiness.health)}`)}
    </div>
    <nav class="workbench-tabs" aria-label="Trader Terminal workbench read-only navigation">
      ${panels.map(([tabId, panelId, label], index) =>
        `<span class="workbench-tab" id="${text(tabId)}" role="tab" aria-selected="${index === 0 ? "true" : "false"}" aria-controls="${text(panelId)}">${text(label)}</span>`
      ).join("")}
    </nav>
    <div class="workbench-boundary">
      <div class="workbench-panel" id="foundation-boundary">
        <h3>Foundation</h3>
        ${panelRow("来源", "v0.21.1 canonical read model")}
        ${panelRow("Artifact", readiness.artifact)}
        ${panelRow("Contract", snapshotValue(primary.contract_version) || "missing_artifact")}
      </div>
      <div class="workbench-panel" id="read-only-boundary">
        <h3>Read-only</h3>
        ${panelRow("节点视图", readModels.length > 0 ? "read_model_runtime" : "degraded_shell")}
        ${panelRow("状态", controlsDisabled ? "all_operation_controls_disabled" : "operation_boundary_degraded")}
        ${panelRow("订单票据", boundaryValue("trader_terminal_order_ticket_enabled"))}
      </div>
      <div class="workbench-panel" id="gated-operation-boundary">
        <h3>Gated operation</h3>
        ${panelRow("Owner approval", "required_before_any_manual_entry")}
        ${panelRow("Risk gate", "required_before_any_manual_entry")}
        ${panelRow("Audit gate", "required_before_any_manual_entry")}
        ${panelRow("Entry status", snapshotValue(primary.operation_entry_status) || "blocked_until_contract_present")}
        ${panelRow("Blocked reason", snapshotValue(primary.operation_entry_blocked_reason) || "missing_operation_entry_contract")}
        ${panelRow("当前阶段", "disabled_gated_preview_only")}
        ${panelRow("产品级终端声明", boundaryValue("product_grade_trading_terminal_claim"))}
      </div>
    </div>
    <div class="workbench-panels">
      ${panels.map(([tabId, panelId, label, status, stage]) => `
        <section class="workbench-panel" id="${text(panelId)}" role="tabpanel" aria-labelledby="${text(tabId)}">
          <h3>${text(label)}</h3>
          ${panelRow("阶段", stage)}
          ${panelRow("状态", status)}
          ${panelRow("来源", readModels.length > 0 ? "canonical_read_model" : "degraded_shell")}
          ${(panelRows[panelId] || []).join("")}
        </section>`
      ).join("")}
    </div>
    <details class="workbench-drawer" id="workbench-artifact-provenance-drawer" open>
      <summary>Artifact / Provenance</summary>
      ${panelRow("Artifact path", readiness.artifact)}
      ${panelRow("Snapshot", snapshotValue(primary.snapshot_id) || "missing_artifact")}
      ${panelRow("Source", `${snapshotValue(primary.source_type) || "unknown"} ${snapshotValue(primary.source_ref) || ""}`)}
      ${panelRow("Redaction", snapshotValue(primary.redaction_state) || "unknown")}
      ${panelRow("Blocking reasons", snapshotValue(primary.blocking_reasons) || "none")}
      ${panelRow("Release provenance", snapshotValue(primary.audit_release_provenance) || "unknown")}
      ${panelRow("Artifact digest", snapshotValue(primary.audit_artifact_digest) || "unknown")}
    </details>`;
}

function renderReadModelRuntime(readModels) {
  document.getElementById("read-model-runtime").innerHTML = readModels.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>当前结论</th>
          <th>Snapshot</th>
          <th>组件状态</th>
          <th>基础状态</th>
          <th>只读边界</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${readModels.map((item) => `
          <tr>
            <td data-label="节点"><strong>${text(item.node_id)}</strong></td>
            <td data-label="当前结论"><span class="status-${safe(item.health)}">${displayText(item.health)}</span><div class="muted">${displayText(snapshotValue(item.readiness_status))}</div><div class="muted">${displayText(snapshotValue(item.diagnostic))}</div>${panelRow("阻塞原因", snapshotValue(item.blocking_reasons))}${panelRow("缺失组件", snapshotValue(item.missing_components))}${panelRow("组件诊断", snapshotValue(item.component_diagnostics))}</td>
            <td data-label="Snapshot">${panelRow("Contract", snapshotValue(item.contract_version))}${panelRow("Schema", snapshotValue(item.schema_version))}${panelRow("ID", snapshotValue(item.snapshot_id))}${panelRow("Kind", snapshotValue(item.snapshot_kind))}${panelRow("Health", snapshotValue(item.snapshot_health_status))}${panelRow("Freshness", snapshotValue(item.freshness_status))}${panelRow("Source", `${snapshotValue(item.source_type)} ${snapshotValue(item.source_ref)}`)}${panelRow("Redaction", snapshotValue(item.redaction_state))}</td>
            <td data-label="组件状态">${panelRow("Account", snapshotValue(item.account_status))}${panelRow("Positions", snapshotValue(item.positions_status))}${panelRow("Orders", snapshotValue(item.orders_status))}${panelRow("Fills", snapshotValue(item.fills_status))}${panelRow("Risk", snapshotValue(item.risk_status))}${panelRow("Lifecycle", snapshotValue(item.lifecycle_status))}${panelRow("v25 Surface", snapshotValue(item.v25_dashboard_surface_status))}${panelRow("v25 Diagnostics", snapshotValue(item.v25_diagnostics_gate_status))}${panelRow("Monitoring", snapshotValue(item.v25_monitoring_status))}${panelRow("Incident", snapshotValue(item.v25_incident_status))}${panelRow("Runbook", snapshotValue(item.v25_runbook_status))}${panelRow("DR Preview", snapshotValue(item.v25_dr_preview_status))}${panelRow("v26 Admin", snapshotValue(item.v26_dashboard_admin_surface_status))}${panelRow("Permission", snapshotValue(item.v26_permission_boundary_status))}${panelRow("Audit", snapshotValue(item.v26_operation_audit_status))}${panelRow("Deployment", snapshotValue(item.v26_deployment_provenance_status))}${panelRow("Stability", snapshotValue(item.v26_stability_status))}</td>
            <td data-label="基础状态">${panelRow("Account", snapshotValue(item.account_summary))}${panelRow("Positions", snapshotValue(item.positions_summary))}${panelRow("Orders", snapshotValue(item.orders_summary))}${panelRow("Fills", snapshotValue(item.fills_summary))}${panelRow("Risk", snapshotValue(item.risk_summary))}${panelRow("Lifecycle", snapshotValue(item.lifecycle_summary))}</td>
            <td data-label="只读边界">${panelRow("新增 Submit 能力", snapshotValue(item.new_submit_capability))}${panelRow("下单控件", snapshotValue(item.dashboard_order_controls_enabled))}${panelRow("审批控件", snapshotValue(item.dashboard_approval_controls_enabled))}${panelRow("撤单控件", snapshotValue(item.dashboard_cancel_controls_enabled))}${panelRow("重试控件", snapshotValue(item.dashboard_retry_controls_enabled))}${panelRow("Submit", snapshotValue(item.dashboard_submit_controls_enabled))}${panelRow("Replace", snapshotValue(item.dashboard_replace_controls_enabled))}${panelRow("Amend", snapshotValue(item.dashboard_amend_controls_enabled))}${panelRow("Flatten", snapshotValue(item.dashboard_flatten_controls_enabled))}${panelRow("订单票据", snapshotValue(item.trader_terminal_order_ticket_enabled))}${panelRow("实盘终端声明", snapshotValue(item.trader_terminal_live_trading_claim))}${panelRow("产品级声明", snapshotValue(item.product_grade_trading_terminal_claim))}${panelRow("v24 preview", snapshotValue(item.v24_order_control_preview_status))}${panelRow("v24 scope", snapshotValue(item.v24_scope_key))}${panelRow("v24 source", snapshotValue(item.v24_source_provenance))}${panelRow("v24 redaction", snapshotValue(item.v24_redaction_state))}${panelRow("v24 missing evidence", snapshotValue(item.v24_missing_preview_evidence))}${panelRow("v25 blockers", snapshotValue(item.v25_surface_blocking_reasons))}${panelRow("v26 blockers", snapshotValue(item.v26_admin_surface_blocking_reasons))}</td>
            <td data-label="工件" class="path">${displayText(snapshotValue(item.artifact_path))}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 Unified Read Model runtime 工件");
}

function renderStrategyRuntime(strategyRuntime) {
  document.getElementById("strategy-runtime").innerHTML = strategyRuntime.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>健康</th>
          <th>会话</th>
          <th>市场流</th>
          <th>信号</th>
          <th>Order Intent</th>
          <th>风控决策</th>
          <th>拒绝原因</th>
          <th>提交边界</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${strategyRuntime.map((runtime) => `
          <tr>
            <td data-label="节点"><strong>${text(runtime.node_id)}</strong><div class="muted">${displayText(snapshotValue(runtime.strategy_id))}</div></td>
            <td data-label="健康"><span class="status-${safe(runtime.health)}">${displayText(runtime.health)}</span><div class="muted">${displayText(snapshotValue(runtime.diagnostic))}</div></td>
            <td data-label="会话"><span class="status-${safe(snapshotValue(runtime.session_state))}">${displayText(snapshotValue(runtime.session_state))}</span><div class="muted">${displayText(snapshotValue(runtime.session_id))}</div></td>
            <td data-label="市场流">${displayText(snapshotValue(runtime.market_stream_status))}<div class="muted">${displayText(snapshotValue(runtime.symbol))}</div></td>
            <td data-label="信号">${displayText(snapshotValue(runtime.signal_count))}<div class="muted">${displayText(snapshotValue(runtime.latest_signal))}</div></td>
            <td data-label="Order Intent">${displayText(snapshotValue(runtime.latest_order_intent))}</td>
            <td data-label="风控决策">${displayText(snapshotValue(runtime.latest_risk_decision))}</td>
            <td data-label="拒绝原因">${displayText(snapshotValue(runtime.rejection_reason))}</td>
            <td data-label="提交边界">${panelRow("模式", snapshotValue(runtime.order_submission_mode))}${panelRow("实际提交", snapshotValue(runtime.actual_submission_count))}</td>
            <td data-label="工件" class="path">
              ${panelRow("session", snapshotValue(runtime.session_status_path))}
              ${panelRow("signal", snapshotValue(runtime.signal_artifact_path))}
              ${panelRow("intent", snapshotValue(runtime.order_intent_artifact_path))}
              ${panelRow("risk", snapshotValue(runtime.risk_decision_artifact_path))}
              ${panelRow("summary", snapshotValue(runtime.summary_artifact_path))}
              ${panelRow("manifest", snapshotValue(runtime.manifest_path))}
            </td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 Strategy Runtime 工件");
}

function renderProductionShadow(productionShadow) {
  document.getElementById("production-shadow").innerHTML = productionShadow.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>健康</th>
          <th>只读探测</th>
          <th>账户快照</th>
          <th>Shadow Intent</th>
          <th>组合快照</th>
          <th>策略会话</th>
          <th>Reconciliation</th>
          <th>Risk halt</th>
          <th>边界</th>
          <th>Manifest</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${productionShadow.map((shadow) => `
          <tr>
            <td data-label="节点"><strong>${text(shadow.node_id)}</strong></td>
            <td data-label="健康"><span class="status-${safe(shadow.health)}">${displayText(shadow.health)}</span><div class="muted">${displayText(snapshotValue(shadow.diagnostic))}</div></td>
            <td data-label="只读探测">${panelRow("版本", snapshotValue(shadow.artifact_version))}${panelRow("Public", snapshotValue(shadow.public_read_status))}${panelRow("Endpoint", snapshotValue(shadow.public_read_endpoint_class))}${panelRow("Shape", snapshotValue(shadow.response_shape_status))}${panelRow("Shape OK", snapshotValue(shadow.response_shape_validated))}</td>
            <td data-label="账户快照">${displayText(snapshotValue(shadow.account_snapshot_status))}<div class="muted">${displayText(snapshotValue(shadow.account_snapshot_endpoint_class))}</div></td>
            <td data-label="Shadow Intent">${displayText(snapshotValue(shadow.shadow_intent_status))}<div class="muted">${displayText(snapshotValue(shadow.shadow_intents_created))} intents</div></td>
            <td data-label="组合快照">${displayText(snapshotValue(shadow.portfolio_snapshot_status))}<div class="muted">${displayText(snapshotValue(shadow.portfolio_exposure_status))} / ${displayText(snapshotValue(shadow.portfolio_pnl_status))}</div></td>
            <td data-label="策略会话">${displayText(snapshotValue(shadow.shadow_strategy_session_status))}<div class="muted">${displayText(snapshotValue(shadow.shadow_strategy_session_heartbeats))} heartbeats</div><div class="muted">${displayText(snapshotValue(shadow.lifecycle_status))} / ${displayText(snapshotValue(shadow.lifecycle_events_created))} legacy events</div></td>
            <td data-label="Reconciliation">${displayText(snapshotValue(shadow.reconciliation_status))}<div class="muted">${displayText(snapshotValue(shadow.reconciliation_classification))}</div><div class="muted">${displayText(snapshotValue(shadow.reconciliation_events_created))} events</div></td>
            <td data-label="Risk halt">${panelRow("Risk halt", snapshotValue(shadow.risk_halted))}${panelRow("人工复核", snapshotValue(shadow.manual_review_required))}${panelRow("新单阻断", snapshotValue(shadow.new_orders_blocked))}${panelRow("Kill switch", snapshotValue(shadow.kill_switch_status))}${panelRow("Kill active", snapshotValue(shadow.kill_switch_active))}${panelRow("Dry-run", snapshotValue(shadow.kill_switch_dry_run))}${panelRow("审批", snapshotValue(shadow.kill_switch_approval_state))}${panelRow("审批记录", snapshotValue(shadow.kill_switch_manual_approval_recorded))}${panelRow("动作", snapshotValue(shadow.reconciliation_recommended_action))}</td>
            <td data-label="边界">${panelRow("实际提交", snapshotValue(shadow.actual_submission_count))}${panelRow("生产提交尝试", snapshotValue(shadow.production_order_submissions_attempted))}${panelRow("生产提交", snapshotValue(shadow.production_orders_submitted))}${panelRow("生产变更", snapshotValue(shadow.production_order_mutations_attempted))}${panelRow("订单状态读取", snapshotValue(shadow.production_order_state_reads_attempted))}${panelRow("listenKey", snapshotValue(shadow.listen_key_lifecycle_attempted))}${panelRow("自动纠错", snapshotValue(shadow.automatic_correction_orders_submitted))}${panelRow("提交允许", snapshotValue(shadow.kill_switch_production_order_submission_allowed))}${panelRow("变更允许", snapshotValue(shadow.kill_switch_production_order_mutation_allowed))}${panelRow("状态读取允许", snapshotValue(shadow.kill_switch_production_order_state_reads_allowed))}${panelRow("listenKey允许", snapshotValue(shadow.kill_switch_listen_key_lifecycle_allowed))}${panelRow("Dashboard 下单控件", snapshotValue(shadow.dashboard_order_controls_enabled))}${panelRow("真实订单", snapshotValue(shadow.real_orders_submitted))}${panelRow("订单状态真值", snapshotValue(shadow.order_state_values_are_exchange_truth))}${panelRow("Shadow 真值", snapshotValue(shadow.shadow_values_are_exchange_truth))}${panelRow("Portfolio 真值", snapshotValue(shadow.portfolio_values_are_exchange_truth))}${panelRow("兼容真值", snapshotValue(shadow.values_are_exchange_truth))}</td>
            <td data-label="Manifest">${displayText(snapshotValue(shadow.manifest_status))}<div class="muted">${displayText(snapshotValue(shadow.manifest_artifact_count))} artifacts</div></td>
            <td data-label="工件" class="path">
              ${panelRow("manifest", snapshotValue(shadow.manifest_path))}
              ${panelRow("public", snapshotValue(shadow.public_read_probe_path))}
              ${panelRow("account", snapshotValue(shadow.account_snapshot_path))}
              ${panelRow("shape", snapshotValue(shadow.response_shape_path))}
              ${panelRow("intent", snapshotValue(shadow.shadow_intent_path))}
              ${panelRow("portfolio", snapshotValue(shadow.portfolio_snapshot_path))}
              ${panelRow("session", snapshotValue(shadow.shadow_strategy_session_path))}
              ${panelRow("lifecycle", snapshotValue(shadow.lifecycle_path))}
              ${panelRow("reconciliation", snapshotValue(shadow.reconciliation_path))}
              ${panelRow("kill-switch", snapshotValue(shadow.kill_switch_approval_artifact_path))}
            </td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 Production Shadow 工件");
}

function renderPreflightReadiness(readiness) {
  document.getElementById("preflight-readiness").innerHTML = readiness.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>健康</th>
          <th>证据状态</th>
          <th>边界</th>
          <th>诊断</th>
        </tr>
      </thead>
      <tbody>
        ${readiness.map((item) => `
          <tr>
            <td data-label="节点"><strong>${text(item.node_id)}</strong><div class="muted">${displayText(snapshotValue(item.evidence_source))}</div></td>
            <td data-label="健康"><span class="status-${safe(item.health)}">${displayText(item.health)}</span><div class="muted">${displayText(snapshotValue(item.readiness_status))}</div></td>
            <td data-label="证据状态">${panelRow("Proof pack", snapshotValue(item.owner_proof_pack_status))}${panelRow("Kill switch", snapshotValue(item.kill_switch_artifact_status))}${panelRow("Shadow preflight", snapshotValue(item.bounded_shadow_preflight_status))}${panelRow("Decimal", snapshotValue(item.decimal_boundary_status))}${panelRow("No mutation gate", snapshotValue(item.no_production_mutation_gate_status))}</td>
            <td data-label="边界">${panelRow("提交允许", snapshotValue(item.production_order_submission_allowed))}${panelRow("变更允许", snapshotValue(item.production_order_mutation_allowed))}${panelRow("状态读取允许", snapshotValue(item.production_order_state_reads_allowed))}${panelRow("listenKey允许", snapshotValue(item.listen_key_lifecycle_allowed))}${panelRow("Dashboard 下单控件", snapshotValue(item.dashboard_order_controls_enabled))}${panelRow("真实订单", snapshotValue(item.real_orders_submitted))}${panelRow("订单状态真值", snapshotValue(item.order_state_values_are_exchange_truth))}${panelRow("Shadow 真值", snapshotValue(item.shadow_values_are_exchange_truth))}${panelRow("Portfolio 真值", snapshotValue(item.portfolio_values_are_exchange_truth))}${panelRow("兼容真值", snapshotValue(item.values_are_exchange_truth))}</td>
            <td data-label="诊断">${displayText(snapshotValue(item.diagnostic))}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 v0.13 预检就绪工件");
}

function renderLiveAlphaDryRun(items) {
  document.getElementById("live-alpha-dry-run").innerHTML = items.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>就绪</th>
          <th>Dry-run Gate</th>
          <th>风控预检</th>
          <th>人工审批</th>
          <th>请求预览</th>
          <th>执行 Dry-run</th>
          <th>Runtime Gate</th>
          <th>Order State</th>
          <th>Reconciliation</th>
          <th>只读边界</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${items.map((item) => `
          <tr>
            <td data-label="节点"><strong>${text(item.node_id)}</strong></td>
            <td data-label="就绪"><span class="status-${safe(item.health)}">${displayText(item.health)}</span><div class="muted">${displayText(snapshotValue(item.readiness_status))}</div><div class="muted">${displayText(snapshotValue(item.diagnostic))}</div></td>
            <td data-label="Dry-run Gate">${panelRow("状态", snapshotValue(item.gate_status))}${panelRow("Gate ready", snapshotValue(item.gate_ready))}${panelRow("Intent", snapshotValue(item.dry_run_order_intent_recorded))}${panelRow("模式", snapshotValue(item.order_submission_mode))}${panelRow("缺失 gate", snapshotValue(item.missing_gate_flags))}</td>
            <td data-label="风控预检">${panelRow("状态", snapshotValue(item.risk_preflight_status))}${panelRow("风控干跑", snapshotValue(item.risk_decision))}${panelRow("执行决策", snapshotValue(item.execution_decision))}${panelRow("原因", snapshotValue(item.risk_reasons))}${panelRow("Kill switch", snapshotValue(item.kill_switch_active))}</td>
            <td data-label="人工审批">${panelRow("状态", snapshotValue(item.manual_approval_status))}${panelRow("审批状态", snapshotValue(item.manual_approval_state))}${panelRow("已记录", snapshotValue(item.manual_approval_recorded))}${panelRow("有效", snapshotValue(item.manual_approval_valid))}${panelRow("问题", snapshotValue(item.manual_approval_issues))}${panelRow("一次性", snapshotValue(item.manual_approval_one_time))}${panelRow("已使用", snapshotValue(item.manual_approval_used))}${panelRow("过期时间", snapshotValue(item.manual_approval_expires_at_unix_ms))}</td>
            <td data-label="请求预览">${panelRow("状态", snapshotValue(item.request_preview_status))}${panelRow("允许预览", snapshotValue(item.request_preview_allowed))}${panelRow("已构建", snapshotValue(item.request_preview_built))}${panelRow("已发送", snapshotValue(item.request_sent))}${panelRow("方法", snapshotValue(item.request_method))}${panelRow("目标", snapshotValue(item.request_target))}${panelRow("端点类型", snapshotValue(item.endpoint_class))}${panelRow("端点决策", snapshotValue(item.endpoint_decision))}${panelRow("Query shape", snapshotValue(item.query_shape_without_signature))}${panelRow("签名预检", snapshotValue(item.signature_preflight))}${panelRow("密钥脱敏", snapshotValue(item.secrets_redacted))}${panelRow("签名仅内存", snapshotValue(item.signed_request_memory_only))}</td>
            <td data-label="执行 Dry-run">${panelRow("状态", snapshotValue(item.execution_dry_run_status))}${panelRow("Dry-run adapter", snapshotValue(item.dry_run_execution_adapter_called))}${panelRow("写入工件", snapshotValue(item.dry_run_execution_adapter_wrote_artifact))}${panelRow("仅工件", snapshotValue(item.dry_run_adapter_artifact_only))}${panelRow("生产 adapter", snapshotValue(item.production_adapter_called))}${panelRow("实例化生产 adapter", snapshotValue(item.production_adapter_instantiated))}${panelRow("Intent 已记录", snapshotValue(item.strategy_intent_recorded))}${panelRow("到达风控", snapshotValue(item.strategy_intent_reaches_risk_preflight))}${panelRow("到达 dry-run", snapshotValue(item.strategy_intent_reaches_dry_run_adapter))}${panelRow("到达生产 adapter", snapshotValue(item.strategy_intent_reaches_production_adapter))}</td>
            <td data-label="Runtime Gate">${panelRow("状态", snapshotValue(item.kill_switch_runtime_gate_status))}${panelRow("决策", snapshotValue(item.runtime_gate_decision))}${panelRow("Gate open", snapshotValue(item.runtime_gate_open))}${panelRow("原因", snapshotValue(item.runtime_gate_reasons))}</td>
            <td data-label="Order State">${panelRow("可读", snapshotValue(item.order_state_readable))}${panelRow("读取状态", snapshotValue(item.order_state_read_status))}${panelRow("端点", snapshotValue(item.order_state_endpoint))}${panelRow("网络尝试", snapshotValue(item.order_state_network_attempted))}${panelRow("读取尝试", snapshotValue(item.order_state_read_attempted))}${panelRow("Shape", snapshotValue(item.order_state_shape_validated))}${panelRow("Open orders", snapshotValue(item.open_order_count))}${panelRow("非空订单", snapshotValue(item.non_empty_order_state_observed))}${panelRow("生命周期", snapshotValue(item.order_lifecycle_readiness))}${panelRow("真值来源", snapshotValue(item.order_state_truth_source))}${panelRow("Age", snapshotValue(item.order_state_age_ms))}${panelRow("Max age", snapshotValue(item.max_order_state_age_ms))}${panelRow("Max open", snapshotValue(item.max_open_orders))}</td>
            <td data-label="Reconciliation">${panelRow("状态", snapshotValue(item.reconciliation_status))}${panelRow("提交允许", snapshotValue(item.production_order_submission_allowed))}${panelRow("变更允许", snapshotValue(item.production_order_mutation_allowed))}${panelRow("状态读取允许", snapshotValue(item.production_order_state_reads_allowed))}${panelRow("listenKey允许", snapshotValue(item.listen_key_lifecycle_allowed))}</td>
            <td data-label="只读边界">${panelRow("提交尝试", snapshotValue(item.production_order_submissions_attempted))}${panelRow("生产提交", snapshotValue(item.production_orders_submitted))}${panelRow("生产变更", snapshotValue(item.production_order_mutations_attempted))}${panelRow("状态读取尝试", snapshotValue(item.production_order_state_reads_attempted))}${panelRow("listenKey尝试", snapshotValue(item.listen_key_lifecycle_attempted))}${panelRow("撤改尝试", snapshotValue(item.cancel_replace_amend_attempted))}${panelRow("Execution adapter", snapshotValue(item.execution_adapter_called))}${panelRow("订单端点", snapshotValue(item.order_endpoint_access_attempted))}${panelRow("撮合提交", snapshotValue(item.matching_engine_submission))}${panelRow("实际提交", snapshotValue(item.actual_submission_count))}${panelRow("自动纠错", snapshotValue(item.automatic_correction_orders_submitted))}${panelRow("Dashboard 下单控件", snapshotValue(item.dashboard_order_controls_enabled))}${panelRow("网络尝试", snapshotValue(item.network_attempted))}${panelRow("真实订单", snapshotValue(item.real_orders_submitted))}${panelRow("真实资金", snapshotValue(item.real_funds))}${panelRow("生产交易", snapshotValue(item.production_trading_enabled))}${panelRow("订单状态真值", snapshotValue(item.order_state_values_are_exchange_truth))}${panelRow("Shadow 真值", snapshotValue(item.shadow_values_are_exchange_truth))}${panelRow("Portfolio 真值", snapshotValue(item.portfolio_values_are_exchange_truth))}${panelRow("兼容真值", snapshotValue(item.values_are_exchange_truth))}</td>
            <td data-label="工件" class="path">
              ${panelRow("gate", snapshotValue(item.order_gate_path))}
              ${panelRow("risk", snapshotValue(item.risk_preflight_path))}
              ${panelRow("order-state", snapshotValue(item.order_state_proof_path))}
              ${panelRow("approval", snapshotValue(item.manual_approval_lifecycle_path))}
              ${panelRow("request-preview", snapshotValue(item.request_preview_path))}
              ${panelRow("execution-dry-run", snapshotValue(item.execution_dry_run_path))}
              ${panelRow("runtime-gate", snapshotValue(item.kill_switch_runtime_gate_path))}
            </td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 v0.14 Live Alpha dry-run 工件");
}

function renderProductionMutationEvidence(items) {
  document.getElementById("production-mutation-evidence").innerHTML = items.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>就绪</th>
          <th>审批 / Runtime</th>
          <th>请求 / Send</th>
          <th>响应 / Readback</th>
          <th>审计 / 失败</th>
          <th>候选单</th>
          <th>边界</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${items.map((item) => `
          <tr>
            <td data-label="节点"><strong>${text(item.node_id)}</strong></td>
            <td data-label="就绪"><span class="status-${safe(item.health)}">${displayText(item.health)}</span><div class="muted">${displayText(snapshotValue(item.readiness_status))}</div><div class="muted">${displayText(snapshotValue(item.diagnostic))}</div></td>
            <td data-label="审批 / Runtime">${panelRow("Runtime gate", snapshotValue(item.runtime_gate_status))}${panelRow("Gate open", snapshotValue(item.runtime_gate_open))}${panelRow("签名审批", snapshotValue(item.signing_approval_status))}${panelRow("审批状态", snapshotValue(item.approval_state))}${panelRow("人工记录", snapshotValue(item.manual_approval_recorded))}${panelRow("审批人", snapshotValue(item.approved_by))}</td>
            <td data-label="请求 / Send">${panelRow("Request builder", snapshotValue(item.request_builder_status))}${panelRow("Builder ready", snapshotValue(item.request_builder_ready))}${panelRow("Guarded send", snapshotValue(item.guarded_send_status))}${panelRow("Send ready", snapshotValue(item.guarded_send_ready))}${panelRow("请求发送", snapshotValue(item.request_sent))}${panelRow("网络尝试", snapshotValue(item.network_attempted))}${panelRow("Kill before", snapshotValue(item.kill_switch_checked_before_send))}${panelRow("Kill after", snapshotValue(item.kill_switch_checked_after_send))}</td>
            <td data-label="响应 / Readback">${panelRow("响应脱敏", snapshotValue(item.response_redaction_status))}${panelRow("Redaction ready", snapshotValue(item.response_redaction_ready))}${panelRow("Readback", snapshotValue(item.order_state_readback_status))}${panelRow("Readback ready", snapshotValue(item.readback_contract_ready))}${panelRow("状态读取", snapshotValue(item.order_state_read_attempted))}${panelRow("Shape", snapshotValue(item.response_shape_validated))}</td>
            <td data-label="审计 / 失败">${panelRow("审计", snapshotValue(item.audit_trail_status))}${panelRow("Audit ready", snapshotValue(item.audit_trail_ready))}${panelRow("Failure", snapshotValue(item.failure_semantics_status))}${panelRow("Failure ready", snapshotValue(item.failure_semantics_ready))}${panelRow("模式", snapshotValue(item.failure_mode))}${panelRow("动作", snapshotValue(item.terminal_action))}${panelRow("继续策略", snapshotValue(item.strategy_continuation_allowed))}</td>
            <td data-label="候选单">${panelRow("Symbol", snapshotValue(item.symbol))}${panelRow("Side", snapshotValue(item.side))}${panelRow("Type", snapshotValue(item.order_type))}${panelRow("TIF", snapshotValue(item.time_in_force))}${panelRow("Qty", snapshotValue(item.quantity))}${panelRow("Price", snapshotValue(item.price))}${panelRow("Order ID", snapshotValue(item.order_id))}</td>
            <td data-label="边界">${panelRow("提交尝试", snapshotValue(item.production_order_submissions_attempted))}${panelRow("生产提交", snapshotValue(item.production_orders_submitted))}${panelRow("生产变更", snapshotValue(item.production_order_mutations_attempted))}${panelRow("状态读取", snapshotValue(item.production_order_state_reads_attempted))}${panelRow("listenKey", snapshotValue(item.listen_key_lifecycle_attempted))}${panelRow("重试", snapshotValue(item.retry_attempted))}${panelRow("撤单", snapshotValue(item.cancel_attempted))}${panelRow("改单", snapshotValue(item.replace_attempted))}${panelRow("Amend", snapshotValue(item.amend_attempted))}${panelRow("纠错", snapshotValue(item.correction_attempted))}${panelRow("Flatten", snapshotValue(item.flatten_attempted))}${panelRow("自动补救", snapshotValue(item.remediation_attempted))}${panelRow("Dashboard 下单控件", snapshotValue(item.dashboard_order_controls_enabled))}${panelRow("真实订单", snapshotValue(item.real_orders_submitted))}${panelRow("生产交易", snapshotValue(item.production_trading_enabled))}</td>
            <td data-label="工件" class="path">
              ${panelRow("runtime", snapshotValue(item.runtime_gate_path))}
              ${panelRow("signing", snapshotValue(item.signing_approval_path))}
              ${panelRow("request", snapshotValue(item.request_builder_path))}
              ${panelRow("send", snapshotValue(item.guarded_send_path))}
              ${panelRow("redaction", snapshotValue(item.response_redaction_path))}
              ${panelRow("readback", snapshotValue(item.order_state_readback_path))}
              ${panelRow("audit", snapshotValue(item.audit_trail_path))}
              ${panelRow("failure", snapshotValue(item.failure_semantics_path))}
            </td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 v0.16 production mutation evidence 工件");
}

function renderProductionReconciliationOrphan(items) {
  document.getElementById("production-reconciliation-orphan").innerHTML = items.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>当前结论</th>
          <th>Lineage / 本地</th>
          <th>交易所 Readback</th>
          <th>对账</th>
          <th>孤儿单风险</th>
          <th>安全边界</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${items.map((item) => `
          <tr>
            <td data-label="节点"><strong>${text(item.node_id)}</strong></td>
            <td data-label="当前结论"><span class="status-${safe(item.health)}">${displayText(item.health)}</span><div class="muted">${displayText(snapshotValue(item.readiness_status))}</div><div class="muted">${displayText(snapshotValue(item.diagnostic))}</div>${panelRow("缺失证据", snapshotValue(item.missing_artifacts))}${panelRow("Schema 诊断", snapshotValue(item.schema_diagnostics))}${panelRow("Provenance 诊断", snapshotValue(item.provenance_diagnostics))}${panelRow("Stale 证据", snapshotValue(item.stale_artifacts))}</td>
            <td data-label="Lineage / 本地">${panelRow("Lineage", snapshotValue(item.order_lineage_id))}${panelRow("Ledger", snapshotValue(item.local_ledger_status))}${panelRow("本地状态", snapshotValue(item.local_order_state))}${panelRow("Ledger ready", snapshotValue(item.local_ledger_ready))}${panelRow("可重启读取", snapshotValue(item.restart_readable))}</td>
            <td data-label="交易所 Readback">${panelRow("Mapper", snapshotValue(item.exchange_readback_status))}${panelRow("已映射", snapshotValue(item.exchange_readback_mapped))}${panelRow("状态", snapshotValue(item.exchange_order_state))}${panelRow("原始状态", snapshotValue(item.exchange_order_status))}${panelRow("Open order", snapshotValue(item.open_order_observed))}${panelRow("终态", snapshotValue(item.terminal_state_observed))}</td>
            <td data-label="对账">${panelRow("Classifier", snapshotValue(item.reconciliation_status))}${panelRow("已分类", snapshotValue(item.reconciliation_classified))}${panelRow("结果", snapshotValue(item.reconciliation_outcome))}${panelRow("人工复核", snapshotValue(item.manual_review_required))}${panelRow("新单阻断", snapshotValue(item.new_orders_blocked))}</td>
            <td data-label="孤儿单风险">${panelRow("Detector", snapshotValue(item.orphan_status))}${panelRow("检测完成", snapshotValue(item.orphan_detection_completed))}${panelRow("结果", snapshotValue(item.orphan_detection_outcome))}${panelRow("孤儿单风险", snapshotValue(item.orphan_risk_detected))}${panelRow("风控暂停", snapshotValue(item.risk_halted))}${panelRow("需重启恢复", snapshotValue(item.stale_ledger_restart_required))}</td>
            <td data-label="安全边界">${panelRow("重复提交", snapshotValue(item.duplicate_submit_attempted))}${panelRow("重试", snapshotValue(item.retry_attempted))}${panelRow("撤单", snapshotValue(item.cancel_attempted))}${panelRow("自动补救", snapshotValue(item.remediation_attempted))}${panelRow("自动撤单", snapshotValue(item.automatic_cancel_allowed))}${panelRow("Dashboard 下单控件", snapshotValue(item.dashboard_order_controls_enabled))}${panelRow("Dashboard 撤单控件", snapshotValue(item.dashboard_cancel_controls_enabled))}${panelRow("网络尝试", snapshotValue(item.network_attempted))}${panelRow("提交允许", snapshotValue(item.production_order_submission_allowed))}${panelRow("变更允许", snapshotValue(item.production_order_mutation_allowed))}</td>
            <td data-label="工件" class="path">
              ${panelRow("ledger", snapshotValue(item.local_order_ledger_path))}
              ${panelRow("readback", snapshotValue(item.exchange_readback_mapper_path))}
              ${panelRow("classifier", snapshotValue(item.reconciliation_classifier_path))}
              ${panelRow("orphan", snapshotValue(item.orphan_order_detector_path))}
            </td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 v0.17 对账与孤儿单风险工件");
}

function renderProductionCancelRecovery(items) {
  document.getElementById("production-cancel-recovery").innerHTML = items.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>当前结论</th>
          <th>撤单预览</th>
          <th>风险门禁</th>
          <th>Owner 审批</th>
          <th>Readback</th>
          <th>Incident / Audit</th>
          <th>只读边界</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${items.map((item) => `
          <tr>
            <td data-label="节点"><strong>${text(item.node_id)}</strong></td>
            <td data-label="当前结论"><span class="status-${safe(item.health)}">${displayText(item.health)}</span><div class="muted">${displayText(snapshotValue(item.readiness_status))}</div><div class="muted">${displayText(snapshotValue(item.diagnostic))}</div>${panelRow("缺失证据", snapshotValue(item.missing_artifacts))}${panelRow("Schema 诊断", snapshotValue(item.schema_diagnostics))}${panelRow("Provenance 诊断", snapshotValue(item.provenance_diagnostics))}${panelRow("Stale 证据", snapshotValue(item.stale_artifacts))}</td>
            <td data-label="撤单预览">${panelRow("Lineage", snapshotValue(item.order_lineage_id))}${panelRow("状态", snapshotValue(item.cancel_preview_status))}${panelRow("Preview ready", snapshotValue(item.cancel_request_preview_ready))}${panelRow("原因", snapshotValue(item.cancel_reason))}${panelRow("候选数", snapshotValue(item.candidate_count))}${panelRow("订单", snapshotValue(item.known_order_id))}${panelRow("Client", snapshotValue(item.known_client_order_id))}${panelRow("Symbol", snapshotValue(item.symbol))}${panelRow("Account", snapshotValue(item.account_label))}</td>
            <td data-label="风险门禁">${panelRow("状态", snapshotValue(item.risk_gate_status))}${panelRow("结果", snapshotValue(item.risk_gate_result))}${panelRow("Gate ready", snapshotValue(item.risk_gate_ready))}${panelRow("孤儿单风险", snapshotValue(item.orphan_risk_detected))}${panelRow("风控暂停", snapshotValue(item.risk_halted))}${panelRow("人工复核", snapshotValue(item.manual_review_required))}${panelRow("新单阻断", snapshotValue(item.new_orders_blocked))}</td>
            <td data-label="Owner 审批">${panelRow("状态", snapshotValue(item.approval_lifecycle_status))}${panelRow("审批状态", snapshotValue(item.owner_approval_state))}${panelRow("已记录", snapshotValue(item.manual_approval_recorded))}${panelRow("生命周期有效", snapshotValue(item.approval_lifecycle_valid))}${panelRow("已消耗", snapshotValue(item.approval_consumed))}</td>
            <td data-label="Readback">${panelRow("脱敏合同", snapshotValue(item.redaction_contract_state))}${panelRow("脱敏 ready", snapshotValue(item.cancel_response_redaction_ready))}${panelRow("响应已脱敏", snapshotValue(item.cancel_response_redacted))}${panelRow("Readback 状态", snapshotValue(item.post_cancel_readback_status))}${panelRow("Readback ready", snapshotValue(item.post_cancel_readback_ready))}${panelRow("交易所状态", snapshotValue(item.readback_state))}${panelRow("状态分类", snapshotValue(item.readback_state_class))}${panelRow("结果", snapshotValue(item.readback_outcome))}${panelRow("终态", snapshotValue(item.terminal_state_observed))}${panelRow("歧义状态", snapshotValue(item.ambiguous_state_observed))}</td>
            <td data-label="Incident / Audit">${panelRow("Closeout 状态", snapshotValue(item.incident_closeout_status))}${panelRow("Incident ready", snapshotValue(item.incident_closeout_ready))}${panelRow("Audit ready", snapshotValue(item.audit_trail_ready))}${panelRow("Traceability", snapshotValue(item.audit_traceability_ready))}${panelRow("Lineage ready", snapshotValue(item.cancel_recovery_lineage_ready))}${panelRow("推荐动作", snapshotValue(item.terminal_action_recommendation))}${panelRow("剩余风险", snapshotValue(item.remaining_risk))}${panelRow("需人工复核", snapshotValue(item.remaining_risk_requires_manual_review))}${panelRow("Source issues", snapshotValue(item.source_artifact_issues))}${panelRow("Lineage issues", snapshotValue(item.lineage_issues))}${panelRow("Missing flags", snapshotValue(item.missing_cli_flags))}</td>
            <td data-label="只读边界">${panelRow("允许真实撤单", snapshotValue(item.actual_cancel_send_allowed))}${panelRow("撤单尝试", snapshotValue(item.cancel_attempted))}${panelRow("撤单请求", snapshotValue(item.cancel_requests_sent))}${panelRow("生产变更", snapshotValue(item.production_order_mutations_attempted))}${panelRow("Readback 执行", snapshotValue(item.readback_execution_attempted))}${panelRow("状态读取尝试", snapshotValue(item.production_order_state_reads_attempted))}${panelRow("网络尝试", snapshotValue(item.network_attempted))}${panelRow("Readback 端点", snapshotValue(item.network_readback_endpoint_attempted))}${panelRow("Cancel 端点", snapshotValue(item.network_cancel_endpoint_attempted))}${panelRow("重试", snapshotValue(item.retry_attempted))}${panelRow("补救", snapshotValue(item.remediation_attempted))}${panelRow("自动撤单", snapshotValue(item.automatic_cancel_allowed))}${panelRow("自动补救", snapshotValue(item.automatic_remediation_allowed))}${panelRow("Dashboard 下单控件", snapshotValue(item.dashboard_order_controls_enabled))}${panelRow("Dashboard 撤单控件", snapshotValue(item.dashboard_cancel_controls_enabled))}${panelRow("Dashboard 自动审批", snapshotValue(item.dashboard_auto_approval_allowed))}${panelRow("Dashboard 审批尝试", snapshotValue(item.dashboard_auto_approval_attempted))}</td>
            <td data-label="工件" class="path">
              ${panelRow("preview", snapshotValue(item.cancel_request_preview_path))}
              ${panelRow("risk-gate", snapshotValue(item.cancel_risk_gate_path))}
              ${panelRow("approval", snapshotValue(item.manual_owner_approval_lifecycle_path))}
              ${panelRow("redaction", snapshotValue(item.cancel_response_redaction_path))}
              ${panelRow("readback", snapshotValue(item.post_cancel_readback_path))}
              ${panelRow("closeout", snapshotValue(item.incident_audit_closeout_path))}
            </td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 v0.18 撤单恢复只读工件");
}

function renderProductionActualCancelAudit(items) {
  document.getElementById("production-actual-cancel-audit").innerHTML = items.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>当前结论</th>
          <th>Approval / Risk</th>
          <th>Cancel Attempt / Venue</th>
          <th>Readback</th>
          <th>Outcome / Audit</th>
          <th>只读边界</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${items.map((item) => `
          <tr>
            <td data-label="节点"><strong>${text(item.node_id)}</strong></td>
            <td data-label="当前结论"><span class="status-${safe(item.health)}">${displayText(item.health)}</span><div class="muted">${displayText(snapshotValue(item.readiness_status))}</div><div class="muted">${displayText(snapshotValue(item.audit_state))}</div><div class="muted">${displayText(snapshotValue(item.diagnostic))}</div>${panelRow("缺失证据", snapshotValue(item.missing_artifacts))}${panelRow("Schema 诊断", snapshotValue(item.schema_diagnostics))}${panelRow("Provenance 诊断", snapshotValue(item.provenance_diagnostics))}${panelRow("Stale 证据", snapshotValue(item.stale_artifacts))}</td>
            <td data-label="Approval / Risk">${panelRow("Lineage", snapshotValue(item.order_lineage_id))}${panelRow("审批工件", snapshotValue(item.approval_lifecycle_status))}${panelRow("审批状态", snapshotValue(item.owner_approval_state))}${panelRow("审批有效", snapshotValue(item.approval_lifecycle_valid))}${panelRow("执行授权", snapshotValue(item.approval_execution_authorized))}${panelRow("Risk gate", snapshotValue(item.risk_gate_status))}${panelRow("Risk result", snapshotValue(item.risk_gate_result))}${panelRow("Risk ready", snapshotValue(item.risk_gate_ready))}</td>
            <td data-label="Cancel Attempt / Venue">${panelRow("Attempt 状态", snapshotValue(item.cancel_attempt_status))}${panelRow("命令 ready", snapshotValue(item.actual_cancel_command_ready))}${panelRow("Single-shot gate", snapshotValue(item.single_shot_cancel_allowed))}${panelRow("Request sent", snapshotValue(item.request_sent))}${panelRow("Cancel attempted", snapshotValue(item.cancel_attempted))}${panelRow("Cancel requests", snapshotValue(item.cancel_requests_sent))}${panelRow("Request ID", snapshotValue(item.request_id))}${panelRow("Venue status", snapshotValue(item.venue_response_status))}${panelRow("Venue source", snapshotValue(item.venue_response_source))}${panelRow("Venue code", snapshotValue(item.venue_response_code))}${panelRow("Venue error", snapshotValue(item.venue_response_error_code))}${panelRow("Local audit", snapshotValue(item.local_audit_reference))}</td>
            <td data-label="Readback">${panelRow("Readback 状态", snapshotValue(item.readback_status))}${panelRow("Readback result", snapshotValue(item.readback_result))}${panelRow("Reconciliation", snapshotValue(item.reconciliation_status))}${panelRow("Readback state", snapshotValue(item.readback_state))}${panelRow("Venue state", snapshotValue(item.venue_state))}${panelRow("终态", snapshotValue(item.terminal_state_observed))}${panelRow("Unknown", snapshotValue(item.unknown_observed))}${panelRow("Dashboard 只读可消费", snapshotValue(item.dashboard_read_only_consumable))}${panelRow("Audit view ready", snapshotValue(item.dashboard_audit_view_ready))}</td>
            <td data-label="Outcome / Audit">${panelRow("Evidence 状态", snapshotValue(item.outcome_status))}${panelRow("Cancel outcome", snapshotValue(item.cancel_outcome))}${panelRow("Outcome category", snapshotValue(item.outcome_category))}${panelRow("Recovered", snapshotValue(item.recovered))}${panelRow("Degraded", snapshotValue(item.degraded))}${panelRow("Failed", snapshotValue(item.failed))}${panelRow("Partial success", snapshotValue(item.partial_success))}${panelRow("Operator action", snapshotValue(item.operator_action_required))}${panelRow("Residual risk", snapshotValue(item.residual_risk_visible))}${panelRow("Audit refs", snapshotValue(item.request_response_readback_audit_refs_recorded))}${panelRow("Source issues", snapshotValue(item.source_artifact_issues))}${panelRow("Lineage issues", snapshotValue(item.lineage_issues))}${panelRow("Missing flags", snapshotValue(item.missing_cli_flags))}</td>
            <td data-label="只读边界">${panelRow("Dashboard 下单控件", snapshotValue(item.dashboard_order_controls_enabled))}${panelRow("Dashboard 撤单控件", snapshotValue(item.dashboard_cancel_controls_enabled))}${panelRow("重试", snapshotValue(item.retry_attempted))}${panelRow("补救", snapshotValue(item.remediation_attempted))}${panelRow("自动撤单", snapshotValue(item.automatic_cancel_allowed))}${panelRow("自动补救", snapshotValue(item.automatic_remediation_allowed))}${panelRow("批量撤单允许标记", snapshotValue(item.bulk_cancel_allowed))}${panelRow("二次撤单尝试", snapshotValue(item.second_cancel_attempted))}${panelRow("补偿交易", snapshotValue(item.compensation_trade_attempted))}${panelRow("实际发送允许标记", snapshotValue(item.actual_cancel_send_allowed))}${panelRow("生产变更次数", snapshotValue(item.production_order_mutations_attempted))}${panelRow("Readback 执行", snapshotValue(item.readback_execution_attempted))}${panelRow("状态读取次数", snapshotValue(item.production_order_state_reads_attempted))}${panelRow("网络尝试", snapshotValue(item.network_attempted))}${panelRow("Readback 端点", snapshotValue(item.network_readback_endpoint_attempted))}${panelRow("Cancel 端点", snapshotValue(item.network_cancel_endpoint_attempted))}</td>
            <td data-label="工件" class="path">
              ${panelRow("risk-gate", snapshotValue(item.cancel_risk_gate_path))}
              ${panelRow("approval", snapshotValue(item.owner_approval_lifecycle_path))}
              ${panelRow("attempt", snapshotValue(item.actual_cancel_single_shot_path))}
              ${panelRow("readback", snapshotValue(item.readback_reconciliation_path))}
              ${panelRow("failure", snapshotValue(item.failure_evidence_path))}
            </td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 v0.19 真实撤单审计只读工件");
}

function renderProductionOrderLifecycleAudit(items) {
  document.getElementById("production-order-lifecycle-audit").innerHTML = items.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>节点</th>
          <th>当前结论</th>
          <th>Submit / Approval</th>
          <th>Response / Readback</th>
          <th>Failure / Audit</th>
          <th>只读边界</th>
          <th>工件</th>
        </tr>
      </thead>
      <tbody>
        ${items.map((item) => `
          <tr>
            <td data-label="节点"><strong>${text(item.node_id)}</strong></td>
            <td data-label="当前结论"><span class="status-${safe(item.health)}">${displayText(item.health)}</span><div class="muted">${displayText(snapshotValue(item.readiness_status))}</div><div class="muted">${displayText(snapshotValue(item.audit_state))}</div><div class="muted">${displayText(snapshotValue(item.risk_visibility))}</div><div class="muted">${displayText(snapshotValue(item.diagnostic))}</div>${panelRow("缺失证据", snapshotValue(item.missing_artifacts))}${panelRow("Schema 诊断", snapshotValue(item.schema_diagnostics))}${panelRow("Provenance 诊断", snapshotValue(item.provenance_diagnostics))}${panelRow("Source 诊断", snapshotValue(item.source_diagnostics))}${panelRow("Foundation boundary", snapshotValue(item.foundation_boundary_status))}${panelRow("Boundary diagnostics", snapshotValue(item.foundation_boundary_diagnostics))}${panelRow("Stale 证据", snapshotValue(item.stale_artifacts))}</td>
            <td data-label="Submit / Approval">${panelRow("Lifecycle", snapshotValue(item.lifecycle_id))}${panelRow("Attempt", snapshotValue(item.attempt_id))}${panelRow("Submit 状态", snapshotValue(item.submit_attempt_state))}${panelRow("Submit code", snapshotValue(item.submit_attempt_code))}${panelRow("Owner before", snapshotValue(item.owner_approval_state_before_attempt))}${panelRow("Owner after", snapshotValue(item.owner_approval_state_after_attempt))}${panelRow("Approval consumed", snapshotValue(item.owner_approval_consumed))}${panelRow("Submit attempted", snapshotValue(item.production_submit_attempted))}${panelRow("Readback required", snapshotValue(item.readback_required))}</td>
            <td data-label="Response / Readback">${panelRow("Source class", snapshotValue(item.evidence_source_class))}${panelRow("Adapter runtime", snapshotValue(item.adapter_runtime_integrated))}${panelRow("Foundation only", snapshotValue(item.foundation_only))}${panelRow("Exchange truth", snapshotValue(item.exchange_truth_claimed))}${panelRow("Response 状态", snapshotValue(item.response_state))}${panelRow("Response code", snapshotValue(item.response_code))}${panelRow("Venue status", snapshotValue(item.venue_status))}${panelRow("Order id", snapshotValue(item.venue_order_id))}${panelRow("Client order id", snapshotValue(item.client_order_id))}${panelRow("Readback 状态", snapshotValue(item.readback_state))}${panelRow("Readback code", snapshotValue(item.readback_code))}${panelRow("Mismatch fields", snapshotValue(item.mismatch_fields))}${panelRow("Readback consistent", snapshotValue(item.readback_consistent))}${panelRow("Readback missing", snapshotValue(item.readback_missing))}${panelRow("Readback failed", snapshotValue(item.readback_failed))}</td>
            <td data-label="Failure / Audit">${panelRow("Failure category", snapshotValue(item.failure_category))}${panelRow("Failure code", snapshotValue(item.failure_code))}${panelRow("Next action", snapshotValue(item.next_allowed_action))}${panelRow("No implicit retry", snapshotValue(item.no_implicit_retry))}${panelRow("Unknown visible", snapshotValue(item.unknown_state_visible))}${panelRow("Audit status", snapshotValue(item.audit_closeout_status))}${panelRow("Audit closed", snapshotValue(item.audit_closed))}${panelRow("Dashboard audit", snapshotValue(item.dashboard_audit_consumable))}${panelRow("Release gate", snapshotValue(item.release_gate_consumable))}</td>
            <td data-label="只读边界">${panelRow("Dashboard 下单控件", snapshotValue(item.dashboard_order_controls_enabled))}${panelRow("Dashboard 审批控件", snapshotValue(item.dashboard_approval_controls_enabled))}${panelRow("Dashboard 撤单控件", snapshotValue(item.dashboard_cancel_controls_enabled))}${panelRow("重试", snapshotValue(item.retry_attempted))}${panelRow("Replace", snapshotValue(item.replace_attempted))}${panelRow("Amend", snapshotValue(item.amend_attempted))}${panelRow("Flatten", snapshotValue(item.flatten_attempted))}${panelRow("自动撤单", snapshotValue(item.automatic_cancel_attempted))}${panelRow("自动补救", snapshotValue(item.automatic_remediation_allowed))}${panelRow("策略继续", snapshotValue(item.strategy_continuation_allowed))}</td>
            <td data-label="工件" class="path">
              ${panelRow("submit", snapshotValue(item.submit_candidate_path))}
              ${panelRow("response", snapshotValue(item.response_redaction_path))}
              ${panelRow("readback", snapshotValue(item.readback_reconciliation_path))}
              ${panelRow("failure", snapshotValue(item.failure_no_retry_path))}
              ${panelRow("audit", snapshotValue(item.audit_closeout_path))}
            </td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有 v0.20 生产订单生命周期审计只读工件");
}

function renderDataSources(dataSources) {
  document.getElementById("data-sources").innerHTML = dataSources.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>数据源</th>
          <th>类型</th>
          <th>提供方</th>
          <th>连接</th>
          <th>新鲜度</th>
          <th>延迟</th>
          <th>健康状态</th>
          <th>最近错误</th>
        </tr>
      </thead>
      <tbody>
        ${dataSources.map((source) => `
          <tr>
            <td data-label="数据源"><strong>${text(source.source_id)}</strong></td>
            <td data-label="类型">${displayText(snapshotValue(source.source_kind))}</td>
            <td data-label="提供方">${text(snapshotValue(source.provider))}</td>
            <td data-label="连接">${displayText(source.connection)}</td>
            <td data-label="新鲜度">${displayText(snapshotValue(source.freshness))}</td>
            <td data-label="延迟">${displayText(snapshotValue(source.lag_ms))}</td>
            <td data-label="健康状态"><span class="status-${safe(source.health)}">${displayText(source.health)}</span></td>
            <td data-label="最近错误">${text(dashboardErrorValue(source.last_error))}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有数据源上报");
}

function renderExecutionGateways(gateways) {
  document.getElementById("execution-gateways").innerHTML = gateways.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>网关</th>
          <th>场所</th>
          <th>连接</th>
          <th>已启动</th>
          <th>账户</th>
          <th>订单</th>
          <th>最近上报</th>
          <th>最近错误</th>
        </tr>
      </thead>
      <tbody>
        ${gateways.map((gateway) => `
          <tr>
            <td data-label="网关"><strong>${text(gateway.gateway_id)}</strong></td>
            <td data-label="场所">${text(snapshotValue(gateway.venue))}</td>
            <td data-label="连接">${displayText(gateway.connection)}</td>
            <td data-label="已启动">${displayText(snapshotValue(gateway.started))}</td>
            <td data-label="账户">${text(redactedDashboardValue(gateway.account_ref))}</td>
            <td data-label="订单">未完成 ${text(snapshotValue(gateway.order_counts?.open))} / 处理中 ${text(snapshotValue(gateway.order_counts?.inflight))} / 已关闭 ${text(snapshotValue(gateway.order_counts?.closed))}</td>
            <td data-label="最近上报">${displayText(snapshotValue(gateway.last_report_at))}</td>
            <td data-label="最近错误">${text(dashboardErrorValue(gateway.last_error))}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有执行网关上报");
}

function renderRisk(risk) {
  const lastRejection = risk.last_rejection && risk.last_rejection.value ? "存在（已脱敏）" : displayValue(snapshotValue(risk.last_rejection));
  document.getElementById("risk").innerHTML = [
    renderTile("交易状态", displayValue(risk.trading_state)),
    renderTile("健康状态", displayValue(risk.health), `status-${safe(risk.health)}`),
    renderTile("命令数", displayValue(snapshotValue(risk.command_count))),
    renderTile("事件数", displayValue(snapshotValue(risk.event_count))),
    renderTile("拒绝数", displayValue(snapshotValue(risk.rejections_total))),
    renderTile("最近拒绝", lastRejection),
    renderTile("最近错误", dashboardErrorValue(risk.last_error)),
  ].join("");
}

function renderRuntimeModules(modules) {
  document.getElementById("runtime-modules").innerHTML = modules.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>模块</th>
          <th>状态</th>
          <th>健康状态</th>
          <th>最近可见</th>
          <th>最近错误</th>
          <th>证据</th>
        </tr>
      </thead>
      <tbody>
        ${modules.map((module) => `
          <tr>
            <td data-label="模块"><strong>${text(module.module_name)}</strong></td>
            <td data-label="状态">${displayText(snapshotValue(module.status))}<div class="muted">${displayText(availability(module.status))}</div></td>
            <td data-label="健康状态"><span class="status-${safe(module.health)}">${displayText(module.health)}</span></td>
            <td data-label="最近可见">${displayText(snapshotValue(module.last_seen_at))}</td>
            <td data-label="最近错误">${text(dashboardErrorValue(module.last_error))}</td>
            <td data-label="证据" class="path">${displayText(snapshotValue(module.evidence_source))}<div class="muted">${displayText(availability(module.evidence_source))}</div></td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有运行模块上报");
}

function renderLogsMetrics(logs, metrics) {
  const rows = [
    ...logs.map((log) => ({
      kind: "log",
      id: log.log_id,
      node: snapshotValue(log.node_id),
      path: snapshotValue(log.path),
      availability: log.availability,
      value: snapshotValue(log.last_seen_at),
      lastError: dashboardErrorValue(log.last_error),
    })),
    ...metrics.map((metric) => ({
      kind: "metric",
      id: metric.metric_id,
      node: snapshotValue(metric.node_id),
      path: "指标工件",
      availability: metric.availability,
      value: snapshotValue(metric.value),
      lastError: dashboardErrorValue(metric.last_error),
    })),
  ];
  document.getElementById("logs-metrics").innerHTML = rows.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>类型</th>
          <th>ID</th>
          <th>节点</th>
          <th>证据</th>
          <th>可用性</th>
          <th>值 / 最近可见</th>
          <th>最近错误</th>
        </tr>
      </thead>
      <tbody>
        ${rows.map((row) => `
          <tr>
            <td data-label="类型">${displayText(row.kind)}</td>
            <td data-label="ID"><strong>${text(row.id)}</strong></td>
            <td data-label="节点">${text(row.node)}</td>
            <td data-label="证据" class="path">${text(row.path)}</td>
            <td data-label="可用性">${displayText(row.availability)}</td>
            <td data-label="值 / 最近可见">${displayText(row.value)}</td>
            <td data-label="最近错误">${text(row.lastError)}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("没有日志或指标上报");
}

async function refresh() {
  const snapshot = await loadSnapshot();
  render(snapshot);
}

document.getElementById("refresh").addEventListener("click", () => refresh().catch(console.error));
document.addEventListener("click", async (event) => {
  const button = event.target.closest("[data-dashboard-action]");
  if (!button || button.disabled) return;
  const action = button.getAttribute("data-dashboard-action");
  const nodeId = button.getAttribute("data-node-id");
  button.disabled = true;
  document.getElementById("control-result").innerHTML = `<div class="row">正在对 ${text(nodeId)} 执行 ${text(controlLabel(action))}</div>`;
  try {
    const response = await fetch(`/api/nodes/${encodeURIComponent(nodeId)}/actions/${encodeURIComponent(action)}`, { method: "POST" });
    const payload = await response.json();
    document.getElementById("control-result").innerHTML = `<div class="row"><strong>${displayText(snapshotValue(payload.message))}</strong><span>${displayText(payload.status)} ${displayText(snapshotValue(payload.error_code))}</span></div>`;
    await refresh();
  } catch (error) {
    document.getElementById("control-result").innerHTML = `<div class="row"><strong>控制失败</strong><span>${text(error.message)}</span></div>`;
  }
});
refresh().catch((error) => {
  document.getElementById("overview").innerHTML = renderTile("错误", error.message, "status-error");
});
"#;
