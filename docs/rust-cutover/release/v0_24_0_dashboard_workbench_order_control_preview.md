# NTPRO v0.24.0 Dashboard Workbench Order-Control Preview

Date: 2026-07-04
Executor: Codex
Task: `V240-008` / GitHub issue `#751`
Milestone: `v0.24.0`

## Summary

This document defines the read-only Dashboard / Workbench preview contract for
v0.24.0 order-control evidence. The panel displays order intent, execution
policy, rate-limit, slicing, cancel/replace/amend preview, retry/no-retry,
readback/audit, blocked reasons, scope keys, source provenance, and redaction
state. It does not expose live operation controls or live control APIs.

Plain Chinese summary: 这是 v0.24.0 order-control preview 的 Dashboard /
Workbench 只读展示合约。页面可以展示 intent、policy、rate-limit、slicing、
cancel/replace/amend、retry/no-retry、readback/audit、blocked reason、scope、
provenance 和 redaction；但不会开放 submit/cancel/replace/amend/flatten/order
ticket，也不会新增 live control API。

## Contract Identity

```text
schema_version = ntpro.v240_dashboard_workbench_order_control_preview.v1
contract_id = ntpro.v240_dashboard_workbench_order_control_preview.v1
contract_status = read_only_dashboard_preview_no_operation_controls
start_gate_dependency = scripts/ai/verify_release.sh v24-readback-audit-evidence
render_fixture = tests/golden/v240_dashboard_workbench_order_control_preview.json
```

## Render Coverage

```text
normal_case = v240-dashboard-case-normal
blocked_case = v240-dashboard-case-blocked
missing_provenance_case = v240-dashboard-case-missing-provenance
forbidden_control_case = v240-dashboard-case-forbidden-control
```

## Preview States

```text
ready_preview = evidence complete and readonly boundary locked
blocked = policy or risk block is displayed without operation controls
degraded_unavailable = missing preview evidence displayed as degraded unavailable
fail_closed = forbidden control marker displayed without exposing controls
```

The Dashboard must not turn missing provenance or missing preview evidence into
a ready state. Forbidden control evidence is displayed as failure evidence only.

## Read-Only Operation Boundary

```text
dashboard_submit_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_replace_controls_enabled = false
dashboard_amend_controls_enabled = false
dashboard_flatten_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_entry_enabled = false
manual_operation_submit_allowed = false
manual_operation_cancel_allowed = false
manual_operation_replace_allowed = false
manual_operation_amend_allowed = false
manual_operation_flatten_allowed = false
automatic_operation_action_allowed = false
live_control_api_added = false
network_attempted = false
execution_adapter_call_allowed = false
production_order_mutation_allowed = false
```

## Forbidden Render Surfaces

```text
button_element_allowed = false
form_element_allowed = false
input_element_allowed = false
fetch_call_allowed = false
dashboard_action_attribute_allowed = false
workbench_action_attribute_allowed = false
order_action_endpoint_allowed = false
submit_order_marker_allowed = false
cancel_order_marker_allowed = false
replace_order_marker_allowed = false
amend_order_marker_allowed = false
flatten_position_marker_allowed = false
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v24-dashboard-workbench-preview
```

The gate validates the v24 readback/audit prerequisite, render fixture schema,
four Dashboard render cases, explicit false operation boundaries, missing
provenance degraded-unavailable behavior, forbidden-control fail-closed
evidence, and the absence of real operation controls in the relevant Workbench
and runtime renderers.
