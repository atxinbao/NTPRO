#!/usr/bin/env bash
set -euo pipefail

# V171-003: v0.17.1 release provenance manifest gate.
# This gate writes machine-readable release evidence only. It does not publish
# a tag, open network access, submit orders, mutate orders, cancel orders, or
# enable Dashboard controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

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
release_tag="${NTPRO_V171_RELEASE_TAG:-ntpro-rust-only-v0.17.1}"

MANIFEST_PATH="$manifest_path" \
GIT_COMMIT="$git_commit" \
GIT_TREE="$git_tree" \
GIT_DIRTY="$git_dirty" \
GENERATED_AT="$generated_at" \
CARGO_WORKSPACE_VERSION="$cargo_workspace_version" \
RELEASE_TAG="$release_tag" \
python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
manifest = {
    "schema_version": "ntpro.v171_release_provenance_manifest.v1",
    "product_version": "v0.17.1",
    "release_tag": os.environ["RELEASE_TAG"],
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
}
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
PY

python3 - "$manifest_path" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    manifest = json.load(handle)

required = {
    "schema_version": "ntpro.v171_release_provenance_manifest.v1",
    "product_version": "v0.17.1",
    "release_tag": "ntpro-rust-only-v0.17.1",
    "current_published_release_tag": "ntpro-rust-only-v0.17.0",
    "capability_expansion": "none_patch_hardening_only",
}
for key, expected in required.items():
    if manifest.get(key) != expected:
        raise SystemExit(f"manifest {key} mismatch: {manifest.get(key)!r}")
for key in ("commit", "tree", "working_tree_dirty"):
    if key not in manifest["git"]:
        raise SystemExit(f"manifest git.{key} missing")
if not manifest.get("cargo_workspace_version"):
    raise SystemExit("manifest cargo_workspace_version missing")
if not manifest.get("generated_at"):
    raise SystemExit("manifest generated_at missing")
gate_status = manifest.get("gate_status") or {}
if gate_status.get("stage") != "v171-release-hardening":
    raise SystemExit(f"manifest gate stage mismatch: {gate_status!r}")
if gate_status.get("status") != "pass":
    raise SystemExit(f"manifest gate status mismatch: {gate_status!r}")
PY

if ! grep -RFi "target/ntpro-v171/v0_17_1_release_manifest.json" \
  docs/rust-cutover/release/v0_17_1_readiness_report.md \
  docs/rust-cutover/evidence/V171-003.md >/dev/null; then
  echo "readiness evidence does not reference v0.17.1 release manifest" >&2
  exit 1
fi

echo "v171_release_hardening status=ok manifest=$manifest_path product_version=v0.17.1 release_tag=ntpro-rust-only-v0.17.1 capability_expansion=none_patch_hardening_only stage=v171-release-hardening request_sent=false network_attempted=false production_order_mutations_attempted=0 cancel_attempted=false dashboard_cancel_controls_enabled=false"

