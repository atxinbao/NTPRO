# Deribit dYdX Hyperliquid Rust Adapter Gap Inventory

Date: 2026-05-31
Executor: Codex
Task ID: RADP-010

## Scope

This inventory covers the Rust adapters under `crates/adapters/deribit/`,
`crates/adapters/dydx/`, and `crates/adapters/hyperliquid/`. It records current
Rust-only parser, data, execution, fixture, and adapter-boundary gaps for the
RADP-011 fixture task and the RADP-012 closure task.

The task is inventory-only. It does not change adapter behavior, trading
semantics, exchange protocol handling, credential handling, public APIs,
Python/PyO3 bindings, Cython surfaces, or Cargo feature behavior.

`code-index` was not exposed as a callable local MCP tool in this session;
repository inspection used local `rg`, `find`, and targeted file reads instead.

## Rust Surfaces Inspected

| Adapter | Current Rust surface |
| --- | --- |
| Deribit | `nautilus-deribit` builds as an `rlib` by default. The default feature set is `high-precision`; `python` and `extension-module` are optional. Rust config exposes HTTP and WebSocket JSON-RPC clients, futures, options, spot, future combos, option combos, ticker-derived mark/index/greeks, bars, funding, volatility index custom data, and margin execution. |
| dYdX | `nautilus-dydx` builds as an `rlib` by default. The default feature set is `high-precision`; `python` and `extension-module` are optional. Rust config exposes dYdX v4 Indexer HTTP, Indexer WebSocket, validator gRPC execution, perpetual-market instruments, subaccount/account reports, order-book/trade/candle streams, market status, oracle prices, funding, and margin execution. |
| Hyperliquid | `nautilus-hyperliquid` builds as an `rlib` by default. The default feature set is `high-precision`; `python` and `extension-module` are optional. Rust config exposes mainnet/testnet data and execution clients, perps, spot, HIP-4 outcome instruments, WebSocket market data, WebSocket post execution, account/vault addressing, CLOID routing, outcome settlement helpers, and optional outcome settlement polling. |

## Current Rust Evidence

| Adapter | Evidence |
| --- | --- |
| Deribit | `DeribitProductType` includes `Future`, `Option`, `Spot`, `FutureCombo`, and `OptionCombo`. `parse_deribit_instrument_any` maps those venue product types to Nautilus spot, perpetual, future, option, future-spread, and option-spread instruments. `DeribitDataClient` bootstraps configured product types, subscribes to L2 MBP books, trades, quotes, bars, mark/index prices, option greeks, funding, instrument status, and Deribit volatility index custom data. `DeribitExecutionClient` emits a margin account and maps market, limit, stop, take-limit, and take-market orders with explicit unsupported order and TIF cases. |
| dYdX | `parse_instrument_any` documents that dYdX v4 only lists perpetual markets and creates `CryptoPerpetual` instruments. `DydxDataClient` bootstraps all instruments, subscribes to `v4_markets`, orderbook, trades, candles, subaccount-related state, mark/index/oracle prices, funding, and instrument status. `DydxExecutionClient` uses wallet/subaccount configuration, private WebSocket subaccount subscriptions, gRPC order submission, block-height monitoring, transaction broadcasting, and margin account events. |
| Hyperliquid | `HyperliquidProductType::from_symbol` gates supported symbol families, while instrument loading gathers perp metadata, spot metadata, and best-effort HIP-4 outcome metadata. `HyperliquidDataClient` subscribes to trades, quotes, L2 MBP books, depth10, bars, mark/index prices, funding, all-mids, and open-interest custom data. `HyperliquidExecutionClient` resolves account/vault addresses, signs through the HTTP/WebSocket post path, caches CLOIDs, dispatches WebSocket execution reports, and optionally materializes HIP-4 outcome settlement fills. |

## Fixture And Test Coverage

