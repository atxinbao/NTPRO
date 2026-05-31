# Bybit OKX Kraken Rust Adapter Gap Inventory

Date: 2026-05-31
Executor: Codex
Task ID: RADP-004

## Scope

This inventory covers the Rust adapters under `crates/adapters/bybit/`,
`crates/adapters/okx/`, and `crates/adapters/kraken/`. It records current
Rust-only parser, data, execution, fixture, and adapter-boundary gaps for the
RADP-005 fixture task and the RADP-006 closure task.

The task is inventory-only. It does not change adapter behavior, trading
semantics, exchange protocol handling, credential handling, public APIs,
Python/PyO3 bindings, Cython surfaces, or Cargo feature behavior.

`code-index` was not exposed as a callable MCP tool in this session; repository
inspection used local `rg`, `find`, and targeted file reads instead.

## Rust Surfaces Inspected

| Adapter | Current Rust surface |
| --- | --- |
| Bybit | `nautilus-bybit` builds as an `rlib` by default. The default feature set is `high-precision`; `python` and `extension-module` are optional. Rust config exposes `Spot`, `Linear`, `Inverse`, and `Option` product types for data and execution. Data creates one public WebSocket client per configured product type. Execution uses one private and one trade WebSocket client, then routes requests by product type. |
| OKX | `nautilus-okx` builds as an `rlib` by default. The default feature set is `high-precision`; `python` and `extension-module` are optional. Rust config exposes `Any`, `Spot`, `Margin`, `Swap`, `Futures`, `Option`, and `Events` instrument types. Data creates public and business WebSocket clients. Execution derives account/OMS behavior from the configured instrument types, spot-margin flag, and spread/order route. |
| Kraken | `nautilus-kraken` builds as an `rlib` by default. The default feature set is `high-precision`; `python` and `extension-module` are optional. Rust config exposes `Spot` and `Futures` product types. Factories explicitly instantiate separate Spot or Futures data/execution clients. |

## Current Rust Evidence

| Adapter | Evidence |
| --- | --- |
| Bybit | `BybitProductType` includes `Spot`, `Linear`, `Inverse`, and `Option`. `BybitDataClient::new` creates a public WebSocket client for each configured product type. `BybitExecutionClientFactory` treats `Linear`, `Inverse`, and `Option` as derivatives for margin/netting setup. The integration docs claim Spot, Linear, Inverse, and Option support, with order-capability caveats for options, trailing stops, and demo trading. |
| OKX | `OKXInstrumentType` includes `Any`, `Spot`, `Margin`, `Swap`, `Futures`, `Option`, and `Events`. `OKXDataClient` builds public and business WebSocket clients. `OKXExecutionClientFactory` treats `Swap`, `Futures`, and `Option` as derivatives and can use margin account mode for spot margin. The integration docs claim data and execution support for spot, margin, swaps, futures, options, spreads, and event contracts, with spread and option caveats. |
| Kraken | `KrakenProductType` includes `Spot` and `Futures`. `KrakenDataClientFactory` and `KrakenExecutionClientFactory` match only those two products. Config validation rejects Spot demo mode. Spot L3 requires credentials and depth `10`, `100`, or `1000`. Integration docs describe Spot and Futures support, Spot L3, Spot bar streaming, and Futures mark/index/funding paths. |

## Fixture And Test Coverage

