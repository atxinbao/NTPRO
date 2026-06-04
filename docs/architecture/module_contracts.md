# NTPRO Module Contracts

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-002

## Purpose

This document records current module contracts for the NTPRO Rust-only
architecture. It distinguishes current behavior from future dashboard needs.

It does not refactor module code, add dashboard implementation, or change
public runtime behavior.

## Contract Rules

- Rust crates own runtime behavior.
- Rust CLI, Rust examples, and Rust docs are the public product path.
- Dashboard/control work must consume future stable contracts, not engine
  internals.
- Adapter validation must use fixtures, mocks, dry-runs, or sandbox evidence
  unless a later task explicitly approves live endpoint evidence.
- Python, PyO3, and Cython are not product architecture surfaces.

## Product Surface Contract

| Field | Contract |
| --- | --- |
| Modules | `crates/cli`, `examples/rust`, `docs/getting_started`, `docs/rust-cutover/product`. |
| Responsibilities | Present user-facing commands, examples, install/run docs, and workflow contracts. |
| Inputs | CLI args, config file paths, example TOML files, Cargo package selection. |
| Outputs | Help text, validation output, run artifacts, documentation, evidence files. |
| State | No long-lived trading state should be owned by product docs or CLI parsing alone. |
| Lifecycle | CLI command parse, validate, run or report explicit blocker, exit. |
| Error model | Invalid args/config exit non-zero with concise message; unsupported runtime path reports explicit blocker. |
| Dependency boundary | May call runtime crates through supported APIs; must not require Python/PyO3/Cython. |
| Candidate dashboard fields | Release version, supported command list, last command result, artifact path summary. |

## Node Runtime Contract

| Field | Backtest | Live/Sandbox |
| --- | --- | --- |
| Modules | `crates/backtest` | `crates/live`, `crates/adapters/sandbox` |
| Responsibilities | Historical simulation, backtest exchange, execution client, node, result metadata. | Node build/start/stop, sandbox adapter registration, live runtime lifecycle. |
| Inputs | Backtest config, historical data/catalog, instruments, strategy/runtime parameters. | Live/sandbox config, adapter config, environment, clock/runtime setup. |
| Outputs | Backtest result, run metadata, trace/evidence artifacts. | Lifecycle state, account/cache initialization, connection/disconnection status, smoke output. |
| State | Engine state, simulated exchange, data iterator, result aggregation. | Node state, manager state, adapter/client registration, runtime handles. |
| Lifecycle | Build, validate, initialize, run, stop/dispose, emit result. | Build, register adapters, start, run scoped cycle, stop, dispose. |
| Error model | Config/data/runtime errors return explicit failure; semantic differences require tests/golden trace evidence. | Config/adapter/start/stop errors return explicit failure; live endpoint failures remain scoped or env-gated. |
| Dependency boundary | Uses system/common/model/data/execution/risk/portfolio as runtime dependencies. | Uses system/common/model/data/execution/risk/portfolio and adapter crates. |
| Candidate dashboard fields | run id, environment, current phase, data range, result artifact, error summary. | node id, environment, lifecycle state, connected clients, last event time, stop reason. |

## System Kernel And Trader Contract

| Field | Contract |
| --- | --- |
| Modules | `crates/system` |
| Responsibilities | Compose kernel/trader, register components, own actor/strategy registration, orchestrate lifecycle. |
| Inputs | Trader id, instance id, environment, component configs, actor/strategy registrations. |
| Outputs | Started/stopped runtime, component registry effects, lifecycle events, error summaries. |
| State | Kernel state, trader state, component and actor registries. |
| Lifecycle | Build, register components, initialize, start, stop, dispose. |
| Error model | Composition errors should fail before run; runtime lifecycle errors should surface with component context. |
| Dependency boundary | May depend on runtime engines and common infrastructure; dashboard must not read registries directly. |
| Candidate dashboard fields | trader id, instance id, environment, lifecycle state, registered component counts, last lifecycle error. |

## DataEngine Contract

