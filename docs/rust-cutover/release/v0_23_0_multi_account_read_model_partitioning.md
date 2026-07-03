# v0.23.0 Multi-Account Read-Model Partitioning

Date: 2026-07-03
Executor: Codex
Task: `V230-002`
GitHub issue: `#713`
Milestone: `v0.23.0`
Status: LOCAL_VALIDATION_PASSED

## Summary

V230-002 adds executable Rust read-model replay coverage for multi-account
partitioning. The new fixture set proves that account-scoped read-model rows
carry explicit `account_key` and `isolation_scope_key`, that cross-account
component contamination fails closed, and that unknown account identity fails
closed before downstream strategy or dashboard work consumes the data.

Plain Chinese summary: 这一步先把多账户隔离做成可执行的 read-model 证据。两个账户可以
被只读聚合展示，但每个分区必须保留自己的 `account_key` 和 `isolation_scope_key`；
如果 positions/orders/fills/risk/alerts/audit/provenance 里有任一组件串到别的账户，
或者账户身份是 unknown，就必须 fail closed。

## Contract Mapping

```text
contract = docs/rust-cutover/release/v0_23_0_multi_node_isolation_contract.md
required sections = Identity Model, Account Boundary, Allowed Read Paths, Logs And Evidence Boundary
```

## Executable Replay Cases

```text
read_model.account_partition.isolated_accounts.001 = PASS path, two accounts remain isolated
read_model.account_partition.cross_account_mismatch.001 = FAIL-CLOSED path, component account_key mismatch
read_model.account_partition.missing_account_key.001 = FAIL-CLOSED path, unknown account identity
```

## Boundary

```text
account_key_required = true
isolation_scope_key_required = true
cross_account_component_mismatch = fail_closed
unknown_account_identity = fail_closed
read_path_preserves_provenance = required
dashboard_operation_controls_enabled = false
production_order_submission_allowed = false
strategy_orchestration_started = false
venue_node_lifecycle_started = false
```

## Validation

```text
cargo test -p nautilus-cli --test golden_trace_read_model_projection -- --nocapture = PASS, 2 tests passed
jq . docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json = PASS
scripts/ai/verify_fast.sh = PASS, fast smoke only
git diff --check = PASS
```
