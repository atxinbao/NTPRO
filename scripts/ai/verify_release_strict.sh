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
  python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
manifest = {
    "schema_version": "ntpro.v181_strict_binary_provenance_manifest.v1",
    "task_id": "V181-004",
    "target": "v18",
    "product_version": os.environ["PRODUCT_VERSION"],
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
python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

require(manifest.get("schema_version") == "ntpro.v181_strict_binary_provenance_manifest.v1", "schema_version mismatch")
require(manifest.get("task_id") == "V181-004", "task_id mismatch")
require(manifest.get("target") == "v18", "target mismatch")
require(manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "product_version mismatch")

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
PY

echo "strict_release_provenance status=ok target=v18 product_version=$PRODUCT_VERSION baseline_tag=$BASELINE_RELEASE_TAG baseline_commit=$baseline_commit source_commit=$source_commit source_tree=$source_tree source_dirty=$source_dirty cargo_version=\"$cargo_version\" rustc_version=\"$rustc_version\" binary_path=$NAUTILUS_BIN binary_sha256=$binary_sha256 binary_bytes=$binary_bytes manifest=$MANIFEST_PATH"
