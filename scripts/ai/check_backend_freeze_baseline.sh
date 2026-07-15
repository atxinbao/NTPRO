#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

REGISTRY="${NTPRO_BACKEND_FREEZE_REGISTRY:-docs/rust-cutover/governance/backend_freeze_registry.json}"
RELEASE_MANIFEST="${NTPRO_BACKEND_FREEZE_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_32_0_release_manifest.json}"
POLICY="${NTPRO_BACKEND_FREEZE_POLICY:-docs/rust-cutover/governance/backend_freeze_policy.md}"
README_PATH="${NTPRO_BACKEND_FREEZE_README:-README.md}"
ROADMAP_PATH="${NTPRO_BACKEND_FREEZE_ROADMAP:-ROADMAP.md}"
VERSIONING_PATH="${NTPRO_BACKEND_FREEZE_VERSIONING:-docs/versioning.md}"
RUN_NEGATIVE_SELFTEST="${NTPRO_BACKEND_FREEZE_NEGATIVE_SELFTEST:-1}"

fail() {
  echo "backend freeze baseline drift: $*" >&2
  exit 1
}

for path in "$REGISTRY" "$RELEASE_MANIFEST" "$POLICY" "$README_PATH" "$ROADMAP_PATH" "$VERSIONING_PATH"; do
  [[ -f "$path" ]] || fail "missing required file: $path"
done

registry_identity="$(python3 - "$REGISTRY" <<'PY'
import json
import sys

try:
    registry = json.load(open(sys.argv[1], encoding="utf-8"))
    tag = registry["baseline"]["tag"]
    print("|".join([tag["name"], tag["object_sha"], tag["peeled_commit_sha"]]))
except (KeyError, TypeError, json.JSONDecodeError) as exc:
    raise SystemExit(f"invalid registry identity: {exc}")
PY
)" || fail "cannot read registry identity"

IFS='|' read -r expected_tag expected_tag_object expected_tag_commit <<< "$registry_identity"
[[ -n "$expected_tag" && -n "$expected_tag_object" && -n "$expected_tag_commit" ]] || fail "incomplete registry tag identity"

git rev-parse -q --verify "refs/tags/$expected_tag" >/dev/null || fail "missing local baseline tag: $expected_tag"
local_tag_object="$(git rev-parse "refs/tags/$expected_tag")"
local_tag_commit="$(git rev-parse "$expected_tag^{}")"

REGISTRY="$REGISTRY" \
RELEASE_MANIFEST="$RELEASE_MANIFEST" \
POLICY="$POLICY" \
README_PATH="$README_PATH" \
ROADMAP_PATH="$ROADMAP_PATH" \
VERSIONING_PATH="$VERSIONING_PATH" \
LOCAL_TAG_OBJECT="$local_tag_object" \
LOCAL_TAG_COMMIT="$local_tag_commit" \
python3 <<'PY'
import hashlib
import json
import os
from pathlib import Path

EXPECTED_TAG = "ntpro-rust-only-v0.32.0"
EXPECTED_TAG_OBJECT = "b9a66f12ede051968723ace22b3f06a8e7ac5a09"
EXPECTED_COMMIT = "2b955cb8a989827e3351c08c3d82d9578253e1f6"
EXPECTED_ISSUES = list(range(1042, 1052))
EXPECTED_BOUNDARIES = {
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "automatic_operation_action_allowed",
    "automatic_recovery_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "frontend_completion_claim",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
}
EXPECTED_INHERITANCE_FALSE = {
    "inherits_backend_go_live_claim",
    "inherits_production_submit",
    "inherits_production_mutation",
    "inherits_adapter_send",
    "inherits_live_exchange_request",
    "inherits_retry_scheduler",
    "inherits_automatic_remediation",
    "inherits_dashboard_trading_controls",
    "inherits_admin_workbench_trading_controls",
    "inherits_trader_terminal_order_ticket",
}

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def load_json(path):
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise SystemExit(f"cannot load {path}: {exc}")

registry = load_json(os.environ["REGISTRY"])
manifest = load_json(os.environ["RELEASE_MANIFEST"])
policy = Path(os.environ["POLICY"]).read_text(encoding="utf-8")
readme = Path(os.environ["README_PATH"]).read_text(encoding="utf-8")
roadmap = Path(os.environ["ROADMAP_PATH"]).read_text(encoding="utf-8")
versioning = Path(os.environ["VERSIONING_PATH"]).read_text(encoding="utf-8")

require(registry.get("schema_version") == "ntpro.backend_freeze_registry.v1", "registry schema mismatch")
require(registry.get("task_id") == "BFG-001", "registry task mismatch")
require(registry.get("status") == "active", "registry status must be active")

