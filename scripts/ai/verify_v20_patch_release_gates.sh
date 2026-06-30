#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

PATCH_VERSION="${NTPRO_V201_PATCH_VERSION:-v0.20.1}"
PATCH_TAG="${NTPRO_V201_PATCH_TAG:-ntpro-rust-only-v0.20.1}"
BASE_TAG="${NTPRO_V201_BASE_TAG:-ntpro-rust-only-v0.20.0}"
MANIFEST_PATH="${NTPRO_V201_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_20_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V201_RELEASE_NOTES:-docs/rust-cutover/release/v0_20_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V201_READINESS_REPORT:-docs/rust-cutover/release/v0_20_1_readiness_report.md}"
BASE_MANIFEST_PATH="${NTPRO_V201_BASE_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_20_0_release_manifest.json}"

fail() {
  echo "v20.1 patch release gate failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local needle="$2"
  if ! grep -F -- "$needle" "$path" >/dev/null; then
    fail "missing marker in $path: $needle"
  fi
}

require_file "$MANIFEST_PATH"
require_file "$RELEASE_NOTES_PATH"
require_file "$READINESS_REPORT_PATH"
require_file "$BASE_MANIFEST_PATH"

for task_id in V201-001 V201-002 V201-003 V201-004 V201-005 V201-006 V201-007; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
done

