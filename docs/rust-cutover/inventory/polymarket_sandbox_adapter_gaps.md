# Polymarket Sandbox Rust Adapter Gap Inventory

Date: 2026-05-31
Executor: Codex
Task ID: RADP-022

## Scope

This inventory covers the Rust adapters under `crates/adapters/polymarket/`
and `crates/adapters/sandbox/`. It records current Rust-only parser, data,
execution, fixture, and adapter-boundary gaps for the RADP-023 fixture task and
the RADP-024 closure task.

The task is inventory-only. It does not change adapter behavior, trading
semantics, CLOB protocol handling, sandbox matching behavior, credential
handling, public APIs, Python/PyO3 bindings, Cython surfaces, or Cargo feature
behavior.

`code-index` was not exposed as a callable local MCP tool in this session;
repository inspection used local `rg`, `find`, and targeted file reads instead.

## Rust Surfaces Inspected

| Adapter | Current Rust surface |
| --- | --- |
| Polymarket | `nautilus-polymarket` builds as an `rlib` by default with high precision enabled. Optional features include `examples`, `python`, and `extension-module`. Rust config exposes CLOB REST, CLOB WebSocket, Gamma discovery, Data API trades/positions, binary option instruments, data subscriptions, L1/L2 authentication, EIP-712 signing, order submit/cancel, account/order/fill/position reports, and reconciliation helpers. |
| Sandbox | `nautilus-sandbox` builds as an `rlib` by default with high precision enabled. Optional features include `python`, `extension-module`, and `example-databento`. Rust config exposes a simulated execution client backed by `OrderMatchingEngine`, per-instrument matching engines, cash/margin account configuration, book type, bar/trade execution toggles, stop/GTD/contingent/reduce-only switches, account-state generation, and order event emission. |

## Current Rust Evidence

| Adapter | Evidence |
| --- | --- |
| Polymarket | `PolymarketDataClient` loads instruments from Gamma, resolves CLOB token IDs, requests instruments, books, and trades, subscribes to L2 book deltas, quotes, and trades, supports automatic missing-instrument loading, and handles market WebSocket messages for books, price changes, trades, tick-size changes, and new markets. `PolymarketExecutionClient` signs orders, submits limit and market orders, handles order lists, cancel, cancel-all, batch cancel, account query, order/fill/position reports, mass status, WebSocket order/fill dispatch, and unknown-submit recovery. |
| Sandbox | `SandboxExecutionClient` starts, stops, connects, emits starting account state, registers data message handlers, lazily creates matching engines per instrument, feeds quote/trade/bar/book data into `OrderMatchingEngine`, submits/modifies/cancels orders and order lists, and emits generated order events without external exchange calls. |

## Fixture And Test Coverage

