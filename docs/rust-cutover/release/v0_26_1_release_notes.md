# NTPRO Rust-only v0.26.1

Status: RELEASED
Tag: `ntpro-rust-only-v0.26.1`
Release name: `NTPRO Rust-only v0.26.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.26.1`
Base release: `ntpro-rust-only-v0.26.0`
Date: 2026-07-07
Executor: Codex

v0.26.1 is a patch governance and evidence hardening release for the v0.26.0
Product Hardening Foundation.

Plain Chinese summary: v0.26.1 是 v0.26.0 之后的发布治理补丁。它收口 v0.26.0
release closeout、V260 final scope integration、stale V260 evidence cleanup、
v26 final scope release gate hardening、post-publication strict gate，以及
v26.1 release gates / strict provenance。它不新增 submit，不改变 runtime，不开放
Dashboard 交易控件，不把系统声明为产品级实盘交易终端。

## Included Tasks

- `V261-001` - v0.26.0 release closeout evidence backfill.
- `V261-002` - V260-009..V260-013 final release scope integration.
- `V261-003` - stale V260 release evidence cleanup.
- `V261-004` - v26 final scope release gate hardening.
- `V261-005` - post-publication strict gate and source closeout reconciliation.
- `V261-006` - v26.1 release gates and strict provenance.

## Release Gates

v26.1 release gates = required
v26.1 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
v27 intake gate = hard-blocked until v0.26.1 publication evidence exists
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true

```text
scripts/ai/verify_release.sh v26.1-release-gates
scripts/ai/verify_release.sh v26.1-strict-provenance
scripts/ai/verify_v26_1_release_gates.sh
scripts/ai/verify_v26_1_strict_provenance.sh
scripts/ai/verify_v27_intake_gate.sh
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
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Next Track

v0.27.0 start gate = blocked until v0.26.1 release gate passes, the
`ntpro-rust-only-v0.26.1` GitHub Release is published after the hosted gate, and
V270 intake evidence records that dependency proof.
