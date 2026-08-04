import fs from "node:fs";
import vm from "node:vm";

const source = fs.readFileSync("crates/cli/src/dashboard/control_center.rs", "utf8");
const embedded = source.match(/pub\(super\) const CONTROL_CENTER_JS: &str = r#"(.*?)"#;/s);
if (!embedded) throw new Error("embedded control center JavaScript not found");

const ids = [
  "axis-grid", "business-impact-title", "business-impact-list", "node-grid", "lifecycle-action-buttons", "lifecycle-action-result", "component-table", "observability-grid", "alert-list",
  "event-correlation-panel",
  "source-list", "boundary-list", "context-node", "context-scope", "generated-at",
  "node-health", "alert-count", "footer-environment", "footer-node", "footer-runtime",
  "footer-health", "footer-readiness", "footer-updated", "connection-banner",
  "connection-title", "connection-detail", "connection-badge", "sidebar-state",
  "sidebar-state-dot", "refresh",
];
class Element {
  constructor() {
    this.innerHTML = "";
    this.textContent = "";
    this.className = "";
    this.disabled = false;
  }

  addEventListener() {}
}

const elements = Object.fromEntries(ids.map((id) => [id, new Element()]));
const closedBoundaries = {
  external_venue_connection: false,
  order_submission_allowed: false,
  order_mutation_allowed: false,
  automatic_retry_allowed: false,
  automatic_remediation_allowed: false,
  real_orders_submitted: false,
};
const axis = (status) => ({
  status,
  availability: "available",
  freshness: "fresh",
  source_refs: ["status.json"],
  observed_at_unix_ms: 1,
  reasons: [],
});
const dashboardValue = (value) => ({ availability: "available", value });
const component = (summary) => ({
  status: dashboardValue("available"),
  summary: dashboardValue(summary),
  freshness_status: dashboardValue("fresh"),
  source_ref: dashboardValue("snapshot.json"),
  redaction_state: dashboardValue("redacted"),
});
const baseShared = {
  schema_version: "ntpro.mvp_shared_status_api.response.v1",
  contract_version: "ntpro.mvp_shared_status_api.v1",
  generated_at_unix_ms: 1,
  consumers: ["institution_workbench", "control_center"],
  identity: {
    schema_version: "ntpro.mvp_identity_contract.v1",
    contract_id: "node-1:btc-ema:instance-1",
    identities: {
      strategy_id: "btc-ema",
      strategy_version: "v12",
      backtest_run_id: "bt-1",
      backtest_result_ref: "artifact://backtests/bt-1.json",
      node_id: "node-1",
      strategy_instance_id: "instance-1",
      account_id: "acct-1",
      venue_id: "sandbox",
      environment: "sandbox",
    },
    provenance: { config_path: "config.toml", generated_at_unix_ms: 1 },
    boundaries: { read_only_product_contract: true, ...closedBoundaries },
  },
  status: {
    schema_version: "ntpro.mvp_status_contract.v1",
    identity_contract_id: "node-1:btc-ema:instance-1",
    research: axis("reference_bound"),
    runtime: axis("running"),
    technical_health: axis("healthy"),
    trading_readiness: axis("blocked"),
    provenance: {
      identity_contract_path: "identity.json",
      identity_contract_available: true,
      supervisor_registry_path: "registry.json",
      node_status_path: "status.json",
      node_metrics_path: "metrics.json",
      unified_read_model_path: "snapshot.json",
      freshness_max_age_ms: 2_000,
      generated_at_unix_ms: 1,
    },
    boundaries: {
      read_only_product_contract: true,
      http_success_implies_technical_health: false,
      process_alive_implies_technical_health: false,
      backtest_reference_implies_research_accepted: false,
      backtest_complete_implies_trading_readiness: false,
      ...closedBoundaries,
    },
  },
  business: {
    availability: "available",
    health: "healthy",
    readiness_status: dashboardValue("ready_readonly_artifact"),
    snapshot_id: dashboardValue("snapshot-1"),
    schema_version: dashboardValue("read-model-v1"),
    freshness_status: dashboardValue("fresh"),
    source_type: dashboardValue("artifact"),
    source_ref: dashboardValue("snapshot.json"),
    redaction_state: dashboardValue("redacted"),
    account: component("账户正常"),
    positions: component("无持仓"),
    orders: component("无订单"),
    fills: component("无成交"),
    risk: component("风险阻断"),
    lifecycle: component("运行中"),
    blocking_reasons: dashboardValue("交易能力未授权"),
    diagnostic: dashboardValue("只读"),
  },
  source_refs: ["identity.json", "status.json", "registry.json"],
  boundaries: {
    read_only: true,
    http_success_implies_technical_health: false,
    process_alive_implies_technical_health: false,
    backtest_reference_implies_research_accepted: false,
    backtest_complete_implies_trading_readiness: false,
    raw_event_store_exposed: false,
    raw_venue_payload_exposed: false,
    ...closedBoundaries,
  },
};
const baseSnapshot = {
  schema_version: "ntpro.mvp_control_center_snapshot.v2",
  generated_at: dashboardValue("unix_seconds:1"),
  registry_path: "registry.json",
  local_only: true,
  overview: {
    node_count: 1,
    running_nodes: 1,
    stopped_nodes: 0,
    error_nodes: 0,
    unknown_nodes: 0,
    health: "healthy",
    sandbox_only: true,
    latest_transition_at: dashboardValue("unix_ms:1"),
    latest_error_present: false,
  },
  node: {
    node_id: "node-1",
    lifecycle_state: "running",
    process_mode: "spawned_process",
    process_state: "running",
    pid: dashboardValue(123),
    health: "healthy",
    last_transition_at: dashboardValue("unix_ms:1"),
    error_present: false,
  },
  data_sources: [{
    source_id: "node-1:data",
    source_kind: dashboardValue("local"),
    provider: dashboardValue("sandbox"),
    connection: "connected",
    freshness: dashboardValue("fresh"),
    lag_ms: dashboardValue(0),
    health: "healthy",
    error_present: false,
  }],
  execution_gateways: [{
    gateway_id: "node-1:execution",
    venue: dashboardValue("sandbox"),
    connection: "not_configured",
    started: { availability: "not_configured" },
    last_report_at: { availability: "unknown" },
    error_present: false,
  }],
  runtime_modules: [{
    module_name: "Supervisor",
    status: dashboardValue("running"),
    health: "healthy",
    last_seen_at: dashboardValue("unix_ms:1"),
    evidence_source: dashboardValue("status.json"),
    error_present: false,
  }],
  logs: [{
    log_id: "node-1:stdout",
    availability: "available",
    last_seen_at: dashboardValue("unix_ms:1"),
    error_present: false,
  }],
  metrics: [{
    metric_id: "node-1:uptime_ms",
    value: dashboardValue("1000"),
    availability: "available",
    last_seen_at: dashboardValue("unix_ms:1"),
    error_present: false,
  }],
  alerts: [{ alert_id: "gap-0", severity: "warning", source: "runtime.cache" }],
  gaps: [{
    field_path: "runtime.cache",
    reason: "not_supported",
    owner_task: dashboardValue("MVP-007"),
  }],
  lifecycle_actions: [
    { action: "start", target_node_id: "node-1", method: "POST", enabled: false, reason_code: "requires_stopped" },
    { action: "stop", target_node_id: "node-1", method: "POST", enabled: true, reason_code: "ready" },
  ],
  boundaries: {
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
};

const lifecycleActionBoundaries = {
  supervisor_lifecycle_action: true,
  external_venue_connection: false,
  production_venue_connection: false,
  external_network_attempted: false,
  order_submission_allowed: false,
  order_mutation_allowed: false,
  automatic_retry_allowed: false,
  automatic_remediation_allowed: false,
  real_orders_submitted: false,
};
const baseLifecycleAction = {
  schema_version: "ntpro.mvp_control_center_lifecycle_action.response.v1",
  contract_version: "ntpro.mvp_control_center_lifecycle_action.v1",
  local_only: true,
  target_node_id: "node-1",
  action_name: "stop",
  result: {
    action_id: "stop:node-1:unix_ms:2",
    action: "stop:node-1",
    status: "succeeded",
    previous_state: "running",
    current_state: "stopped",
    started_at: dashboardValue("unix_ms:2"),
    finished_at: dashboardValue("unix_ms:3"),
    error_code: { availability: "unknown" },
    message: dashboardValue("已通过本地监督器完成停止"),
    observability_ref: dashboardValue("registry:node-1"),
  },
  boundaries: lifecycleActionBoundaries,
};

const eventId = "mvp-status:v1:node-1:btc-ema:instance-1:technical-health";
const baseCorrelation = {
  schema_version: "ntpro.mvp_event_correlation_api.response.v1",
  contract_version: "ntpro.mvp_event_correlation_api.v1",
  event: {
    event_id: eventId,
    event_kind: "technical_health_observation",
    event_source: "projected_status_contract",
    identity_contract_id: "node-1:btc-ema:instance-1",
    node_id: "node-1",
    strategy_instance_id: "instance-1",
  },
  links: {
    institution_workbench_path: "/institution-workbench",
    control_center_path: "/control-center",
  },
  boundaries: {
    read_only: true,
    projected_status_event: true,
    raw_event_store_exposed: false,
    raw_event_payload_exposed: false,
    raw_errors_exposed: false,
    supervisor_actions_exposed: false,
    trading_controls_exposed: false,
  },
};

let shared = structuredClone(baseShared);
let snapshot = structuredClone(baseSnapshot);
let correlation = structuredClone(baseCorrelation);
let lifecycleAction = structuredClone(baseLifecycleAction);
let statuses = { shared: 200, snapshot: 200, correlation: 200, action: 200 };
const browserLocation = { search: "" };
const response = (status, value) => ({
  ok: status >= 200 && status < 300,
  status,
  json: async () => structuredClone(value),
});
const context = vm.createContext({
  document: { getElementById: (id) => elements[id], addEventListener: () => {} },
  fetch: async (url, options = {}) => {
    if (url === "/api/mvp/v1/status") return response(statuses.shared, shared);
    if (url === "/api/mvp/v1/control-center") return response(statuses.snapshot, snapshot);
    if (url === "/api/mvp/v1/event-correlation") return response(statuses.correlation, correlation);
    if (url === "/api/mvp/v1/control-center/nodes/node-1/actions/stop" && options.method === "POST") {
      if (statuses.action === 200) {
        shared.status.runtime.status = "stopped";
        snapshot.node.lifecycle_state = "stopped";
        snapshot.node.process_state = "stopped";
        snapshot.overview.running_nodes = 0;
        snapshot.overview.stopped_nodes = 1;
        snapshot.lifecycle_actions[0] = { action: "start", target_node_id: "node-1", method: "POST", enabled: true, reason_code: "ready" };
        snapshot.lifecycle_actions[1] = { action: "stop", target_node_id: "node-1", method: "POST", enabled: false, reason_code: "requires_running_or_paused" };
      }
      return response(statuses.action, lifecycleAction);
    }
    throw new Error(`unexpected URL ${url}`);
  },
  location: browserLocation,
  URLSearchParams,
  console,
  Error,
});
vm.runInContext(embedded[1], context);
const refresh = () => vm.runInContext("refreshControlCenter()", context);

await new Promise((resolve) => setImmediate(resolve));
if (elements["connection-title"].textContent !== "共享与运维状态已对齐") {
  throw new Error("valid control center contract did not render");
}
if (elements["context-node"].textContent !== "node-1") {
  throw new Error("valid contract did not render the correlated node");
}
if (!elements["business-impact-list"].innerHTML.includes("风险：可用")) {
  throw new Error("valid contract did not render shared business impact");
}
if (!elements["event-correlation-panel"].innerHTML.includes("在机构工作台查看业务影响") || !elements["event-correlation-panel"].innerHTML.includes(encodeURIComponent(eventId))) {
  throw new Error("valid event correlation did not render the institution workbench jump");
}
if (!elements["lifecycle-action-buttons"].innerHTML.includes("data-lifecycle-action=\"stop\"") || !elements["lifecycle-action-buttons"].innerHTML.includes("data-lifecycle-action=\"start\"")) {
  throw new Error("valid lifecycle capability did not render start and stop controls");
}
await vm.runInContext('executeLifecycleAction("stop", "node-1")', context);
if (!elements["lifecycle-action-result"].innerHTML.includes("running → stopped")) {
  throw new Error("valid lifecycle action response did not render the state transition");
}
if (!elements["lifecycle-action-buttons"].innerHTML.includes("data-lifecycle-action=\"start\"")) {
  throw new Error("post-action refresh did not render the new start capability");
}

shared = structuredClone(baseShared);
snapshot = structuredClone(baseSnapshot);
correlation = structuredClone(baseCorrelation);
if (await vm.runInContext('refreshControlCenter("stopped")', context)) {
  throw new Error("post-action refresh accepted an aligned but incorrect final lifecycle state");
}
if (!elements["lifecycle-action-buttons"].innerHTML.includes("操作不可用")) {
  throw new Error("incorrect post-action final lifecycle state did not clear action controls");
}

const cases = [
  ["missing_consumer", () => { shared.consumers = ["institution_workbench"]; }],
  ["shared_boundary_true", () => { shared.boundaries.order_submission_allowed = true; }],
  ["ops_schema_mismatch", () => { snapshot.schema_version = "other"; }],
  ["ops_node_mismatch", () => { snapshot.node.node_id = "node-2"; }],
  ["registry_source_mismatch", () => { snapshot.registry_path = "other-registry.json"; }],
  ["registry_provenance_mismatch", () => { shared.status.provenance.supervisor_registry_path = "other-registry.json"; }],
  ["lifecycle_mismatch", () => { snapshot.node.lifecycle_state = "stopped"; }],
  ["process_state_mismatch", () => { snapshot.node.process_state = "stopped"; }],
  ["transitioning_lifecycle_mismatch", () => { shared.status.runtime.status = "transitioning"; }],
  ["stopped_lifecycle_mismatch", () => { shared.status.runtime.status = "stopped"; snapshot.node.process_state = "stopped"; }],
  ["invalid_lifecycle", () => { snapshot.node.lifecycle_state = "arbitrary"; }],
  ["invalid_process_state", () => { snapshot.node.process_state = "arbitrary"; }],
  ["missing_node_pid", () => { delete snapshot.node.pid; }],
  ["invalid_node_pid_type", () => { snapshot.node.pid.value = "123"; }],
  ["invalid_data_source_connection", () => { snapshot.data_sources[0].connection = "arbitrary"; }],
  ["invalid_data_source_lag", () => { snapshot.data_sources[0].lag_ms.value = -1; }],
  ["redacted_value_present", () => { snapshot.data_sources[0].provider = { availability: "redacted", value: "credential=secret" }; }],
  ["invalid_gateway_started", () => { snapshot.execution_gateways[0].started = dashboardValue("true"); }],
  ["missing_runtime_status", () => { delete snapshot.runtime_modules[0].status; }],
  ["missing_log_timestamp", () => { delete snapshot.logs[0].last_seen_at; }],
  ["invalid_metric_error_flag", () => { snapshot.metrics[0].error_present = "false"; }],
  ["overview_count_mismatch", () => { snapshot.overview.running_nodes = 0; }],
  ["ops_action_boundary_false", () => { snapshot.boundaries.supervisor_actions_exposed = false; }],
  ["ops_unsupported_action_boundary_true", () => { snapshot.boundaries.unsupported_supervisor_actions_exposed = true; }],
  ["ops_trading_boundary_true", () => { snapshot.boundaries.trading_controls_exposed = true; }],
  ["missing_lifecycle_actions", () => { snapshot.lifecycle_actions = []; }],
  ["duplicate_lifecycle_action", () => { snapshot.lifecycle_actions[1] = structuredClone(snapshot.lifecycle_actions[0]); }],
  ["unsupported_lifecycle_action", () => { snapshot.lifecycle_actions[1].action = "pause"; }],
  ["lifecycle_action_target_mismatch", () => { snapshot.lifecycle_actions[1].target_node_id = "node-2"; }],
  ["lifecycle_action_method_mismatch", () => { snapshot.lifecycle_actions[1].method = "GET"; }],
  ["lifecycle_action_enabled_mismatch", () => { snapshot.lifecycle_actions[0].enabled = true; snapshot.lifecycle_actions[0].reason_code = "ready"; }],
  ["lifecycle_action_reason_mismatch", () => { snapshot.lifecycle_actions[1].reason_code = "requires_running_or_paused"; }],
  ["raw_node_error", () => { snapshot.node.last_error = "credential=secret"; }],
  ["raw_alert_message", () => { snapshot.alerts[0].message = "raw error"; }],
  ["raw_gap_notes", () => { snapshot.gaps[0].notes = dashboardValue("raw error"); }],
  ["full_snapshot_controls", () => { snapshot.controls = []; }],
  ["shared_http_error", () => { statuses.shared = 503; }],
  ["snapshot_http_error", () => { statuses.snapshot = 503; }],
  ["event_http_error", () => { statuses.correlation = 503; }],
  ["event_identity_mismatch", () => { correlation.event.identity_contract_id = "other"; }],
  ["event_node_mismatch", () => { correlation.event.node_id = "node-2"; }],
  ["event_kind_mismatch", () => { correlation.event.event_kind = "raw_event"; }],
  ["event_target_drift", () => { correlation.links.institution_workbench_path = "/dashboard"; }],
  ["event_boundary_true", () => { correlation.boundaries.raw_event_payload_exposed = true; }],
  ["event_raw_field", () => { correlation.event.message = "raw error"; }],
  ["requested_event_mismatch", () => { browserLocation.search = "?event_id=forged"; }],
  ["duplicate_event_valid_first", () => { browserLocation.search = `?event_id=${encodeURIComponent(eventId)}&event_id=forged`; }],
  ["duplicate_event_valid_last", () => { browserLocation.search = `?event_id=forged&event_id=${encodeURIComponent(eventId)}`; }],
];

for (const [name, mutate] of cases) {
  shared = structuredClone(baseShared);
  snapshot = structuredClone(baseSnapshot);
  correlation = structuredClone(baseCorrelation);
  lifecycleAction = structuredClone(baseLifecycleAction);
  statuses = { shared: 200, snapshot: 200, correlation: 200, action: 200 };
  browserLocation.search = "";
  mutate();
  await refresh();
  if (elements["connection-title"].textContent !== "控制中心已阻断") {
    throw new Error(`${name} did not fail closed`);
  }
  if (elements["context-node"].textContent !== "节点未加载") {
    throw new Error(`${name} retained stale node identity`);
  }
  if (!elements["component-table"].innerHTML.includes("旧数据已清空")) {
    throw new Error(`${name} retained stale operational data`);
  }
  if (elements["business-impact-title"].textContent !== "等待共享状态") {
    throw new Error(`${name} retained stale business impact`);
  }
}

const actionCases = [
  ["action_schema_mismatch", (value) => { value.schema_version = "other"; }],
  ["action_contract_mismatch", (value) => { value.contract_version = "other"; }],
  ["action_not_local", (value) => { value.local_only = false; }],
  ["action_node_mismatch", (value) => { value.target_node_id = "node-2"; }],
  ["action_name_mismatch", (value) => { value.action_name = "start"; }],
  ["action_result_target_mismatch", (value) => { value.result.action = "stop:node-2"; }],
  ["action_status_invalid", (value) => { value.result.status = "complete"; }],
  ["action_state_invalid", (value) => { value.result.current_state = "arbitrary"; }],
  ["action_boundary_false", (value) => { value.boundaries.supervisor_lifecycle_action = false; }],
  ["action_trading_boundary_true", (value) => { value.boundaries.order_submission_allowed = true; }],
  ["action_raw_error", (value) => { value.raw_error = "credential=secret"; }],
];

for (const [name, mutate] of actionCases) {
  const value = structuredClone(baseLifecycleAction);
  mutate(value);
  context.actionEnvelopeUnderTest = value;
  let rejected = false;
  try {
    vm.runInContext('validateLifecycleActionEnvelope(actionEnvelopeUnderTest, "node-1", "stop")', context);
  } catch {
    rejected = true;
  }
  if (!rejected) throw new Error(`${name} lifecycle action envelope did not fail closed`);
}

console.log(`control_center_contract=pass valid=1 lifecycle_action=1 post_action_target_mismatch=1 fail_closed=${cases.length + actionCases.length + 1} stale_clear=${cases.length + 1}`);
