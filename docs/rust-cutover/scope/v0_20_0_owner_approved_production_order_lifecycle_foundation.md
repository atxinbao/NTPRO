# v0.20.0 Owner-Approved Production Order Lifecycle Foundation Scope

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-000`
Status: GO SCOPE DECISION

## Summary

`v0.20.0` may enter the owner-approved production order lifecycle foundation
after the v0.19.1 closeout evidence chain. This is a go decision for bounded
V200 planning and implementation tasks, not a general production trading
platform approval.

Plain Chinese summary: v0.20 可以开始做“有人工批准的一笔生产订单生命周期基础闭环”，但
不是放开实盘交易平台。大白话：后续任务只能围绕一笔订单的 submit、readback、cancel、audit
基础证据走，必须有人批准、必须有风控和回读、必须有审计；不能自动下单，不能批量下单，
不能 retry/改单/flatten，不能让 Dashboard 出现下单或批准按钮。

## v0.19.1 Entry Gate

The v0.20 runtime, adapter, and production-order-lifecycle implementation line
is executable only because all v0.19.1 blockers are closed and closeout evidence
is available:

| Issue | Task | State | Closed at |
| --- | --- | --- | --- |
| #604 | V191-001 v0.19.0 release closeout | CLOSED | 2026-06-28T12:50:56Z |
| #605 | V191-002 v0.18.1 prerequisite release evidence reconciliation | CLOSED | 2026-06-28T13:22:11Z |
| #606 | V191-003 Align current release surface to v0.19.0 | CLOSED | 2026-06-29T00:22:02Z |
| #607 | V191-004 Add v0.19.0 release publication guard | CLOSED | 2026-06-29T08:08:57Z |
| #608 | V191-005 Add v19 strict release provenance | CLOSED | 2026-06-29T08:58:35Z |
| #609 | V191-006 Add V190-004 post-merge review attestation | CLOSED | 2026-06-29T09:33:55Z |
| #610 | V191-007 Harden standalone v19 release gate semantics | CLOSED | 2026-06-29T10:19:08Z |

Closeout evidence:

```text
v0.19.1 readiness report = docs/rust-cutover/release/v0_19_1_readiness_report.md
v0.19.1 release notes = docs/rust-cutover/release/v0_19_1_release_notes.md
v19 strict provenance = docs/rust-cutover/evidence/V191-005.md
V190-004 post-merge review attestation = docs/rust-cutover/evidence/V191-006_actual_cancel_review_attestation.md
standalone v19 gate hardening = docs/rust-cutover/evidence/V191-007.md
v0.19.1 tag = not published
```

The missing v0.19.1 tag does not block this V200 scope decision because the
entry condition is release evidence availability, not tag publication. A later
release task must make an explicit tag and GitHub Release decision if v0.19.1
publication is required.

## Go / No-Go Verdict

```text
go_for_v200_planning = true
go_for_scoped_v200_implementation_after_this_pr_merges = true
go_for_general_production_trading_platform = false
go_for_dashboard_order_controls = false
go_for_automatic_order_placement = false
go_for_multi_order_or_bulk_execution = false
```

## Allowed Minimum Capability Boundary

Allowed only through later V200 tasks and their evidence:

```text
capability = Owner-Approved Production Order Lifecycle Foundation
submit_candidate_builder = allowed
pre_submit_risk_gate = required
owner_approval_lifecycle = required
env_only_signing_material_gate = required
guarded_single_shot_submit_candidate = allowed after required gates
production_submit_response_redaction = required
post_submit_readback_reconciliation = required
failure_and_no_retry_evidence = required
read_only_dashboard_lifecycle_audit = allowed
golden_traces_and_fixtures = required
release_gates_and_strict_provenance = required
```

Minimum execution envelope for any later production submit candidate:

```text
owner_approval_required = true
manual_online_gate_required = true
single_order_required = true
single_venue_required = true
single_account_required = true
single_attempt_required = true
order_type_default = LIMIT
market_order_allowed = false
pre_submit_risk_gate_required = true
adapter_boundary_required = true
readback_required = true
audit_artifact_required = true
raw_secret_persistence_allowed = false
raw_exchange_response_persistence_allowed = false
```

Cancel and audit boundaries:

```text
cancel_scope = owner-approved follow-up or existing v19 actual-cancel boundary only
automatic_cancel_allowed = false
bulk_cancel_allowed = false
second_cancel_allowed = false
dashboard_cancel_controls_enabled = false
dashboard_execution_allowed = false
```

## Default Prohibited Capability

```text
strategy-driven production execution
automatic order placement
multi-order production execution
bulk order
MARKET order unless later explicitly approved
multi-account execution
multi-venue execution
retry
replace
amend
flatten
automatic remediation
Dashboard order button
Dashboard approval button
Dashboard credential input
general production trading platform claim
unbounded production adapter execution
raw secret persistence
raw signed payload persistence
raw exchange response persistence
```

## Required V200 Sequence

```text
1. V200-000 scope decision and go/no-go gate
2. V200-001 production order lifecycle safety contract
3. V200-002 pre-submit risk gate
4. V200-003 owner approval lifecycle for submit
5. V200-004 signing material and env-only gate
6. V200-005 single-shot production order request builder
7. V200-006 guarded single-shot submit candidate
8. V200-007 production submit response redaction
9. V200-008 post-submit readback reconciliation
10. V200-009 failure and no-retry evidence model
11. V200-010 Dashboard read-only production order lifecycle audit
12. V200-011 golden traces and fixture coverage
13. V200-012 v0.20 release gates and strict provenance
```

No later V200 task may claim release readiness until the preceding boundary
evidence it depends on has merged.

## Validation Requirements

Every later V200 implementation PR must provide local evidence for:

```text
owner approval required
manual-online/env-only gates
pre-submit risk gate
adapter boundary
single order / single venue / single account / single attempt
readback required
audit artifact required
no retry / replace / amend / flatten
no automatic remediation
no bulk order
no Dashboard execution controls
redacted response and secret handling
golden trace or fixture coverage where behavior changes
```

## Non-Goals

This scope decision does not implement runtime code, adapter behavior, signing,
HTTP sends, order submission, readback, cancel, Dashboard UI, release gates, or
golden traces. Those remain assigned to later V200 issues.