baseline = registry.get("baseline") or {}
tag = baseline.get("tag") or {}
require(baseline.get("version") == "v0.32.0", "baseline version mismatch")
require(baseline.get("release_status") == "released_and_frozen", "baseline release status mismatch")
require(tag.get("name") == EXPECTED_TAG, "baseline tag mismatch")
require(tag.get("object_sha") == EXPECTED_TAG_OBJECT, "baseline tag object mismatch")
require(tag.get("peeled_commit_sha") == EXPECTED_COMMIT, "baseline commit mismatch")
require(os.environ["LOCAL_TAG_OBJECT"] == EXPECTED_TAG_OBJECT, "local tag object mismatch")
require(os.environ["LOCAL_TAG_COMMIT"] == EXPECTED_COMMIT, "local peeled tag commit mismatch")

release = registry.get("github_release") or {}
require(release.get("draft") is False and release.get("prerelease") is False, "release publication flags drifted")
require(release.get("url", "").endswith(EXPECTED_TAG), "release URL mismatch")
gate = registry.get("hosted_release_gate") or {}
require(gate.get("run_id") == 29371898609, "release gate run mismatch")
require(gate.get("head_sha") == EXPECTED_COMMIT, "release gate head mismatch")
require(gate.get("status") == "completed" and gate.get("conclusion") == "success", "release gate status mismatch")

scope = registry.get("release_scope") or {}
require(scope.get("milestone_number") == 30 and scope.get("milestone_state") == "closed", "release milestone mismatch")
require(scope.get("exact_issue_numbers") == EXPECTED_ISSUES, "release issue set mismatch")
require(scope.get("all_issues_closed") is True, "release issues must remain closed")

registry_boundaries = registry.get("boundary_flags")
manifest_boundaries = manifest.get("boundary_flags")
require(isinstance(registry_boundaries, dict), "registry boundary flags missing")
require(set(registry_boundaries) == EXPECTED_BOUNDARIES, "registry boundary field set mismatch")
require(isinstance(manifest_boundaries, dict), "release manifest boundary flags missing")
require(set(manifest_boundaries) == EXPECTED_BOUNDARIES, "release manifest boundary field set mismatch")
for key in sorted(EXPECTED_BOUNDARIES):
    require(registry_boundaries[key] is False, f"boundary must remain explicit false: {key}")
require(registry_boundaries == manifest_boundaries, "registry and manifest boundaries differ")

source_evidence = registry.get("source_evidence") or {}
require(source_evidence.get("audit_strategy") == "source_tree_plus_github_remote", "audit strategy mismatch")
require(source_evidence.get("local_generated_evidence_required") is False, "local generated evidence must remain optional")
require(source_evidence.get("generated_evidence_sole_proof_allowed") is False, "generated evidence cannot be sole proof")
require(source_evidence.get("remote_reconstruction_required") is True, "remote reconstruction must remain required")
source_files = source_evidence.get("files")
require(isinstance(source_files, list) and len(source_files) == 4, "registered source evidence set mismatch")
for item in source_files:
    path = Path(item.get("path", ""))
    require(path.is_file(), f"registered source evidence missing: {path}")
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    require(digest == item.get("sha256"), f"registered source evidence hash mismatch: {path}")

next_track = registry.get("next_track_contract") or {}
require(next_track.get("default_patch_scheduled") is False, "backend patch must remain unscheduled")
require(next_track.get("default_patch_version") is None, "default patch version must remain null")
require(next_track.get("governance_track") == "backend-freeze-governance", "governance track mismatch")
require(next_track.get("next_capability_track") == "v0.33.0+", "next capability family mismatch")
require(next_track.get("capability_entry") == "separately_scoped_only", "v0.33+ entry must remain separately scoped")
for key in sorted(EXPECTED_INHERITANCE_FALSE):
    require(key in next_track, f"missing inheritance boundary: {key}")
    require(next_track[key] is False, f"v0.33+ inheritance must remain false: {key}")

immutability = registry.get("immutability") or {}
for key in [
    "published_tag_rewrite_allowed",
    "published_release_rewrite_allowed",
    "baseline_release_package_routine_edit_allowed",
]:
    require(immutability.get(key) is False, f"immutability boundary drifted: {key}")
require(immutability.get("backend_patch_requires_proven_baseline_invalidity") is True, "patch exception proof requirement missing")
require(immutability.get("backend_freeze_exception_issue_required") is True, "freeze exception issue requirement missing")

for marker in [EXPECTED_TAG, EXPECTED_COMMIT, "source_tree_plus_github_remote", "There is no scheduled v0.32.1 backend patch", "separately_scoped_only"]:
    require(marker in policy, f"freeze policy marker missing: {marker}")
for label, text in [("README", readme), ("ROADMAP", roadmap)]:
    require("No backend patch is scheduled." in text, f"{label} backend patch status missing")
    require("backend-freeze-governance" in text, f"{label} governance track missing")
    require("v0.33.0+" in text, f"{label} capability family missing")
require("none scheduled; baseline-invalidity exception only" in versioning, "versioning backend patch status missing")
require("backend-freeze-governance" in versioning and "v0.33.0+" in versioning, "versioning governance route missing")

