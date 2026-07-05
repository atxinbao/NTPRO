#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FIXTURE_PATH="${NTPRO_V241_DASHBOARD_ARTIFACT_INGESTION_FIXTURE:-tests/golden/v241_dashboard_order_control_artifact_ingestion.json}"
TASK_PATH="${NTPRO_V241_DASHBOARD_ARTIFACT_INGESTION_TASK:-docs/rust-cutover/tasks/V241-005.md}"
EVIDENCE_PATH="${NTPRO_V241_DASHBOARD_ARTIFACT_INGESTION_EVIDENCE:-docs/rust-cutover/evidence/V241-005.md}"
REPORT_PATH="${NTPRO_V241_DASHBOARD_ARTIFACT_INGESTION_REPORT:-docs/rust-cutover/release/v0_24_1_dashboard_artifact_ingestion.md}"
MANIFEST_PATH="${NTPRO_V241_DASHBOARD_ARTIFACT_INGESTION_MANIFEST:-docs/rust-cutover/release/v0_24_0_release_manifest.json}"

fail() {
  echo "v24.1 dashboard artifact ingestion failed: $*" >&2
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

for path in "$FIXTURE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$REPORT_PATH" "$MANIFEST_PATH" crates/cli/src/dashboard.rs; do
  require_file "$path"
done

python3 -m json.tool "$FIXTURE_PATH" >/dev/null
python3 -m json.tool "$MANIFEST_PATH" >/dev/null

for marker in \
  "Task: \`V241-005\` / GitHub issue \`#774\`" \
  "tests/golden/v241_dashboard_order_control_artifact_ingestion.json" \
  "scripts/ai/verify_release.sh v24.1-dashboard-artifact-ingestion" \
  "forbidden_true_controls = fail_closed" \
  "stale_artifact = not_ready" \
  "missing_provenance = not_ready" \
  "scope_mismatch = fail_closed" \
  "missing_redaction = fail_closed" \
  "dashboard_operation_controls_enabled = false"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
  require_contains "$REPORT_PATH" "$marker"
done

if ! command -v node >/dev/null 2>&1; then
  fail "node is required for dashboard artifact ingestion smoke"
fi

node - "$FIXTURE_PATH" "$MANIFEST_PATH" <<'NODE'
const fs = require("node:fs");
const vm = require("node:vm");

const fixturePath = process.argv[2];
const manifestPath = process.argv[3];
const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
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

const V24_COMPONENT = "v24_order_control_preview";
const EXPECTED_SCHEMA = "ntpro.v241_dashboard_order_control_artifact_ingestion.v1";
const UNIFIED_CONTRACT = "ntpro.v210.unified_read_model.v1";
const UNIFIED_SCHEMA = "ntpro.v210.unified_read_model.schema.v1";
const REQUIRED_FALSE_FIELDS = [
  "new_submit_capability",
  "dashboard_order_controls_enabled",
  "dashboard_approval_controls_enabled",
  "dashboard_cancel_controls_enabled",
  "dashboard_retry_controls_enabled",
  "dashboard_submit_controls_enabled",
  "dashboard_replace_controls_enabled",
  "dashboard_amend_controls_enabled",
  "dashboard_flatten_controls_enabled",
  "dashboard_fill_controls_enabled",
  "trader_terminal_order_ticket_enabled",
  "trader_terminal_live_trading_claim",
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

const HEALTH_RANK = { unknown: 0, healthy: 1, degraded: 2, stale: 3, error: 4 };

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function dv(value) {
  return { availability: "available", value };
}

function unknown() {
  return { availability: "unknown" };
}

function valueAt(root, path) {
  let cursor = root;
  for (const part of path) {
    if (cursor === undefined || cursor === null) {
      return undefined;
    }
    cursor = cursor[part];
  }
  return cursor;
}

function setAt(root, path, value) {
  let cursor = root;
  for (let index = 0; index < path.length - 1; index += 1) {
    const part = path[index];
    if (cursor[part] === undefined || cursor[part] === null || typeof cursor[part] !== "object") {
      cursor[part] = {};
    }
    cursor = cursor[part];
  }
  cursor[path[path.length - 1]] = value;
}

function applyMutations(artifact, mutations) {
  for (const mutation of mutations || []) {
    if (!Array.isArray(mutation.path) || mutation.path.length === 0) {
      throw new Error(`invalid mutation path: ${JSON.stringify(mutation)}`);
    }
    setAt(artifact, mutation.path, clone(mutation.value));
  }
}

function scalar(value) {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (Array.isArray(value)) {
    return value.map((item) => scalar(item)).filter((item) => item !== undefined).join(",");
  }
  if (typeof value === "object") {
    return undefined;
  }
  return String(value);
}

function present(value) {
  return typeof value === "string" ? value.length > 0 : value !== undefined && value !== null;
}

function strongest(current, next) {
  return HEALTH_RANK[next] > HEALTH_RANK[current] ? next : current;
}

function readinessStatus(artifact, health, diagnostics, blockingReasonsPresent) {
  if (artifact.schema_version !== UNIFIED_SCHEMA) {
    return "schema_mismatch";
  }
  if (
    health === "stale" ||
    artifact.freshness?.status === "stale" ||
    diagnostics.some((diagnostic) => diagnostic.endsWith(":freshness_stale"))
  ) {
    return "stale_artifact";
  }
  if (health === "error") {
    return "fail_closed";
  }
  if (health === "degraded" || blockingReasonsPresent) {
    return "degraded_artifact";
  }
  return "ready_readonly_artifact";
}

function buildArtifact(caseItem) {
  const artifact = clone(fixture.artifact_template);
  applyMutations(artifact, caseItem.mutations);
  return artifact;
}

function convertArtifactToRuntime(caseItem) {
  const artifact = buildArtifact(caseItem);
  const component = artifact.components?.[V24_COMPONENT];
  const data = component?.data || {};
  const boundary = artifact.capability_boundary || {};
  const diagnostics = [];
  let health = artifact.health_status === "healthy"
    ? "healthy"
    : artifact.health_status === "degraded"
      ? "degraded"
      : artifact.health_status === "fail_closed"
        ? "error"
        : "error";

  if (artifact.contract_version !== UNIFIED_CONTRACT) {
    diagnostics.push("contract_version_mismatch");
    health = strongest(health, "error");
  }
  if (artifact.schema_version !== UNIFIED_SCHEMA) {
    diagnostics.push("schema_version_mismatch");
    health = strongest(health, "error");
  }
  if (artifact.freshness?.status === "stale") {
    diagnostics.push("snapshot_freshness_stale");
    health = strongest(health, "stale");
  }
  if (!present(artifact.source_provenance?.source_type) || !present(artifact.source_provenance?.source_ref)) {
    diagnostics.push("source_provenance_missing");
    health = strongest(health, "error");
  }
  if (artifact.redaction?.status !== "redacted") {
    diagnostics.push("redaction_state_not_ready");
    health = strongest(health, "error");
  }
  if (!component) {
    diagnostics.push(`${V24_COMPONENT}:missing`);
    health = strongest(health, "error");
  } else {
    if (component.freshness?.status === "stale") {
      diagnostics.push(`${V24_COMPONENT}:freshness_stale`);
      health = strongest(health, "stale");
    } else if (component.freshness?.status !== "fresh") {
      diagnostics.push(`${V24_COMPONENT}:freshness_missing`);
      health = strongest(health, "error");
    }
    if (!present(component.source_provenance?.source_type) || !present(component.source_provenance?.source_ref)) {
      diagnostics.push(`${V24_COMPONENT}:source_provenance_missing`);
      health = strongest(health, "error");
    }
    if (component.redaction?.status !== "redacted") {
      diagnostics.push(`${V24_COMPONENT}:redaction_state_not_ready`);
      health = strongest(health, "error");
    }
    if (data.redaction_state !== "redacted") {
      diagnostics.push(`${V24_COMPONENT}:data_redaction_state_not_ready`);
      health = strongest(health, "error");
    }

    const previewStatus = data.preview_status;
    if (previewStatus === "blocked" || previewStatus === "degraded_unavailable") {
      diagnostics.push(`${V24_COMPONENT}:${previewStatus}`);
      health = strongest(health, "degraded");
    } else if (previewStatus === "fail_closed" || previewStatus === "forbidden_control_detected") {
      diagnostics.push(`${V24_COMPONENT}:${previewStatus}`);
      health = strongest(health, "error");
    } else if (previewStatus !== "ready_preview") {
      diagnostics.push(`${V24_COMPONENT}:preview_status_unexpected:${previewStatus}`);
      health = strongest(health, "error");
    }
    if (String(data.preview_evidence_present) !== "true") {
      diagnostics.push(`${V24_COMPONENT}:preview_evidence_missing`);
      health = strongest(health, "degraded");
    }
    if (previewStatus === "ready_preview" && Array.isArray(data.missing_preview_evidence) && data.missing_preview_evidence.length > 0) {
      diagnostics.push(`${V24_COMPONENT}:ready_with_missing_preview_evidence`);
      health = strongest(health, "error");
    }
    if (String(data.preview_evidence_present) === "true" && !present(data.provenance_ref)) {
      diagnostics.push(`${V24_COMPONENT}:preview_provenance_missing`);
      health = strongest(health, "error");
    }
    const scopeKey = data.scope_key || "";
    const accountId = artifact.snapshot_identity?.account_id || "";
    const venue = (artifact.snapshot_identity?.venue || "").toLowerCase();
    if (!scopeKey.includes(accountId) || !scopeKey.toLowerCase().includes(venue)) {
      diagnostics.push(`${V24_COMPONENT}:scope_mismatch`);
      health = strongest(health, "error");
    }
    if (String(data.forbidden_control_detected) === "true") {
      diagnostics.push(`${V24_COMPONENT}:forbidden_control_detected`);
      health = strongest(health, "error");
    }
  }

  for (const field of REQUIRED_FALSE_FIELDS) {
    if (boundary[field] === true) {
      diagnostics.push(`${field}_true`);
      health = strongest(health, "error");
    } else if (boundary[field] !== false) {
      diagnostics.push(`${field}_missing`);
      health = strongest(health, "error");
    }
  }

  const blockingReasonsPresent = Array.isArray(artifact.blocking_reasons) && artifact.blocking_reasons.length > 0;
  const readiness = readinessStatus(artifact, health, diagnostics, blockingReasonsPresent);
  const runtime = {
    node_id: `terminal-${caseItem.case_id}`,
    health,
    readiness_status: dv(readiness),
    diagnostic: dv(diagnostics.length > 0 ? diagnostics.join(",") : "canonical_unified_read_model_artifact_ready"),
    artifact_path: dv(`${fixturePath}#${caseItem.case_id}`),
    contract_version: dv(artifact.contract_version),
    schema_version: dv(artifact.schema_version),
    snapshot_id: dv(`${artifact.snapshot_id}.${caseItem.case_id}`),
    snapshot_kind: dv(artifact.snapshot_kind),
    snapshot_health_status: dv(artifact.health_status),
    freshness_status: dv(artifact.freshness?.status || "unknown"),
    source_type: dv(artifact.source_provenance?.source_type || "unknown"),
    source_ref: dv(artifact.source_provenance?.source_ref || "unknown"),
    redaction_state: dv(artifact.redaction?.status || "unknown"),
    account_status: dv("healthy"),
    positions_status: dv("healthy"),
    orders_status: dv("healthy"),
    fills_status: dv("healthy"),
    risk_status: dv(health === "error" ? "fail_closed" : health),
    lifecycle_status: dv(data.readback_audit_status || "unknown"),
    operation_entry_status: dv("blocked_disabled_readonly_preview"),
    account_summary: dv(`${artifact.snapshot_identity?.account_id || "unknown"} / strategy-redacted-alpha / venue-node-binance-a`),
    positions_summary: dv(data.scope_key || "unknown"),
    orders_summary: dv(`order-control preview ${data.preview_status || "unknown"}`),
    fills_summary: dv("fills read-only evidence only"),
    risk_summary: dv(`risk ${health}`),
    lifecycle_summary: dv(`readback audit ${data.readback_audit_status || "unknown"}`),
    blocking_reasons: dv((artifact.blocking_reasons || data.blocked_reasons || []).join(",") || "none"),
    missing_components: dv("none"),
    component_diagnostics: dv(diagnostics.join(",") || "none"),
    v24_order_control_preview_status: dv(data.preview_status || "unknown"),
    v24_order_intent_status: dv(data.order_intent_status || "unknown"),
    v24_execution_policy_status: dv(data.execution_policy_status || "unknown"),
    v24_rate_limit_status: dv(data.rate_limit_status || "unknown"),
    v24_slicing_status: dv(data.slicing_status || "unknown"),
    v24_cancel_replace_amend_status: dv(data.cancel_replace_amend_status || "unknown"),
    v24_retry_policy_status: dv(data.retry_policy_status || "unknown"),
    v24_readback_audit_status: dv(data.readback_audit_status || "unknown"),
    v24_blocked_reasons: dv((data.blocked_reasons || []).join(",") || "none"),
    v24_scope_key: dv(data.scope_key || "unknown"),
    v24_source_provenance: dv(data.source_provenance || "unknown"),
    v24_redaction_state: dv(data.redaction_state || "unknown"),
    v24_order_intent_ref: dv(data.order_intent_ref || "unknown"),
    v24_policy_ref: dv(data.policy_ref || "unknown"),
    v24_rate_limit_ref: dv(data.rate_limit_ref || "unknown"),
    v24_slicing_ref: dv(data.slicing_ref || "unknown"),
    v24_cancel_replace_amend_ref: dv(data.cancel_replace_amend_ref || "unknown"),
    v24_retry_policy_ref: dv(data.retry_policy_ref || "unknown"),
    v24_readback_ref: dv(data.readback_ref || "unknown"),
    v24_audit_ref: dv(data.audit_ref || "unknown"),
    v24_provenance_ref: present(data.provenance_ref) ? dv(data.provenance_ref) : unknown(),
    v24_dashboard_redacted_ref: dv(data.dashboard_redacted_ref || "unknown"),
    v24_preview_evidence_present: dv(String(data.preview_evidence_present)),
    v24_missing_preview_evidence: dv((data.missing_preview_evidence || []).join(",") || "none"),
    v24_forbidden_control_detected: dv(String(data.forbidden_control_detected)),
    v24_render_smoke_case: dv(caseItem.case_id),
  };
  for (const field of REQUIRED_FALSE_FIELDS) {
    runtime[field] = boundary[field] === undefined ? unknown() : dv(boundary[field]);
  }
  return { artifact, runtime, diagnostics };
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

if (fixture.schema_version !== EXPECTED_SCHEMA) {
  throw new Error(`unexpected fixture schema: ${fixture.schema_version}`);
}
if (!fixture.artifact_template || !Array.isArray(fixture.cases) || fixture.cases.length !== 7) {
  throw new Error("fixture must contain artifact_template and exactly 7 cases");
}

const manifestEntry = manifest.post_release_dashboard_artifact_ingestion || {};
if (manifestEntry.task_id !== "V241-005" || manifestEntry.issue !== 774) {
  throw new Error("release manifest missing V241-005 dashboard artifact ingestion entry");
}
if (manifestEntry.gate !== "scripts/ai/verify_release.sh v24.1-dashboard-artifact-ingestion") {
  throw new Error("release manifest dashboard artifact ingestion gate mismatch");
}
if (manifestEntry.boundary?.dashboard_operation_controls_enabled !== false) {
  throw new Error("release manifest must keep dashboard operation controls disabled");
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
  "retry_order_action",
];
for (const forbidden of forbiddenSurfaces) {
  if (rendererBodies.includes(forbidden)) {
    throw new Error(`dashboard renderer exposes forbidden action surface: ${forbidden}`);
  }
}

let failClosedCases = 0;
let notReadyCases = 0;
for (const caseItem of fixture.cases) {
  const { runtime, diagnostics } = convertArtifactToRuntime(caseItem);
  if (runtime.health !== caseItem.expected_health) {
    throw new Error(`${caseItem.case_id}: expected health=${caseItem.expected_health}, got ${runtime.health}`);
  }
  if (runtime.readiness_status.value !== caseItem.expected_readiness_status) {
    throw new Error(`${caseItem.case_id}: expected readiness=${caseItem.expected_readiness_status}, got ${runtime.readiness_status.value}`);
  }
  for (const marker of caseItem.expected_diagnostics || []) {
    if (!diagnostics.includes(marker)) {
      throw new Error(`${caseItem.case_id}: missing diagnostic ${marker}; got ${diagnostics.join(",")}`);
    }
  }
  if (!caseItem.expected_ready && runtime.readiness_status.value === "ready_readonly_artifact") {
    throw new Error(`${caseItem.case_id}: negative artifact rendered as ready`);
  }
  if (runtime.readiness_status.value === "fail_closed") {
    failClosedCases += 1;
  }
  if (!caseItem.expected_ready) {
    notReadyCases += 1;
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
    throw new Error(`${caseItem.case_id}: rendered HTML is empty`);
  }

  for (const marker of [
    caseItem.case_id,
    runtime.readiness_status.value,
    runtime.diagnostic.value,
    "v24 Order-control preview",
    "Submit control",
    "Cancel control",
    "Retry policy",
    "Replace control",
    "Amend control",
    "Flatten control",
    "Order ticket",
  ]) {
    if (!html.includes(marker)) {
      throw new Error(`${caseItem.case_id}: rendered HTML missing marker: ${marker}`);
    }
  }
  for (const forbidden of forbiddenSurfaces) {
    if (html.includes(forbidden)) {
      throw new Error(`${caseItem.case_id}: rendered HTML exposes forbidden action surface: ${forbidden}`);
    }
  }
}

const maliciousSelftest = {
  case_id: "v241-dashboard-ingest-selftest-forbidden-submit",
  mutations: [
    {
      path: ["capability_boundary", "dashboard_submit_controls_enabled"],
      value: true,
    },
  ],
};
const selftest = convertArtifactToRuntime(maliciousSelftest);
if (selftest.runtime.readiness_status.value !== "fail_closed" || !selftest.diagnostics.includes("dashboard_submit_controls_enabled_true")) {
  throw new Error("malicious self-test did not fail closed");
}

console.log(
  `v24_1_dashboard_artifact_ingestion status=ok fixture=${fixturePath} cases=${fixture.cases.length} fail_closed_cases=${failClosedCases} not_ready_cases=${notReadyCases} malicious_selftest=1 renderer=readonly`,
);
NODE
