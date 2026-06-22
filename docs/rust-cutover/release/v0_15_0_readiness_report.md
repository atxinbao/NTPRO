# NTPRO v0.15.0 Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness Readiness Report

Date: 2026-06-22
Executor: Codex
Milestone: `ntpro-rust-only-v0.15.0`
Status: PASS - RELEASED

## Summary

`v0.15.0` is the Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness
release line. It defines a narrow owner-gated production mutation research
scope, builds redacted request-preview and dry-run evidence artifacts, and adds
release gates proving the boundary remains no-production-mutation.

Plain Chinese summary: v0.15.0 可以开始把“未来生产下单请求怎么构造、怎么人工审批、
怎么被 kill switch 拦住、怎么只走 dry-run adapter、怎么在 Dashboard 只读查看”跑通。
它仍然不是实盘交易版本：不发生产请求，不真实下单，不撤单，不改单，不重试，不纠错，
不用 listenKey，不用真实资金，不允许 Dashboard 下单按钮。

## Product Claim

```text
capability = Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness
release tag = ntpro-rust-only-v0.15.0
release name = NTPRO Rust-only v0.15.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.15.0
default execution posture = offline fail-closed
production mutation scope = classified and owner-gated for preview only
production order request preview = redacted local artifact only
manual owner approval = required for request preview artifact creation
kill switch runtime gate = active by default and blocks dry-run progression
execution adapter path = local dry-run artifact only
incident rollback = manual evidence artifact only
Dashboard panel = read-only preflight evidence only
production request sent = false
production order submission = not included
production order mutation = not included
production cancel/replace/amend/retry/correction = not included
listenKey lifecycle = not included
real funds = not included
production trading = not included
Dashboard order controls = not included
```

## Included

```text
production mutation scope decision
production mutation endpoint classifier gate
live order request dry-run preview builder
execution adapter isolation artifact
kill switch runtime enforcement artifact
manual approval lifecycle artifact
dry-run mutation golden traces
manual incident, rollback, and emergency-stop artifacts
Dashboard live-alpha mutation preflight read-only panel
v0.15 aggregate release gate wiring
v0.15 readiness and release-note material
```

## Not Included

```text
production order submission
production order-test submission
production cancel, replace, amend, retry, correction, or flatten
production HTTP request execution
production execution adapter implementation or calls
default production network execution
listenKey creation, keepalive, or close lifecycle
signed WebSocket user stream runtime
strategy-driven production execution
automatic production remediation
real funds
production trading
Dashboard order/cancel/replace/amend/retry/reconnect controls
Dashboard credential input
raw production responses, raw credentials, signatures, signed queries, or signed URLs
```

## Prerequisite Accounting

| PR | Status | Classification | Evidence | Capability expansion |
| --- | --- | --- | --- | --- |
| #477 | PASS | V141-007 v0.14.1 readiness and release notes | `docs/rust-cutover/evidence/V141-007.md` | No v0.15 capability; prerequisite hardening closure |

## Merged PR Accounting

| PR | Status | Classification | Evidence | Capability expansion |
| --- | --- | --- | --- | --- |
| #478 | PASS | V150-000 mutation scope decision | `docs/rust-cutover/evidence/V150-000.md` | Boundary only |
| #479 | PASS | V150-001 mutation endpoint classifier gate | `docs/rust-cutover/evidence/V150-001.md` | Classifier/preview scope only, default denied |
| #480 | PASS | V150-002 live order request dry-run builder | `docs/rust-cutover/evidence/V150-002.md` | Redacted request preview only |
| #481 | PASS | V150-003 execution adapter isolation | `docs/rust-cutover/evidence/V150-003.md` | Local dry-run adapter artifact only |
| #482 | PASS | V150-004 kill switch runtime enforcement | `docs/rust-cutover/evidence/V150-004.md` | Local runtime gate only |
| #483 | PASS | V150-005 manual approval lifecycle | `docs/rust-cutover/evidence/V150-005.md` | Approval for preview artifact only |
| #484 | PASS | V150-006 dry-run mutation golden traces | `docs/rust-cutover/evidence/V150-006.md` | Replay evidence only |
| #485 | PASS | V150-007 incident rollback artifact | `docs/rust-cutover/evidence/V150-007.md` | Manual evidence artifacts only |
| #486 | PASS | V150-008 Dashboard mutation preflight panel | `docs/rust-cutover/evidence/V150-008.md` | Read-only Dashboard status only |
| #487 | PASS | V150-009 v0.15 release gates | `docs/rust-cutover/evidence/V150-009.md` | Gate wiring only |
| #488 | PASS | V150-010 readiness and release notes | `docs/rust-cutover/evidence/V150-010.md` | Release accounting only |

