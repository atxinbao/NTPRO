# NTPRO Adapter Support Matrix

Date: 2026-06-04
Executor: Codex
Task ID: NADAPT-001

## Purpose

This matrix records the current NTPRO v0.2.0 adapter support stance for every
adapter crate in the Cargo workspace.

The status labels are intentionally conservative:

- `supported`: Rust adapter code exists and is a v0.2.0 support candidate with
  explicit product/protocol constraints. Routine validation still uses
  fixtures, mocks, dry-runs, or sandbox paths, not real credentials.
- `sandbox-only`: internal simulated execution support only.
- `fixture-only`: useful as data/replay/provider evidence, but not an execution
  venue support claim.
- `deferred`: present in the workspace but not a v0.2.0 supported product
  adapter until a later task supplies local evidence and a scope decision.
- `removed`: not present or intentionally removed from the workspace.

No status in this document authorizes calls to live trading APIs, hardcoded
secrets, or production order flow.

## Workspace Adapter Inventory

The workspace currently contains 17 adapter crates:

```text
crates/adapters/architect_ax
crates/adapters/betfair
crates/adapters/binance
crates/adapters/bitmex
crates/adapters/blockchain
crates/adapters/bybit
crates/adapters/coinbase
crates/adapters/databento
crates/adapters/deribit
crates/adapters/dydx
crates/adapters/hyperliquid
crates/adapters/interactive_brokers
crates/adapters/kraken
crates/adapters/okx
crates/adapters/polymarket
crates/adapters/sandbox
crates/adapters/tardis
```

## Matrix

| Adapter | Crate | Status | Current supported scope | Deferred or scoped-out surfaces | Evidence strategy |
| --- | --- | --- | --- | --- | --- |
| Architect AX | `nautilus-architect-ax` | supported | AX perpetual-futures data/execution, auth, public/private WebSockets, order/fill/position/account paths with AX-specific limits. | Generic multi-asset parity, unsupported index/close subscriptions, unsupported order/TIF paths, optional Python/PyO3 removal. | Fixture-backed parser and protocol tests; no routine live credential calls. |
| Betfair | `nautilus-betfair` | supported | Betting exchange market/race streams, account state, betting order lifecycle, BSP order paths, reconnect/keep-alive behavior with constraints. | Generic quote/bar venue parity, generic order-type parity, optional Python/PyO3 removal. | Fixture-backed REST/stream/order payload tests; no routine app-key/cert live calls. |
| Binance | `nautilus-binance` | supported | Spot, USD-M Futures, and COIN-M Futures data/execution with L2 MBP book constraints and scoped custom data. | Margin and Options are deferred; multi-product single-client parity and optional Python/PyO3 removal remain deferred. | Fixture-backed Spot/Futures HTTP, WebSocket, user-data, and execution tests. |
| BitMEX | `nautilus-bitmex` | supported | Market data, authenticated margin/netting execution, L2/depth10 books, limited bars, broadcaster/dead-man paths with constraints. | Options, swaps, reference baskets, legacy futures, futures spreads, stock perpetuals, and optional Python/PyO3 removal. | Fixture-backed HTTP/WebSocket/data/execution tests; live API keys not required. |
| Blockchain / DeFi | `nautilus-blockchain` | deferred | DeFi data ingestion crate exists, including RPC/Hypersync examples and turmoil simulation support. | Not classified as a v0.2.0 trading venue adapter; live RPC endpoints and Postgres-backed sync remain outside default support. | Defer to a dedicated `NDEFI-*` or adapter task with mock RPC/turmoil evidence. |
| Bybit | `nautilus-bybit` | supported | Spot, linear/inverse derivatives, and options data/execution with product-specific constraints. | Unsupported book depths, unsupported data combinations, demo trade WebSocket caveats, optional Python/PyO3 removal. | Fixture-backed HTTP/WebSocket/account/order/position tests. |
| Coinbase | `nautilus-coinbase` | supported | Coinbase spot and CFM derivatives data/execution, L2 books, account/order/fill paths with order/TIF limits. | Generic Coinbase product parity, WebSocket mark prices, mixed account execution, optional Python/PyO3 removal. | Fixture-backed HTTP/WebSocket/data/execution tests. |
| Databento | `nautilus-databento` | fixture-only | Historical DBN loaders and live LSG feed-handler paths for market data schemas. | Execution is scoped out; granular unsubscribe and some historical request paths remain constrained. | DBN fixtures and live feed-handler tests; no live Databento credentials in routine validation. |
| Deribit | `nautilus-deribit` | supported | Futures/perpetuals, options, spot and combo instruments, market data, greeks, margin execution with Deribit order constraints. | Zero-config non-futures support, unsupported book modes/depths, unsupported order/TIF paths, optional Python/PyO3 removal. | Fixture-backed HTTP/WebSocket/data/execution tests. |
| dYdX | `nautilus-dydx` | supported | dYdX v4 perpetual market data/execution, Indexer channels, block-height and transaction paths with constraints. | Spot/options products are scoped out; several Nautilus order/TIF forms are unsupported; optional Python/PyO3 removal. | Fixture-backed HTTP/WebSocket/gRPC/parser/execution tests. |
| Hyperliquid | `nautilus-hyperliquid` | supported | Perp and spot data/execution plus HIP-4 outcome support where fixtures/tests exist. | Sparse fixture inventory, outcome metadata best-effort behavior, unsupported TIF/order paths, optional Python/PyO3 removal. | Fixture and inline test evidence; follow-up manifest required before high-confidence release claims. |
| Interactive Brokers | `nautilus-interactive-brokers` | deferred | Broad Rust parser/provider/data/historical/execution code exists with inline tests. | Routine support is deferred because there is no broad checked-in fixture inventory and live smoke requires TWS/IB Gateway plus env-gated credentials. | Defer routine automation to mock/fixture/gateway-sim evidence under later `NADAPT-*` work. |
| Kraken | `nautilus-kraken` | supported | Spot and Futures data/execution with Spot/Futures product differences and credentialed Spot L3 constraints. | Kraken Spot demo, unsupported WS order instructions, product-specific data gaps, optional Python/PyO3 removal. | Fixture-backed Spot/Futures HTTP/WebSocket/data/execution tests. |
| OKX | `nautilus-okx` | supported | Spot, margin, swap, futures, options, spreads, and events with explicit product/order/book constraints. | Option/event algo parity, spread limitations, zero-config option families, optional Python/PyO3 removal. | Fixture-backed HTTP/WebSocket/order/book/product tests. |
| Polymarket | `nautilus-polymarket` | supported | Binary outcome CLOB discovery, market data, signing, order submit/cancel, account/order/fill/position reports with venue-specific limits. | Generic spot/futures/options parity, live wallet/API credential flow, optional Python/PyO3 removal. | Fixture-backed Gamma/CLOB/Data API/WebSocket/signing tests; no real private keys in validation. |
| Sandbox | `nautilus-sandbox` | sandbox-only | Internal simulated execution lifecycle, matching-engine integration, account-state generation, and order-event emission. | Not a data provider or external exchange adapter; external venue-style reports are scoped internal/cache-driven. | Cargo live/sandbox smoke, execution tests, and sandbox lifecycle fixtures. |
| Tardis | `nautilus-tardis` | fixture-only | Tardis Machine replay/stream, HTTP instrument bootstrap, CSV and Parquet replay outputs. | Execution is scoped out; live Tardis Machine endpoint requirements and optional Python/PyO3 removal remain deferred. | JSON/CSV fixtures and replay/parser tests; no routine live endpoint dependency. |

