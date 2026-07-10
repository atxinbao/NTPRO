# v0.29.1 Readiness Report

Date: 2026-07-11
Executor: Codex
Milestone: `ntpro-rust-only-v0.29.1`
Status: RELEASE GATE READY

## Summary

v0.29.1 is ready for tag-gate execution when V291-001 through V291-006 evidence
is present, all V291 issues are closed, v29.1 release gates pass, strict
provenance passes, and the release is published only after the hosted release
gate succeeds for the same tag commit.

Plain Chinese summary: v0.29.1 的范围是发布治理和 v30 start gate 硬化，不是 v0.30.0
功能。它要求 V291-001 到 V291-006 全部闭环，hosted release gate 成功后再公开 GitHub
Release。v0.30.0 在 v0.29.1 发布证据和 source-controlled closeout 证据存在前保持
hard-blocked。

## Evidence

V291-001 evidence = docs/rust-cutover/evidence/V291-001.md
V291-002 evidence = docs/rust-cutover/evidence/V291-002.md
V291-003 evidence = docs/rust-cutover/evidence/V291-003.md
V291-004 evidence = docs/rust-cutover/evidence/V291-004.md
V291-005 evidence = docs/rust-cutover/evidence/V291-005.md
V291-006 evidence = docs/rust-cutover/evidence/V291-006.md

## Gates

v29.1 release gates = required
v29.1 strict provenance = required
v29 release gates = required
v29 strict provenance = required
v30 start gate = hard-blocked until v0.29.1 publication evidence exists
release surface current guard = required
release publication guard = required
release publish after gate = required

```text
scripts/ai/verify_v29_1_release_gates.sh
scripts/ai/verify_v29_1_strict_provenance.sh
scripts/ai/verify_v29_1_v30_start_gate.sh
```

## Issue Closeout

#963 V291-001 = must be closed before v0.29.1 tag gate is accepted
#964 V291-002 = must be closed before v0.29.1 tag gate is accepted
#965 V291-003 = must be closed before v0.29.1 tag gate is accepted
#966 V291-004 = must be closed before v0.29.1 tag gate is accepted
#967 V291-005 = must be closed before v0.29.1 tag gate is accepted
#968 V291-006 = must be closed before v0.29.1 tag gate is accepted

## Release Scope

```text
V291 final release scope issue count = 6
V291 final release scope evidence count = 6
V291 exact milestone issue set = #963-#968
V291 registered corrective-scope exception count = 0
registered corrective-scope exceptions required = true
unregistered corrective milestone issues fail closed = true
v0.29.0 dependency proof = required
v0.29.0 release evidence = published
v0.30.0 start gate = blocked until v0.29.1 release evidence is published
strict provenance manifest = target/ntpro-v291/v0_29_1_strict_release_manifest.json
```

## Post-Publication Closeout Target

```text
release tag = ntpro-rust-only-v0.29.1
release name = NTPRO Rust-only v0.29.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.1
source-controlled closeout evidence = docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
v0.30.0 intake requires source-controlled v0.29.1 closeout = true
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
