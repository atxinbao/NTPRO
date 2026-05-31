# Coinbase BitMEX Rust Adapter Gap Inventory

Date: 2026-05-31
Executor: Codex
Task ID: RADP-007

## Scope

This inventory covers the Rust adapters under `crates/adapters/coinbase/` and
`crates/adapters/bitmex/`. It records current Rust-only parser, data, execution,
fixture, and adapter-boundary gaps for the RADP-008 fixture task and the
RADP-009 closure task.

The task is inventory-only. It does not change adapter behavior, trading
semantics, exchange protocol handling, credential handling, public APIs,
Python/PyO3 bindings, Cython surfaces, or Cargo feature behavior.

`code-index` was not exposed as a callable local MCP tool in this session;
repository inspection used local `rg`, `find`, and targeted file reads instead.

## Rust Surfaces Inspected

| Adapter | Current Rust surface |
| --- | --- |
| Coinbase | `nautilus-coinbase` builds as an `rlib` by default. The default feature set is `high-precision`; `python` and `extension-module` are optional. Rust config exposes live and sandbox environments, public data through Coinbase Advanced Trade HTTP/WebSocket, and execution split by `AccountType::Cash` for spot or `AccountType::Margin` for Coinbase Financial Markets derivatives. |
| BitMEX | `nautilus-bitmex` builds as an `rlib` by default. The default feature set is `high-precision`; `python` and `extension-module` are optional. Rust config exposes mainnet/testnet environments, HTTP and WebSocket data, margin execution, optional submit/cancel broadcast pools, and optional dead man's switch handling. |

## Current Rust Evidence

| Adapter | Evidence |
| --- | --- |
| Coinbase | `CoinbaseProductType` includes `Spot`, `Future`, and `Unknown`. `CoinbaseDataClient` bootstraps instruments, subscribes to level2/ticker/trade/candle/status WebSocket channels, and REST-polls derivatives-only index/funding data. `CoinbaseExecutionClientFactory` accepts only `AccountType::Cash` and `AccountType::Margin`, using netting OMS for both. Order payload construction supports market, limit, and stop-limit combinations with explicit unsupported cases. |
| BitMEX | `BitmexDataClient` bootstraps instruments over REST, consumes WebSocket order-book/trade/bar/instrument/funding tables, and supports L2 deltas plus depth10. `BitmexExecutionClientFactory` creates margin/netting execution clients. `BitmexExecutionClient` uses authenticated HTTP and WebSocket clients, optional submit/cancel broadcast pools, and optional dead man's switch. `BitmexOrderType::try_from` maps the supported Nautilus order types and rejects `MarketToLimit`. |

## Fixture And Test Coverage

