#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V221_PRODUCT_VERSION:-v0.22.1}"
RELEASE_TAG="${NTPRO_V221_RELEASE_TAG:-ntpro-rust-only-v0.22.1}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.22.1"
MANIFEST_PATH="${NTPRO_V221_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v221/v0_22_1_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V221_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_22_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V221_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_22_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V221_READINESS_REPORT:-$ROOT_DIR/docs/rust-cutover/release/v0_22_1_readiness_report.md}"
BASE_MANIFEST_PATH="${NTPRO_V221_BASE_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_22_0_release_manifest.json}"
CLOSEOUT_EVIDENCE_PATH="${NTPRO_V221_CLOSEOUT_EVIDENCE:-$ROOT_DIR/docs/rust-cutover/release/v0_22_1_release_closeout_evidence.md}"
REQUIRED_FALSE_BOUNDARY_PATH="${NTPRO_V221_REQUIRED_FALSE_BOUNDARY:-$ROOT_DIR/docs/rust-cutover/release/v0_22_1_required_false_runtime_boundary.md}"
READ_MODEL_REPLAY_PATH="${NTPRO_V221_READ_MODEL_REPLAY:-$ROOT_DIR/docs/rust-cutover/release/v0_22_1_read_model_executable_replay.md}"
GATE_BEFORE_PUBLISH_PATH="${NTPRO_V221_GATE_BEFORE_PUBLISH:-$ROOT_DIR/docs/rust-cutover/release/v0_22_1_gate_before_publish.md}"
GOLDEN_TRACE_MANIFEST_PATH="${NTPRO_V221_GOLDEN_TRACE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
WORKBENCH_RENDER_FIXTURE="${NTPRO_V221_WORKBENCH_RENDER_FIXTURE:-$ROOT_DIR/tests/golden/v221/workbench_render_snapshot.json}"
VERIFY_ONLY="${NTPRO_V221_STRICT_VERIFY_ONLY:-0}"

fail() {
  echo "v22.1 strict release provenance drift: $*" >&2
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

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v22.1 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"
for path in \
  "$RELEASE_MANIFEST_PATH" \
  "$RELEASE_NOTES_PATH" \
  "$READINESS_REPORT_PATH" \
  "$BASE_MANIFEST_PATH" \
  "$CLOSEOUT_EVIDENCE_PATH" \
  "$REQUIRED_FALSE_BOUNDARY_PATH" \
  "$READ_MODEL_REPLAY_PATH" \
  "$GATE_BEFORE_PUBLISH_PATH" \
  "$GOLDEN_TRACE_MANIFEST_PATH" \
  "$WORKBENCH_RENDER_FIXTURE"; do
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
release_manifest_sha256="sha256:$(sha256_file "$RELEASE_MANIFEST_PATH")"
release_notes_sha256="sha256:$(sha256_file "$RELEASE_NOTES_PATH")"
readiness_report_sha256="sha256:$(sha256_file "$READINESS_REPORT_PATH")"
base_manifest_sha256="sha256:$(sha256_file "$BASE_MANIFEST_PATH")"
closeout_evidence_sha256="sha256:$(sha256_file "$CLOSEOUT_EVIDENCE_PATH")"
required_false_boundary_sha256="sha256:$(sha256_file "$REQUIRED_FALSE_BOUNDARY_PATH")"
read_model_replay_sha256="sha256:$(sha256_file "$READ_MODEL_REPLAY_PATH")"
gate_before_publish_sha256="sha256:$(sha256_file "$GATE_BEFORE_PUBLISH_PATH")"
golden_trace_manifest_sha256="sha256:$(sha256_file "$GOLDEN_TRACE_MANIFEST_PATH")"
workbench_render_fixture_sha256="sha256:$(sha256_file "$WORKBENCH_RENDER_FIXTURE")"
release_manifest_rel="${RELEASE_MANIFEST_PATH#$ROOT_DIR/}"
release_notes_rel="${RELEASE_NOTES_PATH#$ROOT_DIR/}"
readiness_report_rel="${READINESS_REPORT_PATH#$ROOT_DIR/}"
base_manifest_rel="${BASE_MANIFEST_PATH#$ROOT_DIR/}"
closeout_evidence_rel="${CLOSEOUT_EVIDENCE_PATH#$ROOT_DIR/}"
required_false_boundary_rel="${REQUIRED_FALSE_BOUNDARY_PATH#$ROOT_DIR/}"
read_model_replay_rel="${READ_MODEL_REPLAY_PATH#$ROOT_DIR/}"
gate_before_publish_rel="${GATE_BEFORE_PUBLISH_PATH#$ROOT_DIR/}"
golden_trace_manifest_rel="${GOLDEN_TRACE_MANIFEST_PATH#$ROOT_DIR/}"
workbench_render_fixture_rel="${WORKBENCH_RENDER_FIXTURE#$ROOT_DIR/}"
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


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_text(text: str, needle: str, label: str) -> None:
    require(needle in text, f"{label} missing required marker: {needle}")


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"release manifest boundary flag must be false: {key}")