## Removed Adapters

No workspace adapter crate is currently classified as `removed` for v0.2.0.
Unsupported surfaces are recorded as `deferred` or scoped out instead of being
silently omitted.

## Infrastructure Cache Adapter Classification

NTPRO also contains cache/database integration code outside the exchange
adapter crates above. These are not trading venues and must not be confused with
the adapter support rows.

| Surface | Path | Status | Supported scope | Unsupported or deferred scope | Evidence |
| --- | --- | --- | --- | --- | --- |
| PostgreSQL cache adapter | `crates/infrastructure/src/sql/cache.rs` | unsupported | Existing `nautilus database init/drop` commands remain database administration utilities. | Durable PostgreSQL cache persistence is not a v0.2 product path; many adapter operations explicitly return `not implemented`, and schema/FK integration tests remain ignored. | `docs/rust-cutover/product/POSTGRES_CACHE_ADAPTER_STATUS.md`, `docs/rust-cutover/evidence/NAUDIT-005.md` |

## Fixture And Sandbox Strategy

Routine adapter validation should use:

- checked-in HTTP/WebSocket/parser fixtures;
- compact fixture manifests where present;
- mock servers or schema validation for protocol behavior;
- `nautilus-sandbox` for local simulated execution;
- `nautilus-live` Cargo smoke paths for local node lifecycle;
- explicit dry-run or env-gated manual procedures for real endpoint workflows.

Routine validation must not require:

- real exchange API keys;
- wallet private keys;
- IB Gateway/TWS sessions;
- DeFi RPC credentials;
- live production order flow.

## Follow-Up Decisions

| Follow-up | Scope |
| --- | --- |
| `NADAPT-*` fixture manifest tasks | Add compact adapter-level manifests for supported, scoped, deferred, and removed surfaces. |
| `NTRACE-*` adapter trace tasks | Bind selected payload fixtures into golden trace or regression evidence where release gates require it. |
| `NPROD-*` adapter docs tasks | Promote Rust-first user docs only for the status and constraints listed in this matrix. |
| `NREM-*` removal tasks | Remove optional Python/PyO3 adapter surfaces only after explicit removal gates approve it. |
