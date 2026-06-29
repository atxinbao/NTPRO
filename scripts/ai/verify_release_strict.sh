#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

target="${1:-}"
if [[ "$target" != "v18" && "$target" != "v19" && "$target" != "v20" ]]; then
  echo "usage: scripts/ai/verify_release_strict.sh v18|v19|v20" >&2
  exit 2
fi

if [[ "$target" == "v20" ]]; then
  exec scripts/ai/verify_v20_strict_provenance.sh
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

if [[ "$target" == "v19" ]]; then
  V19_PRODUCT_VERSION="${NTPRO_RELEASE_STRICT_PRODUCT_VERSION:-v0.19.0}"
  V19_RELEASE_TAG="${NTPRO_RELEASE_STRICT_RELEASE_TAG:-ntpro-rust-only-v0.19.0}"
  V19_EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.19.0"
  V19_MANIFEST_PATH="${NTPRO_RELEASE_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v190/v0_19_strict_release_manifest.json}"
  V19_RELEASE_MANIFEST_PATH="${NTPRO_V19_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_19_0_release_manifest.json}"
  V19_RELEASE_NOTES_PATH="${NTPRO_V19_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_19_0_release_notes.md}"
  V19_READINESS_REPORT_PATH="${NTPRO_V19_READINESS_REPORT:-$ROOT_DIR/docs/rust-cutover/release/v0_19_0_readiness_report.md}"
  V19_GOLDEN_TRACE_MANIFEST_PATH="${NTPRO_V19_GOLDEN_TRACE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
  V19_GATE_ROOT="${NTPRO_V19_RELEASE_GATE_ROOT:-$ROOT_DIR/target/ntpro-v190/v19-release-gates}"
  V19_NAUTILUS_BIN="${NTPRO_RELEASE_STRICT_NAUTILUS_BIN:-$ROOT_DIR/target/release/nautilus}"
  V19_VERIFY_ONLY="${NTPRO_RELEASE_STRICT_VERIFY_ONLY:-0}"

  [[ "$V19_RELEASE_TAG" == "$V19_EXPECTED_RELEASE_TAG" ]] || fail "v19 strict source release tag must be $V19_EXPECTED_RELEASE_TAG, got: $V19_RELEASE_TAG"
  [[ -f "$V19_RELEASE_MANIFEST_PATH" ]] || fail "missing v0.19.0 release manifest: $V19_RELEASE_MANIFEST_PATH"
  [[ -f "$V19_RELEASE_NOTES_PATH" ]] || fail "missing v0.19.0 release notes: $V19_RELEASE_NOTES_PATH"
  [[ -f "$V19_READINESS_REPORT_PATH" ]] || fail "missing v0.19.0 readiness report: $V19_READINESS_REPORT_PATH"
  [[ -f "$V19_GOLDEN_TRACE_MANIFEST_PATH" ]] || fail "missing v0.19 golden trace manifest: $V19_GOLDEN_TRACE_MANIFEST_PATH"

  v19_release_tag_commit="$(git rev-list -n 1 "$V19_RELEASE_TAG")"
  v19_release_tag_tree="$(git rev-parse "$V19_RELEASE_TAG^{tree}")"
  v19_source_commit="$(git rev-parse HEAD)"
  v19_source_tree="$(git rev-parse HEAD^{tree})"
  v19_tracked_status="$(git status --porcelain --untracked-files=no)"
  v19_source_dirty="false"
  if [[ -n "$v19_tracked_status" ]]; then
    v19_source_dirty="true"
  fi
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" && "$v19_source_dirty" == "true" ]]; then
    git status --short >&2
    fail "strict release gate requires a clean tracked working tree"
  fi
  if [[ "${NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG:-0}" == "1" && "$v19_source_commit" != "$v19_release_tag_commit" ]]; then
    fail "HEAD $v19_source_commit does not match $V19_RELEASE_TAG commit $v19_release_tag_commit"
  fi

  v19_cargo_version="$(cargo --version)"
  v19_rustc_version="$(rustc --version)"
  v19_release_manifest_sha256="sha256:$(sha256_file "$V19_RELEASE_MANIFEST_PATH")"
  v19_release_notes_sha256="sha256:$(sha256_file "$V19_RELEASE_NOTES_PATH")"
  v19_readiness_report_sha256="sha256:$(sha256_file "$V19_READINESS_REPORT_PATH")"
  v19_golden_trace_manifest_sha256="sha256:$(sha256_file "$V19_GOLDEN_TRACE_MANIFEST_PATH")"
  v19_release_manifest_rel="${V19_RELEASE_MANIFEST_PATH#$ROOT_DIR/}"
  v19_release_notes_rel="${V19_RELEASE_NOTES_PATH#$ROOT_DIR/}"
  v19_readiness_report_rel="${V19_READINESS_REPORT_PATH#$ROOT_DIR/}"
  v19_golden_trace_manifest_rel="${V19_GOLDEN_TRACE_MANIFEST_PATH#$ROOT_DIR/}"
  v19_gate_root_rel="${V19_GATE_ROOT#$ROOT_DIR/}"
  v19_strict_manifest_rel="${V19_MANIFEST_PATH#$ROOT_DIR/}"

  V19_PRODUCT_VERSION="$V19_PRODUCT_VERSION" \
  V19_RELEASE_TAG="$V19_RELEASE_TAG" \
  V19_RELEASE_TAG_COMMIT="$v19_release_tag_commit" \
  V19_RELEASE_TAG_TREE="$v19_release_tag_tree" \
  V19_RELEASE_MANIFEST_PATH="$V19_RELEASE_MANIFEST_PATH" \
  V19_RELEASE_MANIFEST_REL="$v19_release_manifest_rel" \
  V19_RELEASE_NOTES_PATH="$V19_RELEASE_NOTES_PATH" \
  V19_RELEASE_NOTES_REL="$v19_release_notes_rel" \
  V19_READINESS_REPORT_PATH="$V19_READINESS_REPORT_PATH" \
  V19_READINESS_REPORT_REL="$v19_readiness_report_rel" \
  V19_GOLDEN_TRACE_MANIFEST_PATH="$V19_GOLDEN_TRACE_MANIFEST_PATH" \
  V19_GOLDEN_TRACE_MANIFEST_REL="$v19_golden_trace_manifest_rel" \
  V19_GATE_ROOT_REL="$v19_gate_root_rel" \
  V19_STRICT_MANIFEST_REL="$v19_strict_manifest_rel" \
  V19_CARGO_VERSION="$v19_cargo_version" \
  V19_RUSTC_VERSION="$v19_rustc_version" \
  python3 <<'PY'
