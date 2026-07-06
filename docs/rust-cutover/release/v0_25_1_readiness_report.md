# v0.25.1 Readiness Report

Date: 2026-07-06
Executor: Codex
Milestone: `ntpro-rust-only-v0.25.1`
Status: RELEASED

## Summary

v0.25.1 is ready when V251-001 through V251-006 evidence is present, all V251
issues are closed, v25.1 release gates and strict provenance pass, and the
public release is published after a successful hosted release gate for the same
tag commit.

Plain Chinese summary: v0.25.1 的范围是发布治理和证据硬化，不是 v0.26.0 功能。它
要求 V251-001 到 V251-006 全部闭环，hosted release gate 成功后再公开 GitHub Release。

## Evidence

V251-001 evidence = docs/rust-cutover/evidence/V251-001.md
V251-002 evidence = docs/rust-cutover/evidence/V251-002.md
V251-003 evidence = docs/rust-cutover/evidence/V251-003.md
V251-004 evidence = docs/rust-cutover/evidence/V251-004.md
V251-005 evidence = docs/rust-cutover/evidence/V251-005.md
V251-006 evidence = docs/rust-cutover/evidence/V251-006.md

## Gates

v25.1 release gates = required
v25.1 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required

```text
scripts/ai/verify_release.sh v25.1-release-gates
scripts/ai/verify_release.sh v25.1-strict-provenance
```

## Issue Closeout

#806 V251-001 = closed
#807 V251-002 = closed
#808 V251-003 = closed
#809 V251-004 = closed
#810 V251-005 = closed
#811 V251-006 = must be closed before v0.25.1 tag gate is accepted

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

No V260 implementation starts until all V251 issues are closed and v0.25.1
release evidence is published. V260 intake must reconstruct the v0.25.1 GitHub
Release, hosted release gate, release body/source hash, and strict provenance
manifest before opening capability implementation.
