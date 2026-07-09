#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V29_INTAKE_REPO:-atxinbao/NTPRO}"
V281_RELEASE_VERSION="${NTPRO_V29_INTAKE_V281_RELEASE_VERSION:-v0.28.1}"
V281_RELEASE_TAG="${NTPRO_V29_INTAKE_V281_RELEASE_TAG:-ntpro-rust-only-v0.28.1}"
V281_RELEASE_NAME="${NTPRO_V29_INTAKE_V281_RELEASE_NAME:-NTPRO Rust-only v0.28.1}"
V281_RELEASE_URL="${NTPRO_V29_INTAKE_V281_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.1}"
V281_MANIFEST_PATH="${NTPRO_V29_INTAKE_V281_MANIFEST:-docs/rust-cutover/release/v0_28_1_release_manifest.json}"
V281_READINESS_PATH="${NTPRO_V29_INTAKE_V281_READINESS:-docs/rust-cutover/release/v0_28_1_readiness_report.md}"
V281_RELEASE_NOTES_PATH="${NTPRO_V29_INTAKE_V281_NOTES:-docs/rust-cutover/release/v0_28_1_release_notes.md}"
V281_CLOSEOUT_PATH="${NTPRO_V29_INTAKE_V281_CLOSEOUT:-docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md}"
ALLOW_UNPUBLISHED="${NTPRO_V29_INTAKE_ALLOW_UNPUBLISHED:-0}"

fail() {
  echo "v29 intake gate failed: $*" >&2
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
  "$V281_MANIFEST_PATH" \
  "$V281_READINESS_PATH" \
  "$V281_RELEASE_NOTES_PATH" \
  "$V281_CLOSEOUT_PATH" \
  scripts/ai/verify_v28_1_release_gates.sh \
  scripts/ai/verify_v28_1_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V281-001 V281-002 V281-003 V281-004 V281-005 V281-006 V281-007 V281-008 V281-009; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
done

V281_RELEASE_VERSION="$V281_RELEASE_VERSION" \
V281_RELEASE_TAG="$V281_RELEASE_TAG" \
V281_RELEASE_NAME="$V281_RELEASE_NAME" \
V281_RELEASE_URL="$V281_RELEASE_URL" \
V281_MANIFEST_PATH="$V281_MANIFEST_PATH" \
V281_READINESS_PATH="$V281_READINESS_PATH" \
V281_RELEASE_NOTES_PATH="$V281_RELEASE_NOTES_PATH" \
V281_CLOSEOUT_PATH="$V281_CLOSEOUT_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["V281_MANIFEST_PATH"]).read_text(encoding="utf-8"))
readiness = Path(os.environ["V281_READINESS_PATH"]).read_text(encoding="utf-8")
notes = Path(os.environ["V281_RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["V281_CLOSEOUT_PATH"]).read_text(encoding="utf-8")
expected = {
    "V281-001": 919,
    "V281-002": 920,
    "V281-003": 921,
    "V281-004": 922,
    "V281-005": 923,
    "V281-006": 924,
    "V281-007": 925,
    "V281-008": 944,
    "V281-009": 946,
}

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

require(manifest.get("schema_version") == "ntpro.v281_patch_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V281-009", "manifest task mismatch")
require(manifest.get("product_version") == os.environ["V281_RELEASE_VERSION"], "manifest product version mismatch")
planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["V281_RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == os.environ["V281_RELEASE_NAME"], "planned release name mismatch")
require(planned.get("github_release_url") == os.environ["V281_RELEASE_URL"], "planned release URL mismatch")
evidence = manifest.get("v281_evidence") or []
require(len(evidence) == 9, "V281 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(expected.get(task_id) == item.get("issue"), f"V281 issue mismatch: {task_id}")
    require(Path(item.get("path", "")).is_file(), f"missing V281 evidence: {item}")
scope = manifest.get("release_scope") or {}
require(scope.get("final_release_scope_issue_count") == 9, "V281 final issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 9, "V281 final evidence count mismatch")
require(scope.get("exact_milestone_issue_numbers") == list(expected.values()), "V281 exact issue numbers mismatch")
requirements = manifest.get("post_publication_requirements") or {}
require(requirements.get("github_release_published_required") is True, "GitHub release requirement missing")
require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
require(requirements.get("v0_29_start_gate_fails_without_v281_release_evidence") is True, "v29 hard-block requirement missing")
next_tracks = manifest.get("next_tracks") or {}
require(next_tracks.get("capability") == "v0.29.0", "next capability mismatch")
require(next_tracks.get("start_gate") == "blocked_until_v281_release_evidence_published", "next start gate mismatch")
for marker in (
    "v0.29.0 start gate = blocked until v0.28.1 release evidence is published",
    "source-controlled closeout evidence = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md",
):
    require(marker in readiness, f"missing readiness marker: {marker}")
require("v29 intake gate = hard-blocked until v0.28.1 publication evidence exists" in notes, "notes hard-block marker missing")
for marker in (
    "Status: PENDING PUBLICATION",
    "publication status = pending_publication",
    "generated publication evidence sole proof allowed = false",
    "v0.29.0 intake requires this source-controlled closeout evidence = true",
):
    require(marker in closeout, f"missing closeout target marker: {marker}")
for key in (
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
):
    require((manifest.get("boundary_flags") or {}).get(key) is False, f"v28.1 boundary must remain false: {key}")
PY

if [[ "$ALLOW_UNPUBLISHED" == "1" ]]; then
  echo "v29_intake_gate=blocked_until_v28_1_publication source_gate=ok release_tag=$V281_RELEASE_TAG"
  exit 0
fi

command -v gh >/dev/null 2>&1 || fail "gh is required for live v29 intake proof"
gh_with_retry auth status >/dev/null 2>&1 || fail "gh authentication is required for live v29 intake proof"

release_json="$(gh_with_retry api "/repos/$REPO/releases/tags/$V281_RELEASE_TAG")" || fail "missing GitHub Release for $V281_RELEASE_TAG"
RELEASE_JSON="$release_json" V281_RELEASE_TAG="$V281_RELEASE_TAG" V281_RELEASE_NAME="$V281_RELEASE_NAME" python3 <<'PY'
import json
import os

release = json.loads(os.environ["RELEASE_JSON"])
if release.get("tag_name") != os.environ["V281_RELEASE_TAG"]:
    raise SystemExit("release tag mismatch")
if release.get("name") != os.environ["V281_RELEASE_NAME"]:
    raise SystemExit("release name mismatch")
if release.get("draft") is not False or release.get("prerelease") is not False:
    raise SystemExit("release must be public, non-draft, and non-prerelease")
print("v29_intake_gate=pass release_tag=" + os.environ["V281_RELEASE_TAG"])
PY
