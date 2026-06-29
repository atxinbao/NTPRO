#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_RELEASE_STRICT_PRODUCT_VERSION:-v0.20.0}"
RELEASE_TAG="${NTPRO_RELEASE_STRICT_RELEASE_TAG:-ntpro-rust-only-v0.20.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.20.0"
MANIFEST_PATH="${NTPRO_RELEASE_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v200/v0_20_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V20_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_20_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V20_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_20_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V20_READINESS_REPORT:-$ROOT_DIR/docs/rust-cutover/release/v0_20_0_readiness_report.md}"
GOLDEN_TRACE_MANIFEST_PATH="${NTPRO_V20_GOLDEN_TRACE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
GATE_ROOT="${NTPRO_V20_RELEASE_GATE_ROOT:-$ROOT_DIR/target/ntpro-v200/v20-release-gates}"
NAUTILUS_BIN="${NTPRO_RELEASE_STRICT_NAUTILUS_BIN:-$ROOT_DIR/target/release/nautilus}"
VERIFY_ONLY="${NTPRO_RELEASE_STRICT_VERIFY_ONLY:-0}"

fail() {
  echo "v20 strict release provenance drift: $*" >&2
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

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v20 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"
[[ -f "$RELEASE_MANIFEST_PATH" ]] || fail "missing v0.20.0 release manifest: $RELEASE_MANIFEST_PATH"
[[ -f "$RELEASE_NOTES_PATH" ]] || fail "missing v0.20.0 release notes: $RELEASE_NOTES_PATH"
[[ -f "$READINESS_REPORT_PATH" ]] || fail "missing v0.20.0 readiness report: $READINESS_REPORT_PATH"
[[ -f "$GOLDEN_TRACE_MANIFEST_PATH" ]] || fail "missing v0.20 golden trace manifest: $GOLDEN_TRACE_MANIFEST_PATH"

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
if [[ "${NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG:-0}" == "1" ]]; then
  [[ "$release_tag_exists" == "true" ]] || fail "missing required local release tag: $RELEASE_TAG"
  [[ "$source_commit" == "$release_tag_commit" ]] || fail "HEAD $source_commit does not match $RELEASE_TAG commit $release_tag_commit"
fi

cargo_version="$(cargo --version)"
rustc_version="$(rustc --version)"
release_manifest_sha256="sha256:$(sha256_file "$RELEASE_MANIFEST_PATH")"
release_notes_sha256="sha256:$(sha256_file "$RELEASE_NOTES_PATH")"
readiness_report_sha256="sha256:$(sha256_file "$READINESS_REPORT_PATH")"
golden_trace_manifest_sha256="sha256:$(sha256_file "$GOLDEN_TRACE_MANIFEST_PATH")"
release_manifest_rel="${RELEASE_MANIFEST_PATH#$ROOT_DIR/}"
release_notes_rel="${RELEASE_NOTES_PATH#$ROOT_DIR/}"
readiness_report_rel="${READINESS_REPORT_PATH#$ROOT_DIR/}"
golden_trace_manifest_rel="${GOLDEN_TRACE_MANIFEST_PATH#$ROOT_DIR/}"
gate_root_rel="${GATE_ROOT#$ROOT_DIR/}"
strict_manifest_rel="${MANIFEST_PATH#$ROOT_DIR/}"

PRODUCT_VERSION="$PRODUCT_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_TAG_EXISTS="$release_tag_exists" \
RELEASE_TAG_COMMIT="$release_tag_commit" \
RELEASE_TAG_TREE="$release_tag_tree" \
SOURCE_COMMIT="$source_commit" \
SOURCE_TREE="$source_tree" \
RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
RELEASE_MANIFEST_REL="$release_manifest_rel" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
RELEASE_NOTES_REL="$release_notes_rel" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
READINESS_REPORT_REL="$readiness_report_rel" \
GOLDEN_TRACE_MANIFEST_PATH="$GOLDEN_TRACE_MANIFEST_PATH" \
GOLDEN_TRACE_MANIFEST_REL="$golden_trace_manifest_rel" \
GATE_ROOT_REL="$gate_root_rel" \
STRICT_MANIFEST_REL="$strict_manifest_rel" \
CARGO_VERSION="$cargo_version" \
RUSTC_VERSION="$rustc_version" \
python3 <<'PY'
import json
import os
import pathlib

release_manifest_path = pathlib.Path(os.environ["RELEASE_MANIFEST_PATH"])
release_notes_path = pathlib.Path(os.environ["RELEASE_NOTES_PATH"])
readiness_report_path = pathlib.Path(os.environ["READINESS_REPORT_PATH"])
golden_trace_manifest_path = pathlib.Path(os.environ["GOLDEN_TRACE_MANIFEST_PATH"])

release_manifest = json.loads(release_manifest_path.read_text(encoding="utf-8"))
golden_trace_manifest = json.loads(golden_trace_manifest_path.read_text(encoding="utf-8"))
release_notes = release_notes_path.read_text(encoding="utf-8")
readiness_report = readiness_report_path.read_text(encoding="utf-8")

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def require_text(text: str, needle: str, label: str) -> None:
    require(needle in text, f"{label} missing required marker: {needle}")

def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"release manifest boundary flag must be false: {key}")

