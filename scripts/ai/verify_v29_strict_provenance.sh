#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V290_PRODUCT_VERSION:-v0.29.0}"
RELEASE_TAG="${NTPRO_V290_RELEASE_TAG:-ntpro-rust-only-v0.29.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.29.0"
MANIFEST_PATH="${NTPRO_V290_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v290/v0_29_0_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V290_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_29_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V290_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_29_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V290_READINESS:-$ROOT_DIR/docs/rust-cutover/release/v0_29_0_readiness_report.md}"
HANDOFF_PATH="${NTPRO_V290_HANDOFF:-$ROOT_DIR/docs/rust-cutover/release/v0_29_0_v30_go_live_candidate_handoff.md}"

fail() {
  echo "v29 strict release provenance drift: $*" >&2
  exit 1
}

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v29 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$HANDOFF_PATH"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_intake_gate.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_persistent_audit_storage_production_readiness.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_persistent_audit_storage_production_readiness_artifact.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_telemetry_slo_ingestion_production_readiness.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_telemetry_slo_ingestion_production_readiness_artifact.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_permission_source_production_readiness.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_permission_source_production_readiness_artifact.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_read_only_backend_api_production_readiness.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_read_only_backend_api_production_readiness_artifact.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness_artifact.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_monitoring_alert_incident_production_readiness.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_monitoring_alert_incident_production_readiness_artifact.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_canary_rollback_dr_preflight_readiness.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_canary_rollback_dr_preflight_readiness_artifact.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_backend_production_readiness_fail_closed_hardening.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_29_0_backend_production_readiness_fail_closed_hardening_artifact.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_28_1_release_manifest.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md"
  "$ROOT_DIR/README.md"
  "$ROOT_DIR/ROADMAP.md"
  "$ROOT_DIR/docs/versioning.md"
  "$ROOT_DIR/docs/rust-cutover/release/README.md"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
  "$ROOT_DIR/scripts/ai/verify_release_publish_after_gate.sh"
  "$ROOT_DIR/scripts/ai/publish_ntpro_release_after_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_release.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_intake_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_backend_production_readiness_boundary_contract.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_persistent_audit_storage_production_readiness.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_telemetry_slo_ingestion_production_readiness.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_permission_source_production_readiness.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_read_only_backend_api_production_readiness.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_deployment_config_runbook_production_readiness.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_monitoring_alert_incident_production_readiness.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_canary_rollback_dr_preflight_readiness.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_backend_production_readiness_fail_closed_hardening.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v29_strict_provenance.sh"
  "$ROOT_DIR/.github/workflows/release-tag.yml"
  "$ROOT_DIR/.github/workflows/release-publish.yml"
)

for task_id in V290-000 V290-001 V290-002 V290-003 V290-004 V290-005 V290-006 V290-007 V290-008 V290-009 V290-010 V290-011; do
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
mkdir -p "$(dirname "$MANIFEST_PATH")"

PRODUCT_VERSION="$PRODUCT_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
HANDOFF_PATH="$HANDOFF_PATH" \
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
handoff = Path(os.environ["HANDOFF_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(release_manifest["schema_version"] == "ntpro.v290_release_manifest.v1", "manifest schema mismatch")
require(release_manifest["task_id"] == "V290-011", "manifest task mismatch")
require(release_manifest["product_version"] == os.environ["PRODUCT_VERSION"], "manifest product version mismatch")
require(release_manifest["planned_release"]["tag"] == os.environ["RELEASE_TAG"], "planned tag mismatch")
require("v29 release gates = required" in release_notes, "release notes v29 gate marker missing")
require("v29 strict provenance = required" in readiness, "readiness strict provenance marker missing")
require("v0.30.0 backend production go-live candidate = next track" in handoff, "handoff next-track marker missing")
scope = release_manifest.get("release_scope") or {}
require(scope.get("exact_milestone_issue_numbers") == [926, 927, 928, 929, 930, 931, 932, 933, 934, 935, 936, 961], "exact milestone issue numbers mismatch")
require(scope.get("exact_milestone_issue_set") == "#926-#936, #961", "exact milestone issue set mismatch")
require(scope.get("final_release_scope_issue_count") == 12, "final release scope issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 12, "final release scope evidence count mismatch")
require(scope.get("registered_corrective_scope_exception_count") == 1, "registered corrective exception count mismatch")
require(scope.get("registered_corrective_scope_exception_issue_numbers") == [961], "registered corrective exception issue numbers mismatch")
require(scope.get("production_ready_count") == 11, "production-ready count mismatch")
require(scope.get("readiness_preview_count") == 2, "readiness-preview count mismatch")
require(scope.get("blocked_count") == 0, "blocked count mismatch")
require(scope.get("deferred_count") == 0, "deferred count mismatch")
requirements = release_manifest.get("post_publication_requirements") or {}
require(requirements.get("all_v290_issues_closed_required") is True, "V290 closeout requirement missing")
require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
require(requirements.get("strict_release_body_match_required") is True, "strict body requirement missing")
require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
require(requirements.get("v0_30_start_gate_fails_without_v290_release_evidence") is True, "v30 hard-block requirement missing")
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
    "schema_version": "ntpro.v290_strict_release_provenance.v1",
    "task_id": "V290-011",
    "target": "v29",
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
        "tracked_worktree_dirty": os.environ["SOURCE_DIRTY"] == "true",
    },
    "toolchain": {
        "cargo_version": os.environ["CARGO_VERSION"],
        "rustc_version": os.environ["RUSTC_VERSION"],
    },
    "release_body_source": str(Path(os.environ["RELEASE_NOTES_PATH"]).relative_to(root)),
    "release_body_sha256": "sha256:" + hashlib.sha256(release_notes.encode("utf-8")).hexdigest(),
    "readiness_report_source": str(Path(os.environ["READINESS_REPORT_PATH"]).relative_to(root)),
    "readiness_report_sha256": "sha256:" + hashlib.sha256(readiness.encode("utf-8")).hexdigest(),
    "handoff_source": str(Path(os.environ["HANDOFF_PATH"]).relative_to(root)),
    "handoff_sha256": "sha256:" + hashlib.sha256(handoff.encode("utf-8")).hexdigest(),
    "source_inputs": source_inputs,
    "v290_issue_scope": [926, 927, 928, 929, 930, 931, 932, 933, 934, 935, 936, 961],
    "v290_evidence": release_manifest.get("v290_evidence"),
    "release_scope": release_manifest.get("release_scope"),
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "publication_governance": release_manifest.get("publication_governance"),
    "release_surface_current_guard": release_manifest.get("release_surface_current_guard"),
    "next_tracks": release_manifest.get("next_tracks"),
    "post_publication_requirements": release_manifest.get("post_publication_requirements"),
    "base_release": release_manifest.get("base_release"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 fails if tracked files are dirty",
        "missing_tag": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails without the v0.29.0 release tag",
        "tag_mismatch": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails when HEAD differs from the release tag",
        "open_v290_issue": "NTPRO_RELEASE_GATE=1 fails unless V290 issues #926-#936 and #961 are closed",
        "open_operation_boundary": "v0.29.0 provenance fails if submit, mutation, adapter send, live exchange, retry scheduler, automatic remediation, Dashboard/Admin/Trader Terminal trading, backend go-live, or order-ticket boundary opens",
        "missing_v30_handoff": "v0.30.0 go-live candidate intake fails unless v0.29.0 handoff is source controlled",
    },
    "generated_at": os.environ["GENERATED_AT"],
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

echo "v29_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=${MANIFEST_PATH#$ROOT_DIR/}"
