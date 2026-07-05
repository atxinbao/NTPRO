# NTPRO Rust-only v0.24.0 Release Notes

Date: 2026-07-05
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.24.0`
Release name: `NTPRO Rust-only v0.24.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.24.0`

## Summary

`v0.24.0` is the Execution Algorithms And Order Control Foundation release. It
publishes preview-only order-control evidence for dependency intake, the
execution/order-control contract, order intent and policy artifacts,
rate-limit/throttle gates, order slicing preview, cancel/replace/amend preview,
retry/no-retry ledger, readback/audit evidence, Dashboard Workbench read-only
preview, release gates, and strict provenance.

This release does not add submit capability.
This release is not a product-grade live trading terminal.
It does not open production order mutation, execution adapter send, live
exchange request, strategy-driven production execution, implicit retry, retry
scheduler, automatic cancel, automatic remediation, cancel/replace/amend send,
flatten, shared approval consumption, or Dashboard operation controls.

Plain Chinese summary: v0.24.0 只发布 order-control foundation 的只读 preview
证据链。它把 order intent、policy、rate-limit、slicing、cancel/replace/amend、
retry/no-retry、readback/audit 和 Dashboard Workbench 预览都纳入 release gate；
但它不发送真实订单、不撤单、不改单、不 flatten、不启用 retry scheduler，也不把
Workbench 声称为产品级实盘交易终端。

## Included

- `V240-000` - v0.24.0 intake gate and v0.23.1 dependency proof.
- `V240-001` - execution and order-control contract.
- `V240-002` - order intent and execution policy model.
- `V240-003` - rate-limit and throttle gate preview.
- `V240-004` - deterministic order slicing preview foundation.
- `V240-005` - cancel / replace / amend no-send preview contract.
- `V240-006` - retry / no-retry policy ledger.
- `V240-007` - readback and audit evidence for order-control preview.
- `V240-008` - Dashboard Workbench read-only order-control preview.
- `V240-009` - v0.24.0 release gates and strict provenance.

## Release Gates

```text
v24 intake gate = required
v24 order-control contract = required
v24 order intent policy = required
v24 rate-limit throttle gate = required
v24 order slicing preview = required
v24 cancel replace amend preview = required
v24 retry policy ledger = required
v24 readback audit evidence = required
v24 Dashboard Workbench preview = required
v24 release gates = required
v24 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
```

Commands:

```text
scripts/ai/verify_release.sh v24-intake-gate
scripts/ai/verify_release.sh v24-order-control-contract
scripts/ai/verify_release.sh v24-order-intent-policy
scripts/ai/verify_release.sh v24-rate-limit-throttle-gate
scripts/ai/verify_release.sh v24-order-slicing-preview
scripts/ai/verify_release.sh v24-cancel-replace-amend-preview
scripts/ai/verify_release.sh v24-retry-policy-ledger
scripts/ai/verify_release.sh v24-readback-audit-evidence
scripts/ai/verify_release.sh v24-dashboard-workbench-preview
scripts/ai/verify_release.sh v24-release-gates
scripts/ai/verify_release.sh v24-strict-provenance
scripts/ai/verify_v24_release_gates.sh
scripts/ai/verify_v24_strict_provenance.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

```text
order_control_foundation_preview_only = true
preview_evidence_only = true
strict provenance = required
gate_before_publish = required
publication evidence strategy = source_tree_plus_github_remote
v0.25.0 start gate = blocked until v0.24.0 release evidence is published
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
It remains blocked until the `ntpro-rust-only-v0.24.0` tag exists, the hosted
release gate succeeds for that tag commit, the GitHub Release is published
after the hosted gate, V240-000 through V240-009 issues are closed, the
`v0.24.0` milestone is closed, and strict provenance evidence can reconstruct
the source tree plus GitHub remote state.
