# Binance v0.4 Capability Matrix

Date: 2026-06-13
Executor: Codex
Task ID: V04-002

## Scope

This matrix constrains the `v0.4.0` Binance Sandbox Product Foundation. It is a
product-boundary matrix, not a full Binance adapter parity claim.

Allowed classifications:

- `supported`: release-facing behavior has executable evidence and is inside
  the current product claim.
- `partial`: Rust code exists, but the release-facing product claim has limits
  or missing evidence.
- `sandbox-only`: local sandbox behavior is allowed; production venue behavior
  is not claimed.
- `fixture-only`: checked-in fixtures or deterministic replay evidence may be
  used; live venue behavior is not claimed.
- `deferred`: out of the `v0.4.0` product claim.
- `removed`: not part of NTPRO and must not be advertised.

## Product Boundary Summary

| Product | v0.4 classification | Summary |
| --- | --- | --- |
| Binance Spot | `sandbox-only` | The only Binance product allowed to become a v0.4 user-facing sandbox path after fixture replay, mock order lifecycle, risk rejection, EMA/RSI smokes, and Dashboard panels are proven. |
| Binance USDT-M | `deferred` | Existing Rust adapter surfaces and fixtures may be referenced as adapter evidence, but USDT-M is not the v0.4 product path unless a later scope decision adds explicit sandbox evidence. |

## Binance Spot Matrix

| Surface | Classification | v0.4 claim | Evidence source | Deferred or excluded |
| --- | --- | --- | --- | --- |
| Product identity | `sandbox-only` | Binance Spot is the named v0.4 sandbox foundation target. | `docs/rust-cutover/scope/v0_4_0_binance_sandbox_product_foundation.md` | Production Spot trading, real accounts, and real orders. |
| Market data replay | `fixture-only` | May use checked-in Binance-like fixtures for deterministic local replay after `V04-005`. | `crates/adapters/binance/test_data/spot/**`, `crates/adapters/binance/tests/spot/**` | Live production market data as release evidence. |
| Runtime data adapter | `partial` | Existing Rust Spot data code can inform replay and parser evidence, but v0.4 must keep the product claim local and deterministic. | `crates/adapters/binance/src/spot/data.rs` | Generic Spot data parity and unsupported book modes. |
| Execution behavior | `sandbox-only` | v0.4 may show submit, accept, fill, cancel, and scoped reject through mock lifecycle only after `V04-008`. | Future `V04-008` evidence; current parser/execution code remains reference-only. | Real order submission and production matching parity. |
| Risk behavior | `sandbox-only` | v0.4 must surface deterministic local risk rejection after `V04-009`. | Future `V04-009` evidence. | Production pre-trade risk parity. |
| Strategy behavior | `sandbox-only` | Built-in EMA and RSI smokes may run on Spot fixture replay after `V04-006` and `V04-007`. | Future `V04-006` and `V04-007` evidence. | Strategy profitability, arbitrary user strategy loading, and production alpha claims. |
| Config surface | `partial` | v0.4 config can expose scoped sandbox strategy configuration after `V04-004`. | Future `V04-004` evidence. | Generic Binance production config and arbitrary adapter config loading. |
| Dashboard state | `deferred` | Dashboard exchange, strategy, order, and risk panels are not claimed until `V04-010` proves they consume sandbox state. | Future `V04-010` evidence. | Manual order entry, remote dashboard operation, and placeholder business state. |
| Credentials | `removed` | Real credentials are not a v0.4 product input. | Scope contract and release evidence. | API keys, real account balances, and real venue order acknowledgements. |

## Binance USDT-M Matrix

| Surface | Classification | v0.4 claim | Evidence source | Deferred or excluded |
| --- | --- | --- | --- | --- |
| Product identity | `deferred` | USDT-M is not the v0.4 product path. | `docs/rust-cutover/scope/v0_4_0_binance_sandbox_product_foundation.md` | User-facing v0.4 USDT-M sandbox claim. |
| Market data fixtures | `fixture-only` | Existing futures fixtures may remain adapter evidence, but they do not create a v0.4 product claim. | `crates/adapters/binance/test_data/futures/**`, `crates/adapters/binance/tests/futures/**` | Live USDT-M market-data support as release claim. |
| Runtime data adapter | `partial` | Existing Rust futures data code is adapter reference material only for v0.4. | `crates/adapters/binance/src/futures/data.rs` | Full production USDT-M data parity. |
| Execution behavior | `partial` | Existing Rust futures execution code is not promoted into the v0.4 product path. | `crates/adapters/binance/src/futures/execution.rs` | Real USDT-M order submission, leverage, margin, or close-position product claims. |
| Risk behavior | `deferred` | No USDT-M-specific v0.4 risk product claim. | Future scope decision required. | Production futures pre-trade risk parity. |
| Strategy behavior | `deferred` | EMA/RSI v0.4 smokes should target the Spot sandbox path unless later evidence changes the scope. | Future scope decision required. | USDT-M strategy productization. |
| Config surface | `deferred` | USDT-M-specific v0.4 user config is out of scope. | Future scope decision required. | Futures leverage/margin production config claims. |
| Dashboard state | `deferred` | USDT-M dashboard panels are out of the v0.4 claim. | Future scope decision required. | Business-facing futures dashboard support. |
| Credentials | `removed` | Real futures credentials are not a v0.4 product input. | Scope contract and release evidence. | API keys, real account balances, and real order acknowledgements. |

## Cross-Surface Rules

- `supported` must not be used for a Binance `v0.4.0` user-facing surface until
  the relevant V04 task has executable evidence.
- `sandbox-only` means local product evidence only; it never means production
  venue support.
- `fixture-only` means checked-in deterministic data only; it never means live
  market-data parity.
- `partial` means Rust code or tests exist but the release-facing product claim
  is narrower.
- `deferred` means do not document the surface as a v0.4 product capability.
- `removed` means the surface must not be accepted as a v0.4 product input.

## No-Secret Rule

The v0.4 validation path must not require real Binance API keys, account
balances, production endpoints, or order acknowledgements. Later tasks may use
fixtures, mocks, dry-runs, or sandbox-only local artifacts.

## Follow-Up Gates

- `V04-004` must make the strategy config surface concrete before smokes depend
  on it.
- `V04-005` must prove deterministic fixture replay before strategy or order
  evidence can rely on data flow.
- `V04-008` and `V04-009` must prove mock order lifecycle and risk rejection
  before `V04-006` and `V04-007` strategy smokes can claim order/risk evidence.
- `V04-010` must consume proven sandbox state rather than inventing Dashboard
  placeholders.
