# v0.23.0 Multi-Strategy Supervisor Isolation

Date: 2026-07-03
Executor: Codex
Task: `V230-003`
GitHub issue: `#714`
Milestone: `v0.23.0`
Status: LOCAL VALIDATION PASSED

## Summary

V230-003 adds executable Rust replay coverage for multi-strategy supervisor
identity and isolation. It covers isolated strategy partitions, cross-strategy
component mismatch fail-closed behavior, and unknown strategy identity
fail-closed behavior.

Plain Chinese summary: 本任务证明多策略 supervisor 读路径必须显式携带
`strategy_key` 和 `isolation_scope_key`。两个策略的 supervisor/runtime/risk/events/
audit/provenance 状态不能串线；发现跨策略 mismatch 或 unknown strategy identity 时
必须 fail closed。真实生产下单、策略驱动生产执行、venue node lifecycle 和 Dashboard
操作控件仍不在本任务范围内。

## Contract Mapping

```text
contract = docs/rust-cutover/release/v0_23_0_multi_node_isolation_contract.md
Identity Model = strategy_key, account_key, venue_node_key, isolation_scope_key
Strategy Boundary = strategy supervisor/runtime/risk/events/audit partitions preserve strategy_key
Future Owner-Approved Control Paths = owner approval remains per isolation_scope_key
Logs And Evidence Boundary = audit/provenance components require source_provenance and strategy identity
```

## Executable Replay Cases

```text
read_model.strategy_supervisor.isolated_strategies.001 = PASS path, two strategy scopes remain isolated
read_model.strategy_supervisor.cross_strategy_mismatch.001 = FAIL-CLOSED path, component strategy_key mismatch
read_model.strategy_supervisor.missing_strategy_key.001 = FAIL-CLOSED path, unknown strategy identity
```

## Boundary

```text
runtime_behavior_change = read_model_projection_test_harness_only
adapter_behavior_change = false
dashboard_behavior_change = false
release_publication = false
new_submit_capability = false
strategy_driven_production_execution_allowed = false
production_order_submission_allowed = false
dashboard_operation_controls_enabled = false
venue_node_lifecycle_started = false
```

## Validation

Validation is recorded in `docs/rust-cutover/evidence/V230-003.md` and
`verification.md`.
