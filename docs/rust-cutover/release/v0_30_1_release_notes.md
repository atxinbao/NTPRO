# NTPRO Rust-only v0.30.1

Status: RELEASE GATE READY
Tag: `ntpro-rust-only-v0.30.1`
Release name: `NTPRO Rust-only v0.30.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.30.1`
Base release: `ntpro-rust-only-v0.30.0`
Date: 2026-07-12
Executor: Codex

v0.30.1 is a release governance and v0.31.0 start-gate hardening patch for the
published v0.30.0 Backend Production Go-Live Candidate Foundation line.

Plain Chinese summary: v0.30.1 是 v0.30.0 发布后的治理补丁。它收口 v0.30.0 发布证据、
publish-after-gate current binding、post-publication closeout、V300 stale
evidence、v0.29.1 predecessor closeout、v31 start gate 和最终 v30.1 release gate。
它不新增 submit，不改变 runtime，不开放 backend go-live，不开放 Dashboard/Admin/Trader
Terminal 交易控件，也不把系统声明为产品级实盘交易终端。

## Included Tasks

- `V301-001` - v0.30.0 release closeout evidence.
- `V301-002` - v30 publish-after-gate current-release binding.
- `V301-003` - v30 post-publication closeout gate hardening.
- `V301-004` - stale V300 evidence cleanup.
- `V301-005` - v0.29.1 post-publication reconciliation.
- `V301-006` - v31 start gate hardening and dependency proof.
- `V301-007` - v30.1 release gates and strict provenance.

V301 final release scope issue count = 7
V301 final release scope evidence count = 7
V301 exact milestone issue set = #999-#1005
V301 registered corrective-scope exception count = 0

## Release Gates

v30.1 release gates = required
v30.1 strict provenance = required
v30 release gates = required
v30 strict provenance = required
v31 start gate = hard-blocked until v0.30.1 publication evidence exists
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
generated publication evidence sole proof allowed = false
post-publication closeout evidence path = docs/rust-cutover/release/v0_30_1_release_closeout_evidence.md
v0.31.0 start gate contract = docs/rust-cutover/release/v0_30_1_v31_start_gate.json

```text
scripts/ai/verify_v30_1_release_gates.sh
scripts/ai/verify_v30_1_strict_provenance.sh
scripts/ai/verify_v30_1_v31_start_gate.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
cancel_order_allowed = false
replace_order_allowed = false
amend_order_allowed = false
flatten_position_allowed = false
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
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
```

## Next Track

v0.31.0 start gate = blocked until v0.30.1 release gate passes, all V301 issues
are closed, the `ntpro-rust-only-v0.30.1` GitHub Release is published after the
hosted release gate, and V31 intake evidence reconstructs the v0.30.1 tag,
hosted gate, release body/source hash, source-controlled closeout target, and
explicit scoped approval boundary.
