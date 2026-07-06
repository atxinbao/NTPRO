# v0.26.0 Deployment Provenance Model

Date: 2026-07-06
Executor: Codex
Task: `V260-004` / GitHub issue `#816`
Milestone: `v0.26.0`

## Deployment Evidence Claim

```text
deployment_provenance_scope = deployment_provenance_evidence_only
depends_on = V260-001 product hardening boundary contract
production_deploy_automation = false
external_deployment_system_added = false
real_production_trading_ready = false
adapter_send_allowed = false
live_exchange_request_allowed = false
dashboard_trading_controls_enabled = false
```

The v0.26.0 deployment provenance model is audit evidence only. It records
environment classification, topology, config provenance, artifact digest, release
tag, and runtime boundary evidence without deploying production infrastructure.

## Environment Schema

Each environment record must provide:

```text
environment_id
environment_classification = local | dev | staging | prod_like
environment_truth_source
release_tag
runtime_boundary
artifact_digest
config_source.source_type
config_source.source_ref
config_source.provenance_ref
config_source.redaction = redacted
nodes[].node_id
nodes[].node_role
nodes[].scope
nodes[].artifact_digest
nodes[].config_source_ref
nodes[].runtime_boundary
```

`prod_like` means production-shaped evidence. It is not real production trading
readiness, real funds readiness, or production deployment.

## Runtime Boundary

```text
deployment_execution_allowed = false
production_order_mutation_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
dashboard_trading_controls_enabled = false
real_production_trading_ready = false
```

## Fail-Closed Rules

```text
missing artifact digest/config provenance/release tag/runtime boundary/environment truth => fail_closed_missing_required_evidence
secret/raw credential/signed payload/signed URL/unredacted config => fail_closed_unredacted_config
unknown environment truth => fail_closed_unknown_environment_truth
release tag mismatch => fail_closed_tag_mismatch
cross-node scope mismatch => fail_closed_cross_node_scope_mismatch
adapter send/live exchange request/production mutation/dashboard control opened => fail_closed_forbidden_runtime_boundary
```

## Release Evidence

```text
trace = tests/golden/v260_deployment_provenance_model.jsonl
validator = scripts/ai/verify_v26_deployment_provenance_model.sh
release stage = scripts/ai/verify_release.sh v26-deployment-provenance-model
release replay scope status = validator_executable_replay
```

## Boundary Statement

Deployment provenance evidence can be audited by release gates and displayed by
Dashboard as read-only evidence. It is not production deploy automation, not a
new deployment system, not adapter send, not live exchange request, and not
real-funds production readiness.
