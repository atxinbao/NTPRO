# v0.31.0 Readiness Report

Date: 2026-07-14
Executor: Codex
Milestone: `ntpro-rust-only-v0.31.0`
Status: RELEASE GATE READY

## Summary

v0.31.0 is ready for the release tag gate as a Controlled Backend Production
Enablement Candidate Foundation. It remains evidence/gate/read-model work only;
actual backend production go-live and product-grade live trading remain out of
scope.

Plain Chinese summary: v0.31.0 已具备打 tag 前的 release gate 证据。它是 controlled
backend production enablement candidate foundation，不是默认实盘执行版本，也不是
产品级交易终端。

## Scope

V310 final release scope issue count = 11
V310 final release scope evidence count = 11
V310 exact milestone issue set = #1006-#1015 plus #1033
V310 registered corrective-scope exception count = 1
V310 registered corrective-scope exception issues = #1033

## Evidence

- V310-000 evidence = `docs/rust-cutover/evidence/V310-000.md`
- V310-001 evidence = `docs/rust-cutover/evidence/V310-001.md`
- V310-002 evidence = `docs/rust-cutover/evidence/V310-002.md`
- V310-003 evidence = `docs/rust-cutover/evidence/V310-003.md`
- V310-004 evidence = `docs/rust-cutover/evidence/V310-004.md`
- V310-005 evidence = `docs/rust-cutover/evidence/V310-005.md`
- V310-006 evidence = `docs/rust-cutover/evidence/V310-006.md`
- V310-007 evidence = `docs/rust-cutover/evidence/V310-007.md`
- V310-008 evidence = `docs/rust-cutover/evidence/V310-008.md`
- V310-009 evidence = `docs/rust-cutover/evidence/V310-009.md`
- V310-010 evidence = `docs/rust-cutover/evidence/V310-010.md`

#1015 V310-009 = must be closed before v0.31.0 tag gate is accepted
#1033 V310-010 = must be closed before corrected v0.31.0 tag gate is accepted

## Gates

v31 release gates = required
v31 strict provenance = required
v31 intake gate = v0.30.1 publication evidence satisfied; explicit scoped approval still required
source-controlled release manifest = docs/rust-cutover/release/v0_31_0_release_manifest.json
source-controlled v32 handoff = docs/rust-cutover/release/v0_31_0_v32_backend_production_closeout_handoff.md
publication evidence strategy = source_tree_plus_github_remote
generated publication evidence sole proof allowed = false

## Boundary

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
cancel_order_allowed = false
replace_order_allowed = false
amend_order_allowed = false
flatten_position_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
network_attempted = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
automatic_recovery_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
product_grade_trading_terminal_claim = false
product_grade_live_trading_terminal_claim = false
default_production_execution_allowed = false
```

## Next Track

v0.32.0 start gate = blocked until v0.31.0 release evidence and explicit scoped approval

v32 cannot inherit submit, mutation, adapter send, live exchange request, retry
scheduler, automatic remediation, operation/trading controls, backend go-live,
or product-grade live trading claims from v31.
