#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V301_STALE_CLEANUP_REPO:-atxinbao/NTPRO}"
RELEASE_TAG="${NTPRO_V301_STALE_CLEANUP_RELEASE_TAG:-ntpro-rust-only-v0.30.0}"
FINAL_GATE_RUN="${NTPRO_V301_STALE_CLEANUP_FINAL_GATE_RUN:-29139384219}"
FINAL_TAG_SHA="${NTPRO_V301_STALE_CLEANUP_FINAL_TAG_SHA:-0f0949156401fa6e6016c0160697e7090a6da788}"
MANIFEST_PATH="${NTPRO_V301_STALE_CLEANUP_MANIFEST:-docs/rust-cutover/release/v0_30_0_release_manifest.json}"
V300_011_EVIDENCE="${NTPRO_V301_STALE_CLEANUP_V300_011:-docs/rust-cutover/evidence/V300-011.md}"
V301_TASK="${NTPRO_V301_STALE_CLEANUP_TASK:-docs/rust-cutover/tasks/V301-004.md}"
V301_EVIDENCE="${NTPRO_V301_STALE_CLEANUP_EVIDENCE:-docs/rust-cutover/evidence/V301-004.md}"
CLOSEOUT_EVIDENCE="${NTPRO_V301_STALE_CLEANUP_CLOSEOUT:-docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md}"