for needle in (
    "Status: RELEASED",
    "Tag: `ntpro-rust-only-v0.22.1`",
    "Release name: `NTPRO Rust-only v0.22.1`",
    "Trader Terminal Workbench hardening patch",
    "scripts/ai/verify_v22_1_strict_provenance.sh",
    "scripts/ai/publish_ntpro_release_after_gate.sh",
):
    require_text(release_notes, needle, "release notes")

for needle in (
    "Milestone: `ntpro-rust-only-v0.22.1`",
    "Status: RELEASED",
    "v22.1 strict provenance = required",
    "V221-006 evidence",
    "#710 V221-006 = stays open until tag, hosted gate, public release, and publication evidence are recorded",
):
    require_text(readiness, needle, "readiness report")

require(release_manifest.get("schema_version") == "ntpro.v221_patch_release_manifest.v1", "release manifest schema mismatch")
require(release_manifest.get("task_id") == "V221-006", "release manifest task mismatch")
require(release_manifest.get("product_version") == os.environ["PRODUCT_VERSION"], "release manifest product version mismatch")
require(release_manifest.get("release_status") == "published", "release manifest status mismatch")
require(release_manifest.get("patch_scope") == "trader_terminal_workbench_hardening_patch", "release manifest patch scope mismatch")
require(base_manifest.get("release_status") == "published", "base manifest status mismatch")
require(golden_trace_manifest.get("schema_version") == "golden-trace-release-scope-v1", "golden trace manifest schema mismatch")

planned = release_manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == "NTPRO Rust-only v0.22.1", "planned release name mismatch")
require(planned.get("github_release_url") == "https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1", "planned release URL mismatch")
require(planned.get("draft") is False, "planned draft flag mismatch")
require(planned.get("prerelease") is False, "planned prerelease flag mismatch")
require(planned.get("target_commitish") == "main", "planned target mismatch")

release_gate_commands = {
    gate.get("command")
    for gate in release_manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v22.1-release-gates",
    "scripts/ai/verify_release.sh v22.1-strict-provenance",
    "scripts/ai/verify_v22_1_strict_provenance.sh",
    "scripts/ai/verify_release.sh release-publish-after-gate",
):
    require(command in release_gate_commands, f"release manifest gate missing: {command}")

capability = release_manifest.get("capability") or {}
require(capability.get("capability_expansion") == "none_patch_hardening_only", "capability expansion mismatch")
require(capability.get("trader_terminal_workbench") is True, "trader terminal workbench flag mismatch")
require(capability.get("read_only_first") is True, "read-only-first flag mismatch")
require(capability.get("required_false_runtime_boundary") is True, "required-false boundary mismatch")
require(capability.get("gate_before_publish") is True, "gate-before-publish flag mismatch")
require(capability.get("strict_provenance") is True, "strict provenance flag mismatch")
require(capability.get("complete_executable_read_model_runtime") is False, "complete executable read-model runtime claim mismatch")
require(capability.get("product_grade_live_trading_terminal") is False, "product-grade terminal claim mismatch")

