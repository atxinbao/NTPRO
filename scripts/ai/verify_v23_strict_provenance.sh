#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V23_PRODUCT_VERSION:-v0.23.0}"
RELEASE_TAG="${NTPRO_V23_RELEASE_TAG:-ntpro-rust-only-v0.23.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.23.0"
MANIFEST_PATH="${NTPRO_V23_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v230/v0_23_0_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V23_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_23_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V23_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_23_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V23_READINESS_REPORT:-$ROOT_DIR/docs/rust-cutover/release/v0_23_0_readiness_report.md}"
CONTRACT_PATH="${NTPRO_V23_CONTRACT:-$ROOT_DIR/docs/rust-cutover/release/v0_23_0_multi_node_isolation_contract.md}"
CONTRACT_MANIFEST_PATH="${NTPRO_V23_CONTRACT_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_23_0_isolation_contract_manifest.json}"
GOLDEN_TRACE_MANIFEST_PATH="${NTPRO_V23_GOLDEN_TRACE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
DASHBOARD_SMOKE_PATH="${NTPRO_V23_DASHBOARD_SMOKE:-$ROOT_DIR/scripts/ai/verify_v23_dashboard_observability_smoke.sh}"
VERIFY_ONLY="${NTPRO_V23_STRICT_VERIFY_ONLY:-0}"

