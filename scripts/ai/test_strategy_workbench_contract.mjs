import fs from "node:fs";
import vm from "node:vm";

const source = fs.readFileSync("crates/cli/src/dashboard/strategy_workbench.rs", "utf8");
const embedded = source.match(/pub\(super\) const STRATEGY_WORKBENCH_JS: &str = r#"(.*?)"#;/s);
if (!embedded) throw new Error("embedded strategy workbench JavaScript not found");

class ClassList {
  constructor() { this.values = new Set(); }
  add(value) { this.values.add(value); }
  remove(value) { this.values.delete(value); }
  toggle(value) { this.values.has(value) ? this.values.delete(value) : this.values.add(value); return this.values.has(value); }
}
class Element {
  constructor(dataset = {}) {
    this.innerHTML = "";
    this.textContent = "";
    this.className = "";
    this.disabled = false;
    this.dataset = dataset;
    this.classList = new ClassList();
  }
  addEventListener() {}
  setAttribute() {}
}

const ids = [
  "strategy-workbench", "strategy-name", "strategy-version", "environment", "account-id", "venue-id",
  "run-title", "metric-run", "metric-runtime", "metric-research", "metric-backtest", "metric-health",
  "metric-freshness", "backtest-summary", "demo-summary", "demo-mode-state", "run-table-body", "axis-list",
  "inspector-title", "inspector-kv", "boundary-list", "source-list", "dock-content", "status-data",
  "status-account", "status-venue", "status-node", "status-updated", "connection-banner", "connection-title",
  "connection-detail", "connection-state", "top-health", "rail-dot", "rail-state", "refresh", "drawer-toggle",
  "section-label",
];
const elements = Object.fromEntries(ids.map((id) => [id, new Element()]));
const dockButtons = ["positions", "activity", "fills", "logs"].map((dock) => new Element({ dock }));
const navButtons = ["总览", "策略", "Demo"].map((section) => new Element({ section }));
const modeButtons = ["Backtest", "Demo", "compare"].map((mode) => new Element({ mode }));

const closedBoundaries = {
  external_venue_connection: false,
  order_submission_allowed: false,
  order_mutation_allowed: false,
  automatic_retry_allowed: false,
  automatic_remediation_allowed: false,
  real_orders_submitted: false,
};
const axis = (status) => ({ status, availability: "available", freshness: "fresh", source_refs: ["status.json"], observed_at_unix_ms: 1, reasons: [] });
const value = (entry) => ({ availability: "available", value: entry });
const component = (summary) => ({ status: value("available"), summary: value(summary), freshness_status: value("fresh"), source_ref: value("snapshot.json") });
const basePayload = {
  schema_version: "ntpro.mvp_shared_status_api.response.v1",
  contract_version: "ntpro.mvp_shared_status_api.v1",
  generated_at_unix_ms: 1,
  consumers: ["institution_workbench", "control_center"],
  identity: {
    contract_id: "node-1:btc-ema:instance-1",
    identities: {
      strategy_id: "btc-ema", strategy_version: "v12", backtest_run_id: "bt-1",
      backtest_result_ref: "artifact://backtests/bt-1.json", node_id: "node-1",
      strategy_instance_id: "instance-1", account_id: "acct-1", venue_id: "sandbox", environment: "sandbox",
    },
    boundaries: { read_only_product_contract: true, ...closedBoundaries },
  },
  status: {
    identity_contract_id: "node-1:btc-ema:instance-1",
    research: axis("reference_bound"), runtime: axis("running"), technical_health: axis("healthy"), trading_readiness: axis("blocked"),
    boundaries: { read_only_product_contract: true, ...closedBoundaries },
  },
  business: {
    availability: "available", freshness_status: value("fresh"), source_ref: value("snapshot.json"),
    positions: component("无持仓"), lifecycle: component("运行中"), fills: component("无成交"), diagnostic: value("只读"),
  },
  source_refs: ["identity.json", "status.json"],
  boundaries: { read_only: true, ...closedBoundaries },
};

let payload = structuredClone(basePayload);
const context = vm.createContext({
  document: {
    getElementById: (id) => elements[id],
    querySelectorAll: (selector) => selector === ".dock-tabs button" ? dockButtons : selector === ".nav-item[data-section]" ? navButtons : selector === ".mode-tabs button[data-mode]" ? modeButtons : [],
  },
  fetch: async (url) => {
    if (url !== "/api/mvp/v1/status") throw new Error(`unexpected URL ${url}`);
    return { ok: true, status: 200, json: async () => structuredClone(payload) };
  },
  console,
  Date,
  Error,
});
vm.runInContext(embedded[1], context);
const refresh = () => vm.runInContext("refreshStrategyWorkbench()", context);

await new Promise((resolve) => setImmediate(resolve));
if (elements["connection-title"].textContent !== "策略状态已验证") throw new Error("valid shared status did not render");
if (elements["strategy-name"].textContent !== "btc-ema") throw new Error("strategy identity did not render");
if (!elements["run-table-body"].innerHTML.includes("instance-1")) throw new Error("current run did not render");

const cases = [
  ["missing_strategy", (candidate) => delete candidate.identity.identities.strategy_id],
  ["identity_mismatch", (candidate) => { candidate.status.identity_contract_id = "other"; }],
  ["live_boundary", (candidate) => { candidate.boundaries.real_orders_submitted = true; }],
  ["mutation_boundary", (candidate) => { candidate.identity.boundaries.order_mutation_allowed = true; }],
  ["readiness_drift", (candidate) => { candidate.status.trading_readiness.status = "ready"; }],
  ["environment_drift", (candidate) => { candidate.identity.identities.environment = "live"; }],
  ["missing_source", (candidate) => { candidate.source_refs = []; }],
];
for (const [name, mutate] of cases) {
  payload = structuredClone(basePayload);
  mutate(payload);
  await refresh();
  if (elements["connection-title"].textContent !== "策略工作台已阻断") throw new Error(`${name} did not fail closed`);
  if (elements["strategy-name"].textContent !== "策略未加载") throw new Error(`${name} retained stale strategy identity`);
  if (!elements["run-table-body"].innerHTML.includes("共享状态不可用")) throw new Error(`${name} retained stale run data`);
}

console.log(`strategy_workbench_contract=pass valid=1 fail_closed=${cases.length} stale_clear=${cases.length}`);
