# v0.22.0 Trader Terminal Workbench Scope

Date: 2026-07-01
Executor: Codex
Milestone: `v0.22.0`
Task: `V220-000`
GitHub issue: `#683`
Status: GO SCOPE DECISION

## Summary

`v0.22.0` may start its scoped Trader Terminal workbench line now that the full
`v0.21.1` hardening set is closed and the
`ntpro-rust-only-v0.21.1` release evidence is published. This is a go decision
for read-only first account, position, order, fill, risk, alert, audit, and
provenance drill-down work. It is not a go decision for ungated order
submission, cancel, retry, replace, amend, flatten, strategy-driven live
trading, or a product-grade live trading terminal claim.

Plain Chinese summary: v0.22 可以开始做 Trader Terminal workbench，但第一阶段仍然是
read-only first。它可以把 account、position、order、fill、risk、alert、audit、
provenance 做成交易员工作台视图；任何真实操作入口都必须先经过 owner approval、
risk gate 和 audit gate。不能做无门禁 submit/cancel/retry/replace/amend/flatten，
也不能宣称已经是产品级实盘交易终端。

## v0.21.1 Entry Gate

The v0.22 implementation line is executable because all V211 issues are closed,
the `ntpro-rust-only-v0.21.1` GitHub Release is published, and the hosted
release-tag gate completed successfully.

| Issue | Task | State | Closed at |
| --- | --- | --- | --- |
| #677 | V211-001 v0.21.0 release closeout and milestone evidence backfill | CLOSED | 2026-07-01T15:09:57Z |
| #678 | V211-002 Read model health_status semantics split | CLOSED | 2026-07-01T15:58:42Z |
| #679 | V211-003 Executable read-model projection replay | CLOSED | 2026-07-01T16:51:45Z |
| #680 | V211-004 Tighten unified read model JSON schema boundary source and redaction rules | CLOSED | 2026-07-01T17:43:44Z |
| #681 | V211-005 Trader Terminal read-only runtime bridge to read model artifacts | CLOSED | 2026-07-01T18:48:19Z |
| #682 | V211-006 v0.21.1 release gates strict provenance and v0.22 dependency proof | CLOSED | 2026-07-01T19:50:09Z |

Release evidence:

```text
release tag = ntpro-rust-only-v0.21.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.21.1
release commit = 016bbb32e6f6a343be1e81bf2ad2e270c11e02b0
published at = 2026-07-01T19:50:54Z
hosted release workflow = https://github.com/atxinbao/NTPRO/actions/runs/28543669704
hosted release workflow conclusion = success
hosted release workflow jobs = 58/58 success
v0.21.1 milestone = open, open_issues=0, closed_issues=6
v0.22.0 milestone = open, open_issues=8, closed_issues=0
v0.22.0 issue unblock comments = #683 through #690
```

## Go / No-Go Verdict

```text
go_for_v220_planning = true
go_for_scoped_v220_implementation_after_this_pr_merges = true
go_for_trader_terminal_workbench = true
go_for_read_only_first_workbench = true
go_for_account_position_order_fill_risk_alert_audit_provenance_drill_down = true
go_for_gated_manual_operation_entry_design = true
go_for_ungated_submit = false
go_for_ungated_cancel = false
go_for_ungated_retry = false
go_for_ungated_replace = false
go_for_ungated_amend = false
go_for_ungated_flatten = false
go_for_strategy_driven_live_trading = false
go_for_product_grade_live_trading_terminal = false
```

## Allowed Capability Boundary

Allowed only through later V220 tasks and their evidence:

```text
capability = Trader Terminal Workbench
read_only_first = required
account_workbench_panel = allowed
position_workbench_panel = allowed
order_workbench_panel = allowed
fill_workbench_panel = allowed
risk_alerts_panel = allowed
audit_panel = allowed
provenance_drill_down = allowed
manual_operation_entry_design = allowed
owner_approval_gate = required_for_any_future_real_operation
risk_gate = required_for_any_future_real_operation
audit_gate = required_for_any_future_real_operation
runtime_degradation_tests = required
strict_provenance = required
```

The workbench must preserve the v0.21.1 hardening patch boundaries:

```text
submit_path = existing evidence only
new_order_submit_endpoint = forbidden
ungated_order_control = forbidden
dashboard_mutation_control = forbidden
implicit_retry = forbidden
automatic_cancel = forbidden
automatic_remediation = forbidden
retry_replace_amend_flatten = forbidden
adapter_behavior_change_without_fixture = forbidden
runtime_semantic_change_without_golden_trace = forbidden
product_grade_trading_terminal_claim = forbidden
```

## Default Prohibited Capability

```text
ungated submit
ungated cancel
ungated retry
ungated replace
ungated amend
ungated flatten
automatic remediation
strategy-driven live trading
multi-account production execution expansion
multi-venue production execution expansion
product-grade live trading terminal claim
unattended production execution platform claim
order ticket that can mutate live state without gates
```

## Required V220 Sequence

```text
1. V220-000 v0.22 scope decision and v0.21.1 dependency gate
2. V220-001 Trader Terminal read-only workbench shell and navigation
3. V220-002 Account and position workbench panels
4. V220-003 Order and fill workbench panels
5. V220-004 Risk alerts audit and provenance drill-down panels
6. V220-005 Gated manual operation entry contract
7. V220-006 Trader Terminal runtime degradation and boundary tests
8. V220-007 v0.22 release gates strict provenance and workbench evidence
```

No later V220 task may claim release readiness until the preceding boundary
evidence it depends on has merged.

## Validation Requirements

Every later V220 implementation PR must provide local evidence for:

```text
read_only_first_boundary
source_provenance
freshness_or_staleness_diagnostics
degraded_or_missing_evidence_behavior
owner_approval_gate_for_any_future_real_operation
risk_gate_for_any_future_real_operation
audit_gate_for_any_future_real_operation
no_ungated_submit_cancel_retry_replace_amend_flatten
no_strategy_driven_live_trading
no_product_grade_terminal_claim
```

## Non-Goals

This scope decision does not implement runtime code, adapter behavior,
execution routing, submission, cancellation, Dashboard controls, release tags,
or golden traces. Those remain assigned to later V220 issues.
