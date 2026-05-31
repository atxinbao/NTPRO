# Betfair Architect AX Rust Adapter Gap Inventory

Date: 2026-05-31
Executor: Codex
Task ID: RADP-019

## Scope

This inventory covers the Rust adapters under `crates/adapters/betfair/` and
`crates/adapters/architect_ax/`. It records current Rust-only parser, data,
execution, fixture, and adapter-boundary gaps for the RADP-020 fixture task and
the RADP-021 closure task.

The task is inventory-only. It does not change adapter behavior, trading
semantics, exchange protocol handling, credential handling, public APIs,
Python/PyO3 bindings, Cython surfaces, or Cargo feature behavior.

`code-index` was not exposed as a callable local MCP tool in this session;
repository inspection used local `rg`, `find`, and targeted file reads instead.

## Rust Surfaces Inspected

| Adapter | Current Rust surface |
| --- | --- |
| Betfair | `nautilus-betfair` builds as an `rlib`. Optional features include `examples`, `high-precision`, and `python`. Rust config exposes Betfair REST, streaming market data, streaming order data, market/race navigation, betting exchange instruments, market book deltas, trades, market status/close events, BSP/custom data, account state, order submission, cancel/replace, order lists, and post-reconnect reconciliation. |
| Architect AX | `nautilus-architect-ax` builds as an `rlib` with an empty default feature set. Optional features include `examples`, `python`, and `extension-module`. Rust config exposes AX HTTP, public/private WebSocket clients, perpetual-futures instruments, order books, trades, bars, funding rates, market status, account/order/fill/position reports, market-order simulation, cancel-on-disconnect, and sandbox/production endpoints. |

## Current Rust Evidence

| Adapter | Evidence |
| --- | --- |
| Betfair | `BetfairDataClient` registers custom data, connects REST and stream clients, keeps sessions alive on a 10-hour interval, handles stream reconnect auth refresh, subscribes to market book deltas, trades, instrument status, instrument close, and race data. `BetfairExecutionClient` subscribes to order streams, halts exposure-increasing submits during post-reconnect reconciliation, fetches account state and mass status after reconnect, and maps standard, BSP limit-on-close, and market-on-close order paths. |
| Architect AX | `ArchitectAxDataClient` maps Nautilus book requests to AX market-data levels, subscribes to book deltas, quotes, trades, mark prices, bars, funding rates, and instrument status, and handles REST requests for instruments, books, trades, bars, and funding rates. `ArchitectAxExecutionClient` supports connect/disconnect, submit, modify, cancel, cancel-all, batch cancel, account/order/fill/position status generation, WebSocket order events, and simulated market orders through aggressive IOC limits. |

## Fixture And Test Coverage

