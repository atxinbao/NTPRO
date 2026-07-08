#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V261_PRODUCT_VERSION:-v0.26.1}"
RELEASE_TAG="${NTPRO_V261_RELEASE_TAG:-ntpro-rust-only-v0.26.1}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.26.1"
MANIFEST_PATH="${NTPRO_V261_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v261/v0_26_1_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V261_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_26_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V261_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_26_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V261_READINESS:-$ROOT_DIR/docs/rust-cutover/release/v0_26_1_readiness_report.md}"

fail() {
  echo "v26.1 strict release provenance drift: $*" >&2
  exit 1
}

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v26.1 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_release_manifest.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_26_0_release_closeout_evidence.md"
  "$ROOT_DIR/README.md"
  "$ROOT_DIR/ROADMAP.md"
  "$ROOT_DIR/docs/versioning.md"
  "$ROOT_DIR/docs/rust-cutover/release/README.md"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
  "$ROOT_DIR/scripts/ai/verify_release.sh"
  "$ROOT_DIR/scripts/ai/verify_v26_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v26_strict_provenance.sh"
  "$ROOT_DIR/scripts/ai/verify_v26_1_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v26_1_strict_provenance.sh"
  "$ROOT_DIR/scripts/ai/verify_v27_intake_gate.sh"
  "$ROOT_DIR/.github/workflows/release-tag.yml"
  "$ROOT_DIR/.github/workflows/release-publish.yml"
)

for task_id in V261-001 V261-002 V261-003 V261-004 V261-005 V261-006; do
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


require(release_manifest["schema_version"] == "ntpro.v261_patch_release_manifest.v1", "manifest schema mismatch")
require(release_manifest["product_version"] == os.environ["PRODUCT_VERSION"], "manifest product version mismatch")
require(release_manifest["planned_release"]["tag"] == os.environ["RELEASE_TAG"], "planned tag mismatch")
require("v26.1 release gates = required" in release_notes, "release notes gate marker missing")
require("v26.1 strict provenance = required" in readiness, "readiness strict provenance marker missing")
require(release_manifest["post_publication_requirements"]["v0_27_start_gate_fails_without_v261_release_evidence"] is True, "v0.27 start gate requirement missing")
corrective = release_manifest.get("v261_corrective_scope") or {}
require(corrective.get("classification") == "corrective_scope_exception", "corrective scope classification missing")
require(corrective.get("final_release_scope_issue_count") == 6, "corrective final release scope issue count mismatch")
require(corrective.get("final_release_scope_evidence_count") == 6, "corrective final release scope evidence count mismatch")
require(corrective.get("corrective_scope_exception_count") == 1, "corrective exception count mismatch")
require(corrective.get("registered_corrective_scope_exceptions_closed_required") is True, "corrective closeout rule missing")
require(corrective.get("unregistered_corrective_milestone_issue_fail_closed") is True, "unregistered corrective fail-closed rule missing")
require(corrective.get("v27_intake_reconstructs_corrective_scope_exceptions") is True, "v27 corrective reconstruction rule missing")
exceptions = corrective.get("exceptions") or []
require(len(exceptions) == 1, "corrective exception list mismatch")
exception = exceptions[0]
require(exception.get("task_id") == "V261-007", "corrective task mismatch")
require(exception.get("issue") == 868, "corrective issue mismatch")
require(exception.get("remote_reconstruction_required") is True, "corrective remote reconstruction missing")
require(exception.get("required_for_v27_intake") is True, "corrective v27 intake requirement missing")
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

source_inputs = []
for path in os.environ["INPUT_PATHS"].splitlines():
    payload = Path(path).read_bytes()
    rel = Path(path)
    try:
        rel = rel.relative_to(Path.cwd())
    except ValueError:
        pass
    source_inputs.append({"path": str(rel), "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)})

files = {}
for path in [
    os.environ["RELEASE_MANIFEST_PATH"],
    os.environ["RELEASE_NOTES_PATH"],
    os.environ["READINESS_REPORT_PATH"],
]:
    payload = Path(path).read_bytes()
    files[path] = {"sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)}

payload = {
    "schema_version": "ntpro.v261_strict_release_provenance.v1",
    "product_version": os.environ["PRODUCT_VERSION"],
    "release_tag": os.environ["RELEASE_TAG"],
    "source_commit": os.environ["SOURCE_COMMIT"],
    "source_tree": os.environ["SOURCE_TREE"],
    "source_dirty": os.environ["SOURCE_DIRTY"] == "true",
    "release_tag_exists": os.environ["RELEASE_TAG_EXISTS"] == "true",
    "release_tag_commit": os.environ["RELEASE_TAG_COMMIT"],
    "release_tag_tree": os.environ["RELEASE_TAG_TREE"],
    "release_body_source": os.environ["RELEASE_NOTES_PATH"],
    "release_body_sha256": hashlib.sha256(release_notes.encode("utf-8")).hexdigest(),
    "source_inputs": source_inputs,
    "v261_issue_scope": [847, 848, 849, 850, 851, 852],
    "v261_corrective_scope": release_manifest.get("v261_corrective_scope"),
    "next_intake_gate": {
        "version": "v0.27.0",
        "command": "scripts/ai/verify_release.sh v27-intake-gate",
        "start_gate": "blocked_until_v261_release_evidence_published",
    },
    "files": files,
    "publication_evidence_strategy": "source_tree_plus_github_remote",
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

echo "v26_1_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=$MANIFEST_PATH"
