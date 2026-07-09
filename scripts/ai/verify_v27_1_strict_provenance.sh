#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

PRODUCT_VERSION="${NTPRO_V271_PRODUCT_VERSION:-v0.27.1}"
RELEASE_TAG="${NTPRO_V271_RELEASE_TAG:-ntpro-rust-only-v0.27.1}"
EXPECTED_RELEASE_TAG="ntpro-rust-only-v0.27.1"
MANIFEST_PATH="${NTPRO_V271_STRICT_MANIFEST:-$ROOT_DIR/target/ntpro-v271/v0_27_1_strict_release_manifest.json}"
RELEASE_MANIFEST_PATH="${NTPRO_V271_RELEASE_MANIFEST:-$ROOT_DIR/docs/rust-cutover/release/v0_27_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V271_RELEASE_NOTES:-$ROOT_DIR/docs/rust-cutover/release/v0_27_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V271_READINESS:-$ROOT_DIR/docs/rust-cutover/release/v0_27_1_readiness_report.md}"

fail() {
  echo "v27.1 strict release provenance drift: $*" >&2
  exit 1
}

[[ "$RELEASE_TAG" == "$EXPECTED_RELEASE_TAG" ]] || fail "v27.1 strict release tag must be $EXPECTED_RELEASE_TAG, got: $RELEASE_TAG"

input_paths=(
  "$RELEASE_MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$ROOT_DIR/docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_27_0_release_manifest.json"
  "$ROOT_DIR/docs/rust-cutover/release/v0_27_0_readiness_report.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_27_0_release_notes.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_27_0_release_closeout_evidence.md"
  "$ROOT_DIR/docs/rust-cutover/release/v0_27_0_publication_entry_provenance.md"
  "$ROOT_DIR/README.md"
  "$ROOT_DIR/ROADMAP.md"
  "$ROOT_DIR/docs/versioning.md"
  "$ROOT_DIR/docs/rust-cutover/release/README.md"
  "$ROOT_DIR/scripts/ai/check_release_surface_current.sh"
  "$ROOT_DIR/scripts/ai/check_github_release_published.sh"
  "$ROOT_DIR/scripts/ai/verify_release_publish_after_gate.sh"
  "$ROOT_DIR/scripts/ai/publish_ntpro_release_after_gate.sh"
  "$ROOT_DIR/scripts/ai/verify_release.sh"
  "$ROOT_DIR/scripts/ai/verify_v27_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v27_strict_provenance.sh"
  "$ROOT_DIR/scripts/ai/verify_v27_1_release_gates.sh"
  "$ROOT_DIR/scripts/ai/verify_v27_1_strict_provenance.sh"
  "$ROOT_DIR/.github/workflows/release-tag.yml"
  "$ROOT_DIR/.github/workflows/release-publish.yml"
)

for task_id in V271-001 V271-002 V271-003 V271-004 V271-005 V271-006; do
  input_paths+=("$ROOT_DIR/docs/rust-cutover/evidence/${task_id}.md")
  input_paths+=("$ROOT_DIR/docs/rust-cutover/tasks/${task_id}.md")
done

for path in "${input_paths[@]}"; do
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
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
mkdir -p "$(dirname "$MANIFEST_PATH")"

PRODUCT_VERSION="$PRODUCT_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_MANIFEST_PATH="$RELEASE_MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
MANIFEST_PATH="$MANIFEST_PATH" \
SOURCE_COMMIT="$source_commit" \
SOURCE_TREE="$source_tree" \
SOURCE_DIRTY="$source_dirty" \
RELEASE_TAG_EXISTS="$release_tag_exists" \
RELEASE_TAG_COMMIT="$release_tag_commit" \
RELEASE_TAG_TREE="$release_tag_tree" \
CARGO_VERSION="$cargo_version" \
RUSTC_VERSION="$rustc_version" \
GENERATED_AT="$generated_at" \
ROOT_DIR="$ROOT_DIR" \
INPUT_PATHS="$(printf '%s\n' "${input_paths[@]}")" \
python3 <<'PY'
import hashlib
import json
import os
from pathlib import Path

