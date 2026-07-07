# NTPRO Rust-only v0.26.0

Status: RELEASED
Tag: `ntpro-rust-only-v0.26.0`
Release name: `NTPRO Rust-only v0.26.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.26.0`
Base release: `ntpro-rust-only-v0.25.1`
Date: 2026-07-06
Executor: Codex

v0.26.0 publishes the Product Hardening Foundation. It closes the v26 release
governance boundary for product hardening scope, operator permission evidence,
operation audit evidence, deployment provenance, upgrade/rollback runbook
evidence, SLO/runbook stability evidence, read-only admin Dashboard surface,
v26 release gates, and strict provenance.

Plain Chinese summary: v0.26.0 发布的是 Product Hardening Foundation。它补齐
product hardening boundary、operator permission、operation audit、deployment
provenance、upgrade/rollback runbook、SLO/runbook stability、只读/admin
Dashboard、release gate 和 strict provenance；但它不发送真实订单、不撤单、不改单、
不 flatten、不启用 retry scheduler 或 automatic remediation，也不把 Dashboard /
Trader Terminal 声称为产品级实盘交易终端。

This release does not add submit capability.
This release is not a product-grade live trading terminal.
It does not open production order mutation, execution adapter send, live
exchange request, strategy-driven production execution, implicit retry, retry
scheduler, automatic remediation, automatic recovery, or Dashboard trading
controls.

## Included Tasks

- `V260-000` - v0.26.0 intake gate and v0.25.1 dependency proof.
- `V260-001` - product hardening boundary contract.
- `V260-002` - operator permission model and role boundary evidence.
- `V260-003` - operation audit trail and immutable action evidence.
- `V260-004` - deployment topology and environment provenance model.
- `V260-005` - upgrade rollback and release operation runbook evidence.
- `V260-006` - SLO runbook productization and long-run stability evidence.
- `V260-007` - Dashboard product hardening read-only admin boundary surface.
- `V260-008` - v26 release gates and strict provenance.

Corrective release-publication tasks:

- `V260-009` - release golden trace file gate correction.
- `V260-010` - nested historical release gate correction.
- `V260-011` - release publication guard notes line correction.

## Release Gates

```text
v26 intake gate = required
v26 product hardening boundary contract = required
v26 operator permission model = required
v26 operation audit trail = required
v26 deployment provenance model = required
v26 upgrade rollback runbook evidence = required
v26 SLO runbook stability evidence = required
v26 Dashboard admin boundary surface = required
v26 release gates = required
v26 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
hosted release gate success before public GitHub Release = required
```

Commands:

```text
scripts/ai/verify_release.sh v26-intake-gate
scripts/ai/verify_release.sh v26-product-hardening-boundary-contract
scripts/ai/verify_release.sh v26-operator-permission-model
scripts/ai/verify_release.sh v26-operation-audit-trail
scripts/ai/verify_release.sh v26-deployment-provenance-model
scripts/ai/verify_release.sh v26-upgrade-rollback-runbook-evidence
scripts/ai/verify_release.sh v26-slo-runbook-stability-evidence
scripts/ai/verify_release.sh v26-dashboard-admin-boundary-surface
scripts/ai/verify_release.sh v26-release-gates
scripts/ai/verify_release.sh v26-strict-provenance
scripts/ai/verify_v26_release_gates.sh
scripts/ai/verify_v26_strict_provenance.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

```text
product_hardening_foundation = true
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
- automatic remediation, recovery, retry, repair, alert action, audit action,
  provenance action, or risk action;
- strategy-driven production execution;
- shared approval consumption;
- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, remediation, or order-ticket controls.

## Next Boundary

The next patch track is `v0.26.1`.
The next capability track is `v0.27.0`.
`v0.27.0` must not inherit production submit, mutation, adapter send, live
exchange request, retry scheduler, automatic remediation, or Dashboard trading
controls from v0.26.0 without a separate issue, branch, PR, gate, and release
boundary.
