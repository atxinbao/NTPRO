# v0.9.0 Strategy Runtime Smoke Gates

Date: 2026-06-18
Executor: Codex
Task: V090-012

## Purpose

v0.9.0 adds release and PR gates for the local Strategy Runtime Foundation.
These gates prove that the strategy runtime path can run locally and still
stays outside all order-submission and production-trading boundaries.

## Gates

Two scripts are now available:

```text
scripts/ai/verify_v09_strategy_runtime_smoke.sh
scripts/ai/verify_v09_shadow_mode_no_order_gate.sh
```

`verify_release.sh` exposes matching stages:

```text
v09-strategy-runtime-smoke
v09-shadow-mode-no-order-gate
```

The release tag workflow includes both stages in the matrix.

## What The Gates Prove

- `ntpro-node` starts a local Strategy Session.
- Fixture market stream runs.
- Strategy signals are generated.
- Shadow order intents are generated.
- Shadow risk decisions are generated.
- Order submission remains disabled.
- No exchange order identity is recorded.
- No secrets are required.
- No production connection, real funds, or production trading is added.

## Boundaries

These gates are not Binance testnet order proof. They do not submit orders,
cancel orders, amend orders, connect to production Binance, use real funds, or
claim production trading readiness.

The Binance testnet order lifecycle proof remains deferred to v0.10.0.

## Rollback

Revert the V090-012 PR to remove the v09 smoke scripts and their CI/release
wiring. Existing v0.8 and earlier gates remain unchanged.
