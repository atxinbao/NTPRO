# Binance Rust Adapter Gap Inventory

Date: 2026-05-31
Executor: Codex
Task ID: RADP-001

## Scope

This inventory covers the Rust Binance adapter under
`crates/adapters/binance/`, including product selection, parser surfaces, data
clients, execution clients, fixtures, and Rust-only cutover blockers.

The task is inventory-only. It does not change adapter behavior, trading
semantics, public APIs, exchange protocol handling, credentials, Python/PyO3
bindings, Cython surfaces, or Cargo feature behavior.

## Rust Surfaces Inspected

| Area | Files | Current Rust surface |
| --- | --- | --- |
| Crate and features | `crates/adapters/binance/Cargo.toml`, `crates/adapters/binance/src/lib.rs` | `nautilus-binance` builds as an `rlib` by default. `python` and `extension-module` are optional features; the default feature set is `high-precision`. Example binaries and examples are feature-gated behind `examples`. |
| Product model | `src/common/enums.rs` | `BinanceProductType` includes `Spot`, `Margin`, `UsdM`, `CoinM`, and `Options`. The helper methods model spot, futures, linear, inverse, and options categories. |
| Factory boundary | `src/factories.rs` | Runtime factories create clients for `Spot`, `UsdM`, and `CoinM`. `Margin` and `Options` currently fall through to unsupported-product errors for both data and execution factories. Only the first configured `product_types` entry is used for client selection. |
| Config boundary | `src/config.rs` | Rust configs expose `BinanceDataClientConfig` and `BinanceExecClientConfig`, including product types, environment, HTTP/WS overrides, Ed25519 credentials, instrument status polling, WebSocket trading, futures leverage/margin settings, and WebSocket transport backend. |
| Spot parser/data path | `src/spot/**` | Spot data handles HTTP instrument loading and SBE WebSocket parsing for trades, BBO, depth snapshots, and depth diffs. Spot subscriptions cover book deltas, quotes, trades, bars, and instrument status polling. |
| Spot execution path | `src/spot/execution.rs`, `src/spot/websocket/trading/**` | Spot execution supports WebSocket trading API order submit/cancel/modify/cancel-replace flows, account state/fill/order reports, and user data parsing. |
| Futures parser/data path | `src/futures/**` | Futures data handles USD-M and COIN-M instruments, trades, agg trades, book ticker, depth updates, klines, mark price, liquidation custom data, current open interest, and historical open interest. |
| Futures execution path | `src/futures/execution.rs`, `src/futures/websocket/**` | Futures execution supports USD-M and COIN-M account state, order submit/cancel/modify, position close, leverage/margin setup, user data, WebSocket stream recovery, and WebSocket trading for supported paths. |
| Fixtures and tests | `crates/adapters/binance/test_data/**`, `crates/adapters/binance/tests/**` | Fixture sets exist for spot HTTP JSON, spot user-data JSON, spot user-data SBE, futures HTTP JSON, futures market-data JSON, and futures user-data JSON. Rust tests cover spot/futures HTTP, data clients, execution clients, stream clients, and trading clients. |
| Docs | `docs/integrations/binance.md`, `crates/adapters/binance/README.md` | Integration docs describe both Python and Rust. The docs product table lists Spot, USD-M Futures, and COIN-M Futures as supported while margin is not implemented. The crate README also names spot margin and options, which is broader than the current factory boundary. |
| Python/PyO3 bridge | `src/python/**`, `cfg(feature = "python")` annotations in Rust types | Python bindings remain optional and are not part of the default Rust build. They are a remaining removal-gate surface, not something RADP-001 is allowed to delete. |

## Current Rust Evidence

