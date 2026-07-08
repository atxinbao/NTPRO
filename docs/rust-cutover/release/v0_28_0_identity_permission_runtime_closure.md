# v0.28.0 Identity and Permission Runtime Closure

Date: 2026-07-08
Executor: Codex
Task: `V280-002` / GitHub issue `#895`
Milestone: `v0.28.0`

## Contract

```text
contract_version = ntpro.v280.identity_permission_runtime_closure.v1
schema_version = ntpro.v280.identity_permission_runtime_artifact.v1
release_scope = backend_closure_product_operations_runtime_finalization_only
backend_module = identity_permission_runtime_closure
backend_module_status = runtime_closed
depends_on = V280-001,V280-000,V270-002,V260-002
deterministic_artifact_path = docs/rust-cutover/release/v0_28_0_identity_permission_runtime_artifact.json
operation_authorization_surface = read_admin_only
live_operation_authorization = false
production_trading_authorization = false
external_idp_sso_runtime_integration = false
```

V280-002 closes the identity/permission backend module by making identity source
ingestion, permission mapping, freshness, provenance, redaction, lineage, and
required-false trading permission boundaries replayable from a deterministic
source-controlled artifact.

## Runtime Artifact Path

```text
artifact = docs/rust-cutover/release/v0_28_0_identity_permission_runtime_artifact.json
validator = scripts/ai/verify_v28_identity_permission_runtime_closure.sh
release stage = scripts/ai/verify_release.sh v28-identity-permission-runtime-closure
matrix module = identity_permission_runtime_closure
matrix classification = runtime-closed
```

The artifact is a deterministic backend replay artifact. It is not a live IdP
connection, does not verify live tokens, and does not grant live operation
authorization.

## Required Runtime Inputs

```text
identity_source.provider = required
identity_source.tenant_id_redacted = required
identity_source.issuer = required
identity_source.jwks_fingerprint_sha256 = required
identity_source.collected_at = required
identity_source.freshness_status = fresh
identity_source.provenance_id = required
identity_source.redaction_status = redacted
identity_source.lineage_status = linked
permission_mapping.provenance_id = required
permission_mapping.freshness_status = fresh
permission_mapping.redaction_status = redacted
permission_mapping.lineage_status = linked
```

## Allowed Read/Admin Permissions

```text
operator = dashboard_read, operation_preview_read, runbook_read
admin = dashboard_read, operation_preview_read, runbook_read, audit_read, provenance_read
auditor = dashboard_read, audit_read, provenance_read
release_gatekeeper = dashboard_read, release_gate_read, release_manifest_read
```

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
admin_workbench_trading_controls = false
manual_operation_submit = false
product_grade_trading_terminal_claim = false
```

## Fail-Closed Rules

```text
missing_identity_source_provenance => fail_closed_missing_provenance
missing_permission_mapping_provenance => fail_closed_missing_provenance
stale_identity_source => fail_closed_stale_source
stale_permission_mapping => fail_closed_stale_source
unredacted_identity_or_mapping => fail_closed_provenance_violation
broken_identity_or_mapping_lineage => fail_closed_provenance_violation
unknown_role => fail_closed_permission_drift
permission_not_allowed_for_role => fail_closed_permission_drift
scope_prefix_mismatch => fail_closed_permission_drift
forbidden_trading_permission_true => fail_closed_forbidden_trading_permission
live_operation_authorization_true => fail_closed_forbidden_trading_permission
```

## Unsupported External IdP / SSO Behavior

```text
live_sso_token_verification = false
oauth_authorization_code_flow = false
saml_assertion_validation = false
external_idp_network_call = false
dynamic_group_sync = false
live_operation_authorization_from_idp = false
```

These behaviors remain out of scope for v0.28.0 unless a later scoped release
adds explicit runtime code, evidence, and release gates.

## Boundary Statement

Identity and permission state may be ingested or replayed through the
source-controlled backend artifact only for read/admin permission decisions. It
does not authorize default submit, cancel, retry, replace, amend, flatten,
adapter send, live exchange requests, retry scheduling, automatic remediation,
Dashboard/Admin trading controls, manual operation submit, live IdP/SSO
runtime integration, or product-grade live trading terminal claims.
