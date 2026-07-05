# v0.25.0 Incident Lifecycle And Operator Acknowledgement

Date: 2026-07-05
Executor: Codex
Task: `V250-003`
GitHub issue: `#780`
Milestone: `v0.25.0`
Status: LOCAL VALIDATION PASSED

## Summary

V250-003 defines the v0.25.0 incident lifecycle and operator acknowledgement
evidence contract. Incidents are read-only evidence that connect alert routing
to operator-owned state transitions and acknowledgement proof.

Plain Chinese summary: 本任务只定义 incident lifecycle 和人工确认的证据模型。
incident 可以记录 opened、triaged、acknowledged、mitigated、resolved、postmortem
的状态链路、owner/assignee、source alert、audit trace、lineage、provenance、
freshness 和 redaction；但它不会接入真实工单系统，不会自动停策略、撤单或提交订单，
也不会调用 adapter send 或 live exchange。

## Contract Fields

```text
contract_version = ntpro.v250.incident_lifecycle_acknowledgement.v1
schema_version = ntpro.v250.incident_lifecycle_acknowledgement.schema.v1
incident states = opened | triaged | acknowledged | mitigated | resolved | postmortem
allowed transitions = none->opened, opened->triaged, triaged->acknowledged,
  acknowledged->mitigated, mitigated->resolved, resolved->postmortem
owner = required
assignee = required
source_alert = alert_id / alert_case_id / dedupe_key / severity / category
operator_acknowledgement = required before resolved or postmortem
source_provenance = required
lineage = required non-empty evidence links
audit_trace = required non-empty transition/action evidence
freshness.status = fresh required
redaction_state = redacted required
```

## Incident Boundary

```text
incident_mode = evidence_only
external_ticket_system_connected = false
external_ticket_mutation_allowed = false
automatic_paging_allowed = false
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
read_model.incident_lifecycle_acknowledgement.valid_lifecycle.001 = full lifecycle with owner acknowledgement is accepted as read-only evidence
read_model.incident_lifecycle_acknowledgement.invalid_transition_fail_closed.001 = illegal transition fails closed
read_model.incident_lifecycle_acknowledgement.missing_owner_source_alert_fail_closed.001 = missing owner/source alert fails closed
read_model.incident_lifecycle_acknowledgement.missing_ack_resolved_fail_closed.001 = resolved without acknowledgement fails closed
read_model.incident_lifecycle_acknowledgement.stale_incident_fail_closed.001 = stale incident freshness fails closed
read_model.incident_lifecycle_acknowledgement.redaction_secret_fail_closed.001 = secret/signed/raw payload leak fails closed
read_model.incident_lifecycle_acknowledgement.automatic_action_fail_closed.001 = automatic operation or ticket mutation boundary fails closed
```

## Evidence

Validation is recorded in `docs/rust-cutover/evidence/V250-003.md` and
`verification.md`. The release replay manifest records the V250 incident
lifecycle cases as `validator_executable_replay`; this is incident evidence
validation, not runtime adapter integration or an external ticket integration.
