# v0.30.0 Backend Go-Live Candidate Boundary Contract

Date: 2026-07-11
Executor: Codex
Task: `V300-001` / GitHub issue `#970`
Milestone: `v0.30.0`

## Contract

```text
contract_version = ntpro.v300.backend_go_live_candidate_boundary.v1
schema_version = ntpro.v300.backend_go_live_candidate_boundary.v1
release_scope = backend_production_go_live_candidate_foundation_only
candidate_claim = backend_go_live_candidate_boundary_contract
dependency_start_gate = satisfied
v300_intake_gate = satisfied
v291_release_evidence = published
backend_go_live_candidate_claim = allowed_candidate_evidence_only
candidate_claim_runtime_effect_allowed = false
backend_go_live_claim = false
ambiguous_backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
product_grade_live_trading_terminal_claim = false
production_execution_runtime_claim = false
default_submit_claim = false
default_production_execution_allowed = false
failure_semantics = fail_closed
```

`v0.30.0` is a Backend Production Go-Live Candidate Foundation track. It can
collect source-controlled candidate evidence for a later go/no-go review, but
it does not authorize actual backend production go-live, product-grade live
trading, default production execution, production order mutation, adapter send,
live exchange requests, automatic remediation, or trading controls.

## Terminology

```text
backend_production_readiness = source-controlled readiness evidence only
backend_go_live_candidate = source-controlled candidate evidence for later go/no-go review
actual_backend_production_go_live = runtime enablement that can affect live operational or trading state
product_grade_live_trading_terminal = user-facing live trading surface with submit/mutation/adapter-send/exchange-request/trading controls
candidate_does_not_mean_backend_go_live = true
candidate_does_not_mean_product_grade_terminal = true
candidate_does_not_mean_default_production_execution = true
```

Candidate wording must stay precise: `backend go-live candidate` means the
candidate evidence package is being prepared. It does not mean
`backend_go_live=true`, `backend_production_go_live=true`, or any runtime
production enablement.

## Allowed Candidate Artifacts

Only these artifact classes are allowed in this boundary, and each is evidence
only with `runtime_effect_allowed = false`, `default_enabled = false`, and
`requires_later_enablement_decision = true`.

```text
production_deployment_plan = #971
environment_readiness_record = #971
runtime_enablement_boundary = #972
controlled_feature_flag_inventory = #972
operator_approval_freeze_record = #973
change_window_lifecycle_record = #973
canary_execution_preflight = #974
rollback_disaster_recovery_boundary = #975
production_config_provenance = #976
venue_connectivity_readiness = #976
telemetry_slo_incident_freeze_gate = #977
audit_retention_evidence_export_plan = #978
go_no_go_decision_record = #979
release_gate_v31_handoff = #980
```

## Required Later Enablement Gates

Actual production enablement is out of scope for V300-001. A later scoped issue
must provide all of the following before any enablement can be considered:

```text
owner_operator_approval = required_later
freeze_criteria = required_later
rollback_readiness = required_later
audit_retention = required_later
telemetry_slo_gate = required_later
risk_gate = required_later
release_gate = required_later
scoped_enablement_issue = required_later
approval_bypass_allowed = false
```

## Ambiguous Claim Rejections

```text
backend_go_live=true => fail_closed_ambiguous_go_live_claim
backend_production_go_live=true => fail_closed_ambiguous_go_live_claim
production_go_live=true => fail_closed_ambiguous_go_live_claim
go_live_candidate=production_enabled => fail_closed_ambiguous_go_live_claim
backend_go_live_candidate=live_enabled => fail_closed_ambiguous_go_live_claim
product_grade_live_trading_terminal=true => fail_closed_product_grade_live_trading_claim
default_production_execution=true => fail_closed_default_execution_claim
```

## Required-False Boundary Flags

Every v0.30.0 candidate artifact that declares capability boundaries must carry
these fields explicitly as `false`. Missing fields fail closed.

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
```

## Fail-Closed Rules

```text
backend go-live candidate missing v0.29.1 release evidence => fail_closed_boundary_violation
backend go-live candidate missing V300-000 intake proof => fail_closed_boundary_violation
backend_go_live=true => fail_closed_ambiguous_go_live_claim
actual backend production go-live claim true => fail_closed_boundary_violation
product-grade terminal claim true => fail_closed_boundary_violation
default production execution claim true => fail_closed_boundary_violation
candidate artifact runtime effect true => fail_closed_boundary_violation
candidate artifact missing scoped issue => fail_closed_boundary_violation
candidate artifact missing later go/no-go decision => fail_closed_boundary_violation
required later enablement gate bypass true => fail_closed_boundary_violation
forbidden trading/control boundary true => fail_closed_boundary_violation
required-false boundary missing => fail_closed_boundary_violation
```

## Release Evidence

```text
V300-000 start_gate_status = satisfied
boundary_contract = docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md
boundary_contract_json = docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.json
validator = scripts/ai/verify_v30_backend_go_live_candidate_boundary_contract.sh
release stage = scripts/ai/verify_release.sh v30-backend-go-live-candidate-boundary-contract
v30 intake stage = scripts/ai/verify_release.sh v30-intake-gate
allowed_candidate_artifacts = 14
required_later_enablement_prerequisites = 8
required_false_boundary_flags = 30
ambiguous_claim_rejections = 7
```

## Boundary Statement

This contract opens only the v0.30.0 backend go-live candidate evidence line.
It does not authorize default submit, cancel, retry, replace, amend, flatten,
adapter send, live exchange request, retry scheduling, automatic remediation,
Dashboard operation controls, Dashboard trading controls, Admin Workbench
trading controls, Trader Terminal order tickets, manual operation submit,
actual backend go-live, product-grade live trading terminal claims, or default
production execution.
