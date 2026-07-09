# v0.27.1 Readiness Report

Date: 2026-07-08
Executor: Codex
Milestone: `ntpro-rust-only-v0.27.1`
Status: RELEASED

## Summary

v0.27.1 has completed release gate execution and public GitHub Release
publication. V271-001 through V271-006 evidence is present, all V271 issues are
closed, the v0.27.1 milestone is closed, v27.1 release gates and strict
provenance passed, and the public release was published after a successful
hosted release gate for the same tag commit.

Plain Chinese summary: v0.27.1 的范围是发布治理和证据硬化，不是 v0.28.0 功能。它
要求 V271-001 到 V271-006 全部闭环，hosted release gate 成功后再公开 GitHub Release。
当前发布后 closeout 已记录，v0.28.0 只能依赖这些可重建发布证据进入后续 Backend
Closure / Product Operations Runtime Finalization 范围。

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
#892 V271-006 = closed

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.27.1
release name = NTPRO Rust-only v0.27.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.1
published at = 2026-07-08T13:18:35Z
GitHub Release draft = false
GitHub Release prerelease = false
annotated tag object = ab379be6725243ea1b8a9ffd9631409842361344
annotated tag peeled commit = 0fdc11dc983bbfb9fe124a3f171a58fb1e7ccf19
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28940442369
hosted release gate conclusion = success
hosted release gate jobs = 82/82 success
hosted release gate completed at = 2026-07-08T13:17:36Z
release publication after gate = pass
release body sha256 = 74bbc4d42d8e6f70d93a63fa4b42ae684cba4ccf0ce7c06e60490d4ad3a0f3f0
tracked release notes sha256 = 74bbc4d42d8e6f70d93a63fa4b42ae684cba4ccf0ce7c06e60490d4ad3a0f3f0
release body matches tracked release notes = true
v0.27.1 milestone = closed
v0.27.1 milestone open issues = 0
v0.27.1 milestone closed issues = 6
source-controlled closeout evidence = docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md
release-publication-evidence/ntpro-rust-only-v0.27.1.json = generated artifact, not sole proof
```

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
