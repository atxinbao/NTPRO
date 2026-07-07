# v0.27.0 External Identity and Permission Integration Foundation

Date: 2026-07-07
Executor: Codex
Task: `V270-002` / GitHub issue `#855`
Milestone: `v0.27.0`

## Contract

```text
contract_version = ntpro.v270.external_identity_permission_foundation.v1
schema_version = ntpro.v270.external_identity_permission_foundation.schema.v1
identity_permission_integration_scope = external_identity_permission_foundation_only
dependency_contracts = V270-001,V260-002
operation_authorization_surface = read_admin_only
external_identity_provider_evidence_required = true
role_mapping_provenance_required = true
role_mapping_freshness_required = true
role_mapping_redaction_required = true
role_mapping_lineage_required = true
v26_permission_boundary_alignment_required = true
live_operation_authorization = false
production_trading_authorization = false
runtime_submit_permission_enabled = false
```

## External Identity Evidence Shape

```text
idp_provenance.source_type = external_identity_provider
idp_provenance.provider = required
idp_provenance.tenant_id = required
idp_provenance.issuer = required
idp_provenance.jwks_fingerprint = required
idp_provenance.collected_at = required
idp_provenance.freshness_status = fresh
```

The evidence shape records provenance only. It does not verify live tokens,
open SSO/IAM runtime integration, or grant live operation authorization.

## Role Mapping Boundary

```text
operator = dashboard_read, operation_preview_read, runbook_read
admin = dashboard_read, operation_preview_read, runbook_read, audit_read, provenance_read
auditor = dashboard_read, audit_read, provenance_read
release_gatekeeper = dashboard_read, release_gate_read, release_manifest_read
```

Role mappings must include source provenance, freshness, redaction, and lineage.
The mapping is valid only when the mapped role and requested scope match the
v26 read/admin permission boundary. Cross-scope requests fail closed.

## Required-False Permission Boundary

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
manual_operation_submit = false
product_grade_trading_terminal_claim = false
```

## Fail-Closed Rules

```text
missing_idp_provenance => fail_closed_missing_idp_provenance
stale_role_mapping => fail_closed_stale_role_mapping
unknown_role => fail_closed_unknown_role
cross_scope_action => fail_closed_cross_scope_action
v26_permission_boundary_mismatch => fail_closed_v26_boundary_mismatch
trading_permission_true => fail_closed_trading_permission
```

## Release Evidence

```text
trace = tests/golden/v270_external_identity_permission_foundation.jsonl
validator = scripts/ai/verify_v27_external_identity_permission_foundation.sh
release stage = scripts/ai/verify_release.sh v27-external-identity-permission-foundation
release replay scope status = validator_executable_replay
```

## Boundary Statement

External identity evidence can feed read/admin permission mapping and audit
surfaces only. It is not runtime SSO/IAM integration, not live operation
authorization, not production trading authorization, and not a product-grade
live trading terminal claim.
