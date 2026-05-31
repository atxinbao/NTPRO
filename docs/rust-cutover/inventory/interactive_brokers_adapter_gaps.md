# Interactive Brokers Rust Adapter Gap Inventory

Date: 2026-05-31
Executor: Codex
Task ID: RADP-016

## Scope

This inventory covers the Rust adapter under
`crates/adapters/interactive_brokers/`. It records current Rust-only parser,
data, historical, execution, fixture, and adapter-boundary gaps for the RADP-017
fixture task and RADP-018 closure task.

The task is inventory-only. It does not change adapter behavior, trading
semantics, IB protocol handling, credential handling, public APIs, Python/PyO3
bindings, Cython surfaces, Docker gateway behavior, or Cargo feature behavior.

`code-index` was not exposed as a callable local MCP tool in this session;
repository inspection used local `rg`, `find`, and targeted file reads instead.

## Rust Surface Inspected

| Surface | Current Rust surface |
| --- | --- |
| Crate/package | `nautilus-interactive-brokers` builds as an `rlib`. The default feature set enables `execution`; `python`, `gateway`, `examples`, and `extension-module` are optional. |
| Data | `InteractiveBrokersDataClient` wraps `ibapi` market-data subscriptions, request/response paths, historical replay helpers, quote cache, option-greeks cache, and subscription cancellation tokens. |
| Historical | `HistoricalInteractiveBrokersClient` requests historical bars, ticks, and instruments from IB contracts or Nautilus instrument IDs. |
| Execution | `InteractiveBrokersExecutionClient` supports submit, modify, cancel, order lists, account state, open-order queries, execution reports, position reports, and IB order-update streams. |
| Instruments | `InteractiveBrokersInstrumentProvider` resolves IB contracts and Nautilus instrument IDs, caches instruments, supports stock, FX, crypto, futures, options/FOP, index, CFD, commodity, bond, and BAG spread surfaces. |
| Gateway | `DockerizedIBGateway` is optional behind the `gateway` feature and uses Docker/Bollard configuration. |
| Python bridge | `src/python/**`, `cfg(feature = "python")`, optional PyO3 dependencies, and `extension-module` remain for the migration bridge. |

## Current Rust Evidence

| Area | Evidence |
| --- | --- |
| Parser/model conversion | Inline Rust tests cover quote ticks, trade ticks, realtime bars, bar-size conversion, timestamp conversion, execution timestamp parsing, order status parsing, IB enums, condition parsing, order tag parsing, and instrument parsing helpers. |
| Instrument provider | Provider code supports direct contract lookup, cache load/save, contract ID mappings, symbol/venue conversion, exchange fallback, option/futures chain loading, BAG spread loading, and security-type filtering. |
| Market data | Data client supports quote subscriptions, tick-by-tick quote fallback, index prices for index contracts, option greeks for option contracts, trade subscriptions, realtime and historical bars, market-depth book deltas, and request/response flows for instruments, quotes, trades, and bars. |
| Historical data | Historical client validates bar and tick request inputs, converts Nautilus bar specs to IB historical bar sizes, resolves contracts from instrument IDs, and segments long historical requests into IB-compatible durations. |
| Execution | Execution client validates readiness, rejects unsupported order shapes, transforms Nautilus orders to IB orders, tracks order IDs, handles order updates, parses fills, supports spread fill handling, and subscribes to account summary, PnL, and positions. |
| Live smoke | `tests/connection.rs` contains a paper TWS/Gateway smoke test gated by `NAUTILUS_IB_LIVE_TESTS=1`, so routine validation does not require live IB credentials or a running gateway. |

## Fixture And Test Coverage

