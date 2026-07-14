#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V320_PRODUCT_VERSION:-v0.32.0}"
RELEASE_TAG="${NTPRO_V320_RELEASE_TAG:-ntpro-rust-only-v0.32.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.32.0"
MANIFEST_PATH="${NTPRO_V320_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v320/v0_32_0_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V320_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_32_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V320_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_32_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V320_READINESS:-$ROOT_DIR/docs/rust-cutover/release/v0_32_0_readiness_report.md}"
RELEASE_CLOSEOUT_PATH="${NTPRO_V320_CLOSEOUT:-$ROOT_DIR/docs/rust-cutover/release/v0_32_0_release_closeout_evidence.md}"

fail() { echo "v32 strict release provenance drift: $*" >&2; exit 1; }

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v32 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$RELEASE_CLOSEOUT_PATH"
  "$ROOT_DIR/docs/rust-cutover/release/v0_31_1_release_manifest.json"
  "$ROOT_DIR/docs/rust-cutover/release/README.md"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
  "$ROOT_DIR/scripts/ai/publish_ntpro_release_after_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_v32_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v32_strict_provenance.sh"
  "$ROOT_DIR/.github/workflows/release-tag.yml"
)

for task_id in V320-000 V320-001 V320-002 V320-003 V320-004 V320-005 V320-006 V320-007 V320-008 V320-009; do
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
RELEASE_CLOSEOUT_PATH="$RELEASE_CLOSEOUT_PATH" \
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
closeout = Path(os.environ["RELEASE_CLOSEOUT_PATH"]).read_text(encoding="utf-8")

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(release_manifest["schema_version"] == "ntpro.v320_backend_closeout_release_manifest.v1", "manifest schema mismatch")
require(release_manifest["task_id"] == "V320-009", "manifest task mismatch")
require(release_manifest["product_version"] == os.environ["PRODUCT_VERSION"], "product version mismatch")
require(release_manifest["planned_release"]["tag"] == os.environ["RELEASE_TAG"], "planned tag mismatch")
require("v32 release gates = required" in release_notes, "release notes v32 gate marker missing")
require("v32 strict provenance = required" in readiness, "readiness strict marker missing")
require("release body must match tracked notes" in closeout, "closeout release body marker missing")
scope = release_manifest.get("release_scope") or {}
require(scope.get("exact_milestone_issue_numbers") == list(range(1042, 1052)), "exact issue numbers mismatch")
require(scope.get("final_release_scope_issue_count") == 10, "final issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 10, "final evidence count mismatch")
requirements = release_manifest.get("post_publication_requirements") or {}
require(requirements.get("all_v320_issues_closed_required") is True, "V320 closeout requirement missing")
require(requirements.get("v0_33_capability_inheritance_forbidden") is True, "v33 inheritance guard missing")
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
    "automatic_recovery_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "frontend_completion_claim",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
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
    "schema_version": "ntpro.v320_strict_release_provenance.v1",
    "task_id": "V320-009",
    "target": "v32",
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

echo "v32_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=${MANIFEST_PATH#$ROOT_DIR/}"