| Adapter | Fixtures | Rust test surface |
| --- | --- | --- |
| Bybit | 64 files under `crates/adapters/bybit/test_data/`, including HTTP account, instrument, order, position, wallet, fee, funding, order-book, trade, and WebSocket account/order/position/wallet/kline/order-book/ticker fixtures. | 5 test files plus inline module tests; 609 annotated test entries found by local scan. |
| OKX | 59 files under `crates/adapters/okx/test_data/`, including HTTP instruments for spot, margin, swap, futures, options, spreads, account/order/algo/position/fee fixtures, and WebSocket account/books/candles/funding/instruments/orders/spread/ticker/trade fixtures. | 5 test files plus inline module tests; 676 annotated test entries found by local scan. |
| Kraken | 65 files under `crates/adapters/kraken/test_data/`, including Spot and Futures HTTP order/account/position/fill/instrument/order-book/bar fixtures and WebSocket Spot, Futures, L3, order, ticker, trade, and book fixtures. | 10 test files plus inline module tests; 664 annotated test entries found by local scan. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| BYB-ADP-001 | Partial: Bybit Rust product scope is broad but not captured in a compact parity manifest. | `BybitProductType` includes all four product categories; docs also claim all four. There is no adapter-level manifest that maps supported/scoped/deferred surfaces to fixtures. | Blocks release-gate traceability for Bybit until RADP-005/RADP-006 records fixture-backed support decisions. |
| BYB-ADP-002 | Partial: Bybit data supports multiple product WebSocket clients, but execution remains one private/trade client with product-specific routing. | Data creates one public WebSocket client per configured product type; execution creates one private WebSocket and one trade WebSocket and derives account/OMS type from the configured product list. | Blocks claiming a fully separated multi-product execution runtime until the route and fixture matrix proves each configured product path. |
| BYB-ADP-003 | Scoped: Bybit order-book subscriptions only support L2 MBP depths `1`, `50`, `200`, and `500`. | `BybitDataClient::subscribe_book_deltas` rejects non-L2 MBP and invalid depths. | Unsupported book modes and depths must stay documented; fixture task should pin accepted depth behavior. |
| BYB-ADP-004 | Scoped: Bybit product-specific data requests have unsupported combinations. | Funding rates reject Spot and Option; mark/index prices reject Spot; bars reject Option. | Blocks generic data parity claims across all product types. |
| BYB-ADP-005 | Scoped: Bybit execution supports common order types but still has unsupported order and environment cases. | HTTP submit order rejects unsupported Nautilus order types; docs state trailing stops are not available for Spot/Options and demo trading does not support the WebSocket Trade API. | RADP-006 must keep these as explicit support limits or close them with targeted implementation and fixtures. |
| BYB-ADP-006 | Deferred: optional Bybit Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, and optional PyO3 dependencies remain in the crate. | Blocks final Rust-only removal gate, but RADP-004 does not authorize deletion. |
| OKX-ADP-001 | Partial: OKX Rust instrument scope is very broad and needs a compact parity manifest. | `OKXInstrumentType` includes spot, margin, swaps, futures, options, spreads via separate endpoints, and events; docs claim data and execution support across those surfaces. | Blocks release-gate traceability until every product surface is classified as supported, scoped, or deferred with fixtures. |
| OKX-ADP-002 | Scoped: options and events are not equivalent to regular order paths. | `supports_algo_orders` returns false for `Option` and `Events`; execution rejects trigger/conditional orders for options; WebSocket order code rejects market orders for options. | Blocks claiming full OKX order-type parity for options/events. |
| OKX-ADP-003 | Scoped: spread order support is a separate HTTP/business WebSocket path with limits. | Docs state spread orders route through `/api/v5/sprd/*` endpoints; spread order lists, conditional orders, FOK, and modify requests are not supported by the spread path. | Spread support must remain a separate supported-with-constraints surface. |
| OKX-ADP-004 | Scoped: OKX order-book subscriptions only support L2 MBP and clamp depth to OKX-supported channels. | `OKXDataClient::subscribe_book_deltas` rejects non-L2 MBP and resolves depth to supported channel sizes; WebSocket client validates accepted depths. | Fixture coverage must pin the depth/channel selection behavior. |
| OKX-ADP-005 | Scoped: Option instrument loading requires configured instrument families. | `resolve_instrument_families` skips `Option` when no `instrument_families` are configured. | Blocks treating Options as zero-config parity. |
| OKX-ADP-006 | Deferred: optional OKX Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, and optional PyO3 dependencies remain in the crate. | Blocks final Rust-only removal gate, but RADP-004 does not authorize deletion. |
| KRK-ADP-001 | Partial: Kraken product scope is clear but compact parity manifest coverage is missing. | `KrakenProductType` only includes `Spot` and `Futures`; factories match exactly those clients. There is no adapter-level manifest that maps supported/scoped/deferred surfaces to fixtures. | Blocks release-gate traceability until RADP-005/RADP-006 records closure decisions. |
| KRK-ADP-002 | Scoped: Kraken Spot demo environment is unsupported. | Config validation rejects `KrakenProductType::Spot` with `KrakenEnvironment::Demo`. | Spot demo must remain deferred unless Kraken support is implemented and validated. |
| KRK-ADP-003 | Scoped: Kraken Spot L3 is credentialed and has fixed depth limits. | Spot L3 rejects depths other than `10`, `100`, or `1000` and requires API credentials. | Fixture and docs should keep L3 as supported with authentication/depth constraints. |
| KRK-ADP-004 | Scoped: Kraken Spot and Futures data capability differs by product. | Integration docs list Spot bars but no Futures bar streaming; Spot lacks mark/index/funding while Futures supports those feeds. | Blocks generic Kraken data parity claims across Spot and Futures. |
| KRK-ADP-005 | Scoped: Kraken WebSocket order paths do not cover every order instruction. | WS order param builders reject trailing stops, iceberg/display quantity, FOK, and unsupported order/time-in-force combinations on specific paths. | Unsupported WS execution cases must stay routed to REST or remain deferred. |
| KRK-ADP-006 | Deferred: optional Kraken Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, and optional PyO3 dependencies remain in the crate. | Blocks final Rust-only removal gate, but RADP-004 does not authorize deletion. |

