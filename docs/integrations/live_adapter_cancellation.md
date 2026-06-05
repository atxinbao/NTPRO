# Live Adapter Cancellation Contract

Date: 2026-06-05
Executor: Codex

This document defines the NTPRO Rust-only cancellation contract for live data
adapter startup.

## Scope

The contract applies to implementations of `DataClient::connect` that are
driven by `LiveNode` during startup. It covers stop requests, shutdown requests,
and process interrupts that arrive before a data adapter finishes connecting.

It does not change order routing, subscription semantics, matching behavior, or
real exchange protocol handling.

## Contract

When `LiveNode` starts, data clients connect before execution clients so that
instrument events can populate the cache. During this phase, NTPRO may drop a
pending `DataClient::connect` future if any startup cancellation control fires.

Every live data adapter must therefore treat a dropped connect future as a
failed connection attempt:

- temporary sockets, authentication handshakes, subscriptions, spawned tasks,
  and buffered startup state must be released or moved under explicit cleanup
  ownership;
- the client must not report `is_connected() == true` unless the connect future
  completed successfully;
- a canceled attempt must not leave a half-connected state that changes later
  retry behavior without being visible to the caller;
- retry behavior after cancellation must either start from a disconnected state
  or document the cleanup owner responsible for finishing teardown;
- real adapter proof must use mocks, fixtures, sandbox endpoints, or recorded
  test harnesses, never live private exchange credentials.

## Current Evidence

`NAUDIT-006` adds a live-node startup test with a mock data client connect
future. The future acquires simulated resources and then remains pending. When
startup cancellation is triggered, the live-node driver returns the cancellation
status, drops the pending connect future, and the mock guard releases resources.

This proves the live-node cancellation boundary and the adapter-side cleanup
expectation. It does not prove every real adapter implementation yet.

`DRG-007` re-ran the executable mock evidence on the current v0.2 readiness
branch and closed `G6 Live cancellation proof` for the live-node startup
boundary. The closure remains scoped to mock proof plus the adapter contract
below; it does not claim any real exchange adapter has adapter-specific
cancellation-safety evidence yet.

## Real Adapter Follow-Up Register

The following real adapter data clients still need adapter-specific
cancellation-safety evidence before they can be claimed as fully proven under
this contract:

| Adapter family | Representative data clients | Current cancellation proof |
| --- | --- | --- |
| Architect AX | `AxDataClient` | Pending adapter-specific mock or fixture proof |
| Binance | spot and futures data clients | Pending adapter-specific mock or fixture proof |
| Betfair | data client | Pending adapter-specific mock or fixture proof |
| BitMEX | data client | Pending adapter-specific mock or fixture proof |
| Blockchain / DeFi | blockchain data client | Pending adapter-specific mock or fixture proof |
| Bybit | data client | Pending adapter-specific mock or fixture proof |
| Coinbase | data client | Pending adapter-specific mock or fixture proof |
| Databento | data client | Pending adapter-specific mock or fixture proof |
| Deribit | data client | Pending adapter-specific mock or fixture proof |
| dYdX | data client | Pending adapter-specific mock or fixture proof |
| Hyperliquid | data client | Pending adapter-specific mock or fixture proof |
| Interactive Brokers | data client | Pending adapter-specific mock or fixture proof |
| Kraken | spot and futures data clients | Pending adapter-specific mock or fixture proof |
| OKX | data client | Pending adapter-specific mock or fixture proof |
| Polymarket | data client | Pending adapter-specific mock or fixture proof |
| Tardis | data client | Pending adapter-specific mock or fixture proof |

## Migration Note

Adapter maintainers should audit `connect` implementations for resources
created before the final connected state is reached. If cancellation can drop
the future at any await point, cleanup must happen through RAII guards, owned
connection handles with `Drop`, explicit abort handles, or a documented teardown
path.

Do not mark an adapter as cancellation-safe for NTPRO release evidence until a
fixture, mock, or sandbox smoke test covers a canceled pending connect attempt.