root = Path(os.environ["ROOT_DIR"])
release_manifest = json.loads(Path(os.environ["RELEASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")
v271_006_evidence = (root / "docs/rust-cutover/evidence/V271-006.md").read_text(encoding="utf-8")
closeout_evidence = (root / "docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md").read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(release_manifest["schema_version"] == "ntpro.v271_patch_release_manifest.v1", "manifest schema mismatch")
require(release_manifest["task_id"] == "V271-006", "manifest task mismatch")
require(release_manifest["product_version"] == os.environ["PRODUCT_VERSION"], "manifest product version mismatch")
require(release_manifest.get("release_status") == "released", "manifest release status must be released")
require(release_manifest["planned_release"]["tag"] == os.environ["RELEASE_TAG"], "planned tag mismatch")
require("v27.1 release gates = required" in release_notes, "release notes v27.1 gate marker missing")
require("v27.1 strict provenance = required" in readiness, "readiness strict provenance marker missing")
release_inputs = release_manifest.get("release_inputs") or {}
require(
    release_inputs.get("release_closeout_evidence_path")
    == "docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md",
    "release closeout evidence path missing",
)
published = release_manifest.get("published_release") or {}
require(published.get("tag") == os.environ["RELEASE_TAG"], "published release tag mismatch")
require(published.get("tag_sha") == "0fdc11dc983bbfb9fe124a3f171a58fb1e7ccf19", "published release tag SHA mismatch")
require(published.get("release_body_matches_tracked_release_notes") is True, "published release body/source match missing")
scope = release_manifest.get("release_scope") or {}
require(scope.get("exact_milestone_issue_numbers") == [887, 888, 889, 890, 891, 892], "exact milestone issue numbers mismatch")
require(scope.get("exact_milestone_issue_set") == "#887-#892", "exact milestone issue set mismatch")
require(scope.get("final_release_scope_issue_count") == 6, "final release scope issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 6, "final release scope evidence count mismatch")
require(scope.get("registered_corrective_scope_exception_count") == 0, "registered corrective exception count mismatch")
require(scope.get("unregistered_corrective_milestone_issues_fail_closed") is True, "unregistered corrective fail-closed rule missing")
require(scope.get("v27_0_dependency_proven") is True, "v27.0 dependency proof missing")
require(scope.get("v27_0_release_evidence_published") is True, "v27.0 release proof missing")
requirements = release_manifest.get("post_publication_requirements") or {}
require(requirements.get("all_v271_issues_closed_required") is True, "V271 closeout requirement missing")
require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
require(requirements.get("strict_release_body_match_required") is True, "strict body match requirement missing")
require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
require(requirements.get("v0_28_start_gate_fails_without_v271_release_evidence") is True, "v28 hard-block requirement missing")
closeout = release_manifest.get("post_publication_closeout") or {}
require(closeout.get("task_id") == "V281-003", "closeout task mismatch")
require(closeout.get("issue") == 921, "closeout issue mismatch")
require(closeout.get("source_controlled_closeout_evidence") is True, "source-controlled closeout proof missing")
require(
    closeout.get("source_controlled_closeout_evidence_path")
    == "docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md",
    "source-controlled closeout path mismatch",
)
gate = closeout.get("hosted_release_gate") or {}
require(gate.get("run_id") == 28940442369, "hosted gate run mismatch")
require(gate.get("conclusion") == "success", "hosted gate conclusion mismatch")
require(gate.get("jobs_success") == 82 and gate.get("jobs_failed") == 0, "hosted gate job count mismatch")
for marker in (
    "release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.1",
    "hosted release gate jobs = 82/82 success",
    "release body matches tracked release notes = true",
    "source-controlled closeout evidence = docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md",
):
    require(marker in closeout_evidence, f"closeout evidence marker missing: {marker}")
for marker in (
    "Status: PUBLISHED CLOSEOUT RECORDED",
    "v27_1_release_gates status=ok release_tag=ntpro-rust-only-v0.27.1 base_release=ntpro-rust-only-v0.27.0 current_issue_state=CLOSED v271_issues=6/6_closed",
    "v27_1_strict_provenance status=ok release_tag=ntpro-rust-only-v0.27.1 tag_exists=true source_dirty=false",
    "hosted release gate jobs = 82/82 success",
):
    require(marker in v271_006_evidence, f"V271-006 published closeout marker missing: {marker}")
for marker in (
    "Status: PASS",
    "current_issue_state=OPEN",
    "tag_exists=false",
    "source_dirty=true",
    "5/6_closed_or_current",
):
    require(marker not in v271_006_evidence, f"stale V271-006 closeout marker must not be present: {marker}")
for key in [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "automatic_operation_action_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]:
    require(release_manifest["boundary_flags"].get(key) is False, f"boundary must remain false: {key}")

source_inputs = []
for raw_path in os.environ["INPUT_PATHS"].splitlines():
    path = Path(raw_path)
    payload = path.read_bytes()
    rel = path.relative_to(root) if path.is_absolute() else path
    source_inputs.append({
        "path": str(rel),
        "sha256": "sha256:" + hashlib.sha256(payload).hexdigest(),
        "bytes": len(payload),
    })

payload = {
    "schema_version": "ntpro.v271_strict_release_provenance.v1",
    "task_id": "V271-006",
    "target": "v27.1",
    "product_version": os.environ["PRODUCT_VERSION"],
    "release_tag": os.environ["RELEASE_TAG"],
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
    "release_body_source": str(Path(os.environ["RELEASE_NOTES_PATH"]).relative_to(root)),
    "release_body_sha256": "sha256:" + hashlib.sha256(release_notes.encode("utf-8")).hexdigest(),
    "readiness_report_source": str(Path(os.environ["READINESS_REPORT_PATH"]).relative_to(root)),
    "readiness_report_sha256": "sha256:" + hashlib.sha256(readiness.encode("utf-8")).hexdigest(),
    "source_inputs": source_inputs,
    "v271_issue_scope": [887, 888, 889, 890, 891, 892],
    "v271_evidence": release_manifest.get("v271_evidence"),
    "release_scope": release_manifest.get("release_scope"),
    "capability": release_manifest.get("capability"),
    "boundary_flags": release_manifest.get("boundary_flags"),
    "publication_governance": release_manifest.get("publication_governance"),
    "release_surface_current_guard": release_manifest.get("release_surface_current_guard"),
    "next_tracks": release_manifest.get("next_tracks"),
    "post_publication_requirements": release_manifest.get("post_publication_requirements"),
    "base_release": release_manifest.get("base_release"),
    "failure_paths": {
        "dirty_worktree": "NTPRO_RELEASE_GATE=1 fails if tracked files are dirty",
        "missing_tag": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails without the v0.27.1 release tag",
        "tag_mismatch": "NTPRO_RELEASE_GATE=1 or NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 fails when HEAD differs from the release tag",
        "missing_v271_evidence": "v0.27.1 release gate fails if any V271 evidence path is missing",
        "open_operation_boundary": "v0.27.1 release gate fails if submit, mutation, adapter send, live exchange, retry scheduler, Dashboard/Admin trading, remediation, or order-ticket boundary opens",
        "pre_gate_publication": "public GitHub Release publication must use the gate-before-publish entrypoint after hosted gate success",
        "open_v271_issue_or_milestone": "NTPRO_RELEASE_GATE=1 fails unless V271 issues are closed and the v0.27.1 milestone is closed",
        "v28_unblocked_without_publication": "v0.28.0 intake remains blocked until v0.27.1 publication evidence is reconstructable",
    },
    "generated_at": os.environ["GENERATED_AT"],
}
Path(os.environ["MANIFEST_PATH"]).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if [[ "${NTPRO_V271_STRICT_REQUIRE_PUBLICATION:-0}" == "1" ]]; then
  NTPRO_CURRENT_RELEASE_VERSION="$PRODUCT_VERSION" \
    NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
    NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only $PRODUCT_VERSION" \
    NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 \
    scripts/ai/check_github_release_published.sh
fi

echo "v27_1_strict_provenance status=ok release_tag=$RELEASE_TAG tag_exists=$release_tag_exists source_dirty=$source_dirty manifest=${MANIFEST_PATH#$ROOT_DIR/}"