| Field | Contract |
| --- | --- |
| Modules | `crates/data` |
| Responsibilities | Process subscriptions/requests, route market data, aggregate data, connect data clients to cache/message bus. |
| Inputs | Subscribe/unsubscribe commands, data requests, adapter/client data events, catalog/source config. |
| Outputs | Cache updates, message-bus publications, data response events, validation errors. |
| State | Subscription state, client registry/pool, aggregation state, option-chain state. |
| Lifecycle | Initialize clients, process commands/events, publish/cache data, stop clients. |
| Error model | Invalid subscription/request fails with reason; adapter/client failures remain scoped to client status. |
| Dependency boundary | Owns data routing; should not mutate execution/risk/portfolio directly. |
| Candidate dashboard fields | connected data clients, subscribed instruments, last data event time, lag/error counters. |

## ExecutionEngine Contract

| Field | Contract |
| --- | --- |
| Modules | `crates/execution` |
| Responsibilities | Route orders, manage order lifecycle, coordinate matching/emulation/reconciliation, process execution reports and fills. |
| Inputs | Trading commands, risk-approved orders, venue execution reports, fill reports, matching engine events. |
| Outputs | Order events, fill events, position-related events, cache updates, message-bus publications. |
| State | Order manager state, execution client registry, reconciliation state, matching/emulation state. |
| Lifecycle | Register clients, accept commands, route/emulate/match, reconcile, stop clients. |
| Error model | Reject invalid or unsupported execution paths with explicit reason; do not silently alter order semantics. |
| Dependency boundary | Receives risk-approved flow; should not expose matching/order-manager internals to dashboard/control callers. |
| Candidate dashboard fields | order counts by state, connected execution clients, last report time, rejection/reconciliation summary. |

## RiskEngine Contract

| Field | Contract |
| --- | --- |
| Modules | `crates/risk` |
| Responsibilities | Validate order risk, enforce configured limits, produce accept/reject decisions. |
| Inputs | Order commands, account/portfolio/cache context, risk config, trading-state gates. |
| Outputs | Approved command forwarding, rejection events/reasons, risk status. |
| State | Risk config, bypass flags, counters/status needed for rejection diagnostics. |
| Lifecycle | Initialize config, evaluate commands, publish/return decisions, stop with runtime. |
| Error model | Invalid risk input rejects deterministically; risk differences need tests or golden trace evidence. |
| Dependency boundary | Reads portfolio/cache context through supported runtime paths; dashboard must not invoke checks directly. |
| Candidate dashboard fields | trading enabled, rejection counts by reason, configured limit summary, last rejection. |

## Portfolio Contract

| Field | Contract |
| --- | --- |
| Modules | `crates/portfolio` |
| Responsibilities | Track accounts, positions, exposure, realized/unrealized PnL, equity, snapshots. |
| Inputs | Account state, fills, position events, market data updates, currency rates. |
| Outputs | Portfolio state updates, snapshots, account/equity/PnL summaries. |
| State | Accounts, positions, exposure, PnL, snapshots, update timestamps. |
| Lifecycle | Initialize account state, process events, update snapshots, dispose with runtime. |
| Error model | Missing account/rate/state should surface explicitly; no silent accounting fallbacks. |
| Dependency boundary | Consumes execution/data/account events; dashboard should consume summaries, not `PortfolioState` internals. |
| Candidate dashboard fields | account ids, open position count, cash/equity summary, realized/unrealized PnL, last snapshot time. |

## MessageBus And Cache Contract

| Field | MessageBus | Cache |
| --- | --- | --- |
| Modules | `crates/common` | `crates/common` |
| Responsibilities | Route commands/events, pub/sub, typed endpoints, optional persistence config. | Store instruments, accounts, orders, positions, data, own books, and runtime views. |
| Inputs | Commands, events, requests, endpoint registrations. | Runtime updates from engines/adapters and read requests from runtime consumers. |
| Outputs | Delivered messages, handler invocations, optional persisted messages. | Read views, entity snapshots, update results. |
| State | Subscriptions, endpoints, routing tables, optional backing config. | Entity maps/indexes, cache config, database/reference state. |
| Lifecycle | Register, publish/request, unregister, dispose. | Initialize, mutate through runtime owners, read through views, dispose/snapshot. |
| Error model | Missing handler/endpoint or delivery failure must be visible to caller/evidence. | Missing entity returns explicit absence; mutation errors must not be hidden. |
| Dependency boundary | Dashboard must not depend on routing internals. | Dashboard must not use mutable cache internals as a public state API. |
| Candidate dashboard fields | message counts, endpoint health, last event time. | entity counts, last update time, snapshot availability. |

