# v0.23.0 Dashboard Observability Surface

Date: 2026-07-03
Executor: Codex
Task: `V230-006`
GitHub issue: `#717`
Milestone: `v0.23.0`
Status: LOCAL VALIDATION PASSED

## Summary

V230-006 adds executable Rust replay coverage and a real Dashboard render smoke
for multi-account, multi-strategy, and multi-venue node observability. The
surface remains read-only: it can aggregate, filter, and drill down through
scoped rows, but operation controls remain absent or explicitly disabled.

Plain Chinese summary: 本任务不是发布产品级交易终端，也不是开放真实操作按钮。它把
Dashboard / Workbench 的多维观测能力固定为只读：每一行都必须带 account、strategy、
venue node、isolation scope 和 provenance 标签；过滤和 drill-down 只能显示对应 scope；
标签串线会 fail closed；缺失身份会降级 unavailable。submit/cancel/retry/replace/
amend/flatten、order ticket、manual operation、production mutation 和产品级终端声明仍
全部禁用。

## Contract Traceability

```text
V230-001 Dashboard Boundary = satisfied by per-row visible labels, read-only aggregation, scoped filters, and forbidden-controls assertions
V230-001 Allowed Read Paths = satisfied by cross-node read-model aggregation with isolation_scope_key preserved per row
V230-001 Logs And Evidence Boundary = satisfied by source_provenance assertions and release replay scope entries
```

## Dashboard Boundary

```text
dashboard_read_only_aggregation = allowed
dashboard_filter_by_account_key = required
dashboard_filter_by_strategy_key = required
dashboard_filter_by_venue_node_key = required
dashboard_isolation_scope_key_per_row = required
dashboard_missing_identity_behavior = degraded_unavailable
dashboard_cross_scope_label_mismatch = fail_closed
dashboard_operation_controls_enabled = false
dashboard_submit_cancel_retry_replace_amend_flatten_controls = forbidden
trader_terminal_order_ticket_enabled = false
manual_operation_entry_enabled = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
product_grade_trading_terminal_claim = false
```

## Executable Replay Coverage

```text
read_model.dashboard_observability.multi_scope_readonly.001 = PASS path, two scoped rows aggregate with visible account/strategy/venue/scope labels
read_model.dashboard_observability.filtered_drilldown_isolated.001 = PASS path, account/strategy/venue filters display one isolated scope and block requested controls
read_model.dashboard_observability.cross_scope_label_mismatch.001 = FAIL-CLOSED path, visible labels point at a different strategy scope
read_model.dashboard_observability.missing_identity_degraded.001 = DEGRADED path, missing identity labels degrade unavailable
```

## Render Smoke Coverage

```text
fixture = tests/golden/v230/dashboard_observability_snapshot.json
script = scripts/ai/verify_v23_dashboard_observability_smoke.sh
rows = 2
required_false_operation_boundary_fields = 21
renderer_path = crates/cli/src/dashboard.rs DASHBOARD_JS renderTraderTerminalWorkbench + renderReadModelRuntime
forbidden_action_surfaces = button/form/input/fetch/dashboard-action/workbench-action/control-api/submit/cancel/replace/amend/flatten
release_provenance = ntpro-rust-only-v0.23.0
```

## Non-Goals Confirmed

```text
release_publication = recorded_by_v0_23_0_release_closeout
production_submit_enabled = false
production_order_mutation_enabled = false
manual_operation_enabled = false
order_ticket_enabled = false
dashboard_mutation_controls_enabled = false
marketing_ui_work = false
```

## Evidence

Validation is recorded in `docs/rust-cutover/evidence/V230-006.md` and
`verification.md`. The release replay manifest records all four Dashboard
observability cases as `executable_replay` with
`release_decision = included_in_final_replay_scope`.
