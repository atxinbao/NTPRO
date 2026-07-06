#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V250_PRODUCT_VERSION:-v0.25.0}"
RELEASE_TAG="${NTPRO_V250_RELEASE_TAG:-ntpro-rust-only-v0.25.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.25.0"
MANIFEST_PATH="${NTPRO_V250_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v250/v0_25_0_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V250_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_25_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V250_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_25_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V250_READINESS_REPORT:-$ROOT_DIR/docs/rust-cutover/release/v0_25_0_readiness_report.md}"
BASE_RELEASE_MANIFEST_PATH="${NTPRO_V250_BASE_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_24_1_release_manifest.json}"
BASE_READINESS_PATH="${NTPRO_V250_BASE_READINESS:-$ROOT_DIR/docs/rust-cutover/release/v0_24_1_readiness_report.md}"
GATE_SCRIPT_PATH="${NTPRO_V250_GATE_SCRIPT:-$ROOT_DIR/scripts/ai/verify_v25_release_gates.sh}"
STRICT_SCRIPT_PATH="${NTPRO_V250_STRICT_SCRIPT:-$ROOT_DIR/scripts/ai/verify_v25_strict_provenance.sh}"
VERIFY_ONLY="${NTPRO_V250_STRICT_VERIFY_ONLY:-0}"

fail() {
  echo "v25 strict release provenance drift: $*" >&2
  exit 1
}

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v25 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$BASE_RELEASE_MANIFEST_PATH"
  "$BASE_READINESS_PATH"
  "$GATE_SCRIPT_PATH"
  "$STRICT_SCRIPT_PATH"
  "$ROOT_DIR/scripts/ai/verify_release.sh"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
  "$ROOT_DIR/scripts/ai/publish_ntpro_release_after_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_release_publish_after_gate.sh"
  "$ROOT_DIR/.github/workflows/release-tag.yml"
  "$ROOT_DIR/.github/workflows/release-publish.yml"
  "$ROOT_DIR/docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_intake_gate.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_monitoring_observability_contract.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_alert_taxonomy_routing.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_incident_lifecycle_acknowledgement.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_runbook_audit_evidence.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_dr_preview_drill_evidence.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_dashboard_monitoring_surface.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_slo_freshness_diagnostics_gate.md"
  "$ROOT_DIR/scripts/ai/verify_v25_intake_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_monitoring_observability_contract.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_alert_taxonomy_routing.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_incident_lifecycle_acknowledgement.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_runbook_audit_evidence.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_dr_preview_drill_evidence.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_dashboard_monitoring_surface.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_slo_freshness_diagnostics_gate.sh"
)

for task_id in V250-000 V250-001 V250-002 V250-003 V250-004 V250-005 V250-006 V250-007 V250-008 V250-009; do
  input_paths+=("$ROOT_DIR/docs/rust-cutover/evidence/${task_id}.md")
  input_paths+=("$ROOT_DIR/docs/rust-cutover/tasks/${task_id}.md")
done

for path in "${input_paths[@]}"; do
  [[ -f "$path" ]] || fail "missing strict provenance input: $path"
done

source_commit="$(git rev-parse HEAD)"
source_tree="$(git rev-parse HEAD^{tree})"
tracked_status="$(git status --porcelain --untracked-files=no)"
source_dirty="false"
if [[ -n "$tracked_status" ]]; then
  source_dirty="true"
fi
if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" && "$source_dirty" == "true" ]]; then
  git status --short >&2
  fail "strict release gate requires a clean tracked working tree"
fi

release_tag_exists="false"
release_tag_commit="$source_commit"
release_tag_tree="$source_tree"
if git rev-parse -q --verify "${RELEASE_TAG}^{commit}" >/dev/null; then
  release_tag_exists="true"
  release_tag_commit="$(git rev-list -n 1 "$RELEASE_TAG")"
  release_tag_tree="$(git rev-parse "$RELEASE_TAG^{tree}")"