| Adapter | Fixtures | Rust test surface |
| --- | --- | --- |
| Deribit | 38 files under `crates/adapters/deribit/test_data/`, covering HTTP accounts, instruments, combos, expirations, order book, trades, trading-view bars, and WebSocket books, quotes, charts, orders, portfolio, ticker, trades, subscription, and volatility index payloads. | 5 integration test files plus inline module tests; 210 annotated test entries found by local scan. |
| dYdX | 25 files under `crates/adapters/dydx/test_data/`, covering HTTP block height, candles, fills, funding, order book, orders, perpetual markets, subaccount, time, trades, transfers, and WebSocket markets, orderbook, candles, trades, block height, and subaccounts payloads. | 5 integration test files plus inline module tests; 680 annotated test entries found by local scan. |
| Hyperliquid | 8 files under `crates/adapters/hyperliquid/test_data/`, covering funding history, L2 book snapshots, perp metadata, spot clearinghouse state, all-mids, and WebSocket book data. | 5 integration test files plus inline module tests; 689 annotated test entries found by local scan. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| DER-ADP-001 | Partial: Deribit has broad fixtures and tests, but no compact parity manifest. | Fixtures and tests cover HTTP, WebSocket, data, execution, combo parsing, bars, orders, portfolio, and volatility index paths, but no adapter-level manifest maps supported, scoped, and deferred surfaces to fixtures. | Blocks release-gate traceability until RADP-011/RADP-012 records fixture-backed support decisions. |
| DER-ADP-002 | Scoped: Deribit product support is broad, but default data bootstrap only loads futures unless product types are configured. | `DeribitProductType` covers futures, options, spot, and combos; `DeribitDataClient::connect` defaults to `Future` when `config.product_types` is empty. | Blocks zero-config claims for spot, options, and combo instruments. |
| DER-ADP-003 | Scoped: Deribit order-book subscriptions only support L2 MBP and configured grouped/raw depth behavior. | `subscribe_book_deltas` and `subscribe_book_depth10` reject non-`L2_MBP`, validate Deribit depth, and route raw or grouped book subscriptions based on interval/group/depth parameters. | Fixture coverage must pin accepted depths, grouped book behavior, and unsupported book modes. |
| DER-ADP-004 | Scoped: Deribit market-data surfaces are ticker/channel-specific. | Mark prices, index prices, option greeks, and funding are emitted through Deribit ticker/perpetual channels; only `DeribitVolatilityIndex` is accepted as custom data; bars are limited to Deribit-supported external resolutions. | Blocks generic claims that every Nautilus data subscription maps to a Deribit native stream. |
| DER-ADP-005 | Scoped: Deribit execution is margin/account-authenticated and has an explicit order matrix. | Execution requires credentials on connect, emits `AccountType::Margin`, supports market, limit, stop-limit, stop-market, limit-if-touched, and market-if-touched, maps GTD to `good_til_day`, ignores custom GTD expiry, and rejects unsupported order or TIF values. | Order lifecycle evidence must keep these limits visible before release. |
| DER-ADP-006 | Deferred: optional Deribit Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, and optional PyO3 dependencies remain in the crate. | Blocks final Rust-only removal gate, but RADP-010 does not authorize deletion. |
| DYDX-ADP-001 | Partial: dYdX has broad parser and execution tests, but no compact parity manifest. | Fixtures and tests cover HTTP, WebSocket, gRPC, data, execution, block-time, broadcaster, order-builder, and account/subaccount parsing paths, but no adapter-level manifest maps support decisions to fixtures. | Blocks release-gate traceability until RADP-011/RADP-012 records fixture-backed support decisions. |
| DYDX-ADP-002 | Scoped: dYdX Rust instrument support is perpetual-market only. | `parse_instrument_any` documents dYdX v4 only lists perpetual markets and creates `CryptoPerpetual` instruments; HTTP endpoints target `/v4/perpetualMarkets`. | Blocks spot, options, and generic multi-asset dYdX support claims. |
| DYDX-ADP-003 | Scoped: dYdX market-data subscriptions are tied to v4 Indexer channel semantics. | L2 MBP book deltas are supported; quotes are synthesized from the orderbook channel; mark/index/oracle prices, funding, and status come through `v4_markets`; request book depth is ignored; bars use dYdX candle resolutions. | Fixture coverage must pin channel-to-Nautilus mappings and unsupported depth/book modes. |
| DYDX-ADP-004 | Scoped: dYdX execution uses chain-specific state and a narrower order/TIF contract. | Execution requires private key or authenticator configuration, wallet/subaccount identity, current block height, transaction broadcasting, and gRPC. It rejects quote-quantity orders, `FOK`, `DAY`, trailing stops, `MarketToLimit`, and unsupported order types. | Blocks generic order lifecycle parity unless these restrictions are explicit and fixture-backed. |
| DYDX-ADP-005 | Partial: dYdX operational lifecycle needs compact fixture classification. | Block-height monitoring, transaction manager, broadcaster, private subaccount WebSocket dispatch, hashed u32 client IDs, and order context correlation are implemented and tested, but not captured in an adapter parity manifest. | Blocks clear release-gate decisions for live execution operational risk. |
| DYDX-ADP-006 | Deferred: optional dYdX Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, and optional PyO3 dependencies remain in the crate. | Blocks final Rust-only removal gate, but RADP-010 does not authorize deletion. |
| HYP-ADP-001 | Partial: Hyperliquid Rust coverage is broad, but payload fixture inventory is sparse and no compact parity manifest exists. | Only 8 fixture files exist under `test_data/`, while tests cover many inline paths for signing, dispatch, execution, outcome settlement, parsing, and HTTP/WebSocket behavior. | Blocks release-gate traceability until RADP-011/RADP-012 adds or scopes fixture-backed support decisions. |
| HYP-ADP-002 | Scoped: Hyperliquid product support is perp, spot, and HIP-4 outcome specific. | Instrument loading parses perp metadata, spot metadata, and best-effort outcome metadata; outcome metadata failures are soft-skipped; sanitized symbol collisions are first-write-wins. | Blocks generic Hyperliquid product parity claims and requires explicit outcome/HIP-3/HIP-4 scope decisions. |
| HYP-ADP-003 | Scoped: Hyperliquid market-data support is L2 MBP plus venue-specific custom streams. | Book subscriptions reject non-`L2_MBP`, accept precision parameters, bars use supported external intervals, and custom data is limited to `HyperliquidAllMids` and `HyperliquidOpenInterest`. | Fixture coverage must pin book precision, all-mids/open-interest, bars, and outcome data expectations. |
| HYP-ADP-004 | Scoped: Hyperliquid execution maps market orders to venue IOC limits and has explicit order/product restrictions. | Market orders need cached quotes for derived limit prices, supported order types are market, limit, stop-market, stop-limit, market-if-touched, and limit-if-touched; `FOK`, `GTD`, `DAY`, and other TIFs are rejected; HIP-4 outcomes reject reduce-only and trigger orders. | Order lifecycle fixtures must make the quote/slippage/CLOID and HIP-4 restrictions visible. |
| HYP-ADP-005 | Partial: Hyperliquid operational lifecycle needs compact fixture classification. | WebSocket post execution, CLOID cache, builder attribution, account/vault address resolution, spot/perp clearinghouse reconciliation, and optional outcome settlement polling exist, but are not mapped into a release-gate manifest. | Blocks clear supported/deferred decisions for high-risk live execution and outcome-settlement behavior. |
| HYP-ADP-006 | Deferred: optional Hyperliquid Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, and optional PyO3 dependencies remain in the crate. | Blocks final Rust-only removal gate, but RADP-010 does not authorize deletion. |

