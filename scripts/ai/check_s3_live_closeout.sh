#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

MANIFEST="${NTPRO_S3_CLOSEOUT_MANIFEST:-docs/product/s3_live_closeout_manifest.json}"
MODE="${NTPRO_S3_CLOSEOUT_MODE:-source}"
RUN_NEGATIVE_SELFTEST="${NTPRO_S3_CLOSEOUT_NEGATIVE_SELFTEST:-1}"
EXECUTE_EVIDENCE="${NTPRO_S3_CLOSEOUT_EXECUTE_EVIDENCE:-1}"
EXPECTED_ISSUES='[1288,1290,1292,1294,1296,1298,1300,1302,1306,1308,1310,1312]'
EXPECTED_PRS='[1289,1291,1293,1295,1297,1299,1301,1303,1307,1309,1311]'
EXPECTED_DELIVERY_ISSUES='[1288,1290,1292,1294,1296,1298,1300,1302,1306,1308]'
EXPECTED_DELIVERY_PRS='[1289,1291,1293,1295,1297,1299,1301,1303,1307,1309]'
EXPECTED_REQUIREMENTS='["browser_operator_acceptance","controlled_single_shot_order_path","deterministic_risk_sizing","disconnect_fails_closed","duplicate_and_regressing_reports_fail_closed","independent_live_authorization","interrupted_mutation_requires_manual_recovery","partial_fill_and_manual_cancel","production_real_money_end_to_end_acceptance","strategy_version_and_intent_binding","workspace_rollback_detected_externally"]'
EXPECTED_BOUNDARIES='["additional_orders_allowed","automatic_recovery_allowed","automatic_remediation_allowed","automatic_retry_allowed","backtest_demo_live_permission_inheritance_allowed","bulk_orders_allowed","continuous_strategy_order_submission_allowed","real_money_trade_executed_by_closeout","replace_order_allowed","unapproved_live_order_allowed"]'
EXPECTED_ROADMAP_TOKEN='S3-LV-010 Live 风险预算与仓位 sizing 准入（DONE）'
EXPECTED_PROJECT_TOKEN='S3 正式退出条件尚未满足'
EXPECTED_PRODUCT_CLAIM='S3 technical capability is ready, but the formal product exit remains blocked until issue #1312 records an owner-authorized real-money end-to-end trade.'

fail() {
  echo "s3 live closeout drift: $*" >&2
  exit 1
}

[[ -f "$MANIFEST" ]] || fail "missing manifest: $MANIFEST"
jq -e . "$MANIFEST" >/dev/null || fail "manifest is not valid JSON"

[[ "$(jq -r '.schema_version' "$MANIFEST")" == "ntpro.s3_live_product_closeout.v1" ]] \
  || fail "unexpected schema_version"
review_gate="$(jq -r '.documents.review_gate' "$MANIFEST")"
manifest_status="$(jq -r '.status' "$MANIFEST")"
case "$review_gate:$manifest_status" in
  REVIEW_REQUIRED:blocked_by_s3_lv_011_review_required) ;;
  REVIEW_APPROVED:blocked_by_s3_lv_011_review_approved) ;;
  *) fail "manifest status and review gate are inconsistent" ;;
esac
[[ "$(jq -c '.scope.exact_issue_numbers' "$MANIFEST")" == "$EXPECTED_ISSUES" ]] \
  || fail "exact issue scope mismatch"
[[ "$(jq -c '.scope.exact_pull_request_numbers' "$MANIFEST")" == "$EXPECTED_PRS" ]] \
  || fail "exact pull request scope mismatch"
[[ "$(jq -c '.scope.delivery_issue_numbers' "$MANIFEST")" == "$EXPECTED_DELIVERY_ISSUES" ]] \
  || fail "delivery issue scope mismatch"
[[ "$(jq -c '.scope.delivery_pull_request_numbers' "$MANIFEST")" == "$EXPECTED_DELIVERY_PRS" ]] \
  || fail "delivery pull request scope mismatch"
