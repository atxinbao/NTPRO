#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V251_STALE_REPO:-atxinbao/NTPRO}"
RELEASE_TAG="${NTPRO_V251_STALE_RELEASE_TAG:-ntpro-rust-only-v0.25.0}"
MANIFEST_PATH="${NTPRO_V251_STALE_MANIFEST:-docs/rust-cutover/release/v0_25_0_release_manifest.json}"
V250_EVIDENCE_PATH="${NTPRO_V251_STALE_V250_EVIDENCE:-docs/rust-cutover/evidence/V250-008.md}"
V251_EVIDENCE_PATH="${NTPRO_V251_STALE_EVIDENCE:-docs/rust-cutover/evidence/V251-003.md}"
V251_TASK_PATH="${NTPRO_V251_STALE_TASK:-docs/rust-cutover/tasks/V251-003.md}"
VERIFICATION_PATH="${NTPRO_V251_STALE_VERIFICATION:-verification.md}"
MILESTONE_NUMBER="${NTPRO_V251_STALE_MILESTONE_NUMBER:-16}"

fail() {
  echo "v25.1 stale pre-tag cleanup failed: $*" >&2
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

for path in "$MANIFEST_PATH" "$V250_EVIDENCE_PATH" "$V251_EVIDENCE_PATH" "$V251_TASK_PATH" "$VERIFICATION_PATH"; do
  require_file "$path"
done

for marker in \
  "release tag = $RELEASE_TAG" \
  "release tag exists = true" \
  "#785 V250-008 = closed" \
  "#804 V250-009 corrective issue = closed" \
  "PR #805 = merged" \
  "v0.25.0 milestone state = closed" \
  "pre-release validation snapshot = historical_not_post_release_state"; do
  require_contains "$V250_EVIDENCE_PATH" "$marker"
  require_contains "$V251_EVIDENCE_PATH" "$marker"
done

python3 - "$V250_EVIDENCE_PATH" "$VERIFICATION_PATH" <<'PY'
from pathlib import Path
import sys

v250 = Path(sys.argv[1]).read_text(encoding="utf-8")
verification = Path(sys.argv[2]).read_text(encoding="utf-8")

start = verification.index("# V250-008 Verification")
end = verification.index("# V250-009 Verification", start)
v250_verification = verification[start:end]

stale_markers = [
    "tag_exists=false",
    "current_issue_state=OPEN",
    "missing tag",
    "missing_tag",
    "missing_local_git_tag",
    "offline pre-tag skip",
    "offline-only",
    "offline_skip",
    "public release publication = pending",
    "tag gate run = pending",
    "tag gate result = pending",
]

for label, text in {
    "V250-008 evidence": v250,
    "V250-008 verification section": v250_verification,
}.items():
    for marker in stale_markers:
        if marker in text:
            raise SystemExit(f"{label} contains stale marker: {marker}")

required = [
    "historical pre-release PR validation",
    "post-release current state",
    "#785 closed",
    "#804 closed",
    "PR #805 merged",
    "v0.25.0 milestone closed",
]
for marker in required:
    if marker not in v250_verification:
        raise SystemExit(f"V250-008 verification section missing marker: {marker}")
PY

MANIFEST_PATH="$MANIFEST_PATH" python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
cleanup = manifest.get("post_release_stale_pretag_cleanup") or {}
if cleanup.get("task_id") != "V251-003":
    raise SystemExit("cleanup task mismatch")
if cleanup.get("issue") != 808:
    raise SystemExit("cleanup issue mismatch")
if cleanup.get("release_tag") != "ntpro-rust-only-v0.25.0":
    raise SystemExit("cleanup release tag mismatch")
if cleanup.get("issue_785_state") != "closed":
    raise SystemExit("issue #785 state mismatch")
if cleanup.get("issue_804_state") != "closed":
    raise SystemExit("issue #804 state mismatch")
if cleanup.get("pr_805_state") != "merged":
    raise SystemExit("PR #805 state mismatch")
if cleanup.get("v0_25_0_milestone_state") != "closed":
    raise SystemExit("milestone state mismatch")
if cleanup.get("pre_release_validation_snapshot") != "historical_not_post_release_state":
    raise SystemExit("pre-release snapshot classification mismatch")
if cleanup.get("retag_required") is not False:
    raise SystemExit("retag must not be required")
if cleanup.get("runtime_behavior_changed") is not False:
    raise SystemExit("runtime behavior must not change")
if cleanup.get("trading_behavior_changed") is not False:
    raise SystemExit("trading behavior must not change")
PY

if [[ "${NTPRO_V251_STALE_SELFTEST:-1}" == "1" ]]; then
  tmp_file="$(mktemp "${TMPDIR:-/tmp}/ntpro-v251-stale.XXXXXX.md")"
  cp "$V250_EVIDENCE_PATH" "$tmp_file"
  printf '\ntag_exists=false\n' >>"$tmp_file"
  if python3 - "$tmp_file" "$VERIFICATION_PATH" <<'PY'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text(encoding="utf-8")
raise SystemExit(1 if "tag_exists=false" in text else 0)
PY
  then
    rm -f "$tmp_file"
    fail "stale marker self-test unexpectedly passed"
  fi
  rm -f "$tmp_file"
fi

if ! command -v gh >/dev/null 2>&1; then
  fail "gh is required for live stale cleanup proof"
fi
gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live stale cleanup proof"

issue_785_state="$(gh_with_retry issue view 785 --repo "$REPO" --json state --jq .state)" || fail "could not read issue #785"
[[ "$issue_785_state" == "CLOSED" ]] || fail "issue #785 must be CLOSED, got $issue_785_state"
issue_804_state="$(gh_with_retry issue view 804 --repo "$REPO" --json state --jq .state)" || fail "could not read issue #804"
[[ "$issue_804_state" == "CLOSED" ]] || fail "issue #804 must be CLOSED, got $issue_804_state"

pr_805_json="$(gh_with_retry api "/repos/$REPO/pulls/805")" || fail "could not read PR #805"
PR_805_JSON="$pr_805_json" python3 <<'PY'
import json
import os
pr = json.loads(os.environ["PR_805_JSON"])
if pr["state"] != "closed" or pr["merged_at"] != "2026-07-06T02:36:12Z":
    raise SystemExit(pr)
PY

milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read milestone #$MILESTONE_NUMBER"
MILESTONE_JSON="$milestone_json" python3 <<'PY'
import json
import os
milestone = json.loads(os.environ["MILESTONE_JSON"])
if milestone["title"] != "v0.25.0" or milestone["state"] != "closed" or milestone["open_issues"] != 0 or milestone["closed_issues"] != 9:
    raise SystemExit(milestone)
PY

echo "v25_1_stale_pretag_cleanup status=ok release_tag=$RELEASE_TAG issue_785=closed issue_804=closed pr_805=merged milestone=v0.25.0:closed stale_selftest=${NTPRO_V251_STALE_SELFTEST:-1}"
