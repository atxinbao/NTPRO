#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

MANIFEST="${NTPRO_MVP_FREEZE_MANIFEST:-docs/product/mvp_freeze_manifest.json}"
RELEASE_DOC="${NTPRO_MVP_FREEZE_RELEASE_DOC:-docs/product/mvp_release_and_rollback.md}"
ROADMAP="${NTPRO_MVP_FREEZE_ROADMAP:-docs/product/roadmap.md}"
PROJECT_PAGE="${NTPRO_MVP_FREEZE_PROJECT_PAGE:-project.html}"
RUN_NEGATIVE_SELFTEST="${NTPRO_MVP_FREEZE_NEGATIVE_SELFTEST:-1}"

fail() {
  echo "MVP freeze baseline drift: $*" >&2
  exit 1
}

case "$RUN_NEGATIVE_SELFTEST" in
  0) negative_selftest=false ;;
  1) negative_selftest=true ;;
  *) fail "NTPRO_MVP_FREEZE_NEGATIVE_SELFTEST must be 0 or 1" ;;
esac

for file in "$MANIFEST" "$RELEASE_DOC" "$ROADMAP" "$PROJECT_PAGE"; do
  [[ -f "$file" ]] || fail "required file missing: $file"
done

jq -e '
  .schema_version == "ntpro.mvp_freeze_manifest.v1"
  and .task_id == "MVP-013"
  and .github_issue == 1233
  and .github_pr == 1236
  and .status == "frozen_on_merge"
  and .baseline.name == "single_supervisor_single_sandbox_node_mvp"
  and .baseline.backend_freeze_tag == "ntpro-rust-only-v0.32.0"
  and .baseline.current_backend_maintenance_tag == "ntpro-rust-only-v0.33.0"
  and .baseline.creates_release_tag == false
  and .baseline.creates_github_release == false
  and .baseline.production_trading_authority == false
  and .phases == {"M0":"complete","M1":"complete","M2":"complete","M3":"complete","M4":"frozen_on_merge"}
  and .topology == {
    "supervisor_count":1,
    "node_count":1,
    "strategy_instance_count":1,
    "account_count":1,
    "venue_count":1,
    "environment":"sandbox",
    "multi_node_orchestration":false
  }
  and .product_surfaces.cli_entrypoint == "nautilus mvp serve"
  and .product_surfaces.institution_workbench == "/institution-workbench"
  and .product_surfaces.control_center == "/control-center"
  and .product_surfaces.shared_status_api == "GET /api/mvp/v1/status"
  and .product_surfaces.operator_actions == ["start","stop"]
  and .product_surfaces.institution_workbench_read_only == true
  and .acceptance.fault_case_count == 11
  and .acceptance.browser_viewports == ["1440x1000","390x844"]
  and .acceptance.performance_workload_count == 6
  and (.boundary_flags | length) == 19
  and ([.boundary_flags[] | select(. != false)] | length) == 0
  and .post_freeze_change_policy.separate_issue_required == true
  and .post_freeze_change_policy.explicit_owner_approval_required == true
  and .post_freeze_change_policy.risk_and_rollback_evidence_required == true
  and .post_freeze_change_policy.freeze_manifest_update_required == true
  and .post_freeze_change_policy.inherits_forbidden_capability == false
' "$MANIFEST" >/dev/null || fail "manifest contract mismatch"

expected_tasks='MVP-001,MVP-001A,MVP-002,MVP-003,MVP-004,MVP-005,MVP-005A,MVP-006,MVP-007,MVP-007A,MVP-008,MVP-008A,MVP-009,MVP-009A,MVP-010,MVP-010A,MVP-011,MVP-012,MVP-013'
actual_tasks="$(jq -r '.exact_task_ids | join(",")' "$MANIFEST")"
[[ "$actual_tasks" == "$expected_tasks" ]] || fail "exact task scope mismatch"

expected_issues='1197,1199,1201,1203,1205,1207,1209,1211,1213,1215,1217,1219,1221,1223,1225,1227,1229,1231,1233'
actual_issues="$(jq -r '.exact_issue_numbers | join(",")' "$MANIFEST")"
[[ "$actual_issues" == "$expected_issues" ]] || fail "exact issue scope mismatch"

expected_prs='1198,1200,1202,1204,1206,1208,1210,1212,1214,1216,1218,1220,1222,1224,1226,1228,1230,1232,1236'
actual_prs="$(jq -r '.exact_pr_numbers | join(",")' "$MANIFEST")"
[[ "$actual_prs" == "$expected_prs" ]] || fail "exact PR scope mismatch"

while IFS=$'\t' read -r source expected_sha; do
  [[ -f "$source" ]] || fail "frozen source missing: $source"
  actual_sha="$(shasum -a 256 "$source" | awk '{print $1}')"
  [[ "$actual_sha" == "$expected_sha" ]] || fail "frozen source changed: $source"
done < <(jq -r '.frozen_source_sha256 | to_entries[] | [.key, .value] | @tsv' "$MANIFEST")

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

if "$negative_selftest"; then
  tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-mvp-freeze.XXXXXX")"
  trap 'rm -rf "$tmpdir"' EXIT

  expect_rejected() {
    local name="$1"
    local candidate="$2"
    if NTPRO_MVP_FREEZE_MANIFEST="$candidate" \
      NTPRO_MVP_FREEZE_NEGATIVE_SELFTEST=0 \
      "$0" >/dev/null 2>&1; then
      fail "negative selftest accepted $name"
    fi
  }

  jq '.boundary_flags.order_submission_allowed = true' "$MANIFEST" >"$tmpdir/open-boundary.json"
  expect_rejected "true boundary" "$tmpdir/open-boundary.json"

  jq 'del(.boundary_flags.order_submission_allowed)' "$MANIFEST" >"$tmpdir/missing-boundary.json"
  expect_rejected "missing boundary" "$tmpdir/missing-boundary.json"

  jq '.phases.M4 = "complete"' "$MANIFEST" >"$tmpdir/early-completion.json"
  expect_rejected "premature M4 completion" "$tmpdir/early-completion.json"

  jq '.frozen_source_sha256["crates/cli/src/mvp.rs"] = "0000000000000000000000000000000000000000000000000000000000000000"' \
    "$MANIFEST" >"$tmpdir/source-drift.json"
  expect_rejected "source hash drift" "$tmpdir/source-drift.json"

  echo "mvp_freeze_negative_selftest=pass cases=4"
fi

echo "mvp_freeze_baseline=pass tasks=19 phases=5 boundaries=19 frozen_sources=13 browser_viewports=2 performance_workloads=6"