fail() {
  echo "v23 strict release provenance drift: $*" >&2
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

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v23 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"
for path in \
  "$RELEASE_MANIFEST_PATH" \
  "$RELEASE_NOTES_PATH" \
  "$READINESS_REPORT_PATH" \
  "$CONTRACT_PATH" \
  "$CONTRACT_MANIFEST_PATH" \
  "$GOLDEN_TRACE_MANIFEST_PATH" \
  "$DASHBOARD_SMOKE_PATH"; do
  [[ -f "$path" ]] || fail "missing strict provenance input: $path"
done

for task_id in V230-000 V230-001 V230-002 V230-003 V230-004 V230-005 V230-006 V230-007; do
  [[ -f "$ROOT_DIR/docs/rust-cutover/evidence/${task_id}.md" ]] || fail "missing evidence input: $task_id"
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
release_manifest_sha256="sha256:$(sha256_file "$RELEASE_MANIFEST_PATH")"
release_notes_sha256="sha256:$(sha256_file "$RELEASE_NOTES_PATH")"
readiness_report_sha256="sha256:$(sha256_file "$READINESS_REPORT_PATH")"
contract_sha256="sha256:$(sha256_file "$CONTRACT_PATH")"
contract_manifest_sha256="sha256:$(sha256_file "$CONTRACT_MANIFEST_PATH")"
golden_trace_manifest_sha256="sha256:$(sha256_file "$GOLDEN_TRACE_MANIFEST_PATH")"
dashboard_smoke_sha256="sha256:$(sha256_file "$DASHBOARD_SMOKE_PATH")"
release_manifest_rel="${RELEASE_MANIFEST_PATH#$ROOT_DIR/}"
release_notes_rel="${RELEASE_NOTES_PATH#$ROOT_DIR/}"
readiness_report_rel="${READINESS_REPORT_PATH#$ROOT_DIR/}"
contract_rel="${CONTRACT_PATH#$ROOT_DIR/}"
contract_manifest_rel="${CONTRACT_MANIFEST_PATH#$ROOT_DIR/}"
golden_trace_manifest_rel="${GOLDEN_TRACE_MANIFEST_PATH#$ROOT_DIR/}"
dashboard_smoke_rel="${DASHBOARD_SMOKE_PATH#$ROOT_DIR/}"
strict_manifest_rel="${MANIFEST_PATH#$ROOT_DIR/}"

PRODUCT_VERSION="$PRODUCT_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_TAG_EXISTS="$release_tag_exists" \
RELEASE_TAG_COMMIT="$release_tag_commit" \
RELEASE_TAG_TREE="$release_tag_tree" \
SOURCE_COMMIT="$source_commit" \
SOURCE_TREE="$source_tree" \
SOURCE_DIRTY="$source_dirty" \
RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
CONTRACT_MANIFEST_PATH="$CONTRACT_MANIFEST_PATH" \
CARGO_VERSION="$cargo_version" \
RUSTC_VERSION="$rustc_version" \
python3 <<'PY'
import json
import os
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
contract_manifest = json.loads(Path(os.environ["CONTRACT_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


for needle in (
    "Status: RELEASED",
    "Tag: `ntpro-rust-only-v0.23.0`",
    "Release name: `NTPRO Rust-only v0.23.0`",
    "Multi-Account / Multi-Strategy / Multi-Venue Node Isolation",
    "scripts/ai/verify_v23_strict_provenance.sh",
    "scripts/ai/publish_ntpro_release_after_gate.sh",
):
    require(needle in release_notes, f"release notes missing required marker: {needle}")

for needle in (
    "Milestone: `ntpro-rust-only-v0.23.0`",
    "Status: RELEASED",
    "v23 strict provenance = required",
    "V230-007 evidence",
    "#718 V230-007 = closed after tag, hosted gate, public release, and publication evidence were recorded",
):
    require(needle in readiness, f"readiness report missing required marker: {needle}")

for stale in (
    "ntpro-rust-only-v0.23.0-candidate",
    "#718 V230-007 = stays open until tag, hosted gate, public release, and publication evidence are recorded",
    "public release publication = pending",
    "tag gate run = pending",
    "tag gate result = pending",
    "RELEASE GATE CORRECTIVE FIX IN PROGRESS",
    "corrective fix in progress",
):
    require(stale not in readiness, f"readiness report contains stale marker: {stale}")
    require(stale not in release_notes, f"release notes contain stale marker: {stale}")

require(release_manifest.get("schema_version") == "ntpro.v230_release_manifest.v1", "release manifest schema mismatch")
require(release_manifest.get("task_id") == "V230-007", "release manifest task mismatch")
require(release_manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "release manifest product version mismatch")
require(release_manifest.get("release_status") == "released", "release manifest status mismatch")
require(contract_manifest.get("release") == "v0.23.0", "contract manifest release mismatch")

planned = release_manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == "NTPRO Rust-only v0.23.0", "planned release name mismatch")
require(planned.get("github_release_url") == "https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.0", "planned release URL mismatch")
require(planned.get("draft") is False, "planned draft flag mismatch")
require(planned.get("prerelease") is False, "planned prerelease flag mismatch")
require(planned.get("target_commitish") == "main", "planned target mismatch")

capability = release_manifest.get("capability") or {}
for key in ("multi_account_isolation", "multi_strategy_isolation", "multi_venue_node_isolation", "read_only_dashboard_observability", "strict_provenance"):
    require(capability.get(key) is True, f"capability flag mismatch: {key}")
for key in ("new_submit_capability", "production_order_mutation_expansion", "dashboard_operation_controls", "product_grade_live_trading_terminal"):
    require(capability.get(key) is False, f"forbidden capability flag mismatch: {key}")
PY

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
if [[ "$VERIFY_ONLY" != "1" ]]; then
  mkdir -p "$(dirname "$MANIFEST_PATH")"
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
  RELEASE_MANIFEST_REL="$release_manifest_rel" \
  RELEASE_MANIFEST_SHA256="$release_manifest_sha256" \
  RELEASE_NOTES_REL="$release_notes_rel" \
  RELEASE_NOTES_SHA256="$release_notes_sha256" \
  READINESS_REPORT_REL="$readiness_report_rel" \
  READINESS_REPORT_SHA256="$readiness_report_sha256" \
  CONTRACT_REL="$contract_rel" \
  CONTRACT_SHA256="$contract_sha256" \
  CONTRACT_MANIFEST_REL="$contract_manifest_rel" \
  CONTRACT_MANIFEST_SHA256="$contract_manifest_sha256" \
  GOLDEN_TRACE_MANIFEST_REL="$golden_trace_manifest_rel" \
  GOLDEN_TRACE_MANIFEST_SHA256="$golden_trace_manifest_sha256" \
  DASHBOARD_SMOKE_REL="$dashboard_smoke_rel" \
  DASHBOARD_SMOKE_SHA256="$dashboard_smoke_sha256" \
  python3 <<'PY'
import json
import os
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_REL"]).read_text(encoding="utf-8"))
manifest = {
    "schema_version": "ntpro.v230_strict_release_provenance_manifest.v1",
    "task_id": "V230-007",
    "target": "v23",
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
    "release_inputs": {
        "release_manifest": {"path": os.environ["RELEASE_MANIFEST_REL"], "sha256": os.environ["RELEASE_MANIFEST_SHA256"]},
        "release_notes": {"path": os.environ["RELEASE_NOTES_REL"], "sha256": os.environ["RELEASE_NOTES_SHA256"]},
        "readiness_report": {"path": os.environ["READINESS_REPORT_REL"], "sha256": os.environ["READINESS_REPORT_SHA256"]},
        "contract": {"path": os.environ["CONTRACT_REL"], "sha256": os.environ["CONTRACT_SHA256"]},
        "contract_manifest": {"path": os.environ["CONTRACT_MANIFEST_REL"], "sha256": os.environ["CONTRACT_MANIFEST_SHA256"]},
        "golden_trace_manifest": {"path": os.environ["GOLDEN_TRACE_MANIFEST_REL"], "sha256": os.environ["GOLDEN_TRACE_MANIFEST_SHA256"]},
        "dashboard_observability_smoke": {"path": os.environ["DASHBOARD_SMOKE_REL"], "sha256": os.environ["DASHBOARD_SMOKE_SHA256"]},
    },
    "v230_evidence": release_manifest.get("v230_evidence"),
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "read_model_replay": release_manifest.get("read_model_replay"),
    "publication_governance": release_manifest.get("publication_governance"),
    "release_gates": release_manifest.get("release_gates"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 fails if tracked files are dirty",
        "missing_tag": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails without the release tag",
        "tag_mismatch": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails when HEAD differs from the release tag",
        "manifest_mismatch": "release manifest, notes, readiness, contract, golden trace, and dashboard smoke hashes must match",
        "pre_gate_publication": "public GitHub Release publication must use the gate-before-publish entrypoint after hosted gate success",
    },
    "generated_at": os.environ["GENERATED_AT"],
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
elif [[ ! -f "$MANIFEST_PATH" ]]; then
  fail "manifest does not exist in verify-only mode: $MANIFEST_PATH"
fi

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
RELEASE_MANIFEST_REL="$release_manifest_rel" \
RELEASE_MANIFEST_SHA256="$release_manifest_sha256" \
RELEASE_NOTES_REL="$release_notes_rel" \
RELEASE_NOTES_SHA256="$release_notes_sha256" \
READINESS_REPORT_REL="$readiness_report_rel" \
READINESS_REPORT_SHA256="$readiness_report_sha256" \
CONTRACT_REL="$contract_rel" \
CONTRACT_SHA256="$contract_sha256" \
CONTRACT_MANIFEST_REL="$contract_manifest_rel" \
CONTRACT_MANIFEST_SHA256="$contract_manifest_sha256" \
GOLDEN_TRACE_MANIFEST_REL="$golden_trace_manifest_rel" \
GOLDEN_TRACE_MANIFEST_SHA256="$golden_trace_manifest_sha256" \
DASHBOARD_SMOKE_REL="$dashboard_smoke_rel" \
DASHBOARD_SMOKE_SHA256="$dashboard_smoke_sha256" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(manifest.get("schema_version") == "ntpro.v230_strict_release_provenance_manifest.v1", "strict manifest schema mismatch")
require(manifest.get("task_id") == "V230-007", "strict manifest task mismatch")
require(manifest.get("target") == "v23", "strict manifest target mismatch")
require(manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "strict manifest product version mismatch")
planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "strict manifest tag mismatch")
require(planned.get("tag_exists") == (os.environ["RELEASE_TAG_EXISTS"] == "true"), "strict manifest tag_exists mismatch")
require(planned.get("tag_commit") == os.environ["RELEASE_TAG_COMMIT"], "strict manifest tag commit mismatch")
require(planned.get("tag_tree") == os.environ["RELEASE_TAG_TREE"], "strict manifest tag tree mismatch")
source = manifest.get("source") or {}
require(source.get("commit") == os.environ["SOURCE_COMMIT"], "strict manifest source commit mismatch")
require(source.get("tree") == os.environ["SOURCE_TREE"], "strict manifest source tree mismatch")
require(source.get("tracked_worktree_dirty") == (os.environ["SOURCE_DIRTY"] == "true"), "strict manifest dirty flag mismatch")
toolchain = manifest.get("toolchain") or {}
require(toolchain.get("cargo_version") == os.environ["CARGO_VERSION"], "strict manifest cargo version mismatch")
require(toolchain.get("rustc_version") == os.environ["RUSTC_VERSION"], "strict manifest rustc version mismatch")

inputs = manifest.get("release_inputs") or {}
expected = {
    "release_manifest": (os.environ["RELEASE_MANIFEST_REL"], os.environ["RELEASE_MANIFEST_SHA256"]),
    "release_notes": (os.environ["RELEASE_NOTES_REL"], os.environ["RELEASE_NOTES_SHA256"]),
    "readiness_report": (os.environ["READINESS_REPORT_REL"], os.environ["READINESS_REPORT_SHA256"]),
    "contract": (os.environ["CONTRACT_REL"], os.environ["CONTRACT_SHA256"]),
    "contract_manifest": (os.environ["CONTRACT_MANIFEST_REL"], os.environ["CONTRACT_MANIFEST_SHA256"]),
    "golden_trace_manifest": (os.environ["GOLDEN_TRACE_MANIFEST_REL"], os.environ["GOLDEN_TRACE_MANIFEST_SHA256"]),
    "dashboard_observability_smoke": (os.environ["DASHBOARD_SMOKE_REL"], os.environ["DASHBOARD_SMOKE_SHA256"]),
}
for name, (path, sha256) in expected.items():
    item = inputs.get(name) or {}
    require(item.get("path") == path, f"strict manifest input path mismatch: {name}")
    require(item.get("sha256") == sha256, f"strict manifest input sha mismatch: {name}")

failure_paths = manifest.get("failure_paths") or {}
for key in ("dirty_worktree", "missing_tag", "tag_mismatch", "manifest_mismatch", "pre_gate_publication"):
    require(key in failure_paths, f"strict manifest failure path missing: {key}")
PY

echo "v23_strict_provenance status=ok product_version=$PRODUCT_VERSION release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_commit=$source_commit source_tree=$source_tree source_dirty=$source_dirty cargo_version=\"$cargo_version\" rustc_version=\"$rustc_version\" release_manifest=$RELEASE_MANIFEST_PATH release_manifest_sha256=$release_manifest_sha256 manifest=$MANIFEST_PATH"
