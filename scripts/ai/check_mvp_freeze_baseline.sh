#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

readonly MANIFEST="docs/product/mvp_freeze_manifest.json"
readonly RELEASE_DOC="docs/product/mvp_release_and_rollback.md"
readonly ROADMAP="docs/product/roadmap.md"
readonly PROJECT_PAGE="project.html"

fail() {
  echo "MVP freeze baseline drift: $*" >&2
  exit 1
}

for file in "$MANIFEST" "$RELEASE_DOC" "$ROADMAP" "$PROJECT_PAGE"; do
  [[ -f "$file" ]] || fail "required file missing: $file"
done

manifest_contract_valid() {
  local candidate="$1"
  jq -e '
    (. | keys) == [
      "acceptance",
      "baseline",
      "boundary_flags",
      "exact_issue_numbers",
      "exact_pr_numbers",
      "exact_task_ids",
      "executor",
      "freeze_date",
      "frozen_source_sha256",
      "github_issue",
      "github_pr",
      "phases",
      "post_freeze_change_policy",
      "product_surfaces",
      "schema_version",
      "status",
      "task_id",
      "topology"
    ]
    and .schema_version == "ntpro.mvp_freeze_manifest.v1"
    and .task_id == "MVP-013"
    and .github_issue == 1233
    and .github_pr == 1236
    and .status == "frozen_on_merge"
    and .freeze_date == "2026-08-05"
    and .executor == "Codex"
    and .baseline == {
      "name":"single_supervisor_single_sandbox_node_mvp",
      "backend_freeze_tag":"ntpro-rust-only-v0.32.0",
      "current_backend_maintenance_tag":"ntpro-rust-only-v0.33.0",
      "creates_release_tag":false,
      "creates_github_release":false,
      "production_trading_authority":false
    }
    and .phases == {"M0":"complete","M1":"complete","M2":"complete","M3":"complete","M4":"frozen_on_merge"}
    and .exact_task_ids == ["MVP-001","MVP-001A","MVP-002","MVP-003","MVP-004","MVP-005","MVP-005A","MVP-006","MVP-007","MVP-007A","MVP-008","MVP-008A","MVP-009","MVP-009A","MVP-010","MVP-010A","MVP-011","MVP-012","MVP-013"]
    and .exact_issue_numbers == [1197,1199,1201,1203,1205,1207,1209,1211,1213,1215,1217,1219,1221,1223,1225,1227,1229,1231,1233]
    and .exact_pr_numbers == [1198,1200,1202,1204,1206,1208,1210,1212,1214,1216,1218,1220,1222,1224,1226,1228,1230,1232,1236]
    and .topology == {
      "supervisor_count":1,
      "node_count":1,
      "strategy_instance_count":1,
      "account_count":1,
      "venue_count":1,
      "environment":"sandbox",
      "multi_node_orchestration":false
    }
    and .product_surfaces == {
      "cli_entrypoint":"nautilus mvp serve",
      "institution_workbench":"/institution-workbench",
      "control_center":"/control-center",
      "shared_status_api":"GET /api/mvp/v1/status",
      "event_correlation_api":"GET /api/mvp/v1/event-correlation",
      "control_center_api":"GET /api/mvp/v1/control-center",
      "operator_actions":["start","stop"],
      "institution_workbench_read_only":true
    }
    and .acceptance == {
      "deterministic_loop":"scripts/ai/test_mvp_acceptance.mjs",
      "fault_matrix":"scripts/ai/test_mvp_fault_matrix.mjs",
      "fault_case_count":11,
      "institution_browser":"scripts/ai/test_institution_workbench_browser.mjs",
      "control_center_browser":"scripts/ai/test_control_center_browser.mjs",
      "browser_viewports":["1440x1000","390x844"],
      "performance_workflow":".github/workflows/backend-performance.yml",
      "performance_contract":"docs/rust-cutover/governance/backend_performance_hosted_contract.json",
      "performance_workload_count":6,
      "release_and_rollback":"docs/product/mvp_release_and_rollback.md"
    }
    and .boundary_flags == {
      "external_venue_connection":false,
      "production_venue_connection":false,
      "external_network_attempted":false,
      "real_orders_submitted":false,
      "order_submission_allowed":false,
      "order_mutation_allowed":false,
      "cancel_order_allowed":false,
      "replace_order_allowed":false,
      "amend_order_allowed":false,
      "flatten_position_allowed":false,
      "automatic_retry_allowed":false,
      "automatic_remediation_allowed":false,
      "automatic_recovery_allowed":false,
      "multi_node_orchestration_allowed":false,
      "production_iam_claim":false,
      "product_grade_live_terminal_claim":false,
      "backtest_completion_implies_trading_readiness":false,
      "http_success_implies_technical_health":false,
      "process_alive_implies_technical_health":false
    }
    and (.frozen_source_sha256 | keys) == [
      "configs/nodes/btc-ema-shadow.toml",
      "crates/cli/src/dashboard/control_center.rs",
      "crates/cli/src/dashboard/institution_workbench.rs",
      "crates/cli/src/dashboard/mvp_status_api.rs",
      "crates/cli/src/dashboard/server.rs",
      "crates/cli/src/mvp.rs",
      "crates/cli/src/mvp_contract.rs",
      "crates/cli/src/supervisor.rs",
      "examples/rust/backtest/minimal_engine_smoke.toml",
      "scripts/ai/test_control_center_browser.mjs",
      "scripts/ai/test_institution_workbench_browser.mjs",
      "scripts/ai/test_mvp_acceptance.mjs",
      "scripts/ai/test_mvp_fault_matrix.mjs"
    ]
    and .post_freeze_change_policy == {
      "separate_issue_required":true,
      "explicit_owner_approval_required":true,
      "risk_and_rollback_evidence_required":true,
      "freeze_manifest_update_required":true,
      "inherits_forbidden_capability":false
    }
  ' "$candidate" >/dev/null
}

