#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V300_PRODUCT_VERSION:-v0.30.0}"
RELEASE_TAG="${NTPRO_V300_RELEASE_TAG:-ntpro-rust-only-v0.30.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.30.0"
MANIFEST_PATH="${NTPRO_V300_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v300/v0_30_0_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V300_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_30_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V300_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_30_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V300_READINESS:-$ROOT_DIR/docs/rust-cutover/release/v0_30_0_readiness_report.md}"

fail() {
  echo "v30 strict release provenance drift: $*" >&2
  exit 1
}

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v30 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$ROOT_DIR/docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_30_0_v31_production_enablement_handoff.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_30_0_v31_production_enablement_handoff.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_1_release_manifest.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md"
  "$ROOT_DIR/docs/rust-cutover/release/README.md"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/publish_ntpro_release_after_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_v30_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v30_strict_provenance.sh"
  "$ROOT_DIR/.github/workflows/release-tag.yml"
)

for task_id in V300-000 V300-001 V300-002 V300-003 V300-004 V300-005 V300-006 V300-007 V300-008 V300-009 V300-010 V300-011; do
  input_paths+=("$ROOT_DIR/docs/rust-cutover/evidence/${task_id}.md")
  input_paths+=("$ROOT_DIR/docs/rust-cutover/tasks/${task_id}.md")
done

for script in \
  verify_v30_intake_gate.sh \
  verify_v30_backend_go_live_candidate_boundary_contract.sh \
  verify_v30_production_deployment_plan_environment_readiness.sh \
  verify_v30_runtime_enablement_boundary_controlled_feature_flags.sh \
  verify_v30_operator_approval_freeze_change_window_lifecycle.sh \
  verify_v30_canary_execution_preflight_no_default_execution_gate.sh \
  verify_v30_rollback_disaster_recovery_execution_boundary.sh \
  verify_v30_production_config_provenance_venue_connectivity_readiness.sh \
  verify_v30_telemetry_slo_gate_incident_freeze_integration.sh \
  verify_v30_audit_retention_evidence_export_readiness.sh \
  verify_v30_go_no_go_runbook_live_readiness_decision_record.sh; do
  input_paths+=("$ROOT_DIR/scripts/ai/$script")
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
mkdir -p "$(dirname "$MANIFEST_PATH")"

PRODUCT_VERSION="$PRODUCT_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
MANIFEST_PATH="$MANIFEST_PATH" \
SOURCE_COMMIT="$source_commit" \
SOURCE_TREE="$source_tree" \
SOURCE_DIRTY="$source_dirty" \
RELEASE_TAG_EXISTS="$release_tag_exists" \
RELEASE_TAG_COMMIT="$release_tag_commit" \
RELEASE_TAG_TREE="$release_tag_tree" \
CARGO_VERSION="$cargo_version" \
RUSTC_VERSION="$rustc_version" \
GENERATED_AT="$generated_at" \
ROOT_DIR="$ROOT_DIR" \
INPUT_PATHS="$(printf '%s\n' "${input_paths[@]}")" \
python3 <<'PY'
import hashlib
import json
import os
from pathlib import Path

root = Path(os.environ["ROOT_DIR"])
release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

require(release_manifest["schema_version"] == "ntpro.v300_backend_go_live_candidate_release_manifest.v1", "manifest schema mismatch")
require(release_manifest["task_id"] == "V300-011", "manifest task mismatch")
require(release_manifest["product_version"] == os.environ["PRODUCT_VERSION"], "manifest product version mismatch")
require(release_manifest["planned_release"]["tag"] == os.environ["RELEASE_TAG"], "planned tag mismatch")
require("v30 release gates = required" in release_notes, "release notes v30 gate marker missing")
require("v30 strict provenance = required" in readiness, "readiness strict marker missing")
scope = release_manifest.get("release_scope_facts") or {}
require(scope.get("exact_milestone_issue_numbers") == list(range(969, 981)), "exact issue numbers mismatch")
require(scope.get("final_release_scope_issue_count") == 12, "final issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 12, "final evidence count mismatch")
requirements = release_manifest.get("post_publication_requirements") or {}
require(requirements.get("all_v300_issues_closed_required") is True, "V300 closeout requirement missing")
require(requirements.get("v31_handoff_fails_without_v300_release_evidence") is True, "v31 blocker missing")
next_tracks = release_manifest.get("next_tracks") or {}
require(next_tracks.get("start_gate") == "hard_blocked_until_v30_release_evidence_and_explicit_scoped_approval", "next start gate mismatch")
for key in [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
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
    "backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "product_grade_trading_terminal_claim",
]:
    require(release_manifest["boundary_flags"].get(key) is False, f"boundary must remain false: {key}")

source_inputs = []
for raw_path in os.environ["INPUT_PATHS"].splitlines():
    path = Path(raw_path)
    payload = path.read_bytes()
    rel = path.relative_to(root) if path.is_absolute() else path
    source_inputs.append({
        "path": str(rel),
        "sha256": "sha256:" + hashlib.sha256(payload).hexdigest(),
        "bytes": len(payload),
    })

payload = {
    "schema_version": "ntpro.v300_strict_release_provenance.v1",
    "task_id": "V300-011",
    "target": "v30.0",
    "product_version": os.environ["PRODUCT_VERSION"],
    "release_tag": os.environ["RELEASE_TAG"],
    "planned_release": {
        "tag": os.environ["RELEASE_TAG"],
        "tag_exists": os.environ["RELEASE_TAG_EXISTS"] == "true",
        "tag_commit": os.environ["RELEASE_TAG_COMMIT"],
        "tag_tree": os.environ["RELEASE_TAG_TREE"],
    },
    "source": {
        "commit": os.environ["SOURCE_COMMIT"],
        "tree": os.environ["SOURCE_TREE"],
        "dirty": os.environ["SOURCE_DIRTY"] == "true",
    },
    "toolchain": {
        "cargo": os.environ["CARGO_VERSION"],
        "rustc": os.environ["RUSTC_VERSION"],
    },
    "generated_at": os.environ["GENERATED_AT"],
    "source_inputs": source_inputs,
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

echo "v30_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=${MANIFEST_PATH#$ROOT_DIR/}"
