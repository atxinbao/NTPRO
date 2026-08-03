import fs from "node:fs";
import vm from "node:vm";

const source = fs.readFileSync("crates/cli/src/dashboard/institution_workbench.rs", "utf8");
const embedded = source.match(
  /pub\(super\) const INSTITUTION_WORKBENCH_JS: &str = r#"(.*?)"#;/s,
);
if (!embedded) throw new Error("embedded institution workbench JavaScript not found");

const ids = [
  "axis-grid", "identity-grid", "business-grid", "blocking-panel", "source-list",
  "boundary-list", "context-strategy", "context-scope", "generated-at", "business-state",
  "footer-environment", "footer-account", "footer-venue", "footer-readiness", "footer-updated",
  "connection-banner", "connection-title", "connection-detail", "connection-badge", "sidebar-state",
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
const basePayload = {
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
  source_refs: ["identity.json", "status.json"],
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

let responsePayload = structuredClone(basePayload);
const context = vm.createContext({
  document: { getElementById: (id) => elements[id] },
  fetch: async () => ({
    ok: true,
    status: 200,
    json: async () => structuredClone(responsePayload),
  }),
  console,
  Error,
});
vm.runInContext(embedded[1], context);
const refresh = () => vm.runInContext("refreshInstitutionWorkbench()", context);

await new Promise((resolve) => setImmediate(resolve));
if (elements["connection-title"].textContent !== "共享状态已验证") {
  throw new Error("valid shared status contract did not render");
}

const cases = [
  ["missing_business_readiness", (value) => delete value.business.readiness_status],
  ["invalid_runtime_enum", (value) => { value.status.runtime.status = "arbitrary"; }],
  ["available_without_value", (value) => delete value.business.orders.status.value],
  ["identity_contract_mismatch", (value) => { value.status.identity_contract_id = "other"; }],
  ["mutation_boundary_true", (value) => { value.boundaries.order_submission_allowed = true; }],
  ["error_availability_without_error", (value) => { value.status.runtime.availability = "error"; }],
  ["error_with_non_error_availability", (value) => { value.status.runtime.error = "runtime failed"; }],
  ["healthy_axis_not_fresh", (value) => { value.status.technical_health.freshness = "stale"; }],
  ["healthy_axis_not_available", (value) => { value.status.technical_health.availability = "missing"; }],
];

for (const [name, mutate] of cases) {
  responsePayload = structuredClone(basePayload);
  mutate(responsePayload);
  await refresh();
  if (elements["connection-title"].textContent !== "机构工作台已阻断") {
    throw new Error(`${name} did not fail closed`);
  }
  if (elements["context-strategy"].textContent !== "策略未加载") {
    throw new Error(`${name} retained stale identity`);
  }
  if (!elements["business-grid"].innerHTML.includes("等待共享状态")) {
    throw new Error(`${name} retained stale business data`);
  }
}

console.log(
  `institution_workbench_contract=pass valid=1 fail_closed=${cases.length} stale_clear=${cases.length}`,
);
