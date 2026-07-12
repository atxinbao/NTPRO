# v0.30.1 Release Closeout Evidence

Date: 2026-07-12
Executor: Codex
Release: `ntpro-rust-only-v0.30.1`
Status: RELEASE GATE READY

## Closeout Target

```text
release tag = ntpro-rust-only-v0.30.1
release name = NTPRO Rust-only v0.30.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.30.1
source-controlled closeout evidence = docs/rust-cutover/release/v0_30_1_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
release gate before publication required = true
publication after hosted gate required = true
same tag commit hosted gate required = true
v0.31.0 start rule = hard-blocked until v0.30.1 release evidence is published
```

## Pre-Publication Gate Facts

```text
published release evidence = pending public release
hosted release gate = required after tag
GitHub Release published at = pending public release
published after hosted gate = required
release body hash semantics = normalized_sha256
v0.30.1 milestone = must be closed before tag gate
v0.31.0 start gate = blocked_until_v301_release_evidence_published
v0.31.0 start gate contract = docs/rust-cutover/release/v0_30_1_v31_start_gate.json
```

## Issue Scope

```text
V301 final release issue set = 7/7 required
V301 exact milestone issue set = #999-#1005
#999 V301-001 = must be closed before tag gate
#1000 V301-002 = must be closed before tag gate
#1001 V301-003 = must be closed before tag gate
#1002 V301-004 = must be closed before tag gate
#1003 V301-005 = must be closed before tag gate
#1004 V301-006 = must be closed before tag gate
#1005 V301-007 = must be closed before tag gate
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

## Verification

```text
publication guard = prepublish tag gate required
v30.1 v31 start gate = blocked until v0.30.1 release evidence is published
v31 dependency proof = scripts/ai/verify_v30_1_v31_start_gate.sh
```
