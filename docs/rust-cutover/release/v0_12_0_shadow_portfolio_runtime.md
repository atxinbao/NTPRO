# NTPRO v0.12.0 Shadow Portfolio Runtime

Date: 2026-06-21
Executor: Codex
Milestone: `v0.12.0`
Status: IMPLEMENTED CONTRACT

## Summary

V120-004 implements a local shadow portfolio runtime artifact. It consumes
redacted account snapshot summaries and local shadow execution intents, then
writes an auditable snapshot with provenance for observed, derived, and
unavailable fields.

Plain Chinese summary: 这个 runtime 不是实盘组合账本。它只是把“生产账户只读结果的
脱敏摘要”和“本地 shadow 交易意图”合在一起，让 Dashboard/运维能看到当前 shadow
组合证据：哪些信息来自账户 shape、哪些是本地 intent 推导、哪些因为没有 fill/成本价/
行情价而不可用。

## CLI

```bash
nautilus live production-shadow-portfolio-runtime \
  --run-id v120-shadow \
  --account-snapshot v0_12/production_account_snapshot_redacted.json \
  --shadow-intent v0_11/shadow_execution_intent.jsonl \
  --output v0_12/shadow_portfolio_runtime.json \
  --compat-snapshot-output v0_11/shadow_portfolio_snapshot.json
```

The command is local-only. It does not open network, sign requests, call order
endpoints, or mutate exchange state.

## Runtime Artifact

The v0.12 artifact uses:

```text
schema_version=ntpro.v120_shadow_portfolio_runtime.v1
snapshot_mode=production_readonly_shadow
```

Required sections:

```text
source_account_snapshot_ref
source_shadow_intent_refs
balances
positions
exposure
pnl
risk_summary
provenance
```

## Provenance Rules

Balances:

```text
status=observed_shape_only | unavailable
source=redacted_production_account_snapshot_shape
asset_values_recorded=false
free_values_recorded=false
locked_values_recorded=false
```

Exposure:

```text
status=derived_from_shadow_intents | unavailable
reason=derived from local shadow intent notional only; this is not exchange truth
```

Notional preflight:

```text
status=shadow_decimal_string_evidence_only | unavailable_shadow_notional
aggregation=rust_decimal_string_sum
f64_aggregation_used=false
live_alpha_money_math_ready=false
risk_or_execution_grade=false
```

The `notional_preflight` section is a v0.12.1 hardening field for the v0.12
runtime artifact. It records local shadow intent notional sums as Decimal/string
evidence only, so display and audit surfaces do not depend on `f64`
aggregation. This is still not risk-grade or execution-grade money math. Any
future live-alpha risk or execution path must revalidate notional values with a
dedicated money-math contract before use.

PnL:

```text
status=unavailable
reason=production fills, cost basis, and mark prices are not available
```

Risk summary:

```text
status=risk_halted
new_orders_blocked=true
risk_halted=true
```

## Boundary Counters

The runtime artifact must keep:

```text
actual_submission_count=0
production_orders_submitted=0
production_order_mutations_attempted=0
automatic_correction_orders_submitted=0
dashboard_order_controls_enabled=false
full_production_portfolio_parity_claimed=false
```

## Fail-Closed Inputs

The command rejects account snapshot artifacts that contain raw account response
fields, raw balances, raw permissions, API key values, API secret values,
signatures, signed queries, or signed URLs.

The command rejects shadow intents where any of these are true:

```text
actual_submission
execution_adapter_called
order_endpoint_access_attempted
production_order_mutation_attempted
dashboard_order_controls_enabled
```

## Release Boundary

V120-004 may be used as evidence that NTPRO can persist local shadow portfolio
runtime artifacts from redacted read-only inputs. It must not be used as
evidence of production portfolio parity, live fill reconciliation, real-funds
trading readiness, or exchange-confirmed portfolio state.
