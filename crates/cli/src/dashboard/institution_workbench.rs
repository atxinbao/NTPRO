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

//! 机构工作台静态 shell 与共享只读状态渲染资源。

pub(super) const INSTITUTION_WORKBENCH_HTML: &str = r##"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="icon" href="data:,">
  <title>NTPRO 机构工作台</title>
  <link rel="stylesheet" href="/assets/institution-workbench.css">
</head>
<body>
  <div class="app-shell">
    <aside class="sidebar" aria-label="机构工作台导航">
      <div class="brand"><span class="brand-mark">NT</span><div><strong>NTPRO</strong><span>机构工作台</span></div></div>
      <nav>
        <a class="active" href="#overview">总览</a>
        <a href="#strategy">策略</a>
        <a href="#business">账户与交易</a>
        <a href="#risk">风险</a>
        <a href="#evidence">证据</a>
      </nav>
      <div class="sidebar-state"><span id="sidebar-state-dot" class="state-dot"></span><span id="sidebar-state">等待共享状态</span></div>
    </aside>

    <div class="workspace">
      <header class="context-bar">
        <div class="context-primary">
          <span class="eyebrow">单节点 MVP · 机构只读视图</span>
          <div><strong id="context-strategy">策略未加载</strong><span id="context-scope">账户 / Venue 未加载</span></div>
        </div>
        <button id="refresh" type="button" title="刷新共享状态">刷新</button>
      </header>

      <main>
        <section id="connection-banner" class="connection-banner loading" aria-live="polite">
          <div><strong id="connection-title">正在读取共享状态</strong><span id="connection-detail">等待版本化只读合同</span></div>
          <span id="connection-badge">读取中</span>
        </section>

        <section id="overview" class="section-block">
          <div class="section-heading"><div><span class="eyebrow">共享状态</span><h1>运行与业务总览</h1></div><span id="generated-at" class="section-meta">尚未生成</span></div>
          <div id="axis-grid" class="axis-grid"></div>
        </section>

        <div class="content-grid">
          <div class="primary-column">
            <section id="strategy" class="section-block">
              <div class="section-heading"><div><span class="eyebrow">对象合同</span><h2>策略与运行身份</h2></div></div>
              <div id="identity-grid" class="identity-grid"></div>
            </section>

            <section id="business" class="section-block">
              <div class="section-heading"><div><span class="eyebrow">统一读模型</span><h2>账户与交易状态</h2></div><span id="business-state" class="section-meta">等待数据</span></div>
              <div id="business-grid" class="business-grid"></div>
            </section>

            <section id="risk" class="section-block">
              <div class="section-heading"><div><span class="eyebrow">业务约束</span><h2>风险与阻断原因</h2></div></div>
              <div id="blocking-panel" class="blocking-panel"></div>
            </section>

            <section id="event-correlation" class="section-block">
              <div class="section-heading"><div><span class="eyebrow">跨门户关联</span><h2>业务影响与技术根因</h2></div></div>
              <div id="event-correlation-panel" class="event-correlation-panel"></div>
            </section>
          </div>

          <aside id="evidence" class="evidence-panel" aria-label="当前状态证据">
            <div class="section-heading"><div><span class="eyebrow">Provenance</span><h2>来源与边界</h2></div></div>
            <div id="source-list" class="source-list"></div>
            <div id="boundary-list" class="boundary-list"></div>
          </aside>
        </div>
      </main>

      <footer class="status-bar">
        <span id="footer-environment">环境：未知</span>
        <span id="footer-account">账户：未知</span>
        <span id="footer-venue">Venue：未知</span>
        <span id="footer-readiness">交易准备度：阻断</span>
        <span id="footer-updated">更新时间：未知</span>
      </footer>
    </div>
  </div>
  <script src="/assets/institution-workbench.js"></script>
</body>
</html>
"##;

pub(super) const INSTITUTION_WORKBENCH_CSS: &str = r#":root {
  color-scheme: light;
  font-family: Inter, "Noto Sans CJK SC", "Noto Sans CJK", ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #eef2f0;
  color: #18221e;
}

* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; min-width: 320px; }
button, a { font: inherit; }

