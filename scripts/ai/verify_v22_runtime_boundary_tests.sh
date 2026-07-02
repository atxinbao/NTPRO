#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

echo "== v22 runtime boundary tests: degraded/fail-closed cases =="
cargo test -p nautilus-cli trader_terminal_v220_runtime_degradation_cases_disable_operation_controls --lib -- --nocapture

echo "== v22 runtime boundary tests: forbidden controls =="
cargo test -p nautilus-cli trader_terminal_v220_forbidden_controls_fail_closed_individually --lib -- --nocapture

echo "== v22 runtime boundary tests: display claims =="
cargo test -p nautilus-cli trader_terminal_v220_display_claim_boundary_stays_read_only_first --lib -- --nocapture

echo "== v22 runtime boundary tests: operation entry regression =="
cargo test -p nautilus-cli trader_terminal_operation_entry --lib -- --nocapture

echo "v22_runtime_boundary_tests status=ok degraded_or_fail_closed=covered forbidden_controls=covered read_only_first_boundary=covered product_grade_terminal_claim=false"
