#!/usr/bin/env bash
set -euo pipefail

# V171-008: v0.17.1 release evidence hardening gate.
# This gate produces machine-readable provenance for the patch-hardening
# release surface. It does not open network access, submit orders, mutate
# orders, cancel orders, or enable Dashboard controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V171_SKIP_BUILD:-0}" != "1" && ( -z "${NTPRO_V171_NAUTILUS_BIN:-}" || -z "${NTPRO_V171_NTPRO_NODE_BIN:-}" ) ]]; then
  cargo build -p nautilus-cli --release --bin nautilus --bin ntpro-node
fi

NAUTILUS_BIN="${NTPRO_V171_NAUTILUS_BIN:-$ROOT_DIR/target/release/nautilus}"
NTPRO_NODE_BIN="${NTPRO_V171_NTPRO_NODE_BIN:-$ROOT_DIR/target/release/ntpro-node}"
for bin in "$NAUTILUS_BIN" "$NTPRO_NODE_BIN"; do
  if [[ ! -x "$bin" ]]; then
    echo "missing release binary: $bin" >&2
    exit 1
  fi
  if [[ "$bin" != */target/release/* && "${NTPRO_V171_ALLOW_NON_RELEASE_BIN:-0}" != "1" ]]; then
    echo "release hardening gate requires target/release binary, got: $bin" >&2
    exit 1
  fi
done

manifest_path="${NTPRO_V171_RELEASE_MANIFEST:-$ROOT_DIR/target/ntpro-v171/v0_17_1_release_manifest.json}"
mkdir -p "$(dirname "$manifest_path")"

git_commit="$(git rev-parse HEAD)"
git_tree="$(git rev-parse HEAD^{tree})"
git_dirty="false"
if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then
  git_dirty="true"
fi
if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" && "$git_dirty" == "true" ]]; then
  echo "release gate requires a clean tracked working tree" >&2
  git status --short >&2
  exit 1
fi

cargo_workspace_version="$(cargo metadata --no-deps --format-version=1 \
  | python3 -c 'import json,sys; data=json.load(sys.stdin); print(next(p["version"] for p in data["packages"] if p["name"] == "nautilus-cli"))')"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
nautilus_version="$("$NAUTILUS_BIN" --version)"
ntpro_node_version="$("$NTPRO_NODE_BIN" --version)"

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print $1}'
  else
    shasum -a 256 "$path" | awk '{print $1}'
  fi
}

artifact_json="$(
  python3 - "$ROOT_DIR" "$git_commit" "${NTPRO_V171_RELEASE_TAG:-ntpro-rust-only-v0.17.1}" <<'PY'
import json
import hashlib
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
commit = sys.argv[2]
release_tag = sys.argv[3]
paths = [
    "docs/rust-cutover/evidence/V170-009.md",
    "docs/rust-cutover/evidence/V171-001.md",
    "docs/rust-cutover/evidence/V171-002.md",
    "docs/rust-cutover/evidence/V171-003.md",
    "docs/rust-cutover/evidence/V171-004.md",
    "docs/rust-cutover/evidence/V171-005.md",
    "docs/rust-cutover/evidence/V171-006.md",
    "docs/rust-cutover/evidence/V171-007.md",
    "docs/rust-cutover/evidence/V171-008.md",
    "docs/rust-cutover/release/v0_17_1_readiness_report.md",
    "docs/rust-cutover/release/v0_17_1_release_notes.md",
    "scripts/ai/verify_v171_release_hardening.sh",
]
items = []
for rel in paths:
    path = root / rel
    if not path.exists():
        raise SystemExit(f"missing artifact provenance source: {rel}")
    raw = path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    items.append(
        {
            "path": rel,
            "bytes": len(raw),
            "sha256": f"sha256:{digest}",
            "source_command": "git tracked release evidence",
            "source_commit": commit,
            "source_release_tag": release_tag,
        }
    )
print(json.dumps(items, sort_keys=True))
PY
)"

NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_NODE_BIN="$NTPRO_NODE_BIN" \
MANIFEST_PATH="$manifest_path" \
GIT_COMMIT="$git_commit" \
GIT_TREE="$git_tree" \
GIT_DIRTY="$git_dirty" \
GENERATED_AT="$generated_at" \
CARGO_WORKSPACE_VERSION="$cargo_workspace_version" \
NAUTILUS_VERSION="$nautilus_version" \
NTPRO_NODE_VERSION="$ntpro_node_version" \
NAUTILUS_SHA256="sha256:$(sha256_file "$NAUTILUS_BIN")" \
NTPRO_NODE_SHA256="sha256:$(sha256_file "$NTPRO_NODE_BIN")" \
NAUTILUS_BYTES="$(wc -c < "$NAUTILUS_BIN" | tr -d ' ')" \
NTPRO_NODE_BYTES="$(wc -c < "$NTPRO_NODE_BIN" | tr -d ' ')" \
ARTIFACT_JSON="$artifact_json" \
python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
manifest = {
    "schema_version": "ntpro.v171_release_provenance_manifest.v1",
    "product_version": "v0.17.1",
    "release_tag": os.environ.get("NTPRO_V171_RELEASE_TAG", "ntpro-rust-only-v0.17.1"),
    "current_published_release_tag": "ntpro-rust-only-v0.17.0",
    "git": {
        "commit": os.environ["GIT_COMMIT"],
        "tree": os.environ["GIT_TREE"],
        "working_tree_dirty": os.environ["GIT_DIRTY"] == "true",
    },
    "cargo_workspace_version": os.environ["CARGO_WORKSPACE_VERSION"],
    "capability": "v0.17.1 release evidence hardening",
    "capability_expansion": "none_patch_hardening_only",
    "generated_at": os.environ["GENERATED_AT"],
    "gate_status": {
        "status": "pass",
        "stage": "v171-release-hardening",
        "release_surface_current_guard": "required",
        "release_publication_guard": "required",
    },
    "release_binaries": [
        {
            "name": "nautilus",
            "path": os.environ["NAUTILUS_BIN"],
            "bytes": int(os.environ["NAUTILUS_BYTES"]),
            "sha256": os.environ["NAUTILUS_SHA256"],
            "version_output": os.environ["NAUTILUS_VERSION"],
            "build_timestamp": os.environ["GENERATED_AT"],
            "source_commit": os.environ["GIT_COMMIT"],
        },
        {
            "name": "ntpro-node",
            "path": os.environ["NTPRO_NODE_BIN"],
            "bytes": int(os.environ["NTPRO_NODE_BYTES"]),
            "sha256": os.environ["NTPRO_NODE_SHA256"],
            "version_output": os.environ["NTPRO_NODE_VERSION"],
            "build_timestamp": os.environ["GENERATED_AT"],
            "source_commit": os.environ["GIT_COMMIT"],
        },
    ],
    "artifact_provenance": json.loads(os.environ["ARTIFACT_JSON"]),
}
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
PY

python3 - "$manifest_path" <<'PY'
import json
import sys

manifest = json.loads(open(sys.argv[1], encoding="utf-8").read())
required = {
    "schema_version": "ntpro.v171_release_provenance_manifest.v1",
    "product_version": "v0.17.1",
    "capability_expansion": "none_patch_hardening_only",
}
for key, expected in required.items():
    if manifest.get(key) != expected:
        raise SystemExit(f"manifest {key} mismatch: {manifest.get(key)!r}")
for binary in manifest["release_binaries"]:
    if not binary["sha256"].startswith("sha256:") or binary["bytes"] <= 0:
        raise SystemExit(f"invalid binary provenance: {binary}")
    if not binary["version_output"].startswith("nautilus-cli "):
        raise SystemExit(f"missing CLI version output: {binary}")
for artifact in manifest["artifact_provenance"]:
    if not artifact["sha256"].startswith("sha256:") or artifact["bytes"] <= 0:
        raise SystemExit(f"invalid artifact provenance: {artifact}")
    for field in ("source_command", "source_commit", "source_release_tag"):
        if not artifact.get(field):
            raise SystemExit(f"missing {field}: {artifact}")
PY

if grep -Ei "hosted smoke pending|pending PR|Merge commit = pending|hosted smoke = pending" docs/rust-cutover/evidence/V170-009.md; then
  echo "V170-009 still contains stale pending release evidence" >&2
  exit 1
fi

for marker in \
  "release-surface-current-guard" \
  "release-publication-guard" \
  "none_patch_hardening_only" \
  "verify_fast alone does not prove compile/static-check coverage" \
  "ntpro-rust-only-v0.17.0" \
  "ntpro-rust-only-v0.17.1"; do
  if ! grep -RFi "$marker" \
    docs/rust-cutover/release/v0_17_1_readiness_report.md \
    docs/rust-cutover/release/v0_17_1_release_notes.md \
    docs/rust-cutover/evidence/V171-008.md >/dev/null; then
    echo "missing v0.17.1 hardening marker: $marker" >&2
    exit 1
  fi
done

for evidence in \
  docs/rust-cutover/evidence/V171-001.md \
  docs/rust-cutover/evidence/V171-002.md \
  docs/rust-cutover/evidence/V171-003.md \
  docs/rust-cutover/evidence/V171-004.md \
  docs/rust-cutover/evidence/V171-005.md \
  docs/rust-cutover/evidence/V171-006.md \
  docs/rust-cutover/evidence/V171-007.md \
  docs/rust-cutover/evidence/V171-008.md; do
  if [[ ! -f "$evidence" ]]; then
    echo "missing V171 evidence file: $evidence" >&2
    exit 1
  fi
done

echo "v171_release_hardening status=ok manifest=$manifest_path product_version=v0.17.1 release_tag=ntpro-rust-only-v0.17.1 capability_expansion=none_patch_hardening_only release-surface-current-guard=required release-publication-guard=required request_sent=false network_attempted=false production_order_mutations_attempted=0 cancel_attempted=false dashboard_cancel_controls_enabled=false"
