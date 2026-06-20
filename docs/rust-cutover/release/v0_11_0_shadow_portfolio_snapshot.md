# NTPRO v0.11.0 Shadow Portfolio Snapshot Contract

Date: 2026-06-20
Executor: Codex
Status: PLANNED CONTRACT

## Summary

`v0.11.0` defines a minimal local shadow portfolio snapshot contract. The
snapshot combines read-only account context, local shadow execution intent
state, and local calculations where supported. It is not a production portfolio
ledger and must not be described as proof of production trading readiness.

Plain Chinese summary: 这份合同只定义本地“影子组合快照”的最小证据形状。它可以展示
balances、positions、exposure、PnL 这些字段，但必须讲清楚来源和可信度。算不了的
字段就写 `status=unavailable`，不能编造，也不能说这是实盘组合一致性证明。

## Artifact Location

The default v0.11 artifact path is:

```text
v0_11/shadow_portfolio_snapshot.json
```

The snapshot uses schema:

```text
schema_version=ntpro.v110_shadow_portfolio_snapshot.v1
snapshot_mode=production_readonly_shadow
```

## Required Top-Level Fields

The snapshot must include:

```text
schema_version
run_id
snapshot_id
snapshot_mode=production_readonly_shadow
source_account_snapshot_ref
source_shadow_intent_refs
balances
positions
exposure
pnl
risk_summary
created_at
production_orders_submitted=0
production_order_mutations_attempted=0
automatic_correction_orders_submitted=0
dashboard_order_controls_enabled=false
full_production_portfolio_parity_claimed=false
```

## Balances

`balances` records read-only account balance context or local fixture context.
Each balance entry must include:

```text
asset
free
locked
source=production_readonly_account_snapshot|local_fixture|shadow_state
confidence=observed|derived|unavailable
```

No API key, API secret, signature, signed query, signed URL, or raw exchange
payload with sensitive fields may be persisted in a balance entry.

## Positions

`positions` records a local shadow view. For spot-only flows, open derivative
position values may be unavailable. Entries must include:

```text
instrument_id
quantity
average_price
source=shadow_state|derived_from_balances|unavailable
status=observed|derived|unavailable
reason
```

When a position cannot be calculated, `status=unavailable` and `reason` are
required.

## Exposure

`exposure` records supported local exposure calculations. Each exposure value
must include:

```text
asset
gross
net
notional
quote_currency
status=observed|derived|unavailable
reason
```

Unsupported exposure calculations must be marked as unavailable, not estimated
without provenance.

## PnL

`pnl` records PnL only when supported by available read-only inputs and local
shadow state. It must include:

```text
realized
unrealized
quote_currency
status=derived|unavailable
reason
```

If fills, cost basis, or pricing inputs are incomplete, `status=unavailable` is
required. v0.11 does not claim full production PnL/accounting parity.

## Forbidden Behavior

The shadow portfolio snapshot must not:

- submit, cancel, replace, amend, retry, or correct production orders;
- create automatic correction orders when a mismatch is detected;
- write exchange state;
- enable Dashboard order controls;
- claim full production portfolio parity;
- claim real-funds trading readiness.

## Release Boundary

V110-005 may be used as evidence that NTPRO has a planned minimal local shadow
portfolio snapshot contract. It must not be used as evidence of production
portfolio consistency, live fill reconciliation, or real-funds trading
readiness.
