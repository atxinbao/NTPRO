# NTPRO Rust-only v0.25.0 Release Notes

Date: 2026-07-06
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.25.0`
Release name: `NTPRO Rust-only v0.25.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.25.0`
Base release: `ntpro-rust-only-v0.24.1`

## Summary

v0.25.0 publishes the Monitoring, Incident, and Disaster-Recovery Foundation.
It closes the v25 release governance boundary for intake, monitoring
observability, alert taxonomy/routing, incident lifecycle and acknowledgement,
runbook/audit evidence, DR preview drills, read-only Dashboard monitoring,
SLO/freshness diagnostics, v25 release gates, and strict provenance.

This release does not add submit capability.
This release is not a product-grade live trading terminal.
It does not open production order mutation, execution adapter send, live
exchange request, strategy-driven production execution, implicit retry, retry
scheduler, automatic remediation, automatic recovery, or Dashboard trading
controls.

Plain Chinese summary: v0.25.0 发布的是 monitoring / incident / DR foundation。
它补齐 intake、monitoring、alert、incident、runbook/audit、DR preview、只读
Dashboard、SLO/freshness、release gate 和 strict provenance；但它不发送真实订单、
不撤单、不改单、不 flatten、不启用 retry scheduler 或自动 remediation，也不把
Workbench 声称为产品级实盘交易终端。

## Included

- `V250-000` - v25 intake gate and v0.24.1 release evidence dependency.
- `V250-001` - monitoring observability contract.
- `V250-002` - alert taxonomy, routing, and evidence boundary.
- `V250-003` - incident lifecycle and operator acknowledgement evidence.
- `V250-004` - runbook audit evidence model.
- `V250-005` - disaster recovery preview drill evidence.
- `V250-006` - Dashboard monitoring / incident / DR read-only surface.
- `V250-007` - SLO freshness and fail-closed diagnostics gate.
- `V250-008` - v25 release gates and strict provenance.
- `V250-009` - corrective v25 tag-gate base provenance scoping fix for
  `#804` / PR `#805`; this is release governance only and does not expand v25
  capability.

## Corrective Release Scope

```text
V250 milestone issue scope = #777-#785
V250 milestone issue count = 9
V250 corrective issue scope = #804 / V250-009
V250 final release scope issue count = 10
V250 final release scope evidence count = 10
V250-009 failed release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28762387835
V250-009 final success release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28764231552
V250-009 PR = #805
V250-009 merge commit = eedcdab1d3ca85d6f51b368b5f36208a7b591026
V250-009 capability expansion = false
V250-009 runtime behavior change = false
V250-009 trading behavior change = false
```

## Release Gates

```text
v25 intake gate = required
v25 monitoring observability contract = required
v25 alert taxonomy routing = required
v25 incident lifecycle acknowledgement = required
v25 runbook audit evidence = required
v25 DR preview drill evidence = required
v25 Dashboard monitoring surface = required
v25 SLO freshness diagnostics gate = required
v25 release gates = required
v25 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
```

Commands:

```text
scripts/ai/verify_release.sh v25-intake-gate
scripts/ai/verify_release.sh v25-monitoring-observability-contract
scripts/ai/verify_release.sh v25-alert-taxonomy-routing
scripts/ai/verify_release.sh v25-incident-lifecycle-acknowledgement
scripts/ai/verify_release.sh v25-runbook-audit-evidence
scripts/ai/verify_release.sh v25-dr-preview-drill-evidence
scripts/ai/verify_release.sh v25-dashboard-monitoring-surface
scripts/ai/verify_release.sh v25-slo-freshness-diagnostics-gate
scripts/ai/verify_release.sh v25-release-gates
scripts/ai/verify_release.sh v25-strict-provenance
scripts/ai/verify_v25_release_gates.sh
scripts/ai/verify_v25_strict_provenance.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

```text
monitoring_incident_dr_foundation = true
release_governance = true
strict_provenance = true
gate_before_publish = required
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
network_attempted = false
implicit_retry_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Not Included

- product-grade live trading terminal;
- real submit/cancel/replace/amend/flatten;
- production order mutation;
- execution adapter or exchange network send;
- retry scheduler or implicit retry;
- automatic remediation, recovery, retry, repair, alert action, audit action, provenance action, or risk action;
- strategy-driven production execution;
- shared approval consumption;
- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, remediation, or order-ticket controls.

## Next Boundary

The next patch track is `v0.25.1`.
The next capability track is `v0.26.0`.
`v0.26.0` must not inherit production submit, mutation, adapter send, live
exchange request, retry scheduler, automatic remediation, or Dashboard trading
controls from v0.25.0 without a separate issue, branch, PR, gate, and release
boundary.