[[ "$(jq -r '.milestone.number' "$MANIFEST")" == "39" ]] || fail "milestone number mismatch"
[[ "$(jq -r '.milestone.title' "$MANIFEST")" == "S3 Live 产品化" ]] \
  || fail "milestone title mismatch"
[[ "$(jq -r '.milestone.premerge_state' "$MANIFEST")" == "open" ]] \
  || fail "premerge milestone policy mismatch"
[[ "$(jq -r '.milestone.postmerge_state' "$MANIFEST")" == "open_blocked" ]] \
  || fail "postmerge milestone policy mismatch"
[[ "$(jq -r '.closeout.issue_number' "$MANIFEST")" == "1310" ]] \
  || fail "closeout issue mismatch"
[[ "$(jq -r '.closeout.pull_request_number' "$MANIFEST")" == "1311" ]] \
  || fail "closeout pull request mismatch"
[[ "$(jq -r '.closeout.blocker_issue_number' "$MANIFEST")" == "1312" ]] \
  || fail "closeout blocker mismatch"

jq -e '
  (.requirements | length) == 11
  and ([.requirements[] | select(.status == "satisfied")] | length) == 10
  and ([.requirements[] | select(.status == "blocked")] | length) == 1
  and any(
    .requirements[];
    .id == "production_real_money_end_to_end_acceptance"
      and .status == "blocked"
      and .blocker_issue_number == 1312
      and (.evidence | length) == 0
  )
' "$MANIFEST" >/dev/null || fail "requirement status contract mismatch"
[[ "$(jq -c '[.requirements[].id] | sort' "$MANIFEST")" == "$EXPECTED_REQUIREMENTS" ]] \
  || fail "requirement ids do not match the exact contract"
jq -e '.boundaries | to_entries | length == 10 and all(.value == false)' "$MANIFEST" >/dev/null \
  || fail "one or more forbidden boundaries are enabled"
[[ "$(jq -c '.boundaries | keys' "$MANIFEST")" == "$EXPECTED_BOUNDARIES" ]] \
  || fail "boundary keys do not match the exact contract"

while IFS=$'\t' read -r requirement_id evidence_path symbol; do
  jq -e --arg id "$requirement_id" --arg path "$evidence_path" --arg symbol "$symbol" \
    '.requirements[] | select(.id == $id) | .evidence[] | select(.path == $path and .symbol == $symbol)' \
    "$MANIFEST" >/dev/null || fail "mandatory evidence is not registered: $requirement_id/$symbol"
done <<'EVIDENCE'
independent_live_authorization	crates/cli/src/dashboard/server/tests.rs	live_execution_approvals_require_three_distinct_role_sessions
strategy_version_and_intent_binding	crates/cli/src/dashboard/product_api/live_run.rs	strategy_intent_is_single_use_across_live_runs
controlled_single_shot_order_path	crates/cli/src/dashboard/product_api/live_run.rs	live_execution_admission_is_fail_closed_single_use_and_externally_anchored
disconnect_fails_closed	crates/cli/src/dashboard/product_api/live_run.rs	live_market_data_runtime_external_exit_is_failed_anchored_and_released
duplicate_and_regressing_reports_fail_closed	crates/cli/src/live/node_runtime.rs	exchange_report_must_match_admitted_quantity_and_venue_order
duplicate_and_regressing_reports_fail_closed	crates/cli/src/dashboard/product_api/live_run.rs	execution_order_progression_accepts_partial_fill_and_rejects_regression
partial_fill_and_manual_cancel	crates/cli/src/live/node_runtime.rs	partial_fill_reconciliation_preserves_quantity_and_never_retries
partial_fill_and_manual_cancel	crates/cli/src/dashboard/server/tests.rs	live_execution_cancel_approvals_require_owner_then_operator_sessions
interrupted_mutation_requires_manual_recovery	crates/cli/src/live/node_runtime.rs	interrupted_cancel_is_single_use_and_requires_manual_review
interrupted_mutation_requires_manual_recovery	crates/cli/src/dashboard/product_api/live_run.rs	cancel_publication_recovers_every_owner_and_operator_write_boundary
workspace_rollback_detected_externally	crates/cli/src/dashboard/product_api/live_run.rs	external_anchor_detects_complete_workspace_snapshot_rollback
deterministic_risk_sizing	crates/cli/src/dashboard/product_api/live_run.rs	live_sizing_rounds_down_and_binds_budget
deterministic_risk_sizing	crates/cli/src/dashboard/product_api/live_run.rs	live_sizing_applies_account_budget_fraction_to_sell_inventory
browser_operator_acceptance	apps/strategy-workbench/src/app/AppShell.test.tsx	shows partial-fill reconciliation and submits one owner cancel request
browser_operator_acceptance	apps/strategy-workbench/tests/e2e/workbench.spec.ts	Live page reconciles a partial fill and submits one owner cancel approval
EVIDENCE
[[ "$(jq -r '[.requirements[].evidence[]] | length' "$MANIFEST")" == "15" ]] \
  || fail "unexpected evidence registration count"

