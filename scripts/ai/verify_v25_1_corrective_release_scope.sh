#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

STRICT_MANIFEST="${NTPRO_V251_CORRECTIVE_STRICT_MANIFEST:-target/ntpro-v250/v0_25_0_strict_release_manifest.json}"

fail() {
  echo "v25.1 corrective release scope failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

for path in \
  docs/rust-cutover/tasks/V251-002.md \
  docs/rust-cutover/evidence/V251-002.md \
  docs/rust-cutover/tasks/V250-009.md \
  docs/rust-cutover/evidence/V250-009.md \
  docs/rust-cutover/release/v0_25_0_release_manifest.json \
  docs/rust-cutover/release/v0_25_0_release_notes.md \
  docs/rust-cutover/release/v0_25_0_readiness_report.md \
  scripts/ai/verify_v25_release_gates.sh \
  scripts/ai/verify_v25_strict_provenance.sh; do
  require_file "$path"
done

NTPRO_RELEASE_GATE=0 \
  NTPRO_V250_RELEASE_REQUIRE_CLOSEOUT=1 \
  NTPRO_V250_RELEASE_SKIP_CURRENT_SURFACE_GUARD=1 \
  scripts/ai/verify_release.sh v25-release-gates
NTPRO_RELEASE_GATE=0 \
  scripts/ai/verify_release.sh v25-strict-provenance

require_file "$STRICT_MANIFEST"

STRICT_MANIFEST="$STRICT_MANIFEST" python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["STRICT_MANIFEST"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


inputs = {item.get("path") for item in manifest.get("release_inputs") or []}
for path in (
    "docs/rust-cutover/tasks/V250-009.md",
    "docs/rust-cutover/evidence/V250-009.md",
):
    require(path in inputs, f"strict manifest missing corrective input: {path}")

evidence = manifest.get("v250_evidence") or []
require(any(item.get("task_id") == "V250-009" and item.get("issue") == 804 for item in evidence), "strict manifest missing V250-009 evidence")

scope = manifest.get("release_scope") or {}
require(scope.get("milestone_issue_count") == 9, "milestone issue count mismatch")
require(scope.get("corrective_issue_count") == 1, "corrective issue count mismatch")
require(scope.get("final_release_scope_issue_count") == 10, "final release scope issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 10, "final release scope evidence count mismatch")
require(scope.get("corrective_scope_expands_capability") is False, "corrective scope must not expand capability")

corrective = manifest.get("corrective_release_scope") or {}
require(corrective.get("task_id") == "V250-009", "corrective task mismatch")
require(corrective.get("issue") == 804, "corrective issue mismatch")
require(corrective.get("pull_request") == 805, "corrective PR mismatch")
require(corrective.get("failed_release_gate_run") == 28762387835, "corrective failed run mismatch")
require(corrective.get("final_success_release_gate_run") == 28764231552, "corrective final run mismatch")
require(corrective.get("merge_commit") == "eedcdab1d3ca85d6f51b368b5f36208a7b591026", "corrective merge commit mismatch")
require(corrective.get("included_in_release_tag") is True, "corrective scope must be included in release tag")
require(corrective.get("capability_expansion") is False, "corrective scope must not expand capability")
PY

echo "v25_1_corrective_release_scope status=ok final_scope_issues=10 corrective_issue=804 corrective_pr=805 strict_manifest=$STRICT_MANIFEST"