## Support Classification

| Adapter surface | Classification | Notes |
| --- | --- | --- |
| Bybit Spot data/execution | Supported with constraints | Docs and configs expose Spot. Quotes use order-book depth `1`; funding/mark/index limitations apply. |
| Bybit Linear/Inverse data/execution | Supported with constraints | Perpetual/futures paths exist. Order-book depths and environment/order-type caveats apply. |
| Bybit Options data/execution | Supported with constraints | Options fixtures and docs exist, but bars, funding, and conditional order classes are limited. |
| OKX Spot/Margin data/execution | Supported with constraints | Spot and margin are first-class config surfaces; spot margin behavior depends on `use_spot_margin` and margin mode. |
| OKX Swap/Futures data/execution | Supported with constraints | Derivative paths exist; order-book and order-type constraints apply. |
| OKX Options | Supported with constraints | Options are parsed and documented, but require instrument families for loading and do not support all order/algo paths. |
| OKX Spreads | Supported with constraints | Routed through spread endpoints and business WebSocket channels; order lists, conditional orders, FOK, and modify requests remain unsupported. |
| OKX Events | Supported with constraints | Events parse as `BinaryOption`; algo/order feature parity is narrower than standard instruments. |
| Kraken Spot | Supported with constraints | Spot live/demo split, Spot L3 credential/depth rules, and Spot-only bar streaming caveats apply. |
| Kraken Futures | Supported with constraints | Futures data/execution exists; Futures demo is allowed by URL config, but bar streaming differs from Spot. |
| Python/PyO3 bindings for all three adapters | Deferred for removal gate | Optional bridge remains until removal gates approve deletion. |

## Follow-Ups

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RADP-005 | Adapter & Integration Agent | Add or record executable fixtures for the Bybit, OKX, and Kraken surfaces listed above, especially product boundaries, order-book constraints, option/spread/event behavior, and Kraken Spot/Futures differences. |
| RADP-006 | Adapter & Integration Agent | Close the listed gaps by implementing, deferring, or documenting support decisions in compact adapter parity manifests. |
| RPROD follow-up | Rust Product Surface Agent | Promote Rust-first Bybit, OKX, and Kraken config/examples into user-facing Rust docs instead of leaving docs primarily Python-oriented. |
| RTRACE follow-up | Verification & Release Gatekeeper | Bind these adapter payload fixtures into golden trace checks where release gates require it. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove optional Python/PyO3 adapter surfaces only after Rust product, runtime, adapter, QA, and release gates approve removal. |

## Non-Goals Preserved

- No Bybit, OKX, or Kraken runtime code changes.
- No exchange protocol behavior changes.
- No public API changes.
- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No live exchange API calls and no real credential usage.
- No CI or release gate changes.
