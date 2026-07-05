# v0.25.0 Runbook And Audit Evidence Model

Date: 2026-07-05
Executor: Codex
Task: `V250-004`
GitHub issue: `#781`
Milestone: `v0.25.0`
Status: LOCAL VALIDATION PASSED

## Summary

V250-004 defines the v0.25.0 runbook and audit evidence model. Runbooks are
manual/read-only evidence: they record observations, acknowledgements,
escalations, rollback recommendations, audit traces, and decision reasons
without executing actions.

Plain Chinese summary: 本任务只定义 runbook/audit evidence。runbook step 可以记录
versioned source、input evidence、decision output、owner、audit trace、provenance、
freshness 和 redaction；但不会执行 shell/runbook 自动化，不会自动 cancel/retry/
remediation，也不会授予真实交易操作权限。

## Contract Fields

```text
contract_version = ntpro.v250.runbook_audit_evidence.v1
schema_version = ntpro.v250.runbook_audit_evidence.schema.v1
runbook_id = required
runbook_version = required
step_id / step_name = required
owner = required
versioned_source = source_ref / version / checksum required
input_evidence = required non-empty evidence links
decision_type = manual_observation | manual_acknowledgement | manual_escalation | manual_rollback_recommendation
decision_output = approval_status / visible_result / side_effect
audit_trace = required non-empty
source_provenance = required
lineage = required non-empty
freshness.status = fresh required
redaction_state = redacted required
```

## Execution Boundary

```text
runbook_mode = evidence_only
shell_execution_allowed = false
runbook_automation_allowed = false
permission_system_extension_allowed = false
automatic_remediation_allowed = false
automatic_strategy_stop_allowed = false
automatic_submit_allowed = false
automatic_cancel_allowed = false
automatic_retry_allowed = false
automatic_replace_allowed = false
automatic_amend_allowed = false
automatic_flatten_allowed = false
live_exchange_request_allowed = false
adapter_send_allowed = false
dashboard_trading_control_allowed = false
```

## Golden Trace Coverage

```text
read_model.runbook_audit_evidence.valid_manual_matrix.001 = manual observation, acknowledgement, escalation, and rollback recommendation are accepted as read-only evidence
read_model.runbook_audit_evidence.stale_runbook_fail_closed.001 = stale runbook evidence fails closed
read_model.runbook_audit_evidence.missing_version_fail_closed.001 = missing versioned source fails closed
read_model.runbook_audit_evidence.missing_audit_trace_fail_closed.001 = missing audit trace fails closed
read_model.runbook_audit_evidence.unapproved_action_fail_closed.001 = unapproved real action fails closed instead of granting authority
read_model.runbook_audit_evidence.redaction_secret_fail_closed.001 = secret/signed/raw payload leak fails closed
read_model.runbook_audit_evidence.automatic_execution_fail_closed.001 = shell/runbook automation or trading action boundary fails closed
```

## Evidence

Validation is recorded in `docs/rust-cutover/evidence/V250-004.md` and
`verification.md`. The release replay manifest records the V250 runbook/audit
cases as `validator_executable_replay`; this is evidence validation, not shell
automation, permission extension, adapter integration, or live operation.