## Support Classification

| Adapter surface | Classification | Notes |
| --- | --- | --- |
| Deribit futures/perpetual data and execution | Supported with constraints | Futures are the zero-config data default; execution is margin-authenticated and order/TIF limits apply. |
| Deribit options and option greeks | Supported with constraints | Option instruments and ticker-derived greeks exist, but users must configure product types and fixture manifests must pin the scope. |
| Deribit spot and combo instruments | Supported with constraints | Spot and combo parsers exist; combo leg metadata is attached when available; zero-config data does not imply these products. |
| Deribit custom data | Supported with constraints | Only `DeribitVolatilityIndex` is accepted as custom data. |
| dYdX perpetual market data | Supported with constraints | v4 Indexer perpetuals, orderbook, trades, candles, oracle/mark/index/funding/status streams exist. |
| dYdX execution | Supported with constraints | Margin subaccount, block-height, private-key/authenticator, gRPC, transaction-broadcast, and TIF/order-type restrictions apply. |
| dYdX non-perpetual products | Scoped out | Rust dYdX v4 support is not a spot/options adapter. |
| Hyperliquid perp and spot data/execution | Supported with constraints | Perp/spot instruments and account state exist; market orders depend on cached quotes and derived IOC limit prices. |
| Hyperliquid HIP-4 outcomes | Supported with constraints | Outcome instruments, settlement helpers, and outcome order constraints exist, but metadata is best-effort and fixtures are sparse. |
| Hyperliquid custom data | Supported with constraints | Only all-mids and open-interest custom data are accepted by the data client. |
| Python/PyO3 bindings for all three adapters | Deferred for removal gate | Optional bridge remains until removal gates approve deletion. |

## Follow-Ups

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RADP-011 | Adapter & Integration Agent | Add or record executable fixtures for the Deribit, dYdX, and Hyperliquid surfaces listed above, especially compact manifests, product boundaries, book constraints, order limits, operational lifecycle, dYdX block/transaction paths, and Hyperliquid outcome behavior. |
| RADP-012 | Adapter & Integration Agent | Close the listed gaps by implementing, deferring, or documenting support decisions in compact adapter parity manifests. |
| RPROD follow-up | Rust Product Surface Agent | Promote Rust-first Deribit, dYdX, and Hyperliquid config/examples into user-facing Rust docs instead of leaving docs primarily crate-level. |
| RTRACE follow-up | Verification & Release Gatekeeper | Bind these adapter payload fixtures into golden trace checks where release gates require it. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove optional Python/PyO3 adapter surfaces only after Rust product, runtime, adapter, QA, and release gates approve removal. |

## Non-Goals Preserved

- No Deribit, dYdX, or Hyperliquid runtime code changes.
- No exchange protocol behavior changes.
- No public API changes.
- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No live exchange API calls and no real credential usage.
- No CI or release gate changes.
