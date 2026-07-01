# NTPRO v0.21.0 Readiness Report

Date: 2026-07-01
Executor: Codex
Milestone: `ntpro-rust-only-v0.21.0`
Status: RELEASED

## Summary

v0.21.0 is released as the unified read model foundation for account, position,
order, fill, risk, and Trader Terminal read-only Dashboard evidence. The
milestone closes the V210 issue chain without adding submit, cancel, retry,
remediation, strategy-driven execution, or Dashboard operation controls.

## Publication Closeout

```text
GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.21.0
release tag = ntpro-rust-only-v0.21.0
release name = NTPRO Rust-only v0.21.0
published at = 2026-07-01T11:06:06Z
tag commit = 7e1cb46d692974bb5ef1398967c0927dd51c8091
tag tree = d33fe810aab4c0e0f743340c9dac6bc993a9428a
hosted release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28513012766
hosted release gate conclusion = success
hosted release gate completed at = 2026-07-01T12:35:48Z
```

## Required Evidence

```text
V210-000 evidence = docs/rust-cutover/evidence/V210-000.md
V210-001 evidence = docs/rust-cutover/evidence/V210-001.md
V210-002 evidence = docs/rust-cutover/evidence/V210-002.md
V210-003 evidence = docs/rust-cutover/evidence/V210-003.md
V210-004 evidence = docs/rust-cutover/evidence/V210-004.md
V210-005 evidence = docs/rust-cutover/evidence/V210-005.md
V210-006 evidence = docs/rust-cutover/evidence/V210-006.md
V210-007 evidence = docs/rust-cutover/evidence/V210-007.md
V210-008 evidence = docs/rust-cutover/evidence/V210-008.md
```

## Gates

```text
v21 component gates = required
v21 release gates = required
v21 strict provenance = required
release publication guard = required after GitHub Release publication
release surface current guard = required
golden trace release scope = required
```

## Golden Trace Scope

```text
read_model_contract_schema.jsonl = required
read_model_account_snapshot_schema.jsonl = required
read_model_position_schema.jsonl = required
read_model_order_lifecycle_schema.jsonl = required
read_model_fill_execution_schema.jsonl = required
read_model_risk_state_schema.jsonl = required
read_model_dashboard_schema.jsonl = required
release scope manifest cases = 83
read model schema-only cases = 32
```

## Dependency Proof

```text
base release = ntpro-rust-only-v0.20.1
v0.20.1 release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.20.1
v0.20.1 release gate run = 28452719493
V201 issues #644-#650 = closed
v0.21.0 milestone = #651-#659
v0.21.0 issue closeout = #651-#659 closed
v0.21.0 milestone #8 = closed at 2026-07-01T14:38:16Z
```

## v0.21.1 And v0.22.0 Dependency

```text
v0.21.1 milestone = #677-#682
v0.21.1 status = required hardening and closeout patch
v0.22.0 milestone = #683-#690
v0.22.0 status = blocked until all V211 issues close and v0.21.1 release evidence is published
```

## Boundary

```text
unified_read_model_foundation = true
read_only_foundation = true
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
implicit_retry_allowed = false
automatic_cancel_allowed = false
automatic_remediation_allowed = false
retry_replace_amend_flatten_allowed = false
strategy_driven_production_execution_allowed = false
multi_account_execution_allowed = false
multi_venue_execution_allowed = false
dashboard_order_controls_enabled = false
dashboard_approval_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_retry_controls_enabled = false
dashboard_submit_controls_enabled = false
dashboard_replace_controls_enabled = false
dashboard_amend_controls_enabled = false
dashboard_flatten_controls_enabled = false
trader_terminal_order_ticket_enabled = false
product_grade_trading_terminal_claim = false
executable_read_model_runtime_claim = false
trader_terminal_workbench_claim = false
```