while IFS=$'\t' read -r evidence_path symbol; do
  [[ -f "$evidence_path" ]] || fail "missing evidence path: $evidence_path"
  grep -Fq -- "$symbol" "$evidence_path" || fail "missing evidence symbol '$symbol' in $evidence_path"
done < <(jq -r '.requirements[].evidence[] | [.path, .symbol] | @tsv' "$MANIFEST")

for id in 001 002 003 004 005 006 007 008 009 010; do
  grep -Fxq 'Status: DONE' "docs/rust-cutover/tasks/S3-LV-$id.md" \
    || fail "task S3-LV-$id is not DONE"
  grep -Fxq 'Status: DONE' "docs/rust-cutover/evidence/S3-LV-$id.md" \
    || fail "evidence S3-LV-$id is not DONE"
done
grep -Fxq 'Status: DONE_ON_MERGE' docs/rust-cutover/tasks/S3-CLS-001.md \
  || fail "closeout task status mismatch"
grep -Fxq 'Status: DONE_ON_MERGE' docs/rust-cutover/evidence/S3-CLS-001.md \
  || fail "closeout evidence status mismatch"
grep -Fxq "Review gate: $review_gate" docs/rust-cutover/tasks/S3-CLS-001.md \
  || fail "closeout task review gate mismatch"
grep -Fxq "Review gate: $review_gate" docs/rust-cutover/evidence/S3-CLS-001.md \
  || fail "closeout evidence review gate mismatch"

[[ "$(jq -r '.documents.delivery_task_status' "$MANIFEST")" == "DONE" ]] \
  || fail "manifest delivery task status mismatch"
[[ "$(jq -r '.documents.delivery_evidence_status' "$MANIFEST")" == "DONE" ]] \
  || fail "manifest delivery evidence status mismatch"
[[ "$(jq -r '.documents.closeout_task_status' "$MANIFEST")" == "DONE_ON_MERGE" ]] \
  || fail "manifest closeout task status mismatch"
[[ "$(jq -r '.documents.closeout_evidence_status' "$MANIFEST")" == "DONE_ON_MERGE" ]] \
  || fail "manifest closeout evidence status mismatch"
[[ "$(jq -r '.documents.roadmap_token' "$MANIFEST")" == "$EXPECTED_ROADMAP_TOKEN" ]] \
  || fail "manifest roadmap token mismatch"
[[ "$(jq -r '.documents.project_token' "$MANIFEST")" == "$EXPECTED_PROJECT_TOKEN" ]] \
  || fail "manifest project token mismatch"
[[ "$(jq -r '.product_claim' "$MANIFEST")" == "$EXPECTED_PRODUCT_CLAIM" ]] \
  || fail "product claim mismatch"
