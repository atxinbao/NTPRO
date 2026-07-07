#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V260_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V260_RELEASE_VERSION:-v0.26.0}"
RELEASE_TAG="${NTPRO_V260_RELEASE_TAG:-ntpro-rust-only-v0.26.0}"
RELEASE_NAME="${NTPRO_V260_RELEASE_NAME:-NTPRO Rust-only v0.26.0}"
BASE_RELEASE_TAG="${NTPRO_V260_BASE_RELEASE_TAG:-ntpro-rust-only-v0.25.1}"
MANIFEST_PATH="${NTPRO_V260_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_26_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V260_RELEASE_NOTES:-docs/rust-cutover/release/v0_26_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V260_READINESS_REPORT:-docs/rust-cutover/release/v0_26_0_readiness_report.md}"
TRACE_PATH="${NTPRO_V260_RELEASE_TRACE:-tests/golden/v260/release_gates_strict_provenance.jsonl}"
CURRENT_ISSUE="${NTPRO_V260_CURRENT_ISSUE:-820}"
MILESTONE_NUMBER="${NTPRO_V260_MILESTONE_NUMBER:-18}"

fail() {
  echo "v26 release gate failed: $*" >&2
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
  "$TRACE_PATH" \
  docs/rust-cutover/release/v0_25_1_release_manifest.json \
  docs/rust-cutover/release/v0_25_1_readiness_report.md \
  docs/rust-cutover/release/v0_26_0_intake_gate.md \
  docs/rust-cutover/release/v0_26_0_product_hardening_boundary_contract.md \
  docs/rust-cutover/release/v0_26_0_operator_permission_model.md \
  docs/rust-cutover/release/v0_26_0_operation_audit_trail.md \
  docs/rust-cutover/release/v0_26_0_deployment_provenance_model.md \
  docs/rust-cutover/release/v0_26_0_upgrade_rollback_runbook_evidence.md \
  docs/rust-cutover/release/v0_26_0_slo_runbook_stability_evidence.md \
  docs/rust-cutover/release/v0_26_0_dashboard_admin_boundary_surface.md \
  docs/rust-cutover/release/v0_26_0_release_closeout_evidence.md \
  docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json \
  scripts/ai/verify_v26_release_gates.sh \
  scripts/ai/verify_v26_strict_provenance.sh \
  scripts/ai/verify_v26_1_final_scope_integration.sh; do
  require_file "$path"
done

for task_id in V260-000 V260-001 V260-002 V260-003 V260-004 V260-005 V260-006 V260-007 V260-008 V260-009 V260-010 V260-011 V260-012 V260-013; do
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
  "v0.26.0 publishes the Product Hardening Foundation" \
  "This release does not add submit capability" \
  "This release is not a product-grade live trading terminal" \
  "V260-000" \
  "V260-008" \
  "V260-013" \
  "V260 final release scope issue count = 14" \
  "V260 final release scope evidence count = 14" \
  "V260 corrective issue scope = #837, #839, #841, #843, #845" \
  "V260 corrective PR scope = #838, #840, #842, #844, #846" \
  "v26 release gates = required" \
  "v26 strict provenance = required" \
  "scripts/ai/verify_release.sh v26.1-final-scope-integration" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "hosted release gate success before public GitHub Release = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "local generated publication evidence required in source tree = false" \
  "remote reconstruction required = true" \
  "new_submit_capability = false" \
  "production_order_submission_allowed = false" \
  "production_order_mutation_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "adapter_send_allowed = false" \
  "live_exchange_request_allowed = false" \
  "retry_scheduler_enabled = false" \
  "automatic_remediation_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "dashboard_trading_controls_enabled = false" \
  "trader_terminal_order_ticket_enabled = false" \
  "manual_operation_submit_allowed = false" \
  "product_grade_trading_terminal_claim = false" \
  "scripts/ai/verify_release.sh v26-release-gates" \
  "scripts/ai/verify_release.sh v26-strict-provenance" \
  "scripts/ai/verify_v26_release_gates.sh" \
  "scripts/ai/verify_v26_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V260-000 evidence" \
  "V260-008 evidence" \
  "V260-013 evidence" \
  "Dashboard smoke = cargo test -p nautilus-cli dashboard_v26_admin_surface --lib -j 1" \
  "artifact ingestion tests = scripts/ai/verify_release.sh v26-dashboard-admin-boundary-surface" \
  "v26 release gates = required" \
  "v26 strict provenance = required" \
  "v26.1 final scope integration = required" \
  "#820 V260-008 = must be closed before v0.26.0 tag gate is accepted" \
  "#845 V260-013 = closed, PR #846 merged" \
  "V260 final release scope issue count = 14" \
  "V260 final release scope evidence count = 14" \
  "v0.26.0 milestone = must be closed before public publication"; do
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
    "retry_scheduler_enabled = true" \
    "automatic_remediation_allowed = true" \
    "dashboard_operation_controls_enabled = true" \
    "dashboard_trading_controls_enabled = true" \
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

scripts/ai/verify_release.sh v26-intake-gate
scripts/ai/verify_release.sh v26-product-hardening-boundary-contract
scripts/ai/verify_release.sh v26-operator-permission-model
scripts/ai/verify_release.sh v26-operation-audit-trail
scripts/ai/verify_release.sh v26-deployment-provenance-model
scripts/ai/verify_release.sh v26-upgrade-rollback-runbook-evidence
scripts/ai/verify_release.sh v26-slo-runbook-stability-evidence
scripts/ai/verify_release.sh v26-dashboard-admin-boundary-surface
NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_NEXT_PATCH_VERSION="v0.26.1" \
  NTPRO_NEXT_CAPABILITY_VERSION="v0.27.0" \
  NTPRO_CURRENT_RELEASE_CAPABILITY="v0.26.0 Product Hardening Foundation" \
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
TRACE_PATH="$TRACE_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
rows = [
    json.loads(line)
    for line in Path(os.environ["TRACE_PATH"]).read_text(encoding="utf-8").splitlines()
    if line.strip()
]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


required_false_flags = (
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
)


def classify(row: dict) -> str:
    if row.get("v25_1_release_evidence_published") is not True:
        return "fail_closed_missing_v25_1_release_evidence"
    if row.get("v260_evidence_count") != 14:
        return "fail_closed_missing_v260_evidence"
    if row.get("v260_trace_sets") != 14:
        return "fail_closed_missing_v260_trace_scope"
    if row.get("dashboard_smoke_passed") is not True:
        return "fail_closed_missing_dashboard_smoke"
    if row.get("artifact_ingestion_tests_passed") is not True:
        return "fail_closed_missing_artifact_ingestion_test"
    if row.get("public_release_published_before_hosted_gate") is True:
        return "fail_closed_publication_before_gate"
    boundary = row.get("boundary_flags") or {}
    for key in required_false_flags:
        if boundary.get(key) is not False:
            return "fail_closed_forbidden_boundary_open"
    if row.get("release_gate_success_before_publication_required") is not True:
        return "fail_closed_missing_publication_order_requirement"
    return "release_gate_ready"


def validate_trace(candidate_rows: list[dict]) -> None:
    require(len(candidate_rows) == 6, "V260 release trace case count mismatch")
    seen = set()
    for row in candidate_rows:
        case_id = row.get("case_id")
        require(case_id and case_id not in seen, f"duplicate or empty case_id: {case_id}")
        seen.add(case_id)
        require(row.get("category") == "release", f"release trace category mismatch: {case_id}")
        actual = classify(row)
        require(actual == row.get("expected_status"), f"{case_id} expected {row.get('expected_status')} got {actual}")


def validate_manifest(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v260_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V260-008", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest product version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")
    require(candidate.get("capability_scope") == "product_hardening_foundation", "capability scope mismatch")

    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.25.1", "base release tag mismatch")
    for key in ("release_manifest_path", "readiness_report_path", "intake_gate_path"):
        require(Path(base.get(key, "")).is_file(), f"base release input missing: {key}")

    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned release name mismatch")
    require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['RELEASE_TAG']}", "planned release URL mismatch")
    require(planned.get("target_commitish") == "main", "planned target mismatch")
    require(planned.get("draft") is False and planned.get("prerelease") is False, "planned release flags mismatch")

    inputs = candidate.get("release_inputs") or {}
    for key, value in inputs.items():
        require(Path(value).is_file(), f"release input missing: {key} -> {value}")

    expected = {
        "V260-000": 812,
        "V260-001": 813,
        "V260-002": 814,
        "V260-003": 815,
        "V260-004": 816,
        "V260-005": 817,
        "V260-006": 818,
        "V260-007": 819,
        "V260-008": 820,
        "V260-009": 837,
        "V260-010": 839,
        "V260-011": 841,
        "V260-012": 843,
        "V260-013": 845,
    }
    evidence = candidate.get("v260_evidence") or []
    require(len(evidence) == len(expected), "V260 evidence count mismatch")
    corrective_prs = {
        "V260-009": 838,
        "V260-010": 840,
        "V260-011": 842,
        "V260-012": 844,
        "V260-013": 846,
    }
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V260 evidence issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"V260 evidence file missing: {path}")
        require(task_id in path.read_text(encoding="utf-8"), f"V260 evidence marker missing: {path}")
        if task_id in corrective_prs:
            require(item.get("pull_request") == corrective_prs[task_id], f"V260 corrective PR mismatch: {task_id}")
            require(item.get("scope") == "corrective_release_publication_governance", f"V260 corrective scope mismatch: {task_id}")
            require(item.get("capability_expansion") is False, f"V260 corrective capability expansion must be false: {task_id}")
            require(item.get("runtime_behavior_changed") is False, f"V260 corrective runtime change must be false: {task_id}")
            require(item.get("trading_behavior_changed") is False, f"V260 corrective trading change must be false: {task_id}")

    scope = candidate.get("release_scope") or {}
    require(scope.get("milestone_issue_count") == 14, "milestone issue count mismatch")
    require(scope.get("corrective_issue_count") == 5, "corrective issue count mismatch")
    require(scope.get("corrective_issue_numbers") == [837, 839, 841, 843, 845], "corrective issue numbers mismatch")
    require(scope.get("corrective_pull_requests") == [838, 840, 842, 844, 846], "corrective PR scope mismatch")
    require(scope.get("final_release_scope_issue_count") == 14, "final scope issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 14, "final scope evidence count mismatch")
    require(scope.get("v25_1_release_evidence_published") is True, "v25.1 dependency proof missing")
    require(scope.get("corrective_scope_expands_capability") is False, "corrective scope must not expand capability")
    require(scope.get("corrective_scope_changes_runtime_behavior") is False, "corrective scope must not change runtime behavior")
    require(scope.get("corrective_scope_changes_trading_behavior") is False, "corrective scope must not change trading behavior")
    require(scope.get("capability_scope_expands_trading") is False, "v26 release gate must not expand trading")
    require(scope.get("runtime_behavior_changed_by_release_gate") is False, "release gate must not change runtime behavior")
    require(scope.get("trading_behavior_changed_by_release_gate") is False, "release gate must not change trading behavior")

    corrective = candidate.get("corrective_release_scope") or []
    corrective_by_task = {item.get("task_id"): item for item in corrective}
    require(set(corrective_by_task) == set(corrective_prs), "corrective release scope task set mismatch")
    merge_commits = {
        "V260-009": "70892e473ef0fd63618fd2bb968e8b8fb61cf4f0",
        "V260-010": "eff3e7045e14a5ae9ffba537799fb8b6a7132c00",
        "V260-011": "7147a5e18a8527730cfb91944eada52eaa9e041c",
        "V260-012": "959bc488ee430d76a8eb44ea0716f22b232e39d4",
        "V260-013": "b09ec3a9f96ac718d6660b345a74cb4b7790f19a",
    }
    for task_id, pull_request in corrective_prs.items():
        item = corrective_by_task[task_id]
        require(item.get("issue") == expected[task_id], f"corrective issue mismatch: {task_id}")
        require(item.get("pull_request") == pull_request, f"corrective PR mismatch: {task_id}")
        require(item.get("merge_commit") == merge_commits[task_id], f"corrective merge commit mismatch: {task_id}")
        require(item.get("included_in_release_tag") is True, f"corrective scope must be included in tag: {task_id}")
        require(item.get("capability_expansion") is False, f"corrective capability expansion must be false: {task_id}")
        require(item.get("runtime_behavior_changed") is False, f"corrective runtime flag must be false: {task_id}")
        require(item.get("trading_behavior_changed") is False, f"corrective trading flag must be false: {task_id}")

    commands = {
        gate.get("command")
        for gate in candidate.get("release_gates", [])
        if gate.get("required") is True
    }
    for command in (
        "scripts/ai/verify_release.sh v26-intake-gate",
        "scripts/ai/verify_release.sh v26-product-hardening-boundary-contract",
        "scripts/ai/verify_release.sh v26-operator-permission-model",
        "scripts/ai/verify_release.sh v26-operation-audit-trail",
        "scripts/ai/verify_release.sh v26-deployment-provenance-model",
        "scripts/ai/verify_release.sh v26-upgrade-rollback-runbook-evidence",
        "scripts/ai/verify_release.sh v26-slo-runbook-stability-evidence",
        "scripts/ai/verify_release.sh v26-dashboard-admin-boundary-surface",
        "scripts/ai/verify_release.sh v26-release-gates",
        "scripts/ai/verify_release.sh v26-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_v26_release_gates.sh",
        "scripts/ai/verify_v26_strict_provenance.sh",
        "scripts/ai/verify_release.sh v26.1-final-scope-integration",
        "scripts/ai/verify_v26_1_final_scope_integration.sh",
    ):
        require(command in commands, f"required release gate missing: {command}")

    capability = candidate.get("capability") or {}
    for key in (
        "product_hardening_foundation",
        "release_governance",
        "strict_provenance",
        "operator_permission_evidence",
        "operation_audit_evidence",
        "deployment_provenance_evidence",
        "upgrade_rollback_runbook_evidence",
        "slo_runbook_stability_evidence",
        "dashboard_admin_readonly_surface",
        "gate_before_publish",
    ):
        require(capability.get(key) is True, f"capability must be true: {key}")
    for key in (
        "product_grade_live_trading_terminal",
        "new_submit_capability",
        "production_order_mutation_expansion",
        "dashboard_trading_controls",
        "automatic_remediation_runtime",
    ):
        require(capability.get(key) is False, f"capability must be false: {key}")

    boundary = candidate.get("boundary_flags") or {}
    for key in (
        "new_submit_capability",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "execution_adapter_call_allowed",
        "adapter_send_allowed",
        "live_exchange_request_allowed",
        "network_attempted",
        "implicit_retry_allowed",
        "retry_scheduler_enabled",
        "automatic_cancel_allowed",
        "automatic_retry_allowed",
        "automatic_remediation_allowed",
        "automatic_recovery_allowed",
        "automatic_operation_action_allowed",
        "strategy_driven_production_execution_allowed",
        "shared_approval_consumption_allowed",
        "cancel_replace_amend_send_allowed",
        "flatten_allowed",
        "dashboard_operation_controls_enabled",
        "dashboard_trading_controls_enabled",
        "dashboard_order_controls_enabled",
        "dashboard_approval_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "dashboard_retry_controls_enabled",
        "dashboard_submit_controls_enabled",
        "dashboard_replace_controls_enabled",
        "dashboard_amend_controls_enabled",
        "dashboard_flatten_controls_enabled",
        "dashboard_remediation_controls_enabled",
        "trader_terminal_order_ticket_enabled",
        "trader_terminal_live_trading_claim",
        "manual_operation_entry_enabled",
        "manual_operation_submit_allowed",
        "manual_operation_cancel_allowed",
        "manual_operation_retry_allowed",
        "manual_operation_replace_allowed",
        "manual_operation_amend_allowed",
        "manual_operation_flatten_allowed",
        "product_grade_trading_terminal_claim",
    ):
        require(boundary.get(key) is False, f"boundary flag must be false: {key}")

    publication = candidate.get("publication_governance") or {}
    require(publication.get("gate_before_publish") is True, "gate before publish missing")
    require(publication.get("public_release_requires_successful_hosted_gate_for_same_tag_commit") is True, "same tag hosted gate requirement missing")
    require(publication.get("release_gate_success_before_publication_required") is True, "publication order requirement missing")
    require(publication.get("publication_evidence_strategy") == "source_tree_plus_github_remote", "publication strategy mismatch")

    surface = candidate.get("release_surface_current_guard") or {}
    require(surface.get("current_release_version") == "v0.26.0", "release surface version mismatch")
    require(surface.get("current_release_tag") == "ntpro-rust-only-v0.26.0", "release surface tag mismatch")
    require(surface.get("next_patch_version") == "v0.26.1", "next patch mismatch")
    require(surface.get("next_capability_version") == "v0.27.0", "next capability mismatch")

    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("release_tag") == os.environ["RELEASE_TAG"], "post-publication tag mismatch")
    require(requirements.get("milestone_number") == 18, "milestone number mismatch")
    require(requirements.get("all_v260_issues_closed_required") is True, "issue closeout requirement missing")
    require(requirements.get("v260_milestone_issue_count") == 14, "issue count requirement mismatch")
    require(requirements.get("final_release_scope_issue_count") == 14, "final scope issue count requirement mismatch")
    require(requirements.get("corrective_issue_count") == 5, "corrective issue count requirement mismatch")
    require(requirements.get("corrective_release_scope_closed_required") is True, "corrective closeout requirement missing")
    require(requirements.get("github_release_published_required") is True, "GitHub release requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("strict_release_body_match_required") is True, "strict release body requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")


validate_trace(rows)
validate_manifest(manifest)

if os.environ.get("NTPRO_V260_RELEASE_SELFTEST", "1") == "1":
    missing = copy.deepcopy(manifest)
    missing["v260_evidence"] = missing["v260_evidence"][:-1]
    try:
        validate_manifest(missing)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing V260 evidence")

    opened = copy.deepcopy(manifest)
    opened["boundary_flags"]["dashboard_trading_controls_enabled"] = True
    try:
        validate_manifest(opened)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: dashboard trading controls enabled")

    bad_rows = copy.deepcopy(rows)
    bad_rows[0]["dashboard_smoke_passed"] = False
    bad_rows[0]["expected_status"] = "release_gate_ready"
    try:
        validate_trace(bad_rows)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing dashboard smoke")
PY

if command -v gh >/dev/null 2>&1 && gh_with_retry auth status >/dev/null 2>&1; then
  for issue in 812 813 814 815 816 817 818 819 820 837 839 841 843 845; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" || "${NTPRO_V260_RELEASE_REQUIRE_CLOSEOUT:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "GitHub issue #$CURRENT_ISSUE must be CLOSED for the tag gate, got $current_state"
  else
    [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current release issue state: $current_state"
  fi

  milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read GitHub milestone #$MILESTONE_NUMBER"
  MILESTONE_JSON="$milestone_json" \
  RELEASE_GATE="${NTPRO_RELEASE_GATE:-0}" \
  REQUIRE_CLOSEOUT="${NTPRO_V260_RELEASE_REQUIRE_CLOSEOUT:-0}" \
  python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
if milestone["title"] != "v0.26.0":
    raise SystemExit(milestone)
if os.environ["RELEASE_GATE"] == "1" or os.environ["REQUIRE_CLOSEOUT"] == "1":
    if milestone["state"] != "closed" or milestone["open_issues"] != 0 or milestone["closed_issues"] < 14:
        raise SystemExit(f"v0.26.0 milestone must be closed with at least 14 closed issues for release gate: {milestone}")
else:
    if milestone["state"] not in {"open", "closed"}:
        raise SystemExit(milestone)
PY
else
  fail "gh authentication is required for v26 release gate issue proof"
fi

echo "v26_release_gates status=ok release_tag=$RELEASE_TAG base_release=$BASE_RELEASE_TAG current_issue_state=$current_state final_scope_issues=14 corrective_scope=5 negative_selftest=${NTPRO_V260_RELEASE_SELFTEST:-1}"
