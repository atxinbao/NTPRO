# Databento Tardis Rust Adapter Gap Inventory

Date: 2026-05-31
Executor: Codex
Task ID: RADP-013

## Scope

This inventory covers the Rust adapters under `crates/adapters/databento/` and
`crates/adapters/tardis/`. It records current Rust-only parser, data, fixture,
and adapter-boundary gaps for the RADP-014 fixture task and the RADP-015 closure
task.

The task is inventory-only. It does not change adapter behavior, trading
semantics, market-data decoding, credential handling, public APIs, Python/PyO3
bindings, Cython surfaces, or Cargo feature behavior.

`code-index` was not exposed as a callable local MCP tool in this session;
repository inspection used local `rg`, `find`, and targeted file reads instead.

## Rust Surfaces Inspected

| Adapter | Current Rust surface |
| --- | --- |
| Databento | `nautilus-databento` builds as an `rlib`. The default feature set is `live` plus `high-precision`; `arrow`, `python`, and `extension-module` are optional. Rust modules expose DBN decoding, historical range loading, live LSG feed handling, symbology, Arrow integration, custom Databento data types, and live data-client factories. |
| Tardis | `nautilus-tardis` builds as an `rlib`. The default feature set is `replay` plus `high-precision`; `examples`, `python`, and `extension-module` are optional. Rust modules expose Tardis HTTP instrument bootstrap, Tardis Machine replay/stream parsing, CSV streams, Parquet replay output, data-client factories, and normalized exchange instruments. |

## Current Rust Evidence

| Adapter | Evidence |
| --- | --- |
| Databento | DBN fixtures cover definition, trades, MBO, MBP-1, MBP-10, BBO, CBBO, CMBP, TBBO, OHLCV, imbalance, statistics, and status schemas. Rust historical paths fetch and decode instruments, order-book deltas, depth10, quotes, trades, bars, imbalance, statistics, and status. The live feed handler tests cover authentication, subscriptions, protocol messages, reconnection, backoff, buffered commands, price precision fallback, and Databento LSG schema handling. |
| Tardis | JSON fixtures cover trade, bar, book change, book snapshot, derivative ticker, disconnect, and spot/perpetual/future/option/combo instruments. CSV fixtures cover deltas, deltas with snapshot, derivative ticker, and trades. Rust tests cover HTTP instruments, WebSocket replay/stream messages, disconnect filtering, data-client lifecycle, CSV parsing, replay output selection, bar spec conversion, and instrument construction. |

## Fixture And Test Coverage

| Adapter | Fixtures | Rust test surface |
| --- | --- | --- |
| Databento | 18 files under `crates/adapters/databento/test_data/`, covering DBN schemas for definitions, books, quotes, trades, bars, imbalance, statistics, and status. | 1 integration test file plus inline module tests; 209 annotated test entries found by local scan. |
| Tardis | 15 files under `crates/adapters/tardis/test_data/`, covering normalized WebSocket messages, instruments, disconnect events, and CSV streams. | 3 integration test files plus inline module tests; 162 annotated test entries found by local scan. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| DBN-ADP-001 | Partial: Databento has broad DBN fixtures and tests, but no compact parity manifest. | Fixtures and tests exercise many DBN schemas and live feed-handler paths, but no adapter-level manifest maps supported, scoped, and deferred surfaces to fixture evidence. | Blocks release-gate traceability until RADP-014/RADP-015 records fixture-backed support decisions. |
| DBN-ADP-002 | Scoped: Databento is a market-data adapter, not an execution adapter. | The crate exposes data, historical, loader, live, symbology, and type modules; it does not expose an execution client or order lifecycle surface. | Blocks any claim that Databento closes execution adapter parity. |
| DBN-ADP-003 | Scoped: live data is dataset, publisher, and venue-map driven. | The live data client resolves datasets from venues, sends schema-specific LSG subscriptions, and relies on publisher and symbol venue maps for Nautilus instrument IDs. | Fixture and docs must keep venue/dataset mapping explicit before Rust-only release. |
| DBN-ADP-004 | Scoped: Databento unsubscribe and historical request behavior is not generic. | Live unsubscribe methods log that Databento does not support granular unsubscribing. Historical request helpers still contain fixed `GLBX.MDP3` dataset and symbol-map TODOs in request paths. | Blocks generic subscription lifecycle claims until supported request paths are documented or closed. |
| DBN-ADP-005 | Scoped: historical book and bar surfaces support selected schemas. | Order-book depth fetches are limited to depth 10, bars map only second/minute/hour/day aggregations to DBN OHLCV schemas, and unsupported bar aggregations error or log. | Release evidence must pin accepted depth and bar aggregation scope. |
| DBN-ADP-006 | Partial: Databento custom/statistical schemas are supported only for modeled values. | Imbalance, status, and statistics paths exist, but unsupported `stat_type` values are skipped with warnings instead of becoming modeled Nautilus data. | Golden trace and adapter parity manifests must record skipped/unmodeled Databento schema behavior. |
| DBN-ADP-007 | Deferred: optional Databento Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, optional PyO3 dependencies, and Python module registration remain in the crate. | Blocks final Rust-only removal gate, but RADP-013 does not authorize deletion. |
| TDS-ADP-001 | Partial: Tardis has broad parser, replay, HTTP, CSV, and WebSocket tests, but no compact parity manifest. | Fixtures and tests cover common normalized messages and instruments, but no adapter-level manifest maps support decisions to fixture evidence. | Blocks release-gate traceability until RADP-014/RADP-015 records fixture-backed support decisions. |
| TDS-ADP-002 | Scoped: Tardis is a market-data adapter, not an execution adapter. | The crate exposes HTTP, machine, CSV, replay, data-client, and factory modules; it does not expose an execution client or order lifecycle surface. | Blocks any claim that Tardis closes execution adapter parity. |
| TDS-ADP-003 | Scoped: Tardis data-client startup requires explicit replay or stream options. | `TardisDataClient::connect` errors when both `options` and `stream_options` are empty; replay and stream clients also reject empty options. | Blocks zero-config Rust live-data claims for Tardis. |
| TDS-ADP-004 | Scoped: Tardis replay output is constrained to selected Nautilus data types. | Replay writes deltas, depth10, quotes, trades, and bars; mark price, index price, status, close, custom, and individual delta messages are skipped or routed by configured output mode. | Release evidence must make replay output scope explicit. |
| TDS-ADP-005 | Scoped: Tardis bar and book conversions are venue-message specific. | Bar parsing accepts millisecond, second, minute, tick, and volume suffixes, with hour/day normalized through minute strings; unsupported aggregation suffixes fail. Book snapshots are converted according to the configured `deltas` or `depth10` output mode. | Fixture coverage must pin accepted aggregation strings and book snapshot output modes. |
| TDS-ADP-006 | Partial: Tardis instrument normalization is broad but still fixture-classification dependent. | HTTP parsing creates spot, perpetual, future, option, and combo instruments and stores raw instrument metadata, but the support matrix is not captured in a compact adapter parity manifest. | Blocks clear supported/deferred decisions for multi-exchange Rust-only release docs. |
| TDS-ADP-007 | Deferred: optional Tardis Python/PyO3 surfaces and one Python-featured model dependency remain. | `src/python/**`, `cfg(feature = "python")`, optional PyO3 dependencies, and `nautilus-model` with `features = ["python"]` remain in the crate manifest. | Blocks final Rust-only removal gate, but RADP-013 does not authorize Cargo or binding changes. |

