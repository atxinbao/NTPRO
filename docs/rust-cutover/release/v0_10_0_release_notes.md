# NTPRO Rust-only v0.10.0 Release Notes

Date: 2026-06-19
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION - V100-006 MANUAL ONLINE PROOF PENDING - NOT RELEASED

## Summary

`v0.10.0` is the Binance Testnet Order Proof release package. It adds the
testnet-only boundary, fail-closed gates, offline order-proof artifacts,
reconciliation fixtures, read-only Dashboard proof display, and release gates.

Plain Chinese summary: v0.10.0 的材料已经准备好给 owner 做发布决策。它默认
仍是离线 fail-closed，不会自动连接 Binance，也不会自动下单。真实 Binance
testnet 的 tiny submit-and-cancel 证明还在 `V100-006` 人工 gate 里，当前 release
notes 不能写成“真实下单证明已经完成”。

## Changed

Delivered changes:

- v0.10.0 Binance Testnet Order Proof boundary;
- disabled-by-default `[testnet_order]` execution config contract;
- multi-layer CLI/env owner gate before any order mutation path;
- offline order risk preflight;
- redacted signed Binance testnet order request preview;
- offline `/api/v3/order/test` preflight report;
- execution artifact contract for request, order-test, submit ack, cancel ack,
  terminal lifecycle, and reconciliation evidence;
- offline reconciliation/orphan-order fixture scenarios;
- read-only Dashboard order proof display;
- default offline release gate bundle;
- separate manual order-proof artifact validator.

## Boundary

Included:

```text
Binance testnet-only order proof package
offline fail-closed default gates
redacted request and artifact contracts
offline reconciliation/orphan fixtures
read-only Dashboard evidence display
manual gate infrastructure for future owner-approved V100-006 evidence
release readiness material
```

Not included:

```text
automatic online submit/cancel
completed V100-006 manual online proof
production Binance connectivity
real funds
production trading
strategy-driven production execution
Dashboard order, cancel, replace, retry, or amend controls
GitHub tag creation before owner approval
GitHub Release publication before owner approval
```

## Validation

Readiness evidence for this release package is recorded in:

```text
docs/rust-cutover/evidence/V100-000.md
docs/rust-cutover/evidence/V100-001.md
docs/rust-cutover/evidence/V100-002.md
docs/rust-cutover/evidence/V100-003.md
docs/rust-cutover/evidence/V100-004.md
docs/rust-cutover/evidence/V100-005.md
docs/rust-cutover/evidence/V100-007.md
docs/rust-cutover/evidence/V100-008.md
docs/rust-cutover/evidence/V100-009.md
docs/rust-cutover/evidence/V100-010.md
docs/rust-cutover/evidence/V100-011.md
```

`V100-006` remains manual-gated and is not included in the completed evidence
set unless the owner explicitly approves and runs that proof later.

Hosted validation for the final gate-wiring slice:

```text
PR #389 = merged
Rust Cutover Smoke / smoke = PASS
security-audit checks = PASS
```

## Release Status

This document is ready for a possible `ntpro-rust-only-v0.10.0` release
decision after V100-011 merges.

This task does not create a tag and does not publish a GitHub Release.

If owner-approved publication happens later, the release name should remain:

```text
NTPRO Rust-only v0.10.0
```

and the release boundary must explicitly preserve: Binance testnet-only,
manual-gated, no production Binance, no real funds, no production trading, and
no Dashboard order controls.
