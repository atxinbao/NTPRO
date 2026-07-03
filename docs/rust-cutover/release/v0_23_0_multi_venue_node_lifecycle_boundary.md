# v0.23.0 Multi-Venue Node Lifecycle Boundary

Date: 2026-07-03
Executor: Codex
Task: `V230-004`
GitHub issue: `#715`
Milestone: `v0.23.0`
Status: LOCAL VALIDATION PASSED

## Summary

V230-004 adds executable Rust replay coverage for multi-venue node registry and
lifecycle identity. It covers isolated venue node partitions, cross-node
component mismatch fail-closed behavior, and unknown venue node identity
fail-closed behavior.

Plain Chinese summary: 本任务证明多 venue node 读路径必须显式携带 `venue_node_key`、
`adapter_instance_id` 和 `isolation_scope_key`。两个 node 的
registry/lifecycle/adapter/connection/risk/audit/provenance 状态不能串线；发现跨
node mismatch 或 unknown venue node identity 时必须 fail closed。凭证处理扩展、真实
submit、order mutation、Dashboard 操作控件和跨 node 编排仍不在本任务范围内。

## Node Registry Contract Shape

Each venue node partition must include a `registry_entry` object with:

```text
venue_node_key = stable owner-configured venue node identity
venue = exchange or venue label, not sufficient as identity by itself
environment = sandbox/testnet/production label, not sufficient as identity by itself
adapter_instance_id = stable adapter instance identity for this node
source_provenance = registry evidence source for audit/replay
```

Registry failure rules:

```text
missing venue_node_key = fail_closed
missing adapter_instance_id = degraded_unavailable_or_fail_closed
registry venue_node_key mismatch = fail_closed
missing source_provenance = degraded_unavailable_or_fail_closed
credential alias fallback = forbidden
venue-only fallback = forbidden
process-id fallback = forbidden
```

## Contract Mapping

```text
contract = docs/rust-cutover/release/v0_23_0_multi_node_isolation_contract.md
Identity Model = venue_node_key, account_key, strategy_key, isolation_scope_key
Venue Node Boundary = lifecycle, adapter, connection, risk, audit, provenance partitions preserve venue_node_key
Adapter Boundary = adapter_instance_id and source_provenance required in registry evidence
Logs And Evidence Boundary = audit/provenance components require source_provenance and venue node identity
```

## Executable Replay Cases

```text
read_model.venue_node_lifecycle.isolated_nodes.001 = PASS path, two venue nodes remain isolated
read_model.venue_node_lifecycle.cross_node_mismatch.001 = FAIL-CLOSED path, component venue_node_key mismatch
read_model.venue_node_lifecycle.missing_venue_node_key.001 = FAIL-CLOSED path, unknown venue node identity
```

## Boundary

```text
runtime_behavior_change = read_model_projection_test_harness_only
adapter_behavior_change = false
dashboard_behavior_change = false
release_publication = false
credential_handling_expansion_allowed = false
production_order_mutation_allowed = false
production_order_submission_allowed = false
dashboard_operation_controls_enabled = false
cross_venue_implicit_operation_allowed = false
lifecycle_control_requires_owner_approval = true
```

## Validation

Validation is recorded in `docs/rust-cutover/evidence/V230-004.md` and
`verification.md`.
