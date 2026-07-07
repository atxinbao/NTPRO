#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V260_PRODUCT_VERSION:-v0.26.0}"
RELEASE_TAG="${NTPRO_V260_RELEASE_TAG:-ntpro-rust-only-v0.26.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.26.0"
MANIFEST_PATH="${NTPRO_V260_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v260/v0_26_0_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V260_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_26_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V260_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_26_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V260_READINESS:-$ROOT_DIR/docs/rust-cutover/release/v0_26_0_readiness_report.md}"

fail() {
  echo "v26 strict release provenance drift: $*" >&2
  exit 1
}

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v26 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_1_release_manifest.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_1_readiness_report.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_intake_gate.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_product_hardening_boundary_contract.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_operator_permission_model.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_operation_audit_trail.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_deployment_provenance_model.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_upgrade_rollback_runbook_evidence.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_slo_runbook_stability_evidence.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_dashboard_admin_boundary_surface.md"
  "$ROOT_DIR/tests/golden/v260/release_gates_strict_provenance.jsonl"
  "$ROOT_DIR/docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json"
  "$ROOT_DIR/README.md"
  "$ROOT_DIR/ROADMAP.md"
  "$ROOT_DIR/docs/versioning.md"
  "$ROOT_DIR/docs/rust-cutover/release/README.md"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
  "$ROOT_DIR/scripts/ai/publish_ntpro_release_after_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_release_publish_after_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_release.sh"
  "$ROOT_DIR/scripts/ai/verify_v26_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v26_strict_provenance.sh"
  "$ROOT_DIR/.github/workflows/release-tag.yml"
  "$ROOT_DIR/.github/workflows/release-publish.yml"
)

for task_id in V260-000 V260-001 V260-002 V260-003 V260-004 V260-005 V260-006 V260-007 V260-008; do
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

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(release_manifest["schema_version"] == "ntpro.v260_release_manifest.v1", "manifest schema mismatch")
require(release_manifest["task_id"] == "V260-008", "manifest task mismatch")
require(release_manifest["product_version"] == os.environ["PRODUCT_VERSION"], "manifest product version mismatch")
require(release_manifest["planned_release"]["tag"] == os.environ["RELEASE_TAG"], "planned tag mismatch")
require("v26 release gates = required" in release_notes, "release notes v26 gate marker missing")
require("v26 strict provenance = required" in readiness, "readiness strict provenance marker missing")
require(release_manifest["publication_governance"]["release_gate_success_before_publication_required"] is True, "publication ordering requirement missing")
require(release_manifest["post_publication_requirements"]["all_v260_issues_closed_required"] is True, "issue closeout requirement missing")
for key in [
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
]:
    require(release_manifest["boundary_flags"].get(key) is False, f"boundary must remain false: {key}")

root = Path(os.environ["ROOT_DIR"])
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
    "schema_version": "ntpro.v260_strict_release_provenance_manifest.v1",
    "task_id": "V260-008",
    "target": "v26",
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
    "source_inputs": source_inputs,
    "v260_evidence": release_manifest.get("v260_evidence"),
    "release_scope": release_manifest.get("release_scope"),
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "publication_governance": release_manifest.get("publication_governance"),
    "next_tracks": release_manifest.get("next_tracks"),
    "release_gates": release_manifest.get("release_gates"),
    "post_publication_requirements": release_manifest.get("post_publication_requirements"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 fails if tracked files are dirty",
        "missing_tag": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails without the v0.26.0 release tag",
        "tag_mismatch": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails when HEAD differs from the release tag",
        "missing_v260_evidence": "v0.26.0 release gate fails if any V260 evidence path is missing",
        "open_operation_boundary": "v0.26.0 release gate fails if submit, mutation, adapter send, live exchange, retry scheduler, Dashboard trading, remediation, or order-ticket boundary opens",
        "missing_dashboard_smoke": "v0.26.0 release gate fails without Dashboard v26 admin surface render smoke evidence",
        "pre_gate_publication": "public GitHub Release publication must use the gate-before-publish entrypoint after hosted gate success",
        "open_v260_issue_or_milestone": "NTPRO_RELEASE_GATE=1 fails unless V260 issues are closed and the v0.26.0 milestone is closed",
    },
    "generated_at": os.environ["GENERATED_AT"],
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if [[ "${NTPRO_V260_STRICT_REQUIRE_PUBLICATION:-0}" == "1" ]]; then
  NTPRO_CURRENT_RELEASE_VERSION="$PRODUCT_VERSION" \
    NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
    NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only $PRODUCT_VERSION" \
    NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 \
    scripts/ai/check_github_release_published.sh
fi

echo "v26_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=${MANIFEST_PATH#$ROOT_DIR/}"