grep -Fq -- "$EXPECTED_ROADMAP_TOKEN" docs/product/roadmap.md || fail "roadmap status token missing"
grep -Fq -- "$EXPECTED_PROJECT_TOKEN" project.html || fail "project status token missing"

gh_api_retry() {
  local endpoint="$1"
  local attempt
  for attempt in 1 2 3; do
    if gh api "$endpoint" 2>/dev/null; then
      return 0
    fi
    sleep "$attempt"
  done
  fail "GitHub API unavailable after 3 attempts: $endpoint"
}

require_successful_smoke() {
  local head_sha="$1"
  local runs
  runs="$(gh_api_retry "repos/atxinbao/NTPRO/actions/runs?head_sha=$head_sha&status=completed&per_page=100")"
  jq -e --arg sha "$head_sha" '
    any(
      .workflow_runs[];
      .name == "Rust Cutover Smoke"
        and .event == "pull_request"
        and .head_sha == $sha
        and .conclusion == "success"
    )
  ' <<<"$runs" >/dev/null || fail "no successful pull-request Rust Cutover Smoke for $head_sha"
}

require_merge_on_main() {
  local merge_sha="$1"
  local comparison status
  comparison="$(gh_api_retry "repos/atxinbao/NTPRO/compare/$merge_sha...main")"
  status="$(jq -r '.status' <<<"$comparison")"
  [[ "$status" == "ahead" || "$status" == "identical" ]] \
    || fail "merge commit $merge_sha is not an ancestor of main"
}

check_delivery_pair() {
  local task_id="$1"
  local issue_number="$2"
  local pr_number="$3"
  local issue_json pr_json head_sha merge_sha
  issue_json="$(gh_api_retry "repos/atxinbao/NTPRO/issues/$issue_number")"
  pr_json="$(gh_api_retry "repos/atxinbao/NTPRO/pulls/$pr_number")"
  [[ "$(jq -r '.state' <<<"$issue_json")" == "closed" ]] \
    || fail "$task_id issue #$issue_number is not closed"
  [[ "$(jq -r '.title' <<<"$issue_json")" == "$task_id:"* ]] \
    || fail "$task_id issue title mismatch"
  grep -Fq "Closes #$issue_number" <<<"$(jq -r '.body // ""' <<<"$pr_json")" \
    || fail "PR #$pr_number does not bind issue #$issue_number"
  [[ "$(jq -r '.base.ref' <<<"$pr_json")" == "main" ]] || fail "PR #$pr_number base mismatch"
  [[ -n "$(jq -r '.merged_at // empty' <<<"$pr_json")" ]] || fail "PR #$pr_number is not merged"
  head_sha="$(jq -r '.head.sha' <<<"$pr_json")"
  merge_sha="$(jq -r '.merge_commit_sha' <<<"$pr_json")"
  require_successful_smoke "$head_sha"
  require_merge_on_main "$merge_sha"
}

