# v0.26.0 Operator Permission Model

Date: 2026-07-06
Executor: Codex
Task: `V260-002` / GitHub issue `#814`
Milestone: `v0.26.0`

## Permission Evidence Claim

```text
permission_artifact_scope = operator_permission_evidence_only
depends_on = V260-001 product hardening boundary contract
external_identity_provider_integration = false
live_operation_authorization = false
production_trading_authorization = false
dashboard_trading_controls_enabled = false
```

The v0.26.0 operator permission model is a product hardening evidence model. It
defines roles and audit boundaries that can be displayed by Dashboard or
release gates, but it does not authorize live operations.

## Roles

```text
viewer = dashboard_read only
operator = dashboard_read, operation_preview_read, runbook_read
release_gatekeeper = dashboard_read, release_gate_read, release_manifest_read
incident_owner = dashboard_read, incident_ack_review, runbook_read
auditor = dashboard_read, audit_read, provenance_read
```

All permissions are deny-by-default. A role may only assert the permissions
listed above for its own scope. Missing role identity, missing scope, missing
provenance, missing required permissions, cross-scope permission evidence,
expired approval, or role escalation fails closed.

## Required Approval Evidence

```text
viewer approval_required = false
auditor approval_required = false
operator approval_required = true
release_gatekeeper approval_required = true
incident_owner approval_required = true
approval status must be approved when required
approval expires_at must be later than evaluated_at when required
```

Approval evidence remains read-only. It does not become owner approval runtime
logic and does not authorize submit, cancel, retry, replace, amend, flatten, or
adapter send.

## Forbidden Role Capabilities

Every role must keep these capability fields explicitly `false`.

```text
submit_order = false
cancel_order = false
replace_order = false
amend_order = false
flatten_position = false
retry_scheduler = false
automatic_remediation = false
adapter_send = false
live_exchange_request = false
dashboard_trading_controls = false
role_escalation = false
```

## Fail-Closed Rules

```text
missing role/scope/provenance => fail_closed_missing_required_evidence
missing required role permission => fail_closed_permission_denied
cross-scope permission => fail_closed_cross_scope_permission
expired approval => fail_closed_expired_approval
role escalation => fail_closed_role_escalation
forbidden trading control true => fail_closed_trading_control
```

## Release Evidence

```text
trace = tests/golden/v260_operator_permission_model.jsonl
validator = scripts/ai/verify_v26_operator_permission_model.sh
release stage = scripts/ai/verify_release.sh v26-operator-permission-model
release replay scope status = validator_executable_replay
```

## Boundary Statement

Permission evidence can be shown by Dashboard and release gates for read-only
audit. It is not live operation authorization, not production trading
authorization, not SSO/IAM integration, and not a change to owner approval
lifecycle runtime behavior.
