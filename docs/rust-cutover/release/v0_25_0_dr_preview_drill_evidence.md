# v0.25.0 Disaster Recovery Preview Drill Evidence

Date: 2026-07-05
Executor: Codex
Task: `V250-005`
GitHub issue: `#782`
Milestone: `v0.25.0`
Status: LOCAL VALIDATION PASSED

## Summary

V250-005 defines the v0.25.0 disaster-recovery preview drill evidence contract.
DR previews are audit artifacts only: they describe restart, read-model rebuild,
artifact replay, release rollback recommendation, recovery points, snapshots,
readback checks, and operator approval without executing recovery.

Plain Chinese summary: 本任务只定义 DR preview drill evidence。DR preview 可以记录
scenario、affected scope、snapshot refs、expected recovery point、readback/audit
refs、operator approval status 和 snapshot lineage；但不会自动重启服务，不执行数据
恢复，不发送 live request，也不会修改生产订单或 exchange state。

## Contract Fields

```text
contract_version = ntpro.v250.dr_preview_drill_evidence.v1
schema_version = ntpro.v250.dr_preview_drill_evidence.schema.v1
scenario = restart_preview | read_model_rebuild_preview | artifact_replay_preview | release_rollback_recommendation
affected_scope = account_key / strategy_key / venue_node_key / isolation_scope_key
scope_consistency.expected_isolation_scope_key = required
snapshot_refs = required non-empty
expected_recovery_point = required and fresh
readback_refs = required non-empty
operator_approval.status = owner_approved | audit_gate_approved | blocked_preview
source_provenance = required
snapshot_lineage = required non-empty
audit_trace = required non-empty
preview_output.side_effect = none
preview_output.execution_claim = false
redaction_state = redacted required
```

## Preview Boundary

```text
dr_mode = preview_only
automatic_restart_allowed = false
service_restart_execution_allowed = false
data_restore_execution_allowed = false
artifact_replay_execution_allowed = false
release_rollback_execution_allowed = false
production_order_mutation_allowed = false
exchange_state_mutation_allowed = false
live_exchange_request_allowed = false
adapter_send_allowed = false
automatic_remediation_allowed = false
```

## Golden Trace Coverage

```text
read_model.dr_preview_drill_evidence.valid_preview_matrix.001 = restart, read-model rebuild, artifact replay, and rollback recommendation previews are accepted as read-only evidence
read_model.dr_preview_drill_evidence.missing_snapshot_fail_closed.001 = missing snapshot refs fail closed
read_model.dr_preview_drill_evidence.stale_recovery_point_fail_closed.001 = stale recovery point fails closed
read_model.dr_preview_drill_evidence.scope_mismatch_fail_closed.001 = affected scope mismatch fails closed
read_model.dr_preview_drill_evidence.unapproved_restore_fail_closed.001 = unapproved restore/restart fails closed unless blocked preview
read_model.dr_preview_drill_evidence.actual_execution_claim_fail_closed.001 = actual recovery execution or exchange mutation claim fails closed
read_model.dr_preview_drill_evidence.redaction_secret_fail_closed.001 = secret/signed/raw payload leak fails closed
```

## Evidence

Validation is recorded in `docs/rust-cutover/evidence/V250-005.md` and
`verification.md`. The release replay manifest records the V250 DR preview
cases as `validator_executable_replay`; this is preview evidence validation, not
service restart, data restore, adapter integration, live operation, or exchange
state mutation.
