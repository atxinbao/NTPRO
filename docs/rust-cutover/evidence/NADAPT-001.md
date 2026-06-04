# NADAPT-001 Adapter Support Matrix Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NADAPT-001
Risk: medium

## Scope

NADAPT-001 records the v0.2.0 adapter support status for every adapter crate in
the workspace. It does not change adapter runtime behavior, does not call live
trading APIs, and does not add credentials.

## Files Created

- `docs/integrations/adapter_support_matrix.md`
- `docs/rust-cutover/evidence/NADAPT-001.md`

## Adapter Inventory Command

```bash
find crates/adapters -maxdepth 2 -name Cargo.toml -print | sort
```

Result: 17 workspace adapter crates were found.

```text
crates/adapters/architect_ax/Cargo.toml
crates/adapters/betfair/Cargo.toml
crates/adapters/binance/Cargo.toml
crates/adapters/bitmex/Cargo.toml
crates/adapters/blockchain/Cargo.toml
crates/adapters/bybit/Cargo.toml
crates/adapters/coinbase/Cargo.toml
crates/adapters/databento/Cargo.toml
crates/adapters/deribit/Cargo.toml
crates/adapters/dydx/Cargo.toml
crates/adapters/hyperliquid/Cargo.toml
crates/adapters/interactive_brokers/Cargo.toml
crates/adapters/kraken/Cargo.toml
crates/adapters/okx/Cargo.toml
crates/adapters/polymarket/Cargo.toml
crates/adapters/sandbox/Cargo.toml
crates/adapters/tardis/Cargo.toml
```

## Fixture/Test Inventory

```bash
for d in crates/adapters/*; do
  [ -d "$d" ] || continue
  name=$(basename "$d")
  fixtures=$(find "$d/test_data" -type f 2>/dev/null | wc -l | tr -d ' ')
  tests=$(find "$d/tests" -type f 2>/dev/null | wc -l | tr -d ' ')
  examples=$(find "$d/examples" -type f 2>/dev/null | wc -l | tr -d ' ')
  bins=$(find "$d/bin" -type f 2>/dev/null | wc -l | tr -d ' ')
  pkg=$(sed -n 's/^name = "\\(.*\\)"/\\1/p' "$d/Cargo.toml" | head -1)
  printf '%s|%s|fixtures=%s|tests=%s|examples=%s|bins=%s\n' \
    "$name" "$pkg" "$fixtures" "$tests" "$examples" "$bins"
done | sort
```

Result:

```text
architect_ax|nautilus-architect-ax|fixtures=54|tests=7|examples=2|bins=4
betfair|nautilus-betfair|fixtures=73|tests=6|examples=4|bins=0
binance|nautilus-binance|fixtures=70|tests=13|examples=5|bins=4
bitmex|nautilus-bitmex|fixtures=24|tests=3|examples=3|bins=3
blockchain|nautilus-blockchain|fixtures=0|tests=1|examples=1|bins=1
bybit|nautilus-bybit|fixtures=65|tests=6|examples=5|bins=4
coinbase|nautilus-coinbase|fixtures=22|tests=4|examples=2|bins=3
databento|nautilus-databento|fixtures=20|tests=4|examples=1|bins=0
deribit|nautilus-deribit|fixtures=39|tests=6|examples=4|bins=3
dydx|nautilus-dydx|fixtures=26|tests=6|examples=3|bins=5
hyperliquid|nautilus-hyperliquid|fixtures=9|tests=6|examples=3|bins=9
interactive_brokers|nautilus-interactive-brokers|fixtures=3|tests=2|examples=2|bins=0
kraken|nautilus-kraken|fixtures=66|tests=11|examples=4|bins=3
okx|nautilus-okx|fixtures=60|tests=6|examples=4|bins=5
polymarket|nautilus-polymarket|fixtures=47|tests=5|examples=4|bins=6
sandbox|nautilus-sandbox|fixtures=3|tests=2|examples=1|bins=0
tardis|nautilus-tardis|fixtures=17|tests=4|examples=1|bins=5
```

## Source Documents Reviewed

- `docs/rust-cutover/inventory/binance_adapter_gaps.md`
- `docs/rust-cutover/inventory/bybit_okx_kraken_adapter_gaps.md`
- `docs/rust-cutover/inventory/coinbase_bitmex_adapter_gaps.md`
- `docs/rust-cutover/inventory/databento_tardis_adapter_gaps.md`
- `docs/rust-cutover/inventory/deribit_dydx_hyperliquid_adapter_gaps.md`
- `docs/rust-cutover/inventory/interactive_brokers_adapter_gaps.md`
- `docs/rust-cutover/inventory/betfair_architect_ax_adapter_gaps.md`
- `docs/rust-cutover/inventory/polymarket_sandbox_adapter_gaps.md`
- `crates/adapters/blockchain/README.md`
- `crates/adapters/blockchain/Cargo.toml`

## Classification Summary

| Status | Adapters |
| --- | --- |
| supported | Architect AX, Betfair, Binance, BitMEX, Bybit, Coinbase, Deribit, dYdX, Hyperliquid, Kraken, OKX, Polymarket |
| sandbox-only | Sandbox |
| fixture-only | Databento, Tardis |
| deferred | Blockchain / DeFi, Interactive Brokers |
| removed | None |

## Safety Notes

- `supported` does not mean routine automation may connect to real trading
  endpoints.
- Credentialed behavior remains manual, env-gated, or deferred unless a later
  task explicitly provides mock, fixture, sandbox, or dry-run evidence.
- No adapter runtime code was changed.
- No secret was added or hardcoded.

## Required Validation

```bash
git diff --check
scripts/ai/verify_fast.sh
```

Current results:

- `git diff --check`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No behavior changed. This task adds a documentation matrix and evidence only.

## Public API Impact

No public API changed.

## Rollback Plan

Revert this PR to remove `docs/integrations/adapter_support_matrix.md`,
`docs/rust-cutover/evidence/NADAPT-001.md`, and NADAPT-001 agentflow state
changes.
