#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V280_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V280_RELEASE_VERSION:-v0.28.0}"
RELEASE_TAG="${NTPRO_V280_RELEASE_TAG:-ntpro-rust-only-v0.28.0}"
RELEASE_NAME="${NTPRO_V280_RELEASE_NAME:-NTPRO Rust-only v0.28.0}"
BASE_RELEASE_TAG="${NTPRO_V280_BASE_RELEASE_TAG:-ntpro-rust-only-v0.27.1}"
MANIFEST_PATH="${NTPRO_V280_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_28_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V280_RELEASE_NOTES:-docs/rust-cutover/release/v0_28_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V280_READINESS_REPORT:-docs/rust-cutover/release/v0_28_0_readiness_report.md}"
MATRIX_PATH="${NTPRO_V280_BACKEND_CLOSURE_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
CURRENT_ISSUE="${NTPRO_V280_CURRENT_ISSUE:-902}"
MILESTONE_NUMBER="${NTPRO_V280_MILESTONE_NUMBER:-22}"
MILESTONE_TITLE="${NTPRO_V280_MILESTONE_TITLE:-v0.28.0}"

fail() {
  echo "v28 release gate failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local marker="$2"
  if ! grep -F -- "$marker" "$path" >/dev/null; then
    fail "missing marker in $path: $marker"
  fi
}

require_not_contains() {
  local path="$1"
  local marker="$2"
  if grep -F -- "$marker" "$path" >/dev/null; then
    fail "forbidden marker in $path: $marker"
  fi
}

gh_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if gh "$@"; then
      return 0
    fi
    if (( attempt >= max_attempts )); then
      return 1
    fi
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

for path in \
  "$MANIFEST_PATH" \
  "$RELEASE_NOTES_PATH" \
  "$READINESS_REPORT_PATH" \
  "$MATRIX_PATH" \
  docs/rust-cutover/release/v0_27_1_release_manifest.json \
  docs/rust-cutover/release/v0_27_1_readiness_report.md \
  docs/rust-cutover/release/v0_27_1_release_notes.md \
  docs/rust-cutover/release/v0_28_0_intake_gate.md \
  docs/rust-cutover/release/v0_28_0_backend_closure_boundary_contract.md \
  docs/rust-cutover/release/v0_28_0_identity_permission_runtime_closure.md \
  docs/rust-cutover/release/v0_28_0_persistent_audit_storage_runtime_closure.md \
  docs/rust-cutover/release/v0_28_0_deployment_orchestration_runtime_closure.md \
  docs/rust-cutover/release/v0_28_0_telemetry_slo_ingestion_runtime_closure.md \
  docs/rust-cutover/release/v0_28_0_admin_workbench_backend_state_bridge_closure.md \
  docs/rust-cutover/release/v0_28_0_trader_terminal_backend_api_contract_handoff.md \
  docs/rust-cutover/release/v0_28_0_backend_closure_fail_closed_hardening.md \
  README.md \
  ROADMAP.md \
  docs/versioning.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/check_release_surface_current.sh \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/verify_release_publish_after_gate.sh \
  scripts/ai/publish_ntpro_release_after_gate.sh \
  scripts/ai/verify_v28_intake_gate.sh \
  scripts/ai/verify_v28_backend_closure_boundary_contract.sh \
  scripts/ai/verify_v28_identity_permission_runtime_closure.sh \
  scripts/ai/verify_v28_persistent_audit_storage_runtime_closure.sh \
  scripts/ai/verify_v28_deployment_orchestration_runtime_closure.sh \
  scripts/ai/verify_v28_telemetry_slo_ingestion_runtime_closure.sh \
  scripts/ai/verify_v28_admin_workbench_backend_state_bridge_closure.sh \
  scripts/ai/verify_v28_trader_terminal_backend_api_contract_handoff.sh \
  scripts/ai/verify_v28_backend_closure_fail_closed_hardening.sh \
  scripts/ai/verify_v28_release_gates.sh \
  scripts/ai/verify_v28_strict_provenance.sh \
  scripts/ai/verify_release.sh; do
  require_file "$path"
done

for task_id in V280-000 V280-001 V280-002 V280-003 V280-004 V280-005 V280-006 V280-007 V280-008 V280-009; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
  require_contains "docs/rust-cutover/tasks/${task_id}.md" "$task_id"
done

