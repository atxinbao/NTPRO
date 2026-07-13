# v0.31.0 Production Enablement Boundary Contract

Date: 2026-07-13
Executor: Codex
Task: `V310-001` / GitHub issue `#1007`
Milestone: `v0.31.0`

## Goal

Define the v0.31.0 production enablement boundary and the explicit scoped
approval contract. This is a controlled backend production enablement candidate
contract only; it is not backend go-live and not runtime trading authority.

Plain Chinese summary: 本文档定义 v0.31.0 的生产启用边界和显式 scoped approval
合同。它只能记录“是否具备进入后续启用评估”的证据，不授权 submit、mutation、
adapter send、live exchange request、retry scheduler、automatic remediation、
Dashboard/Admin/Trader Terminal 交易控件、backend go-live 或产品级实盘交易终端声明。

## Boundary Definition

```text
boundary_status = scoped_approval_required_no_execution_authority
capability_track = controlled_backend_production_enablement_candidate
default_runtime_authority = none
approval_source_of_truth = source_controlled_artifact
chat_approval_allowed = false
external_notes_approval_allowed = false
runtime_execution_authorized_by_this_contract = false
```

## Explicit Scoped Approval

```text
explicit scoped approval required = true
missing scoped approval status = fail_closed_missing_scoped_approval
approval alone authorizes execution = false
allowed requested capability = backend_production_enablement_candidate_readiness_evaluation
forbidden requested capability = submit_order
forbidden requested capability = cancel_order
forbidden requested capability = replace_order
forbidden requested capability = amend_order
forbidden requested capability = flatten_position
forbidden requested capability = adapter_send
forbidden requested capability = live_exchange_request
forbidden requested capability = automatic_remediation
forbidden requested capability = dashboard_trading_control
forbidden requested capability = admin_workbench_trading_control
forbidden requested capability = trader_terminal_order_ticket
```

Required approval scope fields:

```text
approval_id
approver
operator
github_issue
release_version
environment
venue_scope
account_scope
change_window_id
requested_capability
request_digest
boundary_digest
issued_at
expires_at
```

## Non-Inheritance Boundary

```text
inherits_submit = false
inherits_mutation = false
inherits_adapter_send = false
inherits_live_exchange_request = false
inherits_retry_scheduler = false
inherits_automatic_remediation = false
inherits_dashboard_trading_controls = false
inherits_admin_workbench_trading_controls = false
inherits_trader_terminal_order_ticket = false
inherits_backend_go_live_claim = false
inherits_product_grade_live_trading_claim = false
```

## Runtime Boundary Flags

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
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
```

## Downstream Gates

Even when scoped approval is present, execution remains blocked until all later
v31 gates are satisfied:

```text
operator_freeze_change_window = required
risk_gate = required
audit_gate = required
go_no_go_record = required
canary_rollback_dr_boundary = required
config_venue_readiness = required
telemetry_slo_gate = required
fail_closed_negative_tests = required
v31_release_gates = required
```

## Deterministic Negative Cases

```text
missing scoped approval -> fail_closed_missing_scoped_approval
inherited submit -> fail_closed_inherited_execution_authority
inherited adapter send -> fail_closed_inherited_execution_authority
inherited trading controls -> fail_closed_inherited_execution_authority
scoped approval without downstream gates -> approval_recorded_execution_still_blocked_by_downstream_gates
contract satisfied -> boundary_contract_satisfied_no_runtime_execution
```

## Auditability

The boundary can be audited from source-controlled artifacts:

```text
source-controlled contract = docs/rust-cutover/release/v0_31_0_production_enablement_boundary_contract.json
source-controlled evidence = docs/rust-cutover/evidence/V310-001.md
chat or external notes sufficient = false
deterministic negative cases required = true
runtime behavior changed = false
trading behavior changed = false
```