| Adapter | Fixtures | Rust test surface |
| --- | --- | --- |
| Polymarket | 45 files under `crates/adapters/polymarket/test_data/`, covering Gamma markets/events/tags/search, CLOB books, fee rates, signed orders, balances, open orders, trades, cancel responses, Data API positions/trades, and market/user WebSocket payloads. | Inline module tests plus `tests/data_client.rs`, `tests/exec_client.rs`, `tests/http.rs`, and `tests/websocket.rs`; 647 annotated test entries found by local scan. |
| Sandbox | No checked-in `test_data/` fixture directory was found under `crates/adapters/sandbox/`. Evidence is inline module tests plus `tests/execution.rs`. | Config, lifecycle, matching-engine, market-data, order, cancel, binary option settlement, and account-state tests; 40 annotated test entries found by local scan. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| PM-ADP-001 | Partial: Polymarket has broad fixtures and tests, but no compact parity manifest. | The crate has 45 checked-in payload fixtures and 647 test annotations across HTTP, data, execution, WebSocket, signing, parser, and reconciliation surfaces, but no adapter-level manifest maps supported, scoped, and deferred surfaces to fixtures. | Blocks release-gate traceability until RADP-023/RADP-024 records fixture-backed support decisions. |
| PM-ADP-002 | Scoped: Polymarket is a prediction-market CLOB adapter for binary outcome contracts, not a generic multi-asset venue. | README and Rust parsing create `BinaryOption` instruments from Gamma/CLOB token IDs. Data and execution paths rely on Polymarket condition IDs, token IDs, pUSD collateral, and CLOB-specific APIs. | Release evidence must not claim generic spot, futures, options, or multi-asset venue parity. |
| PM-ADP-003 | Scoped: Polymarket market-data support has explicit protocol limits. | Data subscriptions support L2 MBP book deltas, quotes derived from snapshots/price changes, trades, automatic instrument loading, and new-market discovery. `subscribe_book_deltas` rejects non-`L2_MBP`; historical bars and generic quote/bar venue parity are not the primary Polymarket contract. | Fixture evidence must pin L2-only book behavior, quote/trade derivation, auto-load behavior, new-market discovery, and unsupported data surfaces. |
| PM-ADP-004 | Scoped: Polymarket execution supports a limited order and instruction matrix. | Execution accepts limit and market orders, routes market orders through a single-order path, supports batch limit order lists, cancel, cancel-all, and batch cancel. It rejects unsupported order types, reduce-only orders, unsupported market TIFs, quote quantity for limit orders, and order modification. | Blocks generic order-lifecycle claims until supported order/TIF/quantity semantics, denial reasons, cancel flows, and unknown-submit recovery are fixture-backed. |
| PM-ADP-005 | Scoped: Polymarket authentication, signing, and operational setup need release-gate evidence. | Config supports L1 EIP-712 signing, L2 HMAC API credentials, signature types, wallet/funder values, WebSocket credentials, retry config, and separate CLOB/Gamma/Data endpoints. Docs require wallet setup, allowances, API credentials, and rate-limit/subscription awareness. | Routine automation must stay on mock/fixture/dry-run validation and must not require real wallets, private keys, API credentials, allowances, or live Polymarket calls. |
| PM-ADP-006 | Partial: account, position, fill, reconciliation, and unknown-submit behavior are implemented but need compact release evidence. | Execution builds account state from balance allowance, order reports from active orders, fill reports from trades, position reports from Data API positions, mass status through reconciliation, and terminal recovery when an order lookup is absent. | Release gate needs a compact manifest for active-order limitations, position synthesis, terminal recovery, fill dedup/snap behavior, and Data API dependencies. |
| PM-ADP-007 | Scoped: Polymarket fee and precision behavior is venue-specific. | Rust execution computes taker commission from instrument fee schedule, snaps dust drift on fills, adjusts market-buy amounts for fee coverage, and has parser tests for fee schedules and commission math. Docs also describe category-specific fees and Python-facing backtest fee-model configuration. | Fixture and closure evidence must avoid implying generic fee model parity and must separate Rust live execution fees from remaining Python-facing backtest fee-model docs. |
| PM-ADP-008 | Deferred: optional Polymarket Python/PyO3 and extension surfaces remain. | `src/python/**`, `cfg(feature = "python")`, optional PyO3 dependencies, Python config/factory/sort helpers, and the `extension-module` feature remain in `Cargo.toml`. | Blocks final Rust-only removal gate, but RADP-022 does not authorize deletion. |
| SBX-ADP-001 | Partial: Sandbox has strong Rust lifecycle tests but no compact adapter fixture or parity manifest. | `tests/execution.rs` covers config, start/stop/connect/disconnect, message handling, order submission, cancel paths, precision filtering, and binary option settlement, but there is no `test_data/` fixture directory or adapter-level manifest. | Blocks release-gate traceability until RADP-023/RADP-024 records sandbox lifecycle and matching support decisions. |
| SBX-ADP-002 | Scoped: Sandbox is simulated execution only, not a data provider or external venue adapter. | The crate provides an execution client and matching-engine integration. Market data must arrive from cache/msgbus or another data source; there is no sandbox data client, REST client, or WebSocket client. | Release evidence must not claim sandbox market-data provider parity or external exchange protocol parity. |
| SBX-ADP-003 | Scoped: matching behavior depends on configured matching-engine and market-data inputs. | Config exposes `book_type`, `bar_execution`, `trade_execution`, `reject_stop_orders`, `support_gtd_orders`, `support_contingent_orders`, `use_position_ids`, `use_random_ids`, and `use_reduce_only`. Execution drops quote/trade/bar inputs whose precision does not match the instrument. | Fixture evidence must pin configuration defaults, precision-drop behavior, and which input event types drive simulated fills. |
| SBX-ADP-004 | Partial: report generation is intentionally internal and currently returns empty optional report payloads. | `query_account` emits current account state, while `query_order`, `generate_order_status_report(s)`, `generate_fill_reports`, `generate_position_status_reports`, and `generate_mass_status` return no external reports because sandbox state is internal/cache-driven. | Release evidence must scope this behavior so QA does not expect exchange-style REST reconciliation from sandbox. |
| SBX-ADP-005 | Deferred: optional Sandbox Python/PyO3 and extension surfaces remain. | `src/python/**`, `cfg(feature = "python")`, optional PyO3 dependencies, Python config/factory helpers, and the `extension-module` feature remain in `Cargo.toml`. | Blocks final Rust-only removal gate, but RADP-022 does not authorize deletion. |