## Persistence And Event Store Contract

| Field | Contract |
| --- | --- |
| Modules | `crates/persistence`, `crates/event_store`, `crates/serialization` |
| Responsibilities | Store/replay runtime artifacts, encode/decode persisted data, support event-store boundaries. |
| Inputs | Runtime events, cache/message-bus artifacts, serialization config, database paths/connections. |
| Outputs | Persisted records, replayable artifacts, decode/encode results. |
| State | Database handles, artifact metadata, event-store indexes, serialization schema/config. |
| Lifecycle | Open/configure, write/read, flush, close/dispose. |
| Error model | I/O, schema, decode, and consistency failures must surface explicitly. |
| Dependency boundary | Runtime modules may persist through stable interfaces; dashboard/history must wait for artifact contracts. |
| Candidate dashboard fields | persistence enabled, backend kind, last write/read time, artifact count, last error summary. |

## Adapter Layer Contract

| Field | Contract |
| --- | --- |
| Modules | `crates/adapters/*` |
| Responsibilities | Translate venue/provider protocols into Rust model/data/execution events and commands. |
| Inputs | Adapter config, credentials through approved channels, HTTP/WebSocket/RPC payloads, fixture payloads. |
| Outputs | Model data, execution reports, account reports, client status, parser errors. |
| State | Client connection state, subscriptions, account/session state, venue-specific caches. |
| Lifecycle | Configure, connect/start, subscribe/request, parse/emit, stop/disconnect. |
| Error model | Protocol, auth, parse, unsupported order/data, and connection errors must be explicit and fixture-testable. |
| Dependency boundary | Adapter internals must not become product support claims without matrix/fixture evidence. |
| Candidate dashboard fields | adapter id, classification, connection status, last heartbeat, last error, credential mode without secret values. |

## Verification Contract

| Field | Contract |
| --- | --- |
| Modules | `scripts/ai`, `crates/testkit`, `tests/golden`, `docs/rust-cutover/evidence` |
| Responsibilities | Validate local changes, release gates, Rust-only surface, golden trace schema/replay, task evidence. |
| Inputs | Workspace source, golden trace JSONL, task metadata, release manifests, command flags. |
| Outputs | Pass/fail status, evidence docs, release verification artifacts, task closeout state. |
| State | Local control/evidence state, Shrimp queue state, release manifests. |
| Lifecycle | Run command, collect output, update evidence, close/dispatch tasks through control scripts. |
| Error model | Fail fast on command failure; blockers must be recorded with attempted commands and next action. |
| Dependency boundary | Verification state is repository control state, not runtime product state. |
| Candidate dashboard fields | Not product dashboard by default; future project dashboard may show last verification summary separately. |

## Current vs Future Dashboard Boundary

Current behavior:

- Runtime modules expose their Rust APIs and internal structs for engine use.
- Product users interact through CLI/docs/examples/Cargo.
- Verification evidence lives in docs and local scripts.

Future dashboard needs:

- read-only node status;
- read-only engine summaries;
- read-only adapter status;
- explicit control action contract;
- lifecycle state machine;
- no direct access to mutable engine/cache/message-bus internals.

The future dashboard boundary is contract work first. Implementation must wait
for NARCH-003, NARCH-004, NARCH-005, and NDASH-001.

## Unresolved Gaps

| Gap | Owner task |
| --- | --- |
| Stable node lifecycle states and transitions are not yet defined. | `NARCH-003` |
| Dashboard-readable observability state model is not yet defined. | `NARCH-004` |
| Control actions are not yet defined as allowed, scoped, or forbidden. | `NARCH-005` |
| Dashboard MVP scope is not locked. | `NDASH-001` |
| Persistence artifact contract remains broad. | Later persistence/trace task |
| Adapter fixture manifests are not complete for every supported adapter. | Later `NADAPT-*` |
