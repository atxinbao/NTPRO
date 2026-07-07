#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V27_INTAKE_REPO:-atxinbao/NTPRO}"
V261_RELEASE_VERSION="${NTPRO_V27_INTAKE_V261_RELEASE_VERSION:-v0.26.1}"
V261_RELEASE_TAG="${NTPRO_V27_INTAKE_V261_RELEASE_TAG:-ntpro-rust-only-v0.26.1}"
V261_RELEASE_NAME="${NTPRO_V27_INTAKE_V261_RELEASE_NAME:-NTPRO Rust-only v0.26.1}"
V261_RELEASE_URL="${NTPRO_V27_INTAKE_V261_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.26.1}"
V261_MANIFEST_PATH="${NTPRO_V27_INTAKE_V261_MANIFEST:-docs/rust-cutover/release/v0_26_1_release_manifest.json}"
V261_READINESS_PATH="${NTPRO_V27_INTAKE_V261_READINESS:-docs/rust-cutover/release/v0_26_1_readiness_report.md}"
V261_RELEASE_NOTES_PATH="${NTPRO_V27_INTAKE_V261_NOTES:-docs/rust-cutover/release/v0_26_1_release_notes.md}"
V270_INTAKE_PATH="${NTPRO_V27_INTAKE_REPORT:-docs/rust-cutover/release/v0_27_0_intake_gate.md}"
V261_MILESTONE_NUMBER="${NTPRO_V27_INTAKE_V261_MILESTONE_NUMBER:-19}"

