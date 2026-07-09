# v0.28.1 Readiness Report

Date: 2026-07-09
Executor: Codex
Milestone: `ntpro-rust-only-v0.28.1`
Status: RELEASE GATE READY

## Summary

v0.28.1 is ready for tag-gate execution when V281-001 through V281-008 evidence
is present, all V281 issues are closed, v28.1 release gates pass, strict
provenance passes, and the release is published only after the hosted release
gate succeeds for the same tag commit.

Plain Chinese summary: v0.28.1 的范围是发布治理和证据硬化，不是 v0.29.0 功能。它
要求 V281-001 到 V281-008 全部闭环，hosted release gate 成功后再公开 GitHub Release。
v0.29.0 在 v0.28.1 发布证据和 source-controlled closeout 证据存在前保持 hard-blocked。

## Evidence

V281-001 evidence = docs/rust-cutover/evidence/V281-001.md
V281-002 evidence = docs/rust-cutover/evidence/V281-002.md
V281-003 evidence = docs/rust-cutover/evidence/V281-003.md
V281-004 evidence = docs/rust-cutover/evidence/V281-004.md
V281-005 evidence = docs/rust-cutover/evidence/V281-005.md
V281-006 evidence = docs/rust-cutover/evidence/V281-006.md
V281-007 evidence = docs/rust-cutover/evidence/V281-007.md
V281-008 evidence = docs/rust-cutover/evidence/V281-008.md

## Gates

v28.1 release gates = required
v28.1 strict provenance = required
v29 intake gate = hard-blocked until v0.28.1 publication evidence exists
v28 release gates = required
v28 strict provenance = required
release body hash normalization = required
runtime-closed terminology hardening = required
release publish after gate current-release binding = required
release surface current guard = required
release publication guard = required
release publish after gate = required

```text
scripts/ai/verify_release.sh v28.1-release-gates
scripts/ai/verify_release.sh v28.1-strict-provenance
NTPRO_V29_INTAKE_ALLOW_UNPUBLISHED=1 scripts/ai/verify_release.sh v29-intake-gate
```

## Issue Closeout

#919 V281-001 = must be closed before v0.28.1 tag gate is accepted
#920 V281-002 = must be closed before v0.28.1 tag gate is accepted
#921 V281-003 = must be closed before v0.28.1 tag gate is accepted
#922 V281-004 = must be closed before v0.28.1 tag gate is accepted
#923 V281-005 = must be closed before v0.28.1 tag gate is accepted
#924 V281-006 = must be closed before v0.28.1 tag gate is accepted
#925 V281-007 = must be closed before v0.28.1 tag gate is accepted
#944 V281-008 = corrective release-gate blocker, must be closed before v0.28.1 tag gate is accepted

## Release Scope

```text
V281 final release scope issue count = 8
V281 final release scope evidence count = 8
V281 exact milestone issue set = #919-#925, #944
V281 registered corrective-scope exception count = 1
registered corrective-scope exceptions required = true
unregistered corrective milestone issues fail closed = true
v0.28.0 dependency proof = required
v0.28.0 release evidence = published
v0.29.0 start gate = blocked until v0.28.1 release evidence is published
strict provenance manifest = target/ntpro-v281/v0_28_1_strict_release_manifest.json
```

## Post-Publication Closeout Target

```text
release tag = ntpro-rust-only-v0.28.1
release name = NTPRO Rust-only v0.28.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.1
source-controlled closeout evidence = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
v0.29.0 intake requires source-controlled v0.28.1 closeout = true
```

## Boundary

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
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
product_grade_trading_terminal_claim = false
```

## Next Track

No V290 implementation starts until all V281 issues are closed and v0.28.1
release evidence is published. V290 intake must reconstruct the v0.28.1 GitHub
Release, hosted release gate, release body/source hash, and strict provenance
manifest before opening capability implementation.
