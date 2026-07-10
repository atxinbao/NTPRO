#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V291_STALE_CLEANUP_REPO:-atxinbao/NTPRO}"
RELEASE_TAG="${NTPRO_V291_STALE_CLEANUP_RELEASE_TAG:-ntpro-rust-only-v0.29.0}"
FINAL_GATE_RUN="${NTPRO_V291_STALE_CLEANUP_FINAL_GATE_RUN:-29091765148}"
FINAL_TAG_SHA="${NTPRO_V291_STALE_CLEANUP_FINAL_TAG_SHA:-85110d29867763f8d3b6395f4ff8154378b475b9}"
MANIFEST_PATH="${NTPRO_V291_STALE_CLEANUP_MANIFEST:-docs/rust-cutover/release/v0_29_0_release_manifest.json}"
V290_010_EVIDENCE="${NTPRO_V291_STALE_CLEANUP_V290_010:-docs/rust-cutover/evidence/V290-010.md}"
V290_011_EVIDENCE="${NTPRO_V291_STALE_CLEANUP_V290_011:-docs/rust-cutover/evidence/V290-011.md}"
V291_TASK="${NTPRO_V291_STALE_CLEANUP_TASK:-docs/rust-cutover/tasks/V291-003.md}"
V291_EVIDENCE="${NTPRO_V291_STALE_CLEANUP_EVIDENCE:-docs/rust-cutover/evidence/V291-003.md}"
CLOSEOUT_EVIDENCE="${NTPRO_V291_STALE_CLEANUP_CLOSEOUT:-docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md}"