| Behavior | Evidence |
| --- | --- |
| Rust default build does not require Python bindings | `Cargo.toml` default features contain `high-precision`; `python` and `extension-module` are optional. |
| Runtime factories support Spot, USD-M, and COIN-M only | `BinanceDataClientFactory` and `BinanceExecutionClientFactory` match `Spot`, `UsdM`, and `CoinM`; every other product type bails as unsupported. |
| Spot market data parser path exists | `spot/data.rs` routes SBE stream messages through `parse_trades_event`, `parse_bbo_event`, `parse_depth_snapshot`, and `parse_depth_diff`. |
| Spot order path exists | `spot/execution.rs` dispatches WebSocket trading submit, cancel, and cancel-replace requests and parses account/order/fill events. |
| Futures market data parser path exists | `futures/data.rs` routes stream messages through trade, agg trade, book ticker, depth update, mark price, kline, and liquidation parsers. |
| Futures custom data path exists | `data_types.rs` defines futures open interest, historical open interest, funding rate update, and liquidation custom data. |
| Futures execution path exists | `futures/execution.rs` includes account state, order submit/cancel/modify, close-position, leverage, margin type, user-data, and stream recovery flows. |
| Fixture coverage exists | `test_data/spot` contains HTTP JSON, user-data JSON, and user-data SBE fixtures. `test_data/futures` contains HTTP JSON, market-data JSON, and user-data JSON fixtures. |
| Rust test coverage exists | The Binance test tree contains 388 annotated Rust tests across spot and futures HTTP, data client, execution client, stream, and trading client files. |
| Docs still contain mixed Python/Rust product guidance | `docs/integrations/binance.md` notes Python and Rust integration, links Python examples, and says the configuration tables describe the Python adapter while Rust config lives in `src/config.rs`. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| BIN-ADP-001 | Open: product support is broader in enum/README than in the runtime factory boundary. | `BinanceProductType` includes `Margin` and `Options`; `crates/adapters/binance/README.md` lists spot margin and European options; `src/factories.rs` only creates `Spot`, `UsdM`, and `CoinM` clients. | Blocks calling Binance Margin or Options Rust-only product support complete. RADP-003 must either implement, defer, or narrow docs/contract explicitly. |
| BIN-ADP-002 | Open: multi-product configs select only the first product type. | `BinanceDataClientFactory` and `BinanceExecutionClientFactory` call `.product_types.first()` to select one client. | Blocks treating a single config as a multi-product adapter registration path. |
| BIN-ADP-003 | Partial: Spot market data is SBE-first and has L2 MBP constraints. | `spot/data.rs` bails unless `BookType::L2_MBP`; depth is normalized to 5/10/20. | RADP-002 should record fixture coverage for these accepted depths and document scoped unsupported book modes. |
| BIN-ADP-004 | Partial: Futures book subscriptions are constrained to supported depth values. | `futures/data.rs` bails unless `BookType::L2_MBP` and validates depth against `BINANCE_BOOK_DEPTHS`. | RADP-002 should record fixture/smoke coverage for accepted depths and scoped unsupported book modes. |
| BIN-ADP-005 | Partial: custom data support is futures-only and narrow. | `futures/data.rs` accepts liquidation subscriptions and open-interest requests; unsupported custom data logs a warning and returns. | Blocks claiming generic Binance custom-data parity; unsupported custom data must remain documented. |
| BIN-ADP-006 | Partial: historical open interest has product-specific limits. | `data_types.rs` states COIN-M historical OI is limited to perpetual instruments; `futures/data.rs` logs unsupported product types in that path. | RADP-003 must either close this gap or keep it as an explicit scope decision. |
| BIN-ADP-007 | Open: docs/config surface is not fully Rust-first. | `docs/integrations/binance.md` still links Python live examples and says configuration tables describe the Python adapter; Rust config users are pointed to source. | Blocks Rust-only user-facing adapter docs until Rust config and examples are first-class. |
| BIN-ADP-008 | Open: optional Python/PyO3 Binance surfaces remain. | `src/python/**`, `cfg(feature = "python")`, `pyo3`, and `pyo3-stub-gen` remain in the crate. | Blocks final Rust-only removal gate, but RADP-001 does not authorize deleting these surfaces. |
| BIN-ADP-009 | Partial: no single adapter parity manifest classifies supported/deferred/removed Binance surfaces. | Current support is inferable from code/tests/docs, but not recorded in a compact release-gate manifest before this task. | This inventory provides the first classification input; RADP-002/RADP-003 must refine it with executable fixtures and closure evidence. |
| BIN-ADP-010 | Partial: no hardcoded real secret was found, but docs/tests/examples contain placeholders and test secrets. | Grep found placeholder strings such as `YOUR_BINANCE_API_KEY`, environment variable names, and test values such as `test_api_secret`; credential debug output redacts secrets. | No immediate release blocker, but fixture capture tools and examples must continue using environment variables and placeholders only. |

## Support Classification