source_hashes_valid() {
  local candidate="$1"
  local source expected_sha actual_sha
  while IFS=$'\t' read -r source expected_sha; do
    [[ -f "$source" ]] || return 1
    actual_sha="$(shasum -a 256 "$source" | awk '{print $1}')"
    [[ "$actual_sha" == "$expected_sha" ]] || return 1
  done < <(jq -r '.frozen_source_sha256 | to_entries[] | [.key, .value] | @tsv' "$candidate")
}

manifest_contract_valid "$MANIFEST" || fail "manifest contract mismatch"
source_hashes_valid "$MANIFEST" || fail "frozen source set or hash mismatch"

for phrase in \
  'Status: FROZEN ON MVP-013 MERGE' \
  '不创建 tag 或 GitHub Release' \
  '不得使用自动 restart/retry' \
  '必须建立独立 GitHub issue'; do
  grep -Fq "$phrase" "$RELEASE_DOC" || fail "release/rollback statement missing: $phrase"
done

grep -Fq 'M4：MVP 验收与冻结（MVP-013 合并即完成）' "$ROADMAP" \
  || fail "roadmap does not bind M4 completion to MVP-013 merge"
grep -Fq 'MVP-013 最终验收与冻结' "$PROJECT_PAGE" \
  || fail "project page does not expose the final MVP freeze candidate"

tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-mvp-freeze.XXXXXX")"
trap 'rm -rf "$tmpdir"' EXIT

expect_rejected() {
  local name="$1"
  local candidate="$2"
  if manifest_contract_valid "$candidate" && source_hashes_valid "$candidate"; then
    fail "negative selftest accepted $name"
  fi
}

jq '.boundary_flags.order_submission_allowed = true' "$MANIFEST" >"$tmpdir/open-boundary.json"
expect_rejected "true boundary" "$tmpdir/open-boundary.json"

jq 'del(.boundary_flags.order_submission_allowed)' "$MANIFEST" >"$tmpdir/missing-boundary.json"
expect_rejected "missing boundary" "$tmpdir/missing-boundary.json"

jq 'del(.boundary_flags.order_submission_allowed) | .boundary_flags.unrelated_placeholder = false' \
  "$MANIFEST" >"$tmpdir/substituted-boundary.json"
expect_rejected "substituted boundary" "$tmpdir/substituted-boundary.json"

jq '.phases.M4 = "complete"' "$MANIFEST" >"$tmpdir/early-completion.json"
expect_rejected "premature M4 completion" "$tmpdir/early-completion.json"

jq '.frozen_source_sha256["crates/cli/src/mvp.rs"] = "0000000000000000000000000000000000000000000000000000000000000000"' \
  "$MANIFEST" >"$tmpdir/source-drift.json"
expect_rejected "source hash drift" "$tmpdir/source-drift.json"

jq 'del(.frozen_source_sha256["crates/cli/src/mvp.rs"])' \
  "$MANIFEST" >"$tmpdir/missing-source.json"
expect_rejected "missing frozen source" "$tmpdir/missing-source.json"

echo "mvp_freeze_negative_selftest=pass cases=6"

echo "mvp_freeze_baseline=pass tasks=19 phases=5 boundaries=19 frozen_sources=13 browser_viewports=2 performance_workloads=6"
