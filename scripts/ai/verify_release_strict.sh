#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

target="${1:-}"
if [[ "$target" != "v18" ]]; then
  echo "usage: scripts/ai/verify_release_strict.sh v18" >&2
  exit 2
fi

PRODUCT_VERSION="${NTPRO_RELEASE_STRICT_PRODUCT_VERSION:-v0.18.1}"
BASELINE_RELEASE_TAG="${NTPRO_RELEASE_STRICT_BASELINE_TAG:-ntpro-rust-only-v0.18.0}"
MANIFEST_PATH="${NTPRO_RELEASE_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v181/v0_18_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V181_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_18_1_release_manifest.json}"
NAUTILUS_BIN="${NTPRO_RELEASE_STRICT_NAUTILUS_BIN:-$ROOT_DIR/target/release/nautilus}"
VERIFY_ONLY="${NTPRO_RELEASE_STRICT_VERIFY_ONLY:-0}"

fail() {
  echo "strict release provenance drift: $*" >&2
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

[[ -f "$RELEASE_MANIFEST_PATH" ]] || fail "missing v0.18.1 release manifest: $RELEASE_MANIFEST_PATH"

if [[ "$VERIFY_ONLY" != "1" && "${NTPRO_RELEASE_STRICT_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build -p nautilus-cli --release --bin nautilus
fi

[[ -x "$NAUTILUS_BIN" ]] || fail "missing release binary: $NAUTILUS_BIN"
if [[ "$NAUTILUS_BIN" != */target/release/nautilus && "${NTPRO_RELEASE_STRICT_ALLOW_NON_RELEASE_BIN:-0}" != "1" ]]; then
  fail "strict gate requires target/release/nautilus, got: $NAUTILUS_BIN"
fi

baseline_commit="$(git rev-list -n 1 "$BASELINE_RELEASE_TAG")"
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
if [[ "${NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG:-0}" == "1" && "$source_commit" != "$baseline_commit" ]]; then
  fail "HEAD $source_commit does not match $BASELINE_RELEASE_TAG commit $baseline_commit"
fi

cargo_version="$(cargo --version)"
rustc_version="$(rustc --version)"
binary_version="$("$NAUTILUS_BIN" --version)"
binary_sha256="sha256:$(sha256_file "$NAUTILUS_BIN")"
binary_bytes="$(wc -c < "$NAUTILUS_BIN" | tr -d ' ')"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
release_manifest_sha256="sha256:$(sha256_file "$RELEASE_MANIFEST_PATH")"
release_manifest_rel="${RELEASE_MANIFEST_PATH#$ROOT_DIR/}"
strict_manifest_rel="${MANIFEST_PATH#$ROOT_DIR/}"

if [[ "$VERIFY_ONLY" != "1" ]]; then
  mkdir -p "$(dirname "$MANIFEST_PATH")"
  MANIFEST_PATH="$MANIFEST_PATH" \
  PRODUCT_VERSION="$PRODUCT_VERSION" \
  BASELINE_RELEASE_TAG="$BASELINE_RELEASE_TAG" \
  BASELINE_COMMIT="$baseline_commit" \
  SOURCE_COMMIT="$source_commit" \
  SOURCE_TREE="$source_tree" \
  SOURCE_DIRTY="$source_dirty" \
  CARGO_VERSION="$cargo_version" \
  RUSTC_VERSION="$rustc_version" \
  NAUTILUS_BIN="$NAUTILUS_BIN" \
  BINARY_SHA256="$binary_sha256" \
  BINARY_BYTES="$binary_bytes" \
  BINARY_VERSION="$binary_version" \
  GENERATED_AT="$generated_at" \
  RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
  RELEASE_MANIFEST_REL="$release_manifest_rel" \
  RELEASE_MANIFEST_SHA256="$release_manifest_sha256" \
  python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
release_manifest_path = pathlib.Path(os.environ["RELEASE_MANIFEST_PATH"])
release_manifest = json.loads(release_manifest_path.read_text(encoding="utf-8"))
manifest = {
    "schema_version": "ntpro.v181_strict_binary_provenance_manifest.v1",
    "task_id": "V181-004",
    "target": "v18",
    "product_version": os.environ["PRODUCT_VERSION"],
    "release_manifest": {
        "path": os.environ["RELEASE_MANIFEST_REL"],
        "sha256": os.environ["RELEASE_MANIFEST_SHA256"],
        "schema_version": release_manifest.get("schema_version"),
        "task_id": release_manifest.get("task_id"),
        "product_version": release_manifest.get("product_version"),
        "planned_patch_tag": (release_manifest.get("patch_release") or {}).get("planned_tag"),
        "actual_patch_tag": (release_manifest.get("patch_release") or {}).get("actual_tag"),
    },
    "baseline_release": {
        "tag": os.environ["BASELINE_RELEASE_TAG"],
        "commit": os.environ["BASELINE_COMMIT"],
    },
    "source": {
        "commit": os.environ["SOURCE_COMMIT"],
        "tree": os.environ["SOURCE_TREE"],
        "tracked_worktree_dirty": os.environ["SOURCE_DIRTY"] == "true",
        "tag_match_required": False,
    },
    "toolchain": {
        "cargo_version": os.environ["CARGO_VERSION"],
        "rustc_version": os.environ["RUSTC_VERSION"],
    },
    "binary": {
        "name": "nautilus",
        "path": os.environ["NAUTILUS_BIN"],
        "bytes": int(os.environ["BINARY_BYTES"]),
        "sha256": os.environ["BINARY_SHA256"],
        "version_output": os.environ["BINARY_VERSION"],
        "build_profile": "release",
        "source_commit": os.environ["SOURCE_COMMIT"],
        "source_tree": os.environ["SOURCE_TREE"],
    },
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1",
        "tag_mismatch": "NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1",
        "manifest_mismatch": "NTPRO_RELEASE_STRICT_VERIFY_ONLY=1",
        "binary_hash_mismatch": "NTPRO_RELEASE_STRICT_VERIFY_ONLY=1",
    },
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "release_gates": release_manifest.get("release_gates"),
    "generated_at": os.environ["GENERATED_AT"],
}
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
elif [[ ! -f "$MANIFEST_PATH" ]]; then
  fail "manifest does not exist in verify-only mode: $MANIFEST_PATH"
fi

MANIFEST_PATH="$MANIFEST_PATH" \
PRODUCT_VERSION="$PRODUCT_VERSION" \
BASELINE_RELEASE_TAG="$BASELINE_RELEASE_TAG" \
BASELINE_COMMIT="$baseline_commit" \
SOURCE_COMMIT="$source_commit" \
SOURCE_TREE="$source_tree" \
SOURCE_DIRTY="$source_dirty" \
CARGO_VERSION="$cargo_version" \
RUSTC_VERSION="$rustc_version" \
NAUTILUS_BIN="$NAUTILUS_BIN" \
BINARY_SHA256="$binary_sha256" \
BINARY_BYTES="$binary_bytes" \
BINARY_VERSION="$binary_version" \
RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
RELEASE_MANIFEST_REL="$release_manifest_rel" \
RELEASE_MANIFEST_SHA256="$release_manifest_sha256" \
STRICT_MANIFEST_REL="$strict_manifest_rel" \
python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
release_manifest = json.loads(pathlib.Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"release manifest boundary flag must be false: {key}")

require(release_manifest.get("schema_version") == "ntpro.v181_release_manifest.v1", "release manifest schema_version mismatch")
require(release_manifest.get("task_id") == "V181-006", "release manifest task_id mismatch")
require(release_manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "release manifest product_version mismatch")
require(release_manifest.get("release_status") == "draft_not_published", "release manifest status mismatch")

release_baseline = release_manifest.get("baseline_release") or {}
require(release_baseline.get("version") == "v0.18.0", "release manifest baseline version mismatch")
require(release_baseline.get("tag") == os.environ["BASELINE_RELEASE_TAG"], "release manifest baseline tag mismatch")
require(release_baseline.get("commit") == os.environ["BASELINE_COMMIT"], "release manifest baseline commit mismatch")

patch_release = release_manifest.get("patch_release") or {}
require(patch_release.get("planned_tag") == "ntpro-rust-only-v0.18.1", "release manifest planned patch tag mismatch")
require(patch_release.get("actual_tag") is None, "release manifest actual patch tag must be null before publication")
require(patch_release.get("publication_status") == "not_published", "release manifest publication status mismatch")

source_provenance = release_manifest.get("source_provenance") or {}
require(source_provenance.get("actual_source_commit") is None, "release manifest actual source commit must be runtime-resolved")
require(source_provenance.get("actual_source_tree") is None, "release manifest actual source tree must be runtime-resolved")
require(source_provenance.get("actual_fields_resolved_by") == "scripts/ai/verify_release_strict.sh v18", "release manifest source resolver mismatch")
require(source_provenance.get("generated_manifest_path") == os.environ["STRICT_MANIFEST_REL"], "release manifest generated manifest path mismatch")

release_toolchain = release_manifest.get("toolchain") or {}
require(release_toolchain.get("cargo_version") == os.environ["CARGO_VERSION"], "release manifest cargo version mismatch")
require(release_toolchain.get("rustc_version") == os.environ["RUSTC_VERSION"], "release manifest rustc version mismatch")

binary_artifacts = release_manifest.get("binary_artifacts") or []
require(len(binary_artifacts) == 1, "release manifest binary_artifacts mismatch")
binary_contract = binary_artifacts[0]
require(binary_contract.get("name") == "nautilus", "release manifest binary name mismatch")
require(binary_contract.get("path") == "target/release/nautilus", "release manifest binary path mismatch")
require(binary_contract.get("build_profile") == "release", "release manifest binary build profile mismatch")
require(binary_contract.get("sha256") is None, "release manifest binary sha256 must be runtime-resolved")
require(binary_contract.get("bytes") is None, "release manifest binary bytes must be runtime-resolved")

release_gate_commands = {
    gate.get("command")
    for gate in (release_manifest.get("release_gates") or [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh release-surface-current-guard",
    "scripts/ai/verify_release.sh release-publication-guard",
    "scripts/ai/verify_release.sh v18-strict-provenance",
    "scripts/ai/verify_release_strict.sh v18",
):
    require(command in release_gate_commands, f"release manifest gate missing: {command}")

capability = release_manifest.get("capability") or {}
require(capability.get("capability_expansion") == "none_patch_hardening_only", "release manifest capability expansion mismatch")
require(capability.get("actual_cancel_scope") == "not_included", "release manifest actual cancel scope mismatch")

boundary_flags = release_manifest.get("boundary_flags") or {}
for key in (
    "actual_cancel_send_allowed",
    "actual_cancel_attempted",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "production_order_mutation_allowed",
    "network_cancel_endpoint_attempted",
    "binary_asset_publication_included",
):
    require_false(boundary_flags, key)

require(manifest.get("schema_version") == "ntpro.v181_strict_binary_provenance_manifest.v1", "schema_version mismatch")
require(manifest.get("task_id") == "V181-004", "task_id mismatch")
require(manifest.get("target") == "v18", "target mismatch")
require(manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "product_version mismatch")

release_manifest_ref = manifest.get("release_manifest") or {}
require(release_manifest_ref.get("path") == os.environ["RELEASE_MANIFEST_REL"], "embedded release manifest path mismatch")
require(release_manifest_ref.get("sha256") == os.environ["RELEASE_MANIFEST_SHA256"], "embedded release manifest sha256 mismatch")
require(release_manifest_ref.get("schema_version") == release_manifest.get("schema_version"), "embedded release manifest schema mismatch")
require(release_manifest_ref.get("task_id") == release_manifest.get("task_id"), "embedded release manifest task mismatch")
require(release_manifest_ref.get("product_version") == release_manifest.get("product_version"), "embedded release manifest product version mismatch")
require(release_manifest_ref.get("planned_patch_tag") == patch_release.get("planned_tag"), "embedded planned patch tag mismatch")
require(release_manifest_ref.get("actual_patch_tag") is None, "embedded actual patch tag must be null before publication")

baseline = manifest.get("baseline_release") or {}
require(baseline.get("tag") == os.environ["BASELINE_RELEASE_TAG"], "baseline tag mismatch")
require(baseline.get("commit") == os.environ["BASELINE_COMMIT"], "baseline commit mismatch")

source = manifest.get("source") or {}
require(source.get("commit") == os.environ["SOURCE_COMMIT"], "source commit mismatch")
require(source.get("tree") == os.environ["SOURCE_TREE"], "source tree mismatch")
require(source.get("tracked_worktree_dirty") == (os.environ["SOURCE_DIRTY"] == "true"), "source dirty mismatch")

toolchain = manifest.get("toolchain") or {}
require(toolchain.get("cargo_version") == os.environ["CARGO_VERSION"], "cargo version mismatch")
require(toolchain.get("rustc_version") == os.environ["RUSTC_VERSION"], "rustc version mismatch")

binary = manifest.get("binary") or {}
require(binary.get("name") == "nautilus", "binary name mismatch")
require(binary.get("path") == os.environ["NAUTILUS_BIN"], "binary path mismatch")
require(binary.get("bytes") == int(os.environ["BINARY_BYTES"]), "binary byte count mismatch")
require(binary.get("sha256") == os.environ["BINARY_SHA256"], "binary sha256 mismatch")
require(binary.get("version_output") == os.environ["BINARY_VERSION"], "binary version output mismatch")
require(binary.get("build_profile") == "release", "binary build profile mismatch")
require(binary.get("source_commit") == os.environ["SOURCE_COMMIT"], "binary source commit mismatch")
require(binary.get("source_tree") == os.environ["SOURCE_TREE"], "binary source tree mismatch")

failure_paths = manifest.get("failure_paths") or {}
for key in ("dirty_worktree", "tag_mismatch", "manifest_mismatch", "binary_hash_mismatch"):
    require(key in failure_paths, f"failure path missing: {key}")

require(manifest.get("capability") == capability, "embedded capability mismatch")
require(manifest.get("boundary_flags") == boundary_flags, "embedded boundary flags mismatch")
require(manifest.get("release_gates") == release_manifest.get("release_gates"), "embedded release gates mismatch")
PY

echo "strict_release_provenance status=ok target=v18 product_version=$PRODUCT_VERSION baseline_tag=$BASELINE_RELEASE_TAG baseline_commit=$baseline_commit source_commit=$source_commit source_tree=$source_tree source_dirty=$source_dirty cargo_version=\"$cargo_version\" rustc_version=\"$rustc_version\" binary_path=$NAUTILUS_BIN binary_sha256=$binary_sha256 binary_bytes=$binary_bytes release_manifest=$RELEASE_MANIFEST_PATH release_manifest_sha256=$release_manifest_sha256 manifest=$MANIFEST_PATH"
