# v0.26.0 SLO Runbook Stability Evidence

Date: 2026-07-06
Executor: Codex
Task: `V260-006` / GitHub issue `#818`
Milestone: `v0.26.0`

## Stability Evidence Claim

```text
stability_artifact_scope = slo_runbook_long_run_stability_evidence_only
depends_on = V260-001 product hardening boundary contract
depends_on = V260-004 deployment provenance model
depends_on = V260-005 upgrade rollback runbook evidence
sample_provenance_required = true
sample_freshness_required = true
sample_redaction_required = true
automatic_remediation_allowed = false
restart_execution_allowed = false
trading_operation_allowed = false
dashboard_execution_controls_enabled = false
```

The v0.26.0 stability model is evidence only. It can classify healthy,
degraded, or fail-closed stability states, and it can display restart
recommendations as preview text, but it must not restart services, stop
strategies, cancel orders, submit orders, recover trading, or remediate runtime
state automatically.

## Stability Schema

Each stability evidence artifact must provide:

```text
long_run_window.window_id
long_run_window.observed_minutes
long_run_window.min_required_minutes
long_run_window.sample_count
long_run_window.expected_component_count
long_run_window.present_component_count
long_run_window.freshness_status = fresh
long_run_window.sample_provenance_ref
long_run_window.redaction = redacted
long_run_window.release_tag
long_run_window.expected_release_tag
sample_provenance.ref
sample_provenance.source_digest
sample_provenance.redaction = redacted
components[].component_id
components[].status = present
components[].freshness_status = fresh
components[].sample_ref
components[].redaction = redacted
slo_objectives[].objective_id
slo_objectives[].target
slo_objectives[].observed
slo_objectives[].error_budget_remaining
slo_objectives[].error_budget_exhausted = false
runbook.runbook_ref
runbook.freshness_status = current
runbook.recommendation_only = true
restart_recommendation.recommended
restart_recommendation.recommendation_only = true
restart_recommendation.execution_triggered = false
```

## Fail-Closed and Degraded Rules

```text
missing sample provenance/redaction => degraded_missing_required_evidence
stale sample freshness/unredacted sample => degraded_stale_or_unredacted_samples
missing component coverage => degraded_missing_components_restart_recommended
error budget exhausted => fail_closed_error_budget_exhausted
runbook stale => degraded_runbook_stale
release drift => fail_closed_release_drift
restart/remediation/trading/dashboard execution opened => fail_closed_forbidden_execution_boundary
```

Restart recommendations are preview-only. They do not restart services and do
not trigger rollback, trading recovery, order mutation, strategy stop, or
automatic remediation.

## Release Evidence

```text
trace = tests/golden/v260_slo_runbook_stability_evidence.jsonl
validator = scripts/ai/verify_v26_slo_runbook_stability_evidence.sh
release stage = scripts/ai/verify_release.sh v26-slo-runbook-stability-evidence
release replay scope status = validator_executable_replay
```

## Boundary Statement

Stability evidence can be audited by release gates and displayed by Dashboard
as read-only evidence. It is not real soak infrastructure, not automatic
recovery, not restart execution, not trading operation, not live exchange
request, and not product-grade live trading terminal readiness.
