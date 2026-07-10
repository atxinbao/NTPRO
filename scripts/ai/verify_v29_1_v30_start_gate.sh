#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V291_V30_START_REPO:-atxinbao/NTPRO}"
REQUIREMENTS_PATH="${NTPRO_V291_V30_START_REQUIREMENTS:-docs/rust-cutover/release/v0_29_1_v30_start_gate_requirements.json}"
START_GATE_DOC_PATH="${NTPRO_V291_V30_START_DOC:-docs/rust-cutover/release/v0_29_1_v30_start_gate.md}"
V290_MANIFEST_PATH="${NTPRO_V291_V30_START_V290_MANIFEST:-docs/rust-cutover/release/v0_29_0_release_manifest.json}"
V290_HANDOFF_PATH="${NTPRO_V291_V30_START_V290_HANDOFF:-docs/rust-cutover/release/v0_29_0_v30_go_live_candidate_handoff.md}"
V290_READINESS_PATH="${NTPRO_V291_V30_START_V290_READINESS:-docs/rust-cutover/release/v0_29_0_readiness_report.md}"
V290_CLOSEOUT_PATH="${NTPRO_V291_V30_START_V290_CLOSEOUT:-docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md}"
V291_MILESTONE_TITLE="${NTPRO_V291_V30_START_MILESTONE:-v0.29.1}"
V291_RELEASE_TAG="${NTPRO_V291_V30_START_RELEASE_TAG:-ntpro-rust-only-v0.29.1}"
V291_RELEASE_CLOSEOUT_PATH="${NTPRO_V291_V30_START_RELEASE_CLOSEOUT:-docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md}"