.app-shell { display: grid; grid-template-columns: 196px minmax(0, 1fr); min-height: 100vh; }
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
.sidebar nav a { color: #bdc9c3; text-decoration: none; padding: 9px 12px; border-left: 2px solid transparent; }
.sidebar nav a:hover, .sidebar nav a.active { background: #223129; border-left-color: #58c995; color: #ffffff; }
.sidebar-state { margin-top: auto; display: flex; align-items: center; gap: 8px; color: #aebbb5; font-size: 12px; padding: 0 6px; }
.state-dot { width: 8px; height: 8px; border-radius: 50%; background: #d49b3c; flex: 0 0 auto; }
.state-dot.ready { background: #58c995; }
.state-dot.blocked { background: #d45f5f; }

.workspace { min-width: 0; display: grid; grid-template-rows: auto 1fr auto; }
.context-bar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  min-height: 72px;
  padding: 12px 24px;
  background: #fafcfb;
  border-bottom: 1px solid #cfd9d4;
}
.context-primary { display: grid; gap: 4px; min-width: 0; }
.context-primary > div { display: flex; flex-wrap: wrap; align-items: baseline; gap: 10px; }
.context-primary strong { font-size: 17px; overflow-wrap: anywhere; }
.context-primary span:last-child { color: #65726c; font-size: 13px; overflow-wrap: anywhere; }
.eyebrow { color: #16734b; font-size: 11px; font-weight: 800; text-transform: uppercase; }
#refresh { flex: 0 0 auto; border: 1px solid #6d7b74; border-radius: 6px; background: #ffffff; color: #18221e; padding: 8px 14px; font-weight: 700; cursor: pointer; }
#refresh:disabled { cursor: wait; opacity: 0.55; }

main { width: min(1320px, calc(100vw - 228px)); margin: 20px auto 32px; }
.connection-banner { display: flex; justify-content: space-between; align-items: center; gap: 20px; margin-bottom: 18px; padding: 12px 14px; background: #ffffff; border: 1px solid #cbd6d0; border-left: 4px solid #d49b3c; border-radius: 6px; }
.connection-banner > div { display: grid; gap: 3px; }
.connection-banner span { color: #65726c; font-size: 12px; }
.connection-banner > span:last-child { color: #765419; font-weight: 800; }
.connection-banner.ready { border-left-color: #299466; }
.connection-banner.ready > span:last-child { color: #16734b; }
.connection-banner.blocked { border-left-color: #bf4747; }
.connection-banner.blocked > span:last-child { color: #9a3131; }

.section-block { margin-bottom: 22px; }
.section-heading { display: flex; justify-content: space-between; align-items: end; gap: 16px; margin-bottom: 10px; }
.section-heading h1, .section-heading h2 { margin: 3px 0 0; letter-spacing: 0; }
.section-heading h1 { font-size: 23px; }
.section-heading h2 { font-size: 17px; }
.section-meta { color: #65726c; font-size: 12px; text-align: right; overflow-wrap: anywhere; }
.axis-grid, .identity-grid, .business-grid { display: grid; gap: 10px; }
.axis-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); }
.identity-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.business-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.axis-card, .identity-card, .business-card, .blocking-panel, .event-correlation-panel, .evidence-panel { background: #ffffff; border: 1px solid #cfd9d4; border-radius: 6px; }
.axis-card, .identity-card, .business-card { padding: 13px; min-width: 0; }
.card-label { color: #65726c; font-size: 11px; font-weight: 700; }
.card-value { margin-top: 6px; font-size: 17px; font-weight: 800; overflow-wrap: anywhere; }
.card-meta { display: flex; justify-content: space-between; gap: 8px; margin-top: 9px; padding-top: 8px; border-top: 1px solid #e7ece9; color: #65726c; font-size: 11px; }
.card-meta span:last-child { text-align: right; overflow-wrap: anywhere; }
.status-healthy, .status-running, .status-available, .status-reference_bound { color: #16734b; }
.status-degraded, .status-stale, .status-unknown, .status-missing { color: #8a5b14; }
.status-unhealthy, .status-error, .status-identity_mismatch, .status-blocked { color: #9a3131; }

.content-grid { display: grid; grid-template-columns: minmax(0, 1fr) 300px; gap: 18px; align-items: start; }
.evidence-panel { position: sticky; top: 92px; padding: 14px; min-width: 0; }
.source-list, .boundary-list { display: grid; gap: 8px; }
.source-list { margin-bottom: 16px; }
.source-item, .boundary-item { padding: 8px 0; border-bottom: 1px solid #e7ece9; font-size: 12px; overflow-wrap: anywhere; }
.source-item:last-child, .boundary-item:last-child { border-bottom: 0; }
.source-item strong, .boundary-item strong { display: block; margin-bottom: 3px; color: #38463f; }
.boundary-item span { color: #16734b; font-weight: 700; }
.blocking-panel { padding: 14px; min-height: 72px; }
.blocking-panel strong { display: block; margin-bottom: 6px; }
.blocking-panel p { margin: 4px 0; color: #4e5d55; font-size: 13px; overflow-wrap: anywhere; }
.event-correlation-panel { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 16px; align-items: center; padding: 14px; min-height: 86px; }
.event-correlation-copy { display: grid; gap: 5px; min-width: 0; }
.event-correlation-copy strong, .event-correlation-copy span { overflow-wrap: anywhere; }
.event-correlation-copy span { color: #53615a; font-size: 12px; }
.portal-link { color: #0c6541; font-size: 12px; font-weight: 800; text-decoration: none; border-bottom: 1px solid currentColor; white-space: nowrap; }
.portal-link:hover { color: #094d32; }
.empty-state { color: #65726c; font-size: 13px; }
.status-bar { display: flex; flex-wrap: wrap; gap: 8px 18px; padding: 9px 24px; background: #17231e; color: #c8d2cd; font-size: 11px; }

@media (max-width: 960px) {
  .app-shell { grid-template-columns: 1fr; }
  .sidebar { position: static; height: auto; gap: 14px; padding: 12px 14px; }
  .sidebar nav { display: flex; overflow-x: auto; }
  .sidebar nav a { flex: 0 0 auto; border-left: 0; border-bottom: 2px solid transparent; }
  .sidebar nav a:hover, .sidebar nav a.active { border-left-color: transparent; border-bottom-color: #58c995; }
  .sidebar-state { display: none; }
  main { width: min(100%, calc(100vw - 28px)); }
  .content-grid { grid-template-columns: 1fr; }
  .evidence-panel { position: static; }
}
@media (max-width: 680px) {
  .brand span:last-child { display: none; }
  .context-bar { align-items: flex-start; padding: 12px 14px; }
  .context-primary > div { display: grid; gap: 3px; }
  .connection-banner { align-items: flex-start; }
  .axis-grid, .identity-grid, .business-grid { grid-template-columns: 1fr; }
  .section-heading { align-items: flex-start; }
  .section-meta { max-width: 44%; }
  .event-correlation-panel { grid-template-columns: 1fr; }
  .portal-link { justify-self: start; white-space: normal; }
  .status-bar { padding: 9px 14px; }
}
"#;

pub(super) const INSTITUTION_WORKBENCH_JS: &str = r#"const SHARED_STATUS_URL = "/api/mvp/v1/status";
const EVENT_CORRELATION_URL = "/api/mvp/v1/event-correlation";
const EXPECTED_SCHEMA = "ntpro.mvp_shared_status_api.response.v1";
const EXPECTED_CONTRACT = "ntpro.mvp_shared_status_api.v1";
const EXPECTED_EVENT_SCHEMA = "ntpro.mvp_event_correlation_api.response.v1";
const EXPECTED_EVENT_CONTRACT = "ntpro.mvp_event_correlation_api.v1";
const EXPECTED_IDENTITY_SCHEMA = "ntpro.mvp_identity_contract.v1";
const EXPECTED_STATUS_SCHEMA = "ntpro.mvp_status_contract.v1";
const DASHBOARD_AVAILABILITIES = ["available", "not_configured", "not_supported", "stale", "redacted", "unknown"];
const STATUS_AVAILABILITIES = ["available", "missing", "error", "unknown"];
const STATUS_FRESHNESS = ["fresh", "stale", "unknown"];
const BUSINESS_AVAILABILITIES = ["available", "missing", "stale", "error", "identity_mismatch"];
const BUSINESS_HEALTH = ["healthy", "degraded", "error", "stale", "unknown"];

const API_FALSE_BOUNDARIES = [
  "http_success_implies_technical_health", "process_alive_implies_technical_health",
  "backtest_reference_implies_research_accepted", "backtest_complete_implies_trading_readiness",
  "raw_event_store_exposed", "raw_venue_payload_exposed", "external_venue_connection",
  "order_submission_allowed", "order_mutation_allowed", "automatic_retry_allowed",
  "automatic_remediation_allowed", "real_orders_submitted",
];
const CONTRACT_FALSE_BOUNDARIES = [
  "external_venue_connection", "order_submission_allowed", "order_mutation_allowed",
  "automatic_retry_allowed", "automatic_remediation_allowed", "real_orders_submitted",
];
const STATUS_FALSE_BOUNDARIES = [
  "http_success_implies_technical_health", "process_alive_implies_technical_health",
  "backtest_reference_implies_research_accepted", "backtest_complete_implies_trading_readiness",
  ...CONTRACT_FALSE_BOUNDARIES,
];
const DISPLAY_TEXT = {
  reference_bound: "已绑定引用", running: "运行中", stopped: "已停止",
  transitioning: "状态切换中", healthy: "健康", degraded: "降级",
  unhealthy: "不健康", not_running: "未运行", blocked: "阻断",
  available: "可用", missing: "缺失", stale: "陈旧", error: "错误",
  identity_mismatch: "身份不一致", fresh: "新鲜", unknown: "未知", sandbox: "沙盒",
};

const safe = (value) => value === null || value === undefined ? "unknown" : String(value);
const text = (value) => safe(value).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;");
const display = (value) => DISPLAY_TEXT[safe(value)] || safe(value);
const dashboardValue = (value) => !value || typeof value !== "object" ? "unknown" : value.value ?? value.availability ?? "unknown";

function requireObject(value, field) {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`共享状态合同缺少 ${field}`);
  return value;
}
function requireString(value, field) {
  if (typeof value !== "string" || value.trim().length === 0) throw new Error(`共享状态合同缺少 ${field}`);
  return value;
}
function requireOneOf(value, allowed, field) {
  requireString(value, field);
  if (!allowed.includes(value)) throw new Error(`共享状态合同字段异常：${field}`);
  return value;
}
function requirePositiveInteger(value, field) {
  if (!Number.isSafeInteger(value) || value <= 0) throw new Error(`共享状态合同字段异常：${field}`);
  return value;
}
function requireStringArray(value, field, allowEmpty = true) {
  if (!Array.isArray(value) || (!allowEmpty && value.length === 0) || value.some((item) => typeof item !== "string" || item.trim().length === 0)) throw new Error(`共享状态合同缺少 ${field}`);
  return value;
}
function requireDashboardValue(value, field) {
  const dashboardValue = requireObject(value, field);
  requireOneOf(dashboardValue.availability, DASHBOARD_AVAILABILITIES, `${field}.availability`);
  const hasValue = Object.prototype.hasOwnProperty.call(dashboardValue, "value");
  if (dashboardValue.availability === "available") {
    if (!hasValue) throw new Error(`共享状态合同缺少 ${field}.value`);
    requireString(dashboardValue.value, `${field}.value`);
  } else if (hasValue && dashboardValue.value !== null) {
    throw new Error(`共享状态合同字段异常：${field}.value`);
  }
  return dashboardValue;
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
  if ((axis.availability === "error") !== hasError) throw new Error(`共享状态合同错误信封异常：${field}`);
  return axis;
}
function requireBoundary(boundaries, field, expected, scope) {
  if (!Object.prototype.hasOwnProperty.call(boundaries, field) || boundaries[field] !== expected) throw new Error(`${scope} 只读边界异常：${field}`);
}
function validateSharedStatus(payload) {
  requireObject(payload, "payload");
  if (payload.schema_version !== EXPECTED_SCHEMA) throw new Error("共享状态 schema 不匹配");
  if (payload.contract_version !== EXPECTED_CONTRACT) throw new Error("共享状态 contract 不匹配");
  if (!Array.isArray(payload.consumers) || !payload.consumers.includes("institution_workbench")) throw new Error("共享状态未授权机构工作台消费");
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
  for (const field of ["strategy_id", "strategy_version", "backtest_run_id", "backtest_result_ref", "node_id", "strategy_instance_id", "account_id", "venue_id", "environment"]) {
    if (typeof identities[field] !== "string" || identities[field].trim().length === 0) throw new Error(`共享身份缺少 ${field}`);
  }
  if (identities.environment !== "sandbox") throw new Error("机构工作台仅允许 sandbox 环境");
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
  for (const field of ["readiness_status", "snapshot_id", "schema_version", "freshness_status", "source_type", "source_ref", "redaction_state"]) requireDashboardValue(business[field], `business.${field}`);
  for (const component of ["account", "positions", "orders", "fills", "risk", "lifecycle"]) {
    const value = requireObject(business[component], `business.${component}`);
    for (const field of ["status", "summary", "freshness_status", "source_ref", "redaction_state"]) requireDashboardValue(value[field], `business.${component}.${field}`);
  }
  requireDashboardValue(business.blocking_reasons, "business.blocking_reasons");
  requireDashboardValue(business.diagnostic, "business.diagnostic");
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

const emptyCard = (label) => `<div class="identity-card"><div class="card-label">${text(label)}</div><div class="card-value">等待共享状态</div></div>`;
function resetSurface(message) {
  document.getElementById("axis-grid").innerHTML = ["研究状态", "运行状态", "技术健康", "交易准备度"].map(emptyCard).join("");
  document.getElementById("identity-grid").innerHTML = ["策略", "回测", "运行实例", "账户", "Venue", "环境"].map(emptyCard).join("");
  document.getElementById("business-grid").innerHTML = ["账户", "持仓", "订单", "成交", "风险", "生命周期"].map(emptyCard).join("");
  document.getElementById("blocking-panel").innerHTML = `<span class="empty-state">${text(message)}</span>`;
  document.getElementById("event-correlation-panel").innerHTML = `<span class="empty-state">${text(message)}</span>`;
  document.getElementById("source-list").innerHTML = `<div class="source-item"><strong>来源</strong>等待已验证合同</div>`;
  document.getElementById("boundary-list").innerHTML = `<div class="boundary-item"><strong>只读边界</strong><span>未验证，保持阻断</span></div>`;
  document.getElementById("context-strategy").textContent = "策略未加载";
  document.getElementById("context-scope").textContent = "账户 / Venue 未加载";
  document.getElementById("generated-at").textContent = "尚未生成";
  document.getElementById("business-state").textContent = "等待数据";
  document.getElementById("footer-environment").textContent = "环境：未知";
  document.getElementById("footer-account").textContent = "账户：未知";
  document.getElementById("footer-venue").textContent = "Venue：未知";
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
function identityCard(label, value, meta) {
  return `<article class="identity-card"><div class="card-label">${text(label)}</div><div class="card-value">${text(value)}</div><div class="card-meta"><span>${text(meta)}</span></div></article>`;
}
function businessCard(label, component) {
  const status = dashboardValue(component.status);
  const summary = dashboardValue(component.summary);
  const freshness = dashboardValue(component.freshness_status);
  const source = dashboardValue(component.source_ref);
  return `<article class="business-card"><div class="card-label">${text(label)}</div><div class="card-value status-${text(status)}">${text(display(summary))}</div><div class="card-meta"><span>${text(display(status))} / ${text(display(freshness))}</span><span>${text(source)}</span></div></article>`;
}
function renderSharedStatus(payload, correlation) {
  const identities = payload.identity.identities;
  const status = payload.status;
  const business = payload.business;
  document.getElementById("context-strategy").textContent = `${identities.strategy_id} · ${identities.strategy_version}`;
  document.getElementById("context-scope").textContent = `${identities.account_id} / ${identities.venue_id} / ${identities.environment}`;
  document.getElementById("generated-at").textContent = `合同时间 ${payload.generated_at_unix_ms}`;
  document.getElementById("axis-grid").innerHTML = [axisCard("研究状态", status.research), axisCard("运行状态", status.runtime), axisCard("技术健康", status.technical_health), axisCard("交易准备度", status.trading_readiness)].join("");
  document.getElementById("identity-grid").innerHTML = [
    identityCard("策略", identities.strategy_id, identities.strategy_version),
    identityCard("回测", identities.backtest_run_id, "仅绑定引用，不代表研究验收"),
    identityCard("运行实例", identities.strategy_instance_id, `Supervisor ${identities.node_id}`),
    identityCard("账户", identities.account_id, "机构业务范围"),
    identityCard("Venue", identities.venue_id, "沙盒身份"),
    identityCard("环境", display(identities.environment), "只读产品合同"),
  ].join("");
  document.getElementById("business-state").textContent = `${display(business.availability)} / ${display(business.health)}`;
  document.getElementById("business-grid").innerHTML = [businessCard("账户", business.account), businessCard("持仓", business.positions), businessCard("订单", business.orders), businessCard("成交", business.fills), businessCard("风险", business.risk), businessCard("生命周期", business.lifecycle)].join("");
  document.getElementById("blocking-panel").innerHTML = `<strong>交易准备度保持阻断</strong><p>${text(display(dashboardValue(business.blocking_reasons)))}</p><p>${text(display(dashboardValue(business.diagnostic)))}</p>`;
  const event = correlation.event;
  document.getElementById("event-correlation-panel").innerHTML = `<div class="event-correlation-copy"><strong>${text(event.event_id)}</strong><span>业务影响：${text(display(business.health))} / ${text(display(business.availability))}</span><span>技术根因：${text(display(status.technical_health.status))} · 节点 ${text(event.node_id)}</span></div><a class="portal-link" href="${text(portalEventLink(correlation.links.control_center_path, event.event_id))}">在控制中心查看技术根因</a>`;
  document.getElementById("source-list").innerHTML = (payload.source_refs || []).map((source, index) => `<div class="source-item"><strong>来源 ${index + 1}</strong>${text(source)}</div>`).join("") || `<div class="source-item"><strong>来源</strong>未提供</div>`;
  document.getElementById("boundary-list").innerHTML = [["共享合同", payload.contract_version], ["只读", "已验证"], ["外部 Venue", "关闭"], ["订单提交与变更", "关闭"], ["自动重试与补救", "关闭"], ["真实订单", "关闭"]].map(([label, value]) => `<div class="boundary-item"><strong>${text(label)}</strong><span>${text(value)}</span></div>`).join("");
  document.getElementById("footer-environment").textContent = `环境：${display(identities.environment)}`;
  document.getElementById("footer-account").textContent = `账户：${identities.account_id}`;
  document.getElementById("footer-venue").textContent = `Venue：${identities.venue_id}`;
  document.getElementById("footer-readiness").textContent = `交易准备度：${display(status.trading_readiness.status)}`;
  document.getElementById("footer-updated").textContent = `更新时间：${payload.generated_at_unix_ms}`;
  setConnection("ready", "共享状态已验证", "机构工作台正在消费版本化只读投影", "只读");
}
function renderBlocked(error) {
  resetSurface("共享状态不可用，旧数据已清空");
  setConnection("blocked", "机构工作台已阻断", error.message, "Fail closed");
}
async function refreshInstitutionWorkbench() {
  const button = document.getElementById("refresh");
  button.disabled = true;
  resetSurface("刷新中，旧数据已清空");
  setConnection("loading", "正在读取共享状态", "等待版本化只读合同", "读取中");
  try {
    const options = { method: "GET", headers: { "Accept": "application/json" }, cache: "no-store" };
    const [statusResponse, correlationResponse] = await Promise.all([fetch(SHARED_STATUS_URL, options), fetch(EVENT_CORRELATION_URL, options)]);
    if (!statusResponse.ok) throw new Error(`共享状态不可用（HTTP ${statusResponse.status}）`);
    if (!correlationResponse.ok) throw new Error(`事件关联不可用（HTTP ${correlationResponse.status}）`);
    const shared = validateSharedStatus(await statusResponse.json());
    renderSharedStatus(shared, validateEventCorrelation(await correlationResponse.json(), shared));
  } catch (error) {
    renderBlocked(error instanceof Error ? error : new Error("共享状态读取失败"));
  } finally {
    button.disabled = false;
  }
}

document.getElementById("refresh").addEventListener("click", refreshInstitutionWorkbench);
refreshInstitutionWorkbench();
"#;