## Support Classification

| Adapter surface | Classification | Notes |
| --- | --- | --- |
| Databento historical DBN loaders | Supported with constraints | DBN schema coverage is broad, but supported schemas and skipped values need compact manifest evidence. |
| Databento live LSG feed handler | Supported with constraints | Authentication, schema subscriptions, reconnection, and price precision fallback have tests; dataset and publisher mapping remain explicit user/config concerns. |
| Databento execution | Scoped out | Databento is market-data only in this workspace. |
| Databento granular unsubscribe | Scoped out | Current live client logs and ignores granular unsubscribe because Databento LSG does not support it. |
| Databento Python/PyO3 bindings | Deferred for removal gate | Optional bridge remains until Rust product, runtime, adapter, QA, and release gates approve removal. |
| Tardis Machine replay and stream | Supported with constraints | Requires explicit replay or stream options and a reachable Tardis Machine endpoint. |
| Tardis HTTP instrument bootstrap | Supported with constraints | Spot, perpetual, future, option, and combo instruments are parsed from normalized metadata. |
| Tardis CSV and Parquet replay output | Supported with constraints | CSV fixtures exist and replay writes supported Nautilus data outputs; unsupported data types are skipped. |
| Tardis execution | Scoped out | Tardis is market-data only in this workspace. |
| Tardis Python/PyO3 bindings | Deferred for removal gate | Optional bridge remains until removal gates approve deletion; Cargo feature cleanup is a later removal task. |

## Follow-Ups

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RADP-014 | Adapter & Integration Agent | Add or record executable fixtures for Databento and Tardis surfaces listed above, especially compact manifests, Databento schema support, Databento live dataset boundaries, Tardis replay/stream options, Tardis book/bar conversions, and data-only adapter scope. |
| RADP-015 | Adapter & Integration Agent | Close the listed gaps by implementing, deferring, or documenting support decisions in compact adapter parity manifests. |
| RPROD follow-up | Rust Product Surface Agent | Promote Rust-first Databento and Tardis config/examples into user-facing Rust docs instead of leaving docs primarily crate-level. |
| RTRACE follow-up | Verification & Release Gatekeeper | Bind Databento DBN and Tardis normalized payload fixtures into golden trace checks where release gates require it. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove optional Python/PyO3 adapter surfaces and Tardis Python-featured dependency paths only after Rust product, runtime, adapter, QA, and release gates approve removal. |

## Non-Goals Preserved

- No Databento or Tardis runtime code changes.
- No market-data decoding or replay behavior changes.
- No exchange/data-provider protocol behavior changes.
- No public API changes.
- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No live data-provider API calls and no real credential usage.
- No CI or release gate changes.