import json
import os
import pathlib

release_manifest_path = pathlib.Path(os.environ["V19_RELEASE_MANIFEST_PATH"])
release_notes_path = pathlib.Path(os.environ["V19_RELEASE_NOTES_PATH"])
readiness_report_path = pathlib.Path(os.environ["V19_READINESS_REPORT_PATH"])
golden_trace_manifest_path = pathlib.Path(os.environ["V19_GOLDEN_TRACE_MANIFEST_PATH"])

release_manifest = json.loads(release_manifest_path.read_text(encoding="utf-8"))
golden_trace_manifest = json.loads(golden_trace_manifest_path.read_text(encoding="utf-8"))
release_notes = release_notes_path.read_text(encoding="utf-8")
readiness_report = readiness_report_path.read_text(encoding="utf-8")

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def require_text(text: str, needle: str, label: str) -> None:
    require(needle in text, f"{label} missing required marker: {needle}")

def require_absent(text: str, needle: str, label: str) -> None:
    require(needle.lower() not in text.lower(), f"{label} contains stale marker: {needle}")

def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"release manifest boundary flag must be false: {key}")

for needle in (
    "Status: RELEASED",
    "Tag: `ntpro-rust-only-v0.19.0`",
    "Release name: `NTPRO Rust-only v0.19.0`",
    "Release commit: `e72a7d29f052757be6c185c1f9ba007ef7146ee0`",
    "Hosted release gate run: `28314859483`",
    "scripts/ai/verify_release_strict.sh v19",
):
    require_text(release_notes, needle, "release notes")

for needle in (
    "Milestone: `ntpro-rust-only-v0.19.0`",
    "Status: RELEASED",
    "release commit = e72a7d29f052757be6c185c1f9ba007ef7146ee0",
    "hosted release gate result = PASS",
    "scripts/ai/verify_release_strict.sh v19",
):
    require_text(readiness_report, needle, "readiness report")

