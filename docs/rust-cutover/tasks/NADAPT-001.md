# NADAPT-001 - Adapter support matrix

Milestone: v0.2.0 Adapter Support
Priority: P1
Default role: Adapter & Integration
Risk: medium

## Goal

Make adapter support status explicit.

## Scope

Classify adapters as:

- supported;
- sandbox-only;
- fixture-only;
- deferred;
- removed.

Adapters to classify:

- Binance;
- OKX;
- Bybit;
- Coinbase;
- Databento;
- Deribit;
- dYdX;
- Hyperliquid;
- Interactive Brokers;
- Kraken;
- Polymarket;
- Tardis;
- Sandbox;
- other workspace adapters.

## Likely files

- `docs/integrations/adapter_support_matrix.md`
- `docs/rust-cutover/evidence/NADAPT-001.md`

## Non-goals

- Do not change adapter runtime behavior.
- Do not call live trading APIs.
- Do not hardcode secrets.

## Dependencies

- `RHARD-005`

## Acceptance criteria

- Every workspace adapter is classified.
- Unsupported adapters are recorded as deferred or removed, not silently
  omitted.
- Fixture and sandbox evidence strategy is noted.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NADAPT-001.md`.
