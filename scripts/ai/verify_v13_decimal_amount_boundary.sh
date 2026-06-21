#!/usr/bin/env bash
set -euo pipefail

# V130-006: Decimal/string-only amount boundary gate.
# This script is CI-safe. It does not open network connections, does not read
# credentials, and does not submit or mutate production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

echo "== v13 Decimal boundary: targeted tests =="
cargo test -p nautilus-cli v13_live_alpha_amount_boundary_uses_decimal_strings_without_f64 --lib
cargo test -p nautilus-cli production_shadow_portfolio_runtime_preserves_decimal_string_notional_preflight --lib

echo "== v13 Decimal boundary: positive contract markers =="
grep -nE \
  "amount_boundary=decimal_string_only|parser=rust_decimal|aggregation=rust_decimal_string_sum|f64_aggregation_used=false|live_alpha_money_math_ready=false|risk_or_execution_grade=false|scientific_notation_allowed=false|production_order_submission_allowed=false|production_order_mutation_allowed=false" \
  docs/rust-cutover/release/v0_13_0_decimal_amount_boundary.md \
  docs/rust-cutover/evidence/V130-006.md >/dev/null

echo "== v13 Decimal boundary: scope linkage =="
grep -nE \
  "Decimal/string-only risk and execution amount boundary|Decimal/string-only amount handling" \
  docs/rust-cutover/release/v0_13_0_scope_decision.md >/dev/null

echo "== v13 Decimal boundary: source parser markers =="
grep -nE \
  "parse_non_negative_decimal|Decimal|string_sum|f64_aggregation_used: false|live_alpha_money_math_ready: false|risk_or_execution_grade: false" \
  crates/cli/src/live.rs >/dev/null

echo "== v13 Decimal boundary: forbidden release claims =="
if grep -nE \
  "amount_boundary=f64|f64_aggregation_used=true|live_alpha_money_math_ready=true|risk_or_execution_grade=true|scientific_notation_allowed=true|production_order_submission_allowed=true|production_order_mutation_allowed=true|production trading ready|real funds ready" \
  docs/rust-cutover/release/v0_13_0_decimal_amount_boundary.md \
  docs/rust-cutover/evidence/V130-006.md \
  docs/rust-cutover/tasks/V130-006.md >/dev/null; then
  echo "v13 Decimal boundary docs contain an enabled money-math or production mutation claim" >&2
  exit 1
fi

echo "v13_decimal_amount_boundary status=ok amount_boundary=decimal_string_only f64_aggregation_used=false live_alpha_money_math_ready=false"