for stale in (
    "RELEASE CANDIDATE",
    "candidate",
    "pending",
    "GitHub Release = pending",
    "hosted release gate = pending",
):
    require_absent(release_notes, stale, "release notes")
    require_absent(readiness_report, stale, "readiness report")

require(release_manifest.get("schema_version") == "ntpro.v190_release_manifest.v1", "release manifest schema_version mismatch")
require(release_manifest.get("task_id") == "V191-005", "release manifest task_id mismatch")
require(release_manifest.get("product_version") == os.environ["V19_PRODUCT_VERSION"], "release manifest product_version mismatch")
require(release_manifest.get("release_status") == "published", "release manifest status mismatch")

formal_release = release_manifest.get("formal_release") or {}
require(formal_release.get("tag") == os.environ["V19_RELEASE_TAG"], "formal release tag mismatch")
require(formal_release.get("name") == "NTPRO Rust-only v0.19.0", "formal release name mismatch")
require(formal_release.get("github_release_url") == "https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.19.0", "formal release URL mismatch")
require(formal_release.get("commit") == os.environ["V19_RELEASE_TAG_COMMIT"], "formal release commit mismatch")
require(formal_release.get("tree") == os.environ["V19_RELEASE_TAG_TREE"], "formal release tree mismatch")
require(formal_release.get("published_at") == "2026-06-28T08:40:28Z", "formal release published_at mismatch")
require(formal_release.get("is_draft") is False, "formal release draft flag mismatch")
require(formal_release.get("is_prerelease") is False, "formal release prerelease flag mismatch")

source_provenance = release_manifest.get("source_provenance") or {}
require(source_provenance.get("release_tag") == os.environ["V19_RELEASE_TAG"], "source release tag mismatch")
require(source_provenance.get("release_tag_commit") == os.environ["V19_RELEASE_TAG_COMMIT"], "source release tag commit mismatch")
require(source_provenance.get("release_tag_tree") == os.environ["V19_RELEASE_TAG_TREE"], "source release tag tree mismatch")
require(source_provenance.get("source_commit_resolved_by") == "scripts/ai/verify_release_strict.sh v19", "source commit resolver mismatch")
require(source_provenance.get("source_tree_resolved_by") == "scripts/ai/verify_release_strict.sh v19", "source tree resolver mismatch")
require(source_provenance.get("generated_manifest_path") == os.environ["V19_STRICT_MANIFEST_REL"], "strict manifest path mismatch")

toolchain = release_manifest.get("toolchain") or {}
require(toolchain.get("cargo_version") == os.environ["V19_CARGO_VERSION"], "release manifest cargo version mismatch")
require(toolchain.get("rustc_version") == os.environ["V19_RUSTC_VERSION"], "release manifest rustc version mismatch")

binary_artifacts = release_manifest.get("binary_artifacts") or []
require(len(binary_artifacts) == 1, "release manifest binary_artifacts mismatch")
binary_contract = binary_artifacts[0]
require(binary_contract.get("name") == "nautilus", "release manifest binary name mismatch")
require(binary_contract.get("path") == "target/release/nautilus", "release manifest binary path mismatch")
require(binary_contract.get("build_profile") == "release", "release manifest binary build profile mismatch")
require(binary_contract.get("sha256") is None, "release manifest binary sha256 must be runtime-resolved")
require(binary_contract.get("bytes") is None, "release manifest binary bytes must be runtime-resolved")
require(binary_contract.get("actual_fields_resolved_by") == "scripts/ai/verify_release_strict.sh v19", "release manifest binary resolver mismatch")

release_inputs = release_manifest.get("release_inputs") or {}
require(release_inputs.get("release_notes_path") == os.environ["V19_RELEASE_NOTES_REL"], "release notes path mismatch")
require(release_inputs.get("readiness_report_path") == os.environ["V19_READINESS_REPORT_REL"], "readiness report path mismatch")
require(release_inputs.get("golden_trace_manifest_path") == os.environ["V19_GOLDEN_TRACE_MANIFEST_REL"], "golden trace manifest path mismatch")
require(release_inputs.get("v19_gate_output_root") == os.environ["V19_GATE_ROOT_REL"], "v19 gate output root mismatch")

