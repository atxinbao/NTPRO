#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

PATCH_VERSION="${NTPRO_V211_PATCH_VERSION:-v0.21.1}"
PATCH_TAG="${NTPRO_V211_PATCH_TAG:-ntpro-rust-only-v0.21.1}"
BASE_VERSION="${NTPRO_V211_BASE_VERSION:-v0.21.0}"
BASE_TAG="${NTPRO_V211_BASE_TAG:-ntpro-rust-only-v0.21.0}"
MANIFEST_PATH="${NTPRO_V211_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_21_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V211_RELEASE_NOTES:-docs/rust-cutover/release/v0_21_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V211_READINESS_REPORT:-docs/rust-cutover/release/v0_21_1_readiness_report.md}"
BASE_MANIFEST_PATH="${NTPRO_V211_BASE_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_21_0_release_manifest.json}"
CURRENT_ISSUE="${NTPRO_V211_CURRENT_ISSUE:-682}"

fail() {
  echo "v21.1 release gate failed: $*" >&2
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

for task_id in V211-001 V211-002 V211-003 V211-004 V211-005 V211-006; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
done

for marker in \
  "Status: RELEASED" \
  "Tag: \`$PATCH_TAG\`" \
  "Release name: \`NTPRO Rust-only $PATCH_VERSION\`" \
  "Unified Read Model Foundation Hardening Patch" \
  "This patch is not the Trader Terminal workbench" \
  "This patch does not add submit capability" \
  "V211-001" \
  "V211-006" \
  "scripts/ai/verify_release.sh v21.1-release-gates" \
  "scripts/ai/verify_release.sh v21.1-strict-provenance" \
  "next capability track is \`v0.22.0\`"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$PATCH_TAG\`" \
  "Status: RELEASED" \
  "V211-001 evidence" \
  "V211-006 evidence" \
  "v0.22.0 dependency source" \
  "v0.22.0 start rule = satisfied only after all V211 issues close and v0.21.1 release evidence is published" \
  "trader_terminal_workbench_claim = false" \
  "new_submit_capability = false"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

scripts/ai/verify_release.sh v21.1-health-status-semantics
scripts/ai/verify_release.sh v21.1-read-model-projection-replay
scripts/ai/verify_release.sh v21.1-read-model-schema-boundary
scripts/ai/verify_release.sh v21.1-trader-terminal-read-model-bridge

PATCH_VERSION="$PATCH_VERSION" \
PATCH_TAG="$PATCH_TAG" \
BASE_VERSION="$BASE_VERSION" \
BASE_TAG="$BASE_TAG" \
MANIFEST_PATH="$MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
BASE_MANIFEST_PATH="$BASE_MANIFEST_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
base_manifest = json.loads(Path(os.environ["BASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")


require(manifest.get("schema_version") == "ntpro.v211_patch_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V211-006", "manifest task_id mismatch")
require(manifest.get("product_version") == os.environ["PATCH_VERSION"], "manifest product version mismatch")
require(manifest.get("release_status") == "published", "manifest status mismatch")
require(manifest.get("patch_scope") == "hardening_patch_only", "manifest patch scope mismatch")

base = manifest.get("base_release") or {}
require(base.get("tag") == os.environ["BASE_TAG"], "base tag mismatch")
require(base.get("version") == os.environ["BASE_VERSION"], "base version mismatch")
require(base.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['BASE_TAG']}", "base release URL mismatch")
require(base_manifest.get("release_status") in {"published_in_source_tree", "published_closeout_complete"}, "base manifest status mismatch")
require((base_manifest.get("published_release") or {}).get("tag") == os.environ["BASE_TAG"], "base manifest published tag mismatch")
require((base_manifest.get("next_patch_dependency") or {}).get("version") == os.environ["PATCH_VERSION"], "base manifest next patch mismatch")

planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["PATCH_TAG"], "planned patch tag mismatch")
require(planned.get("name") == f"NTPRO Rust-only {os.environ['PATCH_VERSION']}", "planned release name mismatch")
require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['PATCH_TAG']}", "planned release URL mismatch")
require(planned.get("target_commitish") == "main", "planned target_commitish mismatch")
require(planned.get("draft") is False, "planned release must not be draft")
require(planned.get("prerelease") is False, "planned release must not be prerelease")

expected = {
    "V211-001": 677,
    "V211-002": 678,
    "V211-003": 679,
    "V211-004": 680,
    "V211-005": 681,
    "V211-006": 682,
}
evidence = manifest.get("v211_evidence") or []
require(len(evidence) == len(expected), "V211 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(expected.get(task_id) == item.get("issue"), f"V211 evidence issue mismatch: {task_id}")
    path = Path(item.get("path", ""))
    require(path.is_file(), f"V211 evidence file missing: {path}")
    text = path.read_text(encoding="utf-8")
    require(task_id in text, f"V211 evidence task marker missing: {path}")

commands = {
    gate.get("command")
    for gate in manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v21-release-gates",
    "scripts/ai/verify_release.sh v21.1-health-status-semantics",
    "scripts/ai/verify_release.sh v21.1-read-model-projection-replay",
    "scripts/ai/verify_release.sh v21.1-read-model-schema-boundary",
    "scripts/ai/verify_release.sh v21.1-trader-terminal-read-model-bridge",
    "scripts/ai/verify_release.sh v21.1-release-gates",
    "scripts/ai/verify_release.sh v21.1-strict-provenance",
    "scripts/ai/verify_v21_1_release_gates.sh",
    "scripts/ai/verify_v21_1_strict_provenance.sh",
    "scripts/ai/verify_release.sh release-surface-current-guard",
    "scripts/ai/verify_release.sh release-publication-guard",
):
    require(command in commands, f"required release gate missing: {command}")

dependency = manifest.get("v220_dependency") or {}
require(dependency.get("milestone") == "v0.22.0", "v220 dependency milestone mismatch")
require(dependency.get("blocked_issues") == [683, 684, 685, 686, 687, 688, 689, 690], "v220 blocked issue set mismatch")
require(dependency.get("blocked_by_issues") == [677, 678, 679, 680, 681, 682], "v220 blocked-by issue set mismatch")
require(dependency.get("dependency_status") == "blocked_until_v0_21_1_publication", "v220 dependency status mismatch")
require("GitHub milestone description" in dependency.get("dependency_sources", []), "missing milestone dependency source")
require("V220 issue bodies" in dependency.get("dependency_sources", []), "missing issue body dependency source")
require("V220 issue comments" in dependency.get("dependency_sources", []), "missing issue comment dependency source")

for key in (
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "implicit_retry_allowed",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "retry_replace_amend_flatten_allowed",
    "strategy_driven_production_execution_allowed",
    "multi_account_execution_allowed",
    "multi_venue_execution_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "dashboard_submit_controls_enabled",
    "dashboard_replace_controls_enabled",
    "dashboard_amend_controls_enabled",
    "dashboard_flatten_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "trader_terminal_workbench_claim",
    "product_grade_trading_terminal_claim",
):
    require_false(manifest.get("boundary_flags") or {}, key)

for text, label in ((release_notes, "release notes"), (readiness, "readiness report")):
    for forbidden in (
        "new production submit capability = true",
        "production order mutation = true",
        "Trader Terminal workbench = true",
        "product-grade live trading terminal readiness = true",
        "dashboard_order_controls_enabled = true",
    ):
        require(forbidden not in text, f"{label} contains forbidden expansion wording: {forbidden}")
PY

if [[ "${NTPRO_V211_SKIP_GITHUB_DEPENDENCY:-0}" != "1" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    fail "gh is required for GitHub dependency proof"
  fi
  if ! gh auth status >/dev/null 2>&1; then
    fail "gh auth is required for GitHub dependency proof"
  fi

  gh release view "$BASE_TAG" --repo atxinbao/NTPRO --json tagName,isDraft,isPrerelease,url >/tmp/ntpro-v211-base-release.json
  BASE_TAG="$BASE_TAG" python3 <<'PY'
import json
import os
from pathlib import Path

payload = json.loads(Path("/tmp/ntpro-v211-base-release.json").read_text(encoding="utf-8"))
if payload.get("tagName") != os.environ["BASE_TAG"]:
    raise SystemExit("base release tag mismatch")
if payload.get("isDraft") or payload.get("isPrerelease"):
    raise SystemExit("base release must be final")
PY

  for issue in 677 678 679 680 681; do
    state="$(gh issue view "$issue" --repo atxinbao/NTPRO --json state --jq .state)"
    [[ "$state" == "CLOSED" ]] || fail "V211 dependency issue #$issue is not closed"
  done

  current_state="$(gh issue view "$CURRENT_ISSUE" --repo atxinbao/NTPRO --json state --jq .state)"
  if [[ "${NTPRO_V211_REQUIRE_CURRENT_ISSUE_CLOSED:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "current V211 issue #$CURRENT_ISSUE is not closed"
  fi

  gh api "repos/atxinbao/NTPRO/milestones?state=all" --paginate --jq '
    def require(cond; msg): if cond then empty else error(msg) end;
    [.[] | select(.title=="v0.21.1" or .title=="v0.22.0")] as $m |
    require(($m | length) == 2; "missing v0.21.1/v0.22.0 milestones") |
    ($m[] | select(.title=="v0.21.1")) as $v211 |
    ($m[] | select(.title=="v0.22.0")) as $v220 |
    require(($v211.description | contains("#677-#682")); "v0.21.1 milestone missing #677-#682") |
    require(($v211.description | contains("Hard-blocks v0.22.0")); "v0.21.1 milestone missing v0.22 hard block") |
    require(($v220.description | contains("#677-#682")); "v0.22.0 milestone missing V211 dependency set") |
    require(($v220.description | contains("Hard-blocked by v0.21.1")); "v0.22.0 milestone missing hard-blocked wording") |
    require(($v220.description | contains("v0.21.1 release evidence publication")); "v0.22.0 milestone missing release evidence publication rule") |
    "github milestone dependency proof ok"
  ' >/tmp/ntpro-v211-milestone-proof.txt
  cat /tmp/ntpro-v211-milestone-proof.txt

  for issue in 683 684 685 686 687 688 689 690; do
    payload="$(gh issue view "$issue" --repo atxinbao/NTPRO --json body,comments)"
    BODY_AND_COMMENTS="$payload" ISSUE="$issue" python3 <<'PY'
import json
import os

payload = json.loads(os.environ["BODY_AND_COMMENTS"])
issue = os.environ["ISSUE"]
body = payload.get("body") or ""
comments = "\n".join((comment.get("body") or "") for comment in payload.get("comments", []))
required = "#677 #678 #679 #680 #681 #682"
if required not in body:
    raise SystemExit(f"issue #{issue} body missing V211 dependency set")
if "v0.21.1 release evidence is published" not in body:
    raise SystemExit(f"issue #{issue} body missing release-evidence start rule")
if required not in comments:
    raise SystemExit(f"issue #{issue} comments missing V211 dependency set")
if "v0.21.1 release evidence is published" not in comments:
    raise SystemExit(f"issue #{issue} comments missing release-evidence publication rule")
PY
  done

  if [[ "${NTPRO_V211_REQUIRE_PUBLISHED_RELEASE:-0}" == "1" ]]; then
    gh release view "$PATCH_TAG" --repo atxinbao/NTPRO --json tagName,isDraft,isPrerelease,url \
      >/tmp/ntpro-v211-patch-release.json
    PATCH_TAG="$PATCH_TAG" python3 <<'PY'
import json
import os
from pathlib import Path

payload = json.loads(Path("/tmp/ntpro-v211-patch-release.json").read_text(encoding="utf-8"))
if payload.get("tagName") != os.environ["PATCH_TAG"]:
    raise SystemExit("patch release tag mismatch")
if payload.get("isDraft") or payload.get("isPrerelease"):
    raise SystemExit("patch release must be final")
PY
  fi
fi

echo "v21_1_release_gates status=ok patch_version=$PATCH_VERSION patch_tag=$PATCH_TAG base_tag=$BASE_TAG v211_evidence=complete v220_dependency=recorded hardening_patch_only=true new_submit_capability=false trader_terminal_workbench_claim=false current_issue_state=${current_state:-unknown}"
