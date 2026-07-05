# NTPRO Rust-only v0.24.1 Release Notes

Date: 2026-07-05
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.24.1`
Release name: `NTPRO Rust-only v0.24.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.24.1`
Base release: `ntpro-rust-only-v0.24.0`

## Summary

v0.24.1 is a hardening patch for the v0.24.0 Execution Algorithms And Order
Control Foundation release. It closes the post-release governance gaps around
v0.24.0 release closeout, tag/main/release-body provenance reconciliation,
stale pre-tag evidence cleanup, v24 schema replay classification, Dashboard
artifact ingestion hardening, Dashboard fixture reference integrity, v24.1
release gates, and strict provenance.

This release does not add submit capability.
This release is not a product-grade live trading terminal.
It does not open production order mutation, execution adapter send, live
exchange request, strategy-driven production execution, implicit retry, retry
scheduler, automatic cancel, automatic remediation, cancel/replace/amend send,
flatten, shared approval consumption, or Dashboard operation controls.

Plain Chinese summary: v0.24.1 只是 v0.24.0 的发布治理和 Dashboard 证据硬化补丁。
它补齐 release closeout、provenance、schema replay 分类、artifact ingestion、
fixture ref integrity 和 release gate；但它不发送真实订单、不撤单、不改单、不
flatten、不启用 retry scheduler，也不把 Workbench 声称为产品级实盘交易终端。

## Included

- `V241-001` - v0.24.0 release closeout evidence backfill.
- `V241-002` - tag/main/release-body provenance reconciliation.
- `V241-003` - V240-009 stale pre-tag evidence cleanup.
- `V241-004` - v24 schema-only trace classification and executable replay advancement.
- `V241-005` - Dashboard artifact ingestion hardening.
- `V241-006` - v24.1 release gates and strict provenance.
- `V241-007` - Dashboard fixture ref integrity and policy ref hardening.

## Release Gates

```text
v24.1 release closeout evidence = required
v24.1 provenance reconciliation = required
v24.1 stale pre-tag cleanup = required
v24.1 schema replay classification = required
v24.1 Dashboard artifact ingestion = required
v24.1 Dashboard fixture ref integrity = required
v24.1 release gates = required
v24.1 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
```

Commands:

```text
scripts/ai/verify_release.sh v24.1-release-closeout-evidence
scripts/ai/verify_release.sh v24.1-provenance-reconciliation
scripts/ai/verify_release.sh v24.1-stale-pretag-cleanup
scripts/ai/verify_release.sh v24.1-schema-replay-classification
scripts/ai/verify_release.sh v24.1-dashboard-artifact-ingestion
scripts/ai/verify_release.sh v24.1-dashboard-fixture-ref-integrity
scripts/ai/verify_release.sh v24.1-release-gates
scripts/ai/verify_release.sh v24.1-strict-provenance
scripts/ai/verify_v24_1_release_gates.sh
scripts/ai/verify_v24_1_strict_provenance.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

```text
patch_hardening_only = true
release_governance_hardening = true
strict provenance = required
gate_before_publish = required
local generated publication evidence required in source tree = false
remote reconstruction required = true
v0.25.0 start gate = blocked until v0.24.1 release evidence is published
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
network_attempted = false
implicit_retry_allowed = false
retry_scheduler_enabled = false
cancel_replace_amend_send_allowed = false
flatten_allowed = false
dashboard_operation_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Not Included

- product-grade live trading terminal;
- complete executable order-control runtime;
- real submit/cancel/replace/amend/flatten;
- execution adapter or exchange network send;
- retry scheduler or implicit retry;
- automatic cancel, retry, remediation, repair, alert, audit, provenance, risk,
  or operation action;
- strategy-driven production execution;
- shared approval consumption;
- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls;
- v0.25.0 monitoring / incident / disaster-recovery implementation.

## Next Boundary

`v0.25.0` is reserved for the monitoring / incident / disaster-recovery track.
It remains blocked until the `ntpro-rust-only-v0.24.1` tag exists, the hosted
release gate succeeds for that tag commit, the GitHub Release is published
after the hosted gate, V241-001 through V241-007 issues are closed, the
`v0.24.1` milestone is closed, and strict provenance evidence can reconstruct
the source tree plus GitHub remote state.