| Adapter | Fixtures | Rust test surface |
| --- | --- | --- |
| Coinbase | 21 files under `crates/adapters/coinbase/test_data/`, covering HTTP accounts, orders, fills, products, product books, candles, CFM balance/positions, and WebSocket level2, ticker, candles, market trades, user, heartbeats, and subscriptions. | 3 integration test files plus inline module tests; 400 annotated test entries found by local scan. |
| BitMEX | 23 files under `crates/adapters/bitmex/test_data/`, covering HTTP instruments, orders, executions, positions, trades, trade bins, wallet, errors, credentials, and WebSocket order book, quote, trade, trade bin, funding, order, execution, margin, position, wallet, and liquidation payloads. | 2 integration test files plus inline module tests; 286 annotated test entries found by local scan. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| CB-ADP-001 | Partial: Coinbase has broad fixtures and tests, but no compact parity manifest. | Fixtures and tests cover HTTP, WebSocket, data, and execution paths, but there is no adapter-level manifest mapping supported, scoped, and deferred surfaces to fixtures. | Blocks release-gate traceability until RADP-008/RADP-009 records fixture-backed support decisions. |
| CB-ADP-002 | Scoped: Coinbase product support is limited to crypto spot and CFM derivatives. | `CoinbaseProductType` includes `Spot` and `Future`; the provider skips unknown products and non-crypto futures; execution factory accepts only `Cash` and `Margin` account types. | Blocks generic claims for every Coinbase Advanced Trade product category. |
| CB-ADP-003 | Scoped: Coinbase mark prices are explicitly unsupported, while index/funding data is REST-polled. | `subscribe_mark_prices` returns an explicit error; `DerivPollManager` periodically fetches `/products/{id}` for `IndexPriceUpdate` and `FundingRateUpdate`. | Blocks claiming WebSocket live parity for mark/index/funding data. |
| CB-ADP-004 | Scoped: Coinbase order-book streaming only supports L2 MBP deltas. | `subscribe_book_deltas` rejects non-`L2_MBP`; alias handling re-keys canonical venue product IDs back to subscribed IDs. | Unsupported book modes and alias behavior must be pinned by fixtures. |
| CB-ADP-005 | Scoped: Coinbase execution supports a narrower order matrix than Nautilus. | Order configuration supports market IOC/FOK, maps market GTC to IOC, supports limit GTC/GTD/FOK, supports stop-limit GTC/GTD, and rejects unsupported TIF/order-type combinations such as stop-market. | RADP-009 must keep these as explicit support limits or close them with targeted implementation and fixtures. |
| CB-ADP-006 | Scoped: Coinbase spot and CFM execution are selected by one account-type flag. | The execution factory dispatches spot via `Cash` and CFM derivatives via `Margin`; one mixed account type is rejected. | Product-surface docs and fixtures must not imply simultaneous spot plus CFM execution in one client instance. |
| CB-ADP-007 | Deferred: optional Coinbase Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, and optional PyO3 dependencies remain in the crate. | Blocks final Rust-only removal gate, but RADP-007 does not authorize deletion. |
| BMX-ADP-001 | Partial: BitMEX has broad fixtures and tests, but no compact parity manifest. | Fixtures and tests cover HTTP, WebSocket, data, execution, broadcaster, and parser paths, but no adapter-level manifest maps supported, scoped, and deferred surfaces to fixtures. | Blocks release-gate traceability until RADP-008/RADP-009 records fixture-backed support decisions. |
| BMX-ADP-002 | Scoped: BitMEX instrument support excludes several venue instrument classes. | Rust parsing supports spot, perpetuals, futures, prediction markets, and index instruments; stock perpetuals, options, swaps, reference baskets, legacy futures, and futures spreads are explicitly unsupported. | Blocks generic BitMEX instrument parity claims. |
| BMX-ADP-003 | Scoped: BitMEX book support is L2/depth10 with exchange-specific depth behavior. | `subscribe_book_deltas` rejects non-`L2_MBP`, maps depths up to 25 to `OrderBookL2_25`, otherwise uses `OrderBookL2`; depth10 has a separate subscription path. | Fixture coverage must pin channel/depth selection. |
| BMX-ADP-004 | Scoped: BitMEX bar support is limited to external last-price bars for selected intervals. | `request_bars` requires external aggregation and `Last` price, then allows only `1m`, `5m`, `1h`, and `1d`; unsupported WebSocket bar specs log errors. | Blocks generic bar aggregation parity claims. |
| BMX-ADP-005 | Scoped: BitMEX execution has authenticated margin/netting-only behavior and explicit order limits. | Execution requires API credentials, uses `AccountType::Margin`, rejects `MarketToLimit`, requires price trailing offsets, requires peg type before peg offset, and only allows pegged overrides for limit orders. | Order lifecycle fixtures must keep these limits visible before release. |
| BMX-ADP-006 | Partial: BitMEX broadcast pools and dead man's switch need release-gate fixture classification. | Execution config exposes submit/cancel broadcaster pools, proxy diversity, and dead man's switch; tests exist, but no parity manifest classifies these operational surfaces. | Blocks a clear supported/deferred decision for high-risk live-order operational behavior. |
| BMX-ADP-007 | Deferred: optional BitMEX Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, and optional PyO3 dependencies remain in the crate. | Blocks final Rust-only removal gate, but RADP-007 does not authorize deletion. |

## Support Classification

| Adapter surface | Classification | Notes |
| --- | --- | --- |
| Coinbase spot data/execution | Supported with constraints | Spot products use `AccountType::Cash`; WebSocket and REST fixtures exist; book mode is L2 MBP only. |
| Coinbase CFM derivatives data/execution | Supported with constraints | Derivatives use `AccountType::Margin`; CFM balance/position fixtures exist; index/funding are REST-polled and mark prices are unsupported. |
| Coinbase order lifecycle | Supported with constraints | Market, limit, and stop-limit paths exist with explicit TIF/order-type restrictions. |
| Coinbase alias/canonical product handling | Supported with constraints | Alias re-keying exists for book/trade/quote/status paths and needs compact fixture classification. |
| BitMEX market data | Supported with constraints | Quotes, trades, L2 deltas, depth10, mark/index, funding, status, and limited bars are present. |
| BitMEX execution | Supported with constraints | Margin/netting authenticated execution exists; WebSocket dispatch, REST submit/cancel, broadcaster, and dead man's switch behavior need fixture classification. |
| BitMEX unsupported instrument classes | Scoped out | Options, swaps, reference baskets, legacy futures, futures spreads, and stock perpetuals are not release-supported by this inventory. |
| Python/PyO3 bindings for both adapters | Deferred for removal gate | Optional bridge remains until removal gates approve deletion. |

## Follow-Ups

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RADP-008 | Adapter & Integration Agent | Add or record executable fixtures for Coinbase and BitMEX, especially product boundaries, book constraints, order limits, CFM derivatives, BitMEX bars, broadcaster, and dead man's switch surfaces. |
| RADP-009 | Adapter & Integration Agent | Close the listed gaps by implementing, deferring, or documenting support decisions in compact adapter parity manifests. |
| RPROD follow-up | Rust Product Surface Agent | Promote Rust-first Coinbase and BitMEX config/examples into user-facing Rust docs instead of leaving docs primarily crate-level. |
| RTRACE follow-up | Verification & Release Gatekeeper | Bind Coinbase and BitMEX adapter payload fixtures into golden trace checks where release gates require it. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove optional Python/PyO3 adapter surfaces only after Rust product, runtime, adapter, QA, and release gates approve removal. |

## Non-Goals Preserved

- No Coinbase or BitMEX runtime code changes.
- No exchange protocol behavior changes.
- No public API changes.
- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No live exchange API calls and no real credential usage.
- No CI or release gate changes.
