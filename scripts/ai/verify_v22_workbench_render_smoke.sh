#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SNAPSHOT_PATH="${NTPRO_V22_WORKBENCH_RENDER_SNAPSHOT:-tests/golden/v221/workbench_render_snapshot.json}"

if [[ ! -f "$SNAPSHOT_PATH" ]]; then
  echo "missing workbench render smoke fixture: $SNAPSHOT_PATH" >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "node is required for workbench render smoke" >&2
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

const runtime = snapshot.read_model_runtime?.[0];
if (!runtime) {
  throw new Error("fixture missing read_model_runtime[0]");
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

for (const field of requiredFalseFields) {
  const value = runtime[field];
  if (!value || value.availability !== "available" || value.value !== false) {
    throw new Error(`${field} must be explicitly available false in ${snapshotPath}`);
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
  `${renderOnlyDashboardJs}\nrenderTraderTerminalWorkbench(__snapshot.read_model_runtime || []);`,
  context,
);

const html = elements["trader-terminal-workbench"]?.innerHTML || "";
if (!html) {
  throw new Error("renderTraderTerminalWorkbench produced empty HTML");
}

const requiredHtmlMarkers = [
  "workbench-panel-account",
  "workbench-panel-positions",
  "workbench-panel-orders",
  "workbench-panel-fills",
  "workbench-panel-risk",
  "workbench-panel-alerts",
  "workbench-panel-audit-provenance",
  "workbench-panel-operation-entry",
  "locked_readonly",
  "所有操作控件禁用",
  "ready_readonly_artifact",
  "acct-v221-render",
  "client-v221-render",
  "fill-v221-render",
  "read_only_boundary_ok",
  "ntpro-rust-only-v0.21.1",
  "manual_operation_preview_only",
  "missing_owner_approval,missing_risk_gate,missing_audit_gate",
  "Submit",
  "Cancel",
  "Replace",
  "Amend",
  "Flatten",
];

for (const marker of requiredHtmlMarkers) {
  if (!html.includes(marker)) {
    throw new Error(`rendered workbench HTML missing marker: ${marker}`);
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

const rendererBody = dashboardFunctionBody("renderTraderTerminalWorkbench");
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
  if (rendererBody.includes(forbidden)) {
    throw new Error(`workbench renderer exposes forbidden action surface: ${forbidden}`);
  }
  if (html.includes(forbidden)) {
    throw new Error(`rendered workbench HTML exposes forbidden action surface: ${forbidden}`);
  }
}

console.log(
  `v22_workbench_render_smoke status=ok fixture=${snapshotPath} panels=8 readonly_boundary=locked false_fields=${requiredFalseFields.length}`,
);
NODE
