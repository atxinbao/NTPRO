#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V251_PRODUCT_VERSION:-v0.25.1}"
RELEASE_TAG="${NTPRO_V251_RELEASE_TAG:-ntpro-rust-only-v0.25.1}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.25.1"
MANIFEST_PATH="${NTPRO_V251_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v251/v0_25_1_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V251_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_25_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V251_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_25_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V251_READINESS:-$ROOT_DIR/docs/rust-cutover/release/v0_25_1_readiness_report.md}"

fail() {
  echo "v25.1 strict release provenance drift: $*" >&2
  exit 1
}

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v25.1 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_release_manifest.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_25_0_release_closeout_evidence.md"
  "$ROOT_DIR/README.md"
  "$ROOT_DIR/ROADMAP.md"
  "$ROOT_DIR/docs/versioning.md"
  "$ROOT_DIR/docs/rust-cutover/release/README.md"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
  "$ROOT_DIR/scripts/ai/verify_release.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_1_corrective_release_scope.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_1_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v25_1_strict_provenance.sh"
  "$ROOT_DIR/.github/workflows/release-tag.yml"
  "$ROOT_DIR/.github/workflows/release-publish.yml"
)

for task_id in V251-001 V251-002 V251-003 V251-004 V251-005 V251-006; do
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


require(release_manifest["schema_version"] == "ntpro.v251_patch_release_manifest.v1", "manifest schema mismatch")
require(release_manifest["product_version"] == os.environ["PRODUCT_VERSION"], "manifest product version mismatch")
require(release_manifest["planned_release"]["tag"] == os.environ["RELEASE_TAG"], "planned tag mismatch")
require("v25.1 release gates = required" in release_notes, "release notes gate marker missing")
require("v25.1 strict provenance = required" in readiness, "readiness strict provenance marker missing")
require(release_manifest["post_publication_requirements"]["v0_26_start_gate_fails_without_v251_release_evidence"] is True, "v0.26 start gate requirement missing")
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

files = {}
source_inputs = []
for path in os.environ["INPUT_PATHS"].splitlines():
    payload = Path(path).read_bytes()
    rel = Path(path)
    try:
        rel = rel.relative_to(Path.cwd())
    except ValueError:
        pass
    source_inputs.append({"path": str(rel), "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)})

for path in [
    os.environ["RELEASE_MANIFEST_PATH"],
    os.environ["RELEASE_NOTES_PATH"],
    os.environ["READINESS_REPORT_PATH"],
]:
    payload = Path(path).read_bytes()
    files[path] = {"sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)}

payload = {
    "schema_version": "ntpro.v251_strict_release_provenance.v1",
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
    "corrective_issue_scope": {
        "issue": 804,
        "pull_request": 805,
        "included_in_base_release_tag": True,
    },
    "v251_issue_scope": [806, 807, 808, 809, 810, 811],
    "files": files,
    "publication_evidence_strategy": "source_tree_plus_github_remote",
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

echo "v25_1_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=$MANIFEST_PATH"