for marker in \
  "Status: RELEASED" \
  "Tag: \`$RELEASE_TAG\`" \
  "Release name: \`$RELEASE_NAME\`" \
  "Release URL: \`https://github.com/atxinbao/NTPRO/releases/tag/$RELEASE_TAG\`" \
  "Base release: \`$BASE_RELEASE_TAG\`" \
  "v0.28.0 publishes the Backend Closure / Product Operations Runtime Finalization track" \
  "V280-000" \
  "V280-009" \
  "V280 final release scope issue count = 10" \
  "V280 final release scope evidence count = 10" \
  "V280 exact milestone issue set = #893-#902" \
  "V280 registered corrective-scope exception count = 0" \
  "v28 release gates = required" \
  "v28 strict provenance = required" \
  "backend closure boundary contract = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "hosted release gate success before public GitHub Release = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "local generated publication evidence required in source tree = false" \
  "remote reconstruction required = true" \
  "scripts/ai/verify_release.sh v28-release-gates" \
  "scripts/ai/verify_release.sh v28-strict-provenance" \
  "scripts/ai/verify_v28_release_gates.sh" \
  "scripts/ai/verify_v28_strict_provenance.sh" \
  "scripts/ai/check_github_release_published.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh" \
  "v0.29.0"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V280-000 evidence" \
  "V280-009 evidence" \
  "v28 release gates = required" \
  "v28 strict provenance = required" \
  "#902 V280-009 = must be closed before v0.28.0 tag gate is accepted" \
  "V280 final release scope issue count = 10" \
  "V280 final release scope evidence count = 10" \
  "V280 exact milestone issue set = #893-#902" \
  "V280 registered corrective-scope exception count = 0" \
  "registered corrective-scope exceptions required = true" \
  "unregistered corrective milestone issues fail closed = true" \
  "v0.28.0 milestone = must be closed before public publication" \
  "runtime_closed_count = 10" \
  "evidence_only_count = 2" \
  "blocked_count = 0" \
  "deferred_count = 0"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH"; do
  for marker in \
    "new_submit_capability = true" \
    "production_order_submission_allowed = true" \
    "production_order_mutation_allowed = true" \
    "execution_adapter_call_allowed = true" \
    "adapter_send_allowed = true" \
    "live_exchange_request_allowed = true" \
    "network_attempted = true" \
    "retry_scheduler_enabled = true" \
    "automatic_remediation_allowed = true" \
    "automatic_operation_action_allowed = true" \
    "dashboard_operation_controls_enabled = true" \
    "dashboard_trading_controls_enabled = true" \
    "admin_workbench_operation_controls_enabled = true" \
    "admin_workbench_trading_controls_enabled = true" \
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

scripts/ai/verify_release.sh v28-intake-gate
scripts/ai/verify_release.sh v28-identity-permission-runtime-closure
scripts/ai/verify_release.sh v28-persistent-audit-storage-runtime-closure
scripts/ai/verify_release.sh v28-deployment-orchestration-runtime-closure
scripts/ai/verify_release.sh v28-telemetry-slo-ingestion-runtime-closure
scripts/ai/verify_release.sh v28-admin-workbench-backend-state-bridge-closure
scripts/ai/verify_release.sh v28-trader-terminal-backend-api-contract-handoff
scripts/ai/verify_release.sh v28-backend-closure-fail-closed-hardening
scripts/ai/verify_release.sh v28-backend-closure-boundary-contract

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_NEXT_PATCH_VERSION="v0.28.1" \
  NTPRO_NEXT_CAPABILITY_VERSION="v0.29.0" \
  NTPRO_CURRENT_RELEASE_CAPABILITY="v0.28.0 Backend Closure / Product Operations Runtime Finalization" \
  NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 \
  scripts/ai/verify_release.sh release-surface-current-guard

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
  NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_GATE:-0}" \
  scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh release-publish-after-gate

if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  git rev-parse -q --verify "${RELEASE_TAG}^{commit}" >/dev/null || fail "missing local release tag: $RELEASE_TAG"
  tag_commit="$(git rev-list -n 1 "$RELEASE_TAG")"
  head_commit="$(git rev-parse HEAD)"
  [[ "$head_commit" == "$tag_commit" ]] || fail "HEAD $head_commit does not match $RELEASE_TAG commit $tag_commit"
fi

RELEASE_VERSION="$RELEASE_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
MANIFEST_PATH="$MANIFEST_PATH" \
MATRIX_PATH="$MATRIX_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
matrix = json.loads(Path(os.environ["MATRIX_PATH"]).read_text(encoding="utf-8"))

