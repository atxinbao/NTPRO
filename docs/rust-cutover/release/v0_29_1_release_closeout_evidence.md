# v0.29.1 Release Closeout Evidence

Date: 2026-07-11
Executor: Codex
Release: `ntpro-rust-only-v0.29.1`
Status: RELEASED

## Closeout Target

```text
release tag = ntpro-rust-only-v0.29.1
release name = NTPRO Rust-only v0.29.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.1
source-controlled closeout evidence = docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
release gate before publication required = true
publication after hosted gate required = true
same tag commit hosted gate required = true
v0.30.0 start rule = hard-blocked until v0.29.1 release evidence is published
```

## Published Evidence

```text
tag object SHA = d3d398530835342dab4aafe355d1c842be0fdd47
peeled tag commit = a831d802e4321f50ed6e10481aea35b15a74b01e
origin/main at publication = a831d802e4321f50ed6e10481aea35b15a74b01e
hosted release gate run = https://github.com/atxinbao/NTPRO/actions/runs/29130876713
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate conclusion = success
hosted release gate jobs = 90/90 success
hosted release gate completed at = 2026-07-11T01:06:27Z
GitHub Release published at = 2026-07-11T01:07:24Z
published after hosted gate = true
release body normalized sha256 = 611c6cfe89480054d5c3a4718215740701ee43536e3e92fa0ff458f7730b204b
release body raw sha256 = 5d5b7c34ceb7bca1a389e8261d04cc7fd28cea0a9d1e48ffe609f449b22ef2d1
tracked release notes normalized sha256 = 611c6cfe89480054d5c3a4718215740701ee43536e3e92fa0ff458f7730b204b
tracked release notes raw sha256 = 5d5b7c34ceb7bca1a389e8261d04cc7fd28cea0a9d1e48ffe609f449b22ef2d1
release body matches tracked release notes = true
release body acceptance rule = normalized_sha256
raw sha256 acceptance rule = false
v0.29.1 milestone = closed
v0.29.1 milestone open issues = 0
v0.29.1 milestone closed issues = 6
v0.30.0 start gate = ready
authoritative predecessor closeout contract = v0_29_1_authoritative_closeout_contract
manifest release_status = released
manifest published_release populated = true
manifest post_publication_closeout populated = true
```

## Issue Scope

```text
V291 final release issue set = 6/6 required
V291 exact milestone issue set = #963-#968
#963 V291-001 = closed
#964 V291-002 = closed
#965 V291-003 = closed
#966 V291-004 = closed
#967 V291-005 = closed
#968 V291-006 = closed
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
publication guard = pass
v29.1 v30 start gate = ready
v30 intake dependency proof = scripts/ai/verify_v30_intake_gate.sh
```