## Gate Evidence

Local validation recorded across V150 evidence:

```text
scripts/ai/verify_v15_live_order_request_dry_run_builder.sh = PASS
scripts/ai/verify_v15_manual_approval_lifecycle.sh = PASS
scripts/ai/verify_v15_execution_adapter_isolation.sh = PASS
scripts/ai/verify_v15_kill_switch_runtime_enforcement.sh = PASS
scripts/ai/verify_v15_incident_rollback_artifact.sh = PASS
scripts/ai/verify_v15_release_gates.sh = PASS
scripts/ai/verify_release.sh v15-release-gates = PASS
cargo test -p nautilus-cli --test golden_trace_live_alpha_mutation_dry_run = PASS
cargo test -p nautilus-cli live_alpha_v15_dashboard --lib = PASS
scripts/ai/verify_release.sh release-surface-current-guard = PASS required for release closure
scripts/ai/verify_fast.sh = PASS required for release closure
git diff --check = PASS required for release closure
```

Hosted PR smoke evidence:

```text
PR #478 Rust Cutover Smoke = PASS, run 27931132826/job 82643078934
PR #479 Rust Cutover Smoke = PASS, run 27931364030/job 82643758467
PR #480 Rust Cutover Smoke = PASS, run 27933434766/job 82649858165
PR #481 Rust Cutover Smoke = PASS, run 27935821253/job 82657144741
PR #482 Rust Cutover Smoke = PASS, run 27938313409/job 82665304793
PR #483 Rust Cutover Smoke = PASS, run 27941300165/job 82675080841
PR #484 Rust Cutover Smoke = PASS, run 27945048003/job 82687553232
PR #484 security-audit checks = PASS, run 27945048076
PR #485 Rust Cutover Smoke = PASS, run 27949043777/job 82701031957
PR #485 security-audit checks = PASS, run 27949043776
PR #486 Rust Cutover Smoke = PASS, run 27950682713/job 82706530660
PR #486 security-audit checks = PASS, run 27950682647
PR #487 Rust Cutover Smoke = PASS, run 27952851654/job 82713776292
PR #487 security-audit checks = PASS, run 27952851662
```

Formal tag-triggered release gate evidence:

```text
tag = ntpro-rust-only-v0.15.0
workflow = Rust Cutover Release Gate
run = 27956715055
url = https://github.com/atxinbao/NTPRO/actions/runs/27956715055
event = push
headBranch = ntpro-rust-only-v0.15.0
headSha = 6bae005c6cf9f2ab4e2cd610abbb579f9cbe7a58
status = completed
conclusion = success
jobs = 47/47 success
createdAt = 2026-06-22T13:35:19Z
updatedAt = 2026-06-22T14:38:00Z
```

## Release Closure Status

The V150 task queue is complete and the formal v0.15.0 publication package is:

```text
formal tag = ntpro-rust-only-v0.15.0
formal release name = NTPRO Rust-only v0.15.0
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.15.0
formal tag-triggered release gate = PASS, run 27956715055, 47/47 jobs success
```

## Final Verdict

The v0.15.0 release package is the formal publication package for
`ntpro-rust-only-v0.15.0`.

Do not describe this readiness PASS as production order submission readiness,
production order mutation readiness, production cancel/replace/amend/retry/
correction readiness, listenKey lifecycle readiness, real-funds readiness,
production trading readiness, automatic production remediation readiness, or
Dashboard order-control readiness.