for marker in \
  "Status: RELEASED" \
  "Tag: \`$PATCH_TAG\`" \
  "Release name: \`NTPRO Rust-only $PATCH_VERSION\`" \
  "Production Order Lifecycle Release Closeout & Provenance Hardening Patch" \
  "This patch does not expand production submit capability" \
  "product-grade live trading terminal readiness" \
  "scripts/ai/verify_release.sh v20.1-release-gates"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$PATCH_TAG\`" \
  "Status: RELEASED" \
  "V201-001 evidence" \
  "V201-007 evidence" \
  "v0.21.0 blocked-by source" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

PATCH_VERSION="$PATCH_VERSION" \
PATCH_TAG="$PATCH_TAG" \
BASE_TAG="$BASE_TAG" \
MANIFEST_PATH="$MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
BASE_MANIFEST_PATH="$BASE_MANIFEST_PATH" \
python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
base_manifest_path = pathlib.Path(os.environ["BASE_MANIFEST_PATH"])
release_notes_path = pathlib.Path(os.environ["RELEASE_NOTES_PATH"])
readiness_report_path = pathlib.Path(os.environ["READINESS_REPORT_PATH"])

manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
base_manifest = json.loads(base_manifest_path.read_text(encoding="utf-8"))
release_notes = release_notes_path.read_text(encoding="utf-8")
readiness_report = readiness_report_path.read_text(encoding="utf-8")

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")

patch_version = os.environ["PATCH_VERSION"]
patch_tag = os.environ["PATCH_TAG"]
base_tag = os.environ["BASE_TAG"]

require(manifest.get("schema_version") == "ntpro.v201_patch_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V201-007", "manifest task_id mismatch")
require(manifest.get("product_version") == patch_version, "manifest product version mismatch")
require(manifest.get("patch_scope") == "hardening_patch_only", "manifest patch scope mismatch")
require(manifest.get("release_status") == "published", "manifest status mismatch")

base_release = manifest.get("base_release") or {}
require(base_release.get("tag") == base_tag, "base release tag mismatch")
require(base_release.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{base_tag}", "base release URL mismatch")
require(base_manifest.get("release_status") == "published", "base release manifest must remain published")
require((base_manifest.get("planned_release") or {}).get("tag") == base_tag, "base manifest planned tag mismatch")
require((base_manifest.get("source_provenance") or {}).get("actual_source_tree"), "base source tree provenance missing")

planned = manifest.get("planned_release") or {}
require(planned.get("tag") == patch_tag, "planned patch tag mismatch")
require(planned.get("name") == f"NTPRO Rust-only {patch_version}", "planned release name mismatch")
require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{patch_tag}", "planned release URL mismatch")
require(planned.get("target_commitish") == "main", "planned target_commitish mismatch")
require(planned.get("draft") is False, "planned release must not be draft")
require(planned.get("prerelease") is False, "planned release must not be prerelease")

evidence = manifest.get("v201_evidence") or []
expected = {
    "V201-001": 644,
    "V201-002": 645,
    "V201-003": 650,
    "V201-004": 646,
    "V201-005": 647,
    "V201-006": 648,
    "V201-007": 649,
}
require(len(evidence) == len(expected), "V201 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(expected.get(task_id) == item.get("issue"), f"V201 evidence issue mismatch: {task_id}")
    path = pathlib.Path(item.get("path", ""))
    require(path.is_file(), f"V201 evidence file missing: {path}")
    text = path.read_text(encoding="utf-8")
    require(task_id in text, f"V201 evidence task marker missing: {path}")

commands = {
    gate.get("command")
    for gate in manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v20-release-gates",
    "scripts/ai/verify_release.sh v20-strict-provenance",
    "scripts/ai/verify_release.sh v20.1-release-gates",
    "scripts/ai/verify_v20_patch_release_gates.sh",
    "scripts/ai/verify_release.sh release-surface-current-guard",
    "scripts/ai/verify_release.sh release-publication-guard",
):
    require(command in commands, f"required release gate missing: {command}")

for key in (
    "new_submit_capability",
    "implicit_retry_allowed",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "bulk_order_allowed",
    "retry_replace_amend_flatten_allowed",
    "strategy_driven_production_execution_allowed",
    "multi_account_execution_allowed",
    "multi_venue_execution_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "product_grade_trading_terminal_claim",
):
    require_false(manifest.get("boundary_flags") or {}, key)

dependency = manifest.get("v21_dependency") or {}
require(dependency.get("milestone") == "v0.21.0", "v21 dependency milestone mismatch")
require(dependency.get("issues") == list(range(651, 660)), "v21 issue set mismatch")
require(dependency.get("blocked_by_milestone") == "v0.20.1", "v21 blocked-by milestone mismatch")
require(dependency.get("blocked_by_issues") == [644, 645, 646, 647, 648, 649, 650], "v21 blocked-by issue set mismatch")

for text, label in ((release_notes, "release notes"), (readiness_report, "readiness report")):
    for forbidden in (
        "new production submit capability = true",
        "implicit retry allowed",
        "automatic cancel allowed",
        "automatic remediation allowed",
        "product-grade live trading terminal readiness = true",
    ):
        require(forbidden not in text, f"{label} contains forbidden expansion wording: {forbidden}")
PY

if [[ "${NTPRO_V201_SKIP_GITHUB_DEPENDENCY:-0}" != "1" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    fail "gh is required for GitHub dependency proof"
  fi
  if ! gh auth status >/dev/null 2>&1; then
    fail "gh auth is required for GitHub dependency proof"
  fi

  gh api repos/atxinbao/NTPRO/milestones --paginate --jq '
    def require(cond; msg): if cond then empty else error(msg) end;
    [.[] | select(.title=="v0.20.1" or .title=="v0.21.0")] as $m |
    require(($m | length) == 2; "missing v0.20.1/v0.21.0 milestones") |
    ($m[] | select(.title=="v0.20.1")) as $v201 |
    ($m[] | select(.title=="v0.21.0")) as $v210 |
    require(($v201.description | contains("#644-#650")); "v0.20.1 milestone missing #644-#650") |
    require(($v201.description | contains("Hard-blocks v0.21.0")); "v0.20.1 milestone missing v0.21 block") |
    require(($v210.description | contains("#651-#659")); "v0.21.0 milestone missing #651-#659") |
    require(($v210.description | contains("Hard-blocked by v0.20.1")); "v0.21.0 milestone missing v0.20.1 dependency") |
    "github milestone dependency proof ok"
  ' >/tmp/ntpro-v201-milestone-proof.txt
  cat /tmp/ntpro-v201-milestone-proof.txt

  for issue in 651 652 653 654 655 656 657 658 659; do
    payload="$(gh issue view "$issue" --repo atxinbao/NTPRO --json body,comments)"
    BODY_AND_COMMENTS="$payload" ISSUE="$issue" python3 <<'PY'
import json
import os

payload = json.loads(os.environ["BODY_AND_COMMENTS"])
issue = os.environ["ISSUE"]
body = payload.get("body") or ""
comments = "\n".join((comment.get("body") or "") for comment in payload.get("comments", []))
required = "#644 #645 #646 #647 #648 #649 #650"
if required not in body:
    raise SystemExit(f"issue #{issue} body missing V201 dependency set")
if "v0.20.1 release evidence is available" not in body:
    raise SystemExit(f"issue #{issue} body missing release-evidence start rule")
if required not in comments:
    raise SystemExit(f"issue #{issue} comments missing V201 dependency set")
if "v0.20.1 release evidence is published" not in comments:
    raise SystemExit(f"issue #{issue} comments missing release-evidence publication rule")
PY
  done
fi

echo "v20_patch_release_gates status=ok patch_version=$PATCH_VERSION patch_tag=$PATCH_TAG base_tag=$BASE_TAG v201_evidence=complete v21_dependency=recorded hardening_patch_only=true new_submit_capability=false"
