# NTPRO v0.22.0 Readiness Report

Date: 2026-07-02
Executor: Codex
Milestone: `ntpro-rust-only-v0.22.0`
Status: RELEASED

## Summary

v0.22.0 is ready as the Trader Terminal Workbench release. It builds on the
published `ntpro-rust-only-v0.21.1` hardening patch and closes the V220 issue
chain from scope decision through release gate and strict provenance.

Plain Chinese summary: v0.22.0 可以作为 Trader Terminal workbench 发布。它完成了
只读工作台外壳、账户/持仓、订单/成交、风控/告警/审计/provenance、人工操作入口合同、
运行时降级和边界测试，以及 release gates / strict provenance。它仍然不是产品级实盘
交易终端，不开放无门禁 submit/cancel/retry/replace/amend/flatten。

## Required Evidence

```text
V220-000 evidence = docs/rust-cutover/evidence/V220-000.md
V220-001 evidence = docs/rust-cutover/evidence/V220-001.md
V220-002 evidence = docs/rust-cutover/evidence/V220-002.md
V220-003 evidence = docs/rust-cutover/evidence/V220-003.md
V220-004 evidence = docs/rust-cutover/evidence/V220-004.md
V220-005 evidence = docs/rust-cutover/evidence/V220-005.md
V220-006 evidence = docs/rust-cutover/evidence/V220-006.md
V220-007 evidence = docs/rust-cutover/evidence/V220-007.md
```

## Workbench Evidence Inputs

```text
scope decision = docs/rust-cutover/scope/v0_22_0_trader_terminal_workbench_scope.md
workbench shell = docs/rust-cutover/release/v0_22_0_trader_terminal_workbench_shell.md
account/position panels = docs/rust-cutover/release/v0_22_0_account_position_workbench_panels.md
order/fill panels = docs/rust-cutover/release/v0_22_0_order_fill_workbench_panels.md
risk/alerts/audit/provenance panels = docs/rust-cutover/release/v0_22_0_risk_alert_audit_provenance_workbench_panels.md
gated manual operation entry contract = docs/rust-cutover/release/v0_22_0_gated_manual_operation_entry_contract.md
runtime degradation boundary tests = docs/rust-cutover/release/v0_22_0_runtime_degradation_boundary_tests.md
```

## Gates

```text
v21.1 base release = required
v22 runtime boundary tests = required
v22 release gates = required
v22 strict provenance = required
release publication guard = required after GitHub Release publication
release surface current guard = required
```

## Issue Closeout

```text
#683 V220-000 = required closed for release
#684 V220-001 = required closed for release
#685 V220-002 = required closed for release
#686 V220-003 = required closed for release
#687 V220-004 = required closed for release
#688 V220-005 = required closed for release
#689 V220-006 = required closed for release
#690 V220-007 = required closed for release
```

## Boundary

```text
trader_terminal_workbench = true
read_only_first = true
gated_operation_boundary = true
owner_approval_gate_required = true
risk_gate_required = true
audit_gate_required = true
new_submit_capability = false
production_order_mutation_allowed = false
ungated_submit_allowed = false
ungated_cancel_allowed = false
ungated_retry_allowed = false
ungated_replace_allowed = false
ungated_amend_allowed = false
ungated_flatten_allowed = false
automatic_cancel_allowed = false
automatic_remediation_allowed = false
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
```

## Release Decision

The v0.22.0 release may be published only after this source-tree evidence is
merged, the GitHub issue #690 closes, `ntpro-rust-only-v0.22.0` is created from
`main`, and the hosted release gate passes on that tag.
