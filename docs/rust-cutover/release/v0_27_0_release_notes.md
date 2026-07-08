# NTPRO Rust-only v0.27.0

Status: RELEASED
Tag: `ntpro-rust-only-v0.27.0`
Release name: `NTPRO Rust-only v0.27.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.0`
Base release: `ntpro-rust-only-v0.26.1`
Date: 2026-07-08
Executor: Codex

v0.27.0 publishes the Product Operations Runtime Integration Foundation.

Plain Chinese summary: v0.27.0 是 Product Operations Runtime Integration
Foundation。它完成 v0.26.1 dependency proof、product operations boundary、
external identity/permission foundation、persistent audit storage foundation、
deployment/upgrade/rollback orchestration foundation、long-run telemetry/SLO
evidence、Admin Workbench runtime state bridge、runtime integration fail-closed
hardening，以及 v27 release gates / strict provenance。它不新增默认下单，不改变
生产订单，不发送 adapter request，不触发 live exchange request，不启用 retry
scheduler，不执行 automatic remediation，不开放 Dashboard/Admin 交易控件，也不把
系统声明为产品级实盘交易终端。

## Included Tasks

- `V270-000` - v0.27.0 intake gate and v0.26.1 dependency proof.
- `V270-001` - product operations runtime integration boundary contract.
- `V270-002` - external identity and permission integration foundation.
- `V270-003` - persistent operation audit storage integration foundation.
- `V270-004` - deployment upgrade rollback runtime orchestration foundation.
- `V270-005` - long-run telemetry ingestion and SLO runtime evidence.
- `V270-006` - Admin Workbench runtime state bridge read-only surface.
- `V270-007` - runtime integration fail-closed and no-trading-control hardening.
- `V270-008` - v27 release gates and strict provenance.

## Release Gates

V270 final release scope issue count = 9
V270 final release scope evidence count = 9
v27 release gates = required
v27 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
hosted release gate success before public GitHub Release = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true

```text
scripts/ai/verify_release.sh v27-release-gates
scripts/ai/verify_release.sh v27-strict-provenance
scripts/ai/verify_v27_release_gates.sh
scripts/ai/verify_v27_strict_provenance.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Next Track

The next patch track is `v0.27.1`. The next capability track is `v0.28.0`.
Neither track inherits production submit, production mutation, adapter send,
live exchange request, retry scheduling, automatic remediation, Dashboard /
Admin trading controls, or product-grade live terminal claims from v0.27.0.
