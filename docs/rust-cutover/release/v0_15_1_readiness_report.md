# NTPRO v0.15.1 Mutation Dry-Run Hardening Readiness Report

Date: 2026-06-23
Executor: Codex
Milestone: `ntpro-rust-only-v0.15.1`
Status: READY FOR OWNER RELEASE DECISION

## Summary

`v0.15.1` is a patch/hardening release for the already published v0.15 guarded
live-alpha mutation dry-run harness. It tightens approval, signing-material,
order-gate, endpoint, execution-adapter-boundary, and release-accounting
evidence without adding production mutation capability.

Plain Chinese summary: v0.15.1 不是新能力版本，是 v0.15.0 之后的补强版本。它把
人工审批、签名材料、LIMIT-only order gate、`/api/v3/order/test` 边界、dry-run
execution adapter 边界和 release gate 证据收紧。它仍然不发生产请求、不真实下单、不撤
单、不改单、不调用生产 adapter、不用真实资金、不开放 Dashboard 下单控件。

## Product Claim

```text
capability = Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness hardening
capability expansion = false
planned release tag = ntpro-rust-only-v0.15.1
default execution posture = offline fail-closed
production mutation scope = owner-gated preview only
production order request preview = redacted local artifact only
manual owner approval = one-time and consumed after request preview
default signing material = synthetic
production signing material = owner-gated dry-run preview only
dry-run order gate = LIMIT only
production order-test preview = denied
execution boundary = StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter
execution command route = dry_run_adapter_only
production_adapter_route_allowed = false
production_adapter_instantiation_allowed = false
production request sent = false
production order submission = not included
production order mutation = not included
production HTTP request execution = not included
production execution adapter implementation = not included
real funds = not included
production trading = not included
Dashboard order controls = not included
```

## Included

```text
formal v0.15.0 tag-triggered release-gate evidence accounting
one-time manual approval consume artifact
synthetic signing-material default for request preview
owner-gated production signing-material dry-run preview
LIMIT-only production live-alpha dry-run order gate
production /api/v3/order/test request-preview denial
explicit execution dry-run adapter boundary contract
v0.15.1 aggregate release gate
v0.15.1 readiness and release-note material
security-audit lockfile remediation for transitive quinn-proto RUSTSEC-2026-0185
cargo-vet policy/exemption alignment for release security gates
```

## Not Included

```text
production order submission
production order-test submission
production cancel, replace, amend, retry, correction, or flatten
production HTTP request execution
production execution adapter implementation
production execution adapter instantiation
production execution adapter calls
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
| #489 | PASS | v0.15.0 publication surface | `docs/rust-cutover/release/v0_15_0_readiness_report.md` | No v0.15.1 capability; formal v0.15.0 publication baseline |

## Merged PR Accounting

| PR | Status | Classification | Evidence | Capability expansion |
| --- | --- | --- | --- | --- |
| #490 | PASS | V151-001 release gate evidence | `docs/rust-cutover/evidence/V151-001.md` | No; evidence/accounting only |
| #491 | PASS | V151-002 manual approval consume | `docs/rust-cutover/evidence/V151-002.md` | No; hardens one-time approval semantics |
| #492 | PASS | V151-003 synthetic signing default | `docs/rust-cutover/evidence/V151-003.md` | No; hardens signing-material boundary |
| #493 | PASS | V151-004 LIMIT-only dry-run order gate | `docs/rust-cutover/evidence/V151-004.md` | No; narrows accepted dry-run order intent |
| #494 | PASS | V151-005 deny production order-test preview | `docs/rust-cutover/evidence/V151-005.md` | No; narrows endpoint preview scope |
| #495 | PASS | V151-006 execution dry-run adapter boundary | `docs/rust-cutover/evidence/V151-006.md` | No; makes dry-run boundary explicit |
| #496 | PENDING | V151-007 v0.15.1 aggregate release gates/readiness and security-audit remediation | `docs/rust-cutover/evidence/V151-007.md` | No; release accounting, gate wiring, transitive patch lockfile update, and supply-chain gate alignment only |

## Gate Evidence

Required local validation for v0.15.1 release decision:

```text
scripts/ai/verify_release.sh v151-release-gates
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_fast.sh
git diff --check
```

Required hosted validation for the V151-007 PR:

```text
Rust Cutover Smoke = PASS
security-audit cargo-audit = PASS
security-audit osv-scanner = PASS
security-audit cargo-deny = PASS
security-audit cargo-vet = PASS
security-audit pip-audit = PASS
security-audit zizmor = PASS
```

The v0.15.1 aggregate gate includes:

```text
cargo test -p nautilus-cli endpoint_classifier --lib
cargo test -p nautilus-cli production_live_alpha_dry_run_order_gate --lib
cargo test -p nautilus-cli production_live_alpha_order_request_preview --lib
cargo test -p nautilus-cli production_live_alpha_execution_dry_run --lib
scripts/ai/verify_v14_release_gates.sh
scripts/ai/verify_v15_release_gates.sh
```

Hosted PR smoke evidence must be recorded after the V151-007 PR lands. Until a
formal `ntpro-rust-only-v0.15.1` tag exists, do not describe this document as
formal release evidence.

## Release Closure Status

```text
latest formal release = ntpro-rust-only-v0.15.0
v0.15.1 readiness = ready for owner release decision after V151-007 lands
v0.15.1 tag = not created by this readiness document
v0.15.1 GitHub Release = not created by this readiness document
```

## Final Verdict

The v0.15.1 source-tree package is ready to be evaluated as a patch/hardening
release once V151-007 validation and hosted smoke pass.

Do not describe this readiness as production order submission readiness,
production order mutation readiness, production order-test submission readiness,
production HTTP execution readiness, production execution-adapter readiness,
listenKey lifecycle readiness, real-funds readiness, production trading
readiness, automatic production remediation readiness, or Dashboard order-control
readiness.
