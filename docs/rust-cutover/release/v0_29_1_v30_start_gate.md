# v0.29.1 to v0.30.0 Start Gate

Date: 2026-07-11
Executor: Codex
Task: `V291-005` / GitHub issue `#967`
Milestone: `v0.29.1`
Status: BLOCKED UNTIL V291 RELEASE EVIDENCE PUBLISHED

## Gate Contract

```text
v0.30.0 backend production go-live candidate = hard-blocked
v0.30.0 start gate = blocked_until_v291_release_evidence_published
v0.29.0 publication evidence alone unlocks v0.30.0 = false
v0.29.1 release evidence required before v0.30.0 intake = true
v0.29.1 exact issue set required = #963-#968
v0.29.1 release closeout proof required = true
v0.29.1 hosted release gate success required = true
v0.29.1 strict provenance required = true
v0.29.1 publication after hosted gate required = true
generated publication evidence sole proof allowed = false
```

v0.29.0 published backend production readiness evidence, but it does not unlock
the v0.30.0 go-live candidate track by itself. v0.30.0 intake stays fail-closed
until the full v0.29.1 issue set is closed and the v0.29.1 release evidence is
source-controlled, published, and reconstructable from GitHub remote state.

## Required v0.29.1 Scope

```text
V291 exact issue set = #963-#968
V291-001 release closeout evidence = required
V291-002 publish-after-gate current binding = required
V291-003 stale V290 evidence cleanup = required
V291-004 post-publication closeout gate = required
V291-005 v30 start gate hardening = required
V291-006 v29.1 release gates and strict provenance = required
V291 open issue count must be 0 before v0.30.0 start = true
V291 release evidence missing blocks v0.30.0 start = true
```

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
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
```

## Verification

```text
requirements = docs/rust-cutover/release/v0_29_1_v30_start_gate_requirements.json
gate = scripts/ai/verify_v29_1_v30_start_gate.sh
current expected status = blocked
blocked reason = V291 issue #968 open or v0.29.1 release evidence missing
```