release_gate_commands = {
    gate.get("command")
    for gate in (release_manifest.get("release_gates") or [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh release-surface-current-guard",
    "scripts/ai/verify_release.sh release-publication-guard",
    "scripts/ai/verify_release.sh v19-release-gates",
    "scripts/ai/verify_release.sh v19-strict-provenance",
    "scripts/ai/verify_release_strict.sh v19",
):
    require(command in release_gate_commands, f"release manifest gate missing: {command}")

capability = release_manifest.get("capability") or {}
require(capability.get("name") == "Owner-Approved Single-Shot Actual Cancel", "capability name mismatch")
require(capability.get("capability_expansion") == "none_patch_hardening_only", "capability expansion mismatch")
require(capability.get("actual_cancel_scope") == "owner_approved_single_shot_only", "actual cancel scope mismatch")
require(capability.get("production_order_submit_lifecycle") == "not_included", "production submit lifecycle mismatch")

boundary_flags = release_manifest.get("boundary_flags") or {}
for key in (
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "bulk_cancel_allowed",
    "cancel_all_allowed",
    "multi_account_cancel_allowed",
    "multi_strategy_cancel_allowed",
    "multi_venue_cancel_allowed",
    "retry_allowed",
    "second_cancel_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_approval_controls_enabled",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_submit_lifecycle_included",
    "binary_asset_publication_included",
):
    require_false(boundary_flags, key)

validation_artifacts = release_manifest.get("validation_artifacts") or {}
require(validation_artifacts.get("strict_release_provenance_manifest_path") == os.environ["V19_STRICT_MANIFEST_REL"], "strict validation artifact path mismatch")
require(validation_artifacts.get("v19_gate_output_root") == os.environ["V19_GATE_ROOT_REL"], "v19 gate validation artifact path mismatch")

require(golden_trace_manifest.get("schema_version") == "golden-trace-release-scope-v1", "golden trace manifest schema mismatch")
cases = golden_trace_manifest.get("cases") or []
actual_cancel_cases = [
    case for case in cases
    if str(case.get("case_id", "")).startswith("actual_cancel.")
    or str(case.get("trace", "")).endswith("actual_cancel_schema.jsonl")
]
require(len(actual_cancel_cases) >= 10, "golden trace manifest missing actual cancel release cases")
PY

  if [[ "$V19_VERIFY_ONLY" != "1" && "${NTPRO_RELEASE_STRICT_SKIP_BUILD:-0}" != "1" ]]; then
    cargo build -p nautilus-cli --release --bin nautilus
  fi

  [[ -x "$V19_NAUTILUS_BIN" ]] || fail "missing release binary: $V19_NAUTILUS_BIN"
  if [[ "$V19_NAUTILUS_BIN" != */target/release/nautilus && "${NTPRO_RELEASE_STRICT_ALLOW_NON_RELEASE_BIN:-0}" != "1" ]]; then
    fail "strict gate requires target/release/nautilus, got: $V19_NAUTILUS_BIN"
  fi

  if [[ "$V19_VERIFY_ONLY" != "1" && "${NTPRO_RELEASE_STRICT_SKIP_V19_GATES:-0}" != "1" ]]; then
    rm -rf "$V19_GATE_ROOT"
    mkdir -p "$V19_GATE_ROOT"
    NTPRO_V19_SKIP_BUILD=1 \
    NTPRO_V19_NAUTILUS_BIN="$V19_NAUTILUS_BIN" \
    NTPRO_V19_RELEASE_GATE_ROOT="$V19_GATE_ROOT" \
    NTPRO_SOURCE_RELEASE_TAG="$V19_RELEASE_TAG" \
    NTPRO_SOURCE_COMMIT="$v19_source_commit" \
      scripts/ai/verify_v19_release_gates.sh
  fi

  v19_binary_version="$("$V19_NAUTILUS_BIN" --version)"
  v19_binary_sha256="sha256:$(sha256_file "$V19_NAUTILUS_BIN")"
  v19_binary_bytes="$(wc -c < "$V19_NAUTILUS_BIN" | tr -d ' ')"
  v19_generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  if [[ "$V19_VERIFY_ONLY" != "1" ]]; then
    mkdir -p "$(dirname "$V19_MANIFEST_PATH")"
    MANIFEST_PATH="$V19_MANIFEST_PATH" \
    V19_PRODUCT_VERSION="$V19_PRODUCT_VERSION" \
    V19_RELEASE_TAG="$V19_RELEASE_TAG" \
    V19_RELEASE_TAG_COMMIT="$v19_release_tag_commit" \
    V19_RELEASE_TAG_TREE="$v19_release_tag_tree" \
    V19_SOURCE_COMMIT="$v19_source_commit" \
    V19_SOURCE_TREE="$v19_source_tree" \
    V19_SOURCE_DIRTY="$v19_source_dirty" \
    V19_CARGO_VERSION="$v19_cargo_version" \
    V19_RUSTC_VERSION="$v19_rustc_version" \
    V19_NAUTILUS_BIN="$V19_NAUTILUS_BIN" \
    V19_BINARY_SHA256="$v19_binary_sha256" \
    V19_BINARY_BYTES="$v19_binary_bytes" \
    V19_BINARY_VERSION="$v19_binary_version" \
    V19_GENERATED_AT="$v19_generated_at" \
    V19_RELEASE_MANIFEST_PATH="$V19_RELEASE_MANIFEST_PATH" \
    V19_RELEASE_MANIFEST_REL="$v19_release_manifest_rel" \
    V19_RELEASE_MANIFEST_SHA256="$v19_release_manifest_sha256" \
    V19_RELEASE_NOTES_REL="$v19_release_notes_rel" \
    V19_RELEASE_NOTES_SHA256="$v19_release_notes_sha256" \
    V19_READINESS_REPORT_REL="$v19_readiness_report_rel" \
    V19_READINESS_REPORT_SHA256="$v19_readiness_report_sha256" \
    V19_GOLDEN_TRACE_MANIFEST_REL="$v19_golden_trace_manifest_rel" \
    V19_GOLDEN_TRACE_MANIFEST_SHA256="$v19_golden_trace_manifest_sha256" \
    V19_GATE_ROOT_REL="$v19_gate_root_rel" \
    python3 <<'PY'
import json
import os
import pathlib

manifest_path = pathlib.Path(os.environ["MANIFEST_PATH"])
release_manifest_path = pathlib.Path(os.environ["V19_RELEASE_MANIFEST_PATH"])
release_manifest = json.loads(release_manifest_path.read_text(encoding="utf-8"))
manifest = {
    "schema_version": "ntpro.v190_strict_release_provenance_manifest.v1",
    "task_id": "V191-005",
    "target": "v19",
    "product_version": os.environ["V19_PRODUCT_VERSION"],
    "release_manifest": {
        "path": os.environ["V19_RELEASE_MANIFEST_REL"],
        "sha256": os.environ["V19_RELEASE_MANIFEST_SHA256"],
        "schema_version": release_manifest.get("schema_version"),
        "task_id": release_manifest.get("task_id"),
        "product_version": release_manifest.get("product_version"),
        "release_status": release_manifest.get("release_status"),
    },
    "source_release_tag": {
        "tag": os.environ["V19_RELEASE_TAG"],
        "commit": os.environ["V19_RELEASE_TAG_COMMIT"],
        "tree": os.environ["V19_RELEASE_TAG_TREE"],
    },
    "source": {
        "commit": os.environ["V19_SOURCE_COMMIT"],
        "tree": os.environ["V19_SOURCE_TREE"],
        "tracked_worktree_dirty": os.environ["V19_SOURCE_DIRTY"] == "true",
        "tag_match_required": False,
    },
    "toolchain": {
        "cargo_version": os.environ["V19_CARGO_VERSION"],
        "rustc_version": os.environ["V19_RUSTC_VERSION"],
    },
    "binary": {
        "name": "nautilus",
        "path": os.environ["V19_NAUTILUS_BIN"],
        "bytes": int(os.environ["V19_BINARY_BYTES"]),
        "sha256": os.environ["V19_BINARY_SHA256"],
        "version_output": os.environ["V19_BINARY_VERSION"],
        "build_profile": "release",
        "source_commit": os.environ["V19_SOURCE_COMMIT"],
        "source_tree": os.environ["V19_SOURCE_TREE"],
    },
    "release_inputs": {
        "release_notes": {
            "path": os.environ["V19_RELEASE_NOTES_REL"],
            "sha256": os.environ["V19_RELEASE_NOTES_SHA256"],
        },
        "readiness_report": {
            "path": os.environ["V19_READINESS_REPORT_REL"],
            "sha256": os.environ["V19_READINESS_REPORT_SHA256"],
        },
        "golden_trace_manifest": {
            "path": os.environ["V19_GOLDEN_TRACE_MANIFEST_REL"],
            "sha256": os.environ["V19_GOLDEN_TRACE_MANIFEST_SHA256"],
        },
        "v19_gate_output_root": os.environ["V19_GATE_ROOT_REL"],
    },
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1",
        "tag_mismatch": "NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1",
        "manifest_mismatch": "NTPRO_RELEASE_STRICT_VERIFY_ONLY=1",
        "binary_hash_mismatch": "NTPRO_RELEASE_STRICT_VERIFY_ONLY=1",
        "stale_release_notes_or_readiness": "release docs must be RELEASED and non-candidate",
    },
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "release_gates": release_manifest.get("release_gates"),
    "generated_at": os.environ["V19_GENERATED_AT"],
}
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  elif [[ ! -f "$V19_MANIFEST_PATH" ]]; then
    fail "manifest does not exist in verify-only mode: $V19_MANIFEST_PATH"
  fi

  MANIFEST_PATH="$V19_MANIFEST_PATH" \
  V19_PRODUCT_VERSION="$V19_PRODUCT_VERSION" \
  V19_RELEASE_TAG="$V19_RELEASE_TAG" \
  V19_RELEASE_TAG_COMMIT="$v19_release_tag_commit" \
  V19_RELEASE_TAG_TREE="$v19_release_tag_tree" \
  V19_SOURCE_COMMIT="$v19_source_commit" \
  V19_SOURCE_TREE="$v19_source_tree" \
  V19_SOURCE_DIRTY="$v19_source_dirty" \
  V19_CARGO_VERSION="$v19_cargo_version" \
  V19_RUSTC_VERSION="$v19_rustc_version" \
  V19_NAUTILUS_BIN="$V19_NAUTILUS_BIN" \
  V19_BINARY_SHA256="$v19_binary_sha256" \
  V19_BINARY_BYTES="$v19_binary_bytes" \
  V19_BINARY_VERSION="$v19_binary_version" \
  V19_RELEASE_MANIFEST_REL="$v19_release_manifest_rel" \
  V19_RELEASE_MANIFEST_SHA256="$v19_release_manifest_sha256" \
  V19_RELEASE_NOTES_REL="$v19_release_notes_rel" \
  V19_RELEASE_NOTES_SHA256="$v19_release_notes_sha256" \
  V19_READINESS_REPORT_REL="$v19_readiness_report_rel" \
  V19_READINESS_REPORT_SHA256="$v19_readiness_report_sha256" \
  V19_GOLDEN_TRACE_MANIFEST_REL="$v19_golden_trace_manifest_rel" \
  V19_GOLDEN_TRACE_MANIFEST_SHA256="$v19_golden_trace_manifest_sha256" \
  V19_GATE_ROOT_REL="$v19_gate_root_rel" \
  python3 <<'PY'
import json
import os
import pathlib

manifest = json.loads(pathlib.Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

require(manifest.get("schema_version") == "ntpro.v190_strict_release_provenance_manifest.v1", "schema_version mismatch")
require(manifest.get("task_id") == "V191-005", "task_id mismatch")
require(manifest.get("target") == "v19", "target mismatch")
require(manifest.get("product_version") == os.environ["V19_PRODUCT_VERSION"], "product_version mismatch")

release_manifest_ref = manifest.get("release_manifest") or {}
require(release_manifest_ref.get("path") == os.environ["V19_RELEASE_MANIFEST_REL"], "release manifest path mismatch")
require(release_manifest_ref.get("sha256") == os.environ["V19_RELEASE_MANIFEST_SHA256"], "release manifest sha256 mismatch")
require(release_manifest_ref.get("schema_version") == "ntpro.v190_release_manifest.v1", "release manifest schema mismatch")
require(release_manifest_ref.get("task_id") == "V191-005", "release manifest task mismatch")
require(release_manifest_ref.get("product_version") == os.environ["V19_PRODUCT_VERSION"], "release manifest product mismatch")
require(release_manifest_ref.get("release_status") == "published", "release manifest status mismatch")

source_release_tag = manifest.get("source_release_tag") or {}
require(source_release_tag.get("tag") == os.environ["V19_RELEASE_TAG"], "source release tag mismatch")
require(source_release_tag.get("commit") == os.environ["V19_RELEASE_TAG_COMMIT"], "source release tag commit mismatch")
require(source_release_tag.get("tree") == os.environ["V19_RELEASE_TAG_TREE"], "source release tag tree mismatch")

source = manifest.get("source") or {}
require(source.get("commit") == os.environ["V19_SOURCE_COMMIT"], "source commit mismatch")
require(source.get("tree") == os.environ["V19_SOURCE_TREE"], "source tree mismatch")
require(source.get("tracked_worktree_dirty") == (os.environ["V19_SOURCE_DIRTY"] == "true"), "source dirty mismatch")

toolchain = manifest.get("toolchain") or {}
require(toolchain.get("cargo_version") == os.environ["V19_CARGO_VERSION"], "cargo version mismatch")
require(toolchain.get("rustc_version") == os.environ["V19_RUSTC_VERSION"], "rustc version mismatch")

binary = manifest.get("binary") or {}
require(binary.get("name") == "nautilus", "binary name mismatch")
require(binary.get("path") == os.environ["V19_NAUTILUS_BIN"], "binary path mismatch")
require(binary.get("bytes") == int(os.environ["V19_BINARY_BYTES"]), "binary byte count mismatch")
require(binary.get("sha256") == os.environ["V19_BINARY_SHA256"], "binary sha256 mismatch")
require(binary.get("version_output") == os.environ["V19_BINARY_VERSION"], "binary version output mismatch")
require(binary.get("build_profile") == "release", "binary build profile mismatch")
require(binary.get("source_commit") == os.environ["V19_SOURCE_COMMIT"], "binary source commit mismatch")
require(binary.get("source_tree") == os.environ["V19_SOURCE_TREE"], "binary source tree mismatch")

release_inputs = manifest.get("release_inputs") or {}
for key, path_env, sha_env in (
    ("release_notes", "V19_RELEASE_NOTES_REL", "V19_RELEASE_NOTES_SHA256"),
    ("readiness_report", "V19_READINESS_REPORT_REL", "V19_READINESS_REPORT_SHA256"),
    ("golden_trace_manifest", "V19_GOLDEN_TRACE_MANIFEST_REL", "V19_GOLDEN_TRACE_MANIFEST_SHA256"),
):
    item = release_inputs.get(key) or {}
    require(item.get("path") == os.environ[path_env], f"{key} path mismatch")
    require(item.get("sha256") == os.environ[sha_env], f"{key} sha256 mismatch")
require(release_inputs.get("v19_gate_output_root") == os.environ["V19_GATE_ROOT_REL"], "v19 gate output root mismatch")

failure_paths = manifest.get("failure_paths") or {}
for key in ("dirty_worktree", "tag_mismatch", "manifest_mismatch", "binary_hash_mismatch", "stale_release_notes_or_readiness"):
    require(key in failure_paths, f"failure path missing: {key}")

boundary_flags = manifest.get("boundary_flags") or {}
for key in (
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "bulk_cancel_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_submit_lifecycle_included",
):
    require(boundary_flags.get(key) is False, f"boundary flag must be false: {key}")

release_gate_commands = {
    gate.get("command")
    for gate in (manifest.get("release_gates") or [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v19-release-gates",
    "scripts/ai/verify_release.sh v19-strict-provenance",
    "scripts/ai/verify_release_strict.sh v19",
):
    require(command in release_gate_commands, f"release gate missing from strict manifest: {command}")
PY

  echo "strict_release_provenance status=ok target=v19 product_version=$V19_PRODUCT_VERSION release_tag=$V19_RELEASE_TAG release_tag_commit=$v19_release_tag_commit release_tag_tree=$v19_release_tag_tree source_commit=$v19_source_commit source_tree=$v19_source_tree source_dirty=$v19_source_dirty cargo_version=\"$v19_cargo_version\" rustc_version=\"$v19_rustc_version\" binary_path=$V19_NAUTILUS_BIN binary_sha256=$v19_binary_sha256 binary_bytes=$v19_binary_bytes release_manifest=$V19_RELEASE_MANIFEST_PATH release_manifest_sha256=$v19_release_manifest_sha256 golden_trace_manifest=$V19_GOLDEN_TRACE_MANIFEST_PATH golden_trace_manifest_sha256=$v19_golden_trace_manifest_sha256 v19_gate_output_root=$V19_GATE_ROOT manifest=$V19_MANIFEST_PATH"
  exit 0
fi

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
