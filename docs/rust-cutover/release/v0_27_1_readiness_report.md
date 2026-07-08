# v0.27.1 Readiness Report

Date: 2026-07-08
Executor: Codex
Milestone: `ntpro-rust-only-v0.27.1`
Status: RELEASED

## Summary

v0.27.1 is ready for release gate execution when V271-001 through V271-006
evidence is present, V271-001 through V271-005 issues are closed, the current
V271-006 issue is closed for tag-gate mode, v27.1 release gates and strict
provenance pass, and the public release is published after a successful hosted
release gate for the same tag commit.

Plain Chinese summary: v0.27.1 的范围是发布治理和证据硬化，不是 v0.28.0 功能。它
要求 V271-001 到 V271-006 全部闭环，hosted release gate 成功后再公开 GitHub Release。
v0.28.0 在 v0.27.1 发布证据存在前保持 hard-blocked。

## Evidence

V271-001 evidence = docs/rust-cutover/evidence/V271-001.md
V271-002 evidence = docs/rust-cutover/evidence/V271-002.md
V271-003 evidence = docs/rust-cutover/evidence/V271-003.md
V271-004 evidence = docs/rust-cutover/evidence/V271-004.md
V271-005 evidence = docs/rust-cutover/evidence/V271-005.md
V271-006 evidence = docs/rust-cutover/evidence/V271-006.md

## Gates

v27.1 release gates = required
v27.1 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
v28 intake gate = hard-blocked until v0.27.1 publication evidence exists

```text
scripts/ai/verify_release.sh v27.1-release-gates
scripts/ai/verify_release.sh v27.1-strict-provenance
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh release-publish-after-gate
```

## Issue Closeout

#887 V271-001 = closed
#888 V271-002 = closed
#889 V271-003 = closed
#890 V271-004 = closed
#891 V271-005 = closed
#892 V271-006 = must be closed before v0.27.1 tag gate is accepted

## Release Scope

```text
V271 final release scope issue count = 6
V271 final release scope evidence count = 6
V271 exact milestone issue set = #887-#892
V271 registered corrective-scope exception count = 0
registered corrective-scope exceptions required = true
unregistered corrective milestone issues fail closed = true
v0.27.0 dependency proof = required
v0.27.0 release evidence = published
v0.28.0 start gate = blocked until v0.27.1 release evidence is published
strict provenance manifest = target/ntpro-v271/v0_27_1_strict_release_manifest.json
```

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
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Next Track

No V280 implementation starts until all V271 issues are closed and v0.27.1
release evidence is published. V280 intake must reconstruct the v0.27.1 GitHub
Release, hosted release gate, release body/source hash, and strict provenance
manifest before opening capability implementation.
