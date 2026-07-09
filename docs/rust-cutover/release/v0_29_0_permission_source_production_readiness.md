# v0.29.0 Permission Source Production Readiness

Date: 2026-07-09
Executor: Codex
Task: `V290-004` / GitHub issue `#930`
Milestone: `v0.29.0`

## Contract

```text
contract_version = ntpro.v290.permission_source_production_readiness.v1
schema_version = ntpro.v290.permission_source_production_readiness_artifact.v1
release_scope = backend_production_readiness_foundation_only
backend_module = permission_source_production_readiness
backend_module_status = production_ready_evidence
readiness_mode = deterministic_readiness_replay
depends_on = V290-000,V290-001,V290-002,V280-002,v0.28.1-release-evidence
deterministic_artifact_path = docs/rust-cutover/release/v0_29_0_permission_source_production_readiness_artifact.json
operation_authorization_surface = read_admin_only
permission_source_claim = source_controlled_sandbox_fixture
live_operation_authorization = false
production_trading_authorization = false
external_idp_sso_runtime_integration = false
release stage = scripts/ai/verify_release.sh v29-permission-source-production-readiness
```

`production-ready` here means permission readiness evidence is deterministic,
auditable, and fail-closed. It does not mean live IdP/SSO integration, live
operation authorization, production trading authorization, or backend go-live.

## Requirements

```text
permission_source.provider = required
permission_source.config_digest = required
permission_source.freshness_status = fresh
permission_source.provenance_status = linked
permission_source.redaction_status = redacted
permission_mapping.freshness_status = fresh
permission_mapping.provenance_status = linked
permission_mapping.revocation_status = enforced
operator/admin/auditor/release_gatekeeper = read/admin only
forbidden trading permissions = false
unsupported external IdP/SSO behavior = false
```

## Fail-Closed Rules

```text
missing_source_provenance => fail_closed_missing_provenance
stale_permission_source => fail_closed_stale_source
unredacted_identity_or_mapping => fail_closed_provenance_violation
broken_identity_or_mapping_lineage => fail_closed_provenance_violation
revoked_subject_permission_check => fail_closed_permission_revoked
unknown_role_or_permission_drift => fail_closed_permission_drift
forbidden_trading_permission_true => fail_closed_forbidden_trading_permission
```

## Boundary Statement

Permission source readiness validates read/admin permission decisions only. It
does not connect to a live IdP, validate live SSO tokens, grant live operation
authorization, enable production trading permissions, call adapters, access live
exchanges, expose trading controls, or claim backend go-live/product-grade
terminal readiness.