| Adapter | Fixtures | Rust test surface |
| --- | --- | --- |
| Betfair | 71 files under `crates/adapters/betfair/test_data/`, covering stream market changes, order changes, race changes, connection/status messages, BSP settlement, REST auth, market catalogue, account details/funds, current/cleared orders, place/cancel/replace orders, batch partial failures, and compressed sample data. | Inline module tests plus crate tests; 511 annotated test entries found by local scan. |
| Architect AX | 52 files under `crates/adapters/architect_ax/test_data/`, covering HTTP auth, instruments, books, candles, trades, tickers, balances, positions, risk, orders, fills, transactions, funding, preview/place/replace/cancel flows, public market-data WebSocket payloads, and private order WebSocket payloads. | Inline module tests plus crate tests and benchmarks; 380 annotated test entries found by local scan. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| BF-ADP-001 | Partial: Betfair has broad fixtures and tests, but no compact parity manifest. | REST and stream fixtures cover many payload families, and inline tests cover parser/data/execution behavior, but no adapter-level manifest maps supported, scoped, and deferred surfaces to fixtures. | Blocks release-gate traceability until RADP-020/RADP-021 records fixture-backed support decisions. |
| BF-ADP-002 | Scoped: Betfair is a betting exchange adapter, not a generic multi-asset market-data venue. | Rust data subscriptions are market/race stream oriented and expose market book deltas, trades, instrument status, instrument close, and Betfair custom data. Generic quote/bar subscription surfaces are not the primary Betfair contract. | Release evidence must not claim generic quote, bar, or multi-asset market-data parity for Betfair. |
| BF-ADP-003 | Scoped: Betfair execution is betting-order specific and needs explicit order-shape classification. | Execution maps standard limit orders, BSP limit-on-close, BSP market-on-close, batch order lists, cancel/replace, current/cleared orders, and order-stream fills; unsupported order types error rather than silently mapping. | Blocks generic order lifecycle parity claims until order shape, BSP, liability, customer order reference, and rejection behavior are fixture-backed. |
| BF-ADP-004 | Partial: Betfair reconnect and operational lifecycle are implemented but need compact release evidence. | Data and execution clients refresh session auth after stream reconnect, run keep-alive tasks, and execution halts exposure-increasing commands while post-reconnect mass status is reconciled. | Release gate needs a manifest decision for reconnect, keep-alive, session expiry, and mass-status recovery behavior without requiring live credentials. |
| BF-ADP-005 | Scoped: Betfair custom data currently uses `#[custom_data(pyo3)]` generated surfaces. | `src/data_types.rs` registers Betfair ticker, starting price, BSP book delta, sequence completed, order voided, race runner, and race progress custom data through the persistence macro with PyO3 generation enabled. | Blocks final Rust-only removal gate until custom data and Python/PyO3 surfaces are explicitly addressed by later removal work. |
| BF-ADP-006 | Scoped: live operation requires Betfair app key, session token/certificate flow, and stream access. | REST login, certificate login, keep-alive, stream auth refresh, market stream, race stream, and order stream paths exist; fixtures cover payloads but do not make real Betfair calls. | Routine automation must stay on mock/fixture/dry-run validation and must not require real Betfair credentials. |
| BF-ADP-007 | Deferred: optional Betfair Python/PyO3 surfaces remain. | `src/python/**`, `cfg(feature = "python")`, optional PyO3 dependencies, and Python-enabled custom data remain in the crate. | Blocks final Rust-only removal gate, but RADP-019 does not authorize deletion. |
| AX-ADP-001 | Partial: Architect AX has broad fixtures and tests, but no compact parity manifest. | HTTP and WebSocket fixtures cover public market data, private orders, auth, account, risk, fills, positions, transactions, and funding, but no adapter-level manifest maps support decisions to fixtures. | Blocks release-gate traceability until RADP-020/RADP-021 records fixture-backed support decisions. |
| AX-ADP-002 | Scoped: Architect AX product support is AX perpetual futures, not a generic venue surface. | README and Rust code target AX Exchange perpetual futures on traditional underlying assets, with AX-specific instruments, accounts, public market-data streams, and private order streams. | Release evidence must not claim spot/options/multi-asset support outside AX's intended product surface. |
| AX-ADP-003 | Scoped: AX market-data support has explicit protocol limits. | `BookType::L1_MBP` book deltas are downgraded to `LEVEL_2`; index prices and instrument close are logged as unsupported; bar specs and market-data levels are AX-specific. | Fixture evidence must pin L1 downgrade, unsupported index/close behavior, bar spec limits, and accepted book levels. |
| AX-ADP-004 | Scoped: AX execution supports a limited order and TIF matrix. | Order WebSocket docs and execution code limit supported order types to market, limit, and stop-limit; market orders are simulated as IOC aggressive limits; `GTD` is rejected. | Blocks generic order lifecycle parity claims until order type, TIF, simulated market order, and rejection decisions are fixture-backed. |
| AX-ADP-005 | Scoped: AX auth/session and endpoint behavior need release-gate evidence. | The adapter authenticates to obtain bearer tokens, supports sandbox/production endpoint selection, public market-data WebSockets, private order WebSockets, and cancel-on-disconnect behavior. | Routine automation must stay on fixtures, schema validation, and dry-run checks; no real AX credential calls should be required. |
| AX-ADP-006 | Deferred: optional Architect AX Python/PyO3 and extension surfaces remain. | `src/python/**`, `cfg(feature = "python")`, optional PyO3 dependencies, and the `extension-module` feature remain in `Cargo.toml`. | Blocks final Rust-only removal gate, but RADP-019 does not authorize deletion. |

## Support Classification

| Adapter surface | Classification | Notes |
| --- | --- | --- |
| Betfair market data | Supported with constraints | Market book deltas, trades, instrument status/close, race data, and Betfair custom data exist; generic quote/bar parity is scoped out. |
| Betfair execution | Supported with constraints | Betting-order submit/cancel/replace, order lists, account state, fills, BSP order types, and reconnect reconciliation exist; unsupported order shapes must stay explicit. |
| Betfair REST and stream lifecycle | Supported with constraints | REST login/keep-alive/reconnect and stream auth refresh exist; local automation should validate through fixtures and mocks only. |
| Betfair custom data | Supported with constraints | Betfair custom data registration exists, but PyO3-generated custom data remains a removal-gate concern. |
| Architect AX market data | Supported with constraints | Books, quotes, trades, bars, funding, mark prices, and status exist; L1 downgrade and unsupported index/close surfaces must remain visible. |
| Architect AX execution | Supported with constraints | Submit/modify/cancel/account/order/fill/position paths exist; market order simulation, limited order types, and GTD rejection are part of the scope. |
| Architect AX auth and endpoints | Supported with constraints | Token auth, sandbox/production endpoints, and public/private WebSocket paths exist; no routine validation should require live credentials. |
| Architect AX Python/PyO3 bindings | Deferred for removal gate | Optional bridge remains until Rust product, runtime, adapter, QA, and release gates approve removal. |

## Follow-Ups

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RADP-020 | Adapter & Integration Agent | Add or record executable fixtures and compact manifests for the Betfair and Architect AX surfaces listed above, especially Betfair betting-order scope, stream reconnect lifecycle, custom data, AX L1 downgrade, unsupported market-data surfaces, order/TIF limits, and auth/session boundaries. |
| RADP-021 | Adapter & Integration Agent | Close the listed gaps by implementing, deferring, or documenting support decisions in compact adapter parity manifests. |
| RPROD follow-up | Rust Product Surface Agent | Promote Rust-first Betfair and Architect AX configuration and examples into user-facing Rust docs without implying unsupported generic venue parity. |
| RTRACE follow-up | Verification & Release Gatekeeper | Bind Betfair and Architect AX payload fixtures into golden trace checks where release gates require it. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove optional Python/PyO3 adapter surfaces only after Rust product, runtime, adapter, QA, and release gates approve removal. |

## Non-Goals Preserved

- No Betfair or Architect AX runtime code changes.
- No market-data, order, account, reconnect, or external protocol behavior
  changes.
- No live exchange API calls and no real credential usage.
- No public API changes.
- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No Cargo feature changes.
- No CI or release gate changes.
