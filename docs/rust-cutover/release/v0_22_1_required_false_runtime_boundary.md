# v0.22.1 Required-false Runtime Boundary

Date: 2026-07-02
Executor: Codex
Task: `V221-002` / GitHub issue `#706`
Status: LOCAL VALIDATION PASSED

## Summary

The Trader Terminal Workbench runtime now treats v22 operation/control boundary
fields as required-false runtime contract fields. A field that is missing or
set to `true` is no longer a soft unknown state; it drives fail-closed runtime
health and diagnostic evidence.

Plain Chinese summary: v0.22.1 把 Workbench 禁用边界变成 runtime 硬边界。
`new_submit_capability`、dashboard submit/replace/amend/flatten 控件、订单票据、
manual operation、automatic repair/action、production mutation 等字段必须显式为
`false`。缺失或 `true` 都会让 runtime health/readiness fail closed。

## Runtime Boundary

```text
explicit_false_required = true
missing_required_boundary = fail_closed
true_required_boundary = fail_closed
new_submit_capability = required_false
dashboard_submit_controls_enabled = required_false
trader_terminal_order_ticket_enabled = required_false
manual_operation_submit_allowed = required_false
production_order_submission_allowed = required_false
production_order_mutation_allowed = required_false
automatic_operation_action_allowed = required_false
product_grade_trading_terminal_claim = required_false
```

## Behavior Impact

- Runtime health is stricter for incomplete v22 Workbench artifacts.
- Missing operation boundary fields now produce `<field>_missing` diagnostics.
- True operation boundary fields continue to produce `<field>_true` diagnostics.
- Explicit false artifacts remain healthy and ready.
- No real submit/cancel/retry/replace/amend/flatten action is enabled.

## Validation

```text
cargo fmt --all -- --check = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS, fast smoke only
cargo test -p nautilus-cli trader_terminal_v221_required_false_boundaries_accept_explicit_false --lib -- --nocapture = PASS
cargo test -p nautilus-cli trader_terminal_v221_missing_required_false_boundaries_fail_closed --lib -- --nocapture = PASS
cargo test -p nautilus-cli trader_terminal_v220_forbidden_controls_fail_closed_individually --lib -- --nocapture = PASS
cargo test -p nautilus-cli trader_terminal_ --lib -- --nocapture = PASS, 28 tests
cargo test -p nautilus-cli --lib = PASS, 475 tests
```