check_live_scope() {
  command -v gh >/dev/null || fail "gh is required for live mode"
  local milestone issues actual_issues closeout_issue blocker_issue closeout_pr state head_sha merge_sha
  milestone="$(gh_api_retry 'repos/atxinbao/NTPRO/milestones/39')"
  issues="$(gh_api_retry 'repos/atxinbao/NTPRO/issues?milestone=39&state=all&per_page=100')"
  actual_issues="$(jq -c '[.[] | select(has("pull_request") | not) | .number] | sort' <<<"$issues")"
  [[ "$actual_issues" == "$EXPECTED_ISSUES" ]] || fail "live milestone issue scope mismatch"

  while read -r task_id issue_number pr_number; do
    check_delivery_pair "$task_id" "$issue_number" "$pr_number"
  done <<'DELIVERIES'
S3-LV-001 1288 1289
S3-LV-002 1290 1291
S3-LV-003 1292 1293
S3-LV-004 1294 1295
S3-LV-005 1296 1297
S3-LV-006 1298 1299
S3-LV-007 1300 1301
S3-LV-008 1302 1303
S3-LV-009 1306 1307
S3-LV-010 1308 1309
DELIVERIES

  state="$(jq -r '.state' <<<"$milestone")"
  closeout_issue="$(gh_api_retry 'repos/atxinbao/NTPRO/issues/1310')"
  blocker_issue="$(gh_api_retry 'repos/atxinbao/NTPRO/issues/1312')"
  closeout_pr="$(gh_api_retry 'repos/atxinbao/NTPRO/pulls/1311')"
  [[ "$(jq -r '.title' <<<"$blocker_issue")" == "S3-LV-011:"* ]] \
    || fail "blocker issue title mismatch"
  [[ "$(jq -r '.state' <<<"$blocker_issue")" == "open" ]] \
    || fail "S3-LV-011 must remain open"
  [[ "$(jq -r '.head.ref' <<<"$closeout_pr")" == "codex/s3-cls-001-live-product-closeout" ]] \
    || fail "closeout PR head mismatch"
  [[ "$(jq -r '.base.ref' <<<"$closeout_pr")" == "main" ]] || fail "closeout PR base mismatch"
  grep -Fq 'Closes #1310' <<<"$(jq -r '.body // ""' <<<"$closeout_pr")" \
    || fail "closeout PR does not close issue #1310"
  head_sha="$(jq -r '.head.sha' <<<"$closeout_pr")"
  require_successful_smoke "$head_sha"

  case "$MODE" in
    live-premerge)
      [[ "$state" == "open" ]] || fail "premerge milestone must be open"
      [[ "$(jq -r '.open_issues' <<<"$milestone")" == "2" ]] || fail "premerge open issue count mismatch"
      [[ "$(jq -r '.closed_issues' <<<"$milestone")" == "10" ]] || fail "premerge closed issue count mismatch"
      [[ "$(jq -r '.state' <<<"$closeout_issue")" == "open" ]] \
        || fail "closeout issue must be open before merge"
      [[ "$(jq -r '.state' <<<"$closeout_pr")" == "open" ]] \
        || fail "closeout PR must be open before merge"
      ;;
    live-postmerge-blocked)
      [[ "$review_gate" == "REVIEW_APPROVED" ]] \
        || fail "postmerge guard requires REVIEW_APPROVED"
      [[ "$state" == "open" ]] || fail "blocked S3 milestone must remain open"
      [[ "$(jq -r '.open_issues' <<<"$milestone")" == "1" ]] || fail "postmerge open issue count mismatch"
      [[ "$(jq -r '.closed_issues' <<<"$milestone")" == "11" ]] || fail "postmerge closed issue count mismatch"
      [[ "$(jq -r '.state' <<<"$closeout_issue")" == "closed" ]] \
        || fail "closeout audit issue is not closed"
      [[ -n "$(jq -r '.merged_at // empty' <<<"$closeout_pr")" ]] \
        || fail "closeout PR is not merged"
      merge_sha="$(jq -r '.merge_commit_sha' <<<"$closeout_pr")"
      require_merge_on_main "$merge_sha"
      ;;
  esac
}

case "$MODE" in
  source) ;;
  live-premerge|live-postmerge-blocked)
    check_live_scope
    case "$EXECUTE_EVIDENCE" in
      1) scripts/ai/test_s3_live_exit_evidence.sh ;;
      0) ;;
      *) fail "NTPRO_S3_CLOSEOUT_EXECUTE_EVIDENCE must be 0 or 1" ;;
    esac
    ;;
  *) fail "unknown mode '$MODE'" ;;
esac

case "$RUN_NEGATIVE_SELFTEST" in
  0) ;;
  1) scripts/ai/test_s3_live_closeout.sh ;;
  *) fail "NTPRO_S3_CLOSEOUT_NEGATIVE_SELFTEST must be 0 or 1" ;;
esac

echo "s3_live_closeout=pass mode=$MODE requirements=11 satisfied=10 blocked=1 boundaries=10 issues=12 prs=11"
