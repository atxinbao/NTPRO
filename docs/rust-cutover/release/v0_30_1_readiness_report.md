# v0.30.1 Readiness Report

Date: 2026-07-12
Executor: Codex
Milestone: `ntpro-rust-only-v0.30.1`
Status: RELEASED

## Summary

v0.30.1 has completed tag-gate execution and public GitHub Release
publication. V301-001 through V301-007 evidence is present, all V301 issues are
closed, v30.1 release gates passed, strict provenance passed, and the release
was published only after the hosted release gate succeeded for the same tag
commit.

Plain Chinese summary: v0.30.1 的范围是发布治理和 v31 start gate 硬化，不是 v0.31.0
功能。V301-001 到 V301-007 已全部闭环，hosted release gate 已成功，GitHub Release
已公开发布。v0.31.0 现在可以记录 v0.30.1 dependency proof，但仍必须在显式 scoped
approval、risk gate、audit gate、rollback readiness 和 telemetry/SLO gate 之前 fail-closed。

## Evidence

V301-001 evidence = docs/rust-cutover/evidence/V301-001.md
V301-002 evidence = docs/rust-cutover/evidence/V301-002.md
V301-003 evidence = docs/rust-cutover/evidence/V301-003.md
V301-004 evidence = docs/rust-cutover/evidence/V301-004.md
V301-005 evidence = docs/rust-cutover/evidence/V301-005.md
V301-006 evidence = docs/rust-cutover/evidence/V301-006.md
V301-007 evidence = docs/rust-cutover/evidence/V301-007.md

## Gates

v30.1 release gates = required
v30.1 strict provenance = required
v30 release gates = required
v30 strict provenance = required
v31 intake gate = v0.30.1 publication evidence satisfied; explicit scoped approval still required
release surface current guard = required
release publication guard = required
release publish after gate = required

```text
scripts/ai/verify_v30_1_release_gates.sh
scripts/ai/verify_v30_1_strict_provenance.sh
scripts/ai/verify_v30_1_v31_start_gate.sh
```

## Issue Closeout

#999 V301-001 = must be closed before v0.30.1 tag gate is accepted
#1000 V301-002 = must be closed before v0.30.1 tag gate is accepted
#1001 V301-003 = must be closed before v0.30.1 tag gate is accepted
#1002 V301-004 = must be closed before v0.30.1 tag gate is accepted
#1003 V301-005 = must be closed before v0.30.1 tag gate is accepted
#1004 V301-006 = must be closed before v0.30.1 tag gate is accepted
#1005 V301-007 = must be closed before v0.30.1 tag gate is accepted

## Release Scope

```text
V301 final release scope issue count = 7
V301 final release scope evidence count = 7
V301 exact milestone issue set = #999-#1005
V301 registered corrective-scope exception count = 0
registered corrective-scope exceptions required = true
unregistered corrective milestone issues fail closed = true
v0.30.0 dependency proof = required
v0.30.0 release evidence = published
v0.31.0 start gate = blocked until v0.30.1 release evidence is published
strict provenance manifest = target/ntpro-v301/v0_30_1_strict_release_manifest.json
```

## Post-Publication Closeout Target

```text
release tag = ntpro-rust-only-v0.30.1
release name = NTPRO Rust-only v0.30.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.30.1
published release status = published_after_gate
GitHub Release published at = 2026-07-12T17:07:13Z
GitHub Release updated at = 2026-07-12T17:10:14Z
hosted release gate run = 29194173422
hosted release gate conclusion = success
hosted release gate jobs = 94/94 success
hosted release gate completed at = 2026-07-12T14:53:56Z
published after hosted gate = true
annotated tag object = 17d2b48ed4df2b21f1a0b20bf739fd46f33659be
peeled tag commit = 5b66335a8f625062dbcdd4f7441cfacab57b5ede
release body normalized sha256 = 1a9a71278ca7716a681b17667f5f7ef9c174f9eebacae0683a3c5a91cc4de4f9
tracked release notes normalized sha256 = 1a9a71278ca7716a681b17667f5f7ef9c174f9eebacae0683a3c5a91cc4de4f9
release body matches tracked release notes = true
v0.30.1 milestone = closed
v0.30.1 milestone open issues = 0
v0.30.1 milestone closed issues = 7
v0.30.1 milestone closed at = 2026-07-12T17:08:50Z
source-controlled closeout evidence = docs/rust-cutover/release/v0_30_1_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
v0.31.0 start gate contract = docs/rust-cutover/release/v0_30_1_v31_start_gate.json
v0.31.0 start gate requires source-controlled v0.30.1 closeout target = true
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
