# NTPRO Rust-only v0.28.1

Status: RELEASE GATE READY
Tag: `ntpro-rust-only-v0.28.1`
Release name: `NTPRO Rust-only v0.28.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.1`
Base release: `ntpro-rust-only-v0.28.0`
Date: 2026-07-09
Executor: Codex

v0.28.1 is a patch governance and provenance hardening release for the v0.28.0
Backend Closure / Product Operations Runtime Finalization line.

Plain Chinese summary: v0.28.1 是 v0.28.0 之后的发布治理补丁。它收口 v0.28.0
发布证据、清理陈旧 V280/V271 证据、规范 release body hash、明确 runtime-closed 仅表示
deterministic artifact replay closure，并把 release-publish-after-gate 绑定到当前
v0.28.0 release proof。它新增 v28.1 release gates / strict provenance 和 v29 intake
dependency target，但不新增 submit，不改变 runtime，不开放 Dashboard/Admin/Trader Terminal
交易控件，不把系统声明为产品级实盘交易终端。

## Included Tasks

- `V281-001` - v0.28.0 release closeout evidence backfill.
- `V281-002` - stale V280-009 evidence cleanup.
- `V281-003` - v0.27.1 base release closeout reconciliation.
- `V281-004` - release body hash normalization contract.
- `V281-005` - runtime-closed terminology hardening.
- `V281-006` - release-publish-after-gate current-release binding.
- `V281-007` - v28.1 release gates and post-publication strict provenance.
- `V281-008` - v28.1 release tag gate prepublication publish-after-gate fix.
- `V281-009` - v28.1 prepublish live-current require semantics.

## Release Gates

v28.1 release gates = required
v28.1 strict provenance = required
v29 intake gate = hard-blocked until v0.28.1 publication evidence exists
v28 release gates = required
v28 strict provenance = required
release body hash normalization = required
runtime-closed terminology hardening = required
release publish after gate current-release binding = required
release surface current guard = required
release publication guard = required
release publish after gate = required
post-publication closeout evidence path = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true

```text
scripts/ai/verify_release.sh v28.1-release-gates
scripts/ai/verify_release.sh v28.1-strict-provenance
scripts/ai/verify_release.sh v29-intake-gate
scripts/ai/verify_v28_1_release_gates.sh
scripts/ai/verify_v28_1_strict_provenance.sh
scripts/ai/verify_v29_intake_gate.sh
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
network_attempted = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Next Track

v0.29.0 start gate = blocked until v0.28.1 release gate passes, all V281 issues
are closed, the `ntpro-rust-only-v0.28.1` GitHub Release is published after the
hosted release gate, and V290 intake evidence reconstructs the v0.28.1 tag,
hosted gate, release body/source hash, and source-controlled closeout evidence.
