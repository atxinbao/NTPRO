#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

CONTRACT_PATH="${NTPRO_V24_DASHBOARD_PREVIEW_CONTRACT:-docs/rust-cutover/release/v0_24_0_dashboard_workbench_order_control_preview.md}"
TASK_PATH="${NTPRO_V24_DASHBOARD_PREVIEW_TASK:-docs/rust-cutover/tasks/V240-008.md}"
EVIDENCE_PATH="${NTPRO_V24_DASHBOARD_PREVIEW_EVIDENCE:-docs/rust-cutover/evidence/V240-008.md}"
FIXTURE_PATH="${NTPRO_V24_DASHBOARD_PREVIEW_FIXTURE:-tests/golden/v240_dashboard_workbench_order_control_preview.json}"

fail() {
  echo "v24 dashboard workbench order-control preview failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

contains() {
  local path="$1"
  local marker="$2"
  grep -F -- "$marker" "$path" >/dev/null
}

require_contains() {
  local path="$1"
  local marker="$2"
  contains "$path" "$marker" || fail "missing marker in $path: $marker"
}

for path in "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$FIXTURE_PATH" crates/cli/src/dashboard.rs; do
  require_file "$path"
done

for marker in \
  "schema_version = ntpro.v240_dashboard_workbench_order_control_preview.v1" \
  "contract_id = ntpro.v240_dashboard_workbench_order_control_preview.v1" \
  "contract_status = read_only_dashboard_preview_no_operation_controls" \
  "start_gate_dependency = scripts/ai/verify_release.sh v24-readback-audit-evidence" \
  "render_fixture = tests/golden/v240_dashboard_workbench_order_control_preview.json" \
  "normal_case = v240-dashboard-case-normal" \
  "blocked_case = v240-dashboard-case-blocked" \
  "missing_provenance_case = v240-dashboard-case-missing-provenance" \
  "forbidden_control_case = v240-dashboard-case-forbidden-control" \
  "ready_preview = evidence complete and readonly boundary locked" \
  "blocked = policy or risk block is displayed without operation controls" \
  "degraded_unavailable = missing preview evidence displayed as degraded unavailable" \
  "fail_closed = forbidden control marker displayed without exposing controls" \
  "dashboard_submit_controls_enabled = false" \
  "dashboard_cancel_controls_enabled = false" \
  "dashboard_replace_controls_enabled = false" \
  "dashboard_amend_controls_enabled = false" \
  "dashboard_flatten_controls_enabled = false" \
  "trader_terminal_order_ticket_enabled = false" \
  "manual_operation_entry_enabled = false" \
  "live_control_api_added = false" \
  "network_attempted = false" \
  "production_order_mutation_allowed = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

for marker in \
  "dashboard_submit_controls_enabled = true" \
  "dashboard_cancel_controls_enabled = true" \
  "dashboard_replace_controls_enabled = true" \
  "dashboard_amend_controls_enabled = true" \
  "dashboard_flatten_controls_enabled = true" \
  "trader_terminal_order_ticket_enabled = true" \
  "manual_operation_entry_enabled = true" \
  "live_control_api_added = true" \
  "production_order_mutation_allowed = true" \
  "network_attempted = true"; do
  if contains "$CONTRACT_PATH" "$marker"; then
    fail "forbidden marker in $CONTRACT_PATH: $marker"
  fi
done

for marker in \
  "Task: \`V240-008\` / GitHub issue \`#751\`" \
  "tests/golden/v240_dashboard_workbench_order_control_preview.json" \
  "scripts/ai/verify_release.sh v24-dashboard-workbench-preview"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

python3 -m json.tool "$FIXTURE_PATH" >/dev/null

if ! command -v node >/dev/null 2>&1; then
  fail "node is required for dashboard workbench preview render smoke"
fi

node - "$FIXTURE_PATH" <<'NODE'
const fs = require("node:fs");
const vm = require("node:vm");

const fixturePath = process.argv[2];
const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
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

const cases = fixture.cases || [];
const expectedCases = new Set([
  "v240-dashboard-case-normal",
  "v240-dashboard-case-blocked",
  "v240-dashboard-case-missing-provenance",
  "v240-dashboard-case-forbidden-control",
]);
if (fixture.schema_version !== "ntpro.v240_dashboard_workbench_order_control_preview_render_smoke.v1") {
  throw new Error(`unexpected fixture schema: ${fixture.schema_version}`);
}
if (cases.length !== expectedCases.size) {
  throw new Error(`expected ${expectedCases.size} render cases, got ${cases.length}`);
}

const dv = (value) => ({ availability: "available", value });
const maybeDv = (value) => value === "" || value === undefined || value === null
  ? { availability: "unknown" }
  : dv(value);
const joinList = (items) => Array.isArray(items) && items.length > 0 ? items.join(",") : "none";
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

function runtimeFromCase(item) {
  const runtime = {
    node_id: `terminal-${item.case_id}`,
    health: item.health,
    readiness_status: dv(item.readiness_status),
    diagnostic: dv(item.diagnostic),
    artifact_path: dv(`${fixturePath}#${item.case_id}`),
    contract_version: dv("ntpro.v240.dashboard_workbench_order_control_preview.v1"),
    schema_version: dv(fixture.schema_version),
    snapshot_id: dv(item.case_id),
    snapshot_kind: dv("dashboard_workbench_order_control_preview"),
    snapshot_health_status: dv(item.health),
    freshness_status: dv("fresh"),
    source_type: dv("fixture"),
    source_ref: dv(fixturePath),
    redaction_state: dv(item.redaction_state),
    account_status: dv("healthy"),
    positions_status: dv("healthy"),
    orders_status: dv("healthy"),
    fills_status: dv("healthy"),
    risk_status: dv(item.preview_status === "forbidden_control_detected" ? "fail_closed" : item.health),
    lifecycle_status: dv(item.readback_audit_status),
    operation_entry_status: dv("blocked_disabled_readonly_preview"),
    account_summary: dv("acct-redacted-001 / strategy-redacted-alpha / venue-node-binance-a"),
    positions_summary: dv(item.scope_key),
    orders_summary: dv(`order-control preview ${item.preview_status}`),
    fills_summary: dv("fills read-only evidence only"),
    risk_summary: dv(`risk ${item.risk_status || item.health}`),
    lifecycle_summary: dv(`readback audit ${item.readback_audit_status}`),
    blocking_reasons: dv(joinList(item.blocked_reasons)),
    missing_components: dv("none"),
    component_diagnostics: dv(item.diagnostic),
    v24_order_control_preview_status: dv(item.preview_status),
    v24_order_intent_status: dv(item.order_intent_status),
    v24_execution_policy_status: dv(item.execution_policy_status),
    v24_rate_limit_status: dv(item.rate_limit_status),
    v24_slicing_status: dv(item.slicing_status),
    v24_cancel_replace_amend_status: dv(item.cancel_replace_amend_status),
    v24_retry_policy_status: dv(item.retry_policy_status),
    v24_readback_audit_status: dv(item.readback_audit_status),
    v24_blocked_reasons: dv(joinList(item.blocked_reasons)),
    v24_scope_key: dv(item.scope_key),
    v24_source_provenance: dv(item.source_provenance),
    v24_redaction_state: dv(item.redaction_state),
    v24_order_intent_ref: dv(item.order_intent_ref),
    v24_policy_ref: dv(item.policy_ref),
    v24_rate_limit_ref: dv(item.rate_limit_ref),
    v24_slicing_ref: dv(item.slicing_ref),
    v24_cancel_replace_amend_ref: dv(item.cancel_replace_amend_ref),
    v24_retry_policy_ref: dv(item.retry_policy_ref),
    v24_readback_ref: dv(item.readback_ref),
    v24_audit_ref: dv(item.audit_ref),
    v24_provenance_ref: maybeDv(item.provenance_ref),
    v24_dashboard_redacted_ref: dv(item.dashboard_redacted_ref),
    v24_preview_evidence_present: dv(String(item.preview_evidence_present)),
    v24_missing_preview_evidence: dv(joinList(item.missing_preview_evidence)),
    v24_forbidden_control_detected: dv(String(item.forbidden_control_detected)),
    v24_render_smoke_case: dv(item.case_id),
  };
  for (const field of requiredFalseFields) {
    runtime[field] = dv(false);
  }
  runtime.trader_terminal_live_trading_claim = dv(false);
  return runtime;
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
const forbiddenSurfaces = [
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
];
for (const forbidden of forbiddenSurfaces) {
  if (rendererBodies.includes(forbidden)) {
    throw new Error(`v24 workbench renderer exposes forbidden action surface: ${forbidden}`);
  }
}

const seen = new Set();
for (const item of cases) {
  if (!expectedCases.has(item.case_id)) {
    throw new Error(`unexpected render case: ${item.case_id}`);
  }
  seen.add(item.case_id);
  const runtime = runtimeFromCase(item);
  for (const field of requiredFalseFields) {
    const value = runtime[field];
    if (!value || value.availability !== "available" || value.value !== false) {
      throw new Error(`${item.case_id}: ${field} must be explicitly available false`);
    }
  }
  if (item.case_id === "v240-dashboard-case-missing-provenance") {
    if (item.preview_status !== "degraded_unavailable" || item.preview_evidence_present !== false) {
      throw new Error(`${item.case_id}: missing provenance must degrade unavailable without ready preview`);
    }
    if (!Array.isArray(item.missing_preview_evidence) || !item.missing_preview_evidence.includes("provenance_ref")) {
      throw new Error(`${item.case_id}: missing provenance evidence marker required`);
    }
  }
  if (item.case_id === "v240-dashboard-case-forbidden-control" && item.forbidden_control_detected !== true) {
    throw new Error(`${item.case_id}: forbidden control marker required`);
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
  const context = { console, document, __runtime: runtime };
  vm.createContext(context);
  vm.runInContext(
    `${renderOnlyDashboardJs}\nrenderTraderTerminalWorkbench([__runtime]);\nrenderReadModelRuntime([__runtime]);`,
    context,
  );

  const workbenchHtml = elements["trader-terminal-workbench"]?.innerHTML || "";
  const runtimeHtml = elements["read-model-runtime"]?.innerHTML || "";
  const html = `${workbenchHtml}\n${runtimeHtml}`;
  if (!workbenchHtml || !runtimeHtml) {
    throw new Error(`${item.case_id}: rendered HTML is empty`);
  }

  for (const marker of [
    item.case_id,
    item.preview_status,
    item.scope_key,
    item.source_provenance,
    item.redaction_state,
    item.dashboard_redacted_ref,
    "v24 Order-control preview",
    "locked_readonly",
    "Submit control",
    "Cancel control",
    "Replace control",
    "Amend control",
    "Flatten control",
    "Order ticket",
  ]) {
    if (!html.includes(marker)) {
      throw new Error(`${item.case_id}: rendered HTML missing marker: ${marker}`);
    }
  }
  for (const forbidden of forbiddenSurfaces) {
    if (html.includes(forbidden)) {
      throw new Error(`${item.case_id}: rendered HTML exposes forbidden action surface: ${forbidden}`);
    }
  }
}

for (const expected of expectedCases) {
  if (!seen.has(expected)) {
    throw new Error(`missing render case: ${expected}`);
  }
}

console.log(
  `v24_dashboard_workbench_order_control_preview status=ok fixture=${fixturePath} cases=${cases.length} readonly_boundary=locked false_fields=${requiredFalseFields.length}`,
);
NODE