| Adapter | Fixtures | Rust test surface |
| --- | --- | --- |
| Interactive Brokers | No checked-in `test_data/` fixture directory was found under `crates/adapters/interactive_brokers/`. Most evidence is inline unit tests and one env-gated live connection smoke. | 1 integration test file plus inline module tests; 204 annotated test entries found by local scan. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| IB-ADP-001 | Partial: the Rust adapter has broad inline tests, but no compact parity manifest or recorded payload fixture inventory. | `src/**` has parser, provider, data, historical, and execution tests; `tests/connection.rs` is live/env-gated; no `test_data/` fixture directory exists. | Blocks release-gate traceability until RADP-017/RADP-018 records supported, scoped, and deferred IB surfaces in fixture-backed evidence. |
| IB-ADP-002 | Scoped: IB requires an external TWS/IB Gateway session and UTC timestamp configuration. | README and `docs/integrations/ib.md` require TWS/IB Gateway, paper/live ports, optional Dockerized gateway, and UTC timestamps; live smoke is skipped unless `NAUTILUS_IB_LIVE_TESTS=1`. | Blocks zero-config Rust adapter claims and requires docs/examples to state gateway and UTC prerequisites. |
| IB-ADP-003 | Partial: instrument normalization is broad but still needs compact classification. | `parse_ib_contract_to_instrument` covers stock, FX, crypto, futures, options/FOP, index, CFD, commodity, bond, and BAG spreads; unsupported security types error or can be filtered. A provider TODO records Python callable filtering as not ported to Rust config. | Blocks clear supported/deferred product-family claims until fixture and closure tasks pin product support and filtered/unsupported secType behavior. |
| IB-ADP-004 | Scoped: market-data subscriptions are IB API specific, not generic Nautilus coverage. | L3 MBO book deltas are rejected; market depth is L1/L2 MBP style; index price subscriptions only run for `SecurityType::Index`; option greeks are option/FOP oriented; CurrencyPair trades are explicitly unsupported. | Release evidence must not claim generic book, greek, index, or trade support across every IB security type. |
| IB-ADP-005 | Scoped: historical request behavior has explicit IB pacing and bar/tick constraints. | `request_bars` rejects ambiguous start/duration inputs, maps only selected second/minute/hour/day/week/month bar sizes, maps unsupported price types to trades by default, and relies on IB duration segmentation. `request_ticks` requires contracts or instrument IDs and an IB tick type. | Fixture evidence must pin supported bar sizes, tick types, and request parameter boundaries before Rust-only release. |
| IB-ADP-006 | Partial: execution order lifecycle is implemented but constrained and needs fixture/golden evidence. | Submit rejects `post_only`, non-inverse quote quantity, unsupported trailing offset types, and unsupported order sides. Transform code maps order/TIF values, applies GTD expiry, account, display quantity, OCA/order-list tags, and IB-specific order tags. Cancel-all ignores order-side filtering by design. | Blocks generic execution parity claims until order transformation, rejection, cancel, and fill/update lifecycle decisions are fixture-backed. |
| IB-ADP-007 | Partial: account, position, PnL, reconnect, and gateway lifecycle rely on live IB streams or Docker state. | Execution connects to account summary, PnL, positions, open orders, execution data, and order updates; Dockerized gateway helpers are optional and tested separately; no recorded mock fixture currently captures these live operational paths. | Release gate needs mock, fixture, dry-run, or explicit scope decisions for operational lifecycle behavior. |
| IB-ADP-008 | Deferred: optional Interactive Brokers Python/PyO3 and extension surfaces remain. | `src/python/**`, `cfg(feature = "python")`, optional `pyo3` and `pyo3-async-runtimes`, and the `extension-module` feature remain in `Cargo.toml`. | Blocks final Rust-only removal gate, but RADP-016 does not authorize deletion. |

## Support Classification

| Adapter surface | Classification | Notes |
| --- | --- | --- |
| IB data client | Supported with constraints | Quotes, trades, bars, index prices, option greeks, and market depth exist, but security-type limits and IB subscription semantics must be explicit. |
| IB historical client | Supported with constraints | Historical bars, ticks, and instrument requests exist, with IB bar-size, tick-type, duration, and pacing boundaries. |
| IB execution client | Supported with constraints | Submit/modify/cancel/order-list/account/fill paths exist, but order shape, TIF, quote-quantity, trailing offset, cancel filtering, and UTC execution timestamp limits apply. |
| IB instrument provider | Supported with constraints | Broad product parsing and BAG spread support exist, but unsupported/filtered security types and the missing Rust replacement for Python callable filtering remain scoped. |
| Dockerized IB Gateway | Supported with constraints | Optional feature, Docker dependency, credentials, and external gateway state make this a dry-run/mock fixture candidate, not a routine local test dependency. |
| Live IB smoke | Deferred for routine automation | The connection smoke is env-gated and should not run in normal automated PR validation without explicit user-controlled credentials and gateway state. |
| Python/PyO3 bindings | Deferred for removal gate | Optional bridge remains until Rust product, runtime, adapter, QA, and release gates approve removal. |

## Follow-Ups

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RADP-017 | Adapter & Integration Agent | Add or record executable fixtures and compact manifests for IB parser, provider, data, historical, execution, account, and lifecycle surfaces listed above. |
| RADP-018 | Adapter & Integration Agent | Close the listed gaps by implementing, deferring, or documenting support decisions in compact adapter parity manifests. |
| RPROD follow-up | Rust Product Surface Agent | Promote Rust-first Interactive Brokers config, gateway, and live/sandbox examples into user-facing Rust docs instead of leaving docs primarily Python-oriented. |
| RTRACE follow-up | Verification & Release Gatekeeper | Bind IB adapter payload, order lifecycle, and operational fixtures into golden trace checks where release gates require it. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove optional Python/PyO3 adapter surfaces only after Rust product, runtime, adapter, QA, and release gates approve removal. |

## Non-Goals Preserved

- No Interactive Brokers runtime code changes.
- No market-data, historical-data, order, account, or gateway behavior changes.
- No external TWS/IB Gateway calls and no real credential usage.
- No public API changes.
- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No Cargo feature changes.
- No CI or release gate changes.