fail() {
  echo "v27 intake gate failed: $*" >&2
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

gh_auth_status_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if gh auth status >/dev/null 2>&1; then
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

for path in "$V261_MANIFEST_PATH" "$V261_READINESS_PATH" "$V261_RELEASE_NOTES_PATH" "$V270_INTAKE_PATH"; do
  require_file "$path"
done

for task_id in V261-001 V261-002 V261-003 V261-004 V261-005 V261-006; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
done

V261_RELEASE_VERSION="$V261_RELEASE_VERSION" \
V261_RELEASE_TAG="$V261_RELEASE_TAG" \
V261_RELEASE_NAME="$V261_RELEASE_NAME" \
V261_RELEASE_URL="$V261_RELEASE_URL" \
V261_MANIFEST_PATH="$V261_MANIFEST_PATH" \
V261_READINESS_PATH="$V261_READINESS_PATH" \
V270_INTAKE_PATH="$V270_INTAKE_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["V261_MANIFEST_PATH"]).read_text(encoding="utf-8"))
readiness = Path(os.environ["V261_READINESS_PATH"]).read_text(encoding="utf-8")
intake = Path(os.environ["V270_INTAKE_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(manifest.get("schema_version") == "ntpro.v261_patch_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V261-006", "manifest task mismatch")
require(manifest.get("product_version") == os.environ["V261_RELEASE_VERSION"], "manifest product version mismatch")
planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["V261_RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == os.environ["V261_RELEASE_NAME"], "planned release name mismatch")
require(planned.get("github_release_url") == os.environ["V261_RELEASE_URL"], "planned release URL mismatch")
evidence = manifest.get("v261_evidence") or []
require(len(evidence) == 6, "V261 evidence count mismatch")
requirements = manifest.get("post_publication_requirements") or {}
require(requirements.get("all_v261_issues_closed_required") is True, "V261 closeout requirement missing")
require(requirements.get("github_release_published_required") is True, "GitHub release requirement missing")
require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
require(requirements.get("strict_release_body_match_required") is True, "strict release body requirement missing")
require(requirements.get("publication_after_hosted_gate_required") is True, "publication ordering requirement missing")
require(requirements.get("v0_27_start_gate_fails_without_v261_release_evidence") is True, "v27 hard block requirement missing")
next_tracks = manifest.get("next_tracks") or {}
require(next_tracks.get("capability") == "v0.27.0", "next capability mismatch")
require(next_tracks.get("start_gate") == "blocked_until_v261_release_evidence_published", "next start gate mismatch")
require(next_tracks.get("implementation_started") is False, "v27 implementation must not start")
require("No V270 implementation starts until all V261 issues are closed and v0.26.1" in readiness, "readiness hard-block marker missing")
for marker in (
    "start_gate_status = satisfied",
    "V261 issues closed = 6/6",
    "v0.26.1 milestone = closed",
    "v0.26.1 release evidence = published",
    "v0.26.1 hosted release gate = success",
    "v0.26.1 GitHub Release published after hosted gate = true",
    "v0.27.0 capability track = product_operations_runtime_integration_foundation_only",
    "v0.27.0 runtime capability inherited from v0.26.1 = false",
):
    require(marker in intake, f"missing v27 intake marker: {marker}")
for key in (
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
):
    require((manifest.get("boundary_flags") or {}).get(key) is False, f"boundary must remain false: {key}")
    require(f"{key} = false" in intake, f"missing v27 intake boundary marker: {key}")
PY

if [[ "${NTPRO_V27_INTAKE_ALLOW_UNPUBLISHED:-0}" == "1" ]]; then
  echo "v27_intake_gate=blocked_until_v26_1_publication source_gate=ok release_tag=$V261_RELEASE_TAG negative_selftest=${NTPRO_V27_INTAKE_SELFTEST:-1}"
  exit 0
fi

command -v gh >/dev/null 2>&1 || fail "gh is required for live intake proof"
gh_auth_status_with_retry || fail "gh authentication is required for live intake proof"

remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V261_RELEASE_TAG^{}" | awk '{print $1}')"
if [[ -z "$remote_tag_commit" ]]; then
  remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V261_RELEASE_TAG" | awk '{print $1}')"
fi
[[ -n "$remote_tag_commit" ]] || fail "missing remote release tag: $V261_RELEASE_TAG"

if git rev-parse -q --verify origin/main^{commit} >/dev/null; then
  origin_main_sha="$(git rev-parse origin/main)"
else
  git fetch --no-tags --depth=1 origin +refs/heads/main:refs/remotes/origin/main >/dev/null 2>&1 || true
  origin_main_sha="$(git rev-parse origin/main)"
fi
git merge-base --is-ancestor "$remote_tag_commit" "$origin_main_sha" || fail "v0.26.1 tag is not ancestor of origin/main: tag=$remote_tag_commit origin_main=$origin_main_sha"

release_json="$(gh_with_retry api "/repos/$REPO/releases/tags/$V261_RELEASE_TAG")" || fail "missing GitHub Release for $V261_RELEASE_TAG"
milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$V261_MILESTONE_NUMBER")" || fail "could not read v0.26.1 milestone"
run_json="$(gh_with_retry run list --repo "$REPO" --workflow release-tag.yml --commit "$remote_tag_commit" --limit 20 --json databaseId,status,conclusion,headSha,url,updatedAt,workflowName)" || fail "could not read v0.26.1 hosted gate runs"

RELEASE_JSON="$release_json" \
MILESTONE_JSON="$milestone_json" \
RUN_JSON="$run_json" \
V261_RELEASE_TAG="$V261_RELEASE_TAG" \
V261_RELEASE_NAME="$V261_RELEASE_NAME" \
V261_RELEASE_URL="$V261_RELEASE_URL" \
V261_RELEASE_NOTES_PATH="$V261_RELEASE_NOTES_PATH" \
REMOTE_TAG_COMMIT="$remote_tag_commit" \
python3 <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path

release = json.loads(os.environ["RELEASE_JSON"])
milestone = json.loads(os.environ["MILESTONE_JSON"])
runs = json.loads(os.environ["RUN_JSON"])
notes = Path(os.environ["V261_RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def normalized(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.splitlines()).strip()


def parse_time(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


require(release.get("tag_name") == os.environ["V261_RELEASE_TAG"], "release tag mismatch")
require(release.get("name") == os.environ["V261_RELEASE_NAME"], "release name mismatch")
require(release.get("html_url") == os.environ["V261_RELEASE_URL"], "release URL mismatch")
require(release.get("draft") is False, "release must not be draft")
require(release.get("prerelease") is False, "release must not be prerelease")
require(normalized(release.get("body") or "") == normalized(notes), "GitHub Release body does not strictly match v0.26.1 release notes")
body_sha = hashlib.sha256(normalized(release.get("body") or "").encode("utf-8")).hexdigest()
notes_sha = hashlib.sha256(normalized(notes).encode("utf-8")).hexdigest()
require(body_sha == notes_sha, "release body hash mismatch")

require(milestone.get("title") == "v0.26.1", "milestone title mismatch")
require(milestone.get("state") == "closed", "v0.26.1 milestone must be closed")
require(milestone.get("open_issues") == 0, "v0.26.1 milestone open issue count must be 0")
require(milestone.get("closed_issues") >= 6, "v0.26.1 milestone closed issue count must be at least 6")

matching = [
    run
    for run in runs
    if run.get("headSha") == os.environ["REMOTE_TAG_COMMIT"]
    and run.get("status") == "completed"
    and run.get("conclusion") == "success"
]
require(matching, "missing successful hosted release gate for v0.26.1 tag commit")
matching.sort(key=lambda run: run.get("updatedAt") or "", reverse=True)
gate = matching[0]
published_at = parse_time(release.get("published_at", ""))
gate_completed = parse_time(gate.get("updatedAt", ""))
require(published_at >= gate_completed, "v0.26.1 release must be published after hosted gate success")
print(f"gate_run={gate['databaseId']} gate_updated_at={gate['updatedAt']} body_sha256={body_sha}")
PY

for issue in 847 848 849 850 851 852; do
  state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
  [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before v27 intake, got $state"
done

echo "v27_intake_gate=pass v261_release_tag=$V261_RELEASE_TAG tag_sha=$remote_tag_commit v261_issues_closed=6/6 negative_selftest=${NTPRO_V27_INTAKE_SELFTEST:-1}"
