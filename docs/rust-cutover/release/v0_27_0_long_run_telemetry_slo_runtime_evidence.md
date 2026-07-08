# v0.27.0 Long-Run Telemetry SLO Runtime Evidence

Date: 2026-07-08
Executor: Codex
Task: `V270-005` / GitHub issue `#858`
Milestone: `v0.27.0`

## Contract

```text
contract_version = ntpro.v270.long_run_telemetry_slo_runtime_evidence.v1
schema_version = ntpro.v270.long_run_telemetry_slo_runtime_evidence.schema.v1
telemetry_ingestion_scope = long_run_telemetry_ingestion_slo_runtime_evidence_only
ingestion_mode = observational_non_remediating
dependency_contracts = V270-001,V260-006
source_contract_required = true
source_freshness_required = true
source_redaction_required = true
sampling_window_required = true
sampling_gap_detection_required = true
slo_rollup_required = true
admin_read_model_degradation_reasons_required = true
dashboard_degradation_reasons_required = true
automatic_remediation_allowed = false
retry_scheduler_enabled = false
adapter_send_allowed = false
live_exchange_request_allowed = false
telemetry_event_triggers_trading_control = false
dashboard_trading_controls_enabled = false
```

## Telemetry Source Contract

```text
source_type = local_runtime_telemetry_artifact
source_ref = required
source_digest = required
source_snapshot_id = required
freshness_status = fresh | stale | missing
observed_age_ms <= max_age_ms for healthy
redaction_status = redacted
raw_secret_persisted = false
raw_exchange_response_persisted = false
release_tag == expected_release_tag
source_scope = artifact_truth_only
runtime_adapter_integration = false
```

## Sampling And SLO Semantics

```text
sampling_window.duration_minutes >= sampling_window.min_duration_minutes
sampling_window.sample_count >= sampling_window.expected_sample_count
sampling_window.gap_count == 0 for healthy
sampling_window.max_gap_ms <= sampling_window.allowed_gap_ms for healthy
slo_rollup.observed_availability >= slo_rollup.availability_target for healthy
slo_rollup.error_budget_remaining >= 0 for healthy/degraded
```

## Read-Only Surface Semantics

```text
admin_read_model.status = healthy | degraded | fail_closed
dashboard.status = healthy | degraded | fail_closed
admin_read_model.degradation_reasons = required when degraded/fail_closed
dashboard.degradation_reasons = required when degraded/fail_closed
read_only = true
display_only = true
operation_controls_enabled = false
trading_controls_enabled = false
```

## Fail-Closed And Degraded Rules

```text
healthy_source_and_window => healthy
sampling_gap_detected => degraded_sampling_gap
stale_source => degraded_stale_source
missing_source_contract => fail_closed_missing_source_contract
redaction_breach => fail_closed_redaction_breach
release_or_source_drift => fail_closed_release_source_drift
forbidden_operation_boundary => fail_closed_forbidden_operation_boundary
```

## Required-False Operation Boundary

```text
automatic_remediation_allowed = false
retry_scheduler_enabled = false
adapter_send_allowed = false
live_exchange_request_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trading_operation_allowed = false
telemetry_event_triggers_remediation = false
telemetry_event_triggers_retry = false
telemetry_event_triggers_adapter_send = false
telemetry_event_triggers_trading_control = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Release Evidence

```text
trace = tests/golden/v270_long_run_telemetry_slo_runtime_evidence.jsonl
validator = scripts/ai/verify_v27_long_run_telemetry_slo_runtime_evidence.sh
release stage = scripts/ai/verify_release.sh v27-long-run-telemetry-slo-runtime-evidence
release replay scope status = validator_executable_replay
```

## Boundary Statement

Telemetry ingestion evidence is observational only. It can explain Admin
Workbench and Dashboard health, degradation, staleness, gaps, redaction
breaches, and release/source drift, but it cannot execute remediation, schedule
retries, call adapters, access live exchanges, submit/mutate orders, or enable
trading controls.
