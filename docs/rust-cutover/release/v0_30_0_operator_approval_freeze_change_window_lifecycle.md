# v0.30.0 Operator Approval Freeze And Change-Window Lifecycle

Date: 2026-07-11
Executor: Codex
Task: `V300-004` / GitHub issue `#973`
Milestone: `v0.30.0`

## Contract

```text
contract_version = ntpro.v300.operator_approval_freeze_change_window_lifecycle.v1
schema_version = ntpro.v300.operator_approval_freeze_change_window_lifecycle.v1
release_scope = backend_production_go_live_candidate_foundation_only
candidate_claim = operator_approval_freeze_change_window_lifecycle
depends_on = V300-001,V300-003,v0.29.1-release-evidence
boundary_contract = docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md
runtime_flag_boundary = docs/rust-cutover/release/v0_30_0_runtime_enablement_boundary_controlled_feature_flags.json
deterministic_artifact_path = docs/rust-cutover/release/v0_30_0_operator_approval_freeze_change_window_lifecycle.json
lifecycle_mode = audited_candidate_evidence_only
candidate_operations_require_approval = true
candidate_operations_require_active_change_window = true
candidate_operation_execution_allowed = false
approval_lifecycle_authorizes_trading_operations = false
validator = scripts/ai/verify_v30_operator_approval_freeze_change_window_lifecycle.sh
```

V300-004 models go/no-go approval, freeze, unfreeze, emergency stop, and
change-window lifecycle as source-controlled audited candidate evidence. It
does not authorize submit, cancel, replace, amend, flatten, adapter send, live
exchange requests, retry scheduling, automatic remediation, backend go-live, or
product-grade live trading claims.

## Approval Evidence

```text
go_no_go_candidate_approval = approved_for_candidate_review, identity = linked, audit = immutable
freeze_operator_approval = approved_for_candidate_freeze, identity = linked, audit = immutable
change_window_operator_approval = approved_for_preview_window, identity = linked, audit = immutable
approval_evidence_authorizes_submit = false
approval_evidence_authorizes_cancel = false
approval_evidence_authorizes_replace = false
approval_evidence_authorizes_amend = false
approval_evidence_authorizes_flatten = false
approval_evidence_authorizes_automatic_remediation = false
```

Approval evidence is required for candidate operation review, but it is not
trading operation authorization.

## Freeze And Change Window

```text
freeze_state = candidate_freeze_active
freeze_bypass_allowed = false
automatic_unfreeze_allowed = false
unfreeze_requires_later_approval = true
emergency_stop_available = true
change_window_status = active_preview_window
active_change_window_required = true
change_window_execution_allowed = false
```

Candidate operations fail closed without both approval evidence and an active
preview change window. The active preview window is evidence for review only;
it does not open production execution.

## Lifecycle Events

```text
go_no_go_review_opened = audited, immutable, identity_linked, trading_authorization_granted = false
candidate_freeze_entered = audited, immutable, identity_linked, trading_authorization_granted = false
preview_change_window_opened = audited, immutable, identity_linked, trading_authorization_granted = false
emergency_stop_bound = audited, immutable, identity_linked, trading_authorization_granted = false
unfreeze_plan_recorded = audited, immutable, identity_linked, trading_authorization_granted = false
```

## Fail-Closed Rules

```text
missing_approval => fail_closed_missing_approval
missing_active_change_window => fail_closed_missing_active_change_window
missing_identity_provenance => fail_closed_missing_identity_provenance
missing_immutable_audit_trail => fail_closed_missing_immutable_audit_trail
freeze_lifecycle_violation => fail_closed_freeze_lifecycle_violation
missing_emergency_stop => fail_closed_missing_emergency_stop
approval_authorizes_trading_operation => fail_closed_trading_authorization_violation
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
runtime_switch_enablement_allowed = false
candidate_operation_execution_allowed = false
approval_lifecycle_authorizes_trading_operations = false
```

## Boundary Statement

This lifecycle evidence can satisfy candidate review prerequisites only. It
does not authorize trading operation mutation, production runtime enablement,
automatic remediation, Dashboard/Admin/Trader Terminal controls, or backend
go-live.
