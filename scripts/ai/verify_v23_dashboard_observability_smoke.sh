#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SNAPSHOT_PATH="${NTPRO_V23_DASHBOARD_OBSERVABILITY_SNAPSHOT:-tests/golden/v230/dashboard_observability_snapshot.json}"

if [[ ! -f "$SNAPSHOT_PATH" ]]; then
  echo "missing dashboard observability fixture: $SNAPSHOT_PATH" >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "node is required for dashboard observability smoke" >&2
  exit 1
fi

node - "$SNAPSHOT_PATH" <<'NODE'
const fs = require("node:fs");
const vm = require("node:vm");

const snapshotPath = process.argv[2];
const snapshot = JSON.parse(fs.readFileSync(snapshotPath, "utf8"));
const dashboardSource = fs.readFileSync("crates/cli/src/dashboard.rs", "utf8");
const jsMatch = dashboardSource.match(/const DASHBOARD_JS: &str = r#"(.*?)"#;/s);
if (!jsMatch) {
  throw new Error("DASHBOARD_JS raw string not found");
}
const dashboardJs = jsMatch[1];
const refreshStart = dashboardJs.indexOf("\nasync function refresh()");
if (refreshStart < 0) {
  throw new Error("dashboard refresh boundary not found");
}
const renderOnlyDashboardJs = dashboardJs.slice(0, refreshStart);

const runtimes = snapshot.read_model_runtime || [];
if (runtimes.length !== 2) {
  throw new Error(`fixture must contain two scoped read_model_runtime rows, got ${runtimes.length}`);
}

const requiredFalseFields = [
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
  "production_order_submission_allowed",
  "production_order_mutation_allowed",
  "product_grade_trading_terminal_claim",
];

for (const [index, runtime] of runtimes.entries()) {
  for (const field of requiredFalseFields) {
    const value = runtime[field];
    if (!value || value.availability !== "available" || value.value !== false) {
      throw new Error(`${field} must be explicitly available false for row ${index} in ${snapshotPath}`);
    }
  }
}

const elements = {};
const document = {
  getElementById(id) {
    if (!elements[id]) {
      elements[id] = { innerHTML: "" };
    }
    return elements[id];
  },
};
const context = { console, document, __snapshot: snapshot };
vm.createContext(context);
vm.runInContext(
  `${renderOnlyDashboardJs}\nrenderTraderTerminalWorkbench(__snapshot.read_model_runtime || []);\nrenderReadModelRuntime(__snapshot.read_model_runtime || []);`,
  context,
);

const workbenchHtml = elements["trader-terminal-workbench"]?.innerHTML || "";
const runtimeHtml = elements["read-model-runtime"]?.innerHTML || "";
const html = `${workbenchHtml}\n${runtimeHtml}`;
if (!workbenchHtml || !runtimeHtml) {
  throw new Error("dashboard observability render produced empty HTML");
}

const requiredHtmlMarkers = [
  "terminal-v230-acct001-strategy-alpha-venue-binance",
  "terminal-v230-acct002-strategy-beta-venue-coinbase",
  "acct-redacted-001 / strategy-redacted-alpha / venue-node-binance-a",
  "acct-redacted-002 / strategy-redacted-beta / venue-node-coinbase-a",
  "scope acct-redacted-001|strategy:strategy-redacted-alpha|venue:venue-node-binance-a",
  "scope acct-redacted-002|strategy:strategy-redacted-beta|venue:venue-node-coinbase-a",
  "dashboard_observability_v230",
  "dashboard_filter_scope_isolated",
  "read_model_runtime",
  "locked_readonly",
  "所有操作控件禁用",
  "2 个节点",
  "tests/golden/read_model_dashboard_observability_schema.jsonl",
  "ntpro-rust-only-v0.23.0-candidate",
  "dashboard_operation_controls_forbidden",
];

for (const marker of requiredHtmlMarkers) {
  if (!html.includes(marker)) {
    throw new Error(`rendered dashboard observability HTML missing marker: ${marker}`);
  }
}

function dashboardFunctionBody(functionName) {
  const needle = `function ${functionName}`;
  const start = dashboardJs.indexOf(needle);
  if (start < 0) {
    throw new Error(`dashboard function not found: ${functionName}`);
  }
  const afterStart = start + needle.length;
  const nextFunction = dashboardJs.indexOf("\nfunction ", afterStart);
  return dashboardJs.slice(start, nextFunction < 0 ? dashboardJs.length : nextFunction);
}

const rendererBodies = [
  dashboardFunctionBody("renderTraderTerminalWorkbench"),
  dashboardFunctionBody("renderReadModelRuntime"),
].join("\n");
for (const forbidden of [
  "<button",
  "<form",
  "<input",
  "fetch(",
  "data-dashboard-action",
  "data-workbench-action",
  "/api/control",
  "/actions/submit",
  "/actions/cancel",
  "/actions/replace",
  "/actions/amend",
  "/actions/flatten",
  "submit_order",
  "cancel_order",
  "replace_order",
  "amend_order",
]) {
  if (rendererBodies.includes(forbidden)) {
    throw new Error(`dashboard observability renderer exposes forbidden action surface: ${forbidden}`);
  }
  if (html.includes(forbidden)) {
    throw new Error(`rendered dashboard observability HTML exposes forbidden action surface: ${forbidden}`);
  }
}

console.log(
  `v23_dashboard_observability_smoke status=ok fixture=${snapshotPath} rows=${runtimes.length} readonly_boundary=locked false_fields=${requiredFalseFields.length}`,
);
NODE
