# v0.19.0 Actual Cancel Safety Contract

Date: 2026-06-27
Executor: Codex
Milestone: `v0.19.0`
Task: `V190-002`
Status: CONTRACT

## Summary

This document defines the v0.19.0 actual cancel safety contract. It allows only
a future owner-approved, single-shot, manual-only cancel recovery path: one
approval, one order, one venue, and one execution attempt. This contract does
not implement the cancel executor, does not add adapter runtime behavior, does
not add Dashboard operation controls, and does not introduce a production order
submit lifecycle.

Plain Chinese summary: 这份文档只定义 v0.19 真撤单必须遵守的安全契约。大白话：以后如果
真的撤单，也只能由 owner 手工批准，一次批准只对应一个订单、一个 venue、一次发送；不能自动撤单、
不能批量撤单、不能复用批准、不能从 Dashboard 点按钮执行，也不能顺手引入生产下单生命周期。

## Start Condition

The v0.19.0 actual-cancel line may proceed only after the v0.18.1 release
surface and provenance closeout is complete and V190-001 has opened the
readiness gate.

```text
blocked_by = V190-001
blocked_by_issue = #577
blocked_by_status_required = closed_completed
release_baseline_required = ntpro-rust-only-v0.18.1
actual_cancel_contract_status = documented_only
runtime_cancel_executor_included = false
adapter_behavior_change_included = false
dashboard_operation_controls_included = false
```

## Contract Constants

Every later v0.19 actual-cancel artifact and implementation gate must preserve
these values unless a later scoped issue explicitly tightens the contract:

```text
schema_version = ntpro.v190_actual_cancel_safety_contract.v1
capability = Owner-Approved Single-Shot Actual Cancel
execution_mode = owner_approved_single_shot_manual_only
approval_scope = one_order_one_venue_one_attempt
owner_approval_required = true
owner_approval_source = owner_manual_action
approval_reusable = false
approval_consumed_before_send = true
approval_consumed_after_send = true
single_order_required = true
single_venue_required = true
single_execution_attempt_required = true
manual_only = true
actual_cancel_send_allowed = gated_by_required_artifacts
production_order_submit_lifecycle = out_of_scope
automatic_cancel_allowed = false
automatic_remediation_allowed = false
bulk_cancel_allowed = false
multi_account_cancel_allowed = false
multi_strategy_cancel_allowed = false
multi_venue_cancel_allowed = false
retry_allowed = false
replace_allowed = false
amend_allowed = false
flatten_allowed = false
dashboard_cancel_controls_enabled = false
dashboard_auto_approval_allowed = false
```

## Required Artifacts Before Any Send

An actual cancel send is forbidden unless every artifact below is present,
fresh, in-scope, and bound to the same order, account, symbol, and venue:

| Artifact | Required proof |
| --- | --- |
| Owner approval | Owner manually approved exactly one cancel candidate; approval is unexpired, unused, and non-reusable. |
| Risk gate | Orphan/cancel risk gate passed for the same order lineage and confirms new orders are blocked. |
| Order evidence | Known order identifier, client order identifier when available, account label, symbol, and venue match the candidate. |
| Release manifest | Current release manifest and provenance identify the source, gates, and actual-cancel capability boundary. |
| Adapter capability evidence | Adapter proves it supports only the scoped single-order cancel path for the selected venue and records no bulk or automatic capability. |

The artifacts must be linked by stable identifiers:

```text
actual_cancel_session_id = required
approval_id = required
order_lineage_id = required
venue = required
symbol = required
account_label = required
release_manifest_ref = required
adapter_capability_ref = required
```

## Fail-Closed Semantics

The implementation must stop before network send and record one or more failure
reasons if any required proof is missing, stale, reused, mismatched, or
forbidden by this contract.

| Condition | Required result | Failure reason |
| --- | --- | --- |
| Owner approval missing | Do not execute cancel. | `missing_owner_approval` |
| Owner approval expired | Do not execute cancel. | `owner_approval_expired` |
| Owner approval already consumed | Do not execute cancel. | `owner_approval_reused` |
| Owner approval points to a different order, account, symbol, or venue | Do not execute cancel. | `owner_approval_scope_mismatch` |
| Risk gate missing or failed | Do not execute cancel. | `risk_gate_not_passed` |
| Order evidence missing or mismatched | Do not execute cancel. | `order_identity_mismatch` |
| Venue evidence missing or mismatched | Do not execute cancel. | `venue_mismatch` |
| Release manifest missing, stale, or not tied to the source under execution | Do not execute cancel. | `release_manifest_not_current` |
| Adapter capability evidence missing | Do not execute cancel. | `adapter_capability_missing` |
| Automatic cancel requested | Do not execute cancel. | `automatic_cancel_requested` |
| Bulk cancel or cancel-all requested | Do not execute cancel. | `bulk_cancel_requested` |
| Multi-account, multi-strategy, or multi-venue cancel requested | Do not execute cancel. | `scope_expansion_requested` |
| Retry, replace, amend, flatten, or remediation requested | Do not execute cancel. | `retry_or_repair_requested` |
| Dashboard operation control or auto-approval requested | Do not execute cancel. | `dashboard_operation_requested` |

## Future Execution Boundary

A later scoped implementation may send exactly one actual cancel request only
after all required artifacts pass the gate:

```text
max_cancel_requests_sent = 1
allowed_attempts = 1
allowed_order_count = 1
allowed_venue_count = 1
allowed_account_count = 1
approval_reuse_after_attempt = forbidden
retry_after_failure = forbidden
fallback_to_bulk_cancel = forbidden
fallback_to_cancel_all = forbidden
fallback_to_submit_order = forbidden
dashboard_triggered_execution = forbidden
```

The future implementation must consume the approval before send and mark it
consumed after the attempt outcome is recorded. If the process crashes between
approval consumption and outcome recording, later recovery must treat the
approval as consumed and must not send a second cancel request.

## Forbidden Expansions

The v0.19.0 safety contract does not authorize:

```text
production order submit lifecycle
automatic cancel
automatic orphan cleanup
bulk cancel
cancel all open orders
multi-account cancel recovery
multi-strategy cancel recovery
multi-venue cancel recovery
retry / replace / amend / flatten
Dashboard cancel button
Dashboard owner approval button
Dashboard credential input
strategy-driven cancel
general production trading platform readiness
real-funds proof in CI
```

## Later Issue Handoff

Later V190 issues must reference this file as the execution boundary:

```text
V190-003 owner approval execution lifecycle = must enforce non-reusable owner approval
V190-004 single-shot cancel command implementation = must enforce one order / one venue / one attempt
V190-005 cancel executor adapter boundary = must prove adapter capability evidence before send
V190-006 post-cancel readback reconciliation = must read back terminal state without retrying cancel
V190-007 failure and partial-success evidence model = must record fail-closed and consumed-approval states
V190-008 Dashboard read-only actual cancel audit view = must not add Dashboard operation controls
V190-009 golden trace and fixture coverage = must cover missing/expired/reused/mismatched approval failures
V190-010 release gates and evidence = must validate this contract before release
```

## Validation Requirement

Once implementation exists, contract tests or unit tests must cover at least:

```text
missing_owner_approval
owner_approval_expired
owner_approval_reused
owner_approval_scope_mismatch
order_identity_mismatch
venue_mismatch
adapter_capability_missing
release_manifest_not_current
bulk_cancel_requested
retry_or_repair_requested
dashboard_operation_requested
```

For V190-002 itself, validation is documentation and boundary validation only.
No runtime cancel path, adapter call, network request, or Dashboard operation
surface is added.
