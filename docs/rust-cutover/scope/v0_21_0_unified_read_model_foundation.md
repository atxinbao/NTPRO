# v0.21.0 Unified Read Model Foundation Scope

Date: 2026-06-30
Executor: Codex
Milestone: `v0.21.0`
Task: `V210-000`
GitHub issue: `#651`
Status: GO SCOPE DECISION

## Summary

`v0.21.0` may start its scoped unified read model foundation now that the
`v0.20.1` hardening patch is closed and published. This is a go decision for
read-only account, position, order, fill, and risk projection work. It is not a
go decision for new submit capability, Dashboard order controls, strategy-driven
live trading, or a product-grade trading terminal claim.

Plain Chinese summary: v0.21 可以开始做交易员视角的数据底座，也就是 account /
position / order / fill / risk 的统一 read model。它只能读取、归一化、展示和验证
已有证据链，不允许新增下单能力，不允许 Dashboard 出现下单、审批、撤单、重试控件，
也不能把系统宣传成产品级实盘交易终端。

## v0.20.1 Entry Gate

The v0.21 implementation line is executable because all V201 issues are closed
and the `ntpro-rust-only-v0.20.1` release evidence is published.

| Issue | Task | State | Closed at |
| --- | --- | --- | --- |
| #644 | V201-001 v0.20.0 release closeout and publication evidence backfill | CLOSED | 2026-06-30T08:25:19Z |
| #645 | V201-002 V20 provenance hardening across tests fixtures and golden traces | CLOSED | 2026-06-30T09:30:08Z |
| #650 | V201-003 Durable single-shot attempt ledger and atomic approval consumption | CLOSED | 2026-06-30T10:33:41Z |
| #646 | V201-004 Pre-submit notional consistency hardening | CLOSED | 2026-06-30T11:28:46Z |
| #647 | V201-005 Adapter source and readback provenance labeling | CLOSED | 2026-06-30T12:33:02Z |
| #648 | V201-006 Dashboard diagnostics hardening for foundation boundaries | CLOSED | 2026-06-30T13:35:36Z |
| #649 | V201-007 v0.20.1 release gates and dependency proof | CLOSED | 2026-06-30T16:08:16Z |

Release evidence:

```text
release tag = ntpro-rust-only-v0.20.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.20.1
release commit = d1eb07f9d39c1d122a6e957eaa5226e788dd4779
published at = 2026-06-30T14:39:21Z
hosted release workflow = https://github.com/atxinbao/NTPRO/actions/runs/28452719493
hosted release workflow conclusion = success
hosted release workflow jobs = 54/54 success
v0.20.1 milestone = closed, open_issues=0, closed_issues=7
```

## Go / No-Go Verdict

```text
go_for_v210_planning = true
go_for_scoped_v210_implementation_after_this_pr_merges = true
go_for_unified_read_model_foundation = true
go_for_read_only_trader_terminal_foundation_dashboard = true
go_for_new_submit_capability = false
go_for_dashboard_order_controls = false
go_for_retry_replace_amend_flatten = false
go_for_strategy_driven_live_trading = false
go_for_product_grade_live_trading_terminal = false
```

## Allowed Capability Boundary

Allowed only through later V210 tasks and their evidence:

```text
capability = Unified Read Model Foundation
account_snapshot_read_model = allowed
position_read_model = allowed
order_lifecycle_read_model = allowed
fill_execution_read_model = allowed
risk_state_projection = allowed
read_only_trader_terminal_foundation_dashboard = allowed
golden_traces_and_strict_provenance = required
source_provenance_labels = required
stale_or_mixed_evidence_diagnostics = required
```

The read model must preserve the production order lifecycle foundation
boundaries:

```text
submit_path = existing v0.20 evidence only
new_submit_endpoint = forbidden
new_order_control = forbidden
dashboard_mutation_control = forbidden
retry_or_remediation = forbidden
adapter_behavior_change_without_fixture = forbidden
runtime_semantic_change_without_golden_trace = forbidden
```

## Default Prohibited Capability

```text
new submit capability
Dashboard order button
Dashboard approval button
Dashboard cancel button
Dashboard retry button
retry
replace
amend
flatten
automatic remediation
strategy-driven live trading
multi-account production execution
multi-venue production execution
product-grade live trading terminal claim
unattended production execution platform claim
```

## Required V210 Sequence

```text
1. V210-000 v0.21 scope decision and dependency gate
2. V210-001 Unified read model contract and schema
3. V210-002 Account snapshot read model
4. V210-003 Position read model and risk projection inputs
5. V210-004 Order lifecycle read model from submit readback cancel and audit evidence
6. V210-005 Fill and execution read model with dedupe and reconciliation
7. V210-006 Unified risk state projection
8. V210-007 Trader Terminal read-only foundation dashboard
9. V210-008 v0.21 golden traces release gates and strict provenance
```

No later V210 task may claim release readiness until the preceding boundary
evidence it depends on has merged.

## Validation Requirements

Every later V210 implementation PR must provide local evidence for:

```text
read_only_boundary
source_provenance
freshness_or_staleness_diagnostics
golden_trace_or_fixture_coverage_for_behavior_changes
no_new_submit_capability
no_dashboard_order_controls
no_retry_replace_amend_flatten
no_product_grade_terminal_claim
```

## Non-Goals

This scope decision does not implement runtime code, adapter behavior,
execution routing, submission, cancellation, Dashboard controls, release tags,
or golden traces. Those remain assigned to later V210 issues.
