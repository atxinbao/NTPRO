# NTPRO Rust-only v0.10.0 Release Notes

Date: 2026-06-19
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.10.0`
Release name: `NTPRO Rust-only v0.10.0`

## Summary

`v0.10.0` is the Binance spot sandbox order proof release. It completes the
v0.10 order-proof queue with a real owner-gated Spot Demo Mode submit/cancel
artifact package while keeping production trading disabled.

Plain Chinese summary: v0.10.0 已经完成一次 owner-gated Binance Spot Demo Mode
小额 LIMIT GTC submit/cancel 证明。它证明的是 sandbox 下单、撤单、终态
reconciliation 和脱敏证据链，不是生产 Binance，不是真实资金，不是实盘交易，
也没有给 Dashboard 增加下单按钮。

## Changed

Delivered changes:

- v0.10.0 Binance spot sandbox order-proof boundary;
- disabled-by-default `[testnet_order]` execution config contract;
- multi-layer CLI/env owner gate before any order mutation path;
- offline order risk preflight;
- redacted signed Binance spot sandbox order request preview;
- offline `/api/v3/order/test` preflight report;
- execution artifact contract for request, order-test, submit ack, cancel ack,
  terminal lifecycle, and reconciliation evidence;
- offline reconciliation/orphan-order fixture scenarios;
- read-only Dashboard order proof display;
- default offline release gate bundle;
- separate manual order-proof artifact validator;
- owner-confirmed Spot Demo Mode proof for V100-006 using
  `https://demo-api.binance.com`.

## Proof

The V100-006 proof artifact is:

```text
target/ntpro-v10-manual-order-proof/v100006-demo-20260619T132533Z/testnet_order_proof
```

Important counters:

```text
endpoint_mode=spot_demo_mode
testnet_orders_submitted=1
testnet_orders_canceled=1
production_orders_submitted=0
production_orders_canceled=0
dashboard_order_controls_enabled=false
redaction_passed=true
manual_submit_cancel_proof_observed=true
status=pass
```

The submitted Spot Demo Mode order reconciled to terminal `CANCELED`.

## Boundary

Included:

```text
Binance spot sandbox order proof package
owner-gated Spot Demo Mode submit/cancel evidence
offline fail-closed default gates
redacted request and artifact contracts
offline reconciliation/orphan fixtures
read-only Dashboard evidence display
release readiness material
```

Not included:

```text
production Binance connectivity
real funds
production trading
strategy-driven production execution
Dashboard order, cancel, replace, retry, or amend controls
automatic online submit/cancel without owner gates
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
docs/rust-cutover/evidence/V100-006.md
docs/rust-cutover/evidence/V100-007.md
docs/rust-cutover/evidence/V100-008.md
docs/rust-cutover/evidence/V100-009.md
docs/rust-cutover/evidence/V100-010.md
docs/rust-cutover/evidence/V100-011.md
```

Final hosted validation before publication:

```text
PR #392 = merged
merge commit = 2ab70b5ea5a547b94bfae65fdf2717c671d9dba7
Rust Cutover Smoke / smoke = PASS
```

## Release Status

This release is approved for formal publication as:

```text
tag = ntpro-rust-only-v0.10.0
release name = NTPRO Rust-only v0.10.0
```

The release boundary must continue to preserve: Binance spot sandbox only,
owner-gated order proof, no production Binance, no real funds, no production
trading, and no Dashboard order controls.
