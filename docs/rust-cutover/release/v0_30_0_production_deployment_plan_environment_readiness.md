# v0.30.0 Production Deployment Plan And Environment Readiness

Date: 2026-07-11
Executor: Codex
Task: `V300-002` / GitHub issue `#971`
Milestone: `v0.30.0`

## Contract

```text
contract_version = ntpro.v300.production_deployment_plan_environment_readiness.v1
schema_version = ntpro.v300.production_deployment_plan_environment_readiness.v1
release_scope = backend_production_go_live_candidate_foundation_only
candidate_claim = production_deployment_plan_environment_readiness
depends_on = V300-000,V300-001,v0.29.1-release-evidence
boundary_contract = docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md
deterministic_artifact_path = docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json
deployment_mode = source_controlled_preview_only
environment_readiness_mode = deterministic_readiness_replay
production_deployment_execution_allowed = false
production_deployment_executed = false
dry_run_or_preview_evidence_only = true
live_environment_mutation_allowed = false
release stage = scripts/ai/verify_release.sh v30-production-deployment-plan-environment-readiness
```

V300-002 records the source-controlled deployment plan and environment
readiness evidence needed for a later go/no-go review. It does not execute a
production deployment, open runtime production enablement, call adapters, reach
live exchanges, mutate orders, or expose Dashboard/Admin/Trader Terminal
operation controls.

## Deployment Targets

```text
prod-control-plane = backend_control_plane, environment = prod-candidate-primary, execution_allowed = false
prod-read-api = read_only_backend_api, environment = prod-candidate-primary, execution_allowed = false
prod-audit-storage = persistent_audit_storage, environment = prod-candidate-primary, execution_allowed = false
prod-telemetry-slo = telemetry_slo_pipeline, environment = prod-candidate-primary, execution_allowed = false
prod-canary-sandbox = canary_preview_lane, environment = prod-candidate-canary, execution_allowed = false
prod-dr-preview = disaster_recovery_preview_lane, environment = prod-candidate-dr, execution_allowed = false
```

Every target is preview-only and requires a later scoped enablement issue,
owner/operator approval, freeze criteria, risk gate, audit gate, telemetry SLO
gate, rollback gate, and release gate before any execution can be considered.

## Environment Inventory

```text
prod-candidate-primary = fresh, linked, artifact_binding = matched, config_binding = matched
prod-candidate-canary = fresh, linked, artifact_binding = matched, config_binding = matched
prod-candidate-dr = fresh, linked, artifact_binding = matched, config_binding = matched
```

Environment inventory is source-controlled candidate evidence only. Missing,
stale, or mismatched inventory fails closed and cannot be treated as deployment
permission.

## Provenance

```text
artifact_provenance.release_tag = ntpro-rust-only-v0.29.1
artifact_provenance.release_manifest = docs/rust-cutover/release/v0_29_1_release_manifest.json
artifact_provenance.release_closeout = docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md
artifact_provenance.boundary_contract = docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md
config_provenance.deployment_plan = docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.md
config_provenance.environment_inventory = docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json
config_provenance.v29_deployment_readiness = docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness_artifact.json
```

Artifact provenance and config provenance must both be fresh, linked, and
matched to the source-controlled plan. Missing or mismatched provenance fails
closed.

## Migration And Upgrade Prerequisites

```text
schema_migration_preview = preview_ready, execution_performed = false
config_upgrade_preview = preview_ready, execution_performed = false
artifact_compatibility_preview = preview_ready, execution_performed = false
operator_handoff_preview = preview_ready, execution_performed = false
```

Each prerequisite is required before any later production execution. This task
only records preview readiness and does not perform migration or upgrade work.

## Rollback Checkpoints

```text
pre_deploy_snapshot_checkpoint = documented, execution_triggered = false
artifact_revert_checkpoint = documented, execution_triggered = false
config_revert_checkpoint = documented, execution_triggered = false
schema_rollback_preview_checkpoint = documented, execution_triggered = false
traffic_revert_checkpoint = documented, execution_triggered = false
```

Rollback checkpoints are documented for reconstruction and later review only.
No rollback action is executed by this task.

## Fail-Closed Rules

```text
missing_environment_evidence => fail_closed_missing_environment_evidence
stale_environment_evidence => fail_closed_stale_environment_evidence
mismatched_environment_evidence => fail_closed_mismatched_environment_evidence
missing_artifact_provenance => fail_closed_missing_artifact_provenance
missing_config_provenance => fail_closed_missing_config_provenance
missing_migration_prerequisite => fail_closed_missing_migration_prerequisite
missing_rollback_checkpoint => fail_closed_missing_rollback_checkpoint
production_deployment_executed => fail_closed_forbidden_execution
required_false_boundary_opened => fail_closed_forbidden_boundary
```

## Required-False Boundary Flags

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
cancel_order_allowed = false
replace_order_allowed = false
amend_order_allowed = false
flatten_position_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
network_attempted = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
backend_go_live_claim = false
ambiguous_backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
production_runtime_enablement_allowed = false
product_grade_trading_terminal_claim = false
product_grade_live_trading_terminal_claim = false
default_production_execution_allowed = false
candidate_artifact_runtime_effect_allowed = false
production_feature_flags_default_enabled = false
shared_approval_consumption_allowed = false
production_deployment_execution_allowed = false
production_deployment_executed = false
live_environment_mutation_allowed = false
```

## Boundary Statement

This plan is reconstructable from source-controlled artifacts and preview
evidence. It does not authorize production deployment, backend go-live, default
production execution, submit, cancel, replace, amend, flatten, adapter send,
live exchange request, retry scheduling, automatic remediation, Dashboard/Admin
/Trader Terminal operation controls, or product-grade live trading claims.