fail() {
  echo "v29.1 v30 start gate failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

gh_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if GODEBUG=http2client=0 gh "$@"; then
      return 0
    fi
    if (( attempt >= max_attempts )); then
      return 1
    fi
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

git_ls_remote_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if git ls-remote "$@"; then
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
  "$REQUIREMENTS_PATH" \
  "$START_GATE_DOC_PATH" \
  "$V290_MANIFEST_PATH" \
  "$V290_HANDOFF_PATH" \
  "$V290_READINESS_PATH" \
  "$V290_CLOSEOUT_PATH" \
  docs/rust-cutover/release/README.md \
  scripts/ai/verify_v29_1_release_closeout_evidence.sh \
  scripts/ai/verify_v29_1_release_publish_after_gate_current_binding.sh \
  scripts/ai/verify_v29_1_stale_v290_evidence_cleanup.sh \
  scripts/ai/verify_v29_1_post_publication_closeout_gate.sh; do
  require_file "$path"
done

for task_id in V291-001 V291-002 V291-003 V291-004 V291-005; do
  require_file "docs/rust-cutover/tasks/${task_id}.md"
  require_file "docs/rust-cutover/evidence/${task_id}.md"
done

REQUIREMENTS_PATH="$REQUIREMENTS_PATH" \
START_GATE_DOC_PATH="$START_GATE_DOC_PATH" \
V290_MANIFEST_PATH="$V290_MANIFEST_PATH" \
V290_HANDOFF_PATH="$V290_HANDOFF_PATH" \
V290_READINESS_PATH="$V290_READINESS_PATH" \
V290_CLOSEOUT_PATH="$V290_CLOSEOUT_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


requirements = json.loads(Path(os.environ["REQUIREMENTS_PATH"]).read_text(encoding="utf-8"))
manifest = json.loads(Path(os.environ["V290_MANIFEST_PATH"]).read_text(encoding="utf-8"))
start_doc = Path(os.environ["START_GATE_DOC_PATH"]).read_text(encoding="utf-8")
handoff = Path(os.environ["V290_HANDOFF_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["V290_READINESS_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["V290_CLOSEOUT_PATH"]).read_text(encoding="utf-8")
evidence = Path("docs/rust-cutover/evidence/V291-005.md").read_text(encoding="utf-8")
task = Path("docs/rust-cutover/tasks/V291-005.md").read_text(encoding="utf-8")

false_flags = [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
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
    "backend_go_live_claim",
    "product_grade_trading_terminal_claim",
]


def validate(req: dict, candidate_manifest: dict) -> None:
    require(req.get("schema_version") == "ntpro.v291_v30_start_gate_requirements.v1", "requirements schema mismatch")
    require(req.get("task_id") == "V291-005", "requirements task mismatch")
    require(req.get("issue") == 967, "requirements issue mismatch")
    require(req.get("blocked_capability") == "v0.30.0", "blocked capability mismatch")
    require(req.get("status") == "blocked_until_v291_release_evidence_published", "start status mismatch")

    prior = req.get("required_prior_release") or {}
    require(prior.get("version") == "v0.29.1", "prior release version mismatch")
    require(prior.get("tag") == "ntpro-rust-only-v0.29.1", "prior release tag mismatch")
    require(prior.get("github_release_published_required") is True, "GitHub release requirement missing")
    require(prior.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(prior.get("publication_after_hosted_gate_required") is True, "publication ordering requirement missing")
    require(prior.get("strict_provenance_required") is True, "strict provenance requirement missing")
    require(prior.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
    require(prior.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")

    scope = req.get("required_v291_issue_scope") or {}
    require(scope.get("exact_issue_numbers") == [963, 964, 965, 966, 967, 968], "V291 exact issue numbers mismatch")
    require(scope.get("exact_issue_set") == "#963-#968", "V291 exact issue set mismatch")
    require(scope.get("all_closed_before_v30_start") is True, "all-closed requirement missing")
    require(scope.get("open_issue_blocks_v30_start") is True, "open issue blocker missing")
    require(scope.get("unregistered_milestone_issue_blocks_v30_start") is True, "unregistered issue blocker missing")

    deps = req.get("required_dependencies") or {}
    for task_id, issue in {
        "V291-001": 963,
        "V291-002": 964,
        "V291-003": 965,
        "V291-004": 966,
        "V291-005": 967,
        "V291-006": 968,
    }.items():
        dep = deps.get(task_id) or {}
        require(dep.get("issue") == issue, f"dependency issue mismatch: {task_id}")
        require(dep.get("required") is True, f"dependency required flag missing: {task_id}")

    start = req.get("v30_start_requirements") or {}
    require(start.get("v29_0_publication_evidence_alone_unlocks_v30") is False, "v29.0-only unlock must be false")
    require(start.get("v29_1_release_evidence_required") is True, "v29.1 release evidence requirement missing")
    require(start.get("v29_post_publication_closeout_gate_required") is True, "post-publication closeout dependency missing")
    require(start.get("v29_publish_after_gate_current_binding_required") is True, "publish-after-gate dependency missing")
    require(start.get("v290_stale_evidence_cleanup_required") is True, "stale cleanup dependency missing")
    require(start.get("v30_intake_fails_if_any_v291_issue_open") is True, "open issue fail-closed missing")
    require(start.get("v30_intake_fails_if_v291_release_evidence_missing") is True, "missing release fail-closed missing")
    require(start.get("v30_intake_requires_exact_v291_issue_scope") is True, "exact scope requirement missing")
    require(start.get("v30_intake_requires_v291_release_closeout_proof") is True, "release closeout requirement missing")

    for key in false_flags:
        require((req.get("v30_default_boundary") or {}).get(key) is False, f"v30 boundary must remain false: {key}")

    next_tracks = candidate_manifest.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.30.0", "manifest next capability mismatch")
    require(next_tracks.get("capability_entry") == "backend_production_go_live_candidate_after_v291_release_evidence", "manifest capability entry mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v291_release_evidence_published", "manifest v30 start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v30 implementation must not start")
    require(next_tracks.get("inherits_backend_go_live_claim") is False, "v30 backend go-live inheritance must be false")

    post_requirements = candidate_manifest.get("post_publication_requirements") or {}
    require(post_requirements.get("v0_30_start_gate_fails_without_v290_release_evidence") is True, "v290 base blocker missing")
    require(post_requirements.get("v0_30_start_gate_fails_without_v291_release_evidence") is True, "v291 blocker missing")
    require(post_requirements.get("v0_29_0_publication_evidence_alone_unlocks_v0_30") is False, "v29.0-only unlock flag mismatch")

    gate = candidate_manifest.get("v291_v30_start_gate") or {}
    require(gate.get("task_id") == "V291-005", "manifest gate task mismatch")
    require(gate.get("issue") == 967, "manifest gate issue mismatch")
    require(gate.get("requirements_path") == os.environ["REQUIREMENTS_PATH"], "manifest requirements path mismatch")
    require(gate.get("gate") == "scripts/ai/verify_v29_1_v30_start_gate.sh", "manifest gate command mismatch")
    require(gate.get("required_v291_issue_numbers") == [963, 964, 965, 966, 967, 968], "manifest V291 issue numbers mismatch")
    require(gate.get("v29_0_publication_evidence_alone_unlocks_v30") is False, "manifest v29.0-only unlock mismatch")
    require(gate.get("v29_1_release_evidence_required") is True, "manifest v29.1 release evidence missing")
    require(gate.get("v30_intake_fails_if_any_v291_issue_open") is True, "manifest open issue blocker missing")
    require(gate.get("v30_intake_fails_if_v291_release_evidence_missing") is True, "manifest missing release blocker missing")
    require(gate.get("v30_default_trading_and_mutation_controls_disabled") is True, "manifest v30 boundary disabled flag missing")
    require(gate.get("runtime_behavior_changed") is False, "runtime behavior must not change")
    require(gate.get("trading_behavior_changed") is False, "trading behavior must not change")


validate(requirements, manifest)

for marker, text, label in [
    ("v0.30.0 start gate = blocked_until_v291_release_evidence_published", start_doc, "start gate doc"),
    ("v0.29.0 publication evidence alone unlocks v0.30.0 = false", start_doc, "start gate doc"),
    ("v0.29.1 exact issue set required = #963-#968", start_doc, "start gate doc"),
    ("v0.29.1 release closeout proof required = true", start_doc, "start gate doc"),
    ("v0.30.0 start gate = blocked_until_v291_release_evidence_published", handoff, "handoff"),
    ("v0.29.0 publication evidence alone unlocks v0.30.0 = false", handoff, "handoff"),
    ("v0.29.1 exact issue set required before v0.30.0 intake = #963-#968", handoff, "handoff"),
    ("v0.30.0 go-live candidate start = blocked until v0.29.1 release evidence exists", readiness, "readiness"),
    ("v0.29.0 publication evidence alone unlocks v0.30.0 = false", readiness, "readiness"),
    ("v0.30.0 start gate = blocked_until_v291_release_evidence_published", closeout, "closeout"),
    ("v0.29.1 release evidence required before v0.30.0 intake = true", closeout, "closeout"),
    ("Task: `V291-005` / GitHub issue `#967`", evidence, "V291-005 evidence"),
    ("Status: LOCAL VALIDATION PASS", evidence, "V291-005 evidence"),
    ("v0.30.0 intake fails if any V291 issue remains open = true", evidence, "V291-005 evidence"),
    ("GitHub issue: `#967`", task, "V291-005 task"),
    ("Hard-block v0.30.0 until v0.29.1 closeout", task, "V291-005 task"),
]:
    require(marker in text, f"{label} missing marker: {marker}")

for key in false_flags:
    marker = f"{key} = false"
    require(marker in start_doc, f"start gate boundary missing: {marker}")
    require(marker in handoff, f"handoff boundary missing: {marker}")

if os.environ.get("NTPRO_V291_V30_START_SELFTEST", "1") == "1":
    mutations = {
        "v29_0_only_unlock": lambda r, m: r["v30_start_requirements"].update({"v29_0_publication_evidence_alone_unlocks_v30": True}),
        "missing_v291_issue": lambda r, m: r["required_v291_issue_scope"].update({"exact_issue_numbers": [963, 964, 965, 966, 967]}),
        "release_evidence_not_required": lambda r, m: r["v30_start_requirements"].update({"v29_1_release_evidence_required": False}),
        "trading_controls_enabled": lambda r, m: r["v30_default_boundary"].update({"production_order_submission_allowed": True}),
        "manifest_old_start_gate": lambda r, m: m["next_tracks"].update({"start_gate": "blocked_until_v290_release_evidence_published"}),
    }
    for name, mutate in mutations.items():
        req = copy.deepcopy(requirements)
        candidate_manifest = copy.deepcopy(manifest)
        mutate(req, candidate_manifest)
        try:
            validate(req, candidate_manifest)
        except SystemExit:
            continue
        raise SystemExit(f"negative self-test unexpectedly passed: {name}")
PY

command -v gh >/dev/null 2>&1 || fail "gh is required for live V291 issue proof"
gh_with_retry auth status >/dev/null 2>&1 || fail "gh authentication is required for live V291 issue proof"

issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$V291_MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read V291 milestone issues"
release_json=""
release_status="missing"
if release_json="$(gh_with_retry release view "$V291_RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,publishedAt,url 2>/dev/null)"; then
  release_status="present"
fi
remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V291_RELEASE_TAG^{}" | awk '{print $1}' || true)"
if [[ -z "$remote_tag_commit" ]]; then
  remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V291_RELEASE_TAG" | awk '{print $1}' || true)"
fi

ISSUES_JSON="$issues_json" \
RELEASE_JSON="$release_json" \
RELEASE_STATUS="$release_status" \
REMOTE_TAG_COMMIT="$remote_tag_commit" \
V291_RELEASE_CLOSEOUT_PATH="$V291_RELEASE_CLOSEOUT_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

expected = [963, 964, 965, 966, 967, 968]
issues = json.loads(os.environ["ISSUES_JSON"])
numbers = sorted(int(item["number"]) for item in issues)
if numbers != expected:
    raise SystemExit(f"V291 milestone issue scope mismatch: got={numbers} expected={expected}")

states = {int(item["number"]): item["state"] for item in issues}
open_issues = [number for number in expected if states.get(number) != "CLOSED"]
release_status = os.environ["RELEASE_STATUS"]
remote_tag_commit = os.environ["REMOTE_TAG_COMMIT"].strip()
closeout_exists = Path(os.environ["V291_RELEASE_CLOSEOUT_PATH"]).is_file()

release_evidence = "missing"
if release_status == "present" and remote_tag_commit and closeout_exists:
    release = json.loads(os.environ["RELEASE_JSON"])
    if (
        release.get("tagName") == "ntpro-rust-only-v0.29.1"
        and release.get("isDraft") is False
        and release.get("isPrerelease") is False
    ):
        release_evidence = "present"

status = "ready" if not open_issues and release_evidence == "present" else "blocked"
if status == "ready":
    open_text = "none"
else:
    open_text = ",".join(str(number) for number in open_issues) if open_issues else "none"

print(
    "v29_1_v30_start_gate "
    f"status={status} "
    "release_tag=ntpro-rust-only-v0.29.1 "
    f"release_evidence={release_evidence} "
    f"open_v291_issues={open_text} "
    "exact_scope=#963-#968 "
    "v29_0_alone_unlocks_v30=false"
)
PY
