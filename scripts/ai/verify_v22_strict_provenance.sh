#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V220_PRODUCT_VERSION:-v0.22.0}"
RELEASE_TAG="${NTPRO_V220_RELEASE_TAG:-ntpro-rust-only-v0.22.0}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.22.0"
MANIFEST_PATH="${NTPRO_V220_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v220/v0_22_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V220_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_22_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V220_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_22_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V220_READINESS_REPORT:-$ROOT_DIR/docs/rust-cutover/release/v0_22_0_readiness_report.md}"
BASE_MANIFEST_PATH="${NTPRO_V220_BASE_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_21_1_release_manifest.json}"
SCOPE_PATH="${NTPRO_V220_SCOPE:-$ROOT_DIR/docs/rust-cutover/scope/v0_22_0_trader_terminal_workbench_scope.md}"
GOLDEN_TRACE_MANIFEST_PATH="${NTPRO_V220_GOLDEN_TRACE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
VERIFY_ONLY="${NTPRO_V220_STRICT_VERIFY_ONLY:-0}"

fail() {
  echo "v22 strict release provenance drift: $*" >&2
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

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v22 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"
[[ -f "$RELEASE_MANIFEST_PATH" ]] || fail "missing v0.22.0 release manifest: $RELEASE_MANIFEST_PATH"
[[ -f "$RELEASE_NOTES_PATH" ]] || fail "missing v0.22.0 release notes: $RELEASE_NOTES_PATH"
[[ -f "$READINESS_REPORT_PATH" ]] || fail "missing v0.22.0 readiness report: $READINESS_REPORT_PATH"
[[ -f "$BASE_MANIFEST_PATH" ]] || fail "missing v0.21.1 release manifest: $BASE_MANIFEST_PATH"
[[ -f "$SCOPE_PATH" ]] || fail "missing v0.22 scope decision: $SCOPE_PATH"
[[ -f "$GOLDEN_TRACE_MANIFEST_PATH" ]] || fail "missing golden trace manifest: $GOLDEN_TRACE_MANIFEST_PATH"

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
require_head_tag="${NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG:-0}"
if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  gate_ref_name="${GITHUB_REF_NAME:-}"
  if [[ -z "$gate_ref_name" || "$gate_ref_name" == "$RELEASE_TAG" ]]; then
    require_head_tag="1"
  else
    echo "v22_strict_provenance historical_stage_head_tag_check=skipped gate_ref=$gate_ref_name expected_release_tag=$RELEASE_TAG"
  fi
fi
if [[ "$require_head_tag" == "1" ]]; then
  [[ "$release_tag_exists" == "true" ]] || fail "missing required local release tag: $RELEASE_TAG"
  [[ "$source_commit" == "$release_tag_commit" ]] || fail "HEAD $source_commit does not match $RELEASE_TAG commit $release_tag_commit"
fi

cargo_version="$(cargo --version)"
rustc_version="$(rustc --version)"
release_manifest_sha256="sha256:$(sha256_file "$RELEASE_MANIFEST_PATH")"
release_notes_sha256="sha256:$(sha256_file "$RELEASE_NOTES_PATH")"
readiness_report_sha256="sha256:$(sha256_file "$READINESS_REPORT_PATH")"
base_manifest_sha256="sha256:$(sha256_file "$BASE_MANIFEST_PATH")"
scope_sha256="sha256:$(sha256_file "$SCOPE_PATH")"
golden_trace_manifest_sha256="sha256:$(sha256_file "$GOLDEN_TRACE_MANIFEST_PATH")"
release_manifest_rel="${RELEASE_MANIFEST_PATH#$ROOT_DIR/}"
release_notes_rel="${RELEASE_NOTES_PATH#$ROOT_DIR/}"
readiness_report_rel="${READINESS_REPORT_PATH#$ROOT_DIR/}"
base_manifest_rel="${BASE_MANIFEST_PATH#$ROOT_DIR/}"
scope_rel="${SCOPE_PATH#$ROOT_DIR/}"
golden_trace_manifest_rel="${GOLDEN_TRACE_MANIFEST_PATH#$ROOT_DIR/}"
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
BASE_MANIFEST_PATH="$BASE_MANIFEST_PATH" \
SCOPE_PATH="$SCOPE_PATH" \
GOLDEN_TRACE_MANIFEST_PATH="$GOLDEN_TRACE_MANIFEST_PATH" \
STRICT_MANIFEST_REL="$strict_manifest_rel" \
CARGO_VERSION="$cargo_version" \
RUSTC_VERSION="$rustc_version" \
python3 <<'PY'
import json
import os
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
base_manifest = json.loads(Path(os.environ["BASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
golden_trace_manifest = json.loads(Path(os.environ["GOLDEN_TRACE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")
scope = Path(os.environ["SCOPE_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_text(text: str, needle: str, label: str) -> None:
    require(needle in text, f"{label} missing required marker: {needle}")


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"release manifest boundary flag must be false: {key}")


for needle in (
    "Status: RELEASED",
    "Tag: `ntpro-rust-only-v0.22.0`",
    "Release name: `NTPRO Rust-only v0.22.0`",
    "Trader Terminal Workbench",
    "This release is read-only first",
    "scripts/ai/verify_v22_strict_provenance.sh",
):
    require_text(release_notes, needle, "release notes")

for needle in (
    "Milestone: `ntpro-rust-only-v0.22.0`",
    "Status: RELEASED",
    "v22 strict provenance = required",
    "V220-007 evidence",
):
    require_text(readiness, needle, "readiness report")

for needle in (
    "read_only_first = required",
    "strict_provenance = required",
    "product_grade_trading_terminal_claim = forbidden",
):
    require_text(scope, needle, "scope")

require(release_manifest.get("schema_version") == "ntpro.v220_release_manifest.v1", "release manifest schema mismatch")
require(release_manifest.get("task_id") == "V220-007", "release manifest task mismatch")
require(release_manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "release manifest product version mismatch")
require(release_manifest.get("release_status") == "published", "release manifest status mismatch")
require(release_manifest.get("release_scope") == "trader_terminal_workbench", "release manifest scope mismatch")
require(base_manifest.get("release_status") == "published", "base manifest status mismatch")
require(golden_trace_manifest.get("schema_version") == "golden-trace-release-scope-v1", "golden trace manifest schema mismatch")

planned = release_manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == "NTPRO Rust-only v0.22.0", "planned release name mismatch")
require(planned.get("github_release_url") == "https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.0", "planned release URL mismatch")
require(planned.get("draft") is False, "planned draft flag mismatch")
require(planned.get("prerelease") is False, "planned prerelease flag mismatch")
require(planned.get("target_commitish") == "main", "planned target mismatch")

release_gate_commands = {
    gate.get("command")
    for gate in release_manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v22-release-gates",
    "scripts/ai/verify_release.sh v22-strict-provenance",
    "scripts/ai/verify_v22_strict_provenance.sh",
    "scripts/ai/verify_release_strict.sh v22",
    "scripts/ai/verify_release.sh release-surface-current-guard",
    "scripts/ai/verify_release.sh release-publication-guard",
):
    require(command in release_gate_commands, f"release manifest gate missing: {command}")

capability = release_manifest.get("capability") or {}
require(capability.get("capability_expansion") == "read_only_first_trader_terminal_workbench", "capability expansion mismatch")
require(capability.get("trader_terminal_workbench") is True, "trader terminal workbench flag mismatch")
require(capability.get("read_only_first") is True, "read-only-first flag mismatch")
require(capability.get("gated_operation_boundary") is True, "gated operation boundary mismatch")
require(capability.get("strict_provenance") is True, "strict provenance flag mismatch")

for key in (
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "ungated_submit_allowed",
    "ungated_cancel_allowed",
    "ungated_retry_allowed",
    "ungated_replace_allowed",
    "ungated_amend_allowed",
    "ungated_flatten_allowed",
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
    require_false(release_manifest.get("boundary_flags") or {}, key)
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
  BASE_MANIFEST_REL="$base_manifest_rel" \
  BASE_MANIFEST_SHA256="$base_manifest_sha256" \
  SCOPE_REL="$scope_rel" \
  SCOPE_SHA256="$scope_sha256" \
  GOLDEN_TRACE_MANIFEST_REL="$golden_trace_manifest_rel" \
  GOLDEN_TRACE_MANIFEST_SHA256="$golden_trace_manifest_sha256" \
  python3 <<'PY'
import json
import os
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_REL"]).read_text(encoding="utf-8"))
manifest = {
    "schema_version": "ntpro.v220_strict_release_provenance_manifest.v1",
    "task_id": "V220-007",
    "target": "v22",
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
        "release_manifest": {
            "path": os.environ["RELEASE_MANIFEST_REL"],
            "sha256": os.environ["RELEASE_MANIFEST_SHA256"],
        },
        "release_notes": {
            "path": os.environ["RELEASE_NOTES_REL"],
            "sha256": os.environ["RELEASE_NOTES_SHA256"],
        },
        "readiness_report": {
            "path": os.environ["READINESS_REPORT_REL"],
            "sha256": os.environ["READINESS_REPORT_SHA256"],
        },
        "base_release_manifest": {
            "path": os.environ["BASE_MANIFEST_REL"],
            "sha256": os.environ["BASE_MANIFEST_SHA256"],
        },
        "scope_decision": {
            "path": os.environ["SCOPE_REL"],
            "sha256": os.environ["SCOPE_SHA256"],
        },
        "golden_trace_manifest": {
            "path": os.environ["GOLDEN_TRACE_MANIFEST_REL"],
            "sha256": os.environ["GOLDEN_TRACE_MANIFEST_SHA256"],
        },
    },
    "v220_evidence": release_manifest.get("v220_evidence"),
    "workbench_evidence": release_manifest.get("workbench_evidence"),
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "release_gates": release_manifest.get("release_gates"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 fails if tracked files are dirty",
        "missing_tag": "NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails without the release tag",
        "tag_mismatch": "NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails when HEAD differs from the release tag",
        "manifest_mismatch": "release manifest, notes, readiness, scope, and base manifest hashes must match",
    },
    "generated_at": os.environ["GENERATED_AT"],
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
elif [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "verify-only mode: manifest not generated"
fi

if [[ -f "$MANIFEST_PATH" ]]; then
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
  BASE_MANIFEST_REL="$base_manifest_rel" \
  BASE_MANIFEST_SHA256="$base_manifest_sha256" \
  SCOPE_REL="$scope_rel" \
  SCOPE_SHA256="$scope_sha256" \
  GOLDEN_TRACE_MANIFEST_REL="$golden_trace_manifest_rel" \
  GOLDEN_TRACE_MANIFEST_SHA256="$golden_trace_manifest_sha256" \
  python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(manifest.get("schema_version") == "ntpro.v220_strict_release_provenance_manifest.v1", "strict manifest schema mismatch")
require(manifest.get("task_id") == "V220-007", "strict manifest task mismatch")
require(manifest.get("target") == "v22", "strict manifest target mismatch")
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
    "base_release_manifest": (os.environ["BASE_MANIFEST_REL"], os.environ["BASE_MANIFEST_SHA256"]),
    "scope_decision": (os.environ["SCOPE_REL"], os.environ["SCOPE_SHA256"]),
    "golden_trace_manifest": (os.environ["GOLDEN_TRACE_MANIFEST_REL"], os.environ["GOLDEN_TRACE_MANIFEST_SHA256"]),
}
for name, (path, sha256) in expected.items():
    item = inputs.get(name) or {}
    require(item.get("path") == path, f"strict manifest input path mismatch: {name}")
    require(item.get("sha256") == sha256, f"strict manifest input sha mismatch: {name}")

failure_paths = manifest.get("failure_paths") or {}
for key in ("dirty_worktree", "missing_tag", "tag_mismatch", "manifest_mismatch"):
    require(key in failure_paths, f"strict manifest failure path missing: {key}")
PY
fi

echo "v22_strict_provenance status=ok product_version=$PRODUCT_VERSION release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_commit=$source_commit source_tree=$source_tree source_dirty=$source_dirty cargo_version=\"$cargo_version\" rustc_version=\"$rustc_version\" release_manifest=$RELEASE_MANIFEST_PATH release_manifest_sha256=$release_manifest_sha256 manifest=$MANIFEST_PATH"
