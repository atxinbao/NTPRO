# v0.30.0 Readiness Report

Date: 2026-07-11
Executor: Codex
Milestone: `ntpro-rust-only-v0.30.0`
Status: RELEASE GATE READY

## Summary

v0.30.0 is ready for tag-gate execution when V300-000 through V300-011 evidence
is present, all V300 issues are closed, v30 release gates pass, strict
provenance passes, and the release is published only after the hosted release
gate succeeds for the same tag commit.

Plain Chinese summary: v0.30.0 是 Backend Production Go-Live Candidate
Foundation。它要求 V300-000 到 V300-011 全部闭环，hosted release gate 成功后再公开
GitHub Release。Ready 表示 candidate-ready，不表示 actual production go-live；v31
生产启用仍需独立 scoped issue 和显式审批。

## Evidence

V300-000 evidence = docs/rust-cutover/evidence/V300-000.md
V300-001 evidence = docs/rust-cutover/evidence/V300-001.md
V300-002 evidence = docs/rust-cutover/evidence/V300-002.md
V300-003 evidence = docs/rust-cutover/evidence/V300-003.md
V300-004 evidence = docs/rust-cutover/evidence/V300-004.md
V300-005 evidence = docs/rust-cutover/evidence/V300-005.md
V300-006 evidence = docs/rust-cutover/evidence/V300-006.md
V300-007 evidence = docs/rust-cutover/evidence/V300-007.md
V300-008 evidence = docs/rust-cutover/evidence/V300-008.md
V300-009 evidence = docs/rust-cutover/evidence/V300-009.md
V300-010 evidence = docs/rust-cutover/evidence/V300-010.md
V300-011 evidence = docs/rust-cutover/evidence/V300-011.md

## Gates

v30 release gates = required
v30 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
v31 production enablement handoff = hard-blocked by v30 release evidence and
explicit scoped approval

```text
scripts/ai/verify_v30_release_gates.sh
scripts/ai/verify_v30_strict_provenance.sh
scripts/ai/check_release_surface_current.sh
scripts/ai/check_github_release_published.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Issue Closeout

#969 V300-000 = must be closed before v0.30.0 tag gate is accepted
#970 V300-001 = must be closed before v0.30.0 tag gate is accepted
#971 V300-002 = must be closed before v0.30.0 tag gate is accepted
#972 V300-003 = must be closed before v0.30.0 tag gate is accepted
#973 V300-004 = must be closed before v0.30.0 tag gate is accepted
#974 V300-005 = must be closed before v0.30.0 tag gate is accepted
#975 V300-006 = must be closed before v0.30.0 tag gate is accepted
#976 V300-007 = must be closed before v0.30.0 tag gate is accepted
#977 V300-008 = must be closed before v0.30.0 tag gate is accepted
#978 V300-009 = must be closed before v0.30.0 tag gate is accepted
#979 V300-010 = must be closed before v0.30.0 tag gate is accepted
#980 V300-011 = must be closed before v0.30.0 tag gate is accepted

## Release Scope

```text
V300 final release scope issue count = 12
V300 final release scope evidence count = 12
V300 exact milestone issue set = #969-#980
V300 registered corrective-scope exception count = 0
registered corrective-scope exceptions required = true
unregistered corrective milestone issues fail closed = true
v0.29.1 dependency proof = required
v0.29.1 release evidence = published
v31 production enablement = hard-blocked until v0.30.0 release evidence and explicit scoped approval
strict provenance manifest = target/ntpro-v300/v0_30_0_strict_release_manifest.json
```

## Post-Publication Closeout Target

```text
release tag = ntpro-rust-only-v0.30.0
release name = NTPRO Rust-only v0.30.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.30.0
source-controlled closeout evidence = docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
v31 handoff requires source-controlled v0.30.0 closeout = true
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
ambiguous_backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
product_grade_trading_terminal_claim = false
product_grade_live_trading_terminal_claim = false
default_production_execution_allowed = false
```
