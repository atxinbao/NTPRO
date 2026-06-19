# NTPRO v0.10.0 Binance Spot Sandbox Order Proof Readiness Report

Date: 2026-06-19
Executor: Codex
Milestone: `ntpro-rust-only-v0.10.0`
Status: PASS - READY FOR FORMAL RELEASE

## Summary

`v0.10.0` is the Binance spot sandbox order proof milestone. The queue defines
the order-proof boundary, config contract, fail-closed order gate, risk
preflight, signed request preview, `/api/v3/order/test` preflight, execution
artifact contract, reconciliation/orphan fixtures, read-only Dashboard proof
display, and release gate wiring.

Plain Chinese summary: v0.10.0 已完成 Binance Spot Demo Mode 的小额 submit/cancel
证明，并且证据包通过 gate。这个 PASS 不能写成生产 Binance、真实资金、实盘交易
或 Dashboard 下单能力。

## Product Claim

```text
capability = Binance spot sandbox order proof release package
current published release before v0.10.0 = ntpro-rust-only-v0.9.0
release tag = ntpro-rust-only-v0.10.0
default execution posture = offline fail-closed
manual online submit/cancel proof = completed through owner-confirmed Spot Demo Mode
proof endpoint mode = spot_demo_mode
production Binance = not included
real funds = not included
production trading = not included
Dashboard order controls = not included
```

## Included

```text
v0.10.0 order proof boundary
disabled-by-default execution config contract
multi-layer owner/manual order gate
offline risk preflight
redacted signed request preview
offline POST /api/v3/order/test preflight contract
execution artifact contract
reconciliation and orphan-order offline fixtures
read-only Dashboard order proof display
offline/manual release gate wiring
owner-gated Spot Demo Mode submit/cancel proof
release-note and readiness closure material
```

## Not Included

```text
production Binance connectivity
real funds
production trading
strategy-driven production execution
Dashboard order/cancel/retry controls
automatic online submit/cancel without owner gates
```

## Task Gate Matrix

| Task | Status | Evidence | Notes |
| --- | --- | --- | --- |
| V100-000 | PASS | `docs/rust-cutover/evidence/V100-000.md` | Defines v0.10.0 spot sandbox/manual-gated order proof boundary. |
| V100-001 | PASS | `docs/rust-cutover/evidence/V100-001.md` | Adds disabled-by-default execution config contract. |
| V100-002 | PASS | `docs/rust-cutover/evidence/V100-002.md` | Adds fail-closed CLI/env owner gate. |
| V100-003 | PASS | `docs/rust-cutover/evidence/V100-003.md` | Adds offline risk preflight. |
| V100-004 | PASS | `docs/rust-cutover/evidence/V100-004.md` | Adds redacted signed request preview without default network/order submission. |
| V100-005 | PASS | `docs/rust-cutover/evidence/V100-005.md` | Adds offline `/api/v3/order/test` preflight contract. |
| V100-006 | PASS | `docs/rust-cutover/evidence/V100-006.md` | Owner-confirmed Spot Demo Mode submit/cancel proof passed and artifact gate validated it. |
| V100-007 | PASS | `docs/rust-cutover/evidence/V100-007.md` | Defines execution artifact contract. |
| V100-008 | PASS | `docs/rust-cutover/evidence/V100-008.md` | Adds offline reconciliation and orphan-order fixtures. |
| V100-009 | PASS | `docs/rust-cutover/evidence/V100-009.md` | Adds read-only Dashboard order proof display without order controls. |
| V100-010 | PASS | `docs/rust-cutover/evidence/V100-010.md` | Wires offline/manual release gates. |
| V100-011 | PASS | `docs/rust-cutover/evidence/V100-011.md` | Finalizes readiness and release-note material for owner decision. |

## Manual Proof Artifact

```text
target/ntpro-v10-manual-order-proof/v100006-demo-20260619T132533Z/testnet_order_proof
```

Validated counters:

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

Validation command:

```bash
NTPRO_V10_MANUAL_ONLINE=1 \
NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V10_MANUAL_ORDER_PROOF_DIR=/Users/mac/Documents/NTPRO/target/ntpro-v10-manual-order-proof/v100006-demo-20260619T132533Z \
scripts/ai/verify_v10_manual_order_proof_gate.sh
```

Result:

```text
v10_manual_order_proof_gate status=artifact_package_ok manual_online=true testnet_orders_submitted=1 testnet_orders_canceled=1 production_orders_submitted=0 dashboard_order_controls=false endpoint_mode=spot_demo_mode
```

## Validation Evidence

Local validation collected across the V100 queue includes:

```text
scripts/ai/verify_fast.sh
cargo fmt --check
cargo test -p nautilus-cli --lib
cargo test -p nautilus-cli dashboard --lib
scripts/ai/verify_v10_offline_fail_closed.sh
scripts/ai/verify_v10_order_preflight.sh
scripts/ai/verify_v10_signed_order_request.sh
scripts/ai/verify_v10_order_test_preflight.sh
scripts/ai/verify_v10_execution_artifact_contract.sh
scripts/ai/verify_v10_reconciliation_fixture.sh
scripts/ai/verify_v10_offline_release_gates.sh
scripts/ai/verify_v10_manual_order_proof_gate.sh
scripts/ai/verify_release.sh v10-manual-order-proof-preflight v10-offline-release-gates
git diff --check
```

Hosted validation for the final V100 proof slice:

```text
PR #392 = merged
merge commit = 2ab70b5ea5a547b94bfae65fdf2717c671d9dba7
Rust Cutover Smoke / smoke = PASS
```

## Release Closure Status

The V100 task queue is complete. Shrimp reports 327 completed tasks and no
pending tasks.

Owner approved release closure on 2026-06-19:

```text
tag = ntpro-rust-only-v0.10.0
release name = NTPRO Rust-only v0.10.0
GitHub Release = formal release
```

## Final Verdict

The v0.10.0 release is ready for tag and GitHub Release publication.

Do not describe this readiness PASS as production Binance readiness, real-funds
readiness, production trading readiness, or Dashboard order-control readiness.
