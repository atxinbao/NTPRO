# v0.26.1 Readiness Report

Date: 2026-07-07
Executor: Codex
Milestone: `ntpro-rust-only-v0.26.1`
Status: RELEASED

## Summary

v0.26.1 is ready for release gate execution when V261-001 through V261-006
evidence is present, V261-001 through V261-005 issues are closed, the current
V261-006 issue is closed for tag-gate mode, v26.1 release gates and strict
provenance pass, and the public release is published after a successful hosted
release gate for the same tag commit.

Plain Chinese summary: v0.26.1 的范围是发布治理和证据硬化，不是 v0.27.0 功能。它
要求 V261-001 到 V261-006 全部闭环，hosted release gate 成功后再公开 GitHub Release。
v0.27.0 在 v0.26.1 发布证据存在前保持 hard-blocked。

## Evidence

V261-001 evidence = docs/rust-cutover/evidence/V261-001.md
V261-002 evidence = docs/rust-cutover/evidence/V261-002.md
V261-003 evidence = docs/rust-cutover/evidence/V261-003.md
V261-004 evidence = docs/rust-cutover/evidence/V261-004.md
V261-005 evidence = docs/rust-cutover/evidence/V261-005.md
V261-006 evidence = docs/rust-cutover/evidence/V261-006.md

## Gates

v26.1 release gates = required
v26.1 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
v27 intake gate = hard-blocked until v0.26.1 publication evidence exists

```text
scripts/ai/verify_release.sh v26.1-release-gates
scripts/ai/verify_release.sh v26.1-strict-provenance
scripts/ai/verify_release.sh v27-intake-gate
```

## Issue Closeout

#847 V261-001 = closed
#848 V261-002 = closed
#849 V261-003 = closed
#850 V261-004 = closed
#851 V261-005 = closed
#852 V261-006 = must be closed before v0.26.1 tag gate is accepted

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

No V270 implementation starts until all V261 issues are closed and v0.26.1
release evidence is published. V270 intake must reconstruct the v0.26.1 GitHub
Release, hosted release gate, release body/source hash, and strict provenance
manifest before opening capability implementation.
