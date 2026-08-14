#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

rust_tests=(
  dashboard::server::tests::live_execution_approvals_require_three_distinct_role_sessions
  dashboard::product_api::live_run::tests::strategy_intent_is_single_use_across_live_runs
  dashboard::product_api::live_run::tests::live_execution_admission_is_fail_closed_single_use_and_externally_anchored
  dashboard::product_api::live_run::tests::live_market_data_runtime_external_exit_is_failed_anchored_and_released
  live::node_runtime::execution_authority_tests::exchange_report_must_match_admitted_quantity_and_venue_order
  dashboard::product_api::live_run::tests::execution_order_progression_accepts_partial_fill_and_rejects_regression
  live::node_runtime::execution_authority_tests::partial_fill_reconciliation_preserves_quantity_and_never_retries
  dashboard::server::tests::live_execution_cancel_approvals_require_owner_then_operator_sessions
  live::node_runtime::execution_authority_tests::interrupted_cancel_is_single_use_and_requires_manual_review
  dashboard::product_api::live_run::tests::cancel_publication_recovers_every_owner_and_operator_write_boundary
  dashboard::product_api::live_run::tests::external_anchor_detects_complete_workspace_snapshot_rollback
  dashboard::product_api::live_run::tests::live_sizing_rounds_down_and_binds_budget
  dashboard::product_api::live_run::tests::live_sizing_applies_account_budget_fraction_to_sell_inventory
)

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-s3-live-evidence.XXXXXX")"
trap 'find "$tmp_dir" -depth -delete 2>/dev/null || true' EXIT

for test_name in "${rust_tests[@]}"; do
  test_output="$tmp_dir/$(printf '%s' "$test_name" | tr ':/' '__').log"
  cargo test -p nautilus-cli --lib "$test_name" -- --exact 2>&1 | tee "$test_output"
  grep -Fq "test $test_name ... ok" "$test_output" || {
    echo "S3 Live evidence test did not execute exactly once: $test_name" >&2
    exit 1
  }
done

(
  cd apps/strategy-workbench
  [[ -x node_modules/.bin/vitest ]] || {
    echo "S3 Live evidence requires frontend dependencies; run npm ci in apps/strategy-workbench" >&2
    exit 1
  }
  npm run test -- src/app/AppShell.test.tsx \
    -t 'shows partial-fill reconciliation and submits one owner cancel request'
  npm run test:e2e -- \
    -g 'Live page reconciles a partial fill and submits one owner cancel approval'
)

echo "s3_live_exit_evidence=pass rust_tests=${#rust_tests[@]} frontend_tests=2 real_money_orders=0"