fi
if [[ "${NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG:-0}" == "1" || "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  [[ "$release_tag_exists" == "true" ]] || fail "missing required local release tag: $RELEASE_TAG"
  [[ "$source_commit" == "$release_tag_commit" ]] || fail "HEAD $source_commit does not match $RELEASE_TAG commit $release_tag_commit"
fi

cargo_version="$(cargo --version)"
rustc_version="$(rustc --version)"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

PRODUCT_VERSION="$PRODUCT_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


for needle in (
    "Status: RELEASED",
    "Tag: `ntpro-rust-only-v0.25.0`",
    "Release name: `NTPRO Rust-only v0.25.0`",
    "v0.25.0 publishes the Monitoring, Incident, and Disaster-Recovery Foundation",
    "This release does not add submit capability",
    "This release is not a product-grade live trading terminal",
    "V250-009",
    "V250 final release scope issue count = 10",
    "V250-009 failed release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28762387835",
    "V250-009 final success release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28764231552",
    "scripts/ai/verify_v25_release_gates.sh",
    "scripts/ai/verify_v25_strict_provenance.sh",
):
    require(needle in release_notes, f"release notes missing required marker: {needle}")

for needle in (
    "Milestone: `ntpro-rust-only-v0.25.0`",
    "Status: RELEASED",
    "V250-008 evidence",
    "V250-009 corrective evidence",
    "v25 strict provenance = required",
    "release surface current guard = required",
    "V250 final release scope issue count = 10",
    "No V260 implementation starts until all V251 issues are closed and v0.25.1",
):
    require(needle in readiness, f"readiness report missing required marker: {needle}")

require(release_manifest.get("schema_version") == "ntpro.v250_release_manifest.v1", "release manifest schema mismatch")
require(release_manifest.get("task_id") == "V250-008", "release manifest task mismatch")
require(release_manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "release manifest product version mismatch")
require(release_manifest.get("release_status") == "released", "release manifest status mismatch")
evidence = release_manifest.get("v250_evidence") or []
require(any(item.get("task_id") == "V250-009" and item.get("issue") == 804 for item in evidence), "V250-009 evidence missing from release manifest")
scope = release_manifest.get("release_scope") or {}
require(scope.get("milestone_issue_count") == 9, "milestone issue count mismatch")
require(scope.get("corrective_issue_count") == 1, "corrective issue count mismatch")
require(scope.get("final_release_scope_issue_count") == 10, "final release scope issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 10, "final release scope evidence count mismatch")
require(scope.get("corrective_scope_expands_capability") is False, "corrective scope must not expand capability")
corrective = release_manifest.get("corrective_release_scope") or {}
require(corrective.get("task_id") == "V250-009", "corrective task mismatch")
require(corrective.get("issue") == 804, "corrective issue mismatch")
require(corrective.get("pull_request") == 805, "corrective PR mismatch")
require(corrective.get("failed_release_gate_run") == 28762387835, "corrective failed run mismatch")
require(corrective.get("final_success_release_gate_run") == 28764231552, "corrective final run mismatch")
require(corrective.get("included_in_release_tag") is True, "corrective scope must be included in release tag")
require(corrective.get("capability_expansion") is False, "corrective scope must not expand capability")
planned = release_manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("draft") is False, "planned draft flag mismatch")
require(planned.get("prerelease") is False, "planned prerelease flag mismatch")
capability = release_manifest.get("capability") or {}
require(capability.get("monitoring_incident_dr_foundation") is True, "monitoring foundation flag mismatch")
require(capability.get("strict_provenance") is True, "strict provenance flag mismatch")
require(capability.get("new_submit_capability") is False, "submit capability flag mismatch")
boundary = release_manifest.get("boundary_flags") or {}
for key in (
    "new_submit_capability",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "product_grade_trading_terminal_claim",
):
    require(boundary.get(key) is False, f"boundary flag must be false: {key}")
PY

if [[ "${NTPRO_V250_STRICT_REQUIRE_PUBLICATION:-0}" == "1" ]]; then
  NTPRO_CURRENT_RELEASE_VERSION="$PRODUCT_VERSION" \
    NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
    NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only $PRODUCT_VERSION" \
    NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 \
    scripts/ai/check_github_release_published.sh
fi

if [[ "$VERIFY_ONLY" != "1" ]]; then
  mkdir -p "$(dirname "$MANIFEST_PATH")"
  input_paths_joined="$(printf '%s\n' "${input_paths[@]}")"
  MANIFEST_PATH="$MANIFEST_PATH" \
  PRODUCT_VERSION="$PRODUCT_VERSION" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_TAG_EXISTS="$release_tag_exists" \
  RELEASE_TAG_COMMIT="$release_tag_commit" \
  RELEASE_TAG_TREE="$release_tag_tree" \
  SOURCE_COMMIT="$source_commit" \
  SOURCE_TREE="$source_tree" \
  SOURCE_DIRTY="$source_dirty" \
  CARGO_VERSION="$cargo_version" \
  RUSTC_VERSION="$rustc_version" \
  GENERATED_AT="$generated_at" \
  ROOT_DIR="$ROOT_DIR" \
  INPUT_PATHS="$input_paths_joined" \
  RELEASE_MANIFEST_PATH="${RELEASE_MANIFEST_PATH#$ROOT_DIR/}" \
  python3 <<'PY'
import hashlib
import json
import os
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
root = Path(os.environ["ROOT_DIR"])
release_inputs = []
for raw_path in os.environ["INPUT_PATHS"].splitlines():
    path = Path(raw_path)
    rel = path.relative_to(root) if path.is_absolute() else path
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    release_inputs.append({"path": str(rel), "sha256": f"sha256:{digest}"})

manifest = {
    "schema_version": "ntpro.v250_strict_release_provenance_manifest.v1",
    "task_id": "V250-008",
    "target": "v25",
    "product_version": os.environ["PRODUCT_VERSION"],
    "planned_release": {
        "tag": os.environ["RELEASE_TAG"],
        "tag_exists": os.environ["RELEASE_TAG_EXISTS"] == "true",
        "tag_commit": os.environ["RELEASE_TAG_COMMIT"],
        "tag_tree": os.environ["RELEASE_TAG_TREE"],
    },
    "source": {
        "commit": os.environ["SOURCE_COMMIT"],
        "tree": os.environ["SOURCE_TREE"],
        "tracked_worktree_dirty": os.environ["SOURCE_DIRTY"] == "true",
    },
    "toolchain": {
        "cargo_version": os.environ["CARGO_VERSION"],
        "rustc_version": os.environ["RUSTC_VERSION"],
    },
    "release_inputs": release_inputs,
    "v250_evidence": release_manifest.get("v250_evidence"),
    "release_scope": release_manifest.get("release_scope"),
    "corrective_release_scope": release_manifest.get("corrective_release_scope"),
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "publication_governance": release_manifest.get("publication_governance"),
    "next_tracks": release_manifest.get("next_tracks"),
    "release_gates": release_manifest.get("release_gates"),
    "post_publication_requirements": release_manifest.get("post_publication_requirements"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 fails if tracked files are dirty",
        "missing_tag": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails without the v0.25.0 release tag",
        "tag_mismatch": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails when HEAD differs from the release tag",
        "missing_v250_evidence": "v0.25.0 release gate fails if any V250 milestone or corrective evidence path is missing",
        "missing_corrective_scope": "v0.25.0 release gate fails if V250-009 corrective task/evidence, #804 closeout, or PR #805 merge proof is missing",
        "open_operation_boundary": "v0.25.0 release gate fails if submit, mutation, adapter send, live exchange, retry scheduler, Dashboard trading, remediation, or order-ticket boundary opens",
        "open_v250_issue_or_milestone": "NTPRO_RELEASE_GATE=1 fails unless V250 issues are closed and the v0.25.0 milestone is closed",
        "pre_gate_publication": "public GitHub Release publication must use the gate-before-publish entrypoint after hosted gate success",
        "v26_start_without_v250_closeout": "v0.26.0 remains blocked until v0.25.0 release evidence is published",
    },
    "generated_at": os.environ["GENERATED_AT"],
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
elif [[ ! -f "$MANIFEST_PATH" ]]; then
  fail "manifest does not exist in verify-only mode: $MANIFEST_PATH"
fi

echo "v25_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=${MANIFEST_PATH#$ROOT_DIR/}"
