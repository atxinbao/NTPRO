# v0.18.0 Cancel Recovery Artifact Contracts

Date: 2026-06-26
Executor: Codex
Status: READY_FOR_REVIEW

## Summary

v0.18.0 defines Owner-Approved Cancel Recovery Preview artifacts.

Plain Chinese summary: v0.18.0 的 artifact contract 只描述撤单恢复预览，不允许真实撤单。

## Required Boundary

```text
Owner-Approved Cancel Recovery Preview
actual_cancel_send_allowed=false
cancel_attempted=false
automatic_cancel_allowed=false
dashboard_cancel_controls_enabled=false
network_attempted=false
production_order_mutations_attempted=0
manual_owner_approval_required=true
owner_approved=false
```

## Artifact Set

```text
ntpro.v180_cancel_recovery_scope_decision.v1
ntpro.v180_cancel_recovery_artifact_contracts.v1
ntpro.v180_cancel_request_preview.v1
ntpro.v180_cancel_risk_gate.v1
ntpro.v180_cancel_manual_approval_lifecycle.v1
ntpro.v180_cancel_response_redaction.v1
ntpro.v180_cancel_post_cancel_readback.v1
ntpro.v180_cancel_incident_audit_closeout.v1
```

## Validation

```text
scripts/ai/verify_v18_cancel_recovery_gates.sh
```

