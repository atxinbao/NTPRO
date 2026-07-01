#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_RELEASE_STRICT_PRODUCT_VERSION:-v0.21.0}"
RELEASE_TAG="${NTPRO_RELEASE_STRICT_RELEASE_TAG:-ntpro-rust-only-v0.21.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.21.0"
MANIFEST_PATH="${NTPRO_RELEASE_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v210/v0_21_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V21_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_21_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V21_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_21_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V21_READINESS_REPORT:-$ROOT_DIR/docs/rust-cutover/release/v0_21_0_readiness_report.md}"
GOLDEN_TRACE_MANIFEST_PATH="${NTPRO_V21_GOLDEN_TRACE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
GATE_ROOT="${NTPRO_V21_RELEASE_GATE_ROOT:-$ROOT_DIR/target/ntpro-v210/v21-release-gates}"
NAUTILUS_BIN="${NTPRO_RELEASE_STRICT_NAUTILUS_BIN:-$ROOT_DIR/target/release/nautilus}"
VERIFY_ONLY="${NTPRO_RELEASE_STRICT_VERIFY_ONLY:-0}"

fail() {
  echo "v21 strict release provenance drift: $*" >&2
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

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v21 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"
[[ -f "$RELEASE_MANIFEST_PATH" ]] || fail "missing v0.21.0 release manifest: $RELEASE_MANIFEST_PATH"
[[ -f "$RELEASE_NOTES_PATH" ]] || fail "missing v0.21.0 release notes: $RELEASE_NOTES_PATH"
[[ -f "$READINESS_REPORT_PATH" ]] || fail "missing v0.21.0 readiness report: $READINESS_REPORT_PATH"
[[ -f "$GOLDEN_TRACE_MANIFEST_PATH" ]] || fail "missing v0.21 golden trace manifest: $GOLDEN_TRACE_MANIFEST_PATH"

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
SOURCE_DIRTY="$source_dirty" \
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
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
golden_trace_manifest = json.loads(Path(os.environ["GOLDEN_TRACE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def require_text(text: str, needle: str, label: str) -> None:
    require(needle in text, f"{label} missing required marker: {needle}")

def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"release manifest boundary flag must be false: {key}")

for needle in (
    "Status: RELEASED",
    "Tag: `ntpro-rust-only-v0.21.0`",
    "Release name: `NTPRO Rust-only v0.21.0`",
    "Unified Read Model Foundation",
    "scripts/ai/verify_release_strict.sh v21",
):
    require_text(release_notes, needle, "release notes")

for needle in (
    "Milestone: `ntpro-rust-only-v0.21.0`",
    "Status: RELEASED",
    "v21 strict provenance = required",
    "release scope manifest cases = 83",
):
    require_text(readiness, needle, "readiness report")

require(release_manifest.get("schema_version") == "ntpro.v210_release_manifest.v1", "release manifest schema mismatch")
require(release_manifest.get("task_id") == "V210-008", "release manifest task mismatch")
require(release_manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "release manifest product version mismatch")
require(
    release_manifest.get("release_status") in {"published_in_source_tree", "published_closeout_complete"},
    "release manifest status mismatch",
)

planned = release_manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == "NTPRO Rust-only v0.21.0", "planned release name mismatch")
require(planned.get("github_release_url") == "https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.21.0", "planned release URL mismatch")
require(planned.get("draft") is False, "planned draft flag mismatch")
require(planned.get("prerelease") is False, "planned prerelease flag mismatch")
require(planned.get("target_commitish") == "main", "planned target mismatch")

release_gate_commands = {
    gate.get("command")
    for gate in release_manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v21-release-gates",
    "scripts/ai/verify_release.sh v21-strict-provenance",
    "scripts/ai/verify_release_strict.sh v21",
):
    require(command in release_gate_commands, f"release manifest gate missing: {command}")

capability = release_manifest.get("capability") or {}
require(capability.get("capability_expansion") == "unified_read_model_foundation", "capability expansion mismatch")
require(capability.get("trader_terminal_scope") == "read_only_foundation", "terminal scope mismatch")

boundary = release_manifest.get("boundary_flags") or {}
for key in (
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "implicit_retry_allowed",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "retry_replace_amend_flatten_allowed",
    "strategy_driven_production_execution_allowed",
    "multi_account_execution_allowed",
    "multi_venue_execution_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "dashboard_submit_controls_enabled",
    "dashboard_replace_controls_enabled",
    "dashboard_amend_controls_enabled",
    "dashboard_flatten_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "product_grade_trading_terminal_claim",
):
    require_false(boundary, key)

cases = golden_trace_manifest.get("cases") or []
require(len(cases) == 83, "golden trace manifest case count mismatch")
read_model_cases = [case for case in cases if case.get("category") == "read_model"]
require(len(read_model_cases) == 32, "read model case count mismatch")
read_model_executable_cases = [
    case for case in read_model_cases
    if case.get("status") == "executable_replay"
]
read_model_schema_only_cases = [
    case for case in read_model_cases
    if case.get("status") == "schema_only_scoped"
]
expected_executable = {
    "read_model.account_snapshot.fresh.001",
    "read_model.account_snapshot.stale.001",
    "read_model.order_lifecycle.matched.001",
    "read_model.order_lifecycle.missing_ledger.001",
    "read_model.risk_state.healthy.001",
    "read_model.risk_state.mismatch.001",
    "read_model.dashboard.readonly_complete.001",
    "read_model.dashboard.missing_evidence_degraded.001",
}
actual_executable = {case.get("case_id") for case in read_model_executable_cases}
require(actual_executable == expected_executable, "read model executable replay case set mismatch")
require(len(read_model_schema_only_cases) == 24, "read model schema-only case count mismatch")
PY

if [[ "$VERIFY_ONLY" != "1" && "${NTPRO_RELEASE_STRICT_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build -p nautilus-cli --release --bin nautilus
fi

if [[ "$VERIFY_ONLY" != "1" && "${NTPRO_RELEASE_STRICT_SKIP_V21_GATES:-0}" != "1" ]]; then
  NTPRO_V21_RELEASE_GATE_ROOT="$GATE_ROOT" scripts/ai/verify_v21_release_gates.sh
fi

binary_sha256=""
binary_bytes="0"
binary_version=""
if [[ -x "$NAUTILUS_BIN" ]]; then
  if [[ "$NAUTILUS_BIN" != */target/release/nautilus && "${NTPRO_RELEASE_STRICT_ALLOW_NON_RELEASE_BIN:-0}" != "1" ]]; then
    fail "strict gate requires target/release/nautilus, got: $NAUTILUS_BIN"
  fi
  binary_version="$("$NAUTILUS_BIN" --version)"
  binary_sha256="sha256:$(sha256_file "$NAUTILUS_BIN")"
  binary_bytes="$(wc -c < "$NAUTILUS_BIN" | tr -d ' ')"
elif [[ "$VERIFY_ONLY" != "1" ]]; then
  fail "missing release binary: $NAUTILUS_BIN"
fi
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
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
manifest = {
    "schema_version": "ntpro.v210_strict_release_provenance_manifest.v1",
    "task_id": "V210-008",
    "target": "v21",
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
    "validation_artifacts": {
        "v21_gate_output_root": os.environ["GATE_ROOT_REL"],
    },
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "release_gates": release_manifest.get("release_gates"),
    "generated_at": os.environ["GENERATED_AT"],
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
elif [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "verify-only mode: manifest not generated"
fi

echo "v21_strict_release_provenance status=ok product_version=$PRODUCT_VERSION release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_commit=$source_commit source_tree=$source_tree source_dirty=$source_dirty release_manifest=$RELEASE_MANIFEST_PATH release_manifest_sha256=$release_manifest_sha256 manifest=$MANIFEST_PATH"
