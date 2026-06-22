# NTPRO v0.15.0 Incident Rollback Artifact Contract

Date: 2026-06-22
Executor: Codex
Milestone: `v0.15.0`
Task: `V150-007`
Status: local validation passed; hosted PR smoke pending

## Summary

`v0.15.0` defines manual incident, rollback, and emergency-stop artifacts for
the production live-alpha mutation dry-run line. These artifacts are evidence
contracts only. They do not execute production cancel, correction, retry,
automatic remediation, real exchange mutation, network access, or production
adapter calls.

Plain Chinese summary: 这不是“自动回滚功能”。它是三份人工证据文件的合同：
出事时记录事故、记录回滚计划、记录紧急停止计划。系统只能生成/校验这些文件，不能
自动去交易所撤单、改单、重试、纠错，也不能做真实 mutation。

## Artifact Files

```text
incident_plan.json
rollback_plan.json
emergency_stop.json
```

## Common Required Boundary Fields

Every artifact must record:

```json
{
  "manual_evidence_only": true,
  "manual_operator_required": true,
  "automatic_remediation_allowed": false,
  "production_cancel_allowed": false,
  "production_correction_allowed": false,
  "production_retry_allowed": false,
  "real_exchange_mutation_allowed": false,
  "network_attempted": false,
  "execution_adapter_called": false,
  "dashboard_order_controls_enabled": false,
  "production_orders_submitted": 0,
  "production_order_mutations_attempted": 0
}
```

## Schemas

### incident_plan.json

```json
{
  "schema_version": "ntpro.v150_incident_plan.v1",
  "artifact_type": "manual_incident_plan",
  "status": "manual_review_required",
  "incident_id": "incident-v150-007-001",
  "severity": "high",
  "trigger": "mutation_dry_run_preflight_rejected",
  "manual_evidence_only": true,
  "manual_operator_required": true
}
```

### rollback_plan.json

```json
{
  "schema_version": "ntpro.v150_rollback_plan.v1",
  "artifact_type": "manual_rollback_plan",
  "status": "manual_rollback_required",
  "rollback_id": "rollback-v150-007-001",
  "source_incident_id": "incident-v150-007-001",
  "manual_evidence_only": true,
  "manual_operator_required": true
}
```

### emergency_stop.json

```json
{
  "schema_version": "ntpro.v150_emergency_stop.v1",
  "artifact_type": "manual_emergency_stop",
  "status": "emergency_stop_required",
  "emergency_stop_id": "emergency-stop-v150-007-001",
  "kill_switch_target_state": "active",
  "manual_evidence_only": true,
  "manual_operator_required": true
}
```

## Forbidden Fields

The verifier must reject any artifact or nested action that records:

```text
automatic_remediation_allowed=true
automatic_remediation_attempted=true
production_cancel_allowed=true
production_cancel_attempted=true
production_correction_allowed=true
production_correction_attempted=true
production_retry_allowed=true
production_retry_attempted=true
real_exchange_mutation_allowed=true
real_exchange_mutation_attempted=true
network_attempted=true
execution_adapter_called=true
dashboard_order_controls_enabled=true
executes_automatically=true
production_orders_submitted > 0
production_order_mutations_attempted > 0
```

## Validation

```text
scripts/ai/verify_v15_incident_rollback_artifact.sh
```
