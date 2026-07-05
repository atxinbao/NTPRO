#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

V240_FIXTURE_PATH="${NTPRO_V241_DASHBOARD_REF_V240_FIXTURE:-tests/golden/v240_dashboard_workbench_order_control_preview.json}"
V241_FIXTURE_PATH="${NTPRO_V241_DASHBOARD_REF_V241_FIXTURE:-tests/golden/v241_dashboard_order_control_artifact_ingestion.json}"
TASK_PATH="${NTPRO_V241_DASHBOARD_REF_TASK:-docs/rust-cutover/tasks/V241-007.md}"
EVIDENCE_PATH="${NTPRO_V241_DASHBOARD_REF_EVIDENCE:-docs/rust-cutover/evidence/V241-007.md}"
REPORT_PATH="${NTPRO_V241_DASHBOARD_REF_REPORT:-docs/rust-cutover/release/v0_24_1_dashboard_fixture_ref_integrity.md}"
MANIFEST_PATH="${NTPRO_V241_DASHBOARD_REF_MANIFEST:-docs/rust-cutover/release/v0_24_0_release_manifest.json}"

fail() {
  echo "v24.1 dashboard fixture ref integrity failed: $*" >&2
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

for path in "$V240_FIXTURE_PATH" "$V241_FIXTURE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$REPORT_PATH" "$MANIFEST_PATH"; do
  require_file "$path"
done

python3 -m json.tool "$V240_FIXTURE_PATH" >/dev/null
python3 -m json.tool "$V241_FIXTURE_PATH" >/dev/null
python3 -m json.tool "$MANIFEST_PATH" >/dev/null

for marker in \
  "Task: \`V241-007\` / GitHub issue \`#776\`" \
  "tests/golden/v240_dashboard_workbench_order_control_preview.json" \
  "tests/golden/v241_dashboard_order_control_artifact_ingestion.json" \
  "scripts/ai/verify_release.sh v24.1-dashboard-fixture-ref-integrity" \
  "policy_ref = docs/rust-cutover/release/v0_24_0_order_intent_execution_policy.md" \
  "bad_path_selftest = fail_closed" \
  "bad_jsonl_anchor_selftest = fail_closed" \
  "bad_markdown_anchor_selftest = fail_closed" \
  "dashboard_operation_controls_enabled = false"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
  require_contains "$REPORT_PATH" "$marker"
done

if ! command -v node >/dev/null 2>&1; then
  fail "node is required for dashboard fixture ref integrity validation"
fi

node - "$V240_FIXTURE_PATH" "$V241_FIXTURE_PATH" "$MANIFEST_PATH" <<'NODE'
const fs = require("node:fs");
const path = require("node:path");

const [v240FixturePath, v241FixturePath, manifestPath] = process.argv.slice(2);
const v240Fixture = JSON.parse(fs.readFileSync(v240FixturePath, "utf8"));
const v241Fixture = JSON.parse(fs.readFileSync(v241FixturePath, "utf8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));

const REF_FIELDS = [
  "order_intent_ref",
  "policy_ref",
  "rate_limit_ref",
  "slicing_ref",
  "cancel_replace_amend_ref",
  "retry_policy_ref",
  "readback_ref",
  "audit_ref",
  "provenance_ref",
  "dashboard_redacted_ref",
];

const JSONL_ANCHOR_ALIASES = {
  "tests/golden/v240_order_intent_execution_policy.jsonl": {
    ready: "execution.v240_order_intent_policy.valid_intent.001",
    risk_rejected: "execution.v240_order_intent_policy.forbidden_operation.001",
  },
  "tests/golden/v240_rate_limit_throttle_gate.jsonl": {
    accepted: "execution.v240_rate_limit_throttle.allowed_preview.001",
  },
  "tests/golden/v240_order_slicing_preview.jsonl": {
    single_slice: "execution.v240_order_slicing.valid_plan.001",
    not_applicable: "execution.v240_order_slicing.policy_missing.001",
  },
  "tests/golden/v240_cancel_replace_amend_preview.jsonl": {
    cancel_preview: "execution.v240_cancel_replace_amend.cancel_preview.001",
    replace_preview: "execution.v240_cancel_replace_amend.replace_preview.001",
    amend_preview: "execution.v240_cancel_replace_amend.amend_preview.001",
    blocked: "execution.v240_cancel_replace_amend.missing_lineage.001",
  },
  "tests/golden/v240_retry_policy_ledger.jsonl": {
    no_retry_terminal: "execution.v240_retry_policy.business_rejection_terminal.001",
    risk_rejection_terminal: "execution.v240_retry_policy.risk_rejection_terminal.001",
    transport_retry_allowed: "execution.v240_retry_policy.transport_retry_allowed.001",
    timeout_retry_allowed: "execution.v240_retry_policy.timeout_retry_allowed.001",
  },
  "tests/golden/v240_readback_audit_evidence.jsonl": {
    readback: "execution.v240_readback_audit.ready_preview.001",
    audit: "execution.v240_readback_audit.ready_preview.001",
    provenance: "execution.v240_readback_audit.ready_preview.001",
    blocked: "execution.v240_readback_audit.cross_scope_mismatch.001",
    audit_blocked: "execution.v240_readback_audit.cross_scope_mismatch.001",
    provenance_blocked: "execution.v240_readback_audit.cross_scope_mismatch.001",
    degraded_unavailable: "execution.v240_readback_audit.degraded_unavailable.001",
    audit_degraded: "execution.v240_readback_audit.degraded_unavailable.001",
    fail_closed: "execution.v240_readback_audit.missing_provenance.001",
    audit_fail_closed: "execution.v240_readback_audit.missing_provenance.001",
    provenance_fail_closed: "execution.v240_readback_audit.missing_provenance.001",
  },
};

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function fail(message) {
  throw new Error(message);
}

function splitRef(ref) {
  const index = ref.indexOf("#");
  if (index < 0) {
    return { filePath: ref, anchor: "" };
  }
  return {
    filePath: ref.slice(0, index),
    anchor: ref.slice(index + 1),
  };
}

function jsonlCaseIds(filePath) {
  return fs.readFileSync(filePath, "utf8")
    .trim()
    .split(/\n+/)
    .filter(Boolean)
    .map((line) => JSON.parse(line).case_id);
}

function markdownSlug(heading) {
  return heading
    .trim()
    .toLowerCase()
    .replace(/`/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function markdownAnchors(filePath) {
  const content = fs.readFileSync(filePath, "utf8");
  const anchors = new Set();
  for (const match of content.matchAll(/<a\s+id=["']([^"']+)["']\s*><\/a>/g)) {
    anchors.add(match[1]);
  }
  for (const line of content.split(/\n/)) {
    const heading = line.match(/^#{1,6}\s+(.+)$/);
    if (heading) {
      anchors.add(markdownSlug(heading[1]));
    }
  }
  return anchors;
}

function resolveRef(ref, context) {
  if (!ref) {
    if (
      context.field === "provenance_ref" &&
      context.record.preview_evidence_present === false &&
      Array.isArray(context.record.missing_preview_evidence) &&
      context.record.missing_preview_evidence.includes("provenance_ref")
    ) {
      return { skippedMissing: true };
    }
    fail(`${context.label}: ${context.field} is empty`);
  }

  const { filePath, anchor } = splitRef(ref);
  if (filePath === "docs/rust-cutover/release/v0_24_0_order_intent_policy.md") {
    fail(`${context.label}: stale policy_ref path remains: ${ref}`);
  }
  if (!fs.existsSync(filePath)) {
    fail(`${context.label}: missing ref path for ${context.field}: ${filePath}`);
  }
  if (!anchor) {
    return { resolved: true, filePath };
  }
  if (filePath.endsWith(".jsonl")) {
    const caseIds = new Set(jsonlCaseIds(filePath));
    const aliases = JSONL_ANCHOR_ALIASES[filePath] || {};
    const target = aliases[anchor] || anchor;
    if (!caseIds.has(target)) {
      fail(`${context.label}: unresolved JSONL anchor for ${context.field}: ${ref}`);
    }
    return { resolved: true, filePath, anchor, target };
  }
  if (filePath.endsWith(".md")) {
    const anchors = markdownAnchors(filePath);
    if (!anchors.has(anchor)) {
      fail(`${context.label}: unresolved Markdown anchor for ${context.field}: ${ref}`);
    }
    return { resolved: true, filePath, anchor };
  }
  fail(`${context.label}: unsupported anchored ref type for ${context.field}: ${ref}`);
}

function v240Records(fixture) {
  return (fixture.cases || []).map((record) => ({
    label: record.case_id,
    record,
  }));
}

function v241Records(fixture) {
  const record = fixture.artifact_template?.components?.v24_order_control_preview?.data;
  if (!record) {
    fail("v241 artifact ingestion fixture missing v24_order_control_preview data");
  }
  return [{ label: "v241-dashboard-artifact-template", record }];
}

function validateRecords(label, records) {
  let resolved = 0;
  let skippedMissing = 0;
  for (const item of records) {
    for (const field of REF_FIELDS) {
      const result = resolveRef(item.record[field], {
        label: `${label}:${item.label}`,
        field,
        record: item.record,
      });
      if (result.skippedMissing) {
        skippedMissing += 1;
      } else {
        resolved += 1;
      }
    }
  }
  return { resolved, skippedMissing };
}

function expectFailure(name, mutate) {
  const fixture = clone(v240Fixture);
  mutate(fixture);
  try {
    validateRecords(name, v240Records(fixture));
  } catch (error) {
    return String(error.message);
  }
  fail(`${name}: expected ref validation to fail`);
}

const manifestEntry = manifest.post_release_dashboard_fixture_ref_integrity || {};
if (manifestEntry.task_id !== "V241-007" || manifestEntry.issue !== 776) {
  fail("release manifest missing V241-007 dashboard fixture ref integrity entry");
}
if (manifestEntry.gate !== "scripts/ai/verify_release.sh v24.1-dashboard-fixture-ref-integrity") {
  fail("release manifest fixture ref integrity gate mismatch");
}
if (manifestEntry.boundary?.dashboard_operation_controls_enabled !== false) {
  fail("release manifest must keep Dashboard operation controls disabled");
}

const v240 = validateRecords("v240-dashboard-workbench", v240Records(v240Fixture));
const v241 = validateRecords("v241-dashboard-artifact-ingestion", v241Records(v241Fixture));
const badPath = expectFailure("bad_path_selftest", (fixture) => {
  fixture.cases[0].policy_ref = "docs/rust-cutover/release/missing_policy.md";
});
const badJsonlAnchor = expectFailure("bad_jsonl_anchor_selftest", (fixture) => {
  fixture.cases[0].order_intent_ref =
    "tests/golden/v240_order_intent_execution_policy.jsonl#missing-anchor";
});
const badMarkdownAnchor = expectFailure("bad_markdown_anchor_selftest", (fixture) => {
  fixture.cases[0].dashboard_redacted_ref =
    "docs/rust-cutover/evidence/V240-008.md#missing-anchor";
});

console.log(
  [
    "v24_1_dashboard_fixture_ref_integrity status=ok",
    `v240_resolved=${v240.resolved}`,
    `v240_skipped_missing=${v240.skippedMissing}`,
    `v241_resolved=${v241.resolved}`,
    "bad_path_selftest=1",
    "bad_jsonl_anchor_selftest=1",
    "bad_markdown_anchor_selftest=1",
    `bad_path_error=${JSON.stringify(badPath)}`,
    `bad_jsonl_anchor_error=${JSON.stringify(badJsonlAnchor)}`,
    `bad_markdown_anchor_error=${JSON.stringify(badMarkdownAnchor)}`,
  ].join(" "),
);
NODE
