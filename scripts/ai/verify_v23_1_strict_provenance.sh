#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V231_PRODUCT_VERSION:-v0.23.1}"
RELEASE_TAG="${NTPRO_V231_RELEASE_TAG:-ntpro-rust-only-v0.23.1}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.23.1"
MANIFEST_PATH="${NTPRO_V231_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v231/v0_23_1_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V231_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_23_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V231_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_23_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V231_READINESS_REPORT:-$ROOT_DIR/docs/rust-cutover/release/v0_23_1_readiness_report.md}"
BASE_RELEASE_MANIFEST_PATH="${NTPRO_V231_BASE_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_23_0_release_manifest.json}"
BASE_CLOSEOUT_PATH="${NTPRO_V231_BASE_CLOSEOUT:-$ROOT_DIR/docs/rust-cutover/release/v0_23_0_release_closeout_evidence.md}"
GATE_SCRIPT_PATH="${NTPRO_V231_GATE_SCRIPT:-$ROOT_DIR/scripts/ai/verify_v23_1_release_gates.sh}"
STRICT_SCRIPT_PATH="${NTPRO_V231_STRICT_SCRIPT:-$ROOT_DIR/scripts/ai/verify_v23_1_strict_provenance.sh}"
VERIFY_ONLY="${NTPRO_V231_STRICT_VERIFY_ONLY:-0}"

fail() {
  echo "v23.1 strict release provenance drift: $*" >&2
  exit 1
}

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print $1}'
  else
    shasum -a 256 "$path" | awk '{print $1}'
  fi
}

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v23.1 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$BASE_RELEASE_MANIFEST_PATH"
  "$BASE_CLOSEOUT_PATH"
  "$GATE_SCRIPT_PATH"
  "$STRICT_SCRIPT_PATH"
  "$ROOT_DIR/scripts/ai/verify_release.sh"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
)

for task_id in V231-001 V231-002 V231-003 V231-004 V231-005 V231-006; do
  input_paths+=("$ROOT_DIR/docs/rust-cutover/evidence/${task_id}.md")
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
    "Tag: `ntpro-rust-only-v0.23.1`",
    "Release name: `NTPRO Rust-only v0.23.1`",
    "v0.23.1 is a patch closeout release",
    "This release does not add submit capability",
    "scripts/ai/verify_v23_1_release_gates.sh",
    "scripts/ai/verify_v23_1_strict_provenance.sh",
):
    require(needle in release_notes, f"release notes missing required marker: {needle}")

for needle in (
    "Milestone: `ntpro-rust-only-v0.23.1`",
    "Status: RELEASED",
    "V231-006 evidence",
    "v23.1 strict provenance = required",
    "release surface current guard = required",
    "No V240 implementation starts until all V231 issues are closed and v0.23.1 release evidence is published",
):
    require(needle in readiness, f"readiness report missing required marker: {needle}")

for stale in (
    "ntpro-rust-only-v0.23.0-candidate",
    "public release publication = pending",
    "tag gate run = pending",
    "tag gate result = pending",
    "RELEASE GATE CORRECTIVE FIX IN PROGRESS",
    "corrective fix in progress",
):
    require(stale not in release_notes, f"release notes contain stale marker: {stale}")
    require(stale not in readiness, f"readiness report contains stale marker: {stale}")

require(release_manifest.get("schema_version") == "ntpro.v231_patch_release_manifest.v1", "release manifest schema mismatch")
require(release_manifest.get("task_id") == "V231-006", "release manifest task mismatch")
require(release_manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "release manifest product version mismatch")
require(release_manifest.get("release_status") == "published", "release manifest status mismatch")
planned = release_manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("draft") is False, "planned draft flag mismatch")
require(planned.get("prerelease") is False, "planned prerelease flag mismatch")
capability = release_manifest.get("capability") or {}
require(capability.get("patch_closeout_only") is True, "patch closeout flag mismatch")
require(capability.get("v0_24_implementation_started") is False, "v0.24 implementation flag mismatch")
boundary = release_manifest.get("boundary_flags") or {}
for key in (
    "new_submit_capability",
    "production_order_mutation_allowed",
    "dashboard_operation_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "product_grade_trading_terminal_claim",
):
    require(boundary.get(key) is False, f"boundary flag must be false: {key}")
PY

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
    "schema_version": "ntpro.v231_strict_release_provenance_manifest.v1",
    "task_id": "V231-006",
    "target": "v23.1",
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
    "v231_evidence": release_manifest.get("v231_evidence"),
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "publication_governance": release_manifest.get("publication_governance"),
    "next_tracks": release_manifest.get("next_tracks"),
    "release_gates": release_manifest.get("release_gates"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 fails if tracked files are dirty",
        "missing_tag": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails without the v0.23.1 release tag",
        "tag_mismatch": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails when HEAD differs from the release tag",
        "missing_v231_evidence": "v0.23.1 release gate fails if any V231 evidence path is missing",
        "open_operation_boundary": "v0.23.1 release gate fails if submit, mutation, Dashboard operation, or order-ticket boundary opens",
        "pre_gate_publication": "public GitHub Release publication must use the gate-before-publish entrypoint after hosted gate success",
    },
    "generated_at": os.environ["GENERATED_AT"],
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
elif [[ ! -f "$MANIFEST_PATH" ]]; then
  fail "manifest does not exist in verify-only mode: $MANIFEST_PATH"
fi

echo "v23_1_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=${MANIFEST_PATH#$ROOT_DIR/}"