BOUNDARY_FALSE_FLAGS = [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "automatic_operation_action_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]
EXPECTED_V280 = {
    "V280-000": 893,
    "V280-001": 894,
    "V280-002": 895,
    "V280-003": 896,
    "V280-004": 897,
    "V280-005": 898,
    "V280-006": 899,
    "V280-007": 900,
    "V280-008": 901,
    "V280-009": 902,
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v280_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V280-009", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned name mismatch")
    require(planned.get("draft") is False and planned.get("prerelease") is False, "planned release flags mismatch")
    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.27.1", "base release tag mismatch")
    evidence = candidate.get("v280_evidence") or []
    require(len(evidence) == 10, "V280 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(EXPECTED_V280.get(task_id) == item.get("issue"), f"V280 issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"missing V280 evidence file: {path}")
    scope = candidate.get("release_scope") or {}
    require(scope.get("final_release_scope_issue_count") == 10, "final release scope issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 10, "final release scope evidence count mismatch")
    require(scope.get("exact_milestone_issue_numbers") == list(EXPECTED_V280.values()), "exact issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#893-#902", "exact issue set mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 0, "registered corrective exception count mismatch")
    require(scope.get("unregistered_corrective_milestone_issues_fail_closed") is True, "unregistered corrective fail-closed rule missing")
    require(scope.get("v27_1_dependency_proven") is True, "v27.1 dependency proof missing")
    require(scope.get("v27_1_release_evidence_published") is True, "v27.1 release proof missing")
    require(scope.get("backend_closure_runtime_closed_count") == 10, "backend closure runtime count mismatch")
    require(scope.get("backend_closure_evidence_only_count") == 2, "backend closure evidence-only count mismatch")
    require(scope.get("backend_closure_blocked_count") == 0, "backend closure blocked count mismatch")
    require(scope.get("backend_closure_deferred_count") == 0, "backend closure deferred count mismatch")
    require(scope.get("frontend_product_work_complete") is False, "frontend product work must remain incomplete")
    require(scope.get("capability_scope_expands_trading") is False, "release gate must not expand trading")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v280_issues_closed_required") is True, "V280 closeout requirement missing")
    require(requirements.get("exact_milestone_issue_set_required") is True, "exact issue-set requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("strict_release_body_match_required") is True, "strict body match requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("patch") == "v0.28.1", "next patch mismatch")
    require(next_tracks.get("capability") == "v0.29.0", "next capability mismatch")
    for key in BOUNDARY_FALSE_FLAGS:
        require((candidate.get("boundary_flags") or {}).get(key) is False, f"boundary must remain false: {key}")


validate(manifest)

bad_boundary = copy.deepcopy(manifest)
bad_boundary["boundary_flags"]["adapter_send_allowed"] = True
try:
    validate(bad_boundary)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed adapter_send_allowed")

missing_evidence = copy.deepcopy(manifest)
missing_evidence["v280_evidence"] = missing_evidence["v280_evidence"][:-1]
try:
    validate(missing_evidence)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed missing V280 evidence")

modules = matrix.get("module_readiness") or []
counts = {"runtime-closed": 0, "evidence-only": 0, "blocked": 0, "deferred": 0}
for item in modules:
    counts[item.get("classification")] = counts.get(item.get("classification"), 0) + 1
module = next((item for item in modules if item.get("module_id") == "v28_release_gates_strict_provenance_handoff"), None)
require(module is not None, "matrix missing v28 release gate module")
require(module.get("classification") == "runtime-closed", "v28 release gate module must be runtime-closed")
require(module.get("closure_claim_allowed") is True, "v28 release gate closure claim must be allowed")
require(module.get("evidence_path") == "docs/rust-cutover/evidence/V280-009.md", "v28 release gate evidence path mismatch")
require(module.get("verification_command") == "scripts/ai/verify_release.sh v28-release-gates", "v28 release gate verification command mismatch")
require(matrix.get("expected_counts") == {"runtime-closed": 10, "evidence-only": 2, "blocked": 0, "deferred": 0}, "matrix expected counts mismatch")
require(counts == {"runtime-closed": 10, "evidence-only": 2, "blocked": 0, "deferred": 0}, f"matrix actual counts mismatch: {counts}")
PY

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  issue_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$MILESTONE_TITLE" --limit 50 --json number,state)"
  ISSUE_JSON="$issue_json" CURRENT_ISSUE="$CURRENT_ISSUE" TAG_GATE="${NTPRO_RELEASE_GATE:-0}" python3 <<'PY'
import json
import os

issues = json.loads(os.environ["ISSUE_JSON"])
states = {int(item["number"]): item["state"] for item in issues}
expected = set(range(893, 903))
current_issue = int(os.environ["CURRENT_ISSUE"])
tag_gate = os.environ.get("TAG_GATE") == "1"
if set(states) != expected:
    raise SystemExit(f"V280 milestone issue set mismatch: {sorted(states)}")
for number in sorted(expected):
    state = states[number]
    if tag_gate or number != current_issue:
        if state != "CLOSED":
            raise SystemExit(f"V280 issue must be closed before tag gate: #{number} state={state}")
    elif state not in {"OPEN", "CLOSED"}:
        raise SystemExit(f"unexpected current issue state: #{number} state={state}")
closed = sum(1 for state in states.values() if state == "CLOSED")
mode = "tag_gate" if tag_gate else "pr_mode"
print(f"v280_issue_scope={mode} closed={closed}/10 current_issue_state={states[current_issue]}")
PY
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
    milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$MILESTONE_NUMBER")"
    MILESTONE_JSON="$milestone_json" MILESTONE_TITLE="$MILESTONE_TITLE" python3 <<'PY'
import json
import os

payload = json.loads(os.environ["MILESTONE_JSON"])
if payload.get("title") != os.environ["MILESTONE_TITLE"]:
    raise SystemExit(f"milestone title mismatch: {payload.get('title')}")
if payload.get("state") != "closed":
    raise SystemExit(f"milestone must be closed before tag gate: {payload.get('state')}")
print("v280_milestone_state=closed")
PY
  fi
elif [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  fail "gh authentication is required for tag-gate issue closeout proof"
else
  echo "v280_issue_scope=offline_skip reason=gh_unavailable_or_unauthenticated"
fi

echo "v28_release_gates=pass release_tag=$RELEASE_TAG final_scope_issues=10 final_scope_evidence=10 negative_selftest=1"