for needle in (
    "Owner-Approved Production Order Lifecycle Foundation",
    "owner approval = required",
    "single submit attempt = required",
    "post-submit readback = required",
    "failure/no-retry evidence = required",
    "Dashboard order controls = not included",
    "implicit retry = not included",
    "strategy-driven production execution = not included",
    "general production trading platform claim = not included",
    "scripts/ai/verify_release_strict.sh v20",
):
    require_text(release_notes, needle, "release notes")
    require_text(readiness_report, needle, "readiness report")

require(release_manifest.get("schema_version") == "ntpro.v200_release_manifest.v1", "release manifest schema_version mismatch")
require(release_manifest.get("task_id") == "V200-012", "release manifest task_id mismatch")
require(release_manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "release manifest product_version mismatch")
require(release_manifest.get("release_status") == "ready_pending_publication", "release manifest status mismatch")

planned = release_manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == "NTPRO Rust-only v0.20.0", "planned release name mismatch")
require(planned.get("github_release_url") == "https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.20.0", "planned release URL mismatch")

source_provenance = release_manifest.get("source_provenance") or {}
require(source_provenance.get("planned_release_tag") == os.environ["RELEASE_TAG"], "source planned release tag mismatch")
require(source_provenance.get("actual_source_commit") is None, "actual source commit must be runtime-resolved")
require(source_provenance.get("actual_source_tree") is None, "actual source tree must be runtime-resolved")
require(source_provenance.get("actual_fields_resolved_by") == "scripts/ai/verify_release_strict.sh v20", "source resolver mismatch")
require(source_provenance.get("generated_manifest_path") == os.environ["STRICT_MANIFEST_REL"], "strict manifest path mismatch")

toolchain = release_manifest.get("toolchain") or {}
require(toolchain.get("cargo_version") == os.environ["CARGO_VERSION"], "release manifest cargo version mismatch")
require(toolchain.get("rustc_version") == os.environ["RUSTC_VERSION"], "release manifest rustc version mismatch")

binary_artifacts = release_manifest.get("binary_artifacts") or []
require(len(binary_artifacts) == 1, "release manifest binary_artifacts mismatch")
binary_contract = binary_artifacts[0]
require(binary_contract.get("name") == "nautilus", "release manifest binary name mismatch")
require(binary_contract.get("path") == "target/release/nautilus", "release manifest binary path mismatch")
require(binary_contract.get("build_profile") == "release", "release manifest binary build profile mismatch")
require(binary_contract.get("sha256") is None, "release manifest binary sha256 must be runtime-resolved")
require(binary_contract.get("bytes") is None, "release manifest binary bytes must be runtime-resolved")