print(f"backend_freeze_baseline=pass tag={EXPECTED_TAG} commit={EXPECTED_COMMIT} boundaries={len(EXPECTED_BOUNDARIES)} source_hashes={len(source_files)}")
PY

[[ "$RUN_NEGATIVE_SELFTEST" == "0" || "$RUN_NEGATIVE_SELFTEST" == "1" ]] || fail "NTPRO_BACKEND_FREEZE_NEGATIVE_SELFTEST must be 0 or 1"

if [[ "$RUN_NEGATIVE_SELFTEST" == "1" ]]; then
  selftest_dir="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-backend-freeze-selftest.XXXXXX")"
  trap 'rm -rf "$selftest_dir"' EXIT
  negative_count=0

  run_negative_case() {
    local case_id="$1"
    local operation="$2"
    local json_path="$3"
    local json_value="${4:-null}"
    local expected_error="${5:-boundary must remain explicit false}"
    local mutated="$selftest_dir/${case_id}.json"

    python3 - "$REGISTRY" "$mutated" "$operation" "$json_path" "$json_value" <<'PY'
import json
import sys
from pathlib import Path

source, target, operation, raw_path, raw_value = sys.argv[1:]
payload = json.loads(Path(source).read_text(encoding="utf-8"))
parts = raw_path.split(".")
cursor = payload
for part in parts[:-1]:
    cursor = cursor[int(part)] if isinstance(cursor, list) else cursor[part]
leaf = parts[-1]
if operation == "delete":
    if isinstance(cursor, list):
        del cursor[int(leaf)]
    else:
        del cursor[leaf]
elif operation == "set":
    value = json.loads(raw_value)
    if isinstance(cursor, list):
        cursor[int(leaf)] = value
    else:
        cursor[leaf] = value
else:
    raise SystemExit(f"unknown mutation operation: {operation}")
Path(target).write_text(json.dumps(payload, indent=2, ensure_ascii=True) + "\n", encoding="utf-8")
PY

    if NTPRO_BACKEND_FREEZE_REGISTRY="$mutated" \
      NTPRO_BACKEND_FREEZE_NEGATIVE_SELFTEST=0 \
      "$0" >"$selftest_dir/${case_id}.log" 2>&1; then
      fail "negative selftest accepted drift: $case_id"
    fi
    if ! grep -F -- "$expected_error" "$selftest_dir/${case_id}.log" >/dev/null; then
      cat "$selftest_dir/${case_id}.log" >&2
      fail "negative selftest failed for unexpected reason: $case_id"
    fi
    negative_count=$((negative_count + 1))
  }

  run_negative_case missing_boundary delete boundary_flags.actual_backend_production_go_live_allowed null "registry boundary field set mismatch"
  run_negative_case submit_enabled set boundary_flags.production_order_submission_allowed true
  run_negative_case mutation_enabled set boundary_flags.production_order_mutation_allowed true
  run_negative_case adapter_call_enabled set boundary_flags.execution_adapter_call_allowed true
  run_negative_case adapter_send_enabled set boundary_flags.adapter_send_allowed true
  run_negative_case live_request_enabled set boundary_flags.live_exchange_request_allowed true
  run_negative_case retry_enabled set boundary_flags.retry_scheduler_enabled true
  run_negative_case remediation_enabled set boundary_flags.automatic_remediation_allowed true
  run_negative_case recovery_enabled set boundary_flags.automatic_recovery_allowed true
  run_negative_case dashboard_controls_enabled set boundary_flags.dashboard_trading_controls_enabled true
  run_negative_case admin_controls_enabled set boundary_flags.admin_workbench_trading_controls_enabled true
  run_negative_case terminal_ticket_enabled set boundary_flags.trader_terminal_order_ticket_enabled true
  run_negative_case manual_submit_enabled set boundary_flags.manual_operation_submit_allowed true
  run_negative_case backend_go_live_enabled set boundary_flags.actual_backend_production_go_live_allowed true
  run_negative_case wrong_tag set baseline.tag.name '"ntpro-rust-only-v0.32.1"' "missing local baseline tag"
  run_negative_case wrong_commit set baseline.tag.peeled_commit_sha '"0000000000000000000000000000000000000000"' "baseline commit mismatch"
  run_negative_case wrong_source_hash set source_evidence.files.0.sha256 '"0000000000000000000000000000000000000000000000000000000000000000"' "registered source evidence hash mismatch"
  run_negative_case inherited_submit set next_track_contract.inherits_production_submit true "v0.33+ inheritance must remain false"
  run_negative_case missing_inheritance delete next_track_contract.inherits_trader_terminal_order_ticket null "missing inheritance boundary"
  run_negative_case scheduled_patch set next_track_contract.default_patch_scheduled true "backend patch must remain unscheduled"

  echo "backend_freeze_negative_selftest=pass cases=$negative_count"
fi