for key in (
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "manual_operation_entry_enabled",
    "manual_operation_submit_allowed",
    "manual_operation_cancel_allowed",
    "manual_operation_retry_allowed",
    "manual_operation_replace_allowed",
    "manual_operation_amend_allowed",
    "manual_operation_flatten_allowed",
    "automatic_operation_action_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "dashboard_fill_controls_enabled",
    "dashboard_risk_controls_enabled",
    "dashboard_submit_controls_enabled",
    "dashboard_replace_controls_enabled",
    "dashboard_amend_controls_enabled",
    "dashboard_flatten_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "trader_terminal_live_trading_claim",
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
  CLOSEOUT_EVIDENCE_REL="$closeout_evidence_rel" \
  CLOSEOUT_EVIDENCE_SHA256="$closeout_evidence_sha256" \
  REQUIRED_FALSE_BOUNDARY_REL="$required_false_boundary_rel" \
  REQUIRED_FALSE_BOUNDARY_SHA256="$required_false_boundary_sha256" \
  READ_MODEL_REPLAY_REL="$read_model_replay_rel" \
  READ_MODEL_REPLAY_SHA256="$read_model_replay_sha256" \
  GATE_BEFORE_PUBLISH_REL="$gate_before_publish_rel" \
  GATE_BEFORE_PUBLISH_SHA256="$gate_before_publish_sha256" \
  GOLDEN_TRACE_MANIFEST_REL="$golden_trace_manifest_rel" \
  GOLDEN_TRACE_MANIFEST_SHA256="$golden_trace_manifest_sha256" \
  WORKBENCH_RENDER_FIXTURE_REL="$workbench_render_fixture_rel" \
  WORKBENCH_RENDER_FIXTURE_SHA256="$workbench_render_fixture_sha256" \
  python3 <<'PY'
import json
import os
from pathlib import Path

release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_REL"]).read_text(encoding="utf-8"))
manifest = {
    "schema_version": "ntpro.v221_strict_release_provenance_manifest.v1",
    "task_id": "V221-006",
    "target": "v22.1",
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
        "release_closeout_evidence": {
            "path": os.environ["CLOSEOUT_EVIDENCE_REL"],
            "sha256": os.environ["CLOSEOUT_EVIDENCE_SHA256"],
        },
        "required_false_runtime_boundary": {
            "path": os.environ["REQUIRED_FALSE_BOUNDARY_REL"],
            "sha256": os.environ["REQUIRED_FALSE_BOUNDARY_SHA256"],
        },
        "read_model_executable_replay": {
            "path": os.environ["READ_MODEL_REPLAY_REL"],
            "sha256": os.environ["READ_MODEL_REPLAY_SHA256"],
        },
        "gate_before_publish": {
            "path": os.environ["GATE_BEFORE_PUBLISH_REL"],
            "sha256": os.environ["GATE_BEFORE_PUBLISH_SHA256"],
        },
        "golden_trace_manifest": {
            "path": os.environ["GOLDEN_TRACE_MANIFEST_REL"],
            "sha256": os.environ["GOLDEN_TRACE_MANIFEST_SHA256"],
        },
        "workbench_render_fixture": {
            "path": os.environ["WORKBENCH_RENDER_FIXTURE_REL"],
            "sha256": os.environ["WORKBENCH_RENDER_FIXTURE_SHA256"],
        },
    },
    "v221_evidence": release_manifest.get("v221_evidence"),
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "read_model_replay": release_manifest.get("read_model_replay"),
    "publication_governance": release_manifest.get("publication_governance"),
    "v230_dependency": release_manifest.get("v230_dependency"),
    "release_gates": release_manifest.get("release_gates"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 fails if tracked files are dirty",
        "missing_tag": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails without the release tag",
        "tag_mismatch": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails when HEAD differs from the release tag",
        "manifest_mismatch": "release manifest, notes, readiness, patch evidence, golden trace, and render fixture hashes must match",
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
BASE_MANIFEST_REL="$base_manifest_rel" \
BASE_MANIFEST_SHA256="$base_manifest_sha256" \
CLOSEOUT_EVIDENCE_REL="$closeout_evidence_rel" \
CLOSEOUT_EVIDENCE_SHA256="$closeout_evidence_sha256" \
REQUIRED_FALSE_BOUNDARY_REL="$required_false_boundary_rel" \
REQUIRED_FALSE_BOUNDARY_SHA256="$required_false_boundary_sha256" \
READ_MODEL_REPLAY_REL="$read_model_replay_rel" \
READ_MODEL_REPLAY_SHA256="$read_model_replay_sha256" \
GATE_BEFORE_PUBLISH_REL="$gate_before_publish_rel" \
GATE_BEFORE_PUBLISH_SHA256="$gate_before_publish_sha256" \
GOLDEN_TRACE_MANIFEST_REL="$golden_trace_manifest_rel" \
GOLDEN_TRACE_MANIFEST_SHA256="$golden_trace_manifest_sha256" \
WORKBENCH_RENDER_FIXTURE_REL="$workbench_render_fixture_rel" \
WORKBENCH_RENDER_FIXTURE_SHA256="$workbench_render_fixture_sha256" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(manifest.get("schema_version") == "ntpro.v221_strict_release_provenance_manifest.v1", "strict manifest schema mismatch")
require(manifest.get("task_id") == "V221-006", "strict manifest task mismatch")
require(manifest.get("target") == "v22.1", "strict manifest target mismatch")
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
    "release_closeout_evidence": (os.environ["CLOSEOUT_EVIDENCE_REL"], os.environ["CLOSEOUT_EVIDENCE_SHA256"]),
    "required_false_runtime_boundary": (os.environ["REQUIRED_FALSE_BOUNDARY_REL"], os.environ["REQUIRED_FALSE_BOUNDARY_SHA256"]),
    "read_model_executable_replay": (os.environ["READ_MODEL_REPLAY_REL"], os.environ["READ_MODEL_REPLAY_SHA256"]),
    "gate_before_publish": (os.environ["GATE_BEFORE_PUBLISH_REL"], os.environ["GATE_BEFORE_PUBLISH_SHA256"]),
    "golden_trace_manifest": (os.environ["GOLDEN_TRACE_MANIFEST_REL"], os.environ["GOLDEN_TRACE_MANIFEST_SHA256"]),
    "workbench_render_fixture": (os.environ["WORKBENCH_RENDER_FIXTURE_REL"], os.environ["WORKBENCH_RENDER_FIXTURE_SHA256"]),
}
for name, (path, sha256) in expected.items():
    item = inputs.get(name) or {}
    require(item.get("path") == path, f"strict manifest input path mismatch: {name}")
    require(item.get("sha256") == sha256, f"strict manifest input sha mismatch: {name}")

failure_paths = manifest.get("failure_paths") or {}
for key in ("dirty_worktree", "missing_tag", "tag_mismatch", "manifest_mismatch", "pre_gate_publication"):
    require(key in failure_paths, f"strict manifest failure path missing: {key}")
PY

echo "v22_1_strict_provenance status=ok product_version=$PRODUCT_VERSION release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_commit=$source_commit source_tree=$source_tree source_dirty=$source_dirty cargo_version=\"$cargo_version\" rustc_version=\"$rustc_version\" release_manifest=$RELEASE_MANIFEST_PATH release_manifest_sha256=$release_manifest_sha256 manifest=$MANIFEST_PATH"