release_inputs = release_manifest.get("release_inputs") or {}
require(release_inputs.get("release_notes_path") == os.environ["RELEASE_NOTES_REL"], "release notes path mismatch")
require(release_inputs.get("readiness_report_path") == os.environ["READINESS_REPORT_REL"], "readiness report path mismatch")
require(release_inputs.get("golden_trace_manifest_path") == os.environ["GOLDEN_TRACE_MANIFEST_REL"], "golden trace manifest path mismatch")
require(release_inputs.get("v20_gate_output_root") == os.environ["GATE_ROOT_REL"], "v20 gate output root mismatch")

release_gate_commands = {
    gate.get("command")
    for gate in (release_manifest.get("release_gates") or [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v20-release-gates",
    "scripts/ai/verify_release.sh v20-strict-provenance",
    "scripts/ai/verify_release_strict.sh v20",
):
    require(command in release_gate_commands, f"release manifest gate missing: {command}")

capability = release_manifest.get("capability") or {}
require(capability.get("name") == "Owner-Approved Production Order Lifecycle Foundation", "capability name mismatch")
require(capability.get("capability_expansion") == "owner_approved_production_order_lifecycle_foundation", "capability expansion mismatch")
require(capability.get("production_order_lifecycle") == "foundation_only", "production lifecycle scope mismatch")

boundary_flags = release_manifest.get("boundary_flags") or {}
for key in (
    "implicit_retry_allowed",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "bulk_order_allowed",
    "multi_account_execution_allowed",
    "multi_venue_execution_allowed",
    "strategy_driven_production_execution_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "raw_credential_plaintext_allowed",
    "raw_exchange_payload_publication_allowed",
    "general_production_trading_platform_claim",
    "binary_asset_publication_included",
):
    require_false(boundary_flags, key)

require(golden_trace_manifest.get("schema_version") == "golden-trace-release-scope-v1", "golden trace manifest schema mismatch")
cases = golden_trace_manifest.get("cases") or []
v20_cases = [
    case for case in cases
    if str(case.get("trace", "")).endswith("production_order_lifecycle_schema.jsonl")
]
require(len(v20_cases) == 6, "golden trace manifest missing v20 production order lifecycle cases")
PY

if [[ "$VERIFY_ONLY" != "1" && "${NTPRO_RELEASE_STRICT_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build -p nautilus-cli --release --bin nautilus
fi

[[ -x "$NAUTILUS_BIN" ]] || fail "missing release binary: $NAUTILUS_BIN"
if [[ "$NAUTILUS_BIN" != */target/release/nautilus && "${NTPRO_RELEASE_STRICT_ALLOW_NON_RELEASE_BIN:-0}" != "1" ]]; then
  fail "strict gate requires target/release/nautilus, got: $NAUTILUS_BIN"
fi

if [[ "$VERIFY_ONLY" != "1" && "${NTPRO_RELEASE_STRICT_SKIP_V20_GATES:-0}" != "1" ]]; then
  rm -rf "$GATE_ROOT"
  mkdir -p "$GATE_ROOT"
  NTPRO_V20_SKIP_BUILD=1 \
  NTPRO_V20_NAUTILUS_BIN="$NAUTILUS_BIN" \
  NTPRO_V20_RELEASE_GATE_ROOT="$GATE_ROOT" \
  NTPRO_SOURCE_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_SOURCE_COMMIT="$source_commit" \
    scripts/ai/verify_v20_release_gates.sh
fi

binary_version="$("$NAUTILUS_BIN" --version)"
binary_sha256="sha256:$(sha256_file "$NAUTILUS_BIN")"
binary_bytes="$(wc -c < "$NAUTILUS_BIN" | tr -d ' ')"
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
  NAUTILUS_BIN="$NAUTILUS_BIN" \
  BINARY_SHA256="$binary_sha256" \
  BINARY_BYTES="$binary_bytes" \
  BINARY_VERSION="$binary_version" \
  GENERATED_AT="$generated_at" \
  RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
  RELEASE_MANIFEST_REL="$release_manifest_rel" \
  RELEASE_MANIFEST_SHA256="$release_manifest_sha256" \
  RELEASE_NOTES_REL="$release_notes_rel" \
  RELEASE_NOTES_SHA256="$release_notes_sha256" \
  READINESS_REPORT_REL="$readiness_report_rel" \
  READINESS_REPORT_SHA256="$readiness_report_sha256" \
  GOLDEN_TRACE_MANIFEST_REL="$golden_trace_manifest_rel" \
  GOLDEN_TRACE_MANIFEST_SHA256="$golden_trace_manifest_sha256" \
  GATE_ROOT_REL="$gate_root_rel" \
  python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
release_manifest = json.loads(pathlib.Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
manifest = {
    "schema_version": "ntpro.v200_strict_release_provenance_manifest.v1",
    "task_id": "V200-012",
    "target": "v20",
    "product_version": os.environ["PRODUCT_VERSION"],
    "planned_release": {
        "tag": os.environ["RELEASE_TAG"],
        "tag_exists": os.environ["RELEASE_TAG_EXISTS"] == "true",
        "resolved_commit": os.environ["RELEASE_TAG_COMMIT"],
        "resolved_tree": os.environ["RELEASE_TAG_TREE"],
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
    "release_manifest": {
        "path": os.environ["RELEASE_MANIFEST_REL"],
        "sha256": os.environ["RELEASE_MANIFEST_SHA256"],
        "schema_version": release_manifest.get("schema_version"),
        "task_id": release_manifest.get("task_id"),
        "product_version": release_manifest.get("product_version"),
        "release_status": release_manifest.get("release_status"),
    },
    "release_inputs": {
        "release_notes": {
            "path": os.environ["RELEASE_NOTES_REL"],
            "sha256": os.environ["RELEASE_NOTES_SHA256"],
        },
        "readiness_report": {
            "path": os.environ["READINESS_REPORT_REL"],
            "sha256": os.environ["READINESS_REPORT_SHA256"],
        },
        "golden_trace_manifest": {
            "path": os.environ["GOLDEN_TRACE_MANIFEST_REL"],
            "sha256": os.environ["GOLDEN_TRACE_MANIFEST_SHA256"],
        },
        "v20_gate_output_root": os.environ["GATE_ROOT_REL"],
    },
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "release_gates": release_manifest.get("release_gates"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 rejects tracked source drift",
        "tag_mismatch": "NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 rejects HEAD/tag mismatch",
        "manifest_mismatch": "schema, task, product, gate, and boundary manifest fields are strict",
        "binary_hash_mismatch": "generated binary sha256/bytes are recorded in this manifest",
        "missing_v200_evidence": "v20 release gates reject missing V200 evidence or trace cases",
    },
    "generated_at": os.environ["GENERATED_AT"],
}
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
fi

echo "strict_release_provenance status=ok target=v20 product_version=$PRODUCT_VERSION release_tag=$RELEASE_TAG release_tag_exists=$release_tag_exists release_tag_commit=$release_tag_commit release_tag_tree=$release_tag_tree source_commit=$source_commit source_tree=$source_tree source_dirty=$source_dirty cargo_version=\"$cargo_version\" rustc_version=\"$rustc_version\" binary_path=$NAUTILUS_BIN binary_sha256=$binary_sha256 binary_bytes=$binary_bytes release_manifest=$RELEASE_MANIFEST_PATH release_manifest_sha256=$release_manifest_sha256 golden_trace_manifest=$GOLDEN_TRACE_MANIFEST_PATH golden_trace_manifest_sha256=$golden_trace_manifest_sha256 v20_gate_output_root=$GATE_ROOT manifest=$MANIFEST_PATH"
