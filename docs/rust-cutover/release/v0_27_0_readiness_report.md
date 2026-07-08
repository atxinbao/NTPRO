# v0.27.0 Readiness Report

Date: 2026-07-08
Executor: Codex
Milestone: `ntpro-rust-only-v0.27.0`
Status: RELEASED

## Summary

v0.27.0 is ready for release gate execution when V270-000 through V270-008
evidence is present, V270-000 through V270-007 issues are closed, the current
V270-008 issue is closed for tag-gate mode, v27 release gates and strict
provenance pass, and the public release is published after a successful hosted
release gate for the same tag commit.

Plain Chinese summary: v0.27.0 的范围是 Product Operations Runtime Integration
Foundation，不是实盘交易控制台。它要求 V270-000 到 V270-008 全部闭环，v0.26.1
发布依赖可重建，hosted release gate 成功后再公开 GitHub Release，并继续保持
no-submit、no-mutation、no-adapter-send、no-live-exchange、no-retry-scheduler、
no-automatic-remediation、no-Dashboard/Admin-trading-controls 边界。

## Evidence

V270-000 evidence = docs/rust-cutover/evidence/V270-000.md
V270-001 evidence = docs/rust-cutover/evidence/V270-001.md
V270-002 evidence = docs/rust-cutover/evidence/V270-002.md
V270-003 evidence = docs/rust-cutover/evidence/V270-003.md
V270-004 evidence = docs/rust-cutover/evidence/V270-004.md
V270-005 evidence = docs/rust-cutover/evidence/V270-005.md
V270-006 evidence = docs/rust-cutover/evidence/V270-006.md
V270-007 evidence = docs/rust-cutover/evidence/V270-007.md
V270-008 evidence = docs/rust-cutover/evidence/V270-008.md

## Gates

v27 release gates = required
v27 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
hosted release gate success before public GitHub Release = required

```text
scripts/ai/verify_release.sh v27-release-gates
scripts/ai/verify_release.sh v27-strict-provenance
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh release-publish-after-gate
```

## Issue Closeout

#853 V270-000 = closed
#854 V270-001 = closed
#855 V270-002 = closed
#856 V270-003 = closed
#857 V270-004 = closed
#858 V270-005 = closed
#859 V270-006 = closed
#860 V270-007 = closed
#861 V270-008 = must be closed before v0.27.0 tag gate is accepted

## Release Scope

```text
V270 final release scope issue count = 9
V270 final release scope evidence count = 9
v0.26.1 dependency proof = required
v0.26.1 release evidence = published
v0.27.0 milestone = must be closed before public publication
release body source = docs/rust-cutover/release/v0_27_0_release_notes.md
strict provenance manifest = target/ntpro-v270/v0_27_0_strict_release_manifest.json
release governance trace = tests/golden/v270_release_gates_strict_provenance.jsonl
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

`v0.27.1` is the next patch track. `v0.28.0` is the next capability track. Both
must start from their own scoped issues, gates, and strict provenance; neither
inherits production submit, production mutation, adapter send, live exchange
request, retry scheduling, automatic remediation, Dashboard / Admin trading
controls, or product-grade live terminal claims from v0.27.0.