fail() {
  echo "v29.1 stale V290 evidence cleanup failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

for path in \
  "$MANIFEST_PATH" \
  "$V290_010_EVIDENCE" \
  "$V290_011_EVIDENCE" \
  "$V291_TASK" \
  "$V291_EVIDENCE" \
  "$CLOSEOUT_EVIDENCE"; do
  require_file "$path"
done

python3 - "$MANIFEST_PATH" "$V290_010_EVIDENCE" "$V290_011_EVIDENCE" "$V291_TASK" "$V291_EVIDENCE" "$CLOSEOUT_EVIDENCE" "$RELEASE_TAG" "$FINAL_GATE_RUN" "$FINAL_TAG_SHA" <<'PY'
import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
v290_010_path = Path(sys.argv[2])
v290_011_path = Path(sys.argv[3])
v291_task_path = Path(sys.argv[4])
v291_evidence_path = Path(sys.argv[5])
closeout_path = Path(sys.argv[6])
release_tag = sys.argv[7]
final_gate_run = sys.argv[8]
final_tag_sha = sys.argv[9]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def validate_v290(text: str, task_id: str, issue: str, label: str) -> None:
    require_contains(text, "Status: PUBLISHED RELEASE CONTEXTUALIZED", label)
    require("Status: LOCAL VALIDATION PASS" not in text, f"{label} still presents local validation as final status")
    require_contains(text, f"original validation phase = {task_id} PR-mode / pre-tag / pre-publication", label)
    require_contains(text, "original strict provenance tag_exists=false source_dirty=true = historical pre-tag output only", label)
    require_contains(text, f"final hosted tag gate run = {final_gate_run}", label)
    require_contains(text, "final hosted tag gate conclusion = success", label)
    require_contains(text, "final hosted tag gate jobs = 88/88 success", label)
    require_contains(text, f"final hosted tag gate head SHA = {final_tag_sha}", label)
    require_contains(text, "final strict provenance = tag_exists=true source_dirty=false", label)
    require_contains(text, "published release closeout = docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md", label)
    require_contains(text, f"v29_release_gates=pass release_tag={release_tag} final_scope_issues=12 final_scope_evidence=12 negative_selftest=1", label)
    require_contains(text, f"v29_strict_provenance status=ok release_tag={release_tag} tag_exists=true source_dirty=false", label)
    require_contains(text, "release publish after gate current-release binding = pass", label)
    require_contains(text, "historical fixture-only current-release proof allowed = false", label)
    require_contains(text, f"Task: `{task_id}` / GitHub issue `{issue}`", label)
    require_contains(text, "Published Release Context", label)

    if "tag_exists=false source_dirty=true" in text:
        require_contains(text, "(original " + task_id + " PR-mode / pre-tag)", label)
        require_contains(text, "historical pre-tag output only", label)


v290_010 = v290_010_path.read_text(encoding="utf-8")
v290_011 = v290_011_path.read_text(encoding="utf-8")
v291_task = v291_task_path.read_text(encoding="utf-8")
v291_evidence = v291_evidence_path.read_text(encoding="utf-8")
closeout = closeout_path.read_text(encoding="utf-8")

validate_v290(v290_010, "V290-010", "#936", "V290-010 evidence")
validate_v290(v290_011, "V290-011", "#961", "V290-011 evidence")
require_contains(v290_011, "failed hosted release gate role = historical corrective context only", "V290-011 evidence")
require_contains(v290_011, "failed hosted release gate run = 29086590411", "V290-011 evidence")

for marker in [
    "Task: `V291-003` / GitHub issue `#965`",
    "Status: LOCAL VALIDATION PASS",
    "V290-010 stale local status removed = true",
    "V290-011 stale local status removed = true",
    "final hosted tag gate run = 29091765148",
    "final strict provenance output = tag_exists=true source_dirty=false",
    "failed hosted release gate 29086590411 = historical corrective context only",
]:
    require_contains(v291_evidence, marker, "V291-003 evidence")

for marker in [
    "GitHub issue: `#965`",
    "Remove stale pre-publication language from V290-010 and V290-011 evidence.",
    "backend go-live;",
    "product-grade live trading claim.",
]:
    require_contains(v291_task, marker, "V291-003 task")

for marker in [
    f"release tag = {release_tag}",
    f"hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/{final_gate_run}",
    "hosted release gate conclusion = success",
    "hosted release gate jobs = 88/88 success",
    "release publish after gate current-release binding = pass",
]:
    require_contains(closeout, marker, "v0.29.0 closeout evidence")

manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
cleanup = manifest.get("post_release_stale_v290_evidence_cleanup") or {}
require(cleanup.get("task_id") == "V291-003", "manifest cleanup task mismatch")
require(cleanup.get("issue") == 965, "manifest cleanup issue mismatch")
require(cleanup.get("release_tag") == release_tag, "manifest cleanup release tag mismatch")
require(cleanup.get("final_hosted_tag_gate_run_id") == int(final_gate_run), "manifest final gate run mismatch")
require(cleanup.get("final_hosted_tag_gate_conclusion") == "success", "manifest final gate conclusion mismatch")
require(cleanup.get("final_hosted_tag_gate_jobs") == "88/88 success", "manifest final gate jobs mismatch")
require(cleanup.get("final_hosted_tag_gate_head_sha") == final_tag_sha, "manifest final gate sha mismatch")
require(cleanup.get("final_strict_provenance", {}).get("tag_exists") is True, "manifest final tag_exists mismatch")
require(cleanup.get("final_strict_provenance", {}).get("source_dirty") is False, "manifest final source_dirty mismatch")
require(cleanup.get("pre_tag_outputs_retained_as_historical") is True, "manifest historical pre-tag classification mismatch")
require(cleanup.get("failed_hosted_gate_29086590411_historical_only") is True, "manifest failed gate classification mismatch")
require(cleanup.get("runtime_behavior_changed") is False, "manifest runtime behavior must not change")
require(cleanup.get("trading_behavior_changed") is False, "manifest trading behavior must not change")
PY

live_mode="${NTPRO_V291_STALE_CLEANUP_REQUIRE_LIVE:-0}"
if [[ "$live_mode" == "1" ]]; then
  command -v gh >/dev/null 2>&1 || fail "gh is required for live stale cleanup proof"
  gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live stale cleanup proof"
  run_json="$(gh run view "$FINAL_GATE_RUN" --repo "$REPO" --json status,conclusion,headSha,workflowName,jobs,url --jq '{status,conclusion,headSha,workflowName,url,jobs_total:(.jobs|length),jobs_success:([.jobs[]|select(.conclusion=="success")]|length)}')" || fail "could not read hosted gate run $FINAL_GATE_RUN"
  RUN_JSON="$run_json" FINAL_TAG_SHA="$FINAL_TAG_SHA" python3 <<'PY'
import json
import os

run = json.loads(os.environ["RUN_JSON"])
if run.get("status") != "completed":
    raise SystemExit(run)
if run.get("conclusion") != "success":
    raise SystemExit(run)
if run.get("workflowName") != "Rust Cutover Release Gate":
    raise SystemExit(run)
if run.get("headSha") != os.environ["FINAL_TAG_SHA"]:
    raise SystemExit(run)
if run.get("jobs_total") != 88 or run.get("jobs_success") != 88:
    raise SystemExit(run)
PY
fi

echo "v29_1_stale_v290_evidence_cleanup status=ok release_tag=$RELEASE_TAG final_gate_run=$FINAL_GATE_RUN tag_sha=$FINAL_TAG_SHA tag_exists=true source_dirty=false live=$live_mode"
