# v0.23.0 Multi-Node Orchestration Control-Plane Gating

Date: 2026-07-03
Executor: Codex
Task: `V230-005`
GitHub issue: `#716`
Milestone: `v0.23.0`
Status: LOCAL VALIDATION PASSED

## Summary

V230-005 adds executable Rust replay coverage for multi-node orchestration and
control-plane gating. It validates that owner approval, risk gate, audit gate,
and approval consumption remain scoped to a single `isolation_scope_key`.

Plain Chinese summary: 本任务不是打开真实交易控制，而是把未来 control-plane 的硬边界先
做成可执行 replay。有效 scoped intent 只能保持 read-only gate；跨
account/strategy/venue node 的 routing mismatch、shared approval consumption、缺失
`isolation_scope_key` 都会 fail closed。submit/cancel/retry/replace/amend/flatten、
automatic cancel、automatic remediation、Dashboard 操作控件仍全部禁用。

## Contract Traceability

```text
V230-001 Identity Model = satisfied by account_key, strategy_key, venue_node_key, and isolation_scope_key replay assertions
V230-001 Future Owner-Approved Control Paths = satisfied by owner_approval_gate_required, risk_gate_required, audit_gate_required, and single-scope approval consumption assertions
V230-001 Isolation Boundaries = satisfied by cross-scope account/strategy/venue node route mismatch fail-closed fixture
V230-001 Logs And Evidence Boundary = satisfied by per-component source_provenance and release replay scope entries
```

## Control-Plane Boundary

```text
owner_approval_gate_required = true
risk_gate_required = true
audit_gate_required = true
approval_consumption = single_scope_only
shared_approval_consumption = forbidden
implicit_cross_account_operation_allowed = false
implicit_cross_strategy_operation_allowed = false
implicit_cross_venue_operation_allowed = false
implicit_cross_node_operation_allowed = false
automatic_cancel_allowed = false
automatic_remediation_allowed = false
ungated_submit_cancel_retry_replace_amend_flatten_allowed = false
dashboard_operation_controls_enabled = false
production_order_submission_allowed = false
```

## Executable Replay Coverage

```text
read_model.orchestration_control_plane.scoped_intents_ready.001 = PASS path, two scoped intents remain gated and read-only
read_model.orchestration_control_plane.cross_scope_route_mismatch.001 = FAIL-CLOSED path, routing points at a different account, strategy, venue node, and scope
read_model.orchestration_control_plane.shared_approval_blocked.001 = FAIL-CLOSED path, one approval reference is consumed by two isolation scopes
read_model.orchestration_control_plane.missing_scope_key.001 = FAIL-CLOSED path, control intent lacks isolation_scope_key
```

## Non-Goals Confirmed

```text
runtime_behavior_change = read_model_projection_test_harness_only
dashboard_behavior_change = false
release_publication = false
production_submit_enabled = false
production_order_mutation_enabled = false
automatic_cancel_enabled = false
automatic_remediation_enabled = false
```

## Evidence

Validation is recorded in `docs/rust-cutover/evidence/V230-005.md` and
`verification.md`. The release replay manifest records all four cases as
`executable_replay` with `release_decision = included_in_final_replay_scope`.
