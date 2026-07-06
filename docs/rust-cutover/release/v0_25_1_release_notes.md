# NTPRO Rust-only v0.25.1

Status: RELEASED
Tag: `ntpro-rust-only-v0.25.1`
Release name: `NTPRO Rust-only v0.25.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.25.1`
Base release: `ntpro-rust-only-v0.25.0`
Date: 2026-07-06
Executor: Codex

v0.25.1 is a patch governance and evidence hardening release for the v0.25.0
Monitoring, Incident, and Disaster-Recovery Foundation.

Plain Chinese summary: v0.25.1 是 v0.25.0 之后的发布治理补丁。它收口 v0.25.0
release closeout、V250-009 corrective scope、V250-008 stale evidence cleanup、
Dashboard source_ref integrity、post-release gate split，以及 v25.1 release
gates / strict provenance。它不新增 submit，不改变 runtime，不开放 Dashboard 交易控件，
不把系统声明为产品级实盘交易终端。

## Included Tasks

- `V251-001` - v0.25.0 release closeout evidence backfill.
- `V251-002` - V250-009 corrective release scope integration.
- `V251-003` - V250-008 stale pre-tag evidence cleanup.
- `V251-004` - v25 Dashboard source_ref path anchor integrity gate.
- `V251-005` - post-release gate split and closeout hardening.
- `V251-006` - v25.1 release gates and strict provenance.

## Release Gates

v25.1 release gates = required
v25.1 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true

```text
scripts/ai/verify_release.sh v25.1-release-gates
scripts/ai/verify_release.sh v25.1-strict-provenance
scripts/ai/verify_v25_1_release_gates.sh
scripts/ai/verify_v25_1_strict_provenance.sh
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

v0.26.0 start gate = blocked until v0.25.1 release gate passes, the
`ntpro-rust-only-v0.25.1` GitHub Release is published after the hosted gate, and
V260 intake evidence records that dependency proof.
