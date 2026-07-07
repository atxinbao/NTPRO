#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

MANIFEST_PATH="${NTPRO_V261_FINAL_SCOPE_MANIFEST:-docs/rust-cutover/release/v0_26_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V261_FINAL_SCOPE_RELEASE_NOTES:-docs/rust-cutover/release/v0_26_0_release_notes.md}"
READINESS_PATH="${NTPRO_V261_FINAL_SCOPE_READINESS:-docs/rust-cutover/release/v0_26_0_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V261_FINAL_SCOPE_CLOSEOUT:-docs/rust-cutover/release/v0_26_0_release_closeout_evidence.md}"

fail() {
  echo "v26.1 final scope integration failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

for path in "$MANIFEST_PATH" "$RELEASE_NOTES_PATH" "$READINESS_PATH" "$CLOSEOUT_PATH"; do
  require_file "$path"
done

MANIFEST_PATH="$MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_PATH="$READINESS_PATH" \
CLOSEOUT_PATH="$CLOSEOUT_PATH" \
ROOT_DIR="$ROOT_DIR" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["CLOSEOUT_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


expected = {
    "V260-000": {"issue": 812, "path": "docs/rust-cutover/evidence/V260-000.md"},
    "V260-001": {"issue": 813, "path": "docs/rust-cutover/evidence/V260-001.md"},
    "V260-002": {"issue": 814, "path": "docs/rust-cutover/evidence/V260-002.md"},
    "V260-003": {"issue": 815, "path": "docs/rust-cutover/evidence/V260-003.md"},
    "V260-004": {"issue": 816, "path": "docs/rust-cutover/evidence/V260-004.md"},
    "V260-005": {"issue": 817, "path": "docs/rust-cutover/evidence/V260-005.md"},
    "V260-006": {"issue": 818, "path": "docs/rust-cutover/evidence/V260-006.md"},
    "V260-007": {"issue": 819, "path": "docs/rust-cutover/evidence/V260-007.md"},
    "V260-008": {"issue": 820, "path": "docs/rust-cutover/evidence/V260-008.md"},
    "V260-009": {"issue": 837, "pr": 838, "path": "docs/rust-cutover/evidence/V260-009.md", "task_path": "docs/rust-cutover/tasks/V260-009.md", "merge_commit": "70892e473ef0fd63618fd2bb968e8b8fb61cf4f0"},
    "V260-010": {"issue": 839, "pr": 840, "path": "docs/rust-cutover/evidence/V260-010.md", "task_path": "docs/rust-cutover/tasks/V260-010.md", "merge_commit": "eff3e7045e14a5ae9ffba537799fb8b6a7132c00"},
    "V260-011": {"issue": 841, "pr": 842, "path": "docs/rust-cutover/evidence/V260-011.md", "task_path": "docs/rust-cutover/tasks/V260-011.md", "merge_commit": "7147a5e18a8527730cfb91944eada52eaa9e041c"},
    "V260-012": {"issue": 843, "pr": 844, "path": "docs/rust-cutover/evidence/V260-012.md", "task_path": "docs/rust-cutover/tasks/V260-012.md", "merge_commit": "959bc488ee430d76a8eb44ea0716f22b232e39d4"},
    "V260-013": {"issue": 845, "pr": 846, "path": "docs/rust-cutover/evidence/V260-013.md", "task_path": "docs/rust-cutover/tasks/V260-013.md", "merge_commit": "b09ec3a9f96ac718d6660b345a74cb4b7790f19a"},
}
corrective_ids = {"V260-009", "V260-010", "V260-011", "V260-012", "V260-013"}


def validate(candidate: dict, notes_text: str, readiness_text: str, closeout_text: str) -> None:
    require(candidate.get("schema_version") == "ntpro.v260_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("product_version") == "v0.26.0", "manifest product version mismatch")

    evidence = candidate.get("v260_evidence") or []
    evidence_by_task = {item.get("task_id"): item for item in evidence}
    require(list(evidence_by_task) == list(expected), "V260 evidence task order/scope mismatch")
    require(len(evidence_by_task) == 14, "V260 final evidence count must be 14")

    for task_id, facts in expected.items():
        item = evidence_by_task.get(task_id) or {}
        require(item.get("issue") == facts["issue"], f"{task_id} issue mismatch")
        require(item.get("path") == facts["path"], f"{task_id} evidence path mismatch")
        evidence_path = Path(facts["path"])
        require(evidence_path.is_file(), f"{task_id} evidence file missing")
        require(task_id in evidence_path.read_text(encoding="utf-8"), f"{task_id} evidence marker missing")
        if task_id in corrective_ids:
            require(item.get("pull_request") == facts["pr"], f"{task_id} PR mismatch")
            require(item.get("scope") == "corrective_release_publication_governance", f"{task_id} corrective scope mismatch")
            require(item.get("capability_expansion") is False, f"{task_id} capability expansion must stay false")
            require(item.get("runtime_behavior_changed") is False, f"{task_id} runtime behavior flag must stay false")
            require(item.get("trading_behavior_changed") is False, f"{task_id} trading behavior flag must stay false")

    scope = candidate.get("release_scope") or {}
    require(scope.get("milestone_issue_set") == "V260-000..V260-013", "milestone issue set mismatch")
    require(scope.get("milestone_issue_count") == 14, "milestone issue count mismatch")
    require(scope.get("corrective_issue_set") == "V260-009..V260-013", "corrective issue set mismatch")
    require(scope.get("corrective_issue_count") == 5, "corrective issue count mismatch")
    require(scope.get("corrective_issue_numbers") == [837, 839, 841, 843, 845], "corrective issue number mismatch")
    require(scope.get("corrective_pull_requests") == [838, 840, 842, 844, 846], "corrective PR mismatch")
    require(scope.get("final_release_scope_issue_count") == 14, "final scope issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 14, "final scope evidence count mismatch")
    require(scope.get("corrective_scope_expands_capability") is False, "corrective scope must not expand capability")
    require(scope.get("corrective_scope_changes_runtime_behavior") is False, "corrective scope must not change runtime behavior")
    require(scope.get("corrective_scope_changes_trading_behavior") is False, "corrective scope must not change trading behavior")

    corrective = candidate.get("corrective_release_scope") or []
    corrective_by_task = {item.get("task_id"): item for item in corrective}
    require(set(corrective_by_task) == corrective_ids, "corrective release scope task set mismatch")
    require(len(corrective_by_task) == 5, "corrective release scope count mismatch")
    for task_id in sorted(corrective_ids):
        facts = expected[task_id]
        item = corrective_by_task[task_id]
        require(item.get("issue") == facts["issue"], f"{task_id} corrective issue mismatch")
        require(item.get("pull_request") == facts["pr"], f"{task_id} corrective PR mismatch")
        require(item.get("merge_commit") == facts["merge_commit"], f"{task_id} merge commit mismatch")
        require(item.get("task_path") == facts["task_path"], f"{task_id} task path mismatch")
        require(item.get("evidence_path") == facts["path"], f"{task_id} corrective evidence path mismatch")
        require(Path(facts["task_path"]).is_file(), f"{task_id} task file missing")
        require(Path(facts["path"]).is_file(), f"{task_id} corrective evidence file missing")
        require(item.get("included_in_release_tag") is True, f"{task_id} must be included in release tag")
        require(item.get("capability_expansion") is False, f"{task_id} corrective capability flag must stay false")
        require(item.get("runtime_behavior_changed") is False, f"{task_id} corrective runtime flag must stay false")
        require(item.get("trading_behavior_changed") is False, f"{task_id} corrective trading flag must stay false")

        for label, text in (
            ("release notes", notes_text),
            ("readiness", readiness_text),
            ("closeout", closeout_text),
        ):
            require(task_id in text, f"{label} missing {task_id}")
            require(f"#{facts['issue']}" in text, f"{label} missing issue #{facts['issue']}")
            require(f"#{facts['pr']}" in text, f"{label} missing PR #{facts['pr']}")

    commands = {
        gate.get("command")
        for gate in candidate.get("release_gates", [])
        if gate.get("required") is True
    }
    require("scripts/ai/verify_v26_1_final_scope_integration.sh" in commands, "release gate missing final scope script")

    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("v260_milestone_issue_count") == 14, "post-publication milestone count mismatch")
    require(requirements.get("final_release_scope_issue_count") == 14, "post-publication final scope count mismatch")
    require(requirements.get("corrective_issue_count") == 5, "post-publication corrective issue count mismatch")
    require(requirements.get("corrective_release_scope_closed_required") is True, "post-publication corrective closeout requirement missing")

    for text, label in ((notes_text, "release notes"), (readiness_text, "readiness"), (closeout_text, "closeout")):
        require("V260 final release scope issue count = 14" in text, f"{label} missing final issue count")
        require("V260 final release scope evidence count = 14" in text, f"{label} missing final evidence count")
        require("corrective" in text, f"{label} missing corrective scope wording")
        require("runtime behavior" in text, f"{label} missing runtime behavior boundary wording")
        require("trading behavior" in text, f"{label} missing trading behavior boundary wording")


validate(manifest, release_notes, readiness, closeout)

if os.environ.get("NTPRO_V261_FINAL_SCOPE_SELFTEST", "1") == "1":
    missing = copy.deepcopy(manifest)
    missing["v260_evidence"] = missing["v260_evidence"][:-1]
    try:
        validate(missing, release_notes, readiness, closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing corrective evidence")

    missing_scope = copy.deepcopy(manifest)
    missing_scope["corrective_release_scope"] = missing_scope["corrective_release_scope"][:-1]
    try:
        validate(missing_scope, release_notes, readiness, closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing corrective scope entry")

    stale_notes = release_notes.replace("#845", "#84X")
    try:
        validate(manifest, stale_notes, readiness, closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: stale release notes corrective issue scope")
PY

echo "v26_1_final_scope_integration status=ok final_scope_issues=14 corrective_scope=5 negative_selftest=${NTPRO_V261_FINAL_SCOPE_SELFTEST:-1}"
