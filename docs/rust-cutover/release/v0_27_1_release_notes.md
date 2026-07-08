# NTPRO Rust-only v0.27.1

Status: RELEASED
Tag: `ntpro-rust-only-v0.27.1`
Release name: `NTPRO Rust-only v0.27.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.1`
Base release: `ntpro-rust-only-v0.27.0`
Date: 2026-07-08
Executor: Codex

v0.27.1 is a patch governance and evidence hardening release for the v0.27.0
Product Operations Runtime Integration Foundation.

Plain Chinese summary: v0.27.1 是 v0.27.0 之后的发布治理补丁。它收口 v0.27.0
release closeout、publication entry provenance、stale V270 evidence cleanup、
v0.26.1 #868 dependency reconciliation、exact release scope gate hardening，以及
v27.1 release gates / strict provenance。它不新增 submit，不改变 runtime，不开放
Dashboard/Admin 交易控件，不把系统声明为产品级实盘交易终端。

## Included Tasks

- `V271-001` - v0.27.0 release closeout evidence backfill.
- `V271-002` - v0.27.0 publication entry provenance.
- `V271-003` - stale V270 evidence cleanup.
- `V271-004` - v0.26.1 dependency closeout reconciliation.
- `V271-005` - exact release scope gate hardening.
- `V271-006` - v27.1 release gates and post-publication strict provenance.

## Release Gates

v27.1 release gates = required
v27.1 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
v28 intake gate = hard-blocked until v0.27.1 publication evidence exists
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true

```text
scripts/ai/verify_release.sh v27.1-release-gates
scripts/ai/verify_release.sh v27.1-strict-provenance
scripts/ai/verify_v27_1_release_gates.sh
scripts/ai/verify_v27_1_strict_provenance.sh
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

v0.28.0 start gate = blocked until v0.27.1 release gate passes, the
`ntpro-rust-only-v0.27.1` GitHub Release is published after the hosted gate, and
V280 intake evidence records that dependency proof.