fail() {
  echo "v30.1 stale V300 evidence cleanup failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

for path in \
  "$MANIFEST_PATH" \
  "$V300_011_EVIDENCE" \
  "$V301_TASK" \
  "$V301_EVIDENCE" \
  "$CLOSEOUT_EVIDENCE"; do
  require_file "$path"
done

python3 - "$MANIFEST_PATH" "$V300_011_EVIDENCE" "$V301_TASK" "$V301_EVIDENCE" "$CLOSEOUT_EVIDENCE" "$RELEASE_TAG" "$FINAL_GATE_RUN" "$FINAL_TAG_SHA" <<'PY'
import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
v300_011_path = Path(sys.argv[2])
v301_task_path = Path(sys.argv[3])
v301_evidence_path = Path(sys.argv[4])
closeout_path = Path(sys.argv[5])
release_tag = sys.argv[6]
final_gate_run = sys.argv[7]
final_tag_sha = sys.argv[8]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


v300_011 = v300_011_path.read_text(encoding="utf-8")
v301_task = v301_task_path.read_text(encoding="utf-8")
v301_evidence = v301_evidence_path.read_text(encoding="utf-8")
closeout = closeout_path.read_text(encoding="utf-8")

require_contains(v300_011, "Status: PUBLISHED RELEASE CONTEXTUALIZED", "V300-011 evidence")
require("Status: LOCAL VALIDATION PASSED" not in v300_011, "V300-011 still presents local validation as final status")
require_contains(v300_011, "original validation phase = V300-011 PR-mode / pre-tag / pre-publication", "V300-011 evidence")
require_contains(v300_011, "original release gate current_issue_state=OPEN = historical current issue exception only", "V300-011 evidence")
require_contains(v300_011, "original strict provenance tag_exists=false source_dirty=false = historical pre-tag output only", "V300-011 evidence")
require_contains(v300_011, f"final hosted tag gate run = {final_gate_run}", "V300-011 evidence")
require_contains(v300_011, "final hosted tag gate conclusion = success", "V300-011 evidence")
require_contains(v300_011, "final hosted tag gate jobs = 92/92 success", "V300-011 evidence")
require_contains(v300_011, f"final hosted tag gate head SHA = {final_tag_sha}", "V300-011 evidence")
require_contains(v300_011, "final strict provenance = tag_exists=true source_dirty=false", "V300-011 evidence")
require_contains(v300_011, "published release closeout = docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md", "V300-011 evidence")
require_contains(v300_011, f"published release URL = https://github.com/atxinbao/NTPRO/releases/tag/{release_tag}", "V300-011 evidence")
require_contains(v300_011, "published at = 2026-07-11T05:37:06Z", "V300-011 evidence")
require_contains(v300_011, "release publish after gate current-release binding = pass", "V300-011 evidence")
require_contains(v300_011, "published_release manifest field populated = true", "V300-011 evidence")
require_contains(v300_011, "post_publication_closeout manifest field populated = true", "V300-011 evidence")
require_contains(v300_011, "Published Release Context", "V300-011 evidence")

for stale in [
    "v300_issue_scope=pr_mode closed=11/12 current_issue_state=OPEN",
    "release_publication_guard=offline_skip reason=missing_local_git_tag:ntpro-rust-only-v0.30.0",
    "v30 release gate failed: V300 issue must be closed before tag gate: #980 state=OPEN",
]:
    if stale in v300_011:
        require("original V300-011 PR-mode" in v300_011, f"stale marker is not contextualized: {stale}")

for marker in [
    "GitHub issue: `#1002`",
    "Clean stale pre-tag and local-validation semantics from V300-011",
    "backend go-live;",
    "product-grade live trading claim.",
]:
    require_contains(v301_task, marker, "V301-004 task")

for marker in [
    "Task: `V301-004` / GitHub issue `#1002`",
    "Status: LOCAL AND LIVE VALIDATION PASS",
    "V300-011 stale local status removed = true",
    "V300-011 pre-tag strict provenance retained as historical = true",
    "V300-011 PR-mode open issue retained as historical = true",
    f"final hosted tag gate run = {final_gate_run}",
    "final strict provenance output = tag_exists=true source_dirty=false",
]:
    require_contains(v301_evidence, marker, "V301-004 evidence")

for marker in [
    f"release tag = {release_tag}",
    f"hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/{final_gate_run}",
    "hosted release gate conclusion = success",
    "hosted release gate jobs = 92/92 success",
    "release publish after gate current-release binding = pass",
    "published_release manifest field populated = true",
    "post_publication_closeout manifest field populated = true",
]:
    require_contains(closeout, marker, "v0.30.0 closeout evidence")

manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
cleanup = manifest.get("post_release_stale_v300_evidence_cleanup") or {}
require(cleanup.get("task_id") == "V301-004", "manifest cleanup task mismatch")
require(cleanup.get("issue") == 1002, "manifest cleanup issue mismatch")
require(cleanup.get("release_tag") == release_tag, "manifest cleanup release tag mismatch")
require(cleanup.get("final_hosted_tag_gate_run_id") == int(final_gate_run), "manifest final gate run mismatch")
require(cleanup.get("final_hosted_tag_gate_conclusion") == "success", "manifest final gate conclusion mismatch")
require(cleanup.get("final_hosted_tag_gate_jobs") == "92/92 success", "manifest final gate jobs mismatch")
require(cleanup.get("final_hosted_tag_gate_head_sha") == final_tag_sha, "manifest final gate sha mismatch")
require(cleanup.get("final_strict_provenance", {}).get("tag_exists") is True, "manifest final tag_exists mismatch")
require(cleanup.get("final_strict_provenance", {}).get("source_dirty") is False, "manifest final source_dirty mismatch")
require(cleanup.get("pre_tag_outputs_retained_as_historical") is True, "manifest historical pre-tag classification mismatch")
require(cleanup.get("pr_mode_open_issue_context_historical_only") is True, "manifest PR-mode classification mismatch")
require(cleanup.get("missing_tag_or_offline_publication_guard_historical_only") is True, "manifest missing-tag classification mismatch")
require(cleanup.get("runtime_behavior_changed") is False, "manifest runtime behavior must not change")
require(cleanup.get("trading_behavior_changed") is False, "manifest trading behavior must not change")
PY

live_mode="${NTPRO_V301_STALE_CLEANUP_REQUIRE_LIVE:-0}"
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
if run.get("jobs_total") != 92 or run.get("jobs_success") != 92:
    raise SystemExit(run)
PY
fi

echo "v30_1_stale_v300_evidence_cleanup status=ok release_tag=$RELEASE_TAG final_gate_run=$FINAL_GATE_RUN tag_sha=$FINAL_TAG_SHA tag_exists=true source_dirty=false live=$live_mode"
