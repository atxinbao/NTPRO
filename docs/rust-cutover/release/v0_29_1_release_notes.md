# NTPRO Rust-only v0.29.1

Status: RELEASE GATE READY
Tag: `ntpro-rust-only-v0.29.1`
Release name: `NTPRO Rust-only v0.29.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.1`
Base release: `ntpro-rust-only-v0.29.0`
Date: 2026-07-11
Executor: Codex

v0.29.1 is a release governance and v0.30.0 start-gate hardening patch for the
published v0.29.0 Backend Production Readiness Foundation line.

Plain Chinese summary: v0.29.1 是 v0.29.0 发布后的治理补丁。它收口 v0.29.0 发布证据、
绑定 publish-after-gate 到当前 v29 release、清理陈旧 V290 证据、增加 post-publication
closeout gate，并把 v0.30.0 start gate 收紧为必须等待 v0.29.1 发布证据。它不新增
submit，不改变 runtime，不开放 backend go-live，不开放 Dashboard/Admin/Trader Terminal
交易控件，也不把系统声明为产品级实盘交易终端。

## Included Tasks

- `V291-001` - v0.29.0 release closeout evidence.
- `V291-002` - v29 publish-after-gate current-release binding.
- `V291-003` - stale V290 evidence cleanup.
- `V291-004` - post-publication closeout gate hardening.
- `V291-005` - v30 start gate hardening and dependency proof.
- `V291-006` - v29.1 release gates and strict provenance.

V291 final release scope issue count = 6
V291 final release scope evidence count = 6
V291 exact milestone issue set = #963-#968
V291 registered corrective-scope exception count = 0

## Release Gates

v29.1 release gates = required
v29.1 strict provenance = required
v29 release gates = required
v29 strict provenance = required
v30 start gate = hard-blocked until v0.29.1 publication evidence exists
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
generated publication evidence sole proof allowed = false
post-publication closeout evidence path = docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md

```text
scripts/ai/verify_v29_1_release_gates.sh
scripts/ai/verify_v29_1_strict_provenance.sh
scripts/ai/verify_v29_1_v30_start_gate.sh
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

v0.30.0 start gate = blocked until v0.29.1 release gate passes, all V291 issues
are closed, the `ntpro-rust-only-v0.29.1` GitHub Release is published after the
hosted release gate, and V300 intake evidence reconstructs the v0.29.1 tag,
hosted gate, release body/source hash, and source-controlled closeout evidence.