## Support Classification

| Adapter surface | Classification | Notes |
| --- | --- | --- |
| Polymarket instrument discovery | Supported with constraints | Gamma and search/event/tag discovery exist, with binary outcome token assumptions, filters, auto-load, transient retry, and rate-limit boundaries. |
| Polymarket market data | Supported with constraints | L2 book deltas, snapshot-derived quotes, price-change quotes, trades, tick-size/new-market events, and book snapshots exist; generic bars and non-L2 books are scoped out. |
| Polymarket execution | Supported with constraints | Limit/market submit, order lists, cancel, cancel-all, batch cancel, account/order/fill/position reports, WebSocket dispatch, and reconciliation exist with venue-specific order/TIF/quantity limits. |
| Polymarket signing and credentials | Supported with constraints | EIP-712 signing and HMAC credentials exist; local automation must use fixtures and mock servers, never real private keys or API credentials. |
| Polymarket fees and precision | Supported with constraints | Rust live execution computes venue-specific commissions and fill dust handling, but Python-facing backtest fee model docs remain a product-surface follow-up. |
| Polymarket Python/PyO3 bindings | Deferred for removal gate | Optional bridge remains until Rust product, runtime, adapter, QA, and release gates approve removal. |
| Sandbox execution lifecycle | Supported with constraints | Start/stop/connect/disconnect, starting account state, matching-engine registration, and order event emission exist without external network dependencies. |
| Sandbox matching simulation | Supported with constraints | Quote/trade/bar/book inputs drive per-instrument matching engines; behavior is controlled by sandbox and matching-engine config. |
| Sandbox reconciliation reports | Scoped internal behavior | Sandbox is cache/internal-state driven and does not provide external venue-style report queries. |
| Sandbox Python/PyO3 bindings | Deferred for removal gate | Optional bridge remains until Rust product, runtime, adapter, QA, and release gates approve removal. |

## Follow-Ups

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RADP-023 | Adapter & Integration Agent | Add or record executable fixtures and compact manifests for Polymarket parser/data/execution/reconciliation/signing surfaces and Sandbox lifecycle/matching/report surfaces listed above. |
| RADP-024 | Adapter & Integration Agent | Close the listed gaps by implementing, deferring, or documenting support decisions in compact adapter parity manifests. |
| RPROD follow-up | Rust Product Surface Agent | Promote Rust-first Polymarket and Sandbox configuration/examples into user-facing Rust docs without implying unsupported generic venue parity. |
| RTRACE follow-up | Verification & Release Gatekeeper | Bind Polymarket adapter payloads and Sandbox lifecycle/matching events into golden trace checks where release gates require it. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove optional Python/PyO3 adapter surfaces only after Rust product, runtime, adapter, QA, and release gates approve removal. |

## Non-Goals Preserved

- No Polymarket or Sandbox runtime code changes.
- No market-data, order, account, reconciliation, matching, or external
  protocol behavior changes.
- No live Polymarket API calls, wallet signing against real funds, allowance
  transactions, or real credential usage.
- No public API changes.
- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No Cargo feature changes.
- No CI or release gate changes.