| Surface | Classification | Notes |
| --- | --- | --- |
| Spot data | Supported with constraints | Instrument load, trades, quotes, bars, status polling, and L2 MBP book deltas exist. Depth is normalized to Binance SBE stream levels. |
| Spot execution | Supported with constraints | WebSocket trading and account/user-data paths exist. Rust-only evidence should keep using mocks/fixtures, not live keys. |
| USD-M Futures data | Supported with constraints | Futures streams, mark price, liquidation, open interest, funding, and status polling exist. |
| USD-M Futures execution | Supported with constraints | Execution path includes order lifecycle, leverage/margin setup, close position, and WebSocket trading support. |
| COIN-M Futures data | Supported with constraints | Futures data path supports Coin-M, with historical OI limitations documented for perpetual instruments. |
| COIN-M Futures execution | Supported with constraints | Factory routes Coin-M to futures execution client; WebSocket trading capability must stay scoped by concrete client behavior. |
| Margin | Deferred | Enum/README mention it, but factories reject unsupported product types for data and execution. |
| Options | Deferred | Enum/README mention it, but factories reject unsupported product types for data and execution. |
| Python/PyO3 bindings | Deferred for removal gate | Optional bridge remains until RREM gates approve deletion. |

## RADP-003 Closure Decisions

RADP-003 closes the Binance adapter parity gaps by making the current Rust
runtime boundary explicit and testable. It does not change exchange protocol
behavior, parser behavior, order behavior, credential handling, public APIs,
Python/PyO3 bindings, or Cython surfaces.

| Gap | RADP-003 decision | Release-gate result |
| --- | --- | --- |
| BIN-ADP-001 | Defer Margin and Options. Current Rust runtime factories support Spot, USD-M Futures, and COIN-M Futures only. | No longer an implicit support claim; dedicated implementation remains required before claiming Margin or Options support. |
| BIN-ADP-002 | Scope multi-product configs to one runtime client per factory creation. Rust-first registration should use one config/client per product target. | No runtime behavior change; no multi-product single-client parity claim. |
| BIN-ADP-003 | Scope Spot books to the existing L2 MBP/SBE depth support. | Unsupported book modes remain out of Rust parity scope. |
| BIN-ADP-004 | Scope Futures books to the existing L2 MBP supported-depth path. | Unsupported book modes remain out of Rust parity scope. |
| BIN-ADP-005 | Scope custom data to existing futures liquidation and open-interest surfaces. | Generic Binance custom-data parity is not claimed. |
| BIN-ADP-006 | Scope COIN-M historical open interest to perpetual instruments. | Non-perpetual COIN-M historical OI remains deferred. |
| BIN-ADP-007 | Close the adapter README mismatch by documenting current Rust runtime support and deferred surfaces. | Full Rust-first integration docs remain a Rust Product Surface follow-up. |
| BIN-ADP-008 | Defer optional Python/PyO3 Binance bridge removal to the removal gate. | No removal authorized by RADP-003. |
| BIN-ADP-009 | Close compact parity-manifest coverage through `rust_fixture_manifest.json` and the manifest test. | Release gate has a single manifest for supported, scoped, deferred, and closed Binance surfaces. |
| BIN-ADP-010 | Keep placeholder/test credential matches as non-blocking evidence. | No real secret identified; examples/capture tools remain placeholder/env-var based. |

## Fixture And Validation Follow-Ups

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RADP-002 | Adapter & Integration Agent | Add or record executable fixtures for Binance parser/lifecycle coverage, especially supported Spot/Futures order book constraints, custom data, user-data, and product boundary blockers. |
| RADP-003 | Adapter & Integration Agent | Implement, defer, or document closure for the Binance product and parity gaps listed above. |
| RPROD follow-up | Rust Product Surface Agent | Promote Rust Binance config/examples into first-class Rust product documentation instead of pointing users to source. |
| RTRACE follow-up | Verification & Release Gatekeeper | Bind Binance adapter payload fixtures into golden trace checks where release gate requires it. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove optional Python/PyO3 Binance surfaces only after Rust product, runtime, adapter, QA, and release gates approve removal. |

## Non-Goals Preserved

- No Binance adapter runtime code changes.
- No exchange protocol behavior changes.
- No public API changes.
- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No live Binance API calls and no real credential usage.
- No CI or release gate changes.
